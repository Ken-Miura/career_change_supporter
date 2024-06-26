// Copyright 2021 Ken Miura

use std::{env::var, error::Error};

use aws_config::{ecs::EcsCredentialsProvider, meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_s3::{
    config::{Credentials, Region},
    primitives::ByteStream,
    Client,
};
use once_cell::sync::Lazy;

pub const KEY_TO_AWS_S3_REGION: &str = "AWS_S3_REGION";
pub static AWS_S3_REGION: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_AWS_S3_REGION).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" (example value: \"ap-northeast-1\") must be set",
            KEY_TO_AWS_S3_REGION
        );
    })
});

pub const KEY_TO_AWS_S3_ACCESS_KEY_ID: &str = "AWS_S3_ACCESS_KEY_ID";
pub static AWS_S3_ACCESS_KEY_ID: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_AWS_S3_ACCESS_KEY_ID).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_AWS_S3_ACCESS_KEY_ID
        );
    })
});

pub const KEY_TO_AWS_S3_SECRET_ACCESS_KEY: &str = "AWS_S3_SECRET_ACCESS_KEY";
pub static AWS_S3_SECRET_ACCESS_KEY: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_AWS_S3_SECRET_ACCESS_KEY).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_AWS_S3_SECRET_ACCESS_KEY
        );
    })
});

pub const KEY_TO_AWS_S3_ENDPOINT_URI: &str = "AWS_S3_ENDPOINT_URI";
pub static AWS_S3_ENDPOINT_URI: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_AWS_S3_ENDPOINT_URI).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" (example value: \"http://storage:9000\") must be set",
            KEY_TO_AWS_S3_ENDPOINT_URI
        );
    })
});

pub const KEY_TO_IDENTITY_IMAGES_BUCKET_NAME: &str = "IDENTITY_IMAGES_BUCKET_NAME";
pub static IDENTITY_IMAGES_BUCKET_NAME: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_IDENTITY_IMAGES_BUCKET_NAME).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" (example value: \"s3-bucket-name-to-identity-images\") must be set",
            KEY_TO_IDENTITY_IMAGES_BUCKET_NAME
        );
    })
});

pub const KEY_TO_CAREER_IMAGES_BUCKET_NAME: &str = "CAREER_IMAGES_BUCKET_NAME";
pub static CAREER_IMAGES_BUCKET_NAME: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_CAREER_IMAGES_BUCKET_NAME).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" (example value: \"s3-bucket-name-to-career-images\") must be set",
            KEY_TO_CAREER_IMAGES_BUCKET_NAME
        );
    })
});

#[derive(Clone)]
pub struct StorageClient {
    client: Client,
}

impl StorageClient {
    /// 引数を用いてAWS S3クライアントを生成する。
    ///
    /// 引数以外の値は環境変数が使われる。環境変数と引数では引数のキーが優先される。
    pub async fn new(
        region: &str,
        access_key_id: &str,
        secret_access_key: &str,
        endpoint_uri: &str,
    ) -> Self {
        let cloned_region = region.to_string();
        let region_provider = RegionProviderChain::first_try(Region::new(cloned_region));
        let credentials = Credentials::new(
            access_key_id,
            secret_access_key,
            None,
            None,
            "aws_s3_credential_provider",
        );

        let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
            .region(region_provider)
            .credentials_provider(credentials)
            .load()
            .await;

        let s3_conf = aws_sdk_s3::config::Builder::from(&config)
            .endpoint_url(endpoint_uri)
            .build();

        Self {
            client: Client::from_conf(s3_conf),
        }
    }

    /// 引数を用いてAWS S3クライアントを生成する。
    ///
    /// この関数で生成したインスタンスは、AWS S3へのアクセス権に関してECSタスクロールを参照する。
    /// 従って、この関数はAWS ECS上でECSタスクロールがアタッチされたコンテナ内で利用されることを前提としている。
    pub async fn new_with_ecs_task_role(region: &str, endpoint_uri: &str) -> Self {
        let cloned_region = region.to_string();
        let region_provider = RegionProviderChain::first_try(Region::new(cloned_region));
        let credentials = EcsCredentialsProvider::builder().build();

        let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
            .region(region_provider)
            .credentials_provider(credentials)
            .load()
            .await;

        let s3_conf = aws_sdk_s3::config::Builder::from(&config)
            .endpoint_url(endpoint_uri)
            .build();

        Self {
            client: Client::from_conf(s3_conf),
        }
    }

    // PutObject操作で発生する可能性のあるエラーで、呼び出し側でハンドリングする必要のあるエラー（リカバリ可能なエラー）は現時点ではない。
    // そのため、Box<dyn Error>にエラーを丸めてログ出力して、問題が発生したときに解析できるだけにしておく。
    // https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/types/enum.SdkError.html
    pub async fn upload_object(
        &self,
        bucket_name: &str,
        key: &str,
        object: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let stream = ByteStream::from(object);
        let resp = self
            .client
            .put_object()
            .bucket(bucket_name)
            .key(key)
            .body(stream)
            .send()
            .await
            .map_err(Box::new)?;
        tracing::debug!("PutObjectOutput: {:?}", resp);
        Ok(())
    }

    // GetObject操作で発生する可能性のあるエラーで、呼び出し側でハンドリングする必要のあるエラー（リカバリ可能なエラー）は現時点ではない。
    // そのため、Box<dyn Error>にエラーを丸めてログ出力して、問題が発生したときに解析できるだけにしておく。
    // https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/types/enum.SdkError.html
    pub async fn download_object(
        &self,
        bucket_name: &str,
        key: &str,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let resp = self
            .client
            .get_object()
            .bucket(bucket_name)
            .key(key)
            .send()
            .await
            .map_err(Box::new)?;
        let aggregated_bytes = resp.body.collect().await.map_err(Box::new)?;
        let object = aggregated_bytes.into_bytes().to_vec();
        Ok(object)
    }

    // DeleteObject操作で発生する可能性のあるエラーで、呼び出し側でハンドリングする必要のあるエラー（リカバリ可能なエラー）は現時点ではない。
    // そのため、Box<dyn Error>にエラーを丸めてログ出力して、問題が発生したときに解析できるだけにしておく。
    // https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/types/enum.SdkError.html
    pub async fn delete_object(&self, bucket_name: &str, key: &str) -> Result<(), Box<dyn Error>> {
        let resp = self
            .client
            .delete_object()
            .bucket(bucket_name)
            .key(key)
            .send()
            .await
            .map_err(Box::new)?;
        tracing::debug!("DeleteObjectOutput: {:?}", resp);
        Ok(())
    }
}
