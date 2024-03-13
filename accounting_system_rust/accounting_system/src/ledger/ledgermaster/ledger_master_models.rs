use derive_builder::Builder;
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;

#[derive(Debug)]
pub struct LedgerMaster {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub display_name: String,
    pub currency_master_id: Uuid,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug,Builder)]
pub struct CreateLedgerMasterEntryRequest {
    pub tenant_id: Uuid,
    pub display_name: String,
    pub currency_master_id: Uuid,
    pub audit_metadata: AuditMetadataBase,
}

#[cfg(test)]
pub mod tests{
    use std::str::FromStr;

    use lazy_static::lazy_static;
    use uuid::Uuid;

    use crate::accounting::currency::currency_models::tests::{an_audit_metadata_base, SEED_CURRENCY_ID};
    use crate::ledger::ledgermaster::ledger_master_models::{CreateLedgerMasterEntryRequest, CreateLedgerMasterEntryRequestBuilder};
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;

    lazy_static! {
    pub static ref SEED_LEDGER_MASTER_ID:Uuid= Uuid::from_str("82a4209a-d298-747f-902f-d323df4f4400").unwrap();
    }
    pub fn a_create_ledger_master_entry_request(builder: CreateLedgerMasterEntryRequestBuilder) -> CreateLedgerMasterEntryRequest {
        CreateLedgerMasterEntryRequest {
            tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
            display_name: builder.display_name.unwrap_or("".to_string()),
            currency_master_id: builder.currency_master_id.unwrap_or(*SEED_CURRENCY_ID),
            audit_metadata: builder.audit_metadata.unwrap_or_else(||
                an_audit_metadata_base(Default::default())),
        }
    }
}