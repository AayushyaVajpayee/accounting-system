use uuid::Uuid;
use crate::accounting::currency::currency_models::AuditMetadataBase;

pub struct CompanyMaster{
    id:Uuid,
    name:String,//50  chars maximum
    gstin:String,
    audit_metadata:AuditMetadataBase

}


#[cfg(test)]
mod tests{

    #[test]
    fn test_uuid(){
        // let k =
    }
}

// b2b companies will raise financial docs between itself and other company
// b2c companies will raise financial docs between itself and other customer
// shipping party will be other


