use crate::accounting::currency::currency_models::AuditMetadataBase;

#[derive(Debug)]
pub struct LedgerMaster {
    pub id: i32,
    pub tenant_id: i32,
    pub display_name: String,
    pub currency_master_id: i16,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug)]
pub struct CreateLedgerMasterEntryRequest {
    pub tenant_id: i32,
    pub display_name: String,
    pub currency_master_id: i16,
    pub audit_metadata: AuditMetadataBase,
}

#[cfg(test)]
#[derive(Default)]
pub struct CreateLedgerMasterEntryRequestTestBuilder {
    pub tenant_id: Option<i32>,
    pub display_name: Option<String>,
    pub currency_master_id: Option<i16>,
    pub audit_metadata: Option<AuditMetadataBase>,
}

#[cfg(test)]
pub fn a_create_ledger_master_entry_request(builder: CreateLedgerMasterEntryRequestTestBuilder) -> CreateLedgerMasterEntryRequest {
    CreateLedgerMasterEntryRequest {
        tenant_id: builder.tenant_id.unwrap_or(1),
        display_name: builder.display_name.unwrap_or("".to_string()),
        currency_master_id: builder.currency_master_id.unwrap_or(1),
        audit_metadata: builder.audit_metadata.unwrap_or_else(||
            crate::accounting::currency::currency_models::
            an_audit_metadata_base(Default::default())),
    }
}