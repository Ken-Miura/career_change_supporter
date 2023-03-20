// Copyright 2023 Ken Miura

use axum::extract::State;
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

use crate::util::session::User;

pub(crate) async fn post_temp_mfa_secret(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
) -> RespResult<PostTempMfaSecretResult> {
    // 既にMFAが有効かどうか確認
    // temp_mfa_secretが最大数作られていないか確認
    // temp_mfa_secretを作成
    todo!()
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct PostTempMfaSecretResult {
    // QRコード
    base64_encoded_image: String,
    // QRコードを読み込めない場合に使うシークレットキー
    base32_encoded_secret: String,
}
