
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