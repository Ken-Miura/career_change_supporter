// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::prelude::Consultation;
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use opensearch::OpenSearch;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::util;
use crate::util::disabled_check::DisabledCheckOperationImpl;
use crate::util::session::User;

use super::{ensure_rating_id_is_positive, ensure_rating_is_in_valid_range, ConsultationInfo};

pub(crate) async fn post_consultant_rating(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
    State(index_client): State<OpenSearch>,
    Json(req): Json<ConsultantRatingParam>,
) -> RespResult<ConsultantRatingResult> {
    let op = ConsultantRatingOperationImpl { pool, index_client };
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    handle_consultant_rating(
        account_id,
        req.consultant_rating_id,
        req.rating,
        &current_date_time,
        op,
    )
    .await
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ConsultantRatingParam {
    consultant_rating_id: i64,
    rating: i16,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct ConsultantRatingResult {}

#[async_trait]
trait ConsultantRatingOperation {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;

    async fn check_if_user_account_is_available(
        &self,
        user_account_id: i64,
    ) -> Result<bool, ErrResp>;

    async fn find_consultation_info_from_consultant_rating(
        &self,
        consultant_rating_id: i64,
    ) -> Result<Option<ConsultationInfo>, ErrResp>;

    async fn update_consultant_rating(
        &self,
        consultant_id: i64,
        consultant_rating_id: i64,
        rating: i16,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;

    async fn filter_consultant_rating_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Vec<i16>, ErrResp>;

    async fn update_rating_on_document_if_needed(
        &self,
        consultant_id: i64,
        averate_rating: f64,
    ) -> Result<(), ErrResp>;

    async fn make_payment_if_needed(&self, consultation_id: i64) -> Result<(), ErrResp>;
}

struct ConsultantRatingOperationImpl {
    pool: DatabaseConnection,
    index_client: OpenSearch,
}

#[async_trait]
impl ConsultantRatingOperation for ConsultantRatingOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::identity_checker::check_if_identity_exists(&self.pool, account_id).await
    }

    async fn check_if_user_account_is_available(
        &self,
        user_account_id: i64,
    ) -> Result<bool, ErrResp> {
        let op = DisabledCheckOperationImpl::new(&self.pool);
        util::disabled_check::check_if_user_account_is_available(user_account_id, op).await
    }

    async fn find_consultation_info_from_consultant_rating(
        &self,
        consultant_rating_id: i64,
    ) -> Result<Option<ConsultationInfo>, ErrResp> {
        let model = entity::consultant_rating::Entity::find_by_id(consultant_rating_id)
            .find_also_related(Consultation)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find consultant_rating and consultation (consultant_rating_id: {}): {}",
                    consultant_rating_id, e
                );
                unexpected_err_resp()
            })?;
        let converted_result = model.map(|m| {
            let c = m.1.ok_or_else(|| {
                error!(
                    "failed to find consultation (consultant_rating_id: {}, consultation_id: {})",
                    consultant_rating_id, m.0.consultation_id
                );
                unexpected_err_resp()
            })?;
            Ok(ConsultationInfo {
                user_account_id: c.user_account_id,
                consultant_id: c.consultant_id,
                consultation_date_time_in_jst: c.meeting_at.with_timezone(&(*JAPANESE_TIME_ZONE)),
            })
        });
        Ok(match converted_result {
            Some(r) => Some(r?),
            None => None,
        })
    }

    async fn update_consultant_rating(
        &self,
        consultant_id: i64,
        consultant_rating_id: i64,
        rating: i16,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        todo!()
    }

    async fn filter_consultant_rating_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Vec<i16>, ErrResp> {
        todo!()
    }

    async fn update_rating_on_document_if_needed(
        &self,
        consultant_id: i64,
        averate_rating: f64,
    ) -> Result<(), ErrResp> {
        todo!()
    }

    async fn make_payment_if_needed(&self, consultation_id: i64) -> Result<(), ErrResp> {
        todo!()
    }
}

async fn handle_consultant_rating(
    account_id: i64,
    consultant_rating_id: i64,
    rating: i16,
    current_date_time: &DateTime<FixedOffset>,
    op: impl ConsultantRatingOperation,
) -> RespResult<ConsultantRatingResult> {
    ensure_rating_id_is_positive(consultant_rating_id)?;
    ensure_rating_is_in_valid_range(rating)?;
    ensure_identity_exists(account_id, &op).await?;
    ensure_user_account_is_available(account_id, &op).await?;

    // consultant_rating_idでconsultant_ratingを取得
    // consultant_ratingのユーザーとaccount_idが一致していることを確認する
    // consultant_ratingにある相談時間とcurrent_date_timeを用いて評価を実施可能かチェックする
    // consultant_ratingを更新する
    //   コンサルタントの存在チェック＋ロック -> 仮に存在しない場合はそれ以降の操作は何もしないで成功で終わらせる
    //   consultant_ratingの取得＋ロック
    //   consultant_ratingのratingがNULLであることを確認 -> NULLでないなら既に評価済を示すエラーを返す
    //   consultant_ratingのratingに値を入れる
    // コンサルタントのDisabledチェック -> Disabledなら何もしない。DisabledでないならOpenSearchにconsultant_ratingの集計結果を投入
    // pay.jpのchargeの更新
    //   settlementテーブルからreceiptテーブルに移す -> settlementテーブルがなければ既に定期ツールが処理済のため、そのままOKを返す
    //   pay.jpにcharge更新のリクエスト
    todo!()
}

async fn ensure_identity_exists(
    account_id: i64,
    op: &impl ConsultantRatingOperation,
) -> Result<(), ErrResp> {
    let identity_exists = op.check_if_identity_exists(account_id).await?;
    if !identity_exists {
        error!("identity is not registered (account_id: {})", account_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoIdentityRegistered as u32,
            }),
        ));
    }
    Ok(())
}

async fn ensure_user_account_is_available(
    user_account_id: i64,
    op: &impl ConsultantRatingOperation,
) -> Result<(), ErrResp> {
    let available = op
        .check_if_user_account_is_available(user_account_id)
        .await?;
    if !available {
        error!(
            "user account is not available (user_account_id: {})",
            user_account_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::UserIsNotAvailable as u32,
            }),
        ));
    }
    Ok(())
}
