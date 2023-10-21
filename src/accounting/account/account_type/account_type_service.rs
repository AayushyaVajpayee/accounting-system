use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

use crate::accounting::account::account_type::account_type_dao::AccountTypeDao;
use crate::accounting::account::account_type::account_type_models::AccountTypeMaster;

#[async_trait]
pub trait AccountTypeService:Send+Sync {
    async fn get_account_type_hierarchy(&self, tenant_id: Uuid) -> Result<Vec<AccountTypeHierarchy>, HierarchyError>;
}

struct AccountTypeServiceImpl {
    account_type_dao: Arc<dyn AccountTypeDao>,

}

#[async_trait]
impl AccountTypeService for AccountTypeServiceImpl {
    async fn get_account_type_hierarchy(&self, tenant_id: Uuid) -> Result<Vec<AccountTypeHierarchy>, HierarchyError> {
        let all_accounts = self
            .account_type_dao.get_all_account_types_for_tenant_id(tenant_id).await;
        AccountTypeServiceImpl::create_hierarchy(&all_accounts)
    }
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum HierarchyError {
    #[error("account id {0} is not present in account master")]
    AccountIdNotPresentInChart(i16),
    #[error("no accounts found for creating hierarchy")]
    EmptyChartOfAccounts,
}

#[derive(Debug, Serialize)]
pub struct AccountTypeHierarchy {
    current_account_id: i16,
    child_account_types: Vec<AccountTypeHierarchy>,

}

impl AccountTypeServiceImpl {

    fn create_hierarchy(all_accounts: &[AccountTypeMaster]) -> Result<Vec<AccountTypeHierarchy>, HierarchyError> {
        let account_map: HashMap<i16, &AccountTypeMaster> = all_accounts
            .iter()
            .map(|r| (r.id, r))
            .collect();
        let root_accounts: Vec<&AccountTypeMaster> = all_accounts.iter()
            .filter(|r| r.parent_id.is_none())
            .collect();
        let root_accounts = if root_accounts.is_empty() {
            vec![all_accounts.iter().min_by_key(|l| l.id).ok_or(HierarchyError::EmptyChartOfAccounts)?]
        } else {
            root_accounts
        };
        root_accounts.iter()
            .map(|acc| AccountTypeServiceImpl::create_account_type_hierarchy(&account_map, acc))
            .collect()
    }

