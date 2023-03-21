// Copyright 2023 Ken Miura

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use common::{AppState, ErrResp};
use serde::Deserialize;

use super::get_agreement_unchecked_user_account_id_from_request_parts;

/// 利用規約に同意したかどうか確認していないユーザーの識別子を保持する構造体
///
/// ハンドラ関数内で利用規約に同意したかどうか確認していないユーザーの情報にアクセスしたい場合、原則としてこの型をパラメータとして受け付ける。
/// このパラメータに含む識別子を用いて、データベースからユーザー情報を取得できる。
/// この型をパラメータとして受け付けると、ハンドラ関数の処理に入る前に下記の前処理を実施する。
/// <ul>
///   <li>ログインセッションが有効であることを確認</li>
///   <li>アカウントが無効でないこと</li>
/// </ul>
#[derive(Deserialize, Clone, Debug)]
pub(crate) struct AgreementUncheckedUser {
    pub(crate) account_id: i64,
}

#[async_trait]
impl<S> FromRequestParts<S> for AgreementUncheckedUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ErrResp;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user_account_id =
            get_agreement_unchecked_user_account_id_from_request_parts(parts, state).await?;
        Ok(AgreementUncheckedUser {
            account_id: user_account_id,
        })
    }
}
