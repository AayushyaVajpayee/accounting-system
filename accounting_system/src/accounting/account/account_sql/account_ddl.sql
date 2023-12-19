create table user_account -- more of a ledger account
(
    id               uuid primary key,
    tenant_id        uuid        not null references tenant (id),
    display_code     varchar(20) not null unique,
    account_type_id  uuid not null references account_type_master (id),
    user_id          uuid        not null references app_user (id),
    ledger_master_id uuid not null references ledger_master (id),
    debits_posted    bigint      not null,
    debits_pending   bigint      not null,
    credits_posted   bigint      not null,
    credits_pending  bigint      not null,
    created_by       uuid        not null references app_user (id),
    updated_by       uuid references app_user (id),
    created_at       bigint default extract(epoch from now()) * 1000000,
    updated_at       bigint default extract(epoch from now()) * 1000000
);


create type create_account_request as
(
    idempotence_key uuid,
    tenant_id       uuid,
    display_code    text,
    account_type_id uuid,
    leger_master_id uuid,
    user_id         uuid,
    created_by      uuid,
    updated_by      uuid,
    created_at      bigint,
    updated_at      bigint
);