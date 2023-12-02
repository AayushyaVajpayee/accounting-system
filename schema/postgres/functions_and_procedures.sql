--link of code from where this implementation is taken: https://gist.github.com/kjmph/5bd772b2c2df145aa645b837da7eca74
create or replace function uuid_generate_v7()
    returns uuid
as
$$
begin
    -- use random v4 uuid as starting point (which has the same variant we need)
    -- then overlay timestamp
    -- then set version 7 by flipping the 2 and 1 bit in the version 4 string
    return encode(
            set_bit(
                    set_bit(
                            overlay(uuid_send(gen_random_uuid())
                                    placing
                                    substring(int8send(floor(extract(epoch from clock_timestamp()) * 1000)::bigint) from
                                              3)
                                    from 1 for 6
                            ),
                            52, 1
                    ),
                    53, 1
            ),
            'hex')::uuid;
end
$$
    language plpgsql
    volatile;


create or replace procedure validate_pending_transfer(trf transfer, pending_trf transfer, inout output_result jsonb) as
$$
declare
    already_posted_voided_tr transfer;
begin
    if trf.transfer_type not in (3, 4) then
        return;
    end if;
    if pending_trf is null then
        --    end if;
        output_result['committed'] = 'false';
        output_result['reason'] = output_result['reason'] ||
                                  concat('["no pending transfer found for pending_id:', trf.pending_id,
                                         '  and tenant_id:', trf.tenant_id, '"]')::jsonb;

        return;
    end if;
    select *
    from transfer
    where pending_id = trf.pending_id
      and tenant_id = trf.tenant_id
    into already_posted_voided_tr;
    if already_posted_voided_tr is null then
        -- everything is cool
    else
        output_result['committed'] = 'false';
        output_result['reason'] = output_result['reason'] ||
                                  concat('["pending transfer with id:', already_posted_voided_tr.pending_id,
                                         ' already processed with id:',
                                         already_posted_voided_tr.id, ', action taken was with code: ',
                                         already_posted_voided_tr.transfer_type, ' "]')::jsonb;
        return;
        -- entry already posted or voided return a proper error message
    end if;
    if trf.amount > pending_trf.amount and trf.transfer_type = 3 then
        output_result['committed'] = 'false';
        output_result['reason'] = output_result['reason'] ||
                                  concat('["posting amount(', trf.amount, ') cannot be more than pending amount(',
                                         pending_trf.amount, ')"]')::jsonb;
        return;
    end if;
end;
$$ language plpgsql;

create or replace procedure validate_transfer(debit_acc user_account,
                                              credit_acc user_account,
                                              txn transfer,
                                              pending_trf transfer,
                                              inout output_result jsonb) as
$$
DECLARE
BEGIN

    if credit_acc is null or debit_acc is null then
        output_result['committed'] = 'false';
        if credit_acc is null then
            output_result['reason'] =
                        output_result['reason'] || concat('["no account for ', txn.credit_account_id, '"]')::jsonb;
        end if;
        if debit_acc is null then
            output_result['reason'] =
                        output_result['reason'] || concat('["no account for ', txn.debit_account_id, '"]')::jsonb;
        end if;
    end if;
    if credit_acc.ledger_master_id != txn.ledger_master_id or debit_acc.ledger_master_id != txn.ledger_master_id then
        output_result['committed'] = 'false';
        output_result['reason'] =
                    output_result['reason'] || concat('["accounts must have the same ledger debit_acc_ledger_id: ',
                                                      debit_acc.ledger_master_id,
                                                      ', credit_acc_ledger_id: ',
                                                      credit_acc.ledger_master_id,
                                                      ', transfer ledger id: ', txn.ledger_master_id, '"]')::jsonb;
    end if;
    if txn.amount <= 0 then
        output_result['committed'] = 'false';
        output_result['reason'] = output_result['reason'] ||
                                  concat('["transfer amount cannot be <=0 but was ', txn.amount, '"]')::jsonb;
    end if;
    call validate_pending_transfer(txn, pending_trf, output_result);
END;
$$ language plpgsql;

create or replace procedure update_accounts_balance_for_transfer(trf transfer, pending_trf transfer,
                                                                 credit_acc user_account, debit_acc user_account) as
$$
DECLARE
    new_credits_posted  bigint=credit_acc.credits_posted;
    new_debits_posted   bigint=debit_acc.debits_posted;
    new_credits_pending bigint=credit_acc.credits_pending;
    new_debits_pending  bigint=debit_acc.debits_pending;
