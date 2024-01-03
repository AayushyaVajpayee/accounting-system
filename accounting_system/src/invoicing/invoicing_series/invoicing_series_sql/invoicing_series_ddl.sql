create table invoicing_series_mst
(
    id                  uuid primary key,
    entity_version_id   integer default 0,
    tenant_id           uuid references tenant (id),
    active              bool,
    approval_status     smallint                      not null,
    remarks             varchar(70),

    name   varchar(30) not null,
    prefix varchar(7)  not null,
    zero_padded_counter bool,

    created_by          uuid references app_user (id) not null,
    updated_by          uuid references app_user (id),
    created_at          bigint  default extract(epoch from now()) * 1000000,
    updated_at          bigint  default extract(epoch from now()) * 1000000

);

create table invoicing_series_counter
(
    id                  uuid primary key,
    entity_version_id   integer default 0,
    tenant_id           uuid references tenant (id)               not null,
    invoicing_series_id uuid references invoicing_series_mst (id) not null,

    financial_year smallint not null,
    counter             integer                                   not null,
    start_value         integer                                   not null,

    created_by          uuid references app_user (id)             not null,
    updated_by          uuid references app_user (id),
    created_at          bigint  default extract(epoch from now()) * 1000000,
    updated_at          bigint  default extract(epoch from now()) * 1000000
);