// Copyright 2021 Ken Miura

use axum::{http::StatusCode, Json};
use lettre;
use lettre::{ClientSecurity, Transport};
use lettre_email::EmailBuilder;
use std::net::SocketAddr;

use crate::{err_code, ApiError, ErrResp};

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
                        code: err_code::UNEXPECTED_ERR,
                    }),
                )
            })?;
        let sock = self.socket.parse::<SocketAddr>().map_err(|e| {
            tracing::error!("failed to parse socket str: str={}, e={}", self.socket, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: err_code::UNEXPECTED_ERR,
                }),
            )
        })?;
        let addr = SocketAddr::from(sock);
        let client = lettre::SmtpClient::new(addr, ClientSecurity::None).map_err(|e| {
            tracing::error!("failed to create lettre::SmtpClient: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: err_code::UNEXPECTED_ERR,
                }),
            )
        })?;
        let mut mailer = client.transport();
        let _ = mailer.send(email.into()).map_err(|e| {
            tracing::error!("failed to send email: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: err_code::UNEXPECTED_ERR,
                }),
            )
        })?;
        Ok(())
    }
}
