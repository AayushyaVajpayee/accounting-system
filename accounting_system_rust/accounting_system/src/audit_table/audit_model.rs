use serde_json::Value;
use uuid::{ Uuid};

#[derive(Debug)]
pub struct AuditEntry {
    pub id: Uuid,
    pub tenant_id: Uuid,
    //v7
    pub audit_record_id: Uuid,
    pub operation_type: i8,
    //0 for update 1 for delete
    //only update and delete to be applicable
    pub old_record: Value,
    //jsonb row now
    pub table_id: u32,
    //jsonb row then
    pub created_at: i64,
}
/*
#[test]
fn test() {
    let p = Uuid::from_str("018afacc-2f89-7bbb-8460-53da82a601de").unwrap();
    let ti = p.get_timestamp().unwrap();
    let kk: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let one = Builder::from_unix_timestamp_millis(1696433676000, &kk)
        .into_uuid();
    let two = Builder::from_unix_timestamp_millis(1696433676000, &kk)
        .into_uuid();
    println!("{}", one);
    println!("{}", two);
}*/