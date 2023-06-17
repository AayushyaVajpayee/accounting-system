use crate::accounting::currency::currency_models::{an_audit_metadata_base, AuditMetadataBase};

#[derive(Debug)]
pub struct AccountTypeMaster {
    pub id: i16,
    pub tenant_id: i32,
    pub display_name: String,
    pub account_code: i16,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug)]
pub struct CreateAccountTypeMasterRequest {
    pub tenant_id: i32,
    pub display_name: String,
    pub account_code: i16,
    pub audit_metadata: AuditMetadataBase,
}

#[cfg(test)]
#[derive(Debug, Default)]
pub struct CreateAccountTypeMasterRequestTestBuilder {
    pub tenant_id: Option<i32>,
    pub display_name: Option<String>,
    pub account_code: Option<i16>,
    pub audit_metadata: Option<AuditMetadataBase>,
}

#[cfg(test)]
pub fn a_create_account_type_master_request(builder: CreateAccountTypeMasterRequestTestBuilder) -> CreateAccountTypeMasterRequest {
    CreateAccountTypeMasterRequest {
        tenant_id: builder.tenant_id.unwrap_or(0),
        account_code: builder.account_code.unwrap_or(0),
        display_name: builder.display_name.unwrap_or("".to_string()),
        audit_metadata: builder.audit_metadata.unwrap_or_else(|| an_audit_metadata_base(Default::default())),
    }
}

#[cfg(test)]
#[derive(Debug, Default)]
pub struct AccountTypeMasterTestBuilder {
    pub id: Option<i16>,
    pub tenant_id: Option<i32>,
    pub display_name: Option<String>,
    pub account_code: Option<i16>,
    pub audit_metadata: Option<AuditMetadataBase>,
}

#[derive(Debug)]
pub struct Account {
    pub id: i32,
    pub tenant_id: i32,
    ///max 20 char string of only numeric data
    pub display_code: String,
    pub account_type_id: i16,
    pub opening_balance: i64,
    pub current_balance: i64,
    pub currency_master_id: i16,
    pub user_id: i32,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug)]
pub struct CreateAccountRequest {
    pub tenant_id: i32,
    pub display_code: String,
    //todo should it be self generated
    pub account_type_id: i16,
    pub opening_balance: i64,
    pub currency_master_id: i16,
    pub user_id: i32,
    pub audit_metadata: AuditMetadataBase,
}

#[cfg(test)]
#[derive(Debug, Default)]
pub struct CreateAccountRequestTestBuilder {
    pub tenant_id: Option<i32>,
    pub display_code: Option<String>,
    pub account_type_id: Option<i16>,
    pub opening_balance: Option<i64>,
    pub currency_master_id: Option<i16>,
    pub user_id: Option<i32>,
    pub audit_metadata: Option<AuditMetadataBase>,
}