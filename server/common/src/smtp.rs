// Copyright 2021 Ken Miura

use axum::{http::StatusCode, Json};
use lettre;
use lettre::{ClientSecurity, Transport};
use lettre_email::EmailBuilder;
use once_cell::sync::Lazy;
use std::env::var;
use std::net::SocketAddr;

use crate::{err, ApiError, ErrResp};

// TODO: 実際にメールアドレスを取得した後、修正する
pub const SYSTEM_EMAIL_ADDRESS: &str = "admin@test.com";
// TODO: 実際にメールアドレスを取得した後、修正する
pub const INQUIRY_EMAIL_ADDRESS: &str = "inquiry@test.com";

pub const KEY_TO_SOCKET_FOR_SMTP_SERVER: &str = "SOCKET_FOR_SMTP_SERVER";

pub static SOCKET_FOR_SMTP_SERVER: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_SOCKET_FOR_SMTP_SERVER).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" (example value: \"127.0.0.1:1080\") must be set",
            KEY_TO_SOCKET_FOR_SMTP_SERVER
        );
    })
});

pub trait SendMail {
    fn send_mail(&self, to: &str, from: &str, subject: &str, text: &str) -> Result<(), ErrResp>;
}

pub struct SmtpClient {
    socket: String,
}

impl SmtpClient {
    pub fn new(socket: String) -> Self {
        Self { socket }
    }
}

impl SendMail for SmtpClient {
    fn send_mail(&self, to: &str, from: &str, subject: &str, text: &str) -> Result<(), ErrResp> {
        let email = EmailBuilder::new()
            .to(to)
            .from(from)
            .subject(subject)
            .text(text)
            .build()
            .map_err(|e| {
                tracing::error!("failed to build email: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError {
                        code: err::Code::UnexpectedErr as u32,
                    }),
                )
            })?;
        let addr = self.socket.parse::<SocketAddr>().map_err(|e| {
            tracing::error!("failed to parse socket str: str={}, e={}", self.socket, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: err::Code::UnexpectedErr as u32,
                }),
            )
        })?;
        let client = lettre::SmtpClient::new(addr, ClientSecurity::None).map_err(|e| {
            tracing::error!("failed to create lettre::SmtpClient: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: err::Code::UnexpectedErr as u32,
                }),
            )
        })?;
        let mut mailer = client.transport();
        let _ = mailer.send(email.into()).map_err(|e| {
            tracing::error!("failed to send email: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: err::Code::UnexpectedErr as u32,
                }),
            )
        })?;
        Ok(())
    }
}
