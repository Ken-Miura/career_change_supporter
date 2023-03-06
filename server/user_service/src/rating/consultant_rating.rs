// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset};
use common::{ErrResp, RespResult};
use entity::sea_orm::DatabaseConnection;
use opensearch::OpenSearch;
use serde::{Deserialize, Serialize};

use crate::util::session::User;

pub(crate) async fn post_consultant_rating(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
    State(index_client): State<OpenSearch>,
    Json(req): Json<ConsultantRatingParam>,
) -> RespResult<ConsultantRatingResult> {
    todo!()
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

    async fn find_consultant_rating_by_consultant_rating_id(
        &self,
        consultant_rating_id: i64,
    ) -> Result<Option<ConsultantRating>, ErrResp>;

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
    ) -> Result<Vec<Option<i16>>, ErrResp>;

    async fn update_rating_on_document_if_needed(
        &self,
        consultant_id: i64,
        averate_rating: i16,
    ) -> Result<(), ErrResp>;

    async fn make_payment_if_needed(&self, charge_id: &str) -> Result<(), ErrResp>;
}

#[derive(Clone, Debug)]
struct ConsultantRating {
    consultant_id: i64,
    user_account_id: i64,
    consultation_date_time_in_jst: DateTime<FixedOffset>,
    charge_id: String,
}

async fn handle_consultant_rating(
    account_id: i64,
    consultant_rating_id: i64,
    current_date_time: &DateTime<FixedOffset>,
    op: impl ConsultantRatingOperation,
) -> RespResult<ConsultantRatingResult> {
    // consultant_rating_idが正の整数であることをチェック
    // ratingの範囲チェック
    // consultant_rating_idでconsultant_ratingを取得
    // consultant_ratingのユーザーとaccount_idが一致していることを確認する
    // consultant_ratingにある相談時間とcurrent_date_timeを用いて評価を実施可能かチェックする
    // 身分証チェック
    // ユーザーのDisabledチェック
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
