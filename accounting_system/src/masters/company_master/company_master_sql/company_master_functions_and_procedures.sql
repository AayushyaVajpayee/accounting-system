--
-- create trigger audit_company_master
--  after update or delete on company_master
--  for each row
--  EXECUTE function create_audit_entry()


create trigger company_master_audit_trigger
    after update or delete
    on company_master
    for each row;



create or replace function create_company_master(req company_master, idemp_key uuid) returns uuid as
$$
DECLARE
    resp           jsonb;
    company_mst_id uuid;
    impacted_rows  int;
BEGIN
    insert into idempotence_store (idempotence_key, workflow_type, response, created_at, updated_at)
    values (idemp_key, 'create_company_mst', null, default, default)
    on conflict do nothing;
    get diagnostics impacted_rows= row_count;
    if impacted_rows != 0 then
        select uuid_generate_v7() into company_mst_id;
        insert into company_master (id, entity_version_id, tenant_id, active, approval_status, remarks, name, cin,
                                    created_by, updated_by, created_at, updated_at)
        values (company_mst_id, 0, req.tenant_id, req.active, req.approval_status, req.remarks, req.name, req.cin,
                req.created_by, req.updated_by, req.created_at, req.updated_at);
        update idempotence_store
        set response=json_build_object('id', company_mst_id)
        where idempotence_store.idempotence_key = idemp_key
          and workflow_type = 'create_company_mst';
        return company_mst_id;
    else
        select response
        from idempotence_store
        where idempotence_key = idemp_key
          and workflow_type = 'create_company_mst'
        into resp;
        return (resp ->> 'id')::uuid;
    end if;
end
$$ language plpgsql;