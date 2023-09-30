use crate::accounting::currency::currency_models::AuditMetadataBase;

struct StateName(String);

impl StateName {
    fn new(name: &str) -> Result<Self, &str> {
        let name = name.trim();
        if name.len() > 60 {
            return Err("state name cannot be greater than 60 chars");
        }
        let name = name.to_ascii_uppercase();
        Ok(Self(name))
    }
}

struct StateMasterModel {
    id: i32,
    state_code: i16,
    state_name: StateName,
    audit_metadata:AuditMetadataBase
}
