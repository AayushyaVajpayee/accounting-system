use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use deadpool_postgres::Pool;
use tokio_postgres::{Row, SimpleQueryMessage};
use uuid::Uuid;

use crate::ledger::ledger_models::{Transfer, TransferCreationDbResponse};
use crate::ledger::ledger_models::TransferType::{Pending, PostPending, Regular, VoidPending};

#[async_trait]
pub trait LedgerTransferDao: Send + Sync {
    async fn create_transfers(&self, transfers: &[Transfer]) -> Vec<TransferCreationDbResponse>;
    async fn create_batch_transfers(&self, transfers: &[Vec<Transfer>]) -> Vec<Vec<TransferCreationDbResponse>>;
    async fn get_transfers_by_id(&self, id: Uuid) -> Option<Transfer>;
    async fn get_transfers_for_account_for_interval(&self);
}

struct LedgerTransferDaoPostgresImpl {
    postgres_client: &'static Pool,
}

const LEDGER_TRANSFER_POSTGRES_SELECT_FIELDS: &str = "id,tenant_id,caused_by_event_id,grouping_id,\
debit_account_id,credit_account_id,pending_id,ledger_master_id,code,\
amount,remarks,transfer_type,created_at";
const LEDGER_TRANSFER_TABLE_NAME: &str = "transfer";
static TRANSFER_BY_ID_QUERY: OnceLock<String> = OnceLock::new();

