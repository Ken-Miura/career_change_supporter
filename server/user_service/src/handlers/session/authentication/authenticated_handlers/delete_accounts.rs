// Copyright 2023 Ken Miura

use async_fred_session::RedisSessionStore;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::SignedCookieJar;
use chrono::{DateTime, Duration, FixedOffset};
use common::opensearch::{delete_document, INDEX_NAME};
use common::smtp::{SendMail, SmtpClient, INQUIRY_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS};
use common::{
    ApiError, ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE, LENGTH_OF_MEETING_IN_MINUTE,
    WEB_SITE_NAME,
};
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    ModelTrait, QueryFilter, Set, TransactionError, TransactionTrait,
};
use once_cell::sync::Lazy;
use opensearch::OpenSearch;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::err::{unexpected_err_resp, Code};
use crate::handlers::session::authentication::user_operation::find_user_account_by_user_account_id_with_exclusive_lock;
use crate::handlers::session::{destroy_session_if_exists, SESSION_ID_COOKIE_NAME};

use super::authenticated_users::get_user_info_from_cookie;
use super::document_operation::find_document_model_by_user_account_id_with_exclusive_lock;

static SUBJECT: Lazy<String> = Lazy::new(|| format!("[{}] アカウント削除完了通知", WEB_SITE_NAME));

// 認証が必要なハンドラは、UserInfoを持つ構造体を利用し、この関数に届く前に認証処理が完了していることを保証するように書く
// しかし、delete_accountsは通常の認証処理とは異なる処理（セッションIDを取得しておき、そのセッションを破棄する処理）が必要なのでCookieを直接受け取り、
// このハンドラ内で認証処理が完了しているか確認する
pub(crate) async fn delete_accounts(
    jar: SignedCookieJar,
    query: Query<DeleteAccountsQuery>,
    State(smtp_client): State<SmtpClient>,
    State(store): State<RedisSessionStore>,
    State(pool): State<DatabaseConnection>,
    State(index_client): State<OpenSearch>,
) -> Result<(StatusCode, SignedCookieJar), ErrResp> {
    let option_cookie = jar.get(SESSION_ID_COOKIE_NAME);
    let user_info = get_user_info_from_cookie(option_cookie.clone(), &store, &pool).await?;

    let account_delete_confirmed = query.0.account_delete_confirmed;
    let current_date_time = chrono::Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = DeleteAccountsOperationImpl { pool, index_client };

    let _ = handle_delete_accounts(
        user_info.account_id,
        user_info.email_address,
        account_delete_confirmed,
        current_date_time,
        &op,
        &smtp_client,
    )
    .await?;

    if let Some(session_id) = option_cookie {
        destroy_session_if_exists(session_id.value(), &store).await?;
    }
    Ok((
        StatusCode::OK,
        jar.remove(Cookie::from(SESSION_ID_COOKIE_NAME)),
    ))
}

#[derive(Deserialize)]
pub(crate) struct DeleteAccountsQuery {
    account_delete_confirmed: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct DeleteAccountsResult {}

#[async_trait]
trait DeleteAccountsOperation {
    async fn get_undone_consultation_ids(
        &self,
        criteria: DateTime<FixedOffset>,
    ) -> Result<Vec<i64>, ErrResp>;

    async fn delete_user_account(
        &self,
        account_id: i64,
        index_name: String,
        deleted_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct DeleteAccountsOperationImpl {
    pool: DatabaseConnection,
    index_client: OpenSearch,
}

#[async_trait]
impl DeleteAccountsOperation for DeleteAccountsOperationImpl {
    async fn get_undone_consultation_ids(
        &self,
        criteria: DateTime<FixedOffset>,
    ) -> Result<Vec<i64>, ErrResp> {
        let undone_consultations = entity::consultation::Entity::find()
            .filter(entity::consultation::Column::MeetingAt.gt(criteria))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter consultation (criteria: {}): {}",
                    criteria, e
                );
                unexpected_err_resp()
            })?;
        Ok(undone_consultations
            .into_iter()
            .map(|m| m.consultation_id)
            .collect())
    }

