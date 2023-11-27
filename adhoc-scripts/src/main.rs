use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, format};
use std::time::{SystemTime, UNIX_EPOCH};
use itertools::Itertools;

use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

fn main() {
    // process_account_type_master_seed();
    // process_currency_master_seed();
    process_ledger_master_seed();
    println!("Hello, world!");
}


#[derive(Debug, Serialize, Deserialize)]
struct CurrencyMaster {
    id: String,
    tenant_id: String,
    scale: String,
    display_name: String,
    description: String,
    created_by: String,
    updated_by: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LedgerMaster {
    id: String,
    tenant_id: String,
    display_name: String,
    currency_master_id: String,
    created_by: String,
    updated_by: String,
    created_at: String,
    updated_at: String,
}

fn process_currency_master_seed() -> Result<(), Box<dyn Error>> {
    let p1 = std::env::current_dir()?;
    let p = p1.join("schema/postgres/seed_data/currency_master.csv");
    let mut reader = csv::Reader::from_path(p.as_path())?;
    let mut writer = csv::Writer::from_path(p1.join("schema/postgres/seed_data/currency_master_temp.csv"))?;
    // writer.write_record(reader.headers()?)?;
    // writer.flush()?;
    let mut map: HashMap<String, String> = HashMap::new();
    for rec in reader.records() {
        let string_record = rec?;
        let mut currency: CurrencyMaster = string_record.deserialize(None)?;
        let uuid = Uuid::now_v7();
        map.insert(currency.id, uuid.to_string());
        currency.id = uuid.to_string();
        writer.serialize(currency)?;
    }
    let mut ledger_master_reader = csv::Reader::
    from_path(p1.join("schema/postgres/seed_data/ledger_master.csv"))?;
    let mut ledger_master_writer = csv::Writer::
    from_path(p1.join("schema/postgres/seed_data/ledger_master_temp.csv"))?;
    // ledger_master_writer.write_record(ledger_master_reader.headers()?)?;
    for rec in ledger_master_reader.records() {
        let string_record = rec?;
        let mut ledger_master: LedgerMaster = string_record.deserialize(None)?;
        let uuid = map.get(ledger_master.currency_master_id.as_str()).unwrap();
        ledger_master.currency_master_id = uuid.clone();
        ledger_master_writer.serialize(ledger_master)?;
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountTypeMaster {
    id: String,
    tenant_id: String,
    child_ids: String,
    parent_id: String,
    display_name: String,
    account_code: String,
    created_by: String,
    updated_by: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Account {
    id: String,
    tenant_id: String,
    display_code: String,
    account_type_id: String,
    user_id: String,
    ledger_master_id: String,
    debits_posted: String,
    debits_pending: String,
    credits_posted: String,
    credits_pending: String,
    created_by: String,
    updated_by: String,
    created_at: String,
    updated_at: String,
}

fn process_account_type_master_seed() -> Result<(), Box<dyn Error>> {
    let p1 = std::env::current_dir()?;
    let p = p1.join("schema/postgres/seed_data/account_type_master.csv");
    let mut account_type_master_reader = csv::Reader::from_path(p.as_path())?;
    let mut account_type_master_writer = csv::Writer::
    from_path(p1.join("schema/postgres/seed_data/account_type_master_temp.csv"))?;
    let mut map: HashMap<String, String> = HashMap::new();
    for rec in account_type_master_reader.records() {
        let string_record = rec?;
        let mut account_type_master: AccountTypeMaster = string_record.deserialize(None)?;
        let id = account_type_master.id.parse::<i32>()?;
        let timestmp = Timestamp::
        from_unix(NoContext,
                  (SystemTime::now()
                      .duration_since(UNIX_EPOCH)
                      .unwrap()
                      .as_micros() as u64) + (id as u64) * 1000, 0);//to generate sortable uuids

        let uuid = Uuid::new_v7(timestmp);
        map.insert(id.to_string(), uuid.to_string());
    }
    println!("looping ");
    let mut account_type_master_reader = csv::Reader::from_path(p.as_path())?;
    for rec in account_type_master_reader.records() {
        let string_record = rec?;
        let mut account_type_master: AccountTypeMaster = string_record.deserialize(None)?;
        account_type_master.id = map.get(account_type_master.id.as_str()).unwrap().clone();
        let k = parse_child_ids_array_in_seed(account_type_master.child_ids.as_str())?
            .iter().map(|a| map.get(a).unwrap().to_string()).join(",");
        if !k.is_empty() {
            account_type_master.child_ids = format!("{{{}}} ", k);
        }
        if !account_type_master.parent_id.is_empty() {
            account_type_master.parent_id = map.get(account_type_master.parent_id.as_str()).unwrap().to_string();
        }
        account_type_master_writer.serialize(account_type_master)?;

        let mut account_reader = csv::Reader::from_path(p1.join("schema/postgres/seed_data/user_account.csv"))?;
        let mut account_writer = csv::Writer::from_path(p1.join("schema/postgres/seed_data/user_account_temp.csv"))?;

        for rec in account_reader.records() {
            let string_record = rec?;
            let mut account: Account = string_record.deserialize(None)?;
            account.account_type_id = map.get(account.account_type_id.as_str()).unwrap().to_string();
            account_writer.serialize(account)?;
        }
        // println!("dafda {:?}",account_type_master.child_ids);
    }
    Ok(())
}

fn parse_child_ids_array_in_seed(array: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut array = array.trim();
    if array.is_empty() {
        return Ok(vec![]);
    }
    array = array.strip_prefix("{").unwrap()
        .strip_suffix("}").unwrap();
    let parsed_ar = array.split(',')
        .map(|a| a.trim().to_string())
        .collect::<Vec<String>>();
    Ok(parsed_ar)
}

fn process_ledger_master_seed() -> Result<(), Box<dyn Error>> {
    let curr_path = std::env::current_dir()?;
    let dest_path = curr_path.join("schema/postgres/seed_data/ledger_master.csv");
    let mut led_mst_reader = csv::Reader::from_path(dest_path)?;
    let mut led_mst_writer = csv::Writer::from_path(curr_path.join("schema/postgres/seed_data/ledger_master_temp.csv"))?;
    let mut map: HashMap<String, String> = HashMap::new();
    for rec in led_mst_reader.records() {
        let string_record = rec?;
        let mut led_mst: LedgerMaster = string_record.deserialize(None)?;
        let id = led_mst.id.parse::<i32>()?;
        let timestmp = Timestamp::
        from_unix(NoContext,
                  (SystemTime::now()
                      .duration_since(UNIX_EPOCH)
                      .unwrap()
                      .as_micros() as u64) + (id as u64) * 1000, 0);//to generate sortable uuids

        let uuid = Uuid::new_v7(timestmp);
        map.insert(led_mst.id, uuid.to_string());
        led_mst.id = uuid.to_string();
        led_mst_writer.serialize(led_mst)?;
    }
    let mut account_reader = csv::Reader::from_path(curr_path.join("schema/postgres/seed_data/user_account.csv"))?;
    let mut account_writer = csv::Writer::from_path(curr_path.join("schema/postgres/seed_data/user_account_temp.csv"))?;
    for rec in account_reader.records() {
        let string_record = rec?;
        let mut account: Account = string_record.deserialize(None)?;
        account.ledger_master_id = map.get(account.ledger_master_id.as_str()).unwrap().to_string();
        account_writer.serialize(account)?;
    }
    Ok(())
}