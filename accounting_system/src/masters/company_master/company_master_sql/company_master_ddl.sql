create table company_master
(
    id                uuid primary key,
    entity_version_id integer default 0,
    tenant_id         uuid references tenant (id),
    active            boolean,
    approval_status   smallint    not null,
    remarks           varchar(70),
    name              varchar(50) not null,
    cin               varchar(21) not null,
    created_by        uuid        not null references app_user (id),
    updated_by        uuid references app_user (id),
    created_at        bigint  default extract(epoch from now()) * 1000000,
    updated_at        bigint  default extract(epoch from now()) * 1000000
);

create unique index unique_cin_company on company_master (tenant_id, cin);
