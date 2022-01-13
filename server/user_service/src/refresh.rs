// Copyright 2021 Ken Miura

use common::ErrResp;

use axum::http::StatusCode;

use crate::util::session::User;

/// 下記の処理を順に行う<br>
///   - ログインセッションが存在しているか確認する
///   - ログインセッションが存在している場合、有効期限を[crate::util::session::LOGIN_SESSION_EXPIRY]だけ延長する
///   - 利用規約に同意しているかどうかを確認する
///
/// すべての処理が完了した場合、ステータスコード200を返す<br>
/// <br>
/// # Errors
/// - ログインセッションが存在しない場合、ステータスコード401、エラーコード[crate::err_code::UNAUTHORIZED]を返す
/// - 利用規約にまだ同意していない場合、ステータスコード400、エラーコード[crate::err_code::NOT_TERMS_OF_USE_AGREED_YET]を返す
pub(crate) async fn get_refresh(User { account_id }: User) -> Result<StatusCode, ErrResp> {
    // NOTE:
    // User構造体を受け取る際のリクエストのプリプロセスで認証 (ログインセッションの延長) と利用規約の同意を実施済
    // そのため、ここまで到達した場合、OKを返すのみで良い
    tracing::debug!("refresh (id: {})", account_id);
    Ok(StatusCode::OK)
}
