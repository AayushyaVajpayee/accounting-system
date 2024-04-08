use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::common_utils::pg_util::pg_util::{create_composite_type_db_row, ToPostgresString};
use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;

#[derive(Debug, Serialize, Deserialize, Builder,Default,PartialEq,Clone)]
pub struct InvoiceTemplateMaster {
    pub base_master_fields: BaseMasterFields,
    pub sample_doc_s3_id: Option<String>,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CreateInvoiceTemplateRequest {
    pub idempotence_key: Uuid,
    pub sample_doc_s3_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CreateInvoiceTemplateDbRequest {
    pub idempotence_key: Uuid,
    pub sample_doc_s3_id: Option<String>,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
}

impl ToPostgresString for CreateInvoiceTemplateDbRequest {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        let fields: &[&dyn ToPostgresString] = &[
            &self.idempotence_key,
            &self.sample_doc_s3_id,
            &self.tenant_id,
            &self.user_id,
        ];
        create_composite_type_db_row(fields, f)
    }

    fn db_type_name(&self) -> &'static str {
        "create_invoice_template_request"
    }
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use lazy_static::lazy_static;
    use uuid::Uuid;

    lazy_static! {
        pub static ref SEED_INVOICE_TEMPLATE_ID: Uuid =
            Uuid::from_str("018d5552-fb70-7d28-bbf6-7e726e5c15eb").unwrap();
    }
}
