use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::LOCAL_HOST;
use crate::util::generate_random_string;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvoiceTemplateRequest {
    pub idempotence_key: Uuid,
    pub sample_doc_s3_id: Option<String>,
}


async fn create_invoice(request: &CreateInvoiceTemplateRequest, tenant_id: Uuid, user_id: Uuid) -> Uuid {
    let cli = reqwest::Client::new();
    let path = format!("{}{}", LOCAL_HOST, "invoice-template/create");
    let req = cli
        .post(path.as_str())
        .json(request)
        .header("x-acc-tenant-id", tenant_id.to_string())
        .header("x-acc-user-id", user_id.to_string())
        .send()
        .await
        .unwrap();
    let d: Uuid = req.json().await.unwrap();
    d
}

pub async fn create_random_invoice_template(tenant_id:Uuid,user_id:Uuid)->Uuid{
    let req =CreateInvoiceTemplateRequest{
        idempotence_key: Uuid::now_v7(),
        sample_doc_s3_id: Some(generate_random_string(10)),
    };
    create_invoice(&req,tenant_id,user_id).await
}