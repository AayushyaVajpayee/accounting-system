
create table line_subtitle
(
    id          uuid primary key,
    tenant_id   uuid references tenant (id),
    description varchar(80),
    xx_hash     bigint,
    created_at  bigint default extract(epoch from now()) * 1000000
);
