create type create_address_request as
(
    idempotence_key uuid,
    tenant_id       uuid,
    line_1          text,
    line_2          text,
    landmark        text,
    city_id         uuid,
    state_id        uuid,
    country_id      uuid,
    pincode_id      uuid,
    created_by      uuid,
    approval_status smallint

);
create or replace function create_address(req create_address_request) returns uuid as
$$
DECLARE
    resp          jsonb;
    address_id    uuid;
    impacted_rows int;
BEGIN
    insert into idempotence_store (idempotence_key, workflow_type, response, created_at, updated_at)
    values (req.idempotence_key, 'create_address', null, default, default)
    on conflict do nothing;
    get diagnostics impacted_rows = row_count;
    if impacted_rows != 0 then
        select uuid_generate_v7() into address_id;
        insert into address (id, entity_version_id, tenant_id, active, approval_status, remarks, pincode_id,
                             city_id, state_id, country, line_1, line_2, landmark, created_by, updated_by,
                             created_at, updated_at)
        values (address_id, 0, req.tenant_id, true, req.approval_status, null, req.pincode_id, req.city_id,
                req.state_id,
                req.country_id, req.line_1, req.line_2, req.landmark, req.created_by, req.created_by, default, default);
        update idempotence_store
        set response=json_build_object('id', address_id)
        where idempotence_store.idempotence_key = req.idempotence_key
          and workflow_type = 'create_address';
        return address_id;
    else
        select response
        from idempotence_store
        where idempotence_key = req.idempotence_key
          and workflow_type = 'create_address'
        into resp;
        return (resp ->> 'id')::uuid;

    end if;
end
$$ language plpgsql;
--