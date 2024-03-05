create table invoice_template
(
    id                uuid primary key, --this id will be mapped in nodejs html
    entity_version_id integer default 0,
    tenant_id         uuid references tenant (id),
    active            bool,
    approval_status   smallint                      not null,
    remarks           varchar(70),
    sample_doc_s3_id  varchar(200),
    created_by        uuid references app_user (id) not null,
    updated_by        uuid references app_user (id),
    created_at        bigint  default extract(epoch from now()) * 1000000,
    updated_at        bigint  default extract(epoch from now()) * 1000000
);
