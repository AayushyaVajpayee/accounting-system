
create table ledger_master
(
    id                 uuid primary key,
    tenant_id  uuid not null references tenant (id),
    display_name       varchar(50),
    currency_master_id uuid not null references currency_master (id),
    created_by uuid not null references app_user (id),
    updated_by         uuid references app_user (id),
    created_at         bigint default extract(epoch from now()) * 1000000,
    updated_at         bigint default extract(epoch from now()) * 1000000
);