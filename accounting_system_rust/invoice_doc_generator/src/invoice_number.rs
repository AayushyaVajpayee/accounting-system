use anyhow::ensure;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::ops::Not;

#[derive(Debug, Serialize, Deserialize,Clone)]
#[serde(try_from = "String")]
pub struct InvoiceNumber(String);

//https://old.cbic.gov.in/htdocs-cbec/gst/Tax_Invoice_and_other_new.pdf
lazy_static! {
    static ref INVOICE_NUMBER_REGEX:Regex = Regex::new("^([[:alnum:]]|[/-]){1,16}$").unwrap();//todo test
}
lazy_static! {
    static ref INVOICE_NUMBER_ONLY_SPECIAL_CHAR_REGEX:Regex = Regex::new("^[/-]{1,16}$").unwrap();//todo test
}
impl InvoiceNumber {
    pub fn new(invoice_number: String) -> anyhow::Result<Self> {
        ensure!(!invoice_number.is_empty(),"invoice number cannot be empty");
        ensure!(invoice_number.len()<= 16,"invoice number cannot be more than 16 chars");
        let valid_invoice_num = INVOICE_NUMBER_ONLY_SPECIAL_CHAR_REGEX.is_match(invoice_number.as_str()).not()
            && INVOICE_NUMBER_REGEX.is_match(invoice_number.as_str());
        ensure!(valid_invoice_num,"invoice number {} is not valid invoice number. It should only contain 16 characters of \
            alphabets,numbers and special chars like / and -. Also cannot contain only special chars",invoice_number);
        Ok(Self(invoice_number))
    }
}

impl TryFrom<String> for InvoiceNumber {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        InvoiceNumber::new(value)
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;

    use crate::invoice_number::InvoiceNumber;

    #[rstest]
    #[trace]
    #[case::blank("", false)]
    #[case::single_char("a", true)]//because less than equal  to 16 char
    #[case::too_long("abdafndljaldkajflajdlkajlkfjlakjlfkjadlfjalk", false)]
    #[case::too_long_alphanumeric("934u932840840230840328093840328408320", false)]
    #[case::only_valid_symbols("/////////////---", false)]//should have at least one number or alphabet
    #[case::valid_only_alphabet("abscdkkfdkajkjkf", true)]
    #[case::valid_only_num("1234567891234567", true)]
    #[case::valid_general("ab/jljklkj/12345", true)]
    #[case::invalid_special_char("123456789*&@7222", false)]
    fn test_invoice_number(#[case] invoice_number: String, #[case] valid: bool) {
        let in_num = InvoiceNumber::new(invoice_number);
        if valid {
            assert_that!(in_num).is_ok();
        } else {
            assert_that!(in_num).is_err();
        }
    }
}
