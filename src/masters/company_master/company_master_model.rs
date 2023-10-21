use uuid::Uuid;

use gstin_validator::gstin_models::validate_gstin;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::masters::company_master::company_master_model::MasterStatusEnum::{
    Approved, ChangesRequested, Deleted, PendingApproval,
};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum MasterStatusEnum {
    PendingApproval = 0,
    Approved = 1,
    ChangesRequested = 2,
    Deleted = 3,
}
impl MasterStatusEnum {
    pub fn get_enum_for_value(value: usize) -> Result<Self, &'static str> {
        match value {
            0 => Ok(PendingApproval),
            1 => Ok(Approved),
            2 => Ok(ChangesRequested),
            3 => Ok(Deleted),
            _ => Err("no master status for this value"),
        }
    }
}
#[cfg(test)]
mod master_status_enum_tests {
    use rstest::rstest;
    use spectral::assert_that;

    use crate::masters::company_master::company_master_model::MasterStatusEnum;
    use crate::masters::company_master::company_master_model::MasterStatusEnum::Approved;
    use crate::masters::company_master::company_master_model::MasterStatusEnum::ChangesRequested;
    use crate::masters::company_master::company_master_model::MasterStatusEnum::Deleted;
    use crate::masters::company_master::company_master_model::MasterStatusEnum::PendingApproval;

    #[rstest]
    #[case(0, Ok(PendingApproval))]
    #[case(1, Ok(Approved))]
    #[case(2, Ok(ChangesRequested))]
    #[case(3, Ok(Deleted))]
    #[case(4, Err("no master status for this value"))]
    fn test_master_status_enum(
        #[case] input: usize,
        #[case] output: Result<MasterStatusEnum, &'static str>,
    ) {
        let k = MasterStatusEnum::get_enum_for_value(input);
        assert_that!(k).matches(|a| (*a) == output);
    }
}
#[derive(Debug)]
pub struct MasterUpdationRemarks(String);

