use std::ops::Not;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::hsn_code_generated::HSN_SET;
use crate::sac_code_generated::SAC_SET;

#[derive(Debug, Serialize, Deserialize, Clone,PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GstItemCode {
    HsnCode(Hsn),
    SacCode(Sac),
}

impl GstItemCode {
    pub fn as_str(&self) -> &str {
        match self {
            GstItemCode::HsnCode(a) => { a.0.as_str() }
            GstItemCode::SacCode(a) => { a.0.as_str() }
        }
    }

    pub fn new(a: String) -> anyhow::Result<Self> {
        return if Hsn::is_hsn(a.as_str()) {
            Ok(GstItemCode::HsnCode(Hsn::new(a)?))
        } else {
            Ok(GstItemCode::SacCode(Sac::new(a)?))
        };
    }
}

#[derive(Debug, Serialize, Deserialize, Clone,PartialEq)]
#[serde(try_from = "String")]
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

    pub fn is_hsn(value: &str) -> bool {
        let parsed_hsn = value.parse::<u32>();
        if let Ok(p) = parsed_hsn {
            return HSN_SET.contains(&p);
        };

        return false;
    }
}

impl TryFrom<String> for Hsn {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Hsn::new(value).context("")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone,PartialEq)]
#[serde(try_from = "String")]
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

impl TryFrom<String> for Sac {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Sac::new(value).context("")
    }
}

#[cfg(test)]
mod hsn_tests {
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;

    use crate::hsc_sac::Hsn;

    #[rstest]
    #[case("", false)]
    #[case("123432432", false)]
    #[case("99", false)]
    #[case("9980k", false)]
    #[case("9801", true)]
    #[case("91081100", true)]
    #[case("910811000", false)]
    #[case("0101", true)]
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
mod sac_tests {
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;

    use crate::hsc_sac::Sac;

    #[rstest]
    #[trace]
    #[case("", false)]
    #[case("123432432", false)]
    #[case("99", false)]
    #[case("9954", true)]
    #[case("9997", true)]
    #[case("9954av", false)]
    #[case("999799", true)]
    #[case("99979900", false)]
    #[case("0101", false)]
    pub fn test_sac(#[case] input: String, #[case] valid: bool) {
        let hsn = Sac::new(input);
        if valid {
            assert_that!(hsn).is_ok();
        } else {
            assert_that!(hsn).is_err();
        }
    }
}