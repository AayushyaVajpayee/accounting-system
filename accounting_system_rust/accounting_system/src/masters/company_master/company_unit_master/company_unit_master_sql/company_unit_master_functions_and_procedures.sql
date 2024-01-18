create type create_company_unit as
(
    idempotency_key        uuid,
    tenant_id              uuid,
    company_id             uuid,
    gstin_no               text,
    created_by             uuid,
    active                 bool,
    approval_status        smallint,
    create_address_request create_address_request,
    existing_address_id    uuid
);

create or replace function create_company_unit_master(req create_company_unit) returns uuid as
$$
DECLARE
    created_address_id  uuid;
    impacted_rows       int;
    company_unit_mst_id uuid;
    resp                jsonb;
BEGIN
    if req.existing_address_id is null then
        select create_address(req.create_address_request) into created_address_id;
    else
        select req.existing_address_id into created_address_id;
    end if;
    insert into idempotence_store(idempotence_key, workflow_type, response, created_at, updated_at)
    VALUES (req.idempotency_key, 'create_company_unit_mst', null, default, default)
    on conflict do nothing;
    get diagnostics impacted_rows= row_count;
    if impacted_rows != 0 then
        select uuid_generate_v7() into company_unit_mst_id;
        insert into company_unit_master (id, entity_version_id, tenant_id, active, approval_status, remarks,
                                         company_id, address_id, gstin, created_by, updated_by, created_at,
                                         updated_at)
        VALUES (company_unit_mst_id, 0, req.tenant_id, req.active,
                req.approval_status, null, req.company_id,
                created_address_id, req.gstin_no, req.created_by,
                req.created_by, default, default);
        update idempotence_store
        set response=json_build_object('id', company_unit_mst_id)
        where workflow_type = 'create_company_unit_mst'
          and idempotence_store.idempotence_key = req.idempotency_key;
        return company_unit_mst_id;
    else
        select response
        from idempotence_store
        where idempotence_key = req.idempotency_key
          and workflow_type = 'create_company_unit_mst'
        into resp;
        return (resp ->> 'id')::uuid;
    end if;
end
$$ language plpgsql;