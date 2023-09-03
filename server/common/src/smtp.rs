// Copyright 2021 Ken Miura

use aws_config::{ecs::EcsCredentialsProvider, meta::region::RegionProviderChain};
use aws_sdk_sesv2::{
    config::{Builder, Credentials, Region},
    types::{Body, Content, Destination, EmailContent, Message},
    Client,
};
use axum::{async_trait, http::StatusCode, Json};
use once_cell::sync::Lazy;
use std::env::var;
use tracing::{error, info};

use crate::{err, ApiError, ErrResp};

pub const KEY_TO_ADMIN_EMAIL_ADDRESS: &str = "ADMIN_EMAIL_ADDRESS";
/// ユーザーが身分確認や職歴確認等を申請した際、管理者に通知を送るためのメールアドレス
/// ユーザーの目に触れる箇所で使わないように注意する
pub static ADMIN_EMAIL_ADDRESS: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_ADMIN_EMAIL_ADDRESS).unwrap_or_else(|_| {
        // UTのためにダミー用の初期値を設定しているだけ
        // 実行時には環境変数を指定して適切なものに入れ替える
        "admin@test.com".to_string()
    })
});

pub const KEY_TO_SYSTEM_EMAIL_ADDRESS: &str = "SYSTEM_EMAIL_ADDRESS";
/// ユーザーに通知を送る際の送信元に使われるメールアドレス
/// ユーザーの目に触れる箇所で使われるので、必ず自ドメインのメールアドレスを指定する。
/// また、送信専用とするため、メールボックスが存在しないアカウントを指定する。
pub static SYSTEM_EMAIL_ADDRESS: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_SYSTEM_EMAIL_ADDRESS).unwrap_or_else(|_| {
        // UTのためにダミー用の初期値を設定しているだけ
        // 実行時には環境変数を指定して適切なものに入れ替える
        "admin-no-reply@test.com".to_string()
    })
});

pub const KEY_TO_INQUIRY_EMAIL_ADDRESS: &str = "INQUIRY_EMAIL_ADDRESS";
/// ユーザーの問い合わせ窓口として使われるメールアドレス
/// ユーザーの目に触れる箇所で使われるので、必ず自ドメインのメールアドレスを指定する。
/// また、ユーザーからのメールを受け取るため、必ずメールボックスが存在するアカウントを指定する。
pub static INQUIRY_EMAIL_ADDRESS: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_INQUIRY_EMAIL_ADDRESS).unwrap_or_else(|_| {
        // UTのためにダミー用の初期値を設定しているだけ
        // 実行時には環境変数を指定して適切なものに入れ替える
        "inquiry@test.com".to_string()
    })
});

pub const KEY_TO_AWS_SES_REGION: &str = "AWS_SES_REGION";
pub static AWS_SES_REGION: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_AWS_SES_REGION).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" (example value: \"us-east-1\") must be set",
            KEY_TO_AWS_SES_REGION
        );
    })
});

pub const KEY_TO_AWS_SES_ACCESS_KEY_ID: &str = "AWS_SES_ACCESS_KEY_ID";
pub static AWS_SES_ACCESS_KEY_ID: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_AWS_SES_ACCESS_KEY_ID).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_AWS_SES_ACCESS_KEY_ID
        );
    })
});

pub const KEY_TO_AWS_SES_SECRET_ACCESS_KEY: &str = "AWS_SES_SECRET_ACCESS_KEY";
pub static AWS_SES_SECRET_ACCESS_KEY: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_AWS_SES_SECRET_ACCESS_KEY).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_AWS_SES_SECRET_ACCESS_KEY
        );
    })
});

pub const KEY_TO_AWS_SES_ENDPOINT_URI: &str = "AWS_SES_ENDPOINT_URI";
pub static AWS_SES_ENDPOINT_URI: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_AWS_SES_ENDPOINT_URI).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" (example value: \"http://smtp:8005\")  must be set",
            KEY_TO_AWS_SES_ENDPOINT_URI
        );
    })
});

#[async_trait]
pub trait SendMail {
    async fn send_mail(
        &self,
        to: &str,
        from: &str,
        subject: &str,
        text: &str,
    ) -> Result<(), ErrResp>;
}

#[derive(Clone)]
pub struct SmtpClient {
    client: Client,
}

impl SmtpClient {
    /// 引数を用いてAWS SES V2クライアントを生成する。
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
            "aws_ses_credential_provider",
        );

        let config = aws_config::from_env()
            .region(region_provider)
            .credentials_provider(credentials)
            .load()
            .await;

        let ses_config = Builder::from(&config).endpoint_url(endpoint_uri).build();

        Self {
            client: Client::from_conf(ses_config),
        }
    }

    /// 引数を用いてAWS SES V2クライアントを生成する。
    ///
    /// この関数で生成したインスタンスは、AWS SESへのアクセス権に関してECSタスクロールを参照する。
    /// 従って、この関数はAWS ECS上でECSタスクロールがアタッチされたコンテナ内で利用されることを前提としている。
    pub async fn new_with_ecs_task_role(region: &str, endpoint_uri: &str) -> Self {
        let cloned_region = region.to_string();
        let region_provider = RegionProviderChain::first_try(Region::new(cloned_region));
        let credentials = EcsCredentialsProvider::builder().build();

        let config = aws_config::from_env()
            .region(region_provider)
            .credentials_provider(credentials)
            .load()
            .await;

        let ses_config = Builder::from(&config).endpoint_url(endpoint_uri).build();

        Self {
            client: Client::from_conf(ses_config),
        }
    }
}

#[async_trait]
impl SendMail for SmtpClient {
    async fn send_mail(
        &self,
        to: &str,
        from: &str,
        subject: &str,
        text: &str,
    ) -> Result<(), ErrResp> {
        let dest = Destination::builder().to_addresses(to).build();

        let subject_content = Content::builder().data(subject).charset("UTF-8").build();
        let body_content = Content::builder().data(text).charset("UTF-8").build();
        let body = Body::builder().text(body_content).build();
        let msg = Message::builder()
            .subject(subject_content)
            .body(body)
            .build();
        let email_content = EmailContent::builder().simple(msg).build();

        let req = self
            .client
            .send_email()
            .from_email_address(from)
            .destination(dest)
            .content(email_content);

        let resp = req.send().await.map_err(|e| {
            error!(
                "failed to send email (to: {}, from: {}, subject: {}, body: {}): {}",
                to, from, subject, text, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: err::Code::UnexpectedErr as u32,
                }),
            )
        })?;

        info!("send email successfull (response: {:?})", resp);
        Ok(())
    }
}
