use std::collections::HashMap;
use std::fmt;

use anyhow::{anyhow, Context};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use typst::eval::Tracer;
use typst::foundations::Bytes;

use crate::world::InMemoryWorld;

const MAIN: &str = include_str!("../typst_templates/invoice/main.typ");
const INVOICE_LINES: &str = include_str!("../typst_templates/invoice/invoice_lines.typ");
const INVOICE_SUMMARY: &str = include_str!("../typst_templates/invoice/invoice_summary.typ");
const TAX_SUMMARY: &str = include_str!("../typst_templates/invoice/tax_summary.typ");
const SUNSET_PNG: &[u8] = include_bytes!("../typst_templates/invoice/sunset.png");
const EINVOICE_QR: &[u8] = include_bytes!("../typst_templates/invoice/einvoice_qr.png");
const TABLEX_PACKAGE_TYP: &[u8] = include_bytes!("../typst_templates/invoice/tablex.typ");
const TABLEX_TOML: &[u8] = include_bytes!("../typst_templates/invoice/typst.toml");

fn get_file_map(data: Vec<u8>) -> HashMap<&'static str, Bytes> {
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
    let empty: Vec<u8> = Vec::new();
    map.insert("main.typ", entry_main);
    map.insert("invoice_lines.typ", entry_invoice_lines);
    map.insert("invoice_summary.typ", entry_invoice_summary);
    map.insert("tax_summary.typ", entry_tax_summary);
    map.insert("sunset.png", entry_sunset_png);
    map.insert("einvoice_qr.png", entry_einvoice_qr);
    map.insert("invoice_data.json", entry_json_data);
    map.insert("preview/tablex/0.0.8/tablex.typ", entry_tablex_package_typ);
    map.insert("preview/tablex/0.0.8/typst.toml", entry_tablex_toml);
    map.insert("preview/tablex/0.0.8", Bytes::from(empty));
    map
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
    pub hsn_sac: String,
    pub batch_no: Option<String>,
    pub expiry_date: Option<DocDate>,
    pub mrp: Option<f32>,
    pub quantity: f64,
    pub free_quantity: f64,
    pub uqc: String,
    pub unit_price: f64,
    pub discount_percentage: f32,
    pub igst_percentage: f32,
    pub cgst_percentage: f32,
    pub sgst_percentage: f32,
    pub cess_percentage: f32,
    pub line_total: f64,
    pub reverse_charge_applicable: bool,
}

#[derive(Debug, Serialize)]
pub struct Header(pub &'static str);

#[derive(Debug, Serialize)]
pub struct Unit(pub String);

#[derive(Debug, Serialize)]
pub struct JsonKey(pub &'static str);

#[derive(Debug, Serialize)]
pub struct HeaderAndUnit(pub Header, pub Unit, pub JsonKey);

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceSummary {
    pub taxable_amt: f64,
    pub tax_amt: f64,
    pub additional_charges_amt: f64,
    pub round_off: f64,
    pub total_payable_amount: f64,
}

#[derive(Debug, PartialEq)]
///(currency display unit stored as string)
pub enum InvoiceTableHeaderNameEnum {
    SerialNo(String),
    ItemDescription(String),
    Hsn(String),
    Sac(String),
    BatchNo(String),
    ExpiryDate(String),
    Mrp(String),
    Uqc(String),
    Qty(String),
    UnitPrice(String),
    Discount(String),
    Igst(String),
    Cgst(String),
    Sgst(String),
    Cess(String),
    LineTotal(String),
}

impl Serialize for InvoiceTableHeaderNameEnum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element(self.header_display_name())?;
        seq.serialize_element(self.display_unit())?;
        seq.serialize_element(self.json_key())?;
        seq.end()
    }
}

impl<'de> Deserialize<'de> for InvoiceTableHeaderNameEnum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        struct EnumVisitor;

        impl<'de> Visitor<'de> for EnumVisitor {
            type Value = InvoiceTableHeaderNameEnum;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of three strings")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
            {
                let header_display_name: &str = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let currency_unit: String = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                let _: &str = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;

                InvoiceTableHeaderNameEnum::from_str(header_display_name, currency_unit)
                    .map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_seq(EnumVisitor)
    }
}

