create or replace function create_currency(req create_currency_request) returns uuid as
$$
DECLARE
    resp          jsonb;
    currency_id   uuid;
    impacted_rows int;
BEGIN
    insert into idempotence_store (idempotence_key, workflow_type, response, created_at, updated_at)
    VALUES (req.idempotence_key, 'create_currency', null, default, default)
    on conflict do nothing;
    get diagnostics impacted_rows= row_count;
    if impacted_rows != 0 then
        select uuid_generate_v7() into currency_id;
        insert into currency_master (id, tenant_id, scale, display_name, description, created_by, updated_by,
                                     created_at, updated_at)
        values (currency_id, req.tenant_id, req.scale, req.display_name, req.description, req.created_by,
                req.updated_by, req.created_at, req.updated_at);
        update idempotence_store
        set response=json_build_object('id', currency_id)
        where idempotence_key = req.idempotence_key
          and workflow_type = 'create_currency';
        return currency_id;
    else
        select response
        from idempotence_store
        where idempotence_key = req.idempotence_key
          and workflow_type = 'create_currency'
        into resp;
        return (resp ->> 'id')::uuid;
    end if;
end
$$ language plpgsql;