BEGIN
    if trf.transfer_type = 1 then
        new_credits_posted = new_credits_posted + trf.amount;
        new_debits_posted = new_debits_posted + trf.amount;
    elsif trf.transfer_type = 2 then
        new_credits_pending = new_credits_pending + trf.amount;
        new_debits_pending = new_debits_pending + trf.amount;
    elsif trf.transfer_type = 3 then --post pending
        new_credits_posted = new_credits_posted + trf.amount;
        new_debits_posted = new_debits_posted + trf.amount;
        new_credits_pending = new_credits_pending - pending_trf.amount;
        new_debits_pending = new_debits_pending - pending_trf.amount;
    elsif trf.transfer_type = 4 then
        new_credits_pending = new_credits_pending - pending_trf.amount;
        new_debits_pending = new_debits_pending - pending_trf.amount;
    end if;
    update user_account
    set (credits_posted, credits_pending)= (new_credits_posted, new_credits_pending)
    where id = trf.credit_account_id
      and tenant_id = trf.tenant_id;
    update user_account
    set (debits_posted, debits_pending)= (new_debits_posted, new_debits_pending)
    where id = trf.debit_account_id
      and tenant_id = trf.tenant_id;

end;
$$ language plpgsql;
--{txn_id:String,committed:boolean,reason:String}
create or replace procedure create_ledger_transfer(txn transfer, inout result jsonb) as
$$
DECLARE
    credit_acc_row   user_account;
    debit_acc_row    user_account;
    declare t        timestamptz := clock_timestamp();
    existing_entry   transfer.id%type;
    pending_transfer transfer;
BEGIN
    select id from transfer where id = txn.id and tenant_id = txn.tenant_id into existing_entry;--isn't this uuid, how to ensure idempotency from client side?
    if existing_entry is not null then
        result['committed'] = 'false';
        result['reason'] = result['reason'] || concat('["transfer already exists with this id"]')::jsonb;
        return;
    end if;
    if txn.transfer_type in (3, 4) then
        select *
        from transfer
        where id = txn.pending_id
          and transfer_type = 2
          and tenant_id = txn.tenant_id
        into pending_transfer;
    end if;
    select * from user_account where id = txn.credit_account_id and tenant_id = txn.tenant_id into credit_acc_row;
    select * from user_account where id = txn.debit_account_id and tenant_id = txn.tenant_id into debit_acc_row;
    call validate_transfer(debit_acc_row, credit_acc_row, txn, pending_transfer, result);
    if (result -> 'committed')::boolean = false then
        return;
    end if;
    INSERT INTO transfer(id, tenant_id, caused_by_event_id, grouping_id, debit_account_id, credit_account_id,
                         pending_id, ledger_master_id, code, amount, remarks, transfer_type, created_at)
    VALUES (txn.id, txn.tenant_id, txn.caused_by_event_id, txn.grouping_id, txn.debit_account_id, txn.credit_account_id,
            txn.pending_id, txn.ledger_master_id, txn.code, txn.amount, txn.remarks, txn.transfer_type, txn.created_at);
    call update_accounts_balance_for_transfer(txn, pending_transfer, credit_acc_row, debit_acc_row);
    raise notice 'time spent=%', clock_timestamp() - t;
end;
$$ language plpgsql;


create or replace function create_linked_transfers(txns transfer[]) returns jsonb as
$$
DECLARE
    failed         boolean= false;
    result_arr     jsonb='[]';
    result_element jsonb;
    txn            transfer;
    failed_res jsonb;
    failed_id  uuid;
BEGIN
    if array_length(txns, 1) > 600 then
        RAISE EXCEPTION 'no of transfers in batch cannot be more than 600 but was %', array_length(txns, 1)
            USING HINT = 'no of transfers in batch cannot be more than 600';
    end if;
    BEGIN
        foreach txn in array txns
            loop
                result_element = json_build_object('txn_id', txn.id, 'committed', not failed, 'reason', '[]'::jsonb);
                call create_ledger_transfer(txn, result_element);
                if (result_element -> 'committed')::boolean = false then
                    failed = true;
                    failed_res = result_element;
                    failed_id = txn.id;
                    raise exception using errcode = 'VALFA',message = 'validation failed or error for transfer',
                        hint = 'check reason incase of validation failure. for error pg logs may help';
                end if;
                select result_arr || result_element into result_arr;
            end loop;
    exception
        when others then
            result_arr = '[]';
            foreach txn in array txns
                loop
                    if txn.id = failed_id then
                        select result_arr || failed_res into result_arr;
                    else
                        result_element = json_build_object('txn_id', txn.id, 'committed', false, 'reason', '[
                          "linked transfer failed"
                        ]'::jsonb);
                        select result_arr || result_element into result_arr;
                    end if;
                end loop;
    END;
    return result_arr;
