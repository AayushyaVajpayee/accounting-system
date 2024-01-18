
create table country_master-- what if we make this an enum?
(
    id         uuid primary key,
    name       varchar(60) not null,
    created_by uuid        not null references app_user (id),
    updated_by uuid references app_user (id),
    created_at bigint default extract(epoch from now()) * 1000000,
    updated_at bigint default extract(epoch from now()) * 1000000
);