    async fn delete_user_account(
        &self,
        account_id: i64,
        index_name: String,
        deleted_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        let index_client = self.index_client.clone();
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    delete_user_account_with_user_account_exclusive_lock(
                        txn,
                        account_id,
                        deleted_date_time,
                    )
                    .await?;

                    delete_career_from_index_with_document_exclusive_lock(
                        txn,
                        account_id,
                        index_name,
                        index_client,
                    )
                    .await?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(db_err) => {
                    error!("connection error: {}", db_err);
                    unexpected_err_resp()
                }
                TransactionError::Transaction(err_resp_struct) => {
                    error!("failed to delete_user_account: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}

async fn delete_user_account_with_user_account_exclusive_lock(
    txn: &DatabaseTransaction,
    account_id: i64,
    deleted_date_time: DateTime<FixedOffset>,
) -> Result<(), ErrRespStruct> {
    let user_option =
        find_user_account_by_user_account_id_with_exclusive_lock(txn, account_id).await?;
    let user = user_option.ok_or_else(|| {
        error!(
            "failed to find user_account (user_account_id: {})",
            account_id
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;

    insert_deleted_user_account(txn, &user, deleted_date_time).await?;

    let _ = user.delete(txn).await.map_err(|e| {
        error!(
            "failed to delete user_account (user_account_id: {}): {}",
            account_id, e
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}

async fn insert_deleted_user_account(
    txn: &DatabaseTransaction,
    model: &entity::user_account::Model,
    deleted_date_time: DateTime<FixedOffset>,
) -> Result<(), ErrRespStruct> {
    let active_model = entity::deleted_user_account::ActiveModel {
        user_account_id: Set(model.user_account_id),
        email_address: Set(model.email_address.clone()),
        hashed_password: Set(model.hashed_password.clone()),
        last_login_time: Set(model.last_login_time),
        created_at: Set(model.created_at),
        mfa_enabled_at: Set(model.mfa_enabled_at),
        disabled_at: Set(model.disabled_at),
        deleted_at: Set(deleted_date_time),
    };
    let _ = active_model.insert(txn).await.map_err(|e| {
        error!(
            "failed to insert deleted_user_account (user_account: {:?}, deleted_date_time: {}): {}",
            model, deleted_date_time, e
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}

async fn delete_career_from_index_with_document_exclusive_lock(
    txn: &DatabaseTransaction,
    account_id: i64,
    index_name: String,
    index_client: OpenSearch,
) -> Result<(), ErrRespStruct> {
    let document_option =
        find_document_model_by_user_account_id_with_exclusive_lock(txn, account_id).await?;
    let document = match document_option {
        Some(d) => d,
        None => {
            info!(
                "no document (career info on index) found (user account id: {})",
                account_id
            );
            return Ok(());
        }
    };

    let document_id = document.document_id.to_string();
    let _ = document.delete(txn).await.map_err(|e| {
        error!(
            "failed to delete document (user_account_id: {}): {}",
            account_id, e
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;

    delete_document(index_name.as_str(), document_id.as_str(), &index_client).await.map_err(|e|{
      error!(
        "failed to delete document (user_account_id: {}, index_name: {}, document_id: {}) from Opensearch",
        account_id, index_name, document_id
      );
      ErrRespStruct {
        err_resp: e,
      }
    })?;

    Ok(())
}

fn create_text() -> String {
    format!(
        r"アカウントの削除が完了しました。

本メールはシステムより自動配信されています。
本メールに返信されましても、回答いたしかねます。
お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

【お問い合わせ先】
Email: {}",
        INQUIRY_EMAIL_ADDRESS.as_str()
    )
}

async fn handle_delete_accounts(
    account_id: i64,
    email_address: String,
    account_delete_confirmed: bool,
    current_date_time: DateTime<FixedOffset>,
    op: &impl DeleteAccountsOperation,
    send_mail: &impl SendMail,
) -> RespResult<DeleteAccountsResult> {
    ensure_account_delete_confirmed(account_id, account_delete_confirmed)?;

    let criteria = current_date_time - Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64);
    let undone_consultation_ids = op.get_undone_consultation_ids(criteria).await?;
    if !undone_consultation_ids.is_empty() {
        error!(
            "cannot delete user_account (user_account: {}) because cnosultations ({:?}) are not done yet",
            account_id, undone_consultation_ids
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ConsultationHasNotBeenFinished as u32,
            }),
        ));
    }

    op.delete_user_account(account_id, INDEX_NAME.to_string(), current_date_time)
        .await?;

    let text = create_text();
    send_mail
        .send_mail(
            &email_address,
            SYSTEM_EMAIL_ADDRESS.as_str(),
            &SUBJECT,
            &text,
        )
        .await?;

    info!(
        "deleted account (user account id: {}, email address: {}) at {}",
        account_id, email_address, current_date_time
    );
    Ok((StatusCode::OK, Json(DeleteAccountsResult {})))
}

fn ensure_account_delete_confirmed(
    account_id: i64,
    account_delete_confirmed: bool,
) -> Result<(), ErrResp> {
    if !account_delete_confirmed {
        error!(
            "user account (account id: {}) does not check account_delete_confirmed",
            account_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::AccountDeleteIsNotConfirmed as u32,
            }),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use chrono::TimeZone;

    use crate::handlers::tests::SendMailMock;

    use super::*;

    struct DeleteAccountsOperationMock {
        account_id: i64,
        undone_consutation_ids: Vec<i64>,
        current_date_time: DateTime<FixedOffset>,
    }

    #[async_trait]
    impl DeleteAccountsOperation for DeleteAccountsOperationMock {
        async fn get_undone_consultation_ids(
            &self,
            criteria: DateTime<FixedOffset>,
        ) -> Result<Vec<i64>, ErrResp> {
            assert_eq!(
                self.current_date_time - Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64),
                criteria
            );
            Ok(self.undone_consutation_ids.clone())
        }

        async fn delete_user_account(
            &self,
            account_id: i64,
            index_name: String,
            deleted_date_time: DateTime<FixedOffset>,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.account_id, account_id);
            assert_eq!(INDEX_NAME, index_name);
            assert_eq!(self.current_date_time, deleted_date_time);
            Ok(())
        }
    }

    #[tokio::test]
    async fn handle_delete_accounts_success_undone_consutation() {
        let account_id = 5517;
        let email_address = "test0@test.com";
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 4, 21, 0, 1, 7)
            .unwrap();
        let op = DeleteAccountsOperationMock {
            account_id,
            undone_consutation_ids: vec![],
            current_date_time,
        };
        let send_mail_mock = SendMailMock::new(
            email_address.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );

        let result = handle_delete_accounts(
            account_id,
            email_address.to_string(),
            true,
            current_date_time,
            &op,
            &send_mail_mock,
        )
        .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(DeleteAccountsResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn handle_delete_accounts_fail_1_undone_consultation() {
        let account_id = 5517;
        let email_address = "test0@test.com";
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 4, 21, 0, 1, 7)
            .unwrap();
        let op = DeleteAccountsOperationMock {
            account_id,
            undone_consutation_ids: vec![51],
            current_date_time,
        };
        let send_mail_mock = SendMailMock::new(
            email_address.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );

        let result = handle_delete_accounts(
            account_id,
            email_address.to_string(),
            true,
            current_date_time,
            &op,
            &send_mail_mock,
        )
        .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::ConsultationHasNotBeenFinished as u32, resp.1 .0.code);
    }

    #[tokio::test]
    async fn handle_delete_accounts_fail_2_undone_consultations() {
        let account_id = 5517;
        let email_address = "test0@test.com";
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 4, 21, 0, 1, 7)
            .unwrap();
        let op = DeleteAccountsOperationMock {
            account_id,
            undone_consutation_ids: vec![51, 89],
            current_date_time,
        };
        let send_mail_mock = SendMailMock::new(
            email_address.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );

        let result = handle_delete_accounts(
            account_id,
            email_address.to_string(),
            true,
            current_date_time,
            &op,
            &send_mail_mock,
        )
        .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::ConsultationHasNotBeenFinished as u32, resp.1 .0.code);
    }

    #[tokio::test]
    async fn handle_delete_accounts_fail() {
        let account_id = 5517;
        let email_address = "test0@test.com";
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 4, 21, 0, 1, 7)
            .unwrap();
        let op = DeleteAccountsOperationMock {
            account_id,
            undone_consutation_ids: vec![],
            current_date_time,
        };
        let send_mail_mock = SendMailMock::new(
            email_address.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );

        let result = handle_delete_accounts(
            account_id,
            email_address.to_string(),
            false,
            current_date_time,
            &op,
            &send_mail_mock,
        )
        .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::AccountDeleteIsNotConfirmed as u32, resp.1 .0.code);
    }
}
