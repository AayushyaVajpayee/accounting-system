use std::any::Any;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::OnceLock;

use postgres::{Client, SimpleQueryMessage};
use uuid::Uuid;

use crate::ledger::ledger_models::{Transfer, TransferCreationDbResponse};

pub trait LedgerTransferDao {
    ///
    ///
    fn create_transfers(self, transfers: Vec<Transfer>);
    fn get_transfers_by_id(self, id: Uuid) -> Option<Transfer>;
    fn get_transfers_for_account_for_interval();
}

pub struct LedgerTransferDaoPostgresImpl {
    postgres_client: Client,
}

const LEDGER_TRANSFER_POSTGRES_SELECT_FIELDS: &str = "id,tenant_id,caused_by_event_id,grouping_id,debit_account_id,credit_account_id,pending_id,reverts_id,adjusts_id,timeout,ledger_master_id,code,amount,remarks,is_pending,post_pending,void_pending,is_reversal,is_adjustment,created_at";
const LEDGER_TRANSFER_TABLE_NAME: &str = "transfer";
static TRANSFER_BY_ID_QUERY: OnceLock<String> = OnceLock::new();

impl LedgerTransferDaoPostgresImpl {
    fn get_transfer_by_id_query() -> &'static str {
        TRANSFER_BY_ID_QUERY.get_or_init(|| {
            format!("select {} from {} where id=$1", LEDGER_TRANSFER_POSTGRES_SELECT_FIELDS, LEDGER_TRANSFER_TABLE_NAME)
        })
    }
}

impl LedgerTransferDao for LedgerTransferDaoPostgresImpl {
    fn create_transfers(mut self, transfers: Vec<Transfer>) {
        let query = convert_transfers_to_postgres_array(&transfers);
        println!("{}", query);
        let p = self.postgres_client.simple_query(&query);
        println!("{:?}", p);
        p.unwrap().iter().for_each(|a| {
            match a {
                SimpleQueryMessage::Row(aa) => {
                    let k = aa.get(0);
                    let p = serde_json::from_str::<Vec<TransferCreationDbResponse>>(k.unwrap()).unwrap();
                    println!("{:?}", k);
                    println!("{}", p.len());
                    println!("{:?}", p);
                }
                SimpleQueryMessage::CommandComplete(_) => {}
                _ => {}
            };
            a;
        })
    }

    fn get_transfers_by_id(mut self, id: Uuid) -> Option<Transfer> {
        let query = LedgerTransferDaoPostgresImpl::get_transfer_by_id_query();
        let k = self.postgres_client.query(
            query,
            &[&id],
        ).unwrap();
        k.iter().map(|row|
            Transfer {}
        )
        todo!()
    }

    fn get_transfers_for_account_for_interval() {
        todo!()
    }
}

fn convert_transfer_to_postgres_composite_type_input_string(transfer: &Transfer) -> String {
    format!("('{}',{},'{}','{}',{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{})",
            transfer.id,
            transfer.tenant_id,
            transfer.caused_by_event_id,
            transfer.grouping_id,
            transfer.debit_account_id,
            transfer.credit_account_id,
            transfer.pending_id.map(|a| format!("'{}'", a)).unwrap_or("null".to_string()),
            transfer.reverts_id.map(|a| format!("'{}'", a)).unwrap_or("null".to_string()),
            transfer.adjusts_id.map(|a| format!("'{}'", a)).unwrap_or("null".to_string()),
            transfer.timeout.map(|a| a.to_string()).unwrap_or("null".to_string()),
            transfer.ledger_master_id,
            transfer.code,
            transfer.amount,
            transfer.remarks.as_ref().map(|a| format!("'{}'", &a)).unwrap_or("null".to_string()),
            transfer.is_pending,
            transfer.post_pending,
            transfer.void_pending,
            transfer.is_reversal,
            transfer.is_adjustment,
            transfer.created_at
    )
}

fn convert_transfers_to_postgres_array(transfers: &Vec<Transfer>) -> String {
    format!("select create_linked_transfers(array[{}]::transfer[])", transfers
        .into_iter()
        .map(convert_transfer_to_postgres_composite_type_input_string)
        .collect::<Vec<String>>().join(","))
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use rstest::rstest;

    use crate::ledger::ledger_models::{a_transfer, Transfer, TransferBuilder};
    use crate::ledger::ledger_transfer_dao::{LedgerTransferDao, LedgerTransferDaoPostgresImpl};
    use crate::test_utils::test_utils_postgres::{create_postgres_client, get_postgres_image_port};

    fn generate_random_transfers(debit_account: i32, credit_account: i32, amount: i64, ledger_master_id: i32) -> Vec<Transfer> {
        let transfer = a_transfer(TransferBuilder {
            debit_account_id: Some(debit_account),
            credit_account_id: Some(credit_account),
            ledger_master_id: Some(ledger_master_id),
            code: Some(rand::thread_rng().gen_range(1..500)),
            amount: Some(amount),
            ..Default::default()
        });
        vec![transfer]
    }

    #[rstest]
    // #[case::empty_list_of_transfer()]
    // #[case::successful_create_transfer_batch()]
    // #[case::success_one_linked_batch_should_pass_and_other_should_fail()]
    ///todo test for all error messages
    /// todo test for idempotency
    /// todo test for max limit
    /// todo test for exceptions. there will be many cases to handle.like, different constraint violations,duplicate entries, some error etc
    /// todo test for 10, 100, and 1000 batch insertions to see how these behave
    /// todo test with and without exception block to see what difference does it make
    /// todo test reverts id
    /// todo test post pending
    /// todo test void pending
    /// todo test adjustment id
    /// todo test timeout if possible
    /// todo check accounts have the same currency
    /// todo how to check if the procedure is present in database or not. pretty important.for the code and database to be in sync
    fn test_create_ledger_transfers_procedure() {
        let port = get_postgres_image_port();
        let postgres_client = create_postgres_client(port);
        let cl = LedgerTransferDaoPostgresImpl { postgres_client };
        cl.create_transfers(generate_random_transfers(1, 2, 100, 1));
        // let p = r#"select create_linked_transfers( array[('a1a2bc89-9d0b-4ef8-bb6d-6bb9bd380a11'
        // 					   ,'1',
        // 					   'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
        // 					   'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
        // 					  1,null,null,null,null,null,3,null,
        // 					   2,null,null,null,null,null,null,null
        // 					  )]::transfer[]);"#;
    }
}