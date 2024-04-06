use anyhow::ensure;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(try_from = "i32")]
pub struct LineNumber(u16);

impl LineNumber {
    pub fn new(line_number: i32) -> anyhow::Result<Self> {
        ensure!(
            line_number > 0,
            "line number ({}) should be greater than 0 ",
            line_number
        );
        ensure!(
            line_number <= 2000,
            "line number ({}) should be less than 2000 ",
            line_number
        );
        Ok(Self(line_number as u16))
    }
}

impl TryFrom<i32> for LineNumber {
    type Error = anyhow::Error;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        LineNumber::new(value)
    }
}

#[cfg(test)]
mod line_number_tests {
    use rstest::rstest;
    use speculoos::assert_that;
    use speculoos::prelude::ResultAssertions;

    use crate::invoice_line::line_number::LineNumber;

    #[rstest]
    #[case(0, false)]
    #[case(1, true)]
    fn test_line_number(#[case] input: i32, #[case] valid: bool) {
        let line_no = LineNumber::new(input);
        if valid {
            assert_that!(line_no).is_ok();
        } else {
            assert_that!(line_no).is_err();
        }
    }
}
