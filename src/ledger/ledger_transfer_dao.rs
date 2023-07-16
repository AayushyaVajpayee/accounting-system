use std::any::Any;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::OnceLock;

use postgres::{Client, Row, SimpleQueryMessage};
use uuid::Uuid;

use crate::ledger::ledger_models::{Transfer, TransferCreationDbResponse};

pub trait LedgerTransferDao {
    ///
    ///
    fn create_transfers(&mut self, transfers: &Vec<Transfer>) -> Vec<TransferCreationDbResponse>;
    fn get_transfers_by_id(&mut self, id: Uuid) -> Option<Transfer>;
    fn get_transfers_for_account_for_interval();
}

pub struct LedgerTransferDaoPostgresImpl {
    postgres_client: Client,
}

const LEDGER_TRANSFER_POSTGRES_SELECT_FIELDS: &str = "id,tenant_id,caused_by_event_id,grouping_id,debit_account_id,credit_account_id,pending_id,reverts_id,adjusts_id,timeout,ledger_master_id,code,amount,remarks,is_pending,post_pending,void_pending,is_reversal,is_adjustment,created_at";
const LEDGER_TRANSFER_TABLE_NAME: &str = "transfer";
static TRANSFER_BY_ID_QUERY: OnceLock<String> = OnceLock::new();

impl TryFrom<&Row> for Transfer {
    type Error = ();

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Transfer {
            id: row.get(0),
            tenant_id: row.get(1),
            caused_by_event_id: row.get(2),
            grouping_id: row.get(3),
            debit_account_id: row.get(4),
            credit_account_id: row.get(5),
            pending_id: row.get(6),
            reverts_id: row.get(7),
            adjusts_id: row.get(8),
            timeout: row.get(9),
            ledger_master_id: row.get(10),
            code: row.get(11),
            amount: row.get(12),
            remarks: row.get(13),
            is_pending: row.get(14),
            post_pending: row.get(15),
            void_pending: row.get(16),
            is_reversal: row.get(17),
            is_adjustment: row.get(18),
            created_at: row.get(19),
        })
    }
}

impl LedgerTransferDaoPostgresImpl {
    fn get_transfer_by_id_query() -> &'static str {
        TRANSFER_BY_ID_QUERY.get_or_init(|| {
            format!("select {} from {} where id=$1", LEDGER_TRANSFER_POSTGRES_SELECT_FIELDS, LEDGER_TRANSFER_TABLE_NAME)
        })
    }
}

impl LedgerTransferDao for LedgerTransferDaoPostgresImpl {
    fn create_transfers(&mut self, transfers: &Vec<Transfer>) -> Vec<TransferCreationDbResponse> {
        let query = convert_transfers_to_postgres_array(&transfers);
        println!("{}", query);
        let p = self.postgres_client.simple_query(&query);
        println!("{:?}", p);
        p.unwrap().iter().map(|a| {
            match a {
                SimpleQueryMessage::Row(aa) => {
                    let k = aa.get(0);
                    let p = serde_json::from_str::<Vec<TransferCreationDbResponse>>(k.unwrap()).unwrap();
                    println!("{:?}", k);
                    println!("{}", p.len());
                    println!("{:?}", p);
                    p
                }
                SimpleQueryMessage::CommandComplete(_) => { todo!() }
                _ => { todo!() }
            }
        }).next().unwrap()
    }

