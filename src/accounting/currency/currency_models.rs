#[derive(Debug)]
pub struct AuditMetadataBase {
    pub created_by: String,
    pub updated_by: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[cfg(test)]
#[derive(Default)]
pub struct AuditMetadataBaseTestBuilder {
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


#[cfg(test)]
pub fn an_audit_metadata_base(test_builder: AuditMetadataBaseTestBuilder) -> AuditMetadataBase {
    AuditMetadataBase {
        created_by: test_builder.created_by.unwrap_or("".to_string()),
        updated_by: test_builder.updated_by.unwrap_or("".to_string()),
        created_at: test_builder.created_at.unwrap_or(0),
        updated_at: test_builder.updated_at.unwrap_or(0),
    }
}

#[derive(Debug)]
pub struct CurrencyMaster {
    pub id: i16,
    pub tenant_id: i32,
    pub scale: i16,
    ///16 char
    pub display_name: String,
    ///50 char
    pub description: String,
    pub audit_metadata: AuditMetadataBase,
}


#[derive(Debug)]
pub struct CreateCurrencyMasterRequest {
    pub tenant_id: i32,
    pub scale: i16,
    pub display_name: String,
    pub description: String,
    pub audit_metadata: AuditMetadataBase,
}

#[cfg(test)]
#[derive(Debug, Default)]
pub struct CreateCurrencyMasterRequestTestBuilder {
    pub tenant_id: Option<i32>,
    pub scale: Option<i16>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub audit_metadata: Option<AuditMetadataBase>,
}

#[cfg(test)]
pub fn a_create_currency_master_request(builder:
                                        CreateCurrencyMasterRequestTestBuilder)
                                        -> CreateCurrencyMasterRequest {
    CreateCurrencyMasterRequest {
        tenant_id: builder.tenant_id.unwrap_or(0),
        scale: builder.scale.unwrap_or(0),
        display_name: builder.display_name.unwrap_or("".to_string()),
        description: builder.description.unwrap_or("".to_string()),
        audit_metadata: builder.audit_metadata
            .unwrap_or_else(||
                an_audit_metadata_base(Default::default())),
    }
}


#[cfg(test)]
#[derive(Default)]
pub struct CurrencyMasterTestBuilder {
    pub id: Option<i16>,
    pub tenant_id: Option<i32>,
    pub scale: Option<i16>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub audit_metadata: Option<AuditMetadataBase>,
}

#[cfg(test)]
pub fn a_currency_master(builder: CurrencyMasterTestBuilder) -> CurrencyMaster {
    CurrencyMaster {
        id: builder.id.unwrap_or(0),
        tenant_id: builder.tenant_id.unwrap_or(0),
        scale: builder.scale.unwrap_or(0),
        display_name: builder.display_name.unwrap_or("".to_string()),
        description: builder.description.unwrap_or("".to_string()),
        audit_metadata: builder.audit_metadata.unwrap_or_else(|| an_audit_metadata_base(Default::default())),
    }
}

#[derive(Debug)]
struct CurrencyAmount {
    scale: i16,
    amount: i64,
}
