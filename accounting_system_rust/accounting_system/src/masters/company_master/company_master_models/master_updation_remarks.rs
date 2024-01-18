use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct MasterUpdationRemarks(String);

impl MasterUpdationRemarks {
    pub fn new(remark: &str) -> anyhow::Result<Self> {
        let remark = remark.trim();
        if remark.is_empty() || remark.len() > 70 {
            bail!("remark cannot be empty or greater than {} chars",70)
        }
        Ok(Self(remark.to_string()))
    }

    pub fn get_str(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(test)]
mod master_updation_remarks_tests {
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;

    use crate::masters::company_master::company_master_models::master_updation_remarks::MasterUpdationRemarks;

    #[rstest]
    #[case("abdfad", true)]
    #[case(" ", false)]
    #[case("", false)]
    #[case(
    "lfjdalfjdalfjldjgldajflkdjalfkjalfkfdaf dafaf jalhijvcnao j flajd foj eo jeo",
    false
    )]
    fn test_failure_conditions_for_master_updation_remarks(
        #[case] input: String,
        #[case] valid: bool,
    ) {
        let k = MasterUpdationRemarks::new(input.as_str());
        if valid {
            assert_that!(k).is_ok();
        } else {
            assert_that!(k).is_err();
        }
    }
}