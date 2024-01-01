-- user before generating an invoice in any case will know who is the supplier entity and will also know the customer
-- this can store invoice details, credit note details, delivery challan details
create table business_entity --should we be storing customers in this table?
(
    id                uuid,
    entity_version_id integer default 0,
    tenant_id         uuid references tenant (id),
    active            bool,
    approval_status   smallint                      not null,
    remarks           varchar(70),
    name              varchar(60),
    email             varchar(30),
    phone             varchar(15),
    address_id        uuid,
    gstin             varchar(50),
    created_by        uuid references app_user (id) not null,
    updated_by        uuid references app_user (id),
    created_at        bigint  default extract(epoch from now()) * 1000000,
    updated_at        bigint  default extract(epoch from now()) * 1000000
);


create table business_entity_invoice_detail --similarly other can be business_entity_delivery_challan_dtl
(
    id                         uuid,
    entity_version_id          integer default 0,
    tenant_id                  uuid references tenant (id),
    active                     bool,
    approval_status            smallint                      not null,
    remarks                    varchar(70),

    business_entity_id         uuid references business_entity (id),
    business_logo_s3_id        uuid,-- will not be a range query filter
    invoice_signature_s3_id    uuid, --will not be a range query filter
    invoice_template_id        uuid,
    e_invoicing_applicable     bool,
    terms_and_conditions_s3_id uuid,

    created_by                 uuid references app_user (id) not null,
    updated_by                 uuid references app_user (id),
    created_at                 bigint  default extract(epoch from now()) * 1000000,
    updated_at                 bigint  default extract(epoch from now()) * 1000000
);


create table invoice_templates
(
    id                uuid primary key, --this id will be mapped in nodejs html
    entity_version_id integer default 0,
    tenant_id         uuid references tenant (id),
    active            bool,
    approval_status   smallint                      not null,
    remarks           varchar(70),
    sample_doc_s3_id  varchar(60),
    created_by        uuid references app_user (id) not null,
    updated_by        uuid references app_user (id),
    created_at        bigint  default extract(epoch from now()) * 1000000,
    updated_at        bigint  default extract(epoch from now()) * 1000000
);

create table line_title
(
    id          uuid primary key,
    tenant_id   uuid references tenant (id) not null,
    description varchar(80)                 not null,
    hsn_code    varchar(10),
    xx_hash     bigint,
    created_at  bigint default extract(epoch from now()) * 1000000
);
create table line_subtitle
(
    id          uuid,
    tenant_id   uuid references tenant (id),
    description varchar(80),
    xx_hash     bigint,
    created_at  bigint default extract(epoch from now()) * 1000000
);


create table invoice
(
    id                         uuid,
    entity_version_id          integer default 0,
    tenant_id                  uuid references tenant (id),
    active                     bool,
    approval_status            smallint                                            not null,
    remarks                    varchar(70),
    invoicing_counter_id uuid references invoicing_series_counter (id) not null,
    invoice_number             varchar(20),
    currency_id                uuid references currency_master (id)                not null,
    service_invoice            bool                                                not null,
    invoice_date_ms            bigint                                              not null,
    e_invoicing_applicable     bool                                                not null,
    supplier_business_entity   uuid references business_entity_invoice_detail (id) not null,
    b2b_invoice                bool                                                not null,
    billed_to_business_entity  uuid references business_entity (id),--only applicable in b2b invoices
    shipped_to_business_entity uuid references business_entity (id),---only applicable in b2b invoices
    purchase_order_number      varchar(35),
    einvoice_json_s3_id        uuid,
    total_taxable_amount       double precision,
    total_tax_amount           double precision,
--     total_additional_charges_amount double precision,
--     round_off                       double precision,
    total_payable_amount       double precision,
    invoice_pdf_s3_id          uuid
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
    unit_price            integer                         not null,
    tax_rate_bps          integer                         not null,
    discount_bps          integer                         not null,
    cess_bps              integer                         not null,
    line_number           smallint                        not null,
    line_total            double precision                not null, --double precision because quantity is in double which can cause the line total to be in double
    mrp                   integer,
    batch                 varchar(15),
    expiry_date_ms        bigint,
    uqc                   varchar(15),
    igst_applicable       bool                            not null,
    created_by            uuid references app_user (id)   not null,
    updated_by            uuid references app_user (id),
    created_at            bigint  default extract(epoch from now()) * 1000000,
    updated_at            bigint  default extract(epoch from now()) * 1000000
);

create table additional_charge
(
    id            uuid,
    tenant_id     uuid references tenant (id),
    invoice_id    uuid references invoice (id),
    line_no       smallint                        not null,
    line_title_id uuid references line_title (id) not null,
    rate          integer                         not null,
    created_by    uuid references app_user (id)   not null,
    updated_by    uuid references app_user (id),
    created_at    bigint default extract(epoch from now()) * 1000000,
    updated_at    bigint default extract(epoch from now()) * 1000000
);