END;
$$ language plpgsql;



create or replace function batch_process_linked_transfers(txns transfer[][]) returns jsonb as
$$
DECLARE
    txn_list       transfer[];
    result_element jsonb='[]';
    result         jsonb='[]';
BEGIN
    if cardinality(txns) > 500 then
        RAISE EXCEPTION 'no of transfers in batch cannot be more than 500 but was %', cardinality(txns)
            USING HINT = 'no of transfers in batch cannot be more than 500';
    end if;
    foreach txn_list slice 1 in array txns
        loop
            select create_linked_transfers(txn_list) into result_element;
            select result || jsonb_build_array(result_element) into result;
        end loop;
    return result;
end;
$$
    language plpgsql;

create or replace function create_audit_entry() returns trigger as
$$
DECLARE
    op_type char;
BEGIN
    if (TG_OP = 'DELETE') then
        op_type = 'd';
    elsif (TG_OP = 'UPDATE') then
        op_type = 'u';
    end if;
    insert into audit_entries(id, tenant_id, audit_record_id, table_id, operation_type, old_record)
    values (uuid_generate_v7(), old.tenant_id, old.id, TG_RELID, op_type, to_jsonb(old));
    return null;
END ;
$$
    LANGUAGE plpgsql;
--
-- create trigger audit_company_master
--  after update or delete on company_master
--  for each row
--  EXECUTE function create_audit_entry()


create trigger company_master_audit_trigger
    after update or delete
    on company_master
    for each row
execute function create_audit_entry();
create type create_tenant_request as
(
    idempotence_key uuid,
    display_name    text,
    created_by      uuid,
    updated_by      uuid,
    created_at      bigint,
    updated_at      bigint
);

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

create type create_account_type_mst_request as
(
    idempotence_key uuid,
    tenant_id       uuid,
    child_ids       uuid[],
    parent_id       uuid,
    display_name    text,
    account_code    smallint,
    created_by      uuid,
    updated_by      uuid,
    created_at      bigint,
    updated_at      bigint
);
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

create type create_account_request as
(
    idempotence_key uuid,
    tenant_id       uuid,
    display_code    text,
    account_type_id uuid,
    leger_master_id uuid,
    user_id         uuid,
    created_by      uuid,
    updated_by      uuid,
    created_at      bigint,
    updated_at      bigint
);

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

create type create_currency_request as
(
    idempotence_key uuid,
    tenant_id       uuid,
    scale           smallint,
    display_name    text,
    description     text,
    created_by      uuid,
    updated_by      uuid,
    created_at      bigint,
    updated_at      bigint
);

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

create type create_app_user_request as
(
    idempotence_key uuid,
    tenant_id       uuid,
    first_name      text,
    last_name       text,
    email_id        text,
    mobile_number   text,
    created_by      uuid,
    updated_by      uuid,
    created_at      bigint,
    updated_at      bigint
);

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


create or replace function create_company_master(req company_master, idemp_key uuid) returns uuid as
$$
DECLARE
    resp           jsonb;
    company_mst_id uuid;
    impacted_rows  int;
BEGIN
    insert into idempotence_store (idempotence_key, workflow_type, response, created_at, updated_at)
    values (idemp_key, 'create_company_mst', null, default, default)
    on conflict do nothing;
    get diagnostics impacted_rows= row_count;
    if impacted_rows != 0 then
        select uuid_generate_v7() into company_mst_id;
        insert into company_master (id, entity_version_id, tenant_id, active, approval_status, remarks, name, cin,
                                    created_by, updated_by, created_at, updated_at)
        values (company_mst_id, 0, req.tenant_id, req.active, req.approval_status, req.remarks, req.name, req.cin,
                req.created_by, req.updated_by, req.created_at, req.updated_at);
        update idempotence_store
        set response=json_build_object('id', company_mst_id)
        where idempotence_store.idempotence_key = idemp_key
          and workflow_type = 'create_company_mst';
        return company_mst_id;
    else
        select response
        from idempotence_store
        where idempotence_key = idemp_key and workflow_type = 'create_company_mst'
        into resp;
        return (resp ->> 'id')::uuid;
    end if;
end
$$ language plpgsql