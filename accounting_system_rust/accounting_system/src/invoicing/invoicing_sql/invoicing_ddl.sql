-- user before generating an invoice in any case will know who is the supplier entity and will also know the customer
-- this can store invoice details, credit note details, delivery challan details


create table invoice
(
    id                              uuid primary key,
    entity_version_id               integer default 0,
    tenant_id                       uuid references tenant (id),
    active                          bool,
    approval_status                 smallint                                            not null,
    remarks                         varchar(70),
    invoicing_mst_id                uuid references invoicing_series_mst (id)           not null,
    financial_year                  smallint                                            not null,
    invoice_number                  varchar(20),
    currency_id                     uuid references currency_master (id)                not null,
    service_invoice                 bool                                                not null,
    invoice_date_ms                 bigint                                              not null,
    e_invoicing_applicable          bool                                                not null,
    supplier_business_entity        uuid references business_entity_invoice_detail (id) not null,
    b2b_invoice                     bool                                                not null,
    billed_to_business_entity       uuid references business_entity (id),--only applicable in b2b invoices
    shipped_to_business_entity      uuid references business_entity (id),---only applicable in b2b invoices
    purchase_order_number           varchar(35),
    einvoice_json_s3_id             uuid,
    total_taxable_amount            double precision                                    not null,
    total_tax_amount                double precision                                    not null,
    total_additional_charges_amount double precision                                    not null,
    round_off                       double precision                                    not null,
    total_payable_amount            double precision                                    not null,
    igst_applicable                 boolean,
    invoice_pdf_s3_id               uuid,
    invoice_template_id             uuid references invoice_template (id)               not null,
    payment_term_id                 uuid references payment_term,
    created_by                      uuid references app_user (id)                       not null,
    updated_by                      uuid references app_user (id),
    created_at                      bigint  default extract(epoch from now()) * 1000000,
    updated_at                      bigint  default extract(epoch from now()) * 1000000
);


create table invoice_line
(                                                                   --2 things to consider, we need to store what input params were and then what we computed in order to auditable unambiguously
    id                    uuid primary key,
    entity_version_id     integer default 0,
    tenant_id             uuid references tenant (id)     not null,
    active                bool,
    approval_status       smallint                        not null,
    remarks               varchar(70),
    invoice_table_id      uuid references invoice (id)    not null,
    line_title_hsn_sac_id uuid references line_title (id) not null,
    line_subtitle_id      uuid references line_subtitle (id),
    quantity              double precision                not null,
    unit_price            real                            not null,
    tax_rate_percentage   real                            not null,
    discount_percentage   real                            not null,
    cess_percentage       real                            not null,
    line_number           smallint                        not null,
    line_net_total        double precision                not null, --double precision because quantity is in double which can cause the line total to be in double
    mrp                   real,
    batch                 varchar(15),
    expiry_date_ms        bigint,
    uqc                   varchar(15),
    created_by            uuid references app_user (id)   not null,
    updated_by            uuid references app_user (id),
    created_at            bigint  default extract(epoch from now()) * 1000000,
    updated_at            bigint  default extract(epoch from now()) * 1000000
);
