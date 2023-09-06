use std::ops::Not;

use thiserror::Error;

use crate::hsn_code_generated::HSN_SET;

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
        if HSN_SET.contains(&0_u32).not() {
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
        let parsed_hsn = sac.parse::<u32>();
        if parsed_hsn.is_err() {
            return Err(SacError::NotNumeric);
        }
        if !HSN_SET.contains(&0_u32) {
            return Err(SacError::NotFoundInMasterData);
        }
        Ok(Self(sac))
    }
}