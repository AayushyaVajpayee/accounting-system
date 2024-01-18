
create table if not exists tenant
(
    id           uuid primary key,
    display_name varchar(100) not null,
    created_by   uuid         not null,
    updated_by   uuid,
    created_at   bigint default extract(epoch from now()) * 1000000,
    updated_at   bigint default extract(epoch from now()) * 1000000
);

create type create_tenant_request as
(
    idempotence_key uuid,
    display_name    text,
    created_by      uuid,
    updated_by      uuid,
    created_at      bigint,
    updated_at      bigint
);