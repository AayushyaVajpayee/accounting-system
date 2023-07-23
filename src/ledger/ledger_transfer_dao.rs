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
        let mut t = self.postgres_client.transaction().unwrap();
        let p = t.simple_query(&query);
        let mut k = p.unwrap().iter().map(|a| {
            match a {
                SimpleQueryMessage::Row(aa) => {
                    let k = aa.get(0);
                    let p = serde_json::from_str::<Vec<TransferCreationDbResponse>>(k.unwrap()).unwrap();
                    p
                }
                SimpleQueryMessage::CommandComplete(_) => { todo!() }
                _ => { todo!() }
            }
        }).next().unwrap();
        if k.iter().any(|a| !a.committed) {
            t.rollback().unwrap();
            return k.into_iter().map(|mut k| {
                if k.committed {
                    k.committed = false;
                    k.reason = vec!["linked transfer failed".to_string()];
                }
                k
            }).collect();
            //todo the above processing can be done in plpgsql block too and i should try there
            // since major backbone of ours will plpgsql
        } else {
            t.commit().unwrap();
        }
        k
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
        .iter()
        .map(convert_transfer_to_postgres_composite_type_input_string)
        .collect::<Vec<String>>().join(","))
}

#[cfg(test)]
mod tests {
    use std::ops::Not;

    use rand::Rng;
    use rstest::rstest;
    use crate::accounting::account::account_models::{a_create_account_request, Account, CreateAccountRequestTestBuilder};
    use crate::accounting::account::account_service::get_account_service_for_test;

    use crate::ledger::ledger_models::{a_transfer, Transfer, TransferBuilder, TransferType};
    use crate::ledger::ledger_transfer_dao::{LedgerTransferDao, LedgerTransferDaoPostgresImpl};
    use crate::ledger::ledgermaster::ledger_master_models::{a_create_ledger_master_entry_request, CreateLedgerMasterEntryRequestTestBuilder};
    use crate::ledger::ledgermaster::ledger_master_service::get_ledger_master_service_for_test;
    use crate::test_utils::test_utils_postgres::{create_postgres_client, get_postgres_image_port};

    /// need this so that every test case can act on different set of accounts and we can
    /// verify before-after account balance of transfers.
    /// returns account ids created
    fn create_two_accounts_for_transfer() -> Vec<i32> {
        let port = get_postgres_image_port();
        let postgres_client = create_postgres_client(port);
        let mut k = get_account_service_for_test(postgres_client);
        let a1 = a_create_account_request(CreateAccountRequestTestBuilder {
            ..Default::default()
        });
        let a2 = a_create_account_request(CreateAccountRequestTestBuilder {
            ..Default::default()
        });
        let a1_id = k.create_account(&a1);
        let a2_id = k.create_account(&a2);
        vec![a1_id, a2_id]
    }

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


    mod transfer_type_pending_tests {
        use rstest::rstest;
        use crate::accounting::account::account_service::get_account_service_for_test;
        use crate::ledger::ledger_models::{a_transfer, TransferBuilder, TransferType};
        use crate::ledger::ledger_models::TransferType::{Pending, Regular};
        use crate::ledger::ledger_transfer_dao::{LedgerTransferDao, LedgerTransferDaoPostgresImpl};
        use crate::ledger::ledger_transfer_dao::tests::create_two_accounts_for_transfer;
        use crate::test_utils::test_utils_postgres::{create_postgres_client, get_postgres_image_port};

