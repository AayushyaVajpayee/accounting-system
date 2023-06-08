mod currency_service;
mod currency_dao;

pub trait CurrencyDao{
    fn get_currency_entry_by_id();
    fn create_currency_entry();
    fn update_currency_entry();
    fn delete_currency_entry();
}
pub struct CurrencyDaoPostgresImpl{

}

impl CurrencyDao for CurrencyDaoPostgresImpl{
    fn get_currency_entry_by_id() {
        todo!()
    }

    fn create_currency_entry() {
        todo!()
    }

    fn update_currency_entry() {
        todo!()
    }

    fn delete_currency_entry() {
        todo!()
    }
}