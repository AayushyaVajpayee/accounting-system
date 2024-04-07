use std::str::FromStr;

use chrono::{Days, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::util::generate_random_string;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvoiceRequest {
    pub idempotence_key: Uuid,
    pub invoice_template_id: Uuid,
    pub invoicing_series_mst_id: Uuid,
    pub currency_id: Uuid,
    pub service_invoice: bool,
    pub einvoicing_applicable: bool,
    pub b2b_invoice: bool,
    ///billed from id
    pub supplier_id: Uuid,
    ///if  none then same as that of supplier id
    pub dispatch_from_id: Option<Uuid>,
    pub bill_ship_detail: Option<BillShipDetail>,
    pub order_number: Option<String>,
    pub order_date: Option<NaiveDate>,
    pub payment_terms: Option<PaymentTermsValidated>,
    pub invoice_lines: Vec<CreateInvoiceLineRequest>,
    pub additional_charges: Vec<CreateAdditionalChargeRequest>,
    pub invoice_remarks: Option<String>,
    pub ecommerce_gstin: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentTermsValidated {
    pub due_days: u32,
    pub discount_days: Option<u32>,
    pub discount_percent: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BillShipDetail {
    pub billed_to_customer_id: Uuid,
    pub shipped_to_customer_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAdditionalChargeRequest {
    pub line_title: String,
    pub rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LineQuantity {
    pub quantity: f64,
    pub uom: String,//Piece
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateInvoiceLineRequest {
    pub product_item_id: Uuid,
    pub quantity: LineQuantity,
    pub free_quantity: LineQuantity,
    pub unit_price: f64,
    pub discount_percentage: f32,
    pub mrp: Option<f64>,
    pub batch_no: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    //is the line item payable under reverse charge
    pub reverse_charge_applicable: bool,
}


async fn create_invoice(request: &CreateInvoiceRequest, tenant_id: Uuid, user_id: Uuid) -> String {
    let cli = reqwest::Client::new();
    let path = format!("{}{}", "http://localhost:8080/", "/create-invoice");
    let req = cli
        .post(path.as_str())
        .json(request)
        .header("x-acc-tenant-id", tenant_id.to_string())
        .header("x-acc-user-id", user_id.to_string())
        .send()
        .await
        .unwrap();
    let d: String = req.json().await.unwrap();
    d
}

pub async fn create_random_invoice(product_item_id:Uuid,supplier_id: Uuid, bill_ship_detail: BillShipDetail,
                               currency_id: Uuid, invoicing_series_mst_id: Uuid,
                               tenant_id: Uuid, user_id: Uuid) -> String {
    let order_date = Utc::now().checked_sub_days(Days::new(8)).unwrap().date_naive();
    let create_invoice_request = CreateInvoiceRequest {
        idempotence_key: Uuid::now_v7(),
        invoice_template_id: Uuid::from_str("018d5552-fb70-7d28-bbf6-7e726e5c15eb").unwrap(),
        invoicing_series_mst_id,
        currency_id,
        service_invoice: false,
        einvoicing_applicable: false,
        b2b_invoice: true,
        supplier_id,
        dispatch_from_id: None,
        bill_ship_detail: Some(bill_ship_detail),
        order_number: Some(generate_random_string(10)),
        order_date: Some(order_date),
        payment_terms: Some(PaymentTermsValidated {
            due_days: 5,
            discount_days: None,
            discount_percent: None,
        }),
        invoice_lines: vec![
            CreateInvoiceLineRequest {
                product_item_id,
                quantity: LineQuantity { quantity: 10.0, uom: "Piece".to_string() },
                free_quantity: LineQuantity { quantity: 00.0, uom: "Piece".to_string() },
                unit_price: 50.0,
                discount_percentage: 0.0,
                mrp: Some(55.0),
                batch_no: Some(generate_random_string(6)),
                expiry_date: None,
                reverse_charge_applicable: false,
            }
        ],
        additional_charges: vec![],
        invoice_remarks: Some(generate_random_string(35)),
        ecommerce_gstin: None,
    };
    create_invoice(&create_invoice_request, tenant_id, user_id).await
}