use std::sync::Arc;
use async_trait::async_trait;
use storage_service::AwsStorageService;
use storage_service::storage_service::Storage;

#[async_trait]
pub trait StorageService:Send+Sync {
    async fn upload_object(&self, bucket_name: &str, asset_name: &str, bytes: Vec<u8>) -> anyhow::Result<()>;
    async fn create_bucket(&self, bucket_name: &str) -> anyhow::Result<()>;
    async fn get_object(&self, bucket_name: &str, asset_name: &str) -> anyhow::Result<Vec<u8>>;
    async fn delete_object(&self, bucket_name: &str, asset_name: &str) -> anyhow::Result<()>;
}

struct StorageServiceImpl {
    client: AwsStorageService,
}

pub async fn get_storage_service() -> Arc<dyn StorageService> {
    let aws_store = AwsStorageService::new().await;
    Arc::new(StorageServiceImpl {
        client: aws_store
    })
}

#[async_trait]
impl StorageService for StorageServiceImpl {
    async fn upload_object(&self, bucket_name: &str, asset_name: &str, bytes: Vec<u8>) -> anyhow::Result<()> {
        self.client.upload_object(bucket_name, asset_name, bytes).await
    }

    async fn create_bucket(&self, bucket_name: &str) -> anyhow::Result<()> {
        self.client.create_bucket(bucket_name).await
    }

    async fn get_object(&self, bucket_name: &str, asset_name: &str) -> anyhow::Result<Vec<u8>> {
        self.client.get_object(bucket_name, asset_name).await
    }

    async fn delete_object(&self, bucket_name: &str, asset_name: &str) -> anyhow::Result<()> {
        self.client.delete_object(bucket_name, asset_name).await
    }
}