use std::cell::{RefCell, RefMut};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use comemo::Prehashed;
use time::macros::time;
use time::OffsetDateTime;
use typst::{Library, World};
use typst::diag::{eco_format, FileError, FileResult, PackageError, PackageResult};
use typst::foundations::{Bytes, Datetime, panic};
use typst::syntax::{FileId, Source};
use typst::syntax::package::PackageSpec;
use typst::text::{Font, FontBook};

use crate::fonts::register_fonts;
#[derive(Debug)]
pub struct FileEntry {
    bytes: Bytes,
    source: Option<Source>,
}

impl FileEntry {
    pub fn from_bytes(bytes: Bytes, source: Option<Source>) -> Self {
        Self {
            bytes,
            source,
        }
    }
    pub fn new(bytes: Vec<u8>, source: Option<Source>) -> Self {
        //todo we need to provide another constructor that will provide all related files as bytes in a hashmap
        Self {
            bytes: bytes.into(),//todo for our use case we can take static bytes for many things except the json
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
    /// The content of a source.
    source: Source,
    /// The standard library.
    library: Prehashed<Library>,
    ///Metadata about all known fonts.
    book: Prehashed<FontBook>,
    ///all known fonts.
    fonts: Vec<Font>,
    /// Map of all known files.
    files: RefCell<HashMap<FileId, FileEntry>>,
    /// Cache directory (e.g. where packages are downloaded to).
    package_cache_dir: PathBuf,
    /// Datetime.
    time: OffsetDateTime,
    /// http agent to download packages.
    http: ureq::Agent,
    file_map: HashMap<&'static str, Bytes>,
}

impl InMemoryWorld {
    pub fn new(content: &str, file_map: HashMap<&'static str, Bytes>) -> Self {
        let fonts = register_fonts();
        Self {
            root: PathBuf::from(""),
            source: Source::detached(content),
            library: Prehashed::new(Library::builder().build()),
            book: Prehashed::new(FontBook::from_fonts(&fonts)),
            fonts,
            files: RefCell::new(HashMap::new()),
            package_cache_dir: std::env::var_os("CACHE_DIRECTORY")
                .map(|os_path| os_path.into())
                .unwrap_or(std::env::temp_dir()),
            time: OffsetDateTime::now_utc(),
            http: ureq::Agent::new(),
            file_map,
        }
    }
    fn download_package(&self, package: &PackageSpec) -> PackageResult<PathBuf> {
        let package_subdir = format!("{}/{}/{}", package.namespace, package.name, package.version);
        let path = self.package_cache_dir.join(package_subdir);
        if let Some(path_str) = path.to_str() {
            if self.file_map.contains_key(path_str) {
                return Ok(path);
            }
        }
        if path.exists() {
            return Ok(path);
        }
        eprintln!("downloading {package}");
        let url = format!(
            "https://packages.typst.org/{}/{}-{}.tar.gz",
            package.namespace, package.name, package.version,
        );

        let response = retry(|| {
            let response = self
                .http
                .get(&url)
                .call()
                .map_err(|error| eco_format!("{error}"))?;

            let status = response.status();
            if !http_successful(status) {
                return Err(eco_format!(
                    "response returned unsuccessful status code {status}",
                ));
            }

            Ok(response)
        })
            .map_err(|error| PackageError::NetworkFailed(Some(error)))?;

        let mut compressed_archive = Vec::new();
        response
            .into_reader()
            .read_to_end(&mut compressed_archive)
            .map_err(|error| PackageError::NetworkFailed(Some(eco_format!("{error}"))))?;
        let raw_archive = zune_inflate::DeflateDecoder::new(&compressed_archive)
            .decode_gzip()
            .map_err(|error| PackageError::MalformedArchive(Some(eco_format!("{error}"))))?;
        let mut archive = tar::Archive::new(raw_archive.as_slice());
        archive.unpack(&path).map_err(|error| {
            _ = std::fs::remove_dir_all(&path);
            PackageError::MalformedArchive(Some(eco_format!("{error}")))
        })?;
        Ok(path)
    }
    fn file(&self, id: FileId) -> FileResult<RefMut<'_, FileEntry>> {
        if let Ok(entry) = RefMut::filter_map(self.files.borrow_mut(), |files| files.get_mut(&id)) {
            return Ok(entry);
        }
        let path = if let Some(package) = id.package() {
            let package_dir = self.download_package(package)?;
            id.vpath().resolve(&package_dir)
        } else {
            id.vpath().resolve(&self.root)
        }
            .ok_or(FileError::AccessDenied)?;
        if let Some(a) = self.file_map.get(path.to_str().unwrap()) {
            let p = FileEntry::from_bytes(a.clone(), None);
            return Ok(RefMut::map(self.files.borrow_mut(),
                                  |files| files.entry(id).or_insert(p)));
        }
        let content = std::fs::read(&path).map_err(|error| FileError::from_io(error, &path))?;
        Ok(RefMut::map(self.files.borrow_mut(), |files| {
            files.entry(id).or_insert(FileEntry::new(content, None))
        }))
    }
}

impl World for InMemoryWorld {
    fn library(&self) -> &Prehashed<Library> {
        &self.library
    }

    fn book(&self) -> &Prehashed<FontBook> {
        &self.book
    }

    fn main(&self) -> Source {
        self.source.clone()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            self.file(id)?.source(id)
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.file(id).map(|file| file.bytes.clone())
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
    // 2XX
    status / 100 == 2
}