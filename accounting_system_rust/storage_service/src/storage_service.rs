use anyhow::Context;
use async_trait::async_trait;
use aws_config::{BehaviorVersion, SdkConfig};
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{BucketLocationConstraint, CreateBucketConfiguration};


async fn get_sdk_config() -> SdkConfig {
    aws_config::load_defaults(BehaviorVersion::latest())
        .await
}

async fn create_s3_client(config: &SdkConfig) -> Client {
    Client::new(config)
}


#[async_trait]
pub trait StorageService {
    async fn upload_object(&self, bucket_name: &str, asset_name: &str, bytes: Vec<u8>) -> anyhow::Result<()>;
    async fn create_bucket(&self, bucket_name: &str) -> anyhow::Result<()>;
    async fn get_object(&self, bucket_name: &str, asset_name: &str) -> anyhow::Result<Vec<u8>>;
    async fn delete_object(&self, bucket_name: &str, asset_name: &str) -> anyhow::Result<()>;
}

#[derive(Debug)]
pub struct AwsStorageService {
    client: Client,
}

impl AwsStorageService {
    pub async fn new() -> Self {
        let config = get_sdk_config().await;
        let client = create_s3_client(&config).await;
        AwsStorageService {
            client
        }
    }
}

#[async_trait]
impl StorageService for AwsStorageService {
    async fn upload_object(&self, bucket_name: &str, asset_name: &str, bytes: Vec<u8>) -> anyhow::Result<()> {
        let body = ByteStream::from(bytes);
        let _ = self.client.put_object()
            .bucket(bucket_name)
            .body(body)
            .key(asset_name)
            .send()
            .await.context("error during object upload")?;
        Ok(())
    }

    async fn create_bucket(&self, bucket_name: &str) -> anyhow::Result<()> {
        let config = CreateBucketConfiguration::builder()
            .set_location_constraint(Some(BucketLocationConstraint::ApSouth1))
            .build();
        let _ = self.client.create_bucket()
            .bucket(bucket_name)
            .create_bucket_configuration(config)
            .send()
            .await
            .context("error during bucket creation")?;
        Ok(())
    }

    async fn get_object(&self, bucket_name: &str, asset_name: &str) -> anyhow::Result<Vec<u8>> {
        let p = self.client.get_object()
            .bucket(bucket_name)
            .key(asset_name)
            .send()
            .await?;
        let asd = p.body.collect().await
            .context("error during collecting bytes from s3 object")?;
        Ok(asd.to_vec())
    }

    async fn delete_object(&self, bucket_name: &str, asset_name: &str) -> anyhow::Result<()> {
        let _ = self.client.delete_object()
            .bucket(bucket_name)
            .key(asset_name)
            .send()
            .await
            .context("error during object deletion from storage")?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use spectral::assert_that;
    use spectral::prelude::ResultAssertions;
    use crate::storage_service::{AwsStorageService,  StorageService};

    const UNIT_TESTS_BUCKET: &str = "unit-tests-objects-only";


    #[tokio::test]
    async fn test_upload_fetch_and_deletion_of_object() {
        let storage_service = AwsStorageService::new().await;
        let random_file_suffix: u32 = rand::random();
        let asset_name = format!("unit_test_file_{}.txt", random_file_suffix);
        let text = "unit test file content";
        let result = storage_service.upload_object(UNIT_TESTS_BUCKET, &asset_name,
                                                   text.as_bytes().to_vec()).await;
        assert_that!(result)
            .is_ok();
        result.unwrap();
        let output = storage_service.get_object(UNIT_TESTS_BUCKET, &asset_name)
            .await;
        assert_that!(output)
            .is_ok();
        let ou = output.unwrap();
        let s = String::from_utf8(ou).unwrap();
        assert_that!(s).is_equal_to(text.to_string());
        let deletion_result = storage_service.delete_object(UNIT_TESTS_BUCKET,
                                                            &asset_name).await;
        assert_that!(deletion_result).is_ok();
    }
}