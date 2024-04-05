use anyhow::Context;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::invoice_line::line_title::LineTitleError::{
    EmptyTitle, NoReadableChars, TooLong, TooShort,
};

lazy_static! {
    static ref NO_ALPHABET_REGEX: Regex = Regex::new(r"^(?:[^a-z^A-Z]+)$").unwrap();
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(try_from = "String")]
pub struct LineTitle(String);

#[derive(Debug, Error)]
pub enum LineTitleError {
    #[error("line title cannot be empty")]
    EmptyTitle,
    #[error("line title should not be more than {0} char")]
    TooLong(u16),
    #[error("line title does not have any alphabets")]
    NoReadableChars,
    #[error("linet title cannot be less than {0} characters")]
    TooShort(u16),
}

impl LineTitle {
    pub fn new(title: String) -> Result<Self, LineTitleError> {
        let title = title.trim();
        let count = title.chars().count();
        if count >= 80 {
            return Err(TooLong(80));
        }
        if count <= 3 {
            return Err(TooShort(3));
        }

        if NO_ALPHABET_REGEX.is_match(title) {
            return Err(NoReadableChars);
        }
        if title.is_empty() {
            return Err(EmptyTitle);
        }

        Ok(Self(title.to_string()))
    }
    pub fn inner(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<String> for LineTitle {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        LineTitle::new(value).context("")
    }
}

#[cfg(test)]
mod line_title_tests {
    use rstest::rstest;
    use speculoos::assert_that;
    use speculoos::prelude::ResultAssertions;

    use crate::invoice_line::line_title::LineTitle;

    #[rstest]
    #[case("", false)]
    #[case(
        "fdjlajfljldjfaldjfladjflakdjlfkajlfjlajflakjlkasdjlfkdjalkjafldjaljfaljfdlajfdlakjfdal",
        false
    )]
    #[case("\n\n\n\n\n\n", false)]
    #[case("                ", false)]
    #[case("--------------------------", false)]
    #[case("------%%%%%%%%%%%%%%%%%%%%%%%", false)]
    #[case("343242534243", false)]
    #[case("``````", false)]
    #[case("l", false)]
    #[case("kj", false)]
    #[case("   l     ", false)]
    // #[case("%%kj%%", false)]
    #[case("iphone 15 pro max", true)]
    fn test_line_title(#[case] input: String, #[case] valid: bool) {
        let line_no = LineTitle::new(input);
        if valid {
            assert_that!(line_no).is_ok();
        } else {
            assert_that!(line_no).is_err();
        }
    }
}
