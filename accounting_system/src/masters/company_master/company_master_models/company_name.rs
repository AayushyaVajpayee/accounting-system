use serde::{Deserialize, Serialize};
use anyhow::{Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompanyName(String);

impl CompanyName {
    pub fn new(name: &str) -> Result<Self> {
        Self::validate(name)?;
        Ok(Self(name.to_string()))
    }

    pub fn validate(name: &str) -> Result<()> {
        let name = name.trim();
        if name.is_empty() || name.len() > 50 {
            anyhow::bail!("company name cannot be empty or more than {} chars",50);
        }
        Ok(())
    }
    pub fn get_str(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(test)]
mod company_name_tests {
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;
    use crate::masters::company_master::company_master_models::company_name::CompanyName;


    #[rstest]
    #[case("abdfad", true)]
    #[case(" ", false)]
    #[case("", false)]
    #[case("lfjdad lfjdalfjldjgldajflkdjalfkjalfkfdaf dafaf jal", false)]
    fn test_failure_conditions_for_master_updation_remarks(
        #[case] input: String,
        #[case] valid: bool,
    ) {
        let k = CompanyName::new(input.as_str());
        if valid {
            assert_that!(k).is_ok();
        } else {
            assert_that!(k).is_err();
        }
    }
}