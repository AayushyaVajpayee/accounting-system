use anyhow::ensure;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct CreateInvoiceNumberSeriesRequest {
    pub idempotence_key: Uuid,
    pub tenant_id: Uuid,
    pub name: InvoicingSeriesName,
    pub prefix: InvoiceNumberPrefix,
    pub zero_padded_counter: bool,
    ///primarily for migration purpose and nothing else
    pub start_value: Option<u32>,
    pub financial_year: FinancialYear,
    pub created_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(try_from = "i32")]
pub struct FinancialYear(i32);

impl FinancialYear {
    pub fn new(value: i32) -> anyhow::Result<Self> {
        ensure!(value>2023,"financial year needs to be more than 2023");
        ensure!(value<2060,"financial year needs tb be less than 2060");
        Ok(Self(value))
    }

    pub fn inner(&self) -> i32 {
        self.0
    }
}

impl TryFrom<i32> for FinancialYear {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        FinancialYear::new(value)
    }
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct InvoicingSeriesMaster {
    pub base_master_fields: BaseMasterFields,
    pub name: InvoicingSeriesName,
    pub prefix: InvoiceNumberPrefix,
    pub zero_padded_counter: bool,
    pub audit_metadata: AuditMetadataBase,
}

#[derive(Debug)]
pub struct InvoicingSeriesCounter {
    pub id: Uuid,
    pub entity_version_id: i32,
    pub tenant_id: Uuid,
    pub invoicing_series_id: Uuid,
    pub financial_year: FinancialYear,
    pub counter: i32,
    pub start_value: i32,
    pub audit_metadata: AuditMetadataBase,
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(try_from = "String")]
pub struct InvoiceNumberPrefix(String);

impl Default for InvoiceNumberPrefix {
    fn default() -> Self {
        Self("INV".to_string())
    }
}


impl InvoiceNumberPrefix {
    pub fn new(value: &str) -> anyhow::Result<Self> {
        let value = value.trim();
        ensure!(value.len()<=7,"invoice number prefix should be less than equal to 7 chars");
        ensure!(!value.is_empty(),"invoice number prefix cannot be empty");
        ensure!(value.chars()
            .all(|a| a.is_ascii_alphanumeric() || a == '/' || a == '-'),
        "invoice number prefix can only contain alphanumeric characters or / or -");
        Ok(InvoiceNumberPrefix(value.to_uppercase()))
    }
    pub fn inner(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<String> for InvoiceNumberPrefix {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        InvoiceNumberPrefix::new(value.as_str())
    }
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(try_from = "String")]
pub struct InvoicingSeriesName(String);

impl Default for InvoicingSeriesName {
    fn default() -> Self {
        Self("default-series".to_string())
    }
}

impl InvoicingSeriesName {
    pub fn new(value: &str) -> anyhow::Result<Self> {
        let value = value.trim();
        ensure!(value.len() <=30,"series name should be less than 30 chars");
        ensure!(!value.is_empty(),"series name cannot be empty");
        ensure!(value.chars().all(|a|a.is_ascii_alphanumeric()||a=='-'),"series name can only have alphanumeric or - as character");
        Ok(InvoicingSeriesName(value.to_lowercase()))
    }
    pub fn inner(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<String> for InvoicingSeriesName {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        InvoicingSeriesName::new(value.as_str())
    }
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;
    use uuid::Uuid;

    use crate::accounting::user::user_models::SEED_USER_ID;
    use crate::invoicing::invoicing_series::invoicing_series_models::{CreateInvoiceNumberSeriesRequest, CreateInvoiceNumberSeriesRequestBuilder, FinancialYear, InvoiceNumberPrefix, InvoicingSeriesName};
    use crate::tenant::tenant_models::tests::SEED_TENANT_ID;
    use lazy_static::lazy_static;
    lazy_static! {
        pub static ref SEED_INVOICING_SERIES_MST_ID:Uuid= Uuid::from_str("018d417d-e88a-732b-bdd9-db9aec8d3f78").unwrap();
    }
    pub fn a_create_invoice_number_series_request(builder: CreateInvoiceNumberSeriesRequestBuilder) -> CreateInvoiceNumberSeriesRequest {
        CreateInvoiceNumberSeriesRequest {
            idempotence_key: builder.idempotence_key.unwrap_or_else(Uuid::now_v7),
            tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
            name: builder.name.unwrap_or(InvoicingSeriesName::new("test-name").unwrap()),
            prefix: builder.prefix.unwrap_or(InvoiceNumberPrefix::new("tes").unwrap()),
            zero_padded_counter: false,
            start_value: None,
            financial_year: builder.financial_year.unwrap_or(FinancialYear::new(2024).unwrap()),
            created_by: builder.created_by.unwrap_or(*SEED_USER_ID),
        }
    }
}