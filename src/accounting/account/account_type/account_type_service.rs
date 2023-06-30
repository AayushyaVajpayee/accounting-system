use std::collections::HashMap;
use std::ops::{Deref, Not};

use serde::Serialize;

use crate::accounting::account::account_type::account_type_dao::AccountTypeDao;
use crate::accounting::account::account_type::account_type_models::AccountTypeMaster;

pub struct AccountTypeService {
    account_type_dao: Box<dyn AccountTypeDao>,

}

#[derive(Debug, Serialize)]
pub struct AccountTypeHierarchy {
    current_account_id: i16,
    child_account_types: Vec<AccountTypeHierarchy>,

}

impl AccountTypeService {
    pub fn get_account_type_hierarchy(&mut self, tenant_id: &i32) -> Vec<AccountTypeHierarchy> {
        let all_accounts = self.account_type_dao.get_all_account_types_for_tenant_id(&tenant_id);
        AccountTypeService::create_hierarchy(&all_accounts)
    }

    fn create_hierarchy(all_accounts: &Vec<AccountTypeMaster>) -> Vec<AccountTypeHierarchy> {
        let account_map: HashMap<i16, &AccountTypeMaster> = all_accounts
            .iter()
            .map(|r| (r.id, r))
            .collect();
        let root_accounts: Vec<&AccountTypeMaster> = all_accounts.iter()
            .filter(|r| r.parent_id.is_none())
            .collect();
        root_accounts.iter()
            .map(|acc| self::AccountTypeService::create_account_type_hierarchy(&account_map, acc))
            .collect()
    }

    fn create_account_type_hierarchy(account_map: &HashMap<i16, &AccountTypeMaster>, root: &AccountTypeMaster) -> AccountTypeHierarchy {
        let create_hierarchy_object = |id: &i16| -> AccountTypeHierarchy {
            AccountTypeHierarchy {
                current_account_id: id.clone(),
                child_account_types: account_map.get(&id).unwrap()
                    .child_ids.as_ref().map(|c|
                    c.iter()
                        .map(|cc| AccountTypeHierarchy {
                            current_account_id: cc.clone(),
                            child_account_types: vec![],
                        }
                        ).collect::<Vec<AccountTypeHierarchy>>()).unwrap_or(vec![]),
            }
        };
        let mut root_node = create_hierarchy_object(&root.id);
        let mut queue: Vec<&mut AccountTypeHierarchy> = vec![&mut root_node];
        let mut max_iter = 1000;
        //build one level at a time. connect a parent with its immediate children and put children
        // in work queue
        while !queue.is_empty() && max_iter > 0 {
            max_iter -= 1;
            let current_node = queue.pop().unwrap();
            let children = &mut current_node.child_account_types;
            if children.is_empty() {
                continue;
            }
            children.iter_mut()
                .for_each(|mut ah| {
                    let master_item = account_map
                        .get(&ah.current_account_id)
                        .unwrap();
                    ah.child_account_types = master_item.child_ids.as_ref()
                        .map(|ids|
                            ids.iter()
                                .map(|id| create_hierarchy_object(id))
                                .collect::<Vec<AccountTypeHierarchy>>()
                        )
                        .unwrap_or(vec![]);
                    ah.child_account_types.iter_mut().for_each(|t| queue.push(t))
                });
        }
        root_node
    }
}

#[cfg(test)]
mod tests {
    use crate::accounting::account::account_type::account_seed_utils::read_account_type_seed_file;
    use crate::accounting::account::account_type::account_type_service::AccountTypeService;

    #[test]
    fn test_create_account_type_hierarchy() {
        let k = read_account_type_seed_file();
        let p = AccountTypeService::create_hierarchy(&k);
        let pp = serde_json::to_string(&p).unwrap();
        println!("{}", pp);
    }
}