use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum GstinValidationError<'a> {
    #[error("gstin no {0} should be 15 characters but was {1}")]
    Not15Digit(&'a str, usize),
    #[error("gstin no {0} is not a valid")]
    InvalidPattern(&'a str),
    #[error("gstin no {0} is not valid, check any typing error")]
    TypingError(&'a str),
}
lazy_static! {
    static ref REGEX: Regex =
        Regex::new("\\d{2}[a-zA-Z]{5}\\d{4}[a-zA-Z]{1}[a-zA-Z\\d]{1}[zZ]{1}[a-zA-Z\\d]{1}")
            .unwrap();
}
static CONVERSION_TABLE: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

lazy_static! {
    static ref ALPHABET_TO_INT_MAP: HashMap<char, usize> = CONVERSION_TABLE
        .chars()
        .enumerate()
        .map(|(a, i)| (i, a))
        .collect::<HashMap<char, usize>>();
}
lazy_static! {
    static ref INT_TO_ALPHABET_MAP: HashMap<usize, char> = CONVERSION_TABLE
        .chars()
        .enumerate()
        .map(|(a, i)| (a, i))
        .collect::<HashMap<usize, char>>();
}

fn validate_gstin_size(gstin: &str) -> Option<GstinValidationError> {
    let length = gstin.chars().count();
    if length != 15 {
        let err = GstinValidationError::Not15Digit(gstin, length);
        return Some(err);
    }
    None
}

fn validate_gstin_pattern(gstin: &str) -> Option<GstinValidationError> {
    let valid = REGEX.is_match(gstin);
    if valid {
        None
    } else {
        Some(GstinValidationError::InvalidPattern(gstin))
    }
}

fn gstin_checksum(gstin: &str) -> Result<char, &str> {
    let gstin = gstin.to_uppercase();
    let checked_digit = gstin.chars().nth(14);
    if checked_digit.is_none() {
        return Err("less than 14 chars in gstin. cannot calculate checksum");
    }
    let candidate = gstin.chars().take(14);
    let mut multiply_by_2 = false;
    let mut hash_sum = 0;
    for char in candidate {
        if multiply_by_2 {
            let value = ALPHABET_TO_INT_MAP.get(&char).unwrap();
            let product = value * 2;
            let quotient = product / 36;
            let remainder = product % 36;
            hash_sum = hash_sum + quotient + remainder;
            multiply_by_2 = false;
        } else {
            let value = ALPHABET_TO_INT_MAP.get(&char).unwrap();
            let product = value * 1;
            let quotient = product / 36;
            let remainder = product % 36;
            hash_sum = hash_sum + quotient + remainder;
            multiply_by_2 = true;
        }
    }
    let hash_sum_remainder = hash_sum % 36;
    let check_digit = 36 - hash_sum_remainder;
    let check_alpha = INT_TO_ALPHABET_MAP.get(&check_digit).unwrap();
    Ok(*check_alpha)
}

#[cfg(test)]
mod test {
    use crate::gstin_models::{gstin_checksum, validate_gstin_pattern, validate_gstin_size};
    use rstest::rstest;

    #[rstest]
    #[case("", false)]
    #[case("gjlljlj", false)]
    #[case("absciedneiencie", true)]
    fn test_validate_gstin_size(#[case] gstin: String, #[case] valid: bool) {
        let val = validate_gstin_size(gstin.as_str());
        if valid {
            assert!(val.is_none())
        } else {
            assert!(val.is_some())
        }
    }

    #[rstest]
    #[trace]
    #[case("", false)]
    #[case("akljsfljda", false)]
    #[case("123456789123456", false)]
    #[case("07PCZPK9220B1ZG", true)]
    #[case("07PCZPK9220B1zG", true)]
    #[case("07PCZPk9220B1ZG", true)]
    #[case("07pCZPk9220B1ZG", true)]
    #[case("079CZPk9220B1ZG", false)]
    fn test_validate_gstin_pattern(#[case] gstin: String, #[case] valid: bool) {
        let v = validate_gstin_pattern(gstin.as_ref());
        if valid {
            assert!(v.is_none())
        } else {
            assert!(v.is_some())
        }
    }
    #[rstest]
    #[trace]
    #[case("",'A',false)]
    #[case("27AAPFU0939F1ZV",'V',true)]
    #[case("07PCZPK9220B1ZG", '1', true)]
    #[case("05AABCA5291p1ZD", 'D', true)]
    #[case("18AABCU9603R1zM", 'M', true)]
    #[case("16AaBCU9603R1Zq", 'Q', true)]
    fn test_compute_gstin_checksum(
        #[case] gstin: String,
        #[case] checksum_char: char,
        #[case] valid: bool,
    ) {
        let v = gstin_checksum(gstin.as_ref());
        if valid {
            assert!(v.is_ok());
            assert_eq!(v.unwrap(), checksum_char);
        } else {
            assert!(v.is_err());
        }
    }
}
