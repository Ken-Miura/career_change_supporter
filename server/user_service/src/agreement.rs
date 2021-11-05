// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use axum::{body::Body, http::Request, http::StatusCode};
use common::ErrResp;
use headers::{Cookie, HeaderMapExt};

use crate::util::{session::get_user_by_cookie, unexpected_err_resp};

/// ユーザーが利用規約に同意したことを記録する
pub(crate) async fn post_agreement(req: Request<Body>) -> Result<StatusCode, ErrResp> {
    let headers = req.headers();
    let option_cookie = headers.typed_try_get::<Cookie>().map_err(|e| {
        tracing::error!("failed to get cookie: {}", e);
        unexpected_err_resp()
    })?;
    let extentions = req.extensions();
    let store = extentions.get::<RedisSessionStore>().ok_or_else(|| {
        tracing::error!("failed to get session store");
        unexpected_err_resp()
    })?;
    let _user = get_user_by_cookie(option_cookie, store).await?;
    todo!("user account idと利用規約バージョンをもとに利用規約に同意した時刻を入れる (post_agreement_internal = テスト対象)")
}
