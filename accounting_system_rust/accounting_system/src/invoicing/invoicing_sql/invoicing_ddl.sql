CREATE TYPE cess_calculation_strategy AS ENUM ('percentage_of_assessable_value','amount_per_unit',
    'percentage_of_assessable_value_and_amount_per_unit','max_of_percentage_of_assessable_value_and_amount_per_unit',
    'percentage_of_retail_sale_price');
-- user before generating an invoice in any case will know who is the supplier entity and will also know the customer
-- this can store invoice details, credit note details, delivery challan details

create table invoice
(
    id                              uuid primary key,
    entity_version_id               integer default 0,
    tenant_id                       uuid references tenant (id),
    active                          bool,
    approval_status                 smallint                                  not null,
    remarks                         varchar(70),
    invoicing_mst_id                uuid references invoicing_series_mst (id) not null,
    financial_year                  smallint                                  not null,
    invoice_number                  varchar(20)                               not null,
    currency_id                     uuid references currency_master (id)      not null,
    service_invoice                 bool                                      not null,
    invoice_date_ms                 bigint                                    not null,
    e_invoicing_applicable          bool                                      not null,
    supplier_business_entity        uuid references business_entity (id)      not null,--should we change the name to
    dispatch_from_business_entity   uuid references business_entity (id)      not null,
    b2b_invoice                     bool                                      not null,
    billed_to_business_entity       uuid references business_entity (id),--only applicable in b2b invoices
    shipped_to_business_entity      uuid references business_entity (id),---only applicable in b2b invoices
    purchase_order_number           varchar(35),
    einvoice_json_s3_id             varchar(200),
    total_taxable_amount            double precision                          not null,
    total_tax_amount                double precision                          not null,
    total_additional_charges_amount double precision                          not null,
    round_off                       double precision                          not null,
    total_payable_amount            double precision                          not null,
    igst_applicable                 boolean,
    invoice_pdf_s3_id               varchar(200),
    invoice_template_id             uuid references invoice_template (id)     not null,
    payment_term_id                 uuid references payment_term,
    invoice_remarks                 varchar(100),
    ecommerce_gstin                 varchar(16),
    created_by                      uuid references app_user (id)             not null,
    updated_by                      uuid references app_user (id),
    created_at                      bigint  default extract(epoch from now()) * 1000000,
    updated_at                      bigint  default extract(epoch from now()) * 1000000
);


create table invoice_line
(                                                                        --2 things to consider, we need to store what input params were and then what we computed in order to auditable unambiguously
    id                         uuid primary key,
    entity_version_id          integer                                  default 0,
    tenant_id                  uuid references tenant (id)     not null,
    active                     bool,
    approval_status            smallint                        not null,
    remarks                    varchar(70),
    invoice_table_id           uuid references invoice (id)    not null,
    line_title_hsn_sac_id      uuid references line_title (id) not null,
    line_subtitle_id           uuid references line_subtitle (id),
    quantity                   double precision                not null,
    free_quantity              double precision                not null,
    unit_price                 real                            not null,
    tax_percentage             real                            not null,
    discount_percentage        real                            not null,
    cess_percentage            real                            not null,
    cess_amount_per_unit       real                            not null,
    retail_sale_price_for_cess real                            not null,
    cess_calculation_strategy  cess_calculation_strategy not null,--default will be percentage_of_assessable_value and percentage will be 0 in case not applicable
    line_number                smallint                        not null,
    line_net_total             double precision                not null, --double precision because quantity is in double which can cause the line total to be in double
    mrp                        real,
    batch                      varchar(15),
    expiry_date_ms             bigint,
    uqc                        varchar(15),
    reverse_charge_applicable  bool                            not null default false,
    created_by                 uuid references app_user (id)   not null,
    updated_by                 uuid references app_user (id),
    created_at                 bigint                                   default extract(epoch from now()) * 1000000,
    updated_at                 bigint                                   default extract(epoch from now()) * 1000000
);

