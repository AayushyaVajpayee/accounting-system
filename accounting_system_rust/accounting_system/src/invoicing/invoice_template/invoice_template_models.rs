use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;

#[derive(Debug,Serialize,Deserialize,Builder)]
pub struct InvoiceTemplateMaster{
    pub base_master_fields:BaseMasterFields,
    pub sample_doc_s3_id:Option<String>,
    pub audit_metadata:AuditMetadataBase
}


#[cfg(test)]
pub mod tests{
    use std::str::FromStr;

    use lazy_static::lazy_static;
    use uuid::Uuid;

    lazy_static!{
        pub static ref SEED_INVOICE_TEMPLATE_ID:Uuid =Uuid::from_str("018d5552-fb70-7d28-bbf6-7e726e5c15eb").unwrap();
    }
}