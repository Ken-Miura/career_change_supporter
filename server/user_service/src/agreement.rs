// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use axum::async_trait;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, FixedOffset, Utc};
use common::ApiError;
use common::ErrResp;
use entity::sea_orm::DatabaseConnection;
use tower_cookies::Cookies;

use crate::err::unexpected_err_resp;
use crate::err::Code::AlreadyAgreedTermsOfUse;
use crate::util::session::{RefreshOperationImpl, LOGIN_SESSION_EXPIRY};
use crate::util::JAPANESE_TIME_ZONE;
use crate::util::{session::get_user_by_cookie, terms_of_use::TERMS_OF_USE_VERSION};

/// ユーザーが利用規約に同意したことを記録する
pub(crate) async fn post_agreement(
    cookies: Cookies,
    Extension(store): Extension<RedisSessionStore>,
    Extension(pool): Extension<DatabaseConnection>,
) -> Result<StatusCode, ErrResp> {
    let op = RefreshOperationImpl {};
    let user = get_user_by_cookie(cookies, &store, op, LOGIN_SESSION_EXPIRY).await?;
    let op = AgreementOperationImpl::new(pool);
    let agreed_time = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
    let result =
        handle_agreement_req(user.account_id, *TERMS_OF_USE_VERSION, &agreed_time, op).await?;
    Ok(result)
}

async fn handle_agreement_req(
    account_id: i32,
    version: i32,
    agreed_at: &DateTime<FixedOffset>,
    op: impl AgreementOperation,
) -> Result<StatusCode, ErrResp> {
    let option = op.find_account_by_id(account_id).await?;
    let account = option.ok_or_else(|| {
        // 利用規約に同意するリクエストを送った後、アカウント情報が取得される前にアカウントが削除されるような事態は
        // 通常の操作では発生し得ない。そのため、unexpected_err_respとして処理する。
        unexpected_err_resp()
    })?;
    let _ = op
        .agree_terms_of_use(account_id, version, &account.email_address, agreed_at)
        .await?;
    tracing::info!(
        "{} (account id: {}) agreed terms of use version {} at {}",
        &account.email_address,
        account_id,
        version,
        agreed_at
    );
    Ok(StatusCode::OK)
}

#[async_trait]
trait AgreementOperation {
    async fn find_account_by_id(&self, account_id: i32) -> Result<Option<Account>, ErrResp>;
    async fn agree_terms_of_use(
        &self,
        account_id: i32,
        version: i32,
        email_address: &str,
        agreed_at: &DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct AgreementOperationImpl {
    pool: DatabaseConnection,
}

impl AgreementOperationImpl {
    fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }

    fn check_if_unique_violation(e: &diesel::result::Error) -> bool {
        matches!(
            e,
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            )
        )
    }
}

#[async_trait]
impl AgreementOperation for AgreementOperationImpl {
    async fn find_account_by_id(&self, account_id: i32) -> Result<Option<Account>, ErrResp> {
        // let result = user_account_table
        //     .find(id)
        //     .load::<Account>(&self.conn)
        //     .map_err(|e| {
        //         tracing::error!("failed to find user account (id: {}): {}", id, e);
        //         unexpected_err_resp()
        //     })?;
        // Ok(result)
        todo!()
    }

    async fn agree_terms_of_use(
        &self,
        account_id: i32,
        version: i32,
        email_address: &str,
        agreed_at: &DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        // let terms_of_use = NewTermsOfUse {
        //     user_account_id: &id,
        //     ver: &version,
        //     email_address,
        //     agreed_at,
        // };
        // let _ = insert_into(terms_of_use_table)
        //     .values(terms_of_use)
        //     .execute(&self.conn)
        //     .map_err(|e| {
        //         if AgreementOperationImpl::check_if_unique_violation(&e) {
        //             tracing::error!(
        //                 "id ({}) has already agreed terms of use(version: {}): {}",
        //                 id,
        //                 version,
        //                 e
        //             );
        //             return (
        //                 StatusCode::BAD_REQUEST,
        //                 Json(ApiError {
        //                     code: AlreadyAgreedTermsOfUse as u32,
        //                 }),
        //             );
        //         }
        //         tracing::error!(
        //             "failed to insert terms_of_use (id: {}, version: {}): {}",
        //             id,
        //             version,
        //             e
        //         );
        //         unexpected_err_resp()
        //     })?;
        // Ok(())
        todo!()
    }
}

#[derive(Clone, Debug)]
struct Account {
    email_address: String,
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::Json;
    use chrono::TimeZone;
    use common::{ApiError, ErrResp};
    use hyper::StatusCode;

    use crate::{err::Code::AlreadyAgreedTermsOfUse, util::JAPANESE_TIME_ZONE};

    use super::Account;
    use super::{handle_agreement_req, AgreementOperation};

    struct AgreementOperationMock<'a> {
        already_agreed_terms_of_use: bool,
        account_id: i32,
        version: i32,
        email_address: &'a str,
        agreed_at: &'a chrono::DateTime<chrono::FixedOffset>,
    }

    impl<'a> AgreementOperationMock<'a> {
        fn new(
            already_agreed_terms_of_use: bool,
            account_id: i32,
            version: i32,
            email_address: &'a str,
            agreed_at: &'a chrono::DateTime<chrono::FixedOffset>,
        ) -> Self {
            Self {
                already_agreed_terms_of_use,
                account_id,
                version,
                email_address,
                agreed_at,
            }
        }
    }

    #[async_trait]
    impl AgreementOperation for AgreementOperationMock<'_> {
        async fn find_account_by_id(&self, account_id: i32) -> Result<Option<Account>, ErrResp> {
            assert_eq!(self.account_id, account_id);
            let account = Account {
                email_address: self.email_address.to_string(),
            };
            Ok(Some(account))
        }

        async fn agree_terms_of_use(
            &self,
            account_id: i32,
            version: i32,
            email_address: &str,
            agreed_at: &chrono::DateTime<chrono::FixedOffset>,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.account_id, account_id);
            assert_eq!(self.version, version);
            assert_eq!(self.email_address, email_address);
            assert_eq!(self.agreed_at, agreed_at);
            if self.already_agreed_terms_of_use {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: AlreadyAgreedTermsOfUse as u32,
                    }),
                ));
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn handle_agreement_req_success() {
        let id = 51235;
        let email_address = "test@example.com";
        let version = 1;
        let agreed_at = chrono::Utc
            .ymd(2021, 11, 7)
            .and_hms(11, 00, 40)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned());
        let op = AgreementOperationMock::new(false, id, version, email_address, &agreed_at);

        let result = handle_agreement_req(id, version, &agreed_at, op)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result);
    }

    #[tokio::test]
    async fn handle_agreement_req_fail_already_agreed() {
        let id = 82546;
        let email_address = "test1234@example.com";
        let version = 1;
        let agreed_at = chrono::Utc
            .ymd(2021, 11, 7)
            .and_hms(11, 00, 40)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned());
        let op = AgreementOperationMock::new(true, id, version, email_address, &agreed_at);

        let result = handle_agreement_req(id, version, &agreed_at, op)
            .await
            .expect_err("failed to get Err");

        assert_eq!(StatusCode::BAD_REQUEST, result.0);
        assert_eq!(AlreadyAgreedTermsOfUse as u32, result.1 .0.code);
    }
}
