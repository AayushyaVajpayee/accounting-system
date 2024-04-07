use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::util::{generate_random_string, send_request};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvoicingSeriesRequest {
    pub idempotence_key: Uuid,
    pub name: String,
    pub prefix: String,
    pub zero_padded_counter: bool,
    ///primarily for migration purpose and nothing else
    pub start_value: Option<u32>,
    pub financial_year: i32,
}

async fn create_invoicing_series_mst(
    request: &CreateInvoicingSeriesRequest,
    tenant_id: Uuid,
    user_id: Uuid,
) -> Uuid {
    send_request(request, tenant_id, user_id, "invoice-no-series/create").await
}


pub async fn create_random_invoice_series_mst(
    tenant_id: Uuid,
    user_id: Uuid,
) -> Uuid {
    let invoicing_series_req = CreateInvoicingSeriesRequest {
        idempotence_key: Uuid::now_v7(),
        name: generate_random_string(20),
        prefix: format!("IV/{}/", generate_random_string(3)),
        zero_padded_counter: false,
        start_value: None,
        financial_year: 2024,
    };
    create_invoicing_series_mst(&invoicing_series_req, tenant_id, user_id).await
}