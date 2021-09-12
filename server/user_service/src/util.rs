// Copyright 2021 Ken Miura

use axum::{http::StatusCode, Json};
use common::schema::ccs_schema::user_account::dsl::{email_address, user_account};
use common::{ApiError, ErrResp};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use diesel::{
    dsl::count_star,
    query_dsl::methods::{FilterDsl, SelectDsl},
    ExpressionMethods, RunQueryDsl,
};

use crate::err_code;

pub(crate) const ROOT_PATH: &str = "/api";

pub(crate) fn unexpected_err_resp() -> ErrResp {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            code: err_code::UNEXPECTED_ERR,
        }),
    )
}

/// ユーザーが既に存在するか確認する
pub(crate) fn user_exists(
    conn: &PooledConnection<ConnectionManager<PgConnection>>,
    email_addr: &str,
) -> Result<bool, ErrResp> {
    let cnt = user_account
        .filter(email_address.eq(email_addr))
        .select(count_star())
        .get_result::<i64>(conn)
        .map_err(|e| {
            tracing::error!("user ({}) already exists: {}", email_addr, e);
            unexpected_err_resp()
        })?;
    Ok(cnt != 0)
}

/// テストコードで共通で使うコードをまとめるモジュール
#[cfg(test)]
pub(crate) mod tests {
    use common::{smtp::SendMail, ErrResp};

    pub(crate) struct SendMailMock {
        to: String,
        from: String,
        subject: String,
        text: String,
    }

    impl SendMailMock {
        pub(crate) fn new(to: String, from: String, subject: String, text: String) -> Self {
            Self {
                to,
                from,
                subject,
                text,
            }
        }
    }

    impl SendMail for SendMailMock {
        fn send_mail(
            &self,
            to: &str,
            from: &str,
            subject: &str,
            text: &str,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.to, to);
            assert_eq!(self.from, from);
            assert_eq!(self.subject, subject);
            assert_eq!(self.text, text);
            Ok(())
        }
    }
}
