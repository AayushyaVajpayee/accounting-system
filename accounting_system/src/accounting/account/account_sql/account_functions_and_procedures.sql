
create or replace function create_account(req create_account_request) returns uuid as
$$
DECLARE
    resp          jsonb;
    account_id    uuid;
    impacted_rows int;
BEGIN
    insert into idempotence_store (idempotence_key, workflow_type, response, created_at, updated_at)
    values (req.idempotence_key, 'create_account', null, default, default)
    on conflict
        do nothing;
    get diagnostics impacted_rows = row_count;
    if impacted_rows != 0 then
        select uuid_generate_v7() into account_id;
        insert into user_account (id, tenant_id, display_code, account_type_id, user_id, ledger_master_id,
                                  debits_posted, debits_pending, credits_posted, credits_pending, created_by,
                                  updated_by, created_at, updated_at)
        values (account_id, req.tenant_id, req.display_code, req.account_type_id, req.user_id, req.leger_master_id, 0,
                0, 0, 0, req.created_by, req.updated_by, req.created_at, req.updated_at);
        update idempotence_store
        set response=jsonb_build_object('id', account_id)
        where idempotence_key = req.idempotence_key
          and workflow_type = 'create_account';
        return account_id;
    else
        select response
        from idempotence_store
        where idempotence_key = req.idempotence_key
          and workflow_type = 'create_account'
        into resp;
        return (resp ->> 'id')::uuid;
    end if;

end
$$ language plpgsql;