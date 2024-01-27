create table line_title
(
    id          uuid primary key,
    tenant_id   uuid references tenant (id) not null,
    description varchar(80)                 not null,
    hsn_code    varchar(10),
    xx_hash     bigint,
    created_at  bigint default extract(epoch from now()) * 1000000
);