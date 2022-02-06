// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use common::{model::user::Account, ErrResp};
use common::{
    model::user::NewTermsOfUse,
    schema::ccs_schema::terms_of_use::dsl::terms_of_use as terms_of_use_table,
    schema::ccs_schema::user_account::dsl::user_account as user_account_table,
};
use common::{ApiError, DatabaseConnection};
use diesel::{
    insert_into,
    r2d2::{ConnectionManager, PooledConnection},
    PgConnection, QueryDsl, RunQueryDsl,
};
use tower_cookies::Cookies;

use crate::err::Code::AlreadyAgreedTermsOfUse;
use crate::util::session::{RefreshOperationImpl, LOGIN_SESSION_EXPIRY};
use crate::util::{
    session::get_user_by_cookie, terms_of_use::TERMS_OF_USE_VERSION, unexpected_err_resp,
};

/// ユーザーが利用規約に同意したことを記録する
pub(crate) async fn post_agreement(
    cookies: Cookies,
    Extension(store): Extension<RedisSessionStore>,
    DatabaseConnection(conn): DatabaseConnection,
) -> Result<StatusCode, ErrResp> {
    let op = RefreshOperationImpl {};
    let user = get_user_by_cookie(cookies, &store, op, LOGIN_SESSION_EXPIRY).await?;
    let op = AgreementOperationImpl::new(conn);
    let agreed_time = Utc::now();
    let result =
        post_agreement_internal(user.account_id, *TERMS_OF_USE_VERSION, &agreed_time, op).await?;
    Ok(result)
}

trait AgreementOperation {
    fn find_account_by_id(&self, id: i32) -> Result<Vec<Account>, ErrResp>;
    fn agree_terms_of_use(
        &self,
        id: i32,
        version: i32,
        email_address: &str,
        agreed_at: &DateTime<Utc>,
    ) -> Result<(), ErrResp>;
}

struct AgreementOperationImpl {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
}

impl AgreementOperationImpl {
    fn new(conn: PooledConnection<ConnectionManager<PgConnection>>) -> Self {
        Self { conn }
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

impl AgreementOperation for AgreementOperationImpl {
    fn find_account_by_id(&self, id: i32) -> Result<Vec<Account>, ErrResp> {
        let result = user_account_table
            .find(id)
            .load::<Account>(&self.conn)
            .map_err(|e| {
                tracing::error!("failed to find user account (id: {}): {}", id, e);
                unexpected_err_resp()
            })?;
        Ok(result)
    }

    fn agree_terms_of_use(
        &self,
        id: i32,
        version: i32,
        email_address: &str,
        agreed_at: &DateTime<Utc>,
    ) -> Result<(), ErrResp> {
        let terms_of_use = NewTermsOfUse {
            user_account_id: &id,
            ver: &version,
            email_address,
            agreed_at,
        };
        let _ = insert_into(terms_of_use_table)
            .values(terms_of_use)
            .execute(&self.conn)
            .map_err(|e| {
                if AgreementOperationImpl::check_if_unique_violation(&e) {
                    tracing::error!(
                        "id ({}) has already agreed terms of use(version: {}): {}",
                        id,
                        version,
                        e
                    );
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ApiError {
                            code: AlreadyAgreedTermsOfUse as u32,
                        }),
                    );
                }
                tracing::error!(
                    "failed to insert terms_of_use (id: {}, version: {}): {}",
                    id,
                    version,
                    e
                );
                unexpected_err_resp()
            })?;
        Ok(())
    }
}

