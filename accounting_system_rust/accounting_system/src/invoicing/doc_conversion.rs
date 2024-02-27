use std::sync::Arc;
use anyhow::anyhow;
use chrono::{Datelike, NaiveDate, NaiveDateTime};
use pdf_doc_generator::invoice_template::{Address, DocDate, Invoice, InvoiceParty};
use crate::invoicing::invoicing_dao_models::InvoiceDb;
use crate::masters::business_entity_master::business_entity_models::BusinessEntityMaster;

pub fn convert_to_invoice_doc_model(invoice: InvoiceDb,
                                    invoice_number: String,
                                    supplier:Arc<BusinessEntityMaster>
)
    -> anyhow::Result<Invoice> {
    Ok(Invoice {
        invoice_number,
        invoice_date: epoch_ms_to_doc_date(invoice.invoice_date_ms)?,
        order_date: invoice.order_date.map(epoch_ms_to_doc_date).transpose()?,
        payment_term: "".to_string(),//todo how to derive correct payment term
        order_number: invoice.order_number.map(|a|a.to_string()),
        irn_no: "".to_string(),//todo without einvoicing this is garbage. how to derive it
        supplier:InvoiceParty{
            name: supplier.entity_type.get_name().to_string(),
            gstin: supplier.entity_type.extract_gstin()
                .map(|a|a.get_str().to_string())
                .ok_or_else(||anyhow!("gstin mandatory for supplier but was none"))?,
            address: Address {
                line_1: "".to_string(),
                line_2: "".to_string(),
            },
        },
        billed_to: (),
        shipped_to: (),
        additional_charges: vec![],
        tax_summary: (),
        invoice_summary: (),
        invoice_lines_table: (),
    })
}

fn epoch_ms_to_doc_date(epoch_ms: i64) -> anyhow::Result<DocDate> {
    let jp = NaiveDateTime::from_timestamp_millis(epoch_ms)
        .ok_or_else(|| anyhow!("error parsing date"))?;
    Ok(DocDate {
        month: jp.month() as u16,
        year: jp.year() as u16,
        day: jp.day() as u16,
    })
}