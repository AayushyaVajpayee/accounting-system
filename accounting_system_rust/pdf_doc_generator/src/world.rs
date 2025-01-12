use std::sync::{RwLock, RwLockWriteGuard};
use std::collections::HashMap;
use std::path::PathBuf;

use time::OffsetDateTime;
use typst::foundations::{Bytes, Datetime};
use typst::text::{Font, FontBook};
use typst::{Library, World};
use typst::syntax::{FileId, Source};
use typst::diag::{FileError, FileResult, PackageError, PackageResult};
use typst::ecow::EcoString;
use typst::syntax::package::PackageSpec;
use typst::utils::LazyHash;
use crate::fonts::register_fonts;

#[derive(Debug)]
pub struct FileEntry {
    bytes: Bytes,
    source: Option<Source>,
}

impl FileEntry {
    pub fn from_bytes(bytes: Bytes, source: Option<Source>) -> Self {
        Self { bytes, source }
    }

    pub fn new(bytes: Vec<u8>, source: Option<Source>) -> Self {
        Self {
            bytes: Bytes::new(bytes),
            source,
        }
    }

    pub fn source(&mut self, id: FileId) -> FileResult<Source> {
        let source = if let Some(source) = &self.source {
            source
        } else {
            let contents = std::str::from_utf8(&self.bytes).map_err(|_| FileError::InvalidUtf8)?;
            let contents = contents.trim_start_matches('\u{feff}');
            let source = Source::new(id, contents.into());
            self.source.insert(source)
        };
        Ok(source.clone())
    }
}

#[derive(Debug)]
pub struct InMemoryWorld {
    root: PathBuf,
    main_id: FileId,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    files: RwLock<HashMap<FileId, FileEntry>>, // Use RwLock
    package_cache_dir: PathBuf,
    time: OffsetDateTime,
    http: ureq::Agent,
    file_map: HashMap<&'static str, Bytes>,
}

impl InMemoryWorld {
    pub fn new(content: &str, file_map: HashMap<&'static str, Bytes>) -> Self {
        let fonts = register_fonts();
        let main_id = FileId::new(None, typst::syntax::VirtualPath::new("main.typ"));

        let mut files = HashMap::new();
        files.insert(
            main_id,
            FileEntry::new(content.as_bytes().to_vec(), None),
        );

        Self {
            root: PathBuf::from(""),
            main_id,
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(FontBook::from_fonts(&fonts)),
            fonts,
            files: RwLock::new(files),
            package_cache_dir: std::env::var_os("CACHE_DIRECTORY")
                .map(|os_path| os_path.into())
                .unwrap_or(std::env::temp_dir()),
            time: OffsetDateTime::now_utc(),
            http: ureq::Agent::new(),
            file_map,
        }
    }

    fn download_package(&self, spec: &PackageSpec) -> PackageResult<PathBuf> {
        let package_subdir = format!("{}/{}/{}", spec.namespace, spec.name, spec.version);
        let path = self.package_cache_dir.join(package_subdir);

        if let Some(path_str) = path.to_str() {
            if self.file_map.contains_key(path_str) {
                return Ok(path);
            }
        }

        if path.exists() {
            return Ok(path);
        }

        eprintln!("downloading {spec}");
        let url = format!(
            "https://packages.typst.org/{}/{}-{}.tar.gz",
            spec.namespace, spec.name, spec.version,
        );

        let response = retry(|| {
            let response = self
                .http
                .get(&url)
                .call()
                .map_err(|error| format!("{error}"))?;

            let status = response.status();
            if !http_successful(status) {
                return Err(format!(
                    "response returned unsuccessful status code {status}",
                ));
            }

            Ok(response)
        })
            .map_err(|error| PackageError::NetworkFailed(Some(EcoString::from(error))))?;

        let mut compressed_archive = Vec::new();
        response
            .into_reader()
            .read_to_end(&mut compressed_archive)
            .map_err(|error| PackageError::NetworkFailed(Some(format!("{error}").into())))?;

        let raw_archive = zune_inflate::DeflateDecoder::new(&compressed_archive)
            .decode_gzip()
            .map_err(|error| PackageError::MalformedArchive(Some(format!("{error}").into())))?;

        let mut archive = tar::Archive::new(raw_archive.as_slice());
        archive.unpack(&path).map_err(|error| {
            _ = std::fs::remove_dir_all(&path);
            PackageError::MalformedArchive(Some(format!("{error}").into()))
        })?;

        Ok(path)
    }

    fn get_file(&self, id: FileId) -> FileResult<RwLockWriteGuard<'_, HashMap<FileId, FileEntry>>> {
        let mut files = self.files.write().unwrap();
        if let std::collections::hash_map::Entry::Vacant(e) = files.entry(id) {
            let path = if let Some(package) = id.package() {
                let package_dir = self.download_package(package)?;
                id.vpath().resolve(&package_dir)
            } else {
                id.vpath().resolve(&self.root)
            }
                .ok_or(FileError::AccessDenied)?;

            let content = if let Some(a) = self.file_map.get(path.to_str().unwrap()) {
                a.clone()
            } else {
                Bytes::new( std::fs::read(&path).map_err(|error| FileError::from_io(error, &path))?)
            };

            e.insert(FileEntry::new(content.to_vec(), None));
        }
        Ok(files)
    }
}

impl World for InMemoryWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main_id
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        let mut files = self.get_file(id)?;
        files.get_mut(&id).unwrap().source(id)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let files = self.get_file(id)?;
        Ok(files[&id].bytes.clone())
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let offset = offset.unwrap_or(0);
        let offset = time::UtcOffset::from_hms(offset.try_into().ok()?, 0, 0).ok()?;
        let time = self.time.checked_to_offset(offset)?;
        Some(Datetime::Date(time.date()))
    }
}

fn retry<T, E>(mut f: impl FnMut() -> Result<T, E>) -> Result<T, E> {
    if let Ok(ok) = f() {
        Ok(ok)
    } else {
        f()
    }
}

fn http_successful(status: u16) -> bool {
    status / 100 == 2
}
