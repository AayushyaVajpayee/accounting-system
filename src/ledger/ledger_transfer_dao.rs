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


#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::test_utils::test_utils_postgres::get_postgres_image_port;

    #[rstest]
    #[case::empty_list_of_transfer()]
    #[case::successful_create_transfer_batch()]
    #[case::success_one_linked_batch_should_pass_and_other_should_fail()]
    ///todo test for all error messages
    /// todo test for idempotency
    /// todo test for max limit
    /// todo test for exceptions. there will be many cases to handle.like, different constraint violations,duplicate entries, some error etc
    /// todo test for 10, 100, and 1000 batch insertions to see how these behave
    /// todo test with and without exception block to see what difference does it make
    ///
    fn test_create_ledger_transfers_procedure() {
        let k = get_postgres_image_port();
    }
}