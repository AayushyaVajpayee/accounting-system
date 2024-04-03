use std::ops::Add;

use anyhow::{Context, ensure};
use itertools::Itertools;

use invoicing_calculations::invoice_line::InvoiceLine;

use crate::invoicing::invoicing_request_models::{CreateInvoiceLineRequestWithAllDetails, CreateInvoiceWithAllDetailsIncluded};

impl TryFrom<&CreateInvoiceLineRequestWithAllDetails> for InvoiceLine{
    type Error = anyhow::Error;

    fn try_from(value: &CreateInvoiceLineRequestWithAllDetails) -> Result<Self, Self::Error> {
        Ok(InvoiceLine::new(value.quantity.get_quantity(),
                         value.unit_price.inner(),
                         value.discount_percentage.inner(),
                         value.product_item_id.get_tax_rate()?.tax_rate_percentage.inner(),
                         value.product_item_id.get_cess_rate()?.cess_strategy.clone())
            .context("error while calculating line taxable amount")?)
    }
}
impl CreateInvoiceLineRequestWithAllDetails{
    pub fn taxable_amount(&self) -> anyhow::Result<f64> {
        let line:InvoiceLine =self.try_into()?; 
        Ok(line.compute_taxable_amount())
    }

    pub fn tax_amount(&self) -> anyhow::Result<f64> {
        let line:InvoiceLine =self.try_into()?;
        Ok(line.compute_tax_amount())
    }

    pub fn cess_amount(&self) -> anyhow::Result<f64> {
        let line:InvoiceLine =self.try_into()?;
        Ok(line.compute_cess_amount())
    }

    pub fn net_line_total(&self) -> anyhow::Result<f64> {
        let line:InvoiceLine =self.try_into()?;
        Ok(line.compute_line_total_amount())
    }
    pub fn get_discount_amount(&self) -> anyhow::Result<f64> {
        let line:InvoiceLine =self.try_into()?;
        Ok(line.compute_discount_amount())
    }
}

impl CreateInvoiceWithAllDetailsIncluded{
    pub fn total_taxable_amount(&self) -> anyhow::Result<f64> {
        self.invoice_lines.iter()
            .map(|line| line.taxable_amount())
            .fold_ok(0.0, Add::add)
    }

    pub fn total_tax_amount(&self) -> anyhow::Result<f64> {
        self.invoice_lines.iter()
            .map(|line| line.tax_amount())
            .fold_ok(0.0, Add::add)
    }

    pub fn total_additional_charge_amount(&self) -> f64 {
        self.additional_charges.iter()
            .map(|ch| ch.rate.inner())
            .fold(0.0, Add::add)
    }

    pub fn total_cess_amount(&self) -> anyhow::Result<f64> {
        self.invoice_lines.iter()
            .map(|a| a.cess_amount())
            .fold_ok(0.0, Add::add)
    }

    pub fn total_amount(&self, scale: i16) -> anyhow::Result<f64> {
        ensure!(scale<=7,"only scale less than {},is supported but was {}",7,scale);
        let amt = self.total_taxable_amount()?
            + self.total_tax_amount()?
            + self.total_cess_amount()?
            + self.total_additional_charge_amount();
        let s = 10_i32.pow(scale as u32) as f64;
        let rounded_amt = (amt * s).round() / s;
        Ok(rounded_amt)
    }
}