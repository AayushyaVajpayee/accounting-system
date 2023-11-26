use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn main() {
    // process_currency_master_seed();
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
    let k = std::fs::read_to_string(p.as_path())?;
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

// fn process_