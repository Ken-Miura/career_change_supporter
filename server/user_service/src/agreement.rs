// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use axum::Json;
use axum::{body::Body, http::Request, http::StatusCode};
use chrono::{DateTime, Utc};
use common::ApiError;
use common::{model::user::Account, ConnectionPool, ErrResp};
use common::{
    model::user::NewTermsOfUse,
    schema::ccs_schema::terms_of_use::dsl::terms_of_use as terms_of_use_table,
    schema::ccs_schema::user_account::dsl::user_account as user_account_table,
};
use diesel::{
    insert_into,
    r2d2::{ConnectionManager, PooledConnection},
    PgConnection, QueryDsl, RunQueryDsl,
};
use headers::{Cookie, HeaderMapExt};

use crate::err_code::ALREADY_AGREED_TERMS_OF_USE;
use crate::util::{
    session::get_user_by_cookie, terms_of_use::TERMS_OF_USE_VERSION, unexpected_err_resp,
};

/// ユーザーが利用規約に同意したことを記録する
pub(crate) async fn post_agreement(req: Request<Body>) -> Result<StatusCode, ErrResp> {
    let headers = req.headers();
    let option_cookie = headers.typed_try_get::<Cookie>().map_err(|e| {
        tracing::error!("failed to get cookie: {}", e);
        unexpected_err_resp()
    })?;
    let extentions = req.extensions();
    let store = extentions.get::<RedisSessionStore>().ok_or_else(|| {
        tracing::error!("failed to get session store");
        unexpected_err_resp()
    })?;
    let user = get_user_by_cookie(option_cookie, store).await?;

    let pool = extentions.get::<ConnectionPool>().ok_or_else(|| {
        tracing::error!("failed to get session store");
        unexpected_err_resp()
    })?;
    let conn = pool.get().map_err(|e| {
        tracing::error!("failed to get connection from pool: {}", e);
        unexpected_err_resp()
    })?;
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
        match e {
            diesel::result::Error::DatabaseError(kind, _) => match kind {
                diesel::result::DatabaseErrorKind::UniqueViolation => true,
                _ => false,
            },
            _ => false,
        }
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
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ApiError {
                            code: ALREADY_AGREED_TERMS_OF_USE,
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
            tracing::error!(
                "number of user accounts is not 1 (id: {}, length: {})",
                id,
                len
            );
            // 利用規約同意には、アカウントを削除された後の事も想定し、アカウントID以外にも最低限のユーザー情報（Eメールアドレス）を含めたい。
            // そのため、Eメールアドレスが取得できない場合はエラーとする。
            // (アカウントが削除されると、アカウントIDとそれに紐付いている情報はすべて削除される。
            // もし、アカウントIDしか利用規約に保存していないと、紐付いている連絡先（Eメールアドレス）の追跡ができなくなる）
            //
            // 利用規約に同意するのリクエストを送った後、ユーザー情報の取得と利用規約同意の記録までの間にアカウントが削除されるような事態は通常の操作では発生し得ない。
            // そのため、unexpected errとして処理する。
            return Err(unexpected_err_resp());
        }
        op.agree_terms_of_use(id, version, &accounts[0].email_address, agreed_at)
    }
    .await?;
    Ok(StatusCode::OK)
}
