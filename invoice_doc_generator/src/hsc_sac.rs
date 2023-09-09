use std::ops::Not;

use thiserror::Error;

use crate::hsn_code_generated::HSN_SET;
use crate::sac_code_generated::SAC_SET;

#[derive(Debug)]
pub enum GstItemCode {
    HSN(Hsn),
    SAC(Sac),
}
#[derive(Debug)]
pub struct Hsn(String);

#[derive(Debug, Error)]
pub enum HsnError {
    #[error("hsn length {0} is invalid. it should be of minimum 4 digits and maximum 8")]
    LengthInvalid(usize),
    #[error("hsn should contain only numbers")]
    NotNumeric,
    #[error("hsn not found in gst data. Please enter a valid hsn code")]
    NotFoundInMasterData,
}
impl Hsn {
    pub fn new(hsn: String) -> Result<Self, HsnError> {
        if hsn.len() < 4 || hsn.len() > 8 {
            return Err(HsnError::LengthInvalid(hsn.len()));
        }
        let parsed_hsn = hsn.parse::<u32>();
        if parsed_hsn.is_err() {
            return Err(HsnError::NotNumeric);
        }
        if HSN_SET.contains(&parsed_hsn.unwrap()).not() {
            return Err(HsnError::NotFoundInMasterData);
        }
        Ok(Self(hsn))
    }
}
#[derive(Debug)]
pub struct Sac(String);
#[derive(Debug, Error)]
pub enum SacError {
    #[error("Sac length {0} is invalid. it should be of minimum 4 digits and maximum 6")]
    LengthInvalid(usize),
    #[error("Sac should contain only numbers")]
    NotNumeric,
    #[error("Sac not found in mater sdata. Please enter a valid sac code")]
    NotFoundInMasterData,
}

impl Sac {
    pub fn new(sac: String) -> Result<Self, SacError> {
        if sac.len() < 4 || sac.len() > 6 {
            return Err(SacError::LengthInvalid(sac.len()));
        }
        let parsed_sac = sac.parse::<u32>();
        if parsed_sac.is_err() {
            return Err(SacError::NotNumeric);
        }
        if !SAC_SET.contains(&parsed_sac.unwrap()) {
            return Err(SacError::NotFoundInMasterData);
        }
        Ok(Self(sac))
    }
}

#[cfg(test)]
mod hsn_tests {
    use crate::hsc_sac::Hsn;
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;

    #[rstest]
    #[case("", false)]
    #[case("123432432", false)]
    #[case("99", false)]
    #[case("9980k", false)]
    #[case("9801",true)]
    #[case("91081100",true)]
    #[case("910811000",false)]
    #[case("0101",true)]
    pub fn test_hsn(#[case] input: String, #[case] valid: bool) {
        let hsn = Hsn::new(input);
        if valid {
            assert_that!(hsn).is_ok();
        } else {
            assert_that!(hsn).is_err();
        }
    }
}
#[cfg(test)]
mod sac_tests{
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;
    use crate::hsc_sac::Sac;

    #[rstest]
    #[trace]
    #[case("", false)]
    #[case("123432432", false)]
    #[case("99", false)]
    #[case("9954",true)]
    #[case("9997",true)]
    #[case("9954av",false)]
    #[case("999799",true)]
    #[case("99979900",false)]
    #[case("0101",false)]
    pub fn test_sac(#[case] input: String, #[case] valid: bool) {
        let hsn = Sac::new(input);
        if valid {
            assert_that!(hsn).is_ok();
        } else {
            assert_that!(hsn).is_err();
        }
    }
}