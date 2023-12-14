use serde::{Deserialize, Serialize};

use gstin_validator::gstin_models::validate_gstin;

#[derive(Debug, Serialize, Deserialize,Default,PartialEq)]
pub struct GstinNo(String);

impl GstinNo {
    pub fn new(gstin: String) -> Result<Self, String> {
        let validation_errors = validate_gstin(gstin.as_str());
        if let Some(err) = validation_errors {
            return Err(err.to_string());
        }
        Ok(GstinNo(gstin))
    }
}

#[cfg(test)]
pub mod gstin_no_tests {
    use rand::Rng;
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;

    use gstin_validator::gstin_models::gstin_checksum;

    use crate::masters::company_master::company_master_models::gstin_no::GstinNo;

    const GST_STATE_CODE_LIST: [u16; 39] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 26,
        27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 97, 99,
    ];
    const ALPHABETS: &[u8] = b"ABCDEFGHIJKLNMNOPQRSTUVWXYZ";
    const SEED_GSTIN: &str = "05AABCA5291p1ZD";

    impl GstinNo {
        pub fn get_str(&self) -> &str {
            self.0.as_str()
        }
    }

    pub fn generate_random_gstin_no() -> GstinNo {
        let mut rng = rand::thread_rng();
        let gst_idx = rng.gen_range(0..GST_STATE_CODE_LIST.len());
        let gst_state_code = format!("{:0>2}", GST_STATE_CODE_LIST[gst_idx]);
        let gst_mid_random_part = (0..5)
            .map(|_| {
                let idx = rng.gen_range(0..ALPHABETS.len());
                ALPHABETS[idx] as char
            })
            .collect::<String>();
        let mut new_gst = format!(
            "{}{}{}",
            gst_state_code,
            gst_mid_random_part,
            &SEED_GSTIN[7..]
        );
        let check_sum = gstin_checksum(new_gst.as_str()).unwrap();
        new_gst.remove(14);
        new_gst.push(check_sum);
        GstinNo::new(new_gst).unwrap()
    }

    #[rstest]
    #[case("", false)]
    #[case("dfaafda", false)]
    #[case("dfafdadad", false)]
    #[case("22AAAAA0000A1Z5", false)]
    fn test_gstin_no(#[case] input: String, #[case] valid: bool) {
        let k = GstinNo::new(input);
        if valid {
            assert_that!(k).is_ok();
        } else {
            assert_that!(k).is_err();
        }
    }
}

