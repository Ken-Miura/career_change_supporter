// Copyright 2021 Ken Miura

use axum::{async_trait, http::StatusCode, Json};
use lettre::{
    message::Mailbox, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use once_cell::sync::Lazy;
use std::env::var;
use tracing::{error, info};

use crate::{err, ApiError, ErrResp};

// TODO: 実際にメールアドレスを取得した後、修正する
pub const ADMIN_EMAIL_ADDRESS: &str = "admin@test.com";
// TODO: 実際にメールアドレスを取得した後、修正する
pub const SYSTEM_EMAIL_ADDRESS: &str = "admin-no-reply@test.com";
// TODO: 実際にメールアドレスを取得した後、修正する
pub const INQUIRY_EMAIL_ADDRESS: &str = "inquiry@test.com";

pub const KEY_TO_SMTP_HOST: &str = "SMTP_HOST";
pub static SMTP_HOST: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_SMTP_HOST).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" (example value: \"127.0.0.1\") must be set",
            KEY_TO_SMTP_HOST
        );
    })
});

pub const KEY_TO_SMTP_PORT: &str = "SMTP_PORT";
pub static SMTP_PORT: Lazy<u16> = Lazy::new(|| {
    let port_str = var(KEY_TO_SMTP_PORT).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" (example value: \"1025\") must be set",
            KEY_TO_SMTP_PORT
        );
    });
    port_str.parse::<u16>().unwrap_or_else(|op| {
        panic!("failed to parse SMTP_PORT ({}): {}", port_str, op);
    })
});

pub const KEY_TO_SMTP_USERNAME: &str = "SMTP_USERNAME";
pub static SMTP_USERNAME: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_SMTP_USERNAME).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" (example value: \"username\") must be set",
            KEY_TO_SMTP_USERNAME
        );
    })
});

pub const KEY_TO_SMTP_PASSWORD: &str = "SMTP_PASSWORD";
pub static SMTP_PASSWORD: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_SMTP_PASSWORD).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" (example value: \"password\") must be set",
            KEY_TO_SMTP_PASSWORD
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

pub struct SmtpClient {
    host: String,
    port: u16,
    username: String,
    password: String,
}

impl SmtpClient {
    pub fn new(host: String, port: u16, username: String, password: String) -> Self {
        Self {
            host,
            port,
            username,
            password,
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
        let to_email_addr = SmtpClient::parse_email_address(to)?;
        let from_email_addr = SmtpClient::parse_email_address(from)?;
        let email = Message::builder()
            .to(to_email_addr)
            .from(from_email_addr)
            .subject(subject)
            .body(String::from(text))
            .map_err(|e| {
                error!("failed to build email: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError {
                        code: err::Code::UnexpectedErr as u32,
                    }),
                )
            })?;

        let credentials = Credentials::new(self.username.clone(), self.password.clone());
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(self.host.clone())
            .port(self.port)
            .credentials(credentials)
            .build();

        let resp = mailer.send(email.clone()).await.map_err(|e| {
            error!("failed to send email ({:?}): {}", email, e);
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

impl SmtpClient {
    fn parse_email_address(email_address: &str) -> Result<Mailbox, ErrResp> {
        email_address.parse().map_err(|e| {
            error!("failed to parse email_address ({}): {}", e, email_address);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: err::Code::UnexpectedErr as u32,
                }),
            )
        })
    }
}
