create table state_master-- this can also be an enum
(
    id uuid primary key,
    state_name varchar(60),
    created_by uuid not null references app_user (id),
    updated_by uuid references app_user (id),
    created_at bigint default extract(epoch from now()) * 1000000,
    updated_at bigint default extract(epoch from now()) * 1000000,
    country_id uuid references country_master (id)
);