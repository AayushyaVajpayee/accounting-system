create or replace function create_account_type_master(req create_account_type_mst_request) returns uuid as
$$
DECLARE
    resp                jsonb;
    account_type_mst_id uuid;
    impacted_rows       int;
BEGIN
    insert into idempotence_store (idempotence_key, workflow_type, response, created_at, updated_at)
    values (req.idempotence_key, 'create_account_type_mst', null, default, default)
    on conflict do nothing;
    get diagnostics impacted_rows= row_count;
    if impacted_rows != 0 then
        select uuid_generate_v7() into account_type_mst_id;
        insert into account_type_master (id, tenant_id, child_ids, parent_id, display_name, account_code, created_by,
                                         updated_by, created_at, updated_at)
        values (account_type_mst_id, req.tenant_id, req.child_ids, req.parent_id, req.display_name, req.account_code,
                req.created_by, req.updated_by, req.created_at, req.updated_at);
        update idempotence_store
        set response=jsonb_build_object('id', account_type_mst_id)
        where idempotence_key = req.idempotence_key
          and workflow_type = 'create_account_type_mst';
        return account_type_mst_id;
    else
        select response
        from idempotence_store
        where idempotence_key = req.idempotence_key
          and workflow_type = 'create_account_type_mst'
        into resp;
        return (resp ->> 'id')::uuid;
    end if;
end
$$ language plpgsql;