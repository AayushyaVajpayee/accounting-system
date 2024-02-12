use std::ops::Add;

use anyhow::{Context, ensure};
use itertools::Itertools;

use invoicing_calculations::invoice_line::{compute_cess_amount, compute_discount_amount,
                                           compute_line_total_amount, compute_tax_amount,
                                           compute_taxable_amount, InvoiceLine};

use crate::invoicing::invoicing_request_models::{CreateAdditionalChargeRequest, CreateInvoiceLineRequest, CreateInvoiceRequest};

impl CreateInvoiceLineRequest {
    pub fn taxable_amount(&self) -> anyhow::Result<f64> {
        let line = InvoiceLine::new(self.quantity.get_quantity(),
                                    self.unit_price.inner(),
                                    self.discount_percentage.inner(),
                                    self.tax_rate_percentage.inner(),
                                    self.cess_percentage.inner())
            .context("error while calculating line taxable amount")?;
        Ok(compute_taxable_amount(&line))
    }
    pub fn tax_amount(&self) -> anyhow::Result<f64> {
        let line = InvoiceLine::new(self.quantity.get_quantity(),
                                    self.unit_price.inner(),
                                    self.discount_percentage.inner(),
                                    self.tax_rate_percentage.inner(),
                                    self.cess_percentage.inner())
            .context("error while calculating line tax amount")?;
        Ok(compute_tax_amount(&line))
    }
    pub fn cess_amount(&self) -> anyhow::Result<f64> {
        let line = InvoiceLine::new(self.quantity.get_quantity(),
                                    self.unit_price.inner(),
                                    self.discount_percentage.inner(),
                                    self.tax_rate_percentage.inner(),
                                    self.cess_percentage.inner())
            .context("error while calculating line cess amount")?;
//todo multiple cess calculation implementations.
//todo https://learn.microsoft.com/en-us/dynamics365/business-central/localfunctionality/india/gst-cess-calculations
        Ok(compute_cess_amount(&line))
    }
    pub fn net_line_total(&self) -> anyhow::Result<f64> {
        let line = InvoiceLine::new(self.quantity.get_quantity(),
                                    self.unit_price.inner(),
                                    self.discount_percentage.inner(),
                                    self.tax_rate_percentage.inner(),
                                    self.cess_percentage.inner())
            .context("error while calculating net line total amount")?;
        Ok(compute_line_total_amount(&line))
    }
    pub fn get_discount_amount(&self) -> anyhow::Result<f64> {
        let line = InvoiceLine::new(self.quantity.get_quantity(),
                                    self.unit_price.inner(),
                                    self.discount_percentage.inner(),
                                    self.tax_rate_percentage.inner(),
                                    self.cess_percentage.inner())
            .context("error while calculating line discount amount")?;
        Ok(compute_discount_amount(&line))
    }
}



impl CreateInvoiceRequest {
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

    pub fn total_amount(&self,scale:u32) -> anyhow::Result<f64> {
        ensure!(scale<=7,"only scale less than {},is supported but was {}",7,scale);
        let amt = self.total_taxable_amount()?
            + self.total_tax_amount()?
            + self.total_cess_amount()?
            + self.total_additional_charge_amount();
        let s = 10_i32.pow(scale) as f64;
        let rounded_amt =(amt*s).round()/s;
        Ok(rounded_amt)
    }
}