        #[rstest]
        #[case::regular_entry(Some(TransferType::Regular))]
        #[case::pending_entry(Some(TransferType::Pending))]
        fn test_posting_a_pending_entry(#[case] entry_type: Option<TransferType>) {
            let port = get_postgres_image_port();
            let postgres_client = create_postgres_client(port);
            let mut cl = LedgerTransferDaoPostgresImpl { postgres_client };
            let accs = create_two_accounts_for_transfer();
            let mut acc_ser = get_account_service_for_test(create_postgres_client(port));
            let acc1 = acc_ser.get_account_by_id(&accs[0]).unwrap();
            let acc2 = acc_ser.get_account_by_id(&accs[1]).unwrap();
            let mut k = a_transfer(TransferBuilder {
                transfer_type: entry_type.clone(),
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                ..Default::default()
            });
            let p = cl.create_transfers(&vec![k.clone()]);
            let kkk = cl.get_transfers_by_id(k.id).unwrap();
            let acc_1_after = acc_ser.get_account_by_id(&accs[0]).unwrap();
            let acc_2_after = acc_ser.get_account_by_id(&accs[1]).unwrap();
            if entry_type.clone().unwrap() == Pending {
                assert_eq!(acc1.credits_posted, acc_1_after.credits_posted);
                assert_eq!(acc1.debits_posted, acc_1_after.debits_posted);
                assert_eq!(acc2.credits_posted, acc_2_after.credits_posted);
                assert_eq!(acc2.debits_posted, acc_2_after.debits_posted);
                assert_eq!(acc1.debits_pending + 100, acc_1_after.debits_pending);
                assert_eq!(acc2.credits_pending + 100, acc_2_after.credits_pending)
            }
            if (entry_type.clone().unwrap() == Regular) {
                assert_eq!(acc1.credits_pending, acc_1_after.credits_pending);
                assert_eq!(acc1.debits_pending, acc_1_after.debits_pending);
                assert_eq!(acc2.credits_pending, acc_2_after.credits_pending);
                assert_eq!(acc2.debits_pending, acc_2_after.debits_pending);
                assert_eq!(acc1.debits_posted + 100, acc_1_after.debits_posted);
                assert_eq!(acc2.credits_posted + 100, acc_2_after.credits_posted)
            }

            assert_eq!(kkk.transfer_type, entry_type.unwrap());
            assert_eq!(1, p.len());
            assert!(p[0].committed);
            assert_eq!(0, p[0].reason.len());
        }
    }

    mod pending_transfer_resolution_tests {
        use rstest::{fixture, rstest};
        use uuid::Uuid;
        use crate::accounting::account::account_service::get_account_service_for_test;
        use crate::ledger::ledger_models::{a_transfer, Transfer, TransferBuilder, TransferType};
        use crate::ledger::ledger_transfer_dao::{LedgerTransferDao, LedgerTransferDaoPostgresImpl};
        use crate::ledger::ledger_transfer_dao::tests::create_two_accounts_for_transfer;
        use crate::test_utils::test_utils_postgres::{create_postgres_client, get_postgres_image_port};

        fn pending_transfer() -> Transfer {
            let accs = create_two_accounts_for_transfer();
            a_transfer(TransferBuilder {
                transfer_type: Some(TransferType::Pending),
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(100),
                ..Default::default()
            })
        }

