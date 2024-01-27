use async_trait::async_trait;
use crate::invoicing::invoicing_request_models::CreateInvoiceRequest;

#[async_trait]
pub trait InvoicingService {
    fn create_invoice(req: &CreateInvoiceRequest);
}


struct InvoicingServiceImpl {}

impl InvoicingService for InvoicingServiceImpl {
    fn create_invoice(req: &CreateInvoiceRequest) {
        //validate
        //calculate invoice fields
    }
}
