create or replace function create_tenant(req create_tenant_request) returns uuid as
$$
DECLARE
    resp          jsonb;
    tenant_id     uuid;
    impacted_rows int;
--      t        timestamptz := clock_timestamp();
begin
    --         raise notice 'resp=%', resp;
    insert into idempotence_store (idempotence_key, workflow_type, response, created_at, updated_at)
    values (req.idempotence_key, 'create_tenant', null, default, default)
    on conflict do nothing;
    get diagnostics impacted_rows= row_count;
--         raise notice 'resp=%', impacted_rows;
    if impacted_rows != 0 then
        select uuid_generate_v7() into tenant_id;
        insert into tenant (id, display_name, created_by, updated_by, created_at, updated_at)
        values (tenant_id, req.display_name, req.created_by, req.updated_by, req.created_at, req.updated_at);
        update idempotence_store
        set response=jsonb_build_object('id', tenant_id)
        where idempotence_key = req.idempotence_key
          and workflow_type = 'create_tenant';
--             raise notice 'time spent=%', clock_timestamp() - t;
        return tenant_id;
    else
        select response
        from idempotence_store
        where idempotence_key = req.idempotence_key
          and workflow_type = 'create_tenant'
        into resp;
--             raise notice 'time spent=%', clock_timestamp() - t;
        return (resp ->> 'id')::uuid;

    end if;

end
$$ language plpgsql;