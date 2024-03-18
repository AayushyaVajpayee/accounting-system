use std::sync::Arc;

use anyhow::{anyhow, Context};
use chrono::{Datelike, NaiveDateTime};
use itertools::Itertools;
use uuid::Uuid;

use invoicing_calculations::invoice_line::{compute_tax_amount, InvoiceLine};
use pdf_doc_generator::invoice_template::{AdditionalCharge, Address, DocDate, Invoice, InvoiceLineTable, InvoiceParty, InvoiceSummary, InvoiceTableHeaderNameEnum, TaxLine, TaxSummary};
use pdf_doc_generator::invoice_template::InvoiceTableHeaderNameEnum::{Cgst, ItemDescription, LineTotal, Qty, SerialNo, Sgst, UnitPrice, Uqc};

use crate::accounting::currency::currency_models::CurrencyMaster;
use crate::invoicing::invoicing_dao_models::{InvoiceDb, InvoiceLineDb};
use crate::masters::business_entity_master::business_entity_models::BusinessEntityDto;
use crate::masters::business_entity_master::business_entity_service::BusinessEntityService;

pub async fn convert_to_invoice_doc_model<'a>(invoice: &InvoiceDb<'a>,
                                              invoice_number: String,
                                              business_entity_service: Arc<dyn BusinessEntityService>,
                                              currency: Arc<CurrencyMaster>,
)
                                              -> anyhow::Result<Invoice> {
    let supplier = fetch_business_entity(Some(invoice.supplier_id), invoice.tenant_id, business_entity_service.clone())
        .await?
        .context("supplier cannot be null")?;
    let billed_to = fetch_business_entity(invoice.billed_to_customer_id, invoice.tenant_id, business_entity_service.clone())
        .await?;
    let shipped_to = fetch_business_entity(invoice.shipped_to_customer_id, invoice.tenant_id, business_entity_service.clone())
        .await?;
    Ok(Invoice {
        invoice_number,
        invoice_date: epoch_ms_to_doc_date(invoice.invoice_date_ms)?,
        order_date: invoice.order_date.map(epoch_ms_to_doc_date).transpose()?,
        payment_term: "".to_string(),//todo how to derive correct payment term. credit, cash, advance,etc?
        order_number: invoice.order_number.map(|a| a.to_string()),
        irn_no: "".to_string(),//todo without einvoicing this is garbage. how to derive it
        service_invoice:invoice.service_invoice,
        supplier: convert_business_entity_to_invoice_party(supplier),
        billed_to: billed_to.map(|a| convert_business_entity_to_invoice_party(a)),
        shipped_to: shipped_to.map(|a| convert_business_entity_to_invoice_party(a)),
        additional_charges: invoice.additional_charges
            .iter()
            .map(|a| AdditionalCharge {
                name: a.line_title.to_string(),
                rate: a.rate,
            }).collect_vec(),
        tax_summary: create_invoice_tax_summary(&invoice)?,
        invoice_summary: create_invoice_summary(&invoice),
        invoice_lines_table: create_invoice_line_table(&invoice, currency)?,
    })
}

fn create_invoice_line_table(invoice: &InvoiceDb, currency: Arc<CurrencyMaster>) -> anyhow::Result<InvoiceLineTable> {
    Ok(InvoiceLineTable {
        invoice_lines_total: invoice.invoice_lines
            .iter()
            .map(|a| a.line_net_total)
            .sum(),
        header_and_units: get_header_and_units(&invoice, currency),
        lines: invoice.invoice_lines.iter().map(|a|
            convert_db_line_to_doc_line(a, invoice.igst_applicable)
        ).collect::<anyhow::Result<Vec<_>>>()?,
    })
}

fn create_invoice_summary(invoice: &InvoiceDb) -> InvoiceSummary {
    InvoiceSummary {
        taxable_amt: invoice.total_taxable_amount,
        tax_amt: invoice.total_tax_amount,
        additional_charges_amt: invoice.total_additional_charges_amount,
        round_off: invoice.round_off,
        total_payable_amount: invoice.total_payable_amount,
    }
}

