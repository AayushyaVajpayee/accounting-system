use async_trait::async_trait;

#[async_trait]
trait PincodeMasterService{
    async fn get_all_pincodes();
    async fn get_pincode_by_id();
}