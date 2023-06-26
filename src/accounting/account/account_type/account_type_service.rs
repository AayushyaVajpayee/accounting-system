use crate::accounting::account::account_type::account_type_dao::AccountTypeDao;

pub struct AccountTypeService {
    account_type_dao: Box<dyn AccountTypeDao>,

}

struct AccountTypeHierarchy {
    current_account_id: i16,
    child_account_types: Vec<AccountTypeService>,

}

impl AccountTypeService {
    pub fn get_account_type_hierarchy(&mut self, tenant_id: &i32) {
        self.account_type_dao.get_all_account_types_for_tenant_id(&tenant_id);
    }
}
