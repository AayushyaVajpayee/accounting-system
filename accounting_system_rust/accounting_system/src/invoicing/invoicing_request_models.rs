use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, ensure};
use chrono::NaiveDate;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use invoice_doc_generator::invoice_line::line_quantity::{FreeLineQuantity, LineQuantity};
use invoice_doc_generator::invoice_line::line_title::LineTitle;
use invoice_doc_generator::invoice_line::unit_price::Price;
use invoice_doc_generator::percentages::tax_discount_cess::DiscountPercentage;
use pdf_doc_generator::invoice_template::Invoice;

use crate::masters::company_master::company_master_models::gstin_no::GstinNo;
use crate::masters::product_item_master::product_item_models::ProductItemResponse;

#[derive(Debug)]
pub struct CreateInvoiceWithAllDetailsIncluded {
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
    pub order_number: Option<PurchaseOrderNo>,
    pub order_date: Option<PurchaseOrderDate>,
    pub payment_terms: Option<PaymentTermsValidated>,
    pub invoice_lines: Vec<CreateInvoiceLineRequestWithAllDetails>,
    pub additional_charges: Vec<CreateAdditionalChargeRequest>,
    pub invoice_remarks: Option<InvoiceRemarks>,
    pub ecommerce_gstin: Option<GstinNo>,
}
#[derive(Debug)]
pub struct CreateInvoiceLineRequestWithAllDetails {
    pub product_item_id: Arc<ProductItemResponse>,
    pub quantity: LineQuantity,
    pub free_quantity: FreeLineQuantity,
    pub unit_price: Price,
    pub discount_percentage: DiscountPercentage,
    pub mrp: Option<Price>,
    pub batch_no: Option<BatchNo>,
    pub expiry_date: Option<ExpiryDateMs>,
    //is the line item payable under reverse charge
    pub reverse_charge_applicable: bool,
}

impl CreateInvoiceRequest {
    pub fn to_create_invoice_with_all_details_included(
        self,
        product_items: Vec<Arc<ProductItemResponse>>,
    ) -> anyhow::Result<CreateInvoiceWithAllDetailsIncluded> {
        let map: HashMap<Uuid, Arc<ProductItemResponse>> = product_items
            .into_iter()
            .map(|a| (a.base_master_fields.id, a))
            .collect();
        let mut invoice_lines: Vec<CreateInvoiceLineRequestWithAllDetails> =
            Vec::with_capacity(self.invoice_lines.len());
        for il in self.invoice_lines.into_iter() {
            let pr = CreateInvoiceLineRequestWithAllDetails {
                product_item_id: map
                    .get(&il.product_item_id)
                    .context(
                        "product item not found for product id during invoice creation request",
                    )?
                    .clone(),
                quantity: il.quantity,
                free_quantity: il.free_quantity,
                unit_price: il.unit_price,
                discount_percentage: il.discount_percentage,
                mrp: il.mrp,
                batch_no: il.batch_no,
                expiry_date: il.expiry_date,
                reverse_charge_applicable: il.reverse_charge_applicable,
            };
            invoice_lines.push(pr);
        }

        Ok(CreateInvoiceWithAllDetailsIncluded {
            idempotence_key: self.idempotence_key,
            invoice_template_id: self.invoice_template_id,
            invoicing_series_mst_id: self.invoicing_series_mst_id,
            currency_id: self.currency_id,
            service_invoice: self.service_invoice,
            einvoicing_applicable: self.einvoicing_applicable,
            b2b_invoice: self.b2b_invoice,
            supplier_id: self.supplier_id,
            dispatch_from_id: self.dispatch_from_id,
            bill_ship_detail: self.bill_ship_detail,
            order_number: self.order_number,
            order_date: self.order_date,
            payment_terms: self.payment_terms,
            invoice_lines,
            additional_charges: self.additional_charges,
            invoice_remarks: self.invoice_remarks,
            ecommerce_gstin: self.ecommerce_gstin,
        })
    }
}
#[derive(Debug, Serialize, Deserialize, Builder)]
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
    pub order_number: Option<PurchaseOrderNo>,
    pub order_date: Option<PurchaseOrderDate>,
    pub payment_terms: Option<PaymentTermsValidated>,
    pub invoice_lines: Vec<CreateInvoiceLineRequest>,
    pub additional_charges: Vec<CreateAdditionalChargeRequest>,
    pub invoice_remarks: Option<InvoiceRemarks>,
    pub ecommerce_gstin: Option<GstinNo>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Builder)]
