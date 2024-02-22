use std::collections::HashMap;
use std::fs;

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use typst::eval::Tracer;
use typst::foundations::Bytes;

use crate::world::InMemoryWorld;

const MAIN: &str = include_str!("../typst_templates/invoice/main.typ");
const INVOICE_LINES: &str = include_str!("../typst_templates/invoice/invoice_lines.typ");
const INVOICE_SUMMARY: &str = include_str!("../typst_templates/invoice/invoice_summary.typ");
const TAX_SUMMARY: &str = include_str!("../typst_templates/invoice/tax_summary.typ");
const SUNSET_PNG: &[u8] = include_bytes!("../typst_templates/invoice/sunset.png");
const EINVOICE_QR: &[u8] = include_bytes!("../typst_templates/invoice/einvoice_qr.png");
const TABLEX_PACKAGE_TYP: &[u8] =include_bytes!("../typst_templates/invoice/tablex.typ");
const TABLEX_TOML:&[u8] = include_bytes!("../typst_templates/invoice/typst.toml");
const JSON_DATA: &[u8] = include_bytes!("../typst_templates/invoice/invoice_data.json");

fn get_file_map(data:Vec<u8>) -> HashMap<&'static str, Bytes> {
    let entry_main = Bytes::from_static(MAIN.as_bytes());
    let entry_invoice_lines = Bytes::from_static(INVOICE_LINES.as_bytes());
    let entry_invoice_summary = Bytes::from_static(INVOICE_SUMMARY.as_bytes());
    let entry_tax_summary = Bytes::from_static(TAX_SUMMARY.as_bytes());
    let entry_sunset_png = Bytes::from_static(SUNSET_PNG);
    let entry_einvoice_qr = Bytes::from_static(EINVOICE_QR);
    let entry_tablex_package_typ = Bytes::from_static(TABLEX_PACKAGE_TYP);
    let entry_tablex_toml = Bytes::from_static(TABLEX_TOML);
    let entry_json_data = Bytes::from(data);
    let mut map = HashMap::new();
    let empty:Vec<u8> =Vec::new();
    map.insert("main.typ", entry_main);
    map.insert("invoice_lines.typ", entry_invoice_lines);
    map.insert("invoice_summary.typ", entry_invoice_summary);
    map.insert("tax_summary.typ", entry_tax_summary);
    map.insert("sunset.png", entry_sunset_png);
    map.insert("einvoice_qr.png", entry_einvoice_qr);
    map.insert("invoice_data.json", entry_json_data);
    map.insert("preview/tablex/0.0.8/tablex.typ",entry_tablex_package_typ);
    map.insert("preview/tablex/0.0.8/typst.toml",entry_tablex_toml);
    map.insert("preview/tablex/0.0.8",Bytes::from(empty));
    map
}

#[test]
fn test_pdf_creation() {
    let data = JSON_DATA.to_vec();
    let map = get_file_map(data);
    let world = InMemoryWorld::new(MAIN, map);
    let mut tracer = Tracer::default();
    let k = std::time::SystemTime::now();
    let document = typst::compile(&world, &mut tracer).expect("Error compiling typst.");
    let pdf = typst_pdf::pdf(&document, None, None);
    fs::write("./out220913.pdf", pdf).expect("Error writing PDF.");
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocDate {
    //todo derive from epoch millis or date. decide on one
    month: u16,
    year: u16,
    day: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceLine {
    line_no: u16,
    item: String,
    hsn: String,
    batch_no: Option<String>,
    expiry_date: Option<DocDate>,
    mrp: Option<i32>,
    uqc: String,
    unit_price: i32,
    discount_percentage: i32,
    igst_percentage: Option<u16>,
    cgst_percentage: Option<u16>,
    sgst_percentage: Option<u16>,
    cess_percentage: Option<u16>,
    line_total: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Header(String);

#[derive(Debug, Serialize, Deserialize)]
pub struct Unit(String);

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonKey(String);

#[derive(Debug, Serialize, Deserialize)]
pub struct HeaderAndUnit(Header, Unit, JsonKey);

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceSummary {
    taxable_amt: i32,
    tax_amt: i32,
    additional_charges_amt: i32,
    round_off: i32,
    total_payable_amount: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceLineTable {
    invoice_lines_total: i32,
    header_and_units: Vec<HeaderAndUnit>,
    lines: Vec<InvoiceLine>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaxLine {
    tax_slab: i32,
    tax_amount: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaxSummary {
    igst_lines: Vec<TaxLine>,
    cgst_lines: Vec<TaxLine>,
    sgst_lines: Vec<TaxLine>,
    total_tax_amount: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdditionalCharge {
    name: String,
    rate: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    line_1: String,
    line_2: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceParty {
    name: String,
    gstin: String,
    address: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Invoice {
    invoice_number: String,
    invoice_date: DocDate,
    order_date: Option<DocDate>,
    payment_term: String,
    order_number: Option<String>,
    irn_no: String,
    supplier: InvoiceParty,
    billed_to: InvoiceParty,
    shipped_to: InvoiceParty,
    additional_charges: Vec<AdditionalCharge>,
    tax_summary: TaxSummary,
    invoice_summary: InvoiceSummary,
    invoice_lines_table: InvoiceLineTable,
}


pub fn create_invoice_pdf(input: Invoice) ->
anyhow::Result<Vec<u8>> {
    let a =serde_json::to_vec(&input).context("error during serialisation")?;
    let map = get_file_map(a);
    let world = InMemoryWorld::new(MAIN, map);
    let mut tracer = Tracer::default();
    let k = std::time::SystemTime::now();
    let document = typst::compile(&world, &mut tracer)
        .map_err(|a|anyhow!("error during typst compilation"))?;
    let pdf = typst_pdf::pdf(&document, None, None);
    Ok(pdf)
}