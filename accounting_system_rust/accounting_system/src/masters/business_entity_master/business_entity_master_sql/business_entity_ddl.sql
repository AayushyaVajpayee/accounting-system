create table business_entity
(
    id                uuid primary key,
    entity_version_id integer default 0,
    tenant_id         uuid references tenant (id),
    active            bool,
    approval_status   smallint                      not null,
    remarks           varchar(70),
    eligible_supplier bool                          not null,
    name              varchar(80),
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
    id                         uuid primary key,
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
