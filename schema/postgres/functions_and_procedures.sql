create or replace procedure validate_transfer(debit_acc user_account,
credit_acc user_account,
txn transfer,
inout output_result jsonb)  as $$
    DECLARE
    BEGIN

        if credit_acc is null or debit_acc is null then
			    output_result['committed']='false';
			    if credit_acc is null then
			        output_result['reason']= output_result['reason']||concat('["no account for ', txn.credit_account_id,'"]')::jsonb;
			    end if;
			    if debit_acc is null then
			        output_result['reason']= output_result['reason']||concat('["no account for ', txn.debit_account_id,'"]')::jsonb;
			    end if;
			    raise notice 'my jsonb value is %',output_result;
		end if;
		if credit_acc.ledger_master_id!=txn.ledger_master_id or debit_acc.ledger_master_id!=txn.ledger_master_id then
                output_result['committed']='false';
                output_result['reason'] = output_result['reason']||concat('["accounts must have the same ledger debit_acc_ledger_id: ',
                debit_acc.ledger_master_id,
                ', credit_acc_ledger_id: ',
                credit_acc.ledger_master_id,
                ', transfer ledger id: ',txn.ledger_master_id,'"]')::jsonb;
		end if;
		if txn.amount<=0 then
			    output_result['committed']='false';
			    output_result['reason'] = output_result['reason']||concat('["transfer amount cannot be <=0 but was ',txn.amount,'"]')::jsonb;
		end if;
    END;
$$ language plpgsql;
--{txn_id:String,committed:boolean,reason:String}
create or replace procedure create_ledger_transfer(txn transfer,inout result jsonb) as
    $$
        DECLARE
			credit_acc_row user_account;
			debit_acc_row user_account;
--			output_result JSONB='{"txn_id":"","committed":true,"reason":[]}';
--			validation_result jsonb='{"txn_id":"","committed":true,"reason":[]}';
			declare t timestamptz := clock_timestamp();
			existing_entry transfer.id%type;
        BEGIN
            select id from transfer where id=txn.id and tenant_id=txn.tenant_id into existing_entry;
            if existing_entry is not null then
                result['committed']='false';
                result['reason']= result['reason']||concat('["transfer already exists with this id"]')::jsonb;
                return;
            end if;
			select * from user_account where id=txn.credit_account_id and tenant_id=txn.tenant_id into credit_acc_row;
			select * from user_account where id=txn.debit_account_id and tenant_id=txn.tenant_id into debit_acc_row;
			call validate_transfer(debit_acc_row,credit_acc_row,txn,result);
            if (result->'committed')::boolean=false  then
            raise notice 'early return called';
                return;
            end if;
			 INSERT INTO transfer(
	id, tenant_id, caused_by_event_id, grouping_id, debit_account_id, credit_account_id, pending_id, ledger_master_id, code, amount, remarks,transfer_type, created_at)
	VALUES (
txn.id,txn.tenant_id,txn.caused_by_event_id,txn.grouping_id,txn.debit_account_id,txn.credit_account_id,txn.pending_id,txn.ledger_master_id,txn.code,txn.amount,txn.remarks,txn.transfer_type,txn.created_at);
      update user_account set credits_posted=credits_posted-txn.amount where id=txn.credit_account_id and tenant_id=txn.tenant_id;
	  update user_account set debits_posted=debits_posted+txn.amount where id=txn.debit_account_id and tenant_id=txn.tenant_id;
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
       failed boolean=false;
       result_arr jsonb='[]';
       result_element jsonb;
       txn transfer;
    BEGIN
      if array_length(txns,1) >600 then
             RAISE EXCEPTION 'no of transfers in batch cannot be more than 600 but was %', array_length(txns,1)
             USING HINT = 'no of transfers in batch cannot be more than 600';
      end if;
      foreach txn in array txns
      loop
       result_element=json_build_object('txn_id',txn.id,'committed',not failed,'reason','[]'::jsonb);
       raise notice 'result_element %',result_element;
        if failed then
            result_element['reason']='["linked transfer failed"]';
            result_arr=result_arr||result_element;
            --prepare a default failed response as linked transfer failed
        else
            raise notice 'calling create_ledger_transfer';
            call create_ledger_transfer(txn,result_element);
            raise notice 'create_ledger_transfer output %',result_element;
             if (result_element -> 'committed')::boolean=false then
               select true into failed;
             end if;
             raise notice 'result_arr before %',result_arr;
             select result_arr||result_element into result_arr;--verify this line
             raise notice 'result_arr after %',result_arr;
             --append the result into jsonb array
        end if;
      end loop;
      return result_arr;
    END;
$$ language plpgsql





