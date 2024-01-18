use crate::common_utils::mime_types::MimeType::{Csv, Docx, Jpeg, Json, Pdf, Png, Txt, Xlsx};
use anyhow::anyhow;
use std::error::Error;
use tokio_postgres::types::{FromSql, Type};

#[derive(Debug)]
pub enum MimeType {
    Csv,
    Docx,
    Jpeg,
    Json,
    Png,
    Pdf,
    Txt,
    Xlsx,
}

impl MimeType {
    pub fn get_mime_type(&self) -> &'static str {
        match &self {
            Csv => "text/csv",
            Docx => {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            }
            Jpeg => "image/jpeg",
            Json => "application/json",
            Png => "image/png",
            Pdf => "application/pdf",
            Txt => "text/plain",
            Xlsx => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        }
    }
    pub fn from_text(mime_type: &str) -> anyhow::Result<Self> {
        match mime_type {
            "csv" => Ok(Csv),
            "docx" => Ok(Docx),
            "jpeg" => Ok(Jpeg),
            "json" => Ok(Json),
            "png" => Ok(Png),
            "pdf" => Ok(Pdf),
            "txt" => Ok(Txt),
            "xlsx" => Ok(Xlsx),
            _ => Err(anyhow!(
                "not supported by system, extension {} for mime type conversion ",
                mime_type
            )),
        }
    }
}

impl<'a> FromSql<'a> for MimeType {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        let k = String::from_utf8_lossy(raw);
        match MimeType::from_text(k.as_ref()){
            Ok(a) => {Ok(a)}
            Err(b) => {Err(b.into())}
        }
    }

    fn accepts(ty: &Type) -> bool {
         ty.name()=="anyenum"
    }
}
