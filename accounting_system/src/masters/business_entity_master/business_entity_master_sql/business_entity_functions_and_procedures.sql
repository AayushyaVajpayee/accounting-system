create type create_business_entity_request as
(
    idempotence_key   uuid,
    tenant_id         uuid,
    approval_status   smallint,
    eligible_supplier bool,
    name              text,
    email             text,
    phone             text,
    address_id        uuid,
    gstin             text,
    created_by        uuid
);
create or replace function create_business_entity(req create_business_entity_request) returns uuid as
$$
DECLARE
    resp               jsonb;
    business_entity_id uuid;
    impacted_rows      int;
BEGIN
    insert into idempotence_store (idempotence_key, workflow_type, response, created_at, updated_at)
    values (req.idempotence_key, 'create_business_entity', null, default, default)
    on conflict do nothing;
    get diagnostics impacted_rows= row_count;
    if impacted_rows != 0 then
        select uuid_generate_v7() into business_entity_id;
        insert into business_entity(id, entity_version_id, tenant_id, active, approval_status,
                                    remarks, eligible_supplier, name, email, phone, address_id,
                                    gstin, created_by, updated_by, created_at, updated_at)
        values (business_entity_id, 0, req.tenant_id, true, req.approval_status, null, req.eligible_supplier, req.name,
                req.email, req.phone, req.address_id, req.gstin, req.created_by, req.created_by, default, default);
        update idempotence_store
        set response=json_build_object('id', business_entity_id)
        where idempotence_store.idempotence_key = req.idempotence_key
          and workflow_type = 'create_business_entity';
        return business_entity_id;
    else
        select response
        from idempotence_store
        where idempotence_key = req.idempotence_key
          and workflow_type = 'create_business_entity'
        into resp;
        return (resp ->> 'id')::uuid;
    end if;
end ;
$$ language plpgsql;