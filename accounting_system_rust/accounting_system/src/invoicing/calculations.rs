use anyhow::Context;

use invoicing_calculations::invoice_line::{compute_cess_amount, compute_discount_amount,
                                           compute_line_total_amount, compute_tax_amount,
                                           compute_taxable_amount, InvoiceLine};

use crate::invoicing::invoicing_request_models::CreateInvoiceLineRequest;

impl CreateInvoiceLineRequest {
    #[allow(dead_code)]
    pub fn calculate_taxable_amount(&self) -> anyhow::Result<f64> {
        let line = InvoiceLine::new(self.quantity.get_quantity(),
                                    self.unit_price.inner(),
                                    self.discount_percentage.inner(),
                                    self.tax_rate_percentage.inner() ,
                                    self.cess_percentage.inner())
            .context("error while calculating line taxable amount")?;
        Ok(compute_taxable_amount(&line))
    }
    #[allow(dead_code)]
    pub fn calculate_tax_amount(&self) -> anyhow::Result<f64> {
        let line = InvoiceLine::new(self.quantity.get_quantity(),
                                    self.unit_price.inner(),
                                    self.discount_percentage.inner(),
                                    self.tax_rate_percentage.inner(),
                                    self.cess_percentage.inner())
            .context("error while calculating line tax amount")?;
        Ok(compute_tax_amount(&line))
    }
    #[allow(dead_code)]
    pub fn calculate_cess_amount(&self) -> anyhow::Result<f64> {
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
    #[allow(dead_code)]
    pub fn calculate_net_line_total(&self) -> anyhow::Result<f64> {
        let line = InvoiceLine::new(self.quantity.get_quantity(),
                                    self.unit_price.inner(),
                                    self.discount_percentage.inner(),
                                    self.tax_rate_percentage.inner(),
                                    self.cess_percentage.inner())
            .context("error while calculating net line total amount")?;
        Ok(compute_line_total_amount(&line))
    }
    #[allow(dead_code)]
    pub fn get_discount(&self) -> anyhow::Result<f64> {
        let line = InvoiceLine::new(self.quantity.get_quantity(),
                                    self.unit_price.inner(),
                                    self.discount_percentage.inner(),
                                    self.tax_rate_percentage.inner() ,
                                    self.cess_percentage.inner())
            .context("error while calculating line discount amount")?;
        Ok(compute_discount_amount(&line))
    }
}