impl MasterUpdationRemarks {
    pub fn new(remark: &str) -> Result<Self, &'static str> {
        let remark = remark.trim();
        if remark.is_empty() || remark.len() > 70 {
            return Err("remark cannot be empty or greater than 70 chars");
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

    use crate::masters::company_master::company_master_model::MasterUpdationRemarks;

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
#[derive(Debug)]
pub struct CompanyName(String);

impl CompanyName {
    pub fn new(name: &str) -> Result<Self, &'static str> {
        Self::validate(name)?;
        Ok(Self(name.to_string()))
    }

    pub fn validate(name:&str)->Result<(),&'static str>{
        let name = name.trim();
        if name.is_empty() || name.len() > 50 {
            return Err("company name cannot be empty or more than 50 chars");
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

    use crate::masters::company_master::company_master_model::CompanyName;

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

#[derive(Debug)]
pub struct CompanyIdentificationNumber(String);
impl CompanyIdentificationNumber {
    pub fn new(cin: &str) -> Result<Self, &'static str> {
        Self::validate(cin)?;
        Ok(CompanyIdentificationNumber(cin.to_string()))
    }
    pub fn validate(cin:&str)->Result<(),&'static str>{
        let cin = cin.trim();
        if cin.len() != 21 {
            return Err("cin length should be 21 chars and should be alphanumeric");
        }
        Ok(())
    }
    pub fn get_str(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(test)]
mod cin_tests {
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;

    use crate::masters::company_master::company_master_model::CompanyIdentificationNumber;

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
#[derive(Debug)]
pub struct BaseMasterFields {
    pub id: Uuid,
    pub entity_version_id: i32,
    pub tenant_id: Uuid,
    pub active: bool,
    pub approval_status: MasterStatusEnum,
    pub remarks: Option<MasterUpdationRemarks>,
}
#[derive(Debug)]
pub struct CompanyMaster {
    pub base_master_fields: BaseMasterFields,
    pub name: CompanyName,
    pub cin: CompanyIdentificationNumber,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug)]
pub struct CompanyUnitMaster {
    base_master_fields: BaseMasterFields,
    company_id: Uuid,
    address_id: Uuid,
    gstin: GstinNo,
    audit_metadata: AuditMetadataBase,
}

#[derive(Debug)]
pub struct GstinNo(String);

impl GstinNo {
    pub fn new(gstin: String) -> Result<Self, String> {
        let validation_errors = validate_gstin(gstin.as_str());
        if validation_errors.is_some() {
            return Err(validation_errors.unwrap().to_string());
        }
        Ok(GstinNo(gstin))
    }
}
#[cfg(test)]
mod gstin_no_tests {
    use rstest::rstest;
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;

    use crate::masters::company_master::company_master_model::GstinNo;

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


#[cfg(test)]
pub mod test_data {
    use rand::Rng;
    use rand::distributions::Alphanumeric;
    use uuid::Uuid;

    use gstin_validator::gstin_models::gstin_checksum;

    use crate::accounting::currency::currency_models::{an_audit_metadata_base, AuditMetadataBase};
    use crate::masters::company_master::company_master_model::{BaseMasterFields, CompanyIdentificationNumber, CompanyMaster, CompanyName, GstinNo, MasterStatusEnum, MasterUpdationRemarks};
    use crate::masters::company_master::company_master_model::MasterStatusEnum::PendingApproval;
    use crate::tenant::tenant_models::SEED_TENANT_ID;

    const GST_STATE_CODE_LIST: [u16; 39] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 26,
        27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 97, 99,
    ];
    const ALPHABETS: &[u8] = b"ABCDEFGHIJKLNMNOPQRSTUVWXYZ";
    const SEED_GSTIN: &str = "05AABCA5291p1ZD";

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

    pub fn generate_random_company_identification_number() -> CompanyIdentificationNumber {
        let rng = rand::thread_rng();
        let p = rng.sample_iter(Alphanumeric)
            .take(21)
            .map(|a|a as char)
            .collect::<String>();
        CompanyIdentificationNumber::new(p.as_str()).unwrap()
    }
    #[derive(Debug,Default)]
    pub struct BaseMasterFieldsTestDataBuilder {
        pub id: Option<Uuid>,
        pub entity_version_id: Option<i32>,
        pub tenant_id: Option<Uuid>,
        pub active: Option<bool>,
        pub approval_status: Option<MasterStatusEnum>,
        pub remarks: Option<MasterUpdationRemarks>,
    }
    #[derive(Debug,Default)]
    pub struct CompanyMasterTestDataBuilder {
        pub base_master_fields: Option<BaseMasterFields>,
        pub name: Option<CompanyName>,
        pub cin: Option<CompanyIdentificationNumber>,
        pub audit_metadata: Option<AuditMetadataBase>,
    }

    pub fn a_base_master_field(builder:BaseMasterFieldsTestDataBuilder)->BaseMasterFields{
        BaseMasterFields{
            id: builder.id.unwrap_or_else(Uuid::now_v7),
            entity_version_id: builder.entity_version_id.unwrap_or(0),
            tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
            active: builder.active.unwrap_or(true),
            approval_status: builder.approval_status.unwrap_or(PendingApproval),
            remarks: builder.remarks,
        }
    }
    pub fn a_company_master(builder:CompanyMasterTestDataBuilder)->CompanyMaster{
        CompanyMaster{
            base_master_fields: builder.base_master_fields
                .unwrap_or_else(||a_base_master_field(Default::default())),
            name: builder.name.unwrap_or(CompanyName::new("test_company").unwrap()),
            cin:builder.cin.unwrap_or_else(generate_random_company_identification_number),
            audit_metadata: builder.audit_metadata.unwrap_or_else(||an_audit_metadata_base(Default::default())),
        }
    }
}
