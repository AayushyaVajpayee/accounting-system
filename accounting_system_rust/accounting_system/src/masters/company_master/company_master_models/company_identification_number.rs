use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompanyIdentificationNumber(String);

impl CompanyIdentificationNumber {
    pub fn new(cin: &str) -> anyhow::Result<Self> {
        Self::validate(cin)?;
        Ok(CompanyIdentificationNumber(cin.to_string()))
    }
    pub fn validate(cin: &str) -> anyhow::Result<()> {
        let cin = cin.trim();
        if cin.len() != 21 {
            bail!(
                "cin length should be {} chars and should be alphanumeric",
                21
            )
        }
        Ok(())
    }
    pub fn get_str(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(test)]
pub mod cin_tests {
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    use rstest::rstest;
    use speculoos::assert_that;
    use speculoos::prelude::ResultAssertions;

    use crate::masters::company_master::company_master_models::company_identification_number::CompanyIdentificationNumber;

    pub fn generate_random_company_identification_number() -> CompanyIdentificationNumber {
        let rng = rand::thread_rng();
        let p = rng
            .sample_iter(Alphanumeric)
            .take(21)
            .map(|a| a as char)
            .collect::<String>();
        CompanyIdentificationNumber::new(p.as_str()).unwrap()
    }

    #[rstest]
    #[case("", false)]
    #[case("   ", false)]
    #[case("fdjkkjajfajfkajlkjdal", true)]
    fn test_cin_cases(#[case] input: String, #[case] valid: bool) {
        let k = CompanyIdentificationNumber::new(input.as_str());
        if valid {
            assert_that!(k).is_ok();
        } else {
            assert_that!(k).is_err();
        }
    }
}