    fn get_transfers_by_id(&mut self, id: Uuid) -> Option<Transfer> {
        let query = LedgerTransferDaoPostgresImpl::get_transfer_by_id_query();
        let k = self.postgres_client.query(
            query,
            &[&id],
        ).unwrap();
        k.iter().map(|row|
            row.try_into().unwrap()
        ).next()
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
    use std::ops::Not;

    use rand::Rng;
    use rstest::rstest;

    use crate::ledger::ledger_models::{a_transfer, Transfer, TransferBuilder};
    use crate::ledger::ledger_transfer_dao::{LedgerTransferDao, LedgerTransferDaoPostgresImpl};
    use crate::test_utils::test_utils_postgres::{create_postgres_client, get_postgres_image_port};

    fn generate_random_transfers(debit_account: i32, credit_account: i32, amount: i64, ledger_master_id: i32, size: usize) -> Vec<Transfer> {
        let mut transfers: Vec<Transfer> = Vec::with_capacity(size);
        for i in 0..size {
            transfers.push(a_transfer(TransferBuilder {
                debit_account_id: Some(debit_account),
                credit_account_id: Some(credit_account),
                ledger_master_id: Some(ledger_master_id),
                code: Some(rand::thread_rng().gen_range(1..500)),
                amount: Some(amount),
                ..Default::default()
            }));
        }

        transfers
    }

    #[rstest]
    #[case::empty_list(0)]
    #[case::single_element(1)]
    #[case::two_elements(2)]
    #[case::eight_elements(8)]
    fn test_successful_create_transfers_of_multiple_sizes(#[case] size: usize) {
        let port = get_postgres_image_port();
        let postgres_client = create_postgres_client(port);
        let mut cl = LedgerTransferDaoPostgresImpl { postgres_client };
        let transfer_candidates = generate_random_transfers(1, 2, 100, 1, size);
        let p = cl.create_transfers(&transfer_candidates);
        assert_eq!(size, p.len());
        for i in 0..size {
            assert!(p[i].committed);
            assert!(p[i].reason.is_empty());
            assert_eq!(transfer_candidates[i].id, p[i].txn_id)
        }
    }

    #[rstest]
    #[case::debit_acc_wrong(100, 2, 1, false, true)]
    #[case::debit_acc_wrong(100, 2, 3, false, true)]
    #[case::credit_acc_wrong(1, 200, 1, true, false)]
    #[case::credit_acc_wrong(1, 200, 3, true, false)]
    #[case::both_wrong(100, 200, 1, false, false)]
    #[case::both_wrong(100, 200, 3, false, false)]
    fn should_fail_if_account_not_present(
        #[case]debit_acc_id: i32,
        #[case]credit_acc_id: i32,
        #[case] size: usize,
        #[case]debit_account_correct: bool,
        #[case]credit_account_correct: bool) {
        let port = get_postgres_image_port();
        let postgres_client = create_postgres_client(port);
        let mut cl = LedgerTransferDaoPostgresImpl { postgres_client };
        let transfer_candidates = generate_random_transfers(debit_acc_id, credit_acc_id, 100, 1, size);
        let p = cl.create_transfers(&transfer_candidates);
        for i in 0..size {
            let response = &p[i];
            let candidate = &transfer_candidates[i];
            if i == 0 {
                assert!(response.committed.not());
                assert_eq!(response.txn_id, candidate.id);
                assert!(response.reason.is_empty().not());
                if !debit_account_correct {
                    let error = format!("no account for {}", debit_acc_id);
                    assert!(response.reason.contains(&error))
                }
                if !credit_account_correct {
                    let error = format!("no account for {}", credit_acc_id);
                    assert!(response.reason.contains(&error));
                }
            } else {
                assert_eq!(1, response.reason.len());
                assert!(response.reason[0] == "linked transfer failed");
            }
        }
        // println!("{:?}", p);
    }

    #[rstest]
    #[case::transfer_ledger_id_diff(false, false, true)]
    #[case::cred_acc_ledger_id_diff(false, true, false)]
    #[case::deb_acc_led_id_diff(true, false, false)]
    #[case::all_diff(true, true, true)]
    fn should_fail_if_accounts_have_different_ledger_ids(#[case] debit_account_ledger_id_same: bool,
                                                         #[case] credit_account_ledger_same: bool, #[case] transfer_ledger_id_same: bool) {
        let port = get_postgres_image_port();
        let postgres_client = create_postgres_client(port);
        let mut cl = LedgerTransferDaoPostgresImpl { postgres_client };
        let mut ledger_master_service = get_ledger_master_service_for_test(
            create_postgres_client(port));
        let k = a_create_ledger_master_entry_request(CreateLedgerMasterEntryRequestTestBuilder {
            ..Default::default()
        });
        let mut account_master_service = get_account_service_for_test(create_postgres_client(port));
        let mut db_acc_led_id = 1;
        let mut cr_acc_led_id = 1;
        let mut tr_led_id = 1;
        let mut db_acc_id = 1;
        let mut cr_acc_id = 2;
        if !debit_account_ledger_id_same {
            db_acc_led_id = ledger_master_service.create_ledger_master_entry(&k);
            db_acc_id = account_master_service.create_account(
                &a_create_account_request(
                    CreateAccountRequestTestBuilder {
                        ledger_master_id: Some(db_acc_led_id),
                        ..Default::default()
                    }));
        }
        if !credit_account_ledger_same {
            cr_acc_led_id = ledger_master_service.create_ledger_master_entry(&k);
            cr_acc_id = account_master_service.create_account(
                &a_create_account_request(
                    CreateAccountRequestTestBuilder {
                        ledger_master_id: Some(cr_acc_led_id),
                        ..Default::default()
                    }));
        }
        if transfer_ledger_id_same {
            tr_led_id = ledger_master_service.create_ledger_master_entry(&k);
        }
        /// ensure we have accounts with the given ledger ids
        ///create accounts with the ledger ids
        ///todo either setup test data here or prepare seed data that we can test
        let transfer_candidates = generate_random_transfers(
            db_acc_id,
            cr_acc_id,
            100,
            tr_led_id, 1);
        let p = cl.create_transfers(&transfer_candidates);
        println!("{:?}", p);
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
        let mut cl = LedgerTransferDaoPostgresImpl { postgres_client };
        let transfer_candidates = generate_random_transfers(1, 2, 100, 1, 1);
        let p = cl.create_transfers(&transfer_candidates);
        println!("{:?}", p);
        let k = cl.get_transfers_by_id(p.first().unwrap().txn_id);
        println!("{:?}", k.unwrap())
        // let p = r#"select create_linked_transfers( array[('a1a2bc89-9d0b-4ef8-bb6d-6bb9bd380a11'
        // 					   ,'1',
        // 					   'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
        // 					   'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
        // 					  1,null,null,null,null,null,3,null,
        // 					   2,null,null,null,null,null,null,null
        // 					  )]::transfer[]);"#;
    }
}