use postgres::Client;

pub trait LedgerTransferDao {
    ///
    ///
    fn create_transfers();
    fn get_transfers_by_id();
    fn get_transfers_for_account_for_interval();
}

pub struct LedgerTransferDaoPostgresImpl {
    postgres_client: Client,
}

