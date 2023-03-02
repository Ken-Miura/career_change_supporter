// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset};
use common::RespResult;
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
trait ConsultantRatingOperation {}

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
