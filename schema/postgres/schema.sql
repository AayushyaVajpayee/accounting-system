create table if not exists tenant
(
    id           serial primary key,
    display_name varchar(100) not null,
    created_by   varchar(50)  not null,
    updated_by   varchar(50),
    created_at   bigint default extract(epoch from now()) * 1000000,
    updated_at   bigint default extract(epoch from now()) * 1000000
);
alter sequence if exists tenant_id_seq restart with 1000;

create table if not exists currency_master
(
    id           smallserial primary key,
    tenant_id    integer     not null references tenant (id),
    scale        smallint    not null,
    display_name varchar(16) not null,
    description  varchar(50),
    created_by   varchar(50) not null,
    updated_by   varchar(50),
    created_at   bigint default extract(epoch from now()) * 1000000,
    updated_at   bigint default extract(epoch from now()) * 1000000
);
alter sequence if exists currency_master_id_seq restart with 1000;
create table if not exists account_type_master
(
    id           smallserial primary key,
    tenant_id    integer     not null references tenant (id),
    child_ids    smallint[],
    parent_id    smallint references account_type_master (id),
    display_name varchar(30) not null,
    account_code smallint,
    created_by   varchar(50) not null,
    updated_by   varchar(50),
    created_at   bigint default extract(epoch from now()) * 1000000,
    updated_at   bigint default extract(epoch from now()) * 1000000
);
alter sequence if exists account_type_master_id_seq restart with 1000;
create table if not exists app_user
(
    id            serial primary key,
    tenant_id     integer     not null references tenant (id),
    first_name    varchar(50) not null,
    last_name     varchar(50),
    email_id      varchar(200),
    mobile_number varchar(14),
    created_by    varchar(50) not null,
    updated_by    varchar(50),
    created_at    bigint default extract(epoch from now()) * 1000000,
    updated_at    bigint default extract(epoch from now()) * 1000000
);
alter sequence if exists app_user_id_seq restart with 1000;

create table ledger_master
(
    id                 serial primary key,
    tenant_id          integer     not null references tenant (id),
    display_name       varchar(50),
    currency_master_id smallint    not null references currency_master (id),
    created_by         varchar(50) not null,
    updated_by         varchar(50),
    created_at         bigint default extract(epoch from now()) * 1000000,
    updated_at         bigint default extract(epoch from now()) * 1000000
);
alter sequence if exists ledger_master_id_seq restart with 1000;
create table user_account
(
    id               serial primary key,
    tenant_id        integer     not null references tenant (id),
    display_code     varchar(20) not null unique,
    account_type_id  smallint    not null references account_type_master (id),
    user_id          integer     not null references app_user (id),
    ledger_master_id integer     not null references ledger_master (id),
    debits_posted    bigint      not null,
    debits_pending   bigint      not null,
    credits_posted   bigint      not null,
    credits_pending  bigint      not null,
    created_by       varchar(50) not null,
    updated_by       varchar(50),
    created_at       bigint default extract(epoch from now()) * 1000000,
    updated_at       bigint default extract(epoch from now()) * 1000000
);
alter sequence if exists user_account_id_seq restart with 1000;
create table transfer
(
    id                 UUID primary key,
    tenant_id          integer references tenant (id),
    caused_by_event_id UUID    not null,
    grouping_id        UUID    not null,
    debit_account_id   integer not null,
    credit_account_id  integer not null,
    pending_id         UUID,
    ledger_master_id   integer,
    code               smallint,
    amount             bigint  not null,
    remarks            varchar(40),
--1 for regular, 2 for pending, 3 for post pending , 4 void pending
    transfer_type      smallint,
    created_at         bigint default extract(epoch from now()) * 1000000
);


create table state_master
(
    id         serial primary key,
    state_name varchar(60),
    created_by   varchar(50) not null,
    updated_by varchar(50),
    created_at bigint default extract(epoch from now()) * 1000000,
    updated_at bigint default extract(epoch from now()) * 1000000
);
alter sequence if exists state_master_id_seq restart with 1000;
create table city_master
(
    id         serial primary key,
    city_name  varchar(60),
    state_id   integer references state_master (id),
    created_by varchar(50) not null,
    updated_by varchar(50),
    created_at bigint default extract(epoch from now()) * 1000000,
    updated_at bigint default extract(epoch from now()) * 1000000
);
alter sequence if exists city_master_id_seq restart with 10000;

create table pincode_master
(
    id         serial primary key,
    pincode    integer,
    city_id    integer references city_master (id),
    created_by varchar(50) not null,
    updated_by varchar(50),
    created_at bigint default extract(epoch from now()) * 1000000,
    updated_at bigint default extract(epoch from now()) * 1000000
);

alter sequence if exists pincode_master_id_seq restart with 500000;

