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
