use lazy_static::lazy_static;
use regex::Regex;
use std::ops::Not;
use thiserror::Error;
use crate::invoice_number::InvoiceNumberError::InvalidPattern;

#[derive(Debug)]
pub struct InvoiceNumber(String);
#[derive(Debug, Error)]
pub enum InvoiceNumberError {
    #[error("invoice number cannot be empty")]
    Empty,
    #[error("invoice number cannot be more than 16 chars")]
    TooLong,
    #[error("invoice number {0} is not valid invoice number. It should only contain 16 characters of\
     alphabets,numbers and special chars like / and -")]
    InvalidPattern(String)
}
//https://old.cbic.gov.in/htdocs-cbec/gst/Tax_Invoice_and_other_new.pdf
lazy_static! {
    static ref INVOICE_NUMBER_REGEX:Regex = Regex::new(r"[a-zA-Z]|[\/]|[-]|[\d]").unwrap();//todo test
}
impl InvoiceNumber {
    pub fn new(invoice_number: String) -> Result<Self, InvoiceNumberError> {
        if invoice_number.is_empty() {
            return Err(InvoiceNumberError::Empty);
        }

        if invoice_number.len() > 16 {
            return Err(InvoiceNumberError::TooLong);
        }
        if INVOICE_NUMBER_REGEX.is_match(invoice_number.as_str()).not() {
            return Err(InvalidPattern(invoice_number));
        }

        Ok(Self(invoice_number))
    }
}
