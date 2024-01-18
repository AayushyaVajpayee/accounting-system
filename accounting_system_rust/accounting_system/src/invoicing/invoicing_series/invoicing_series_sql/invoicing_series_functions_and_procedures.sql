create type create_invoice_series_request as
(
    idempotence_key     uuid,
    tenant_id           uuid,
    name                text,
    prefix              text,
    zero_padded_counter bool,
    start_value         int,
    financial_year      int,
    created_by          uuid
);

create or replace function create_invoice_series(req create_invoice_series_request) returns uuid as
$$
DECLARE
    resp                      jsonb;
    invoice_series_id         uuid;
    invoice_series_counter_id uuid;
    impacted_rows             int;
BEGIN
    insert into idempotence_store (idempotence_key, workflow_type, response, created_at, updated_at)
    values (req.idempotence_key, 'create_invoice_no_series', null, default, default)
    on conflict do nothing;
    get diagnostics impacted_rows= row_count;
    if impacted_rows != 0 then
        select uuid_generate_v7() into invoice_series_id;
        select uuid_generate_v7() into invoice_series_counter_id;
        insert into invoicing_series_mst(id, entity_version_id, tenant_id, active, approval_status, remarks, name,
                                         prefix, zero_padded_counter, created_by, updated_by, created_at, updated_at)
        values (invoice_series_id, 0, req.tenant_id, true, 1, null, req.name, req.prefix, req.zero_padded_counter,
                req.created_by, req.created_by, default, default);
        insert into invoicing_series_counter(id, entity_version_id, tenant_id, invoicing_series_id, financial_year,
                                             counter, start_value, created_by,
                                             updated_by, created_at, updated_at)
        values (invoice_series_counter_id, 0, req.tenant_id, invoice_series_id, req.financial_year, 0,
                req.start_value, req.created_by, req.created_by, default, default);
        update idempotence_store
        set response=json_build_object('id', invoice_series_id)
        where idempotence_store.idempotence_key = req.idempotence_key
          and workflow_type = 'create_invoice_no_series';
        return invoice_series_id;
    else
        select response
        from idempotence_store
        where idempotence_store.idempotence_key = req.idempotence_key
          and workflow_type = 'create_invoice_no_series'
        into resp;
        return (resp ->> 'id')::uuid;
    end if;
end
$$ language plpgsql