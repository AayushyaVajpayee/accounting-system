flows

## create invoice

    create request ---> (1) validate request params -> (2) compute invoice fields
        --> (3) go to db and store it or retrieve existing --> (4) einvoicing if applicable -> (5) store einvoicing response in s3
        and update db --> (6) generate invoice docs -->(7) store them in s3 -> (8) update db 

we will use self-hosted temporal. temporal go lets use for less memory requirements. we have to be very cheap. also load
wont be much
so let's hope we can deliver. Also, our temporal and main postgres server will be same with different db. This will be
cheaper I guess.

#### Create Request body

###### CreateInvoiceRequest body

1. idempotency_key-->uuid
2. tenant_id-->uuid
3. invoice_number (this we will generate and cannot be in invoice)
4. currency_id-->uuid
5. service_invoice-->boolean
6. supplier_business_entity_id or register supplier business request-->enum of either existing id or new request
7. b2b_invoice --> boolean
8. billed_to_business_entity_id --> mandatorily applicable if its a b2b invoice or if in case of b2c we have data of
   customer
9. shipped_to_business_entity_id --> mandatorily applicable if its a b2b invoice or if in case of b2d we have data of
   customer
10. purchase_order_number --> if available
11. purchase_order_date --> if available
12. create_invoice_lines_request
13. create_additional_charges_request
14. payment_terms

###### CreateInvoiceLineRequest body

1. line_no --> optional<u16>
2. hsn_sac_code -->if service invoice, then sac code and if goods invoice then hsn code.
3. item_description --> Should not be more than our specified limit and should not be blank either
4. quantity --> Quantity should be greater than 0 and less than some threshold
5. uqc --> this can be for now enum
6. unit_price --> this should be greater than 0 and less than a certain threshold for now
7. tax_rate_bps --> should be gst slabs and nothing else.
8. discount_bps --> only allow discount +-(100)% and not more than that for now.
9. cess_bps -->  allow greater than 0 but less than 500%
10. mrp --> optional but should be greater than 0 and less than certain threshold
11. batch_no --> optional lets keep it less than equal to 20 chars
12. expiry_date --> optional just be a valid date

###### CreateAdditionalCharges body

1. line_no -->optional<u16>
2. item_description -->Should not be more than our specified limit and should not be blank either
3. rate --> this should be greater than 0 and less than a certain threshold for now