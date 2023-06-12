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
pub fn an_audit_metadata_base(test_builder:AuditMetadataBaseTestBuilder) -> AuditMetadataBase {
     AuditMetadataBase{
        created_by: test_builder.created_by.unwrap_or("".to_string()),
        updated_by: test_builder.updated_by.unwrap_or("".to_string()),
        created_at: test_builder.created_at.unwrap_or(0),
        updated_at: test_builder.updated_at.unwrap_or(0),
    }
}

#[derive(Debug)]
pub struct CurrencyMaster {
    pub id: i32,
    pub tenant_id: i32,
    pub scale: i16,
    ///16 char
    pub display_name: String,
    ///50 char
    pub description: String,
    pub audit_metadata: AuditMetadataBase,
}

struct CurrencyAmount {
    scale: i16,
    amount: i64,
}
