use async_trait::async_trait;

#[async_trait]
trait CityMasterService{
    async fn get_city_by_id();
    async fn get_all_cities();
    async fn get_city_by_name();

}