fn create_invoice_tax_summary(invoice: &InvoiceDb) -> anyhow::Result<TaxSummary> {
    if invoice.igst_applicable {
        Ok(
            TaxSummary {
                igst_lines: convert_to_tax_lines(&invoice.invoice_lines, invoice.igst_applicable)?,
                cgst_lines: vec![],
                sgst_lines: vec![],
                total_tax_amount: invoice.total_tax_amount,
            }
        )
    } else {
        Ok(TaxSummary {
            igst_lines: vec![],
            cgst_lines: convert_to_tax_lines(&invoice.invoice_lines, invoice.igst_applicable)?,
            sgst_lines: convert_to_tax_lines(&invoice.invoice_lines, invoice.igst_applicable)?,
            total_tax_amount: invoice.total_tax_amount,
        })
    }
}


impl InvoiceDb<'_> {
    fn hsn_sac_header(&self) -> Option<InvoiceTableHeaderNameEnum> {
        if self.service_invoice {
            Some(InvoiceTableHeaderNameEnum::Sac("".to_string()))
        } else {
            Some(InvoiceTableHeaderNameEnum::Hsn("".to_string()))
        }
    }
    fn batch_no_header(&self) -> Option<InvoiceTableHeaderNameEnum> {
        if self.invoice_lines
            .iter()
            .all(|a| a.batch_no.is_none()) {
            None
        } else {
            Some(InvoiceTableHeaderNameEnum::BatchNo("".to_string()))
        }
    }
    fn expiry_date_header(&self) -> Option<InvoiceTableHeaderNameEnum> {
        if self.invoice_lines
            .iter()
            .all(|a| a.expiry_date_ms.is_none()) {
            None
        } else {
            Some(InvoiceTableHeaderNameEnum::ExpiryDate("".to_string()))
        }
    }
    fn mrp_header(&self, currency_unit: String) -> Option<InvoiceTableHeaderNameEnum> {
        if self.invoice_lines
            .iter()
            .all(|a| a.mrp.is_none()) {
            None
        } else {
            Some(InvoiceTableHeaderNameEnum::Mrp(currency_unit))
        }
    }
    fn discount_percentage_header(&self) -> Option<InvoiceTableHeaderNameEnum> {
        if self.invoice_lines
            .iter()
            .all(|a| a.discount_percentage == 0.0) {
            None
        } else {
            Some(InvoiceTableHeaderNameEnum::Discount("%".to_string()))
        }
    }
    fn igst_header(&self) -> Option<InvoiceTableHeaderNameEnum> {
        if self.igst_applicable && self.invoice_lines.iter().any(|a| a.tax_percentage != 0.0) {
            Some(InvoiceTableHeaderNameEnum::Igst("%".to_string()))
        } else {
            None
        }
    }

    fn cgst_header(&self) -> Option<InvoiceTableHeaderNameEnum> {
        let po = Cgst("%".to_string());
        self.cgst_sgst_header(po)
    }
    fn cgst_sgst_header(&self, h: InvoiceTableHeaderNameEnum) -> Option<InvoiceTableHeaderNameEnum> {
        // assert!(h==Cgst||h==Sgst,"input arg can only be one Sgst or Cgst but was {h}");
        if !self.igst_applicable &&
            self.invoice_lines.iter().any(|a| a.tax_percentage != 0.0) {
            Some(h)
        } else { None }
    }
    fn sgst_header(&self) -> Option<InvoiceTableHeaderNameEnum> {
        let po = Sgst("%".to_string());
        self.cgst_sgst_header(po)
    }
    fn cess_header(&self) -> Option<InvoiceTableHeaderNameEnum> {
        if self.invoice_lines.iter()
            .all(|l| l.cess_percentage == 0.0) {
            None
        } else {
            Some(InvoiceTableHeaderNameEnum::Cess("%".to_string()))
        }
    }
}

fn get_header_and_units(invoice: &InvoiceDb, currency_master: Arc<CurrencyMaster>) -> Vec<InvoiceTableHeaderNameEnum> {
    let curr_unit = currency_master.display_name.as_str();
    vec![
        Some(SerialNo("".to_string())),
        Some(ItemDescription("".to_string())),
        invoice.hsn_sac_header(),
        invoice.batch_no_header(),
        invoice.expiry_date_header(),
        invoice.mrp_header(curr_unit.to_string()),
        Some(Uqc("".to_string())),
        Some(Qty("".to_string())),
        Some(UnitPrice(curr_unit.to_string())),
        invoice.discount_percentage_header(),
        invoice.igst_header(),
        invoice.cgst_header(),
        invoice.sgst_header(),
        invoice.cess_header(),
        Some(LineTotal(curr_unit.to_string())),
    ].into_iter().flatten().collect_vec()
}

