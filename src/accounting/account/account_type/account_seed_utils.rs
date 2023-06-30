use std::collections::HashMap;

use csv::StringRecord;

use crate::accounting::account::account_type::account_type_models::AccountTypeMaster;
use crate::accounting::currency::currency_models::AuditMetadataBase;
use crate::seeddata::constants::SEED_FILES_LOCATION;

pub fn read_account_type_seed_file() -> Vec<AccountTypeMaster> {
    let pth = format!("{}account_type_master.csv", SEED_FILES_LOCATION);
    let mut fil = csv::Reader::from_path(pth).unwrap();
    let header = fil.headers().unwrap().clone();
    fil.records()
        .map(|k| map_csv_row_to_account_type(k.unwrap(), &header))
        .collect::<Vec<AccountTypeMaster>>()
}

fn map_csv_row_to_account_type(row: StringRecord, header: &StringRecord) -> AccountTypeMaster {
    let map = header
        .iter()
        .enumerate()
        .map(|e| (e.1, e.0))
        .collect::<HashMap<&str, usize>>();
    AccountTypeMaster {
        id: row.get(*map.get("id").unwrap()).unwrap().parse().unwrap(),
        tenant_id: row.get(*map.get("tenant_id").unwrap()).unwrap().parse().unwrap(),
        child_ids: map_csv_child_ids_to_rust_model(row.get(*map.get("child_ids").unwrap()).unwrap()),
        parent_id: row.get(*map.get("parent_id").unwrap()).map(|p| p.parse::<i16>().ok()).unwrap(),
        display_name: row.get(*map.get("display_name").unwrap()).unwrap_or("").to_string(),
        account_code: row.get(*map.get("account_code").unwrap()).map(|k| k.trim().parse::<i16>().ok()).unwrap(),
        audit_metadata: AuditMetadataBase {
            created_by: row.get(*map.get("created_by").unwrap()).unwrap_or("").to_string(),
            updated_by: row.get(*map.get("updated_by").unwrap()).unwrap_or("").to_string(),
            created_at: row.get(*map.get("created_at").unwrap()).unwrap().parse().unwrap(),
            updated_at: row.get(*map.get("updated_at").unwrap()).unwrap().parse().unwrap(),
        },
    }
}

fn map_csv_child_ids_to_rust_model(csv_field: &str) -> Option<Vec<i16>> {
    if csv_field.trim() == "" {
        return None;
    }
    let k = csv_field.trim()
        .strip_prefix('{')
        .unwrap()
        .strip_suffix('}')
        .unwrap()
        .split(',')
        .map(|o| o.parse::<i16>().unwrap())
        .collect::<Vec<i16>>();
    Some(k)
}


#[cfg(test)]
mod tests {
    use crate::accounting::account::account_type::account_seed_utils::read_account_type_seed_file;

    #[test]
    fn test() {
        read_account_type_seed_file();
    }
}