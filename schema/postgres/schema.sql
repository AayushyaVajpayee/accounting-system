create table if not exists tenant
(
    id           uuid primary key,
    display_name varchar(100) not null,
    created_by   uuid         not null,
    updated_by   uuid,
    created_at   bigint default extract(epoch from now()) * 1000000,
    updated_at   bigint default extract(epoch from now()) * 1000000
);
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

create table ledger_master
(
    id                 uuid primary key,
    tenant_id  uuid not null references tenant (id),
    display_name       varchar(50),
    currency_master_id uuid not null references currency_master (id),
    created_by uuid not null references app_user (id),
    updated_by         uuid references app_user (id),
    created_at         bigint default extract(epoch from now()) * 1000000,
    updated_at         bigint default extract(epoch from now()) * 1000000
);
create table user_account -- more of a ledger account
(
    id               uuid primary key,
    tenant_id        uuid        not null references tenant (id),
    display_code     varchar(20) not null unique,
    account_type_id  uuid not null references account_type_master (id),
    user_id          uuid        not null references app_user (id),
    ledger_master_id uuid not null references ledger_master (id),
    debits_posted    bigint      not null,
    debits_pending   bigint      not null,
    credits_posted   bigint      not null,
    credits_pending  bigint      not null,
    created_by       uuid        not null references app_user (id),
    updated_by       uuid references app_user (id),
    created_at       bigint default extract(epoch from now()) * 1000000,
    updated_at       bigint default extract(epoch from now()) * 1000000
);
create table transfer
(
    id                 UUID primary key,
    tenant_id          uuid references tenant (id),
    caused_by_event_id UUID   not null,
    grouping_id        UUID   not null,
    debit_account_id   uuid   not null,
    credit_account_id  uuid   not null,
    pending_id         UUID,
    ledger_master_id   uuid references ledger_master,
    code               smallint,
    amount             bigint not null,
    remarks            varchar(40),
--1 for regular, 2 for pending, 3 for post pending , 4 void pending
    transfer_type      smallint,
    created_at         bigint default extract(epoch from now()) * 1000000
);

create table audit_entries
(
    id              UUID primary key,
    tenant_id       uuid references tenant (id),
    audit_record_id Uuid,
    operation_type  "char",--u for update d for delete
    old_record      jsonb,
    table_id        oid,
    created_at      bigint default extract(epoch from now()) * 1000000
);

create table country_master-- what if we make this an enum?
(
    id         uuid primary key,
    name       varchar(60) not null,
    created_by uuid        not null references app_user (id),
    updated_by uuid references app_user (id),
    created_at bigint default extract(epoch from now()) * 1000000,
    updated_at bigint default extract(epoch from now()) * 1000000
);
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

create table city_master
(
    id       uuid primary key,
    city_name  varchar(60),
    state_id uuid references state_master (id),
    created_by uuid not null references app_user (id),
    updated_by uuid references app_user (id),
    created_at bigint default extract(epoch from now()) * 1000000,
    updated_at bigint default extract(epoch from now()) * 1000000,
    country_id uuid references country_master (id)
);

create table pincode_master
(
    id      uuid primary key,
    pincode    varchar(20),--if india then integer otherwise varchar
    city_id uuid references city_master (id),
    created_by uuid not null references app_user (id),
    updated_by uuid references app_user (id),
    created_at bigint default extract(epoch from now()) * 1000000,
    updated_at bigint default extract(epoch from now()) * 1000000,
    country_id uuid references country_master (id)
);



-- assumption is address will be of india only. how do we make this internationsl?
-- what will be required to make the system be able to serve international boundaries without much change
create table address
(
    id         uuid primary key,
    tenant_id  uuid references tenant (id),
    pincode_id uuid references pincode_master (id),
    city_id    uuid references city_master (id),
    country    uuid references country_master (id) not null,
    line_1     varchar(60)                         not null,
    line_2     varchar(60)                         not null,
    line_3     varchar(60), -- mostly landmark
    created_by uuid                                not null references app_user (id),
    updated_by uuid references app_user (id),
    created_at bigint default extract(epoch from now()) * 1000000,
    updated_at bigint default extract(epoch from now()) * 1000000
);


create table company_master
(
    id                uuid primary key,
    entity_version_id integer default 0,
    tenant_id         uuid references tenant (id),
    active            boolean,
    approval_status   smallint    not null,
    remarks           varchar(70),
    name              varchar(50) not null,
    cin               varchar(21) not null,
    created_by        uuid        not null references app_user (id),
    updated_by        uuid references app_user (id),
    created_at        bigint  default extract(epoch from now()) * 1000000,
    updated_at        bigint  default extract(epoch from now()) * 1000000
);

create unique index unique_cin_company on company_master (tenant_id, cin);


create type mime_type as enum ('csv','docx','jpeg','json','png','pdf','txt','xlsx');

create type workflow_type as enum ('dummy_test','create_tenant','create_account_type_mst','create_account','create_currency');

create table idempotence_store
(
    idempotence_key uuid          not null,
    workflow_type   workflow_type not null,
--     request jsonb not null,
    response        jsonb,
    created_at      bigint default extract(epoch from now()) * 1000000,
    updated_at      bigint default extract(epoch from now()) * 1000000,
    primary key (idempotence_key, workflow_type)
)
