// Copyright 2023 Ken Miura

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use common::{AppState, ErrResp};
use serde::Deserialize;

use crate::util::user_info::UserInfo;

use super::get_user_info_from_request_parts;

/// （利用規約に同意済みである）ユーザーの情報を保持する構造体
///
/// ハンドラ関数内で利用規約に同意済みであるユーザーの情報にアクセスしたい場合、原則としてこの型をパラメータとして受け付ける。
/// ユーザーの情報内に含む識別子を用いて、データベースからユーザー情報を取得できる。
/// [super::agreement_unchecked_user::AgreementUncheckedUser]の確認に加えて、下記の確認を実施する。
/// <ul>
///   <li>利用規約に同意済みである確認</li>
/// </ul>
#[derive(Deserialize, Clone, Debug)]
pub(crate) struct User {
    pub(crate) user_info: UserInfo,
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ErrResp;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user_info = get_user_info_from_request_parts(parts, state).await?;
        Ok(User { user_info })
    }
}
