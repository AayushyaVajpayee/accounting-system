use crate::accounting::currency::currency_models::{AuditMetadataBase};

#[derive(Debug)]
pub struct Tenant {
    pub id: i64,
    pub display_name: String,
    pub audit_metadata: AuditMetadataBase,
}

#[cfg(test)]
#[derive(Default)]
pub struct TenantTestBuilder {
    pub id: Option<i64>,
    pub display_name: Option<String>,
    pub audit_metadata: Option<AuditMetadataBase>,
}

#[cfg(test)]
pub fn a_tenant(builder: TenantTestBuilder) -> Tenant {
    Tenant {
        id: builder.id.unwrap_or(0),
        display_name: builder.display_name.unwrap_or("".to_string()),
        audit_metadata: builder.audit_metadata.unwrap_or_else(||
            crate::accounting::currency::currency_models::
            an_audit_metadata_base(Default::default())),
    }
}