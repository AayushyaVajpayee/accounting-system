create table if not exists account_type_master
(
    id        uuid primary key,
    tenant_id    uuid        not null references tenant (id),
    child_ids uuid[],
    parent_id uuid references account_type_master (id),
    display_name varchar(30) not null,
    account_code smallint,
    created_by   uuid        not null references app_user (id),
    updated_by   uuid references app_user (id),
    created_at   bigint default extract(epoch from now()) * 1000000,
    updated_at   bigint default extract(epoch from now()) * 1000000
);

create type create_account_type_mst_request as
(
    idempotence_key uuid,
    tenant_id       uuid,
    child_ids       uuid[],
    parent_id       uuid,
    display_name    text,
    account_code    smallint,
    created_by      uuid,
    updated_by      uuid,
    created_at      bigint,
    updated_at      bigint
);