
create table payment_term
(
    id               uuid primary key,
    tenant_id        uuid references tenant (id),
    due_days         integer                       not null,
    discount_days    integer,
    discount_percent integer,
    created_by       uuid references app_user (id) not null,
    updated_by       uuid references app_user (id),
    created_at       bigint default extract(epoch from now()) * 1000000,
    updated_at       bigint default extract(epoch from now()) * 1000000
);