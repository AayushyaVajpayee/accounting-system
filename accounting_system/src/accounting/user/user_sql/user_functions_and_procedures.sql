create or replace function create_app_user(req create_app_user_request) returns uuid as
$$
DECLARE
    resp          jsonb;
    app_user_id   uuid;
    impacted_rows int;
BEGIN
    insert into idempotence_store (idempotence_key, workflow_type, response, created_at, updated_at)
    values (req.idempotence_key, 'create_app_user', null, default, default)
    on conflict do nothing;
    get diagnostics impacted_rows= row_count;
    if impacted_rows != 0 then
        select uuid_generate_v7() into app_user_id;
        insert into app_user (id, tenant_id, first_name, last_name, email_id, mobile_number, created_by, updated_by,
                              created_at, updated_at)
        values (app_user_id, req.tenant_id, req.first_name, req.last_name, req.email_id, req.mobile_number,
                req.created_by, req.updated_by, req.created_at, req.updated_at);
        update idempotence_store
        set response=json_build_object('id', app_user_id)
        where idempotence_key = req.idempotence_key
          and workflow_type = 'create_app_user';
        return app_user_id;
    else
        select response
        from idempotence_store
        where idempotence_key = req.idempotence_key
          and workflow_type = 'create_app_user'
        into resp;
        return (resp ->> 'id')::uuid;
    end if;
end
$$ language plpgsql;