fn convert_db_line_to_doc_line(a: &InvoiceLineDb, igst_applicable: bool) -> anyhow::Result<pdf_doc_generator::invoice_template::InvoiceLine> {
    let line = pdf_doc_generator::invoice_template::InvoiceLine {
        line_no: a.line_no as u16,
        item: a.line_title.to_string(),
        hsn_sac: a.hsn_sac_code.to_string(),
        batch_no: a.batch_no.map(|b| b.to_string()),
        expiry_date: a.expiry_date_ms.map(|e| epoch_ms_to_doc_date(e)).transpose()?,
        mrp: a.mrp,
        quantity: a.quantity,
        uqc: a.uqc.to_string(),
        unit_price: a.unit_price,
        discount_percentage: a.discount_percentage,
        igst_percentage: if igst_applicable { a.tax_percentage } else { 0.0 },
        cgst_percentage: if igst_applicable { 0.0 } else { a.tax_percentage / 2.0 },
        sgst_percentage: if igst_applicable { 0.0 } else { a.tax_percentage / 2.0 },
        cess_percentage: a.cess_percentage,
        line_total: a.line_net_total,
    };
    Ok(line)
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

async fn fetch_business_entity(id: Option<Uuid>, tenant_id: Uuid, service: Arc<dyn BusinessEntityService>) -> anyhow::Result<Option<Arc<BusinessEntityDto>>> {
    if let Some(id) = id {
        let entity = service.get_business_entity_by_id(&id, &tenant_id).await?;
        Ok(entity)
    } else {
        Ok(None)
    }
}

fn convert_to_tax_lines(lines: &Vec<InvoiceLineDb>, igst_applicable: bool) -> anyhow::Result<Vec<TaxLine>> {
    let lines_and_tax_amt: Vec<(&InvoiceLineDb, f64)> = lines.iter()
        .map(|l|
            {
                let line = InvoiceLine::new(
                    l.quantity,
                    l.unit_price,
                    l.discount_percentage,
                    l.tax_percentage,
                    l.cess_percentage,
                );
                line.map(|a| (l, compute_tax_amount(&a)))
            }).try_collect()?;
    let tax_lines = lines_and_tax_amt.iter()
        .group_by(|a| a.0.tax_percentage)
        .into_iter()
        .map(|(tax_percentage, lines)| {
            let total_tax_amt: f64 = lines.into_iter()
                .map(|(_, tax_amt)| tax_amt)
                .sum();
            TaxLine {
                tax_slab: if igst_applicable { tax_percentage } else { tax_percentage / 2.0 },
                tax_amount: if igst_applicable { total_tax_amt } else { total_tax_amt / 2.0 },
            }
        }).collect_vec();
    Ok(tax_lines)
}

fn convert_business_entity_to_invoice_party(e: Arc<BusinessEntityDto>) -> InvoiceParty {
    InvoiceParty {
        name: e.business_entity.entity_type.get_name().to_string(),
        gstin: e.business_entity.entity_type.extract_gstin()
            .map(|gstin| gstin.get_str().to_string())
            .unwrap_or_else(|| "".to_string()),
        address: e.address.as_ref().map(|add| Address {
            line_1: add.address.line_1.get_inner().to_string(),
            line_2: add.address.line_2.as_ref()
                .map(|a| a.get_inner().to_string())
                .unwrap_or_else(|| "".to_string()),
            city_name:add.city.city_name.inner().to_string(),
            pincode:add.pincode.pincode.to_string(),
            gst_state_code:add.state.state_code.clone()
        }).unwrap_or_else(|| Address {
            line_1: "".to_string(),
            line_2: "".to_string(),
            city_name: "".to_string(),
            pincode: "".to_string(),
            gst_state_code: "".to_string(),
        }),
    }
}