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
        -- convey message in the inout param that the pending id does not exist
        return; -- should we return early or not
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
        -- entry amount to be posted or voided is greater than pending amount, cannot proceed.
        -- entry amount to be checked only in case if its being posted otherwise not.
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
        raise notice 'my jsonb value is %',output_result;
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
--			output_result JSONB='{"txn_id":"","committed":true,"reason":[]}';
--			validation_result jsonb='{"txn_id":"","committed":true,"reason":[]}';
    declare t        timestamptz := clock_timestamp();
    existing_entry   transfer.id%type;
    pending_transfer transfer;
BEGIN
    select id from transfer where id = txn.id and tenant_id = txn.tenant_id into existing_entry;
    if existing_entry is not null then
        result['committed'] = 'false';
        result['reason'] = result['reason'] || concat('["transfer already exists with this id"]')::jsonb;
        return;
    end if;
    if txn.transfer_type in (3, 4) then
        select *
        from transfer
        where id = txn.pending_id and transfer_type = 2 and tenant_id = txn.tenant_id
        into pending_transfer;
    end if;
    select * from user_account where id = txn.credit_account_id and tenant_id = txn.tenant_id into credit_acc_row;
    select * from user_account where id = txn.debit_account_id and tenant_id = txn.tenant_id into debit_acc_row;
    call validate_transfer(debit_acc_row, credit_acc_row, txn, pending_transfer, result);
    if (result -> 'committed')::boolean = false then
        raise notice 'early return called';
        return;
    end if;
    INSERT INTO transfer(id, tenant_id, caused_by_event_id, grouping_id, debit_account_id, credit_account_id,
                         pending_id, ledger_master_id, code, amount, remarks, transfer_type, created_at)
    VALUES (txn.id, txn.tenant_id, txn.caused_by_event_id, txn.grouping_id, txn.debit_account_id, txn.credit_account_id,
            txn.pending_id, txn.ledger_master_id, txn.code, txn.amount, txn.remarks, txn.transfer_type, txn.created_at);
    call update_accounts_balance_for_transfer(txn, pending_transfer, credit_acc_row, debit_acc_row);
-- 	  commit;
    raise notice 'time spent=%', clock_timestamp() - t;
-- 	  return 0;
end;
$$ language plpgsql;


--what should be the response
--it should be a jsonb list with each txn unique id and correspondingly if that was committed or not and
--then a reason for not getting committed
create or replace function create_linked_transfers(txns transfer[]) returns jsonb as
$$
--should not execute for more than 5000 elements. ensure this by adding a validation
DECLARE
    failed         boolean= false;
    result_arr     jsonb='[]';
    result_element jsonb;
    txn            transfer;
    failed_index   integer;
BEGIN
    if array_length(txns, 1) > 600 then
        RAISE EXCEPTION 'no of transfers in batch cannot be more than 600 but was %', array_length(txns, 1)
            USING HINT = 'no of transfers in batch cannot be more than 600';
    end if;
    foreach txn in array txns
        loop
            result_element = json_build_object('txn_id', txn.id, 'committed', not failed, 'reason', '[]'::jsonb);
            raise notice 'result_element %',result_element;
            if failed then
                result_element['reason'] = '["linked transfer failed"]';
                result_arr = result_arr || result_element;
                --prepare a default failed response as linked transfer failed
            else
                raise notice 'calling create_ledger_transfer';
                call create_ledger_transfer(txn, result_element);
                raise notice 'create_ledger_transfer output %',result_element;
                if (result_element -> 'committed')::boolean = false then
                    select true into failed;
                end if;
                raise notice 'result_arr before %',result_arr;
                select result_arr || result_element into result_arr;--verify this line
                raise notice 'result_arr after %',result_arr;
                --append the result into jsonb array
            end if;
        end loop;
    return result_arr;
END;
$$ language plpgsql;







