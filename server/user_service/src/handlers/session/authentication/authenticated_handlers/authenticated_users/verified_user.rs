// Copyright 2023 Ken Miura

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use common::{AppState, ErrResp};
use serde::Deserialize;

use crate::util::user_info::UserInfo;

use super::extract_singed_jar_from_request_parts;
use super::{super::super::super::SESSION_ID_COOKIE_NAME, get_verified_user_info_from_cookie};

/// 身分証確認済みであるユーザーの情報を保持する構造体
///
/// ハンドラ関数内で身分証確認済みであるユーザーの情報にアクセスしたい場合、原則としてこの型をパラメータとして受け付ける。
/// ユーザーの情報内に含む識別子を用いて、データベースからユーザー情報を取得できる。
/// [super::user::User]の確認に加えて、下記の確認を実施する。
/// <ul>
///   <li>身分証を確認済みであること</li>
/// </ul>
#[derive(Deserialize, Clone, Debug)]
pub(crate) struct VerifiedUser {
    pub(in crate::handlers::session::authentication::authenticated_handlers) user_info: UserInfo,
}

#[async_trait]
impl<S> FromRequestParts<S> for VerifiedUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ErrResp;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user_info = get_verified_user_info_from_request_parts(parts, state).await?;
        Ok(VerifiedUser { user_info })
    }
}

async fn get_verified_user_info_from_request_parts<S>(
    parts: &mut Parts,
    state: &S,
) -> Result<UserInfo, ErrResp>
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    let signed_cookies = extract_singed_jar_from_request_parts(parts, state).await?;
    let option_cookie = signed_cookies.get(SESSION_ID_COOKIE_NAME);

    let app_state = AppState::from_ref(state);
    let store = app_state.store;
    let pool = app_state.pool;

    let user_info = get_verified_user_info_from_cookie(option_cookie, &store, &pool).await?;

    Ok(user_info)
}
