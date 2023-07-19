use std::sync::OnceLock;

use postgres::{Client, Row, SimpleQueryMessage};
use uuid::Uuid;

use crate::ledger::ledger_models::{Transfer, TransferCreationDbResponse, TransferType};
use crate::ledger::ledger_models::TransferType::{Pending, PostPending, Regular, VoidPending};

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

const LEDGER_TRANSFER_POSTGRES_SELECT_FIELDS: &str = "id,tenant_id,caused_by_event_id,grouping_id,\
debit_account_id,credit_account_id,pending_id,ledger_master_id,code,\
amount,remarks,transfer_type,created_at";
const LEDGER_TRANSFER_TABLE_NAME: &str = "transfer";
static TRANSFER_BY_ID_QUERY: OnceLock<String> = OnceLock::new();

impl TryFrom<&Row> for Transfer {
    type Error = ();

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let k: i16 = row.get(11);
        let transfer_type = match k {
            1 => Regular,
            2 => Pending,
            3 => PostPending { pending_id: row.get(6) },
            4 => VoidPending { pending_id: row.get(6) },
            _ => panic!("{} is not mapped to transferType enum", k)
        };
        Ok(Transfer {
            id: row.get(0),
            tenant_id: row.get(1),
            caused_by_event_id: row.get(2),
            grouping_id: row.get(3),
            debit_account_id: row.get(4),
            credit_account_id: row.get(5),
            ledger_master_id: row.get(7),
            code: row.get(8),
            amount: row.get(9),
            remarks: row.get(10),
            transfer_type,
            created_at: row.get(12),

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
        let p = self.postgres_client.simple_query(&query);
        p.unwrap().iter().map(|a| {
            match a {
                SimpleQueryMessage::Row(aa) => {
                    let k = aa.get(0);
                    let p = serde_json::from_str::<Vec<TransferCreationDbResponse>>(k.unwrap()).unwrap();
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
    format!("('{}',{},'{}','{}',{},{},{},{},{},{},{},{},{})",
            transfer.id,
            transfer.tenant_id,
            transfer.caused_by_event_id,
            transfer.grouping_id,
            transfer.debit_account_id,
            transfer.credit_account_id,
            match transfer.transfer_type {
                Regular | Pending => { "null".to_string() }
                PostPending { pending_id }
                | VoidPending { pending_id } =>
                    { format!("'{}'", pending_id) }
            },
            transfer.ledger_master_id,
            transfer.code,
            transfer.amount,
            transfer.remarks.as_ref().map(|a| format!("'{}'", &a)).unwrap_or("null".to_string()),
            match transfer.transfer_type {
                Regular => 1,
                Pending => 2,
                PostPending { .. } => 3,
                VoidPending { .. } => 4,
            },
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
    use crate::accounting::account::account_models::{a_create_account_request, CreateAccountRequestTestBuilder};
    use crate::accounting::account::account_service::get_account_service_for_test;

    use crate::ledger::ledger_models::{a_transfer, Transfer, TransferBuilder};
    use crate::ledger::ledger_transfer_dao::{LedgerTransferDao, LedgerTransferDaoPostgresImpl};
    use crate::ledger::ledgermaster::ledger_master_models::{a_create_ledger_master_entry_request, CreateLedgerMasterEntryRequestTestBuilder};
    use crate::ledger::ledgermaster::ledger_master_service::get_ledger_master_service_for_test;
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

    mod transfer_type_post_tests {
        use rstest::rstest;

        // #[rstest]
        // #[case::should()]
        fn test_posting_a_post_entry(post: bool, adjust: bool, pending: bool, post_pending: bool, void_pending: bool) {
            //test_successful_create_transfers_of_multiple_sizes tests this
        }
    }

    mod transfer_type_pending_tests {
        #[test]
        fn test_posting_a_pending_entry() {
            todo!()
        }

        #[test]
        fn should_not_allow_reverting_flag_in_a_pending_entry() {
            todo!()
        }

        #[test]
        fn should_not_allow_adjusting_flag_in_a_pending_entry() {
            todo!()
        }

        #[test]
        fn should_not_allow_void_pending_flag_in_a_pending_entry() {
            todo!()
        }

        #[test]
        fn should_not_allow_post_pending_entry_in_a_pending_entry() {
            todo!()
        }
    }

    mod transfer_type_post_pending_tests {
        #[test]
        fn should_post_pending_entry_for_a_pending_entry_in_full() {
            todo!()
        }

        #[test]
        fn should_post_pending_entry_for_a_pending_entry_partially() {
            todo!()
        }

        #[test]
        fn should_not_post_pending_entry_for_a_pending_entry_in_excess() {
            todo!()
        }

        #[test]
        fn should_error_out_posting_entry_for_an_invalid_pending_entry_id() {
            todo!()
        }

        #[test]
        fn should_error_out_posting_entry_for_already_posted_pending_entry() {
            todo!()
        }

        #[test]
        fn should_error_out_posting_entry_for_an_already_voided_pending_entry_id() {
            todo!()
        }
    }

    mod transfer_type_void_pending_tests {
        #[test]
        fn should_be_able_to_void_a_pending_entry() {
            todo!()
        }

        #[test]
        fn should_not_void_an_already_voided_pending_entry() {
            todo!()
        }

        #[test]
        fn should_not_post_a_void_entry_for_missing_pending_entry() {
            todo!()
        }

        #[test]
        fn should_not_post_a_void_entry_for_post_pending_entry() {
            todo!()
        }

        #[test]
        fn should_not_post_a_void_entry_for_post_entry() {
            todo!()
        }
    }

    #[rstest]
    fn should_not_commit_transactions_which_have_been_already_persisted_idempotency() {
        let port = get_postgres_image_port();
        let postgres_client = create_postgres_client(port);
        let mut cl = LedgerTransferDaoPostgresImpl { postgres_client };
        let initial_transfers = generate_random_transfers(1, 2, 100, 1, 1);
        let re1 = cl.create_transfers(&initial_transfers);
        assert_eq!(re1.len(), 1);
        assert!(re1.first().unwrap().committed);
        let re2 = cl.create_transfers(&initial_transfers);
        println!("{:?}", re2);
        assert_eq!(re2.len(), 1);
        assert!(!re2.first().unwrap().committed);
        assert_eq!(re2.first().unwrap().reason.len(), 1);
        assert_eq!(re2.first().unwrap().reason[0], "transfer already exists with this id");
        let mut more_transfers = generate_random_transfers(1, 2, 100, 1, 2);
        for initial_transfer in initial_transfers {
            more_transfers.push(initial_transfer);
        }
        let re3 = cl.create_transfers(&more_transfers);
        assert_eq!(re3.len(), 3);
        for re in re3 {
            if re.txn_id == re1[0].txn_id {
                assert!(!re.committed);
                assert_eq!(re.reason.len(), 1);
                assert_eq!(re.reason[0], "transfer already exists with this id")
            } else {
                assert!(re.committed);
                assert_eq!(re.reason.len(), 0);
            }
        }
    }

    #[test]
    #[should_panic]
    fn should_fail_for_more_than_600_in_batch() {
        let port = get_postgres_image_port();
        let postgres_client = create_postgres_client(port);
        let mut cl = LedgerTransferDaoPostgresImpl { postgres_client };
        let transfer_candidates = generate_random_transfers(1, 2, 100, 1, 601);
        let p = cl.create_transfers(&transfer_candidates);
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
        let transfer_candidates = generate_random_transfers(
            db_acc_id,
            cr_acc_id,
            100,
            tr_led_id, 1);
        let p = cl.create_transfers(&transfer_candidates);
        assert_eq!(p.len(), 1);
        assert!(!p.first().unwrap().committed);
        assert_eq!(p.first().unwrap().reason.len(), 1);
        let err_message = format!("accounts must have the same ledger debit_acc_ledger_id: {}, credit_acc_ledger_id: {}, transfer ledger id: {}", db_acc_led_id, cr_acc_led_id, tr_led_id);
        assert_eq!(p.first().unwrap().reason.first().unwrap(), err_message.as_str());
    }

    #[rstest]
    #[case(- 1)]
    #[case(- 0)]
    fn should_fail_transfer_amounts_of_less_than_equal_to_zero(#[case] amount: i64) {
        let port = get_postgres_image_port();
        let postgres_client = create_postgres_client(port);
        let mut cl = LedgerTransferDaoPostgresImpl { postgres_client };
        let transfer_candidates = generate_random_transfers(1, 2, amount, 1, 1);
        let p = cl.create_transfers(&transfer_candidates);
        assert_eq!(p.len(), 1);
        assert!(!p.first().unwrap().committed);
        assert_eq!(p.first().unwrap().reason.len(), 1);
        assert_eq!(p.first().unwrap().reason[0], format!("transfer amount cannot be <=0 but was {}", amount).as_str());
    }

    #[rstest]
    ///todo test for all error messages --done
    /// todo test for idempotency --done
    /// todo test for max limit --done
    /// todo test post pending
    /// todo test void pending
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