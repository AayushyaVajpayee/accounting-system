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
    pub month: u16,
    pub year: u16,
    pub day: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceLine {
    pub line_no: u16,
    pub item: String,
    pub hsn: String,
    pub batch_no: Option<String>,
    pub expiry_date: Option<DocDate>,
    pub mrp: Option<i32>,
    pub uqc: String,
    pub unit_price: i32,
    pub discount_percentage: i32,
    pub igst_percentage: Option<u16>,
    pub cgst_percentage: Option<u16>,
    pub sgst_percentage: Option<u16>,
    pub cess_percentage: Option<u16>,
    pub line_total: i32,
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
    pub taxable_amt: i32,
    pub tax_amt: i32,
    pub additional_charges_amt: i32,
    pub round_off: i32,
    pub total_payable_amount: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceLineTable {
   pub invoice_lines_total: i32,
   pub header_and_units: Vec<HeaderAndUnit>,
   pub lines: Vec<InvoiceLine>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaxLine {
    pub tax_slab: i32,
    pub tax_amount: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaxSummary {
    pub igst_lines: Vec<TaxLine>,
    pub cgst_lines: Vec<TaxLine>,
    pub sgst_lines: Vec<TaxLine>,
    pub total_tax_amount: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdditionalCharge {
    pub name: String,
    pub rate: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    pub line_1: String,
    pub line_2: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceParty {
    pub name: String,
    pub gstin: String,
    pub address: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Invoice {
    pub invoice_number: String,
    pub invoice_date: DocDate,
    pub order_date: Option<DocDate>,
    pub payment_term: String,
    pub order_number: Option<String>,
    pub irn_no: String,
    pub supplier: InvoiceParty,
    pub billed_to: InvoiceParty,
    pub shipped_to: InvoiceParty,
    pub additional_charges: Vec<AdditionalCharge>,
    pub tax_summary: TaxSummary,
    pub invoice_summary: InvoiceSummary,
    pub invoice_lines_table: InvoiceLineTable,
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