        #[rstest]
        #[case::should_post_pending_entry_for_a_pending_entry_in_full(pending_transfer(),
        Some(TransferType::PostPending{pending_id: uuid::Uuid::new_v4()}),
        Some(100))]
        #[case::should_post_pending_entry_for_a_pending_entry_partially(pending_transfer(),
        Some(TransferType::PostPending{pending_id: uuid::Uuid::new_v4()}),
        Some(99))]
        #[case::should_be_able_to_void_a_pending_entry(pending_transfer(),
        Some(TransferType::VoidPending{pending_id: uuid::Uuid::new_v4()}),
        Some(100))]
        // #[case(Some(TransferType::Pending),Some(TransferType::VoidPending{pending_id:uuid::Uuid::new_v4()}))]
        fn should_be_able_to_resolve_pending_transfer(
            #[case]pending_transfer: Transfer,
            #[case] pending_resolution_type: Option<TransferType>, #[case] pending_resolution_amount: Option<i64>)
        {
            let pending_resolution_type = match pending_resolution_type.unwrap() {
                TransferType::Regular | TransferType::Pending => { panic!("invalid state") }
                TransferType::PostPending { .. } => { TransferType::PostPending { pending_id: pending_transfer.id } }
                TransferType::VoidPending { .. } => { TransferType::VoidPending { pending_id: pending_transfer.id } }
            };
            let port = get_postgres_image_port();
            let postgres_client = create_postgres_client(port);
            let mut cl = LedgerTransferDaoPostgresImpl { postgres_client };
            let resolved_pending_transfer = a_transfer(TransferBuilder {
                transfer_type: Some(pending_resolution_type),
                debit_account_id: Some(pending_transfer.debit_account_id),
                credit_account_id: Some(pending_transfer.credit_account_id),
                amount: pending_resolution_amount,
                ..Default::default()
            });
            let mut acc_ser = get_account_service_for_test(create_postgres_client(port));
            let acc1 = acc_ser.get_account_by_id(&resolved_pending_transfer.debit_account_id).unwrap();
            let acc2 = acc_ser.get_account_by_id(&resolved_pending_transfer.credit_account_id).unwrap();
            let p = cl.create_transfers(&vec![pending_transfer, resolved_pending_transfer.clone()]);
            println!("{:?}", p);
            let acc1_after = acc_ser.get_account_by_id(&resolved_pending_transfer.debit_account_id).unwrap();
            let acc2_after = acc_ser.get_account_by_id(&resolved_pending_transfer.credit_account_id).unwrap();
            if matches!(resolved_pending_transfer.transfer_type,TransferType::PostPending {..}) {
                assert_eq!(acc1.debits_posted + resolved_pending_transfer.amount, acc1_after.debits_posted);
                assert_eq!(acc1.debits_pending, acc1_after.debits_pending);
                assert_eq!(acc1.credits_pending, acc1_after.credits_pending);
                assert_eq!(acc1.credits_posted, acc1_after.credits_posted);
                assert_eq!(acc2.debits_posted, acc2_after.debits_posted);
                assert_eq!(acc2.debits_pending, acc2_after.debits_pending);
                assert_eq!(acc2.credits_pending, acc2_after.credits_pending);
                assert_eq!(acc2.credits_posted + resolved_pending_transfer.amount, acc2_after.credits_posted);
            }
            if matches!(resolved_pending_transfer.transfer_type,TransferType::VoidPending {..}) {
                assert_eq!(acc1.debits_posted, acc1_after.debits_posted);
                assert_eq!(acc1.debits_pending, acc1_after.debits_pending);
                assert_eq!(acc1.credits_pending, acc1_after.credits_pending);
                assert_eq!(acc1.credits_posted, acc1_after.credits_posted);
                assert_eq!(acc2.debits_posted, acc2_after.debits_posted);
                assert_eq!(acc2.debits_pending, acc2_after.debits_pending);
                assert_eq!(acc2.credits_pending, acc2_after.credits_pending);
                assert_eq!(acc2.credits_posted, acc2_after.credits_posted);
            }

            let kkk = cl.get_transfers_by_id(resolved_pending_transfer.id).unwrap();
            // matches!(kkk.transfer_type,TransferType::PostPending {..});

            assert_eq!(2, p.len());
            for x in p {
                assert!(x.committed);
                assert_eq!(0, x.reason.len());
            }
        }

        #[rstest]
        #[case(Some(TransferType::PostPending{pending_id: Uuid::new_v4()}))]
        #[case(Some(TransferType::VoidPending{pending_id: Uuid::new_v4()}))]
        fn should_error_out_posting_entry_for_an_invalid_pending_entry_id(#[case] transfer_type: Option<TransferType>)
        {
            let port = get_postgres_image_port();
            let postgres_client = create_postgres_client(port);
            let accs = create_two_accounts_for_transfer();
            let mut acc_service = get_account_service_for_test(create_postgres_client(port));
            let acc1 = acc_service.get_account_by_id(&accs[0]).unwrap();
            let acc2 = acc_service.get_account_by_id(&accs[1]).unwrap();
            let mut cl = LedgerTransferDaoPostgresImpl { postgres_client };
            let mut pending_transfer = a_transfer(TransferBuilder {
                transfer_type: Some(TransferType::Pending),
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(100),
                ..Default::default()
            });
            let mut resolved_pending_transfer = a_transfer(TransferBuilder {
                transfer_type: transfer_type,
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(100),
                ..Default::default()
            });
            let p = cl.create_transfers(&vec![pending_transfer.clone(), resolved_pending_transfer.clone()]);
            println!("{:?}", &p);
            let acc1_after = acc_service.get_account_by_id(&accs[0]).unwrap();
            let acc2_after = acc_service.get_account_by_id(&accs[1]).unwrap();
            assert_eq!(acc1.debits_posted, acc1_after.debits_posted);
            assert_eq!(acc1.debits_pending, acc1_after.debits_pending);
            assert_eq!(acc1.credits_pending, acc1_after.credits_pending);
            assert_eq!(acc1.credits_posted, acc1_after.credits_posted);
            assert_eq!(acc2.debits_posted, acc2_after.debits_posted);
            assert_eq!(acc2.debits_pending, acc2_after.debits_pending);
            assert_eq!(acc2.credits_pending, acc2_after.credits_pending);
            assert_eq!(acc2.credits_posted, acc2_after.credits_posted);
            assert!(cl.get_transfers_by_id(pending_transfer.id).is_none());
            assert!(cl.get_transfers_by_id(resolved_pending_transfer.id).is_none());
            assert_eq!(2, p.len());
            for x in p {
                assert!(!x.committed);
                assert_eq!(1, x.reason.len());
            }
        }

        #[rstest]
        fn should_not_act_on_an_already_resolved_pending_transfer(
            #[values("pp", "vp")]
            first_resolved_transfer_type: String,
            #[values("pp", "vp")]
            second_resolved_transfer_type: String,
        )
        {
            //todo may be this can be combined with above test
            let port = get_postgres_image_port();
            let postgres_client = create_postgres_client(port);
            let accs = create_two_accounts_for_transfer();
            let mut acc_service = get_account_service_for_test(create_postgres_client(port));
            let acc1 = acc_service.get_account_by_id(&accs[0]).unwrap();
            let acc2 = acc_service.get_account_by_id(&accs[1]).unwrap();
            let mut cl = LedgerTransferDaoPostgresImpl { postgres_client };
            let pending_transfer = a_transfer(TransferBuilder {
                transfer_type: Some(TransferType::Pending),
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(100),
                ..Default::default()
            });
            let first_resolved_transfer = a_transfer(TransferBuilder {
                transfer_type: if first_resolved_transfer_type == "pp" {
                    Some(TransferType::PostPending { pending_id: pending_transfer.id })
                } else {
                    Some(TransferType::VoidPending { pending_id: pending_transfer.id })
                },
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(100),
                ..Default::default()
            });
            let second_resolved_transfer = a_transfer(TransferBuilder {
                transfer_type: if second_resolved_transfer_type == "pp" {
                    Some(TransferType::PostPending { pending_id: pending_transfer.id })
                } else {
                    Some(TransferType::VoidPending { pending_id: pending_transfer.id })
                },
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(100),
                ..Default::default()
            });
            //todo first 2 should be committed together and third one should fail.
            let p = cl.create_transfers(&vec![pending_transfer.clone(), first_resolved_transfer.clone(), second_resolved_transfer.clone()]);
            let acc1_after = acc_service.get_account_by_id(&accs[0]).unwrap();
            let acc2_after = acc_service.get_account_by_id(&accs[1]).unwrap();
            assert_eq!(acc1.debits_posted, acc1_after.debits_posted);
            assert_eq!(acc1.debits_pending, acc1_after.debits_pending);
            assert_eq!(acc1.credits_pending, acc1_after.credits_pending);
            assert_eq!(acc1.credits_posted, acc1_after.credits_posted);
            assert_eq!(acc2.debits_posted, acc2_after.debits_posted);
            assert_eq!(acc2.debits_pending, acc2_after.debits_pending);
            assert_eq!(acc2.credits_pending, acc2_after.credits_pending);
            assert_eq!(acc2.credits_posted, acc2_after.credits_posted);
            println!("{:?}", &p);
            assert!(cl.get_transfers_by_id(pending_transfer.id).is_none());
            assert!(cl.get_transfers_by_id(first_resolved_transfer.id).is_none());
            assert!(cl.get_transfers_by_id(second_resolved_transfer.id).is_none());
            assert_eq!(3, p.len());
            for x in p {
                assert!(!x.committed);
                assert_eq!(1, x.reason.len());
            }
        }

        #[ignore]
        #[test]
        fn verify_credits_debits_posted_pending_after_posting_void_transfer() {
            todo!()
        }

        #[test]
        fn should_not_post_pending_entry_for_a_pending_entry_in_excess() {
            let port = get_postgres_image_port();
            let postgres_client = create_postgres_client(port);
            let accs = create_two_accounts_for_transfer();
            let mut acc_service = get_account_service_for_test(create_postgres_client(port));
            let acc1 = acc_service.get_account_by_id(&accs[0]).unwrap();
            let acc2 = acc_service.get_account_by_id(&accs[1]).unwrap();
            let mut cl = LedgerTransferDaoPostgresImpl { postgres_client };
            let mut pending_transfer = a_transfer(TransferBuilder {
                transfer_type: Some(TransferType::Pending),
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(100),
                ..Default::default()
            });
            let mut post_pending_transfer = a_transfer(TransferBuilder {
                transfer_type: Some(TransferType::PostPending { pending_id: pending_transfer.id.clone() }),
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(101),
                ..Default::default()
            });
            let p = cl.create_transfers(&vec![pending_transfer.clone(), post_pending_transfer.clone()]);
            println!("{:?}", &p);
            let acc1_after = acc_service.get_account_by_id(&accs[0]).unwrap();
            let acc2_after = acc_service.get_account_by_id(&accs[1]).unwrap();
            assert_eq!(acc1.debits_posted, acc1_after.debits_posted);
            assert_eq!(acc1.debits_pending, acc1_after.debits_pending);
            assert_eq!(acc1.credits_pending, acc1_after.credits_pending);
            assert_eq!(acc1.credits_posted, acc1_after.credits_posted);
            assert_eq!(acc2.debits_posted, acc2_after.debits_posted);
            assert_eq!(acc2.debits_pending, acc2_after.debits_pending);
            assert_eq!(acc2.credits_pending, acc2_after.credits_pending);
            assert_eq!(acc2.credits_posted, acc2_after.credits_posted);
            assert!(cl.get_transfers_by_id(pending_transfer.id).is_none());
            assert!(cl.get_transfers_by_id(post_pending_transfer.id).is_none());
            assert_eq!(2, p.len());
            for x in p {
                assert!(!x.committed);
                assert_eq!(1, x.reason.len());
            }
        }

        #[rstest]
        fn should_not_resolve_a_post_transfer(
            #[values("pp")]resolution_type: String) {
            //todo important test
            let port = get_postgres_image_port();
            let postgres_client = create_postgres_client(port);
            let accs = create_two_accounts_for_transfer();
            let mut acc_service = get_account_service_for_test(create_postgres_client(port));
            let acc1 = acc_service.get_account_by_id(&accs[0]).unwrap();
            let acc2 = acc_service.get_account_by_id(&accs[1]).unwrap();
            let mut cl = LedgerTransferDaoPostgresImpl { postgres_client };
            let mut regular_transfer = a_transfer(TransferBuilder {
                transfer_type: Some(TransferType::Regular),
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(100),
                ..Default::default()
            });

            let mut resolving_transfer = a_transfer(TransferBuilder {
                transfer_type: if resolution_type == "pp" {
                    Some(TransferType::PostPending { pending_id: regular_transfer.id.clone() })
                } else {
                    Some(TransferType::VoidPending { pending_id: regular_transfer.id.clone() })
                },
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(100),
                ..Default::default()
            });
            //todo first 2 should be committed together and third one should fail.
            let p = cl.create_transfers(&vec![regular_transfer.clone(), resolving_transfer.clone()]);
            println!("{:?}", &p);
            let acc1_after = acc_service.get_account_by_id(&accs[0]).unwrap();
            let acc2_after = acc_service.get_account_by_id(&accs[1]).unwrap();
            assert_eq!(acc1.debits_posted, acc1_after.debits_posted);
            assert_eq!(acc1.debits_pending, acc1_after.debits_pending);
            assert_eq!(acc1.credits_pending, acc1_after.credits_pending);
            assert_eq!(acc1.credits_posted, acc1_after.credits_posted);
            assert_eq!(acc2.debits_posted, acc2_after.debits_posted);
            assert_eq!(acc2.debits_pending, acc2_after.debits_pending);
            assert_eq!(acc2.credits_pending, acc2_after.credits_pending);
            assert_eq!(acc2.credits_posted, acc2_after.credits_posted);
            assert!(cl.get_transfers_by_id(regular_transfer.id).is_none());
            assert!(cl.get_transfers_by_id(resolving_transfer.id).is_none());
            assert_eq!(2, p.len());
            for x in p {
                assert!(!x.committed);
                assert_eq!(1, x.reason.len());
            }
        }
    }


    #[rstest]
    fn should_not_commit_transactions_which_have_been_already_persisted_idempotency() {
        let port = get_postgres_image_port();
        let postgres_client = create_postgres_client(port);
        let accs = create_two_accounts_for_transfer();
        let mut cl = LedgerTransferDaoPostgresImpl { postgres_client };

        let initial_transfers = generate_random_transfers(accs[0], accs[1], 100, 1, 1);
        let re1 = cl.create_transfers(&initial_transfers);
        assert_eq!(re1.len(), 1);
        assert!(re1.first().unwrap().committed);
        let re2 = cl.create_transfers(&initial_transfers);
        assert_eq!(re2.len(), 1);
        assert!(!re2.first().unwrap().committed);
        assert_eq!(re2.first().unwrap().reason.len(), 1);
        assert_eq!(re2.first().unwrap().reason[0], "transfer already exists with this id");
        let mut more_transfers = generate_random_transfers(accs[0], accs[1], 100, 1, 2);
        for initial_transfer in initial_transfers {
            more_transfers.push(initial_transfer);
        }
        let mut acc_service = get_account_service_for_test(create_postgres_client(port));
        let acc1 = acc_service.get_account_by_id(&accs[0]).unwrap();
        let acc2 = acc_service.get_account_by_id(&accs[1]).unwrap();
        let re3 = cl.create_transfers(&more_transfers);
        println!("{:?}", re3);
        let acc1_after = acc_service.get_account_by_id(&accs[0]).unwrap();
        let acc2_after = acc_service.get_account_by_id(&accs[1]).unwrap();
        assert_eq!(acc1.debits_posted, acc1_after.debits_posted);
        assert_eq!(acc1.debits_pending, acc1_after.debits_pending);
        assert_eq!(acc1.credits_pending, acc1_after.credits_pending);
        assert_eq!(acc1.credits_posted, acc1_after.credits_posted);
        assert_eq!(acc2.debits_posted, acc2_after.debits_posted);
        assert_eq!(acc2.debits_pending, acc2_after.debits_pending);
        assert_eq!(acc2.credits_pending, acc2_after.credits_pending);
        assert_eq!(acc2.credits_posted, acc2_after.credits_posted);
        assert_eq!(re3.len(), 3);
        for re in re3 {
            assert!(!re.committed);
            assert_eq!(re.reason.len(), 1);
            assert!(re.reason[0] == "transfer already exists with this id" || re.reason[0] == "linked transfer failed")
        }
    }

    #[test]
    #[should_panic]
    fn should_fail_for_more_than_600_in_batch() {
        let port = get_postgres_image_port();
        let postgres_client = create_postgres_client(port);
        let accs = create_two_accounts_for_transfer();
        let mut cl = LedgerTransferDaoPostgresImpl { postgres_client };
        let transfer_candidates = generate_random_transfers(accs[0], accs[1], 100, 1, 601);
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
        let accs = create_two_accounts_for_transfer();

        let mut cl = LedgerTransferDaoPostgresImpl { postgres_client };
        let mut acc_service = get_account_service_for_test(create_postgres_client(port));
        let acc1 = acc_service.get_account_by_id(&accs[0]).unwrap();
        let acc2 = acc_service.get_account_by_id(&accs[1]).unwrap();
        let transfer_candidates = generate_random_transfers(accs[0], accs[1], 100, 1, size);
        let p = cl.create_transfers(&transfer_candidates);
        let mut acc_service = get_account_service_for_test(create_postgres_client(port));
        let acc1_after = acc_service.get_account_by_id(&accs[0]).unwrap();
        let acc2_after = acc_service.get_account_by_id(&accs[1]).unwrap();
        let sum: i64 = transfer_candidates.iter().map(|a| a.amount).sum();
        assert_eq!(acc1.debits_posted + sum, acc1_after.debits_posted);
        assert_eq!(acc1.debits_pending, acc1_after.debits_pending);
        assert_eq!(acc1.credits_pending, acc1_after.credits_pending);
        assert_eq!(acc1.credits_posted, acc1_after.credits_posted);
        assert_eq!(acc2.debits_posted, acc2_after.debits_posted);
        assert_eq!(acc2.debits_pending, acc2_after.debits_pending);
        assert_eq!(acc2.credits_pending, acc2_after.credits_pending);
        assert_eq!(acc2.credits_posted + sum, acc2_after.credits_posted);
        assert_eq!(size, p.len());
        for i in 0..size {
            assert!(p[i].committed);
            assert!(p[i].reason.is_empty());
            assert_eq!(transfer_candidates[i].id, p[i].txn_id)
        }
    }

    #[rstest]
    #[case::debit_acc_wrong(- 100, 2, 1, false, true)]
    #[case::debit_acc_wrong(- 100, 2, 3, false, true)]
    #[case::credit_acc_wrong(1, - 200, 1, true, false)]
    #[case::credit_acc_wrong(1, - 200, 3, true, false)]
    #[case::both_wrong(- 100, - 200, 1, false, false)]
    #[case::both_wrong(- 100, - 200, 3, false, false)]
    fn should_fail_if_account_not_present(
        #[case]debit_acc_id: i32,
        #[case]credit_acc_id: i32,
        #[case] size: usize,
        #[case]debit_account_correct: bool,
        #[case]credit_account_correct: bool)
    {
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
                                                         #[case] credit_account_ledger_same: bool, #[case] transfer_ledger_id_same: bool)
    {
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
        println!("{:?}", p[0].reason);
        assert_eq!(p.first().unwrap().reason.len(), 1);
        let err_message = format!("accounts must have the same ledger debit_acc_ledger_id: {}, credit_acc_ledger_id: {}, transfer ledger id: {}", db_acc_led_id, cr_acc_led_id, tr_led_id);
        assert_eq!(p.first().unwrap().reason.first().unwrap(), err_message.as_str());
    }

    #[rstest]
    #[case(- 1)]
    #[case(- 0)]
    fn should_fail_transfer_amounts_of_less_than_equal_to_zero(#[case] amount: i64)
    {
        let port = get_postgres_image_port();
        let postgres_client = create_postgres_client(port);
        let accs = create_two_accounts_for_transfer();
        let mut acc_service = get_account_service_for_test(create_postgres_client(port));
        let acc1 = acc_service.get_account_by_id(&accs[0]).unwrap();
        let acc2 = acc_service.get_account_by_id(&accs[1]).unwrap();
        let mut cl = LedgerTransferDaoPostgresImpl { postgres_client };
        let transfer_candidates = generate_random_transfers(accs[0], accs[1], amount, 1, 1);
        let p = cl.create_transfers(&transfer_candidates);
        let acc1_after = acc_service.get_account_by_id(&accs[0]).unwrap();
        let acc2_after = acc_service.get_account_by_id(&accs[1]).unwrap();
        assert_eq!(acc1.debits_posted, acc1_after.debits_posted);
        assert_eq!(acc1.debits_pending, acc1_after.debits_pending);
        assert_eq!(acc1.credits_pending, acc1_after.credits_pending);
        assert_eq!(acc1.credits_posted, acc1_after.credits_posted);
        assert_eq!(acc2.debits_posted, acc2_after.debits_posted);
        assert_eq!(acc2.debits_pending, acc2_after.debits_pending);
        assert_eq!(acc2.credits_pending, acc2_after.credits_pending);
        assert_eq!(acc2.credits_posted, acc2_after.credits_posted);
        assert_eq!(p.len(), 1);
        assert!(!p.first().unwrap().committed);
        assert_eq!(p.first().unwrap().reason.len(), 1);
        assert_eq!(p.first().unwrap().reason[0], format!("transfer amount cannot be <=0 but was {}", amount).as_str());
    }
}