    fn create_account_type_hierarchy(account_map: &HashMap<i16, &AccountTypeMaster>, root: &AccountTypeMaster) -> Result<AccountTypeHierarchy, HierarchyError> {
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
        let mut max_iter = 30;
        //build one level at a time. connect a parent with its immediate children and put children
        // in work queue
        while !queue.is_empty() && max_iter > 0 {
            max_iter -= 1;
            let current_node = queue.pop().unwrap();
            let children = &mut current_node.child_account_types;
            if children.is_empty() {
                continue;
            }
            for ah in children.iter_mut() {
                let master_item = account_map
                    .get(&ah.current_account_id)
                    .ok_or_else(|| HierarchyError::AccountIdNotPresentInChart(ah.current_account_id))?;
                ah.child_account_types = master_item.child_ids.as_ref()
                    .map(|ids|
                        ids.iter()
                            .map(|id| create_hierarchy_object(id))
                            .collect::<Vec<AccountTypeHierarchy>>()
                    )
                    .unwrap_or(vec![]);
                ah.child_account_types.iter_mut().for_each(|t| queue.push(t))
            }
        }
        Ok(root_node)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::OnceLock;

    use regex::Regex;
    use rstest::rstest;

    use crate::accounting::account::account_type::account_type_models::AccountTypeMaster;
    use crate::accounting::account::account_type::account_type_service::{AccountTypeHierarchy, AccountTypeServiceImpl};
    use crate::accounting::currency::currency_models::AuditMetadataBase;
    use crate::accounting::user::user_models::SEED_USER_ID;
    use crate::tenant::tenant_models::SEED_TENANT_ID;

    const ADJACENCY_LIST_STR: &str = r"(\d+)(\[)(((\d*)|(\d+,))*)(])";
    static ADJACENCY_LIST_REGEX: OnceLock<Regex> = OnceLock::new();

    #[derive(Debug)]
    struct AdjacencyListEntry {
        id: i16,
        adj_links: HashSet<i16>,
    }

    fn serialise_account_hierarchy(hierarchy: &mut AccountTypeHierarchy) -> String {
        let mut adj: Vec<AdjacencyListEntry> = vec![];
        let mut work_queue: Vec<&AccountTypeHierarchy> = vec![hierarchy];
        let mut max_iter = 30;
        while !work_queue.is_empty() && max_iter > 0 {
            max_iter -= 1;
            let curr = work_queue.pop().unwrap();
            let mut adj_entry = adj.iter_mut().find(|l| l.id == curr.current_account_id);
            if adj_entry.is_some() {
                curr.child_account_types.iter().for_each(|a| {
                    adj_entry.as_deref_mut().unwrap().adj_links.insert(a.current_account_id);
                })
            } else {
                adj.push(AdjacencyListEntry {
                    id: curr.current_account_id,
                    adj_links: curr.child_account_types.iter().map(|l| l.current_account_id).collect::<HashSet<i16>>(),
                });
                curr.child_account_types.iter().for_each(|a| work_queue.push(a));
            }
        }
        adj.sort_by_key(|s| s.id);
        adj.iter()
            .map(|o| {
                let mut new_str = String::new();
                new_str
                    .push_str(o.id.to_string().as_str());
                new_str.push('[');
                let mut ooo = o.adj_links
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<String>>();
                ooo.sort_by_key(|a| a.parse::<i16>().unwrap());

                new_str.push_str(ooo.join(",").as_str());
                new_str.push(']');
                new_str
            }
            ).collect::<Vec<String>>().join(",")
    }

    fn parse_account_tree(acc_tree: &str) -> Vec<AccountTypeMaster> {
        if acc_tree.is_empty() {
            return vec![];
        }
        let k = ADJACENCY_LIST_REGEX.get_or_init(|| Regex::new(ADJACENCY_LIST_STR).unwrap());
        let pp = k
            .find_iter(acc_tree)
            .inspect(|a| println!("{}", a.as_str()))
            .map(|k| {
                let p = acc_tree[k.range()].to_string();
                let id = p.split_once('[')
                    .map(|l| l.0.parse::<i16>().unwrap())
                    .unwrap();
                let child_ids = p
                    .split_once('[')
                    .unwrap().1
                    .strip_suffix(']')
                    .filter(|k| !k.is_empty())
                    .map(|p| p.split(',')
                        .map(|l| l.trim().parse::<i16>().unwrap())
                        .collect::<Vec<i16>>())
                    .unwrap_or(vec![]);
                AdjacencyListEntry {
                    id,
                    adj_links: child_ids.into_iter().collect::<HashSet<i16>>(),
                }
            }
            ).inspect(|a| println!("{:?}", a))
            .collect::<Vec<AdjacencyListEntry>>();

        pp.iter()
            .map(|o|
                create_account_type_master(o.id,
                                           &o.adj_links,
                                           find_parent_for_id(&pp, o.id)))
            .inspect(|a| println!("{:?}", a))
            .collect::<Vec<AccountTypeMaster>>()
    }

    fn find_parent_for_id(adj_list: &Vec<AdjacencyListEntry>, id: i16) -> Option<i16> {
        adj_list.iter().filter(|k| k.id != id)
            .find(|k| k.adj_links.contains(&id))
            .map(|k| k.id)
    }

    fn create_account_type_master(id: i16, child_ids: &HashSet<i16>, parent_id: Option<i16>) -> AccountTypeMaster {
        AccountTypeMaster {
            id,
            tenant_id: *SEED_TENANT_ID,
            child_ids: Some(child_ids.iter().copied().collect::<Vec<i16>>()),
            parent_id,
            display_name: "".to_string(),
            account_code: None,
            audit_metadata: AuditMetadataBase {
                created_by: *SEED_USER_ID,
                updated_by: *SEED_USER_ID,
                created_at: 0,
                updated_at: 0,
            },
        }
    }

    #[rstest]
    #[case::all_independent("1[],2[],3[],4[],5[]", "[1,5]")]
    #[case::one_root("1[2,3],2[4],3[5],4[],5[]", "[1,5]")]
    #[case::two_roots("1[2,3],2[],3[],4[5],5[]", "[1,5]")]
    #[case::cyclic_graph_should_fail("1[2],2[1]", "[1,2]")]
    #[case::cyclic_graph_with_more_than_two_nodes_should_fail("1[2],2[3],3[4],4[1]", "[1,4]")]
    #[should_panic]
    #[case::incomplete_adjacency_list_info_should_fail("1[2]", "[1,2]")]
    #[should_panic]
    #[case::empty_list_should_pass("", "[]")]
    fn test_create_account_type_hierarchy(#[case] account_tree: String, #[case] acc_id_range: String) {
        // let k = read_account_type_seed_file();
        let range = acc_id_range
            .strip_suffix(']').unwrap_or("")
            .strip_prefix('[').unwrap_or("")
            .split(',')
            .map(|a| a.trim())
            .filter(|a| !a.is_empty())
            .map(|a| a.parse::<i16>().unwrap())
            .collect::<Vec<i16>>();
        println!("{:?}", range);
        let total_accounts_count = if range.len() == 2 {
            Some(range.get(1).unwrap() - range.first().unwrap() + 1)
        } else { None };
        let accounts = parse_account_tree(&account_tree);
        if total_accounts_count.is_some() {
            accounts.iter().for_each(|a| {
                assert!(a.id >= range[0]);
                assert!(a.id <= range[1]);
            })
        }
        println!("accs1: {}", serde_json::to_string(&accounts).unwrap());
        let mut p = AccountTypeServiceImpl::create_hierarchy(&accounts).unwrap();
        let pp = serde_json::to_string(&p).unwrap();
        println!("hierarchy: {}", pp);
        let k = p.iter_mut()
            .map(serialise_account_hierarchy)
            .inspect(|a| println!("{}", a))
            .collect::<Vec<String>>()
            .join(",");
        assert_eq!(account_tree, k);
        println!("{}", k);
    }
}