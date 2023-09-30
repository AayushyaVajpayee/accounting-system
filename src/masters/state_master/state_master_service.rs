use async_trait::async_trait;

#[async_trait]
trait StateMasterService{
    async fn get_all_states();

    async fn get_state_by_id();

    async fn get_state_by_gst_code();

}