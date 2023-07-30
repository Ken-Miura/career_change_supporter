// Copyright 2021 Ken Miura

use std::{env::var, error::Error};

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{
    config::{Credentials, Region},
    primitives::ByteStream,
    Client,
};
use once_cell::sync::Lazy;

pub const KEY_TO_AWS_S3_ENDPOINT_URI: &str = "AWS_S3_ENDPOINT_URI";
pub static AWS_S3_ENDPOINT_URI: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_AWS_S3_ENDPOINT_URI).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" (example value: \"http://storage:9000\") must be set",
            KEY_TO_AWS_S3_ENDPOINT_URI
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

pub const KEY_TO_AWS_S3_REGION: &str = "AWS_S3_REGION";
pub static AWS_S3_REGION: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_AWS_S3_REGION).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" (example value: \"ap-northeast-1\") must be set",
            KEY_TO_AWS_S3_REGION
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

// PutObject操作で発生する可能性のあるエラーで、呼び出し側でハンドリングする必要のあるエラー（リカバリ可能なエラー）は現時点ではない。
// そのため、Box<dyn Error>にエラーを丸めてログ出力して、問題が発生したときに解析できるだけにしておく。
// https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/types/enum.SdkError.html
pub async fn upload_object(
    bucket_name: &str,
    key: &str,
    object: Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    let endpoint = AWS_S3_ENDPOINT_URI.to_string();
    let client = create_client(&endpoint).await?;
    let stream = ByteStream::from(object);
    let resp = client
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
pub async fn download_object(bucket_name: &str, key: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let endpoint = AWS_S3_ENDPOINT_URI.to_string();
    let client = create_client(&endpoint).await?;

    let resp = client
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
pub async fn delete_object(bucket_name: &str, key: &str) -> Result<(), Box<dyn Error>> {
    let endpoint = AWS_S3_ENDPOINT_URI.to_string();
    let client = create_client(&endpoint).await?;

    let resp = client
        .delete_object()
        .bucket(bucket_name)
        .key(key)
        .send()
        .await
        .map_err(Box::new)?;
    tracing::debug!("DeleteObjectOutput: {:?}", resp);
    Ok(())
}

async fn create_client(endpoint_uri: &str) -> Result<Client, Box<dyn Error>> {
    let region_provider = RegionProviderChain::first_try(Region::new(AWS_S3_REGION.as_str()));
    let credentials = Credentials::new(
        AWS_S3_ACCESS_KEY_ID.as_str(),
        AWS_S3_SECRET_ACCESS_KEY.as_str(),
        None,
        None,
        "aws_ses_credential_provider",
    );

    let config = aws_config::from_env()
        .region(region_provider)
        .credentials_provider(credentials)
        .load()
        .await;

    let s3_conf = aws_sdk_s3::config::Builder::from(&config)
        .endpoint_url(endpoint_uri)
        .build();

    Ok(Client::from_conf(s3_conf))
}
