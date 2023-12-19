create table if not exists currency_master
(
    id uuid primary key,
    tenant_id    uuid        not null references tenant (id),
    scale        smallint    not null,
    display_name varchar(16) not null,
    description  varchar(50),
    created_by   uuid        not null references app_user (id),
    updated_by   uuid references app_user (id),
    created_at   bigint default extract(epoch from now()) * 1000000,
    updated_at   bigint default extract(epoch from now()) * 1000000
);


create type create_currency_request as
(
    idempotence_key uuid,
    tenant_id       uuid,
    scale           smallint,
    display_name    text,
    description     text,
    created_by      uuid,
    updated_by      uuid,
    created_at      bigint,
    updated_at      bigint
);