impl TryFrom<&Row> for Transfer {
    type Error = ();

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let transfer_type_numeric_code: i16 = row.get(11);
        let transfer_type = match transfer_type_numeric_code {
            1 => Regular,
            2 => Pending,
            3 => PostPending { pending_id: row.get(6) },
            4 => VoidPending { pending_id: row.get(6) },
            _ => panic!("{} is not mapped to transferType enum", transfer_type_numeric_code)//todo error handling  part. also will have to write test for this too
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


pub fn get_ledger_transfer_dao(pool: &'static Pool) -> Arc<dyn LedgerTransferDao> {
    Arc::new(LedgerTransferDaoPostgresImpl { postgres_client: pool })
}

#[async_trait]
impl LedgerTransferDao for LedgerTransferDaoPostgresImpl {
    async fn create_transfers(&self, transfers: &[Transfer]) -> Vec<TransferCreationDbResponse> {
        let query = convert_transfers_to_postgres_array(transfers);
        let conn = self.postgres_client.get().await.unwrap();
        let query_results = conn.simple_query(&query).await;
        let query_results = &query_results.unwrap()[1];
        let transfers_db_response =
            match query_results {
                SimpleQueryMessage::Row(row) => {
                    let raw_str = row.get(0);
                    serde_json::from_str::<Vec<TransferCreationDbResponse>>(raw_str.unwrap()).unwrap()
                }
                SimpleQueryMessage::CommandComplete(_) => { todo!() } //todo this will come in error handling part. will have to write test for this too. writing test may not be possible for this
                _ => { todo!() }  //todo this will come in error handling part. will have to write test for this too. writing test may not be possible for this
            };
        transfers_db_response
    }

    async fn create_batch_transfers(&self, transfers: &[Vec<Transfer>]) -> Vec<Vec<TransferCreationDbResponse>> {
        if transfers.is_empty() {
            return vec![];
        }
        let formatted_array = format!("start transaction isolation level repeatable read;\
        select batch_process_linked_transfers(array[{}]::transfer[][]);\
        commit;", transfers.iter().map(
            |a| {
                format!("[{}]", a.iter()
                    .map(convert_transfer_to_postgres_composite_type_input_string)
                    .collect::<Vec<String>>().join(","))
            }
        ).collect::<Vec<String>>().join(","));
        let conn = &self.postgres_client.get().await.unwrap();
        let query_results = &conn.simple_query(&formatted_array).await.unwrap()[1];
        let batch_transfers_response = match query_results {
            SimpleQueryMessage::Row(r) => {
                serde_json::from_str::<Vec<Vec<TransferCreationDbResponse>>>(r.get(0).unwrap()).unwrap()
            }
            SimpleQueryMessage::CommandComplete(_) => { todo!() }//todo this will come in error handling part. will have to write test for this too. writing test may not be possible for this
            _ => { todo!() }//todo this will come in error handling part. will have to write test for this too. writing test may not be possible for this
        };
        batch_transfers_response
    }


    async fn get_transfers_by_id(&self, id: Uuid) -> Option<Transfer> {
        let query = LedgerTransferDaoPostgresImpl::get_transfer_by_id_query();
        let conn = self.postgres_client.get().await.unwrap();
        let k = conn.query(
            query,
            &[&id],
        ).await.unwrap();
        k.iter().map(|row|
            row.try_into().unwrap()
        ).next()
    }

    async fn get_transfers_for_account_for_interval(&self) {
        todo!()
    }
}

fn convert_transfer_to_postgres_composite_type_input_string(transfer: &Transfer) -> String {
    format!("('{}','{}','{}','{}','{}','{}',{},'{}',{},{},{},{},{})",
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

fn convert_transfers_to_postgres_array(transfers: &[Transfer]) -> String {
    format!("start transaction isolation level REPEATABLE READ;\
    select create_linked_transfers(array[{}]::transfer[]);\
    commit;", transfers
        .iter()
        .map(convert_transfer_to_postgres_composite_type_input_string)
        .collect::<Vec<String>>().join(","))
}

#[cfg(test)]
mod tests {
    use std::ops::Not;
    use std::time::{SystemTime, UNIX_EPOCH};

    use rand::Rng;
    use rstest::rstest;
    use uuid::Uuid;
    use crate::accounting::account::account_models::tests::{a_create_account_request, CreateAccountRequestTestBuilder, SEED_CREDIT_ACCOUNT_ID, SEED_DEBIT_ACCOUNT_ID};

    use crate::accounting::account::account_service::get_account_service_for_test;
    use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
    use crate::ledger::ledger_models::{Transfer, TransferBuilder};
    use crate::ledger::ledger_models::tests::a_transfer;
    use crate::ledger::ledger_transfer_dao::{LedgerTransferDao, LedgerTransferDaoPostgresImpl};
    use crate::ledger::ledgermaster::ledger_master_models::{a_create_ledger_master_entry_request, CreateLedgerMasterEntryRequestTestBuilder, SEED_LEDGER_MASTER_ID};
    use crate::ledger::ledgermaster::ledger_master_service::get_ledger_master_service_for_test;

    /// need this so that every test case can act on different set of accounts and we can
    /// verify before-after account balance of transfers.
    /// returns account ids created
    async fn create_two_accounts_for_transfer() -> Vec<Uuid> {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let account_service = get_account_service_for_test(postgres_client);
        let a1 = a_create_account_request(CreateAccountRequestTestBuilder {
            ..Default::default()
        });
        let a2 = a_create_account_request(CreateAccountRequestTestBuilder {
            ..Default::default()
        });
        let a1_id = account_service.create_account(&a1).await;
        let a2_id = account_service.create_account(&a2).await;
        vec![a1_id, a2_id]
    }

    fn generate_random_transfers(debit_account: Uuid, credit_account: Uuid, amount: i64, ledger_master_id: Uuid, size: usize) -> Vec<Transfer> {
        let mut transfers: Vec<Transfer> = Vec::with_capacity(size);
        for _i in 0..size {
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

    mod create_batch_transfer_tests {
        use rstest::rstest;
        use uuid::Uuid;

        use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
        use crate::ledger::ledger_models::Transfer;
        use crate::ledger::ledger_transfer_dao::{LedgerTransferDao, LedgerTransferDaoPostgresImpl};
        use crate::ledger::ledger_transfer_dao::tests::{create_two_accounts_for_transfer, generate_random_transfers};
        use crate::ledger::ledgermaster::ledger_master_models::SEED_LEDGER_MASTER_ID;

        #[rstest]
        #[trace]
        async fn should_be_able_to_post_multiple_linked_transfers(
            #[values(0, 1, 2)]outer_arr_size: i32,
            #[values(1, 2)]inner_arr_size: i32)
        {
            let port = get_postgres_image_port().await;
            let postgres_client = get_postgres_conn_pool(port).await;
            let accs = create_two_accounts_for_transfer().await;
            let ledger_transfer_dao = LedgerTransferDaoPostgresImpl { postgres_client };
            let transfer = generate_random_transfers(accs[0],
                                                     accs[1],
                                                     100,
                                                     *SEED_LEDGER_MASTER_ID,
                                                     (outer_arr_size * inner_arr_size) as usize)
                .chunks(inner_arr_size as usize).map(|a| a.to_vec()).collect::<Vec<Vec<Transfer>>>();
            let batch_transfer_responses = ledger_transfer_dao.create_batch_transfers(&transfer).await;
            println!("{:?}", batch_transfer_responses);
            for batch in batch_transfer_responses {
                for trf_resp in batch {
                    assert!(trf_resp.committed);
                    assert!(trf_resp.reason.is_empty())
                }
            }
        }

        #[tokio::test]
        async fn should_fail_only_one_batch_due_to_error_and_not_others() {
            let port = get_postgres_image_port().await;
            let postgres_client = get_postgres_conn_pool(port).await;
            let accs = create_two_accounts_for_transfer().await;
            let mut ledger_transfer_dao = LedgerTransferDaoPostgresImpl { postgres_client };
            let mut a_trf = generate_random_transfers(accs[0],
                                                      accs[1],
                                                      100,
                                                      *SEED_LEDGER_MASTER_ID,
                                                      2)
                .chunks(2_usize)
                .map(|a| a.to_vec())
                .collect::<Vec<Vec<Transfer>>>();
            let sample_trfs = generate_random_transfers(
                Uuid::now_v7(),
                Uuid::now_v7(),
                100,
                *SEED_LEDGER_MASTER_ID,
                2);
            a_trf.push(sample_trfs);
            let batch_trfs_resp = ledger_transfer_dao.create_batch_transfers(&a_trf).await;
            println!("{:?}", batch_trfs_resp);
            for trf_resp in &batch_trfs_resp[0] {
                assert!(trf_resp.committed);
                assert!(trf_resp.reason.is_empty());
                assert!(ledger_transfer_dao.get_transfers_by_id(trf_resp.txn_id).await.is_some())
            }
            for trf_resp in &batch_trfs_resp[1] {
                assert!(!trf_resp.committed);
                assert!(!trf_resp.reason.is_empty());
                assert!(ledger_transfer_dao.get_transfers_by_id(trf_resp.txn_id).await.is_none())
            }
        }

        #[tokio::test]
        #[should_panic]
        async fn should_panic_if_transfer_more_than_500() {
            let port = get_postgres_image_port().await;
            let postgres_client = get_postgres_conn_pool(port).await;
            let accs = create_two_accounts_for_transfer().await;
            let mut ledger_transfer_dao = LedgerTransferDaoPostgresImpl { postgres_client };
            let mut a_trf = generate_random_transfers(accs[0],
                                                      accs[1],
                                                      100,
                                                      *SEED_LEDGER_MASTER_ID,
                                                      502)
                .chunks(50usize)
                .map(|a| a.to_vec())
                .collect::<Vec<Vec<Transfer>>>();
            let sample_trfs = generate_random_transfers(
                Uuid::now_v7(),
                Uuid::now_v7(),
                100,
                *SEED_LEDGER_MASTER_ID,
                2);
            a_trf.push(sample_trfs);
            let batch_trf_resps = ledger_transfer_dao.create_batch_transfers(&a_trf).await;
            println!("{:?}", batch_trf_resps);
        }
    }

    mod transfer_type_pending_tests {
        use rstest::rstest;

        use crate::accounting::account::account_service::get_account_service_for_test;
        use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
        use crate::ledger::ledger_models::{TransferBuilder, TransferType};
        use crate::ledger::ledger_models::tests::a_transfer;
        use crate::ledger::ledger_models::TransferType::{Pending, Regular};
        use crate::ledger::ledger_transfer_dao::{LedgerTransferDao, LedgerTransferDaoPostgresImpl};
        use crate::ledger::ledger_transfer_dao::tests::create_two_accounts_for_transfer;

        #[rstest]
        #[case::regular_entry(Some(Regular))]
        #[case::pending_entry(Some(Pending))]
        async fn test_posting_a_pending_entry(#[case] entry_type: Option<TransferType>) {
            let port = get_postgres_image_port().await;
            let postgres_client = get_postgres_conn_pool(port).await;
            let led_trf_dao = LedgerTransferDaoPostgresImpl { postgres_client };
            let accs = create_two_accounts_for_transfer().await;
            let acc_ser = get_account_service_for_test(get_postgres_conn_pool(port).await);
            let acc1 = acc_ser.get_account_by_id(&accs[0]).await.unwrap();
            let acc2 = acc_ser.get_account_by_id(&accs[1]).await.unwrap();
            let a_trf = a_transfer(TransferBuilder {
                transfer_type: entry_type.clone(),
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                ..Default::default()
            });
            let trfs_resp = led_trf_dao.create_transfers(&vec![a_trf.clone()]).await;
            let fetched_trfs = led_trf_dao.get_transfers_by_id(a_trf.id).await.unwrap();
            let acc_1_after = acc_ser.get_account_by_id(&accs[0]).await.unwrap();
            let acc_2_after = acc_ser.get_account_by_id(&accs[1]).await.unwrap();
            if entry_type.clone().unwrap() == Pending {
                assert_eq!(acc1.credits_posted, acc_1_after.credits_posted);
                assert_eq!(acc1.debits_posted, acc_1_after.debits_posted);
                assert_eq!(acc2.credits_posted, acc_2_after.credits_posted);
                assert_eq!(acc2.debits_posted, acc_2_after.debits_posted);
                assert_eq!(acc1.debits_pending + 100, acc_1_after.debits_pending);
                assert_eq!(acc2.credits_pending + 100, acc_2_after.credits_pending)
            }
            if entry_type.clone().unwrap() == Regular {
                assert_eq!(acc1.credits_pending, acc_1_after.credits_pending);
                assert_eq!(acc1.debits_pending, acc_1_after.debits_pending);
                assert_eq!(acc2.credits_pending, acc_2_after.credits_pending);
                assert_eq!(acc2.debits_pending, acc_2_after.debits_pending);
                assert_eq!(acc1.debits_posted + 100, acc_1_after.debits_posted);
                assert_eq!(acc2.credits_posted + 100, acc_2_after.credits_posted)
            }

            assert_eq!(fetched_trfs.transfer_type, entry_type.unwrap());
            assert_eq!(1, trfs_resp.len());
            assert!(trfs_resp[0].committed);
            assert_eq!(0, trfs_resp[0].reason.len());
        }
    }

    mod pending_transfer_resolution_tests {
        use rstest::rstest;
        use uuid::Uuid;

        use crate::accounting::account::account_service::get_account_service_for_test;
        use crate::accounting::postgres_factory::test_utils_postgres::{get_postgres_conn_pool, get_postgres_image_port};
        use crate::ledger::ledger_models::{Transfer, TransferBuilder, TransferType};
        use crate::ledger::ledger_models::tests::a_transfer;
        use crate::ledger::ledger_transfer_dao::{LedgerTransferDao, LedgerTransferDaoPostgresImpl};
        use crate::ledger::ledger_transfer_dao::tests::create_two_accounts_for_transfer;

        async fn pending_transfer() -> Transfer {
            let accs = create_two_accounts_for_transfer().await;
            a_transfer(TransferBuilder {
                transfer_type: Some(TransferType::Pending),
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(100),
                ..Default::default()
            })
        }

        #[rstest]
        #[case::should_post_pending_entry_for_a_pending_entry_in_full(pending_transfer().await,
        Some(TransferType::PostPending{pending_id: Uuid::now_v7()}),
        Some(100))]
        #[case::should_post_pending_entry_for_a_pending_entry_partially(pending_transfer().await,
        Some(TransferType::PostPending{pending_id: Uuid::now_v7()}),
        Some(99))]
        #[case::should_be_able_to_void_a_pending_entry(pending_transfer().await,
        Some(TransferType::VoidPending{pending_id: Uuid::now_v7()}),
        Some(100))]
        async fn should_be_able_to_resolve_pending_transfer(
            #[case] pending_transfer: Transfer,
            #[case] pending_resolution_type: Option<TransferType>, #[case] pending_resolution_amount: Option<i64>)
        {
            let pending_resolution_type = match pending_resolution_type.unwrap() {
                TransferType::Regular | TransferType::Pending => { panic!("invalid state") }
                TransferType::PostPending { .. } => { TransferType::PostPending { pending_id: pending_transfer.id } }
                TransferType::VoidPending { .. } => { TransferType::VoidPending { pending_id: pending_transfer.id } }
            };
            let port = get_postgres_image_port().await;
            let postgres_client = get_postgres_conn_pool(port).await;
            let cl = LedgerTransferDaoPostgresImpl { postgres_client };
            let resolved_pending_transfer = a_transfer(TransferBuilder {
                transfer_type: Some(pending_resolution_type),
                debit_account_id: Some(pending_transfer.debit_account_id),
                credit_account_id: Some(pending_transfer.credit_account_id),
                amount: pending_resolution_amount,
                ..Default::default()
            });
            let acc_ser = get_account_service_for_test(get_postgres_conn_pool(port).await);
            let acc1 = acc_ser.get_account_by_id(&resolved_pending_transfer.debit_account_id).await.unwrap();
            let acc2 = acc_ser.get_account_by_id(&resolved_pending_transfer.credit_account_id).await.unwrap();
            let trf_resps = cl.create_transfers(&vec![pending_transfer, resolved_pending_transfer.clone()]).await;
            println!("{:?}", trf_resps);
            let acc1_after = acc_ser.get_account_by_id(&resolved_pending_transfer.debit_account_id).await.unwrap();
            let acc2_after = acc_ser.get_account_by_id(&resolved_pending_transfer.credit_account_id).await.unwrap();
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

            let _fetched_trf_resp = cl.get_transfers_by_id(resolved_pending_transfer.id).await.unwrap();
            assert_eq!(2, trf_resps.len());
            for x in trf_resps {
                assert!(x.committed);
                assert_eq!(0, x.reason.len());
            }
        }

        #[rstest]
        #[case(Some(TransferType::PostPending{pending_id: Uuid::now_v7()}))]
        #[case(Some(TransferType::VoidPending{pending_id: Uuid::now_v7()}))]
        #[trace]
        async fn should_error_out_posting_entry_for_an_invalid_pending_entry_id(#[case] transfer_type: Option<TransferType>)
        {
            let port = get_postgres_image_port().await;
            let postgres_client = get_postgres_conn_pool(port).await;
            let accs = create_two_accounts_for_transfer().await;
            let acc_service = get_account_service_for_test(postgres_client);
            let acc1 = acc_service.get_account_by_id(&accs[0]).await.unwrap();
            let acc2 = acc_service.get_account_by_id(&accs[1]).await.unwrap();
            let led_trf_dao = LedgerTransferDaoPostgresImpl { postgres_client };
            let pending_trf = a_transfer(TransferBuilder {
                transfer_type: Some(TransferType::Pending),
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(100),
                ..Default::default()
            });
            let resolved_pending_trf = a_transfer(TransferBuilder {
                transfer_type,
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(100),
                ..Default::default()
            });
            let trf_resps = led_trf_dao.create_transfers(&vec![pending_trf.clone(), resolved_pending_trf.clone()]).await;
            println!("{:?}", &trf_resps);
            let acc1_after = acc_service.get_account_by_id(&accs[0]).await.unwrap();
            let acc2_after = acc_service.get_account_by_id(&accs[1]).await.unwrap();
            assert_eq!(acc1.debits_posted, acc1_after.debits_posted);
            assert_eq!(acc1.debits_pending, acc1_after.debits_pending);
            assert_eq!(acc1.credits_pending, acc1_after.credits_pending);
            assert_eq!(acc1.credits_posted, acc1_after.credits_posted);
            assert_eq!(acc2.debits_posted, acc2_after.debits_posted);
            assert_eq!(acc2.debits_pending, acc2_after.debits_pending);
            assert_eq!(acc2.credits_pending, acc2_after.credits_pending);
            assert_eq!(acc2.credits_posted, acc2_after.credits_posted);
            assert!(led_trf_dao.get_transfers_by_id(pending_trf.id).await.is_none());
            assert!(led_trf_dao.get_transfers_by_id(resolved_pending_trf.id).await.is_none());
            assert_eq!(2, trf_resps.len());
            for trf_resp in trf_resps {
                assert!(!trf_resp.committed);
                assert_eq!(1, trf_resp.reason.len());
            }
        }

        #[rstest]
        async fn should_not_act_on_an_already_resolved_pending_transfer(
            #[values("pp", "vp")]
            first_resolved_transfer_type: String,
            #[values("pp", "vp")]
            second_resolved_transfer_type: String,
        )
        {
            //todo may be this can be combined with above test
            let port = get_postgres_image_port().await;
            let postgres_client = get_postgres_conn_pool(port).await;
            let accs = create_two_accounts_for_transfer().await;
            let acc_service = get_account_service_for_test(get_postgres_conn_pool(port).await);
            let acc1 = acc_service.get_account_by_id(&accs[0]).await.unwrap();
            let acc2 = acc_service.get_account_by_id(&accs[1]).await.unwrap();
            let led_trf_dao = LedgerTransferDaoPostgresImpl { postgres_client };
            let pending_trf = a_transfer(TransferBuilder {
                transfer_type: Some(TransferType::Pending),
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(100),
                ..Default::default()
            });
            let first_resolved_trf = a_transfer(TransferBuilder {
                transfer_type: if first_resolved_transfer_type == "pp" {
                    Some(TransferType::PostPending { pending_id: pending_trf.id })
                } else {
                    Some(TransferType::VoidPending { pending_id: pending_trf.id })
                },
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(100),
                ..Default::default()
            });
            let second_resolved_trf = a_transfer(TransferBuilder {
                transfer_type: if second_resolved_transfer_type == "pp" {
                    Some(TransferType::PostPending { pending_id: pending_trf.id })
                } else {
                    Some(TransferType::VoidPending { pending_id: pending_trf.id })
                },
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(100),
                ..Default::default()
            });
            //todo first 2 should be committed together and third one should fail.
            let trf_resps = led_trf_dao.create_transfers(&vec![pending_trf.clone(), first_resolved_trf.clone(), second_resolved_trf.clone()]).await;
            let acc1_after = acc_service.get_account_by_id(&accs[0]).await.unwrap();
            let acc2_after = acc_service.get_account_by_id(&accs[1]).await.unwrap();
            assert_eq!(acc1.debits_posted, acc1_after.debits_posted);
            assert_eq!(acc1.debits_pending, acc1_after.debits_pending);
            assert_eq!(acc1.credits_pending, acc1_after.credits_pending);
            assert_eq!(acc1.credits_posted, acc1_after.credits_posted);
            assert_eq!(acc2.debits_posted, acc2_after.debits_posted);
            assert_eq!(acc2.debits_pending, acc2_after.debits_pending);
            assert_eq!(acc2.credits_pending, acc2_after.credits_pending);
            assert_eq!(acc2.credits_posted, acc2_after.credits_posted);
            println!("{:?}", &trf_resps);
            assert!(led_trf_dao.get_transfers_by_id(pending_trf.id).await.is_none());
            assert!(led_trf_dao.get_transfers_by_id(first_resolved_trf.id).await.is_none());
            assert!(led_trf_dao.get_transfers_by_id(second_resolved_trf.id).await.is_none());
            assert_eq!(3, trf_resps.len());
            for trf_resp in trf_resps {
                assert!(!trf_resp.committed);
                assert_eq!(1, trf_resp.reason.len());
            }
        }

        #[tokio::test]
        async fn should_not_post_pending_entry_for_a_pending_entry_in_excess() {
            let port = get_postgres_image_port().await;
            let postgres_client = get_postgres_conn_pool(port).await;
            let accs = create_two_accounts_for_transfer().await;
            let acc_service = get_account_service_for_test(get_postgres_conn_pool(port).await);
            let acc1 = acc_service.get_account_by_id(&accs[0]).await.unwrap();
            let acc2 = acc_service.get_account_by_id(&accs[1]).await.unwrap();
            let mut led_trf_dao = LedgerTransferDaoPostgresImpl { postgres_client };
            let pending_trf = a_transfer(TransferBuilder {
                transfer_type: Some(TransferType::Pending),
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(100),
                ..Default::default()
            });
            let post_pending_trf = a_transfer(TransferBuilder {
                transfer_type: Some(TransferType::PostPending { pending_id: pending_trf.id.clone() }),
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(101),
                ..Default::default()
            });
            let trf_resps = led_trf_dao.create_transfers(&vec![pending_trf.clone(), post_pending_trf.clone()]).await;
            println!("{:?}", &trf_resps);
            let acc1_after = acc_service.get_account_by_id(&accs[0]).await.unwrap();
            let acc2_after = acc_service.get_account_by_id(&accs[1]).await.unwrap();
            assert_eq!(acc1.debits_posted, acc1_after.debits_posted);
            assert_eq!(acc1.debits_pending, acc1_after.debits_pending);
            assert_eq!(acc1.credits_pending, acc1_after.credits_pending);
            assert_eq!(acc1.credits_posted, acc1_after.credits_posted);
            assert_eq!(acc2.debits_posted, acc2_after.debits_posted);
            assert_eq!(acc2.debits_pending, acc2_after.debits_pending);
            assert_eq!(acc2.credits_pending, acc2_after.credits_pending);
            assert_eq!(acc2.credits_posted, acc2_after.credits_posted);
            assert!(led_trf_dao.get_transfers_by_id(pending_trf.id).await.is_none());
            assert!(led_trf_dao.get_transfers_by_id(post_pending_trf.id).await.is_none());
            assert_eq!(2, trf_resps.len());
            for trf_resp in trf_resps {
                assert!(!trf_resp.committed);
                assert_eq!(1, trf_resp.reason.len());
            }
        }

        #[rstest]
        async fn should_not_resolve_a_post_transfer(
            #[values("pp")]resolution_type: String) {
            //todo important test
            let port = get_postgres_image_port().await;
            let postgres_client = get_postgres_conn_pool(port).await;
            let accs = create_two_accounts_for_transfer().await;
            let acc_service = get_account_service_for_test(get_postgres_conn_pool(port).await);
            let acc1 = acc_service.get_account_by_id(&accs[0]).await.unwrap();
            let acc2 = acc_service.get_account_by_id(&accs[1]).await.unwrap();
            let led_trf_dao = LedgerTransferDaoPostgresImpl { postgres_client };
            let regular_trf = a_transfer(TransferBuilder {
                transfer_type: Some(TransferType::Regular),
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(100),
                ..Default::default()
            });

            let resolving_trf = a_transfer(TransferBuilder {
                transfer_type: if resolution_type == "pp" {
                    Some(TransferType::PostPending { pending_id: regular_trf.id.clone() })
                } else {
                    Some(TransferType::VoidPending { pending_id: regular_trf.id.clone() })
                },
                debit_account_id: Some(accs[0]),
                credit_account_id: Some(accs[1]),
                amount: Some(100),
                ..Default::default()
            });
            //todo first 2 should be committed together and third one should fail.
            let trf_resps = led_trf_dao.create_transfers(&vec![regular_trf.clone(), resolving_trf.clone()]).await;
            println!("{:?}", &trf_resps);
            let acc1_after = acc_service.get_account_by_id(&accs[0]).await.unwrap();
            let acc2_after = acc_service.get_account_by_id(&accs[1]).await.unwrap();
            assert_eq!(acc1.debits_posted, acc1_after.debits_posted);
            assert_eq!(acc1.debits_pending, acc1_after.debits_pending);
            assert_eq!(acc1.credits_pending, acc1_after.credits_pending);
            assert_eq!(acc1.credits_posted, acc1_after.credits_posted);
            assert_eq!(acc2.debits_posted, acc2_after.debits_posted);
            assert_eq!(acc2.debits_pending, acc2_after.debits_pending);
            assert_eq!(acc2.credits_pending, acc2_after.credits_pending);
            assert_eq!(acc2.credits_posted, acc2_after.credits_posted);
            assert!(led_trf_dao.get_transfers_by_id(regular_trf.id).await.is_none());
            assert!(led_trf_dao.get_transfers_by_id(resolving_trf.id).await.is_none());
            assert_eq!(2, trf_resps.len());
            for trf_resp in trf_resps {
                assert!(!trf_resp.committed);
                assert_eq!(1, trf_resp.reason.len());
            }
        }
    }


    #[rstest]
    async fn should_not_commit_transactions_which_have_been_already_persisted_idempotency() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let accs = create_two_accounts_for_transfer().await;
        let led_trf_dao = LedgerTransferDaoPostgresImpl { postgres_client };

        let initial_trfs = generate_random_transfers(accs[0], accs[1], 100, *SEED_LEDGER_MASTER_ID, 1);
        let trf_resps_1 = led_trf_dao.create_transfers(&initial_trfs).await;
        assert_eq!(trf_resps_1.len(), 1);
        assert!(trf_resps_1.first().unwrap().committed);
        let trf_resps_2 = led_trf_dao.create_transfers(&initial_trfs).await;
        assert_eq!(trf_resps_2.len(), 1);
        assert!(!trf_resps_2.first().unwrap().committed);
        assert_eq!(trf_resps_2.first().unwrap().reason.len(), 1);
        assert_eq!(trf_resps_2.first().unwrap().reason[0], "transfer already exists with this id");
        let mut more_trfs = generate_random_transfers(accs[0], accs[1], 100, *SEED_LEDGER_MASTER_ID, 2);
        for initial_trf in initial_trfs {
            more_trfs.push(initial_trf);
        }
        let acc_service = get_account_service_for_test(get_postgres_conn_pool(port).await);
        let acc1 = acc_service.get_account_by_id(&accs[0]).await.unwrap();
        let acc2 = acc_service.get_account_by_id(&accs[1]).await.unwrap();
        let trf_resps_3 = led_trf_dao.create_transfers(&more_trfs).await;
        println!("{:?}", trf_resps_3);
        let acc1_after = acc_service.get_account_by_id(&accs[0]).await.unwrap();
        let acc2_after = acc_service.get_account_by_id(&accs[1]).await.unwrap();
        assert_eq!(acc1.debits_posted, acc1_after.debits_posted);
        assert_eq!(acc1.debits_pending, acc1_after.debits_pending);
        assert_eq!(acc1.credits_pending, acc1_after.credits_pending);
        assert_eq!(acc1.credits_posted, acc1_after.credits_posted);
        assert_eq!(acc2.debits_posted, acc2_after.debits_posted);
        assert_eq!(acc2.debits_pending, acc2_after.debits_pending);
        assert_eq!(acc2.credits_pending, acc2_after.credits_pending);
        assert_eq!(acc2.credits_posted, acc2_after.credits_posted);
        assert_eq!(trf_resps_3.len(), 3);
        for trf_resp in trf_resps_3 {
            assert!(!trf_resp.committed);
            assert_eq!(trf_resp.reason.len(), 1);
            assert!(trf_resp.reason[0] == "transfer already exists with this id" || trf_resp.reason[0] == "linked transfer failed")
        }
    }

    #[tokio::test]
    #[should_panic]
    async fn should_fail_for_more_than_600_in_batch() {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let accs = create_two_accounts_for_transfer().await;
        let mut led_trf_dao = LedgerTransferDaoPostgresImpl { postgres_client };
        let transfer_candidates = generate_random_transfers(accs[0], accs[1], 100, *SEED_LEDGER_MASTER_ID, 601);
        let _trf_resps = led_trf_dao.create_transfers(&transfer_candidates).await;
    }

    #[rstest]
    #[case::empty_list(0)]
    #[case::single_element(1)]
    #[case::two_elements(2)]
    #[case::eight_elements(8)]
    async fn test_successful_create_transfers_of_multiple_sizes(#[case] size: usize) {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let accs = create_two_accounts_for_transfer().await;
        let led_trf_dao = LedgerTransferDaoPostgresImpl { postgres_client };
        let acc_service = get_account_service_for_test(get_postgres_conn_pool(port).await);
        let acc1 = acc_service.get_account_by_id(&accs[0]).await.unwrap();
        let acc2 = acc_service.get_account_by_id(&accs[1]).await.unwrap();
        let transfer_candidates = generate_random_transfers(accs[0], accs[1], 100, *SEED_LEDGER_MASTER_ID, size);
        let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let trf_resps = led_trf_dao.create_transfers(&transfer_candidates).await;
        let _a = led_trf_dao.create_transfers(&transfer_candidates).await;
        let stop = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        println!("{}", stop - start);
        let acc_service = get_account_service_for_test(get_postgres_conn_pool(port).await);
        let acc1_after = acc_service.get_account_by_id(&accs[0]).await.unwrap();
        let acc2_after = acc_service.get_account_by_id(&accs[1]).await.unwrap();
        let sum: i64 = transfer_candidates.iter().map(|a| a.amount).sum();
        assert_eq!(acc1.debits_posted + sum, acc1_after.debits_posted);
        assert_eq!(acc1.debits_pending, acc1_after.debits_pending);
        assert_eq!(acc1.credits_pending, acc1_after.credits_pending);
        assert_eq!(acc1.credits_posted, acc1_after.credits_posted);
        assert_eq!(acc2.debits_posted, acc2_after.debits_posted);
        assert_eq!(acc2.debits_pending, acc2_after.debits_pending);
        assert_eq!(acc2.credits_pending, acc2_after.credits_pending);
        assert_eq!(acc2.credits_posted + sum, acc2_after.credits_posted);
        assert_eq!(size, trf_resps.len());
        for i in 0..size {
            assert!(trf_resps[i].committed);
            assert!(trf_resps[i].reason.is_empty());
            assert_eq!(transfer_candidates[i].id, trf_resps[i].txn_id)
        }
    }

    #[rstest]
    #[case::debit_acc_wrong(Uuid::now_v7(), * SEED_CREDIT_ACCOUNT_ID, 1, false, true)]
    #[case::debit_acc_wrong(Uuid::now_v7(), * SEED_CREDIT_ACCOUNT_ID, 3, false, true)]
    #[case::credit_acc_wrong(* SEED_DEBIT_ACCOUNT_ID, Uuid::now_v7(), 1, true, false)]
    #[case::credit_acc_wrong(* SEED_DEBIT_ACCOUNT_ID, Uuid::now_v7(), 3, true, false)]
    #[case::both_wrong(Uuid::now_v7(), Uuid::now_v7(), 1, false, false)]
    #[case::both_wrong(Uuid::now_v7(), Uuid::now_v7(), 3, false, false)]
    async fn should_fail_if_account_not_present(
        #[case]debit_acc_id: Uuid,
        #[case]credit_acc_id: Uuid,
        #[case] size: usize,
        #[case]debit_account_correct: bool,
        #[case]credit_account_correct: bool)
    {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let led_trf_dao = LedgerTransferDaoPostgresImpl { postgres_client };
        let transfer_candidates = generate_random_transfers(debit_acc_id, credit_acc_id, 100, *SEED_LEDGER_MASTER_ID, size);
        let trf_resps = led_trf_dao.create_transfers(&transfer_candidates).await;
        println!("{:?}", trf_resps);
        for i in 0..size {
            let response = &trf_resps[i];
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
                assert_eq!(response.reason[0], "linked transfer failed");
            }
        }
    }

    #[rstest]
    #[case::transfer_ledger_id_diff(false, false, true)]
    #[case::cred_acc_ledger_id_diff(false, true, false)]
    #[case::deb_acc_led_id_diff(true, false, false)]
    #[case::all_diff(true, true, true)]
    async fn should_fail_if_accounts_have_different_ledger_ids(#[case] debit_account_ledger_id_same: bool,
                                                               #[case] credit_account_ledger_same: bool, #[case] transfer_ledger_id_same: bool)
    {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let led_trf_dao = LedgerTransferDaoPostgresImpl { postgres_client };
        let ledger_master_service = get_ledger_master_service_for_test(
            get_postgres_conn_pool(port).await);
        let led_mst_req = a_create_ledger_master_entry_request(CreateLedgerMasterEntryRequestTestBuilder {
            ..Default::default()
        });
        let account_master_service = get_account_service_for_test(get_postgres_conn_pool(port).await);
        let id = Uuid::now_v7();
        let id2 = Uuid::now_v7();
        let mut db_acc_led_id = *SEED_LEDGER_MASTER_ID;
        let mut cr_acc_led_id = *SEED_LEDGER_MASTER_ID;
        let mut tr_led_id = *SEED_LEDGER_MASTER_ID;
        let mut db_acc_id = *SEED_DEBIT_ACCOUNT_ID;
        let mut cr_acc_id = *SEED_CREDIT_ACCOUNT_ID;
        if !debit_account_ledger_id_same {
            db_acc_led_id = ledger_master_service.create_ledger_master_entry(&led_mst_req).await;
            db_acc_id = account_master_service.create_account(
                &a_create_account_request(
                    CreateAccountRequestTestBuilder {
                        ledger_master_id: Some(db_acc_led_id),
                        ..Default::default()
                    })).await;
        }
        if !credit_account_ledger_same {
            cr_acc_led_id = ledger_master_service.create_ledger_master_entry(&led_mst_req).await;
            cr_acc_id = account_master_service.create_account(
                &a_create_account_request(
                    CreateAccountRequestTestBuilder {
                        ledger_master_id: Some(cr_acc_led_id),
                        ..Default::default()
                    })).await;
        }
        if transfer_ledger_id_same {
            tr_led_id = ledger_master_service.create_ledger_master_entry(&led_mst_req).await;
        }
        let transfer_candidates = generate_random_transfers(
            db_acc_id,
            cr_acc_id,
            100,
            tr_led_id, 1);
        let trf_resps = led_trf_dao.create_transfers(&transfer_candidates).await;
        assert_eq!(trf_resps.len(), 1);
        assert!(!trf_resps.first().unwrap().committed);
        println!("{:?}", trf_resps[0].reason);
        assert_eq!(trf_resps.first().unwrap().reason.len(), 1);
        let err_message = format!("accounts must have the same ledger debit_acc_ledger_id: {}, credit_acc_ledger_id: {}, transfer ledger id: {}", db_acc_led_id, cr_acc_led_id, tr_led_id);
        assert_eq!(trf_resps.first().unwrap().reason.first().unwrap(), err_message.as_str());
    }

    #[rstest]
    #[case(- 1)]
    #[case(- 0)]
    async fn should_fail_transfer_amounts_of_less_than_equal_to_zero(#[case] amount: i64)
    {
        let port = get_postgres_image_port().await;
        let postgres_client = get_postgres_conn_pool(port).await;
        let accs = create_two_accounts_for_transfer().await;
        let acc_service = get_account_service_for_test(get_postgres_conn_pool(port).await);
        let acc1 = acc_service.get_account_by_id(&accs[0]).await.unwrap();
        let acc2 = acc_service.get_account_by_id(&accs[1]).await.unwrap();
        let led_trf_dao = LedgerTransferDaoPostgresImpl { postgres_client };
        let transfer_candidates = generate_random_transfers(accs[0], accs[1], amount, *SEED_LEDGER_MASTER_ID, 1);
        let trf_resps = led_trf_dao.create_transfers(&transfer_candidates).await;
        let acc1_after = acc_service.get_account_by_id(&accs[0]).await.unwrap();
        let acc2_after = acc_service.get_account_by_id(&accs[1]).await.unwrap();
        assert_eq!(acc1.debits_posted, acc1_after.debits_posted);
        assert_eq!(acc1.debits_pending, acc1_after.debits_pending);
        assert_eq!(acc1.credits_pending, acc1_after.credits_pending);
        assert_eq!(acc1.credits_posted, acc1_after.credits_posted);
        assert_eq!(acc2.debits_posted, acc2_after.debits_posted);
        assert_eq!(acc2.debits_pending, acc2_after.debits_pending);
        assert_eq!(acc2.credits_pending, acc2_after.credits_pending);
        assert_eq!(acc2.credits_posted, acc2_after.credits_posted);
        assert_eq!(trf_resps.len(), 1);
        assert!(!trf_resps.first().unwrap().committed);
        assert_eq!(trf_resps.first().unwrap().reason.len(), 1);
        assert_eq!(trf_resps.first().unwrap().reason[0], format!("transfer amount cannot be <=0 but was {}", amount).as_str());
    }
}