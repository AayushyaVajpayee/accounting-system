create table if not exists app_user
(
    id            uuid primary key,
    tenant_id     uuid        not null references tenant (id),
    first_name    varchar(50) not null,
    last_name     varchar(50),
    email_id      varchar(200),
    mobile_number varchar(14),
    created_by    uuid        not null references app_user (id),
    updated_by    uuid references app_user (id),
    created_at    bigint default extract(epoch from now()) * 1000000,
    updated_at    bigint default extract(epoch from now()) * 1000000
);


create type create_app_user_request as
(
    idempotence_key uuid,
    tenant_id       uuid,
    first_name      text,
    last_name       text,
    email_id        text,
    mobile_number   text,
    created_by      uuid,
    updated_by      uuid,
    created_at      bigint,
    updated_at      bigint
);