pub struct BillShipDetail {
    pub billed_to_customer_id: Uuid,
    pub shipped_to_customer_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Builder, Clone)]
pub struct CreateAdditionalChargeRequest {
    pub line_title: LineTitle,
    pub rate: Price,
}

#[derive(Debug, Serialize, Deserialize, Builder, Clone)]
pub struct CreateInvoiceLineRequest {
    pub product_item_id: Uuid,
    pub quantity: LineQuantity,
    pub free_quantity: FreeLineQuantity,
    pub unit_price: Price,
    pub discount_percentage: DiscountPercentage,
    pub mrp: Option<Price>,
    pub batch_no: Option<BatchNo>,
    pub expiry_date: Option<ExpiryDateMs>,
    //is the line item payable under reverse charge
    pub reverse_charge_applicable: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
#[serde(try_from = "String")]
pub struct InvoiceRemarks(String);

impl InvoiceRemarks {
    pub fn new(remark: &str) -> anyhow::Result<Self> {
        let remark = remark.trim();
        ensure!(
            !remark.is_empty() || remark.len() <= 100,
            "remark cannot be empty or greater than {} chars",
            100
        );
        Ok(Self(remark.to_string()))
    }

    pub fn get_str(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<String> for InvoiceRemarks {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PaymentTerms {
    pub due_days: DueDays,
    pub discount_days: Option<DiscountDays>,
    pub discount_percent: Option<DiscountPercentage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(try_from = "PaymentTerms")]
pub struct PaymentTermsValidated {
    pub due_days: DueDays,
    pub discount_days: Option<DiscountDays>,
    pub discount_percent: Option<DiscountPercentage>,
}

impl TryFrom<PaymentTerms> for PaymentTermsValidated {
    type Error = anyhow::Error;

    fn try_from(value: PaymentTerms) -> Result<Self, Self::Error> {
        if let Some(discount_days) = &value.discount_days {
            ensure!(
                value.due_days.0 >= discount_days.0,
                "discount days {} cannot be more than due days {} in payment terms",
                discount_days.0,
                value.due_days.0
            );
        }
        Ok(PaymentTermsValidated {
            due_days: value.due_days,
            discount_days: value.discount_days,
            discount_percent: value.discount_percent,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(try_from = "i32")]
pub struct DiscountDays(u16);

impl DiscountDays {
    pub fn new(value: i32) -> anyhow::Result<Self> {
        ensure!(value >= 0, "discount days cannot be less than 0");
        Ok(DiscountDays(value as u16))
    }
    #[allow(dead_code)]
    pub fn inner(&self) -> u16 {
        self.0
    }
}

impl TryFrom<i32> for DiscountDays {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        DiscountDays::new(value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(try_from = "i32")]
pub struct DueDays(u16);

impl DueDays {
    pub fn new(value: i32) -> anyhow::Result<Self> {
        ensure!(value >= 0, "due days cannot be less than 0");
        ensure!(value <= 400, "due days cannot be more than 400");
        Ok(DueDays(value as u16))
    }
    #[allow(dead_code)]
    pub fn inner(&self) -> u16 {
        self.0
    }
}

impl TryFrom<i32> for DueDays {
    type Error = anyhow::Error;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        DueDays::new(value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(try_from = "String")]
pub struct PurchaseOrderNo(String);

impl PurchaseOrderNo {
    pub fn new(value: String) -> anyhow::Result<Self> {
        let value = value.trim();
        ensure!(
            value.len() <= 30,
            "purchase order no cannot be more than 30 chars"
        );
        ensure!(
            !value.is_empty(),
            "purchase order no cannot be empty, make it null if you don't need it"
        );
        ensure!(
            value
                .chars()
                .all(|a| a.is_ascii_alphanumeric() || a == '/' || a == '-'),
            "purchase order no can only contain alphanumeric characters or / or -"
        );
        Ok(PurchaseOrderNo(value.to_owned()))
    }
    pub fn inner(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<String> for PurchaseOrderNo {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        PurchaseOrderNo::new(value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(try_from = "String")]
pub struct PurchaseOrderDate(NaiveDate);

impl PurchaseOrderDate {
    pub fn new(value: String) -> anyhow::Result<Self> {
        PurchaseOrderDate::from_str(value.as_str())
    }
    pub fn from_str(value: &str) -> anyhow::Result<Self> {
        let p = NaiveDate::parse_from_str(value, "%Y-%m-%d").context("")?;
        PurchaseOrderDate::from_date(p)
    }
    pub fn from_date(date: NaiveDate) -> anyhow::Result<Self> {
        Ok(PurchaseOrderDate(date))
    }
    pub fn get_date(&self) -> &NaiveDate {
        &self.0
    }
    pub fn epoch_millis(&self) -> Option<i64> {
        return self
            .0
            .and_hms_milli_opt(0, 0, 0, 0)
            .map(|a| a.timestamp_millis());
    }
}

impl TryFrom<String> for PurchaseOrderDate {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        PurchaseOrderDate::new(value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(try_from = "String")]
pub struct BatchNo(String);

impl BatchNo {
    pub fn new(value: String) -> anyhow::Result<Self> {
        let value = value.trim();
        ensure!(
            value.len() <= 20,
            "batch no cannot be more than 20 chars but was {} chars",
            value.len()
        );
        ensure!(
            value.chars().all(|a| a.is_ascii_alphanumeric()),
            "batch no can only contain alphanumeric characters"
        );
        Ok(Self(value.to_owned()))
    }
    pub fn inner(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<String> for BatchNo {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        BatchNo::new(value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(try_from = "String")]
pub struct ExpiryDateMs(NaiveDate);

impl ExpiryDateMs {
    pub fn new(value: String) -> anyhow::Result<Self> {
        let p = NaiveDate::parse_from_str(value.as_str(), "%Y-%m-%d").context("")?;
        Ok(ExpiryDateMs(p))
    }
    pub fn epoch_millis(&self) -> Option<i64> {
        self.0
            .and_hms_milli_opt(0, 0, 0, 0)
            .map(|a| a.timestamp_millis())
    }
}

impl TryFrom<String> for ExpiryDateMs {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ExpiryDateMs::new(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoicePdfRequest {
    pub tenant_id: Uuid,
    pub invoice_id: Uuid,
    pub invoice: Invoice,
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use lazy_static::lazy_static;
    use uuid::Uuid;

    use invoice_doc_generator::invoice_line::line_quantity::test_utils::{a_free_line_quantity, a_line_quantity};
    use invoice_doc_generator::invoice_line::line_title::LineTitle;
    use invoice_doc_generator::invoice_line::unit_price::Price;
    use invoice_doc_generator::percentages::tax_discount_cess::DiscountPercentage;

    use crate::accounting::currency::currency_models::tests::SEED_CURRENCY_ID;
    use crate::invoicing::invoice_template::invoice_template_models::tests::SEED_INVOICE_TEMPLATE_ID;
    use crate::invoicing::invoicing_request_models::{
        BillShipDetail, BillShipDetailBuilder, CreateAdditionalChargeRequest,
        CreateAdditionalChargeRequestBuilder, CreateInvoiceLineRequest,
        CreateInvoiceLineRequestBuilder, CreateInvoiceRequest, CreateInvoiceRequestBuilder,
    };
    use crate::invoicing::invoicing_series::invoicing_series_models::tests::SEED_INVOICING_SERIES_MST_ID;
    use crate::masters::business_entity_master::business_entity_models::tests::{
        SEED_BUSINESS_ENTITY_ID1, SEED_BUSINESS_ENTITY_ID2,
    };
    use crate::masters::product_item_master::product_item_models::tests::SEED_PRODUCT_ITEM_ID;

    lazy_static! {
        pub static ref SEED_INVOICE_ID: Uuid =
            Uuid::from_str("018d5559-745a-7371-80c6-a4efaa2cafe6").unwrap();
    }

    pub fn a_create_invoice_request(builder: CreateInvoiceRequestBuilder) -> CreateInvoiceRequest {
        CreateInvoiceRequest {
            idempotence_key: builder.idempotence_key.unwrap_or_else(Uuid::now_v7),
            invoice_template_id: builder
                .invoice_template_id
                .unwrap_or(*SEED_INVOICE_TEMPLATE_ID),
            invoicing_series_mst_id: builder
                .invoicing_series_mst_id
                .unwrap_or(*SEED_INVOICING_SERIES_MST_ID),
            currency_id: builder.currency_id.unwrap_or(*SEED_CURRENCY_ID),
            einvoicing_applicable: builder.einvoicing_applicable.unwrap_or(false),
            b2b_invoice: builder.b2b_invoice.unwrap_or(true),
            service_invoice: builder.service_invoice.unwrap_or(false),
            supplier_id: builder.supplier_id.unwrap_or(*SEED_BUSINESS_ENTITY_ID1),
            bill_ship_detail: builder
                .bill_ship_detail
                .unwrap_or_else(|| Some(a_bill_ship_detail(Default::default()))),
            order_number: builder.order_number.flatten(),
            order_date: builder.order_date.flatten(),
            payment_terms: builder.payment_terms.flatten(),
            invoice_lines: builder
                .invoice_lines
                .unwrap_or_else(|| vec![a_create_invoice_line_request(Default::default())]),
            additional_charges: builder
                .additional_charges
                .unwrap_or_else(|| vec![a_create_additional_charge_request(Default::default())]),
            invoice_remarks: builder.invoice_remarks.flatten(),
            ecommerce_gstin: builder.ecommerce_gstin.flatten(),
            dispatch_from_id: builder.dispatch_from_id.flatten(),
        }
    }

    pub fn a_bill_ship_detail(builder: BillShipDetailBuilder) -> BillShipDetail {
        BillShipDetail {
            billed_to_customer_id: builder
                .billed_to_customer_id
                .unwrap_or(*SEED_BUSINESS_ENTITY_ID2),
            shipped_to_customer_id: builder
                .shipped_to_customer_id
                .unwrap_or(*SEED_BUSINESS_ENTITY_ID2),
        }
    }

    pub fn a_create_additional_charge_request(
        builder: CreateAdditionalChargeRequestBuilder,
    ) -> CreateAdditionalChargeRequest {
        CreateAdditionalChargeRequest {
            line_title: builder
                .line_title
                .unwrap_or(LineTitle::new("some line title".to_string()).unwrap()),
            rate: builder.rate.unwrap_or_else(|| Price::new(0.0).unwrap()),
        }
    }

    pub fn a_create_invoice_line_request(
        builder: CreateInvoiceLineRequestBuilder,
    ) -> CreateInvoiceLineRequest {
        CreateInvoiceLineRequest {
            product_item_id: builder.product_item_id.unwrap_or(*SEED_PRODUCT_ITEM_ID),
            quantity: builder
                .quantity
                .unwrap_or_else(|| a_line_quantity(Default::default())),
            free_quantity: builder
                .free_quantity
                .unwrap_or_else(|| a_free_line_quantity(Default::default())),
            unit_price: builder
                .unit_price
                .unwrap_or_else(|| Price::new(10.0).unwrap()),
            discount_percentage: builder
                .discount_percentage
                .unwrap_or_else(|| DiscountPercentage::new(0.0).unwrap()),
            mrp: builder.mrp.flatten(),
            batch_no: builder.batch_no.flatten(),
            expiry_date: builder.expiry_date.flatten(),
            reverse_charge_applicable: builder.reverse_charge_applicable.unwrap_or(false),
        }
    }
}
