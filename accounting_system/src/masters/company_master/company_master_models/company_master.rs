use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;
use crate::masters::company_master::company_master_models::company_identification_number::CompanyIdentificationNumber;
use crate::masters::company_master::company_master_models::company_name::CompanyName;

#[derive(Debug, Serialize, Deserialize)]
pub struct CompanyMaster {
    pub base_master_fields: BaseMasterFields,
    pub name: CompanyName,
    pub cin: CompanyIdentificationNumber,
    pub audit_metadata: AuditMetadataBase,
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use lazy_static::lazy_static;
    use uuid::Uuid;

    use crate::accounting::currency::currency_models::{an_audit_metadata_base, AuditMetadataBase};
    use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;
    use crate::masters::company_master::company_master_models::base_master_fields::tests::a_base_master_field;
    use crate::masters::company_master::company_master_models::company_identification_number::cin_tests::generate_random_company_identification_number;
    use crate::masters::company_master::company_master_models::company_identification_number::CompanyIdentificationNumber;
    use crate::masters::company_master::company_master_models::company_master::CompanyMaster;
    use crate::masters::company_master::company_master_models::company_name::CompanyName;

    lazy_static! {
    pub static ref SEED_COMPANY_MASTER_ID:Uuid= Uuid::from_str("018c5e2d-615b-742f-85e2-907c65daf8f4").unwrap();
}


    #[derive(Debug, Default)]
    pub struct CompanyMasterTestDataBuilder {
        pub base_master_fields: Option<BaseMasterFields>,
        pub name: Option<CompanyName>,
        pub cin: Option<CompanyIdentificationNumber>,
        pub audit_metadata: Option<AuditMetadataBase>,
    }


    pub fn a_company_master(builder: CompanyMasterTestDataBuilder) -> CompanyMaster {
        CompanyMaster {
            base_master_fields: builder.base_master_fields
                .unwrap_or_else(|| a_base_master_field(Default::default())),
            name: builder.name.unwrap_or(CompanyName::new("test_company").unwrap()),
            cin: builder.cin.unwrap_or_else(generate_random_company_identification_number),
            audit_metadata: builder.audit_metadata.unwrap_or_else(|| an_audit_metadata_base(Default::default())),
        }
    }
}