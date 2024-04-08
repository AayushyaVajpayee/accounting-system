create type create_invoice_template_request as
(
    idempotence_key  uuid,
    sample_doc_s3_id text,
    tenant_id        uuid,
    user_id          uuid
);

create or replace function create_invoice_template(req create_invoice_template_request) returns uuid as
    $$
    DECLARE
        invoice_template_id uuid;
        impacted_rows   int;
        resp            jsonb;
    BEGIN
        insert into idempotence_store (idempotence_key, workflow_type, response, created_at, updated_at)
        values (req.idempotence_key, 'create_invoice_template', null, default, default)
        on conflict do nothing;
        get diagnostics impacted_rows= row_count;
        if impacted_rows !=0 then
            select uuid_generate_v7() into invoice_template_id;
            insert into invoice_template (id, entity_version_id, tenant_id, active, approval_status, remarks,
                                          sample_doc_s3_id, created_by, updated_by, created_at, updated_at)
            values (invoice_template_id,0,req.tenant_id,true,1,null,req.sample_doc_s3_id,req.user_id,
                    req.user_id,default,default);
            update idempotence_store
            set response=json_build_object('id', invoice_template_id)
            where idempotence_key = req.idempotence_key
              and workflow_type = 'create_invoice_template';
            return invoice_template_id;
        else
            select response
            from idempotence_store
            where idempotence_store.idempotence_key = req.idempotence_key
              and workflow_type = 'create_invoice_template'
            into resp;
            return (resp ->> 'id')::uuid;
        end if;
    end
    $$ language plpgsql