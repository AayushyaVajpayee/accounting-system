pub trait CurrencyService {
    fn create_currency_entry();
    fn get_currency_entry();
    fn update_currency_entry();
    fn delete_currency_entry();
}

struct CurrencyServiceImpl {}


impl CurrencyService for CurrencyServiceImpl {
    fn create_currency_entry() {}
    fn get_currency_entry() {}
    fn update_currency_entry() {}
    fn delete_currency_entry() {}
}