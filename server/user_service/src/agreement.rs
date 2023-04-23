// Copyright 2021 Ken Miura

use axum::async_trait;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, FixedOffset, Utc};
use common::ApiError;
use common::ErrResp;
use common::JAPANESE_TIME_ZONE;
use entity::prelude::TermsOfUse;
use entity::sea_orm::ActiveModelTrait;
use entity::sea_orm::DatabaseConnection;
use entity::sea_orm::EntityTrait;
use entity::sea_orm::Set;
use entity::terms_of_use;
use tracing::error;
use tracing::info;

use crate::err::unexpected_err_resp;
use crate::err::Code::AlreadyAgreedTermsOfUse;
use crate::util::session::agreement_unchecked_user::AgreementUncheckedUser;
use crate::util::terms_of_use::TERMS_OF_USE_VERSION;

/// ユーザーが利用規約に同意したことを記録する
pub(crate) async fn post_agreement(
    AgreementUncheckedUser { user_info }: AgreementUncheckedUser,
    State(pool): State<DatabaseConnection>,
) -> Result<StatusCode, ErrResp> {
    let op = AgreementOperationImpl::new(pool);
    let agreed_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let result = handle_agreement_req(
        user_info.account_id,
        user_info.email_address,
        *TERMS_OF_USE_VERSION,
        &agreed_time,
        op,
    )
    .await?;
    Ok(result)
}

async fn handle_agreement_req(
    account_id: i64,
    email_address: String,
    version: i32,
    agreed_at: &DateTime<FixedOffset>,
    op: impl AgreementOperation,
) -> Result<StatusCode, ErrResp> {
    let agreement_date_option = op.check_if_already_agreed(account_id, version).await?;
    if let Some(agreement_date) = agreement_date_option {
        error!(
            "{} (account id: {}) has already agreed terms of use (version {}) at {}",
            &email_address,
            account_id,
            version,
            agreement_date.with_timezone(&(*JAPANESE_TIME_ZONE))
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: AlreadyAgreedTermsOfUse as u32,
            }),
        ));
    }
    // ACID特性が重要な箇所ではないので、op.check_if_already_agreedとまとめてトランザクションにしていない
    op.agree_terms_of_use(account_id, version, &email_address, agreed_at)
        .await?;
    info!(
        "{} (account id: {}) agreed terms of use (version {}) at {}",
        &email_address, account_id, version, agreed_at
    );
    Ok(StatusCode::OK)
}

#[async_trait]
trait AgreementOperation {
    async fn check_if_already_agreed(
        &self,
        account_id: i64,
        version: i32,
    ) -> Result<Option<AgreedDateTime>, ErrResp>;
    async fn agree_terms_of_use(
        &self,
        account_id: i64,
        version: i32,
        email_address: &str,
        agreed_at: &DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

type AgreedDateTime = DateTime<FixedOffset>;

struct AgreementOperationImpl {
    pool: DatabaseConnection,
}

impl AgreementOperationImpl {
    fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AgreementOperation for AgreementOperationImpl {
    async fn check_if_already_agreed(
        &self,
        account_id: i64,
        version: i32,
    ) -> Result<Option<AgreedDateTime>, ErrResp> {
        let model = TermsOfUse::find_by_id((account_id, version))
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find terms_of_use (user_account_id: {}, ver: {}): {}",
                    account_id, version, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| m.agreed_at))
    }

    async fn agree_terms_of_use(
        &self,
        account_id: i64,
        version: i32,
        email_address: &str,
        agreed_at: &DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        let terms_of_use_model = terms_of_use::ActiveModel {
            user_account_id: Set(account_id),
            ver: Set(version),
            email_address: Set(email_address.to_string()),
            agreed_at: Set(*agreed_at),
        };
        let _ = terms_of_use_model.insert(&self.pool).await.map_err(|e| {
            error!("failed to insert terms_of_use (user_account_id: {}, email_address: {}, ver: {}): {}", 
            account_id, email_address, version, e);
            unexpected_err_resp()
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use chrono::TimeZone;
    use common::ErrResp;
    use common::JAPANESE_TIME_ZONE;

    use crate::err::Code::AlreadyAgreedTermsOfUse;

    use super::AgreedDateTime;
    use super::{handle_agreement_req, AgreementOperation};

    struct AgreementOperationMock<'a> {
        already_agreed_terms_of_use: bool,
        agreed_at_before: &'a chrono::DateTime<chrono::FixedOffset>,
        account_id: i64,
        version: i32,
        email_address: &'a str,
        agreed_at: &'a chrono::DateTime<chrono::FixedOffset>,
    }

    impl<'a> AgreementOperationMock<'a> {
        fn new(
            already_agreed_terms_of_use: bool,
            agreed_at_before: &'a chrono::DateTime<chrono::FixedOffset>,
            account_id: i64,
            version: i32,
            email_address: &'a str,
            agreed_at: &'a chrono::DateTime<chrono::FixedOffset>,
        ) -> Self {
            Self {
                already_agreed_terms_of_use,
                agreed_at_before,
                account_id,
                version,
                email_address,
                agreed_at,
            }
        }
    }

    #[async_trait]
    impl AgreementOperation for AgreementOperationMock<'_> {
        async fn check_if_already_agreed(
            &self,
            account_id: i64,
            version: i32,
        ) -> Result<Option<AgreedDateTime>, ErrResp> {
            assert_eq!(self.account_id, account_id);
            assert_eq!(self.version, version);
            if self.already_agreed_terms_of_use {
                return Ok(Some(*self.agreed_at_before));
            }
            Ok(None)
        }

        async fn agree_terms_of_use(
            &self,
            account_id: i64,
            version: i32,
            email_address: &str,
            agreed_at: &chrono::DateTime<chrono::FixedOffset>,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.account_id, account_id);
            assert_eq!(self.version, version);
            assert_eq!(self.email_address, email_address);
            assert_eq!(self.agreed_at, agreed_at);
            Ok(())
        }
    }

    #[tokio::test]
    async fn handle_agreement_req_success() {
        let id = 51235;
        let email_address = "test@example.com";
        let version = 1;
        let agreed_at = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 11, 7, 11, 00, 40)
            .unwrap();
        let agreed_at_before = agreed_at - chrono::Duration::days(1);
        let op = AgreementOperationMock::new(
            false,
            &agreed_at_before,
            id,
            version,
            email_address,
            &agreed_at,
        );

        let result = handle_agreement_req(id, email_address.to_string(), version, &agreed_at, op)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result);
    }

    #[tokio::test]
    async fn handle_agreement_req_fail_already_agreed() {
        let id = 82546;
        let email_address = "test1234@example.com";
        let version = 1;
        let agreed_at = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 11, 7, 11, 00, 40)
            .unwrap();
        let agreed_at_before = agreed_at - chrono::Duration::days(1);
        let op = AgreementOperationMock::new(
            true,
            &agreed_at_before,
            id,
            version,
            email_address,
            &agreed_at,
        );

        let result = handle_agreement_req(id, email_address.to_string(), version, &agreed_at, op)
            .await
            .expect_err("failed to get Err");

        assert_eq!(StatusCode::BAD_REQUEST, result.0);
        assert_eq!(AlreadyAgreedTermsOfUse as u32, result.1 .0.code);
    }
}