async fn post_agreement_internal(
    id: i32,
    version: i32,
    agreed_at: &DateTime<Utc>,
    op: impl AgreementOperation,
) -> Result<StatusCode, ErrResp> {
    let _ = async move {
        let accounts = op.find_account_by_id(id)?;
        let len = accounts.len();
        if len != 1 {
            // len != 1 -> アカウントが存在するかどうか（削除されていないかどうか）のチェック
            // 利用規約同意には、アカウントを削除された後の事も想定し、アカウントID以外にも最低限のユーザー情報（Eメールアドレス）を含めたい。
            // そのため、Eメールアドレスが取得できない場合はエラーとする。
            // (アカウントが削除されると、アカウントIDとそれに紐付いている情報はすべて削除される。
            // もし、アカウントIDしか利用規約に保存していないと、紐付いている連絡先（Eメールアドレス）の追跡ができなくなる）
            tracing::error!(
                "number of user accounts is not 1 (id: {}, length: {})",
                id,
                len
            );
            // 利用規約に同意するのリクエストを送った後、ユーザー情報の取得と利用規約同意の記録までの間にアカウントが削除されるような事態は通常の操作では発生し得ない。
            // そのため、unexpected errとして処理する。
            return Err(unexpected_err_resp());
        }
        let _ = op.agree_terms_of_use(id, version, &accounts[0].email_address, agreed_at)?;
        tracing::info!(
            "{} (id: {}) agreed terms of use version {} at {}",
            &accounts[0].email_address,
            id,
            version,
            agreed_at
        );
        Ok(())
    }
    .await?;
    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use axum::Json;
    use chrono::{Duration, TimeZone};
    use common::{model::user::Account, util::hash_password, ApiError, ErrResp};
    use hyper::StatusCode;

    use crate::err::Code::AlreadyAgreedTermsOfUse;

    use super::{post_agreement_internal, AgreementOperation};

    struct AgreementOperationMock<'a> {
        already_agreed_terms_of_use: bool,
        id: i32,
        version: i32,
        email_address: &'a str,
        agreed_at: &'a chrono::DateTime<chrono::Utc>,
    }

    impl<'a> AgreementOperationMock<'a> {
        fn new(
            already_agreed_terms_of_use: bool,
            id: i32,
            version: i32,
            email_address: &'a str,
            agreed_at: &'a chrono::DateTime<chrono::Utc>,
        ) -> Self {
            Self {
                already_agreed_terms_of_use,
                id,
                version,
                email_address,
                agreed_at,
            }
        }
    }

    impl AgreementOperation for AgreementOperationMock<'_> {
        fn find_account_by_id(&self, id: i32) -> Result<Vec<Account>, ErrResp> {
            assert_eq!(self.id, id);
            let hashed_password = hash_password("aaaaaaaaaA")
                .map_err(|e| panic!("failed to handle password: {}", e))?;
            let last_login_time = *self.agreed_at - Duration::minutes(10);
            let created_at = *self.agreed_at - Duration::minutes(30);
            let account = Account {
                user_account_id: self.id,
                email_address: self.email_address.to_string(),
                hashed_password,
                last_login_time: Some(last_login_time),
                created_at: created_at,
            };
            Ok(vec![account])
        }

        fn agree_terms_of_use(
            &self,
            id: i32,
            version: i32,
            email_address: &str,
            agreed_at: &chrono::DateTime<chrono::Utc>,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.id, id);
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
            return Ok(());
        }
    }

    #[tokio::test]
    async fn agreement_success() {
        let id = 51235;
        let email_address = "test@example.com";
        let version = 1;
        let agreed_at = chrono::Utc.ymd(2021, 11, 7).and_hms(11, 00, 40);
        let op = AgreementOperationMock::new(false, id, version, email_address, &agreed_at);

        let result = post_agreement_internal(id, version, &agreed_at, op)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result);
    }

    #[tokio::test]
    async fn agreement_fail_already_agreed() {
        let id = 82546;
        let email_address = "test1234@example.com";
        let version = 1;
        let agreed_at = chrono::Utc.ymd(2021, 11, 7).and_hms(11, 00, 40);
        let op = AgreementOperationMock::new(true, id, version, email_address, &agreed_at);

        let result = post_agreement_internal(id, version, &agreed_at, op)
            .await
            .expect_err("failed to get Err");

        assert_eq!(StatusCode::BAD_REQUEST, result.0);
        assert_eq!(AlreadyAgreedTermsOfUse as u32, result.1 .0.code);
    }
}
