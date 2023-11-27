use lazy_static::lazy_static;
use uuid::Uuid;
use crate::accounting::currency::currency_models::{AuditMetadataBase, SEED_CURRENCY_ID};
use crate::tenant::tenant_models::SEED_TENANT_ID;
use std::str::FromStr;

lazy_static! {
    pub static ref SEED_LEDGER_MASTER_ID:Uuid= Uuid::from_str("").unwrap();//todo
}
#[derive(Debug)]
pub struct LedgerMaster {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub display_name: String,
    pub currency_master_id: Uuid,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug)]
pub struct CreateLedgerMasterEntryRequest {
    pub tenant_id: Uuid,
    pub display_name: String,
    pub currency_master_id: Uuid,
    pub audit_metadata: AuditMetadataBase,
}

#[cfg(test)]
#[derive(Default)]
pub struct CreateLedgerMasterEntryRequestTestBuilder {
    pub tenant_id: Option<Uuid>,
    pub display_name: Option<String>,
    pub currency_master_id: Option<Uuid>,
    pub audit_metadata: Option<AuditMetadataBase>,
}

#[cfg(test)]
pub fn a_create_ledger_master_entry_request(builder: CreateLedgerMasterEntryRequestTestBuilder) -> CreateLedgerMasterEntryRequest {
    CreateLedgerMasterEntryRequest {
        tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
        display_name: builder.display_name.unwrap_or("".to_string()),
        currency_master_id: builder.currency_master_id.unwrap_or(*SEED_CURRENCY_ID),
        audit_metadata: builder.audit_metadata.unwrap_or_else(||
            crate::accounting::currency::currency_models::
            an_audit_metadata_base(Default::default())),
    }
}