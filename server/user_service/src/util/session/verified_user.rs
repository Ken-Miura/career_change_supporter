// Copyright 2023 Ken Miura

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use common::{AppState, ErrResp};
use serde::Deserialize;

use super::get_verified_user_account_id_from_request_parts;

/// 身分証確認済みであるユーザーの識別子を保持する構造体
///
/// ハンドラ関数内で身分証確認済みであるユーザーの情報にアクセスしたい場合、原則としてこの型をパラメータとして受け付ける。
/// このパラメータに含む識別子を用いて、データベースから身分証確認済みであるユーザー情報を取得できる。
/// [super::user::User]の確認に加えて、下記の確認を実施する。
/// <ul>
///   <li>身分証を確認済みであること</li>
/// </ul>
#[derive(Deserialize, Clone, Debug)]
pub(crate) struct VerifiedUser {
    pub(crate) account_id: i64,
}

#[async_trait]
impl<S> FromRequestParts<S> for VerifiedUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ErrResp;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user_account_id = get_verified_user_account_id_from_request_parts(parts, state).await?;
        Ok(VerifiedUser {
            account_id: user_account_id,
        })
    }
}
