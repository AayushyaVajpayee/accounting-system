use thiserror::Error;
use crate::invoice_line::line_number::LineNumberError::ShouldBeGreaterThan0;

#[derive(Debug)]
pub struct LineNumber(u16);

#[derive(Debug, Error)]
pub enum LineNumberError {
    #[error("line number {0} should start from 1")]
    ShouldBeGreaterThan0(u16),
}
impl LineNumber {
    pub fn new(line_number: u16) -> Result<Self, LineNumberError> {
        if line_number == 0 {
            return Err(ShouldBeGreaterThan0(line_number));
        }
        Ok(Self(line_number))
    }
}

#[cfg(test)]
mod line_number_tests {
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;
    use crate::invoice_line::line_number::LineNumber;

    #[rstest]
    #[case(0, false)]
    #[case(1, true)]
    fn test_line_number(#[case] input: u16, #[case] valid: bool) {
        let line_no = LineNumber::new(input);
        if valid {
            assert_that!(line_no).is_ok();
        } else {
            assert_that!(line_no).is_err();
        }
    }
}