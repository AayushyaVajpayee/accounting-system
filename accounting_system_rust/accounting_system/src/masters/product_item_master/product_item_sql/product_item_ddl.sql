create table product_item
(
    id                uuid primary key,
    tenant_id         uuid                          not null,
    entity_version_id integer default 0,
    active            bool                          not null,
    approval_status   smallint                      not null,
    remarks           varchar(70),
    title             varchar(80)                   not null,
    subtitle          varchar(80),
    hsn_sac_code      varchar(10)                   not null,
    hash              varchar(20)                   not null,--hash of title subtitle todo put unique constraint on tenant_id,hash
    created_by        uuid references app_user (id) not null,
    updated_by        uuid references app_user (id),
    created_at        bigint  default extract(epoch from now()) * 1000000,
    updated_at        bigint  default extract(epoch from now()) * 1000000
);

create table product_tax_rate
(
    id                  uuid primary key,
    tenant_id           uuid                              not null,
    entity_version_id   integer default 0,
    active              bool                              not null,
    approval_status     smallint                          not null,
    remarks             varchar(70),
    product_id          uuid references product_item (id) not null,
    tax_rate_percentage real                              not null,
    start_date          timestamp with time zone          not null,
    end_date            timestamp with time zone,
    created_by          uuid references app_user (id)     not null,
    updated_by          uuid references app_user (id),
    created_at          bigint  default extract(epoch from now()) * 1000000,
    updated_at          bigint  default extract(epoch from now()) * 1000000
);

create table cess_tax_rate
(
    id                   uuid primary key,
    tenant_id            uuid                              not null,
    entity_version_id    integer default 0,
    active               bool                              not null,
    approval_status      smallint                          not null,
    remarks              varchar(70),
    product_id           uuid references product_item (id) not null,
    cess_strategy        varchar(80)                       not null,
    cess_rate_percentage real                              not null,--ad valorem
    cess_amount_per_unit double precision                  not null, --non ad valorem
    retail_sale_price    double precision                  not null,
    start_date           timestamp with time zone          not null,
    end_date             timestamp with time zone,
    created_by           uuid references app_user (id)     not null,
    updated_by           uuid references app_user (id),
    created_at           bigint  default extract(epoch from now()) * 1000000,
    updated_at           bigint  default extract(epoch from now()) * 1000000
);
