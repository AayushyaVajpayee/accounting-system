create table additional_charge
(
    id               uuid primary key,
    tenant_id        uuid references tenant (id),
    invoice_table_id uuid references invoice (id),
    line_no          smallint                        not null,
    line_title_id    uuid references line_title (id) not null,
    rate             double precision                         not null,
    created_by       uuid references app_user (id)   not null,
    updated_by       uuid references app_user (id),
    created_at       bigint default extract(epoch from now()) * 1000000,
    updated_at       bigint default extract(epoch from now()) * 1000000
);
create type create_additional_charge_request as
(
    line_id       uuid,
    line_no       smallint,
    line_title    text,
    title_xx_hash bigint,
    rate          double precision
);
