create or replace function create_audit_entry() returns trigger as
$$
DECLARE
    op_type char;
BEGIN
    if (TG_OP = 'DELETE') then
        op_type = 'd';
    elsif (TG_OP = 'UPDATE') then
        op_type = 'u';
    end if;
    insert into audit_entries(id, tenant_id, audit_record_id, table_id, operation_type, old_record)
    values (uuid_generate_v7(), old.tenant_id, old.id, TG_RELID, op_type, to_jsonb(old));
    return null;
END ;
$$
    LANGUAGE plpgsql;