impl InvoiceTableHeaderNameEnum {
    fn from_str(
        header_display_name: &str,
        currency_unit: String,
    ) -> Result<Self, String> {
        match header_display_name {
            "sl no" => Ok(InvoiceTableHeaderNameEnum::SerialNo(currency_unit)),
            "item" => Ok(InvoiceTableHeaderNameEnum::ItemDescription(currency_unit)),
            "hsn" => Ok(InvoiceTableHeaderNameEnum::Hsn(currency_unit)),
            "sac" => Ok(InvoiceTableHeaderNameEnum::Sac(currency_unit)),
            "batch_no" => Ok(InvoiceTableHeaderNameEnum::BatchNo(currency_unit)),
            "expiry_date" => Ok(InvoiceTableHeaderNameEnum::ExpiryDate(currency_unit)),
            "mrp" => Ok(InvoiceTableHeaderNameEnum::Mrp(currency_unit)),
            "uqc" => Ok(InvoiceTableHeaderNameEnum::Uqc(currency_unit)),
            "qty" => Ok(InvoiceTableHeaderNameEnum::Qty(currency_unit)),
            "unit price" => Ok(InvoiceTableHeaderNameEnum::UnitPrice(currency_unit)),
            "discount" => Ok(InvoiceTableHeaderNameEnum::Discount(currency_unit)),
            "igst" => Ok(InvoiceTableHeaderNameEnum::Igst(currency_unit)),
            "cgst" => Ok(InvoiceTableHeaderNameEnum::Cgst(currency_unit)),
            "sgst" => Ok(InvoiceTableHeaderNameEnum::Sgst(currency_unit)),
            "cess" => Ok(InvoiceTableHeaderNameEnum::Cess(currency_unit)),
            "line total" => Ok(InvoiceTableHeaderNameEnum::LineTotal(currency_unit)),
            _ => Err(format!(
                "Invalid combination of header_display_name '{}'",
                header_display_name
            )),
        }
    }
}

