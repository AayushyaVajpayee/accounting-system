create table company_unit_master
(
    id                uuid primary key,
    entity_version_id integer default 0,
    tenant_id         uuid references tenant (id),
    active            bool,
    approval_status   smallint                      not null,
    remarks           varchar(70),
    company_id        uuid references company_master (id),
    address_id        uuid references address (id),
    gstin             varchar(16),
    created_by        uuid references app_user (id) not null,
    updated_y         uuid references app_user (id),
    created_at        bigint  default extract(epoch from now()) * 1000000,
    updated_at        bigint  default extract(epoch from now()) * 1000000
)