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