impl InvoiceTableHeaderNameEnum {
    fn header_display_name(&self) -> &'static str {
        match &self {
            InvoiceTableHeaderNameEnum::SerialNo(_) => { "sl no" }
            InvoiceTableHeaderNameEnum::ItemDescription(_) => { "item" }
            InvoiceTableHeaderNameEnum::Hsn(_) => { "hsn" }
            InvoiceTableHeaderNameEnum::Sac(_) => { "sac" }
            InvoiceTableHeaderNameEnum::BatchNo(_) => { "batch_no" }
            InvoiceTableHeaderNameEnum::ExpiryDate(_) => { "expiry_date" }
            InvoiceTableHeaderNameEnum::Mrp(_) => { "mrp" }
            InvoiceTableHeaderNameEnum::Uqc(_) => { "uqc" }
            InvoiceTableHeaderNameEnum::Qty(_) => { "qty" }
            InvoiceTableHeaderNameEnum::UnitPrice(_) => { "unit price" }
            InvoiceTableHeaderNameEnum::Discount(_) => { "discount" }
            InvoiceTableHeaderNameEnum::Igst(_) => { "igst" }
            InvoiceTableHeaderNameEnum::Cgst(_) => { "cgst" }
            InvoiceTableHeaderNameEnum::Sgst(_) => { "sgst" }
            InvoiceTableHeaderNameEnum::Cess(_) => { "cess" }
            InvoiceTableHeaderNameEnum::LineTotal(_) => { "line total" }
        }
    }
    fn display_unit(&self) -> &str {
        match &self {
            InvoiceTableHeaderNameEnum::SerialNo(a) => { a }
            InvoiceTableHeaderNameEnum::ItemDescription(a) => { a }
            InvoiceTableHeaderNameEnum::Hsn(a) => { a }
            InvoiceTableHeaderNameEnum::Sac(a) => { a }
            InvoiceTableHeaderNameEnum::BatchNo(a) => { a }
            InvoiceTableHeaderNameEnum::ExpiryDate(a) => { a }
            InvoiceTableHeaderNameEnum::Mrp(a) => { a }
            InvoiceTableHeaderNameEnum::Uqc(a) => { a }
            InvoiceTableHeaderNameEnum::Qty(a) => { a }
            InvoiceTableHeaderNameEnum::UnitPrice(a) => { a }
            InvoiceTableHeaderNameEnum::Discount(a) => { a }
            InvoiceTableHeaderNameEnum::Igst(a) => { a }
            InvoiceTableHeaderNameEnum::Cgst(a) => { a }
            InvoiceTableHeaderNameEnum::Sgst(a) => { a }
            InvoiceTableHeaderNameEnum::Cess(a) => { a }
            InvoiceTableHeaderNameEnum::LineTotal(a) => { a }
        }
    }
    fn json_key(&self) -> &'static str {
        match &self {
            InvoiceTableHeaderNameEnum::SerialNo(_) => { "line_no" }
            InvoiceTableHeaderNameEnum::ItemDescription(_) => { "item" }
            InvoiceTableHeaderNameEnum::Hsn(_) => { "hsn_sac" }
            InvoiceTableHeaderNameEnum::Sac(_) => { "hsn_sac" }
            InvoiceTableHeaderNameEnum::BatchNo(_) => { "batch_no" }
            InvoiceTableHeaderNameEnum::ExpiryDate(_) => { "expiry_date" }
            InvoiceTableHeaderNameEnum::Mrp(_) => { "mrp" }
            InvoiceTableHeaderNameEnum::Uqc(_) => { "uqc" }
            InvoiceTableHeaderNameEnum::Qty(_) => { "quantity" }
            InvoiceTableHeaderNameEnum::UnitPrice(_) => { "unit_price" }
            InvoiceTableHeaderNameEnum::Discount(_) => { "discount_percentage" }
            InvoiceTableHeaderNameEnum::Igst(_) => { "igst_percentage" }
            InvoiceTableHeaderNameEnum::Cgst(_) => { "cgst_percentage" }
            InvoiceTableHeaderNameEnum::Sgst(_) => { "sgst_percentage" }
            InvoiceTableHeaderNameEnum::Cess(_) => { "cess_percentage" }
            InvoiceTableHeaderNameEnum::LineTotal(_) => { "line_total" }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceLineTable {
    pub invoice_lines_total: f64,
    pub header_and_units: Vec<InvoiceTableHeaderNameEnum>,
    pub lines: Vec<InvoiceLine>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaxLine {
    pub tax_slab: f32,
    pub tax_amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaxSummary {
    pub igst_lines: Vec<TaxLine>,
    pub cgst_lines: Vec<TaxLine>,
    pub sgst_lines: Vec<TaxLine>,
    pub total_tax_amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdditionalCharge {
    pub name: String,
    pub rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    pub line_1: String,
    pub line_2: String,
    pub city_name: String,
    pub pincode: String,
    pub gst_state_code: String,
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
    pub service_invoice: bool,
    pub irn_no: String,
    pub supplier: InvoiceParty,
    pub dispatch_from: Option<InvoiceParty>,
    pub billed_to: Option<InvoiceParty>,
    pub shipped_to: Option<InvoiceParty>,
    pub additional_charges: Vec<AdditionalCharge>,
    pub tax_summary: TaxSummary,
    pub invoice_summary: InvoiceSummary,
    pub invoice_lines_table: InvoiceLineTable,
    pub invoice_remarks:Option<String>,
    pub ecommerce_gstin:Option<String>,
}


pub fn create_invoice_pdf(input: Invoice) ->
anyhow::Result<Vec<u8>> {
    let a = serde_json::to_vec(&input).context("error during serialisation")?;
    let map = get_file_map(a);
    let world = InMemoryWorld::new(MAIN, map);
    let mut tracer = Tracer::default();
    let _k = std::time::SystemTime::now();
    let document = typst::compile(&world, &mut tracer)
        .map_err(|_a| anyhow!("error during typst compilation"))?;
    let pdf = typst_pdf::pdf(&document, None, None);
    //invoice creation does not have that much reusable data. also this evicts all cache everywhere
    comemo::evict(0);
    Ok(pdf)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use typst::eval::Tracer;

    use crate::invoice_template::{get_file_map, InvoiceTableHeaderNameEnum, MAIN};
    use crate::world::InMemoryWorld;

    const JSON_DATA: &[u8] = include_bytes!("../typst_templates/invoice/invoice_data.json");

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

    #[test]
    fn test_serialization_and_deserialization() {
        let a = InvoiceTableHeaderNameEnum::Discount("%".to_string());
        let po = serde_json::to_string(&a).unwrap();
        assert_eq!(po, r#"["discount","%","discount_percentage"]"#);
        let j: InvoiceTableHeaderNameEnum = serde_json::from_str(&po).unwrap();
        assert_eq!(a, j);
    }
}

