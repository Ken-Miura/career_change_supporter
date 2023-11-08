// Copyright 2021 Ken Miura

use common::ErrResp;

use axum::http::StatusCode;
use tracing::info;

use crate::handlers::session::authentication::authenticated_handlers::authenticated_users::user::User;

/// 下記の処理を順に行う<br>
///   - ログインセッションが存在しているか確認する
///   - ログインセッションが存在している場合、有効期限を[LOGIN_SESSION_EXPIRY]だけ延長する
///   - 利用規約に同意しているかどうかを確認する
///
/// すべての処理が完了した場合、ステータスコード200を返す<br>
/// <br>
/// # Errors
/// - ログインセッションが存在しない場合、ステータスコード401、エラーコード[crate::err::Code::Unauthorized]を返す
/// - 利用規約にまだ同意していない場合、ステータスコード400、エラーコード[crate::err::Code::NotTermsOfUseAgreedYet]を返す
pub(crate) async fn get_refresh(User { user_info }: User) -> Result<StatusCode, ErrResp> {
    // NOTE:
    // User構造体を受け取る際のリクエストのプリプロセスで認証 (ログインセッションの延長) と利用規約の同意を実施済
    // そのため、ここまで到達した場合、OKを返すのみで良い
    info!("refresh (account id: {})", user_info.account_id);
    // Ok(StatusCode::OK)
    Ok(StatusCode::INTERNAL_SERVER_ERROR)
}
