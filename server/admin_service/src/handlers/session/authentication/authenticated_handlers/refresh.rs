// Copyright 2021 Ken Miura

use common::ErrResp;

use axum::http::StatusCode;
use tracing::info;

use super::admin::Admin;

/// 下記の処理を順に行う<br>
///   - ログインセッションが存在しているか確認する
///   - ログインセッションが存在している場合、有効期限を[LOGIN_SESSION_EXPIRY]だけ延長する
///
/// すべての処理が完了した場合、ステータスコード200を返す<br>
/// <br>
/// # Errors
/// - ログインセッションが存在しない場合、ステータスコード401、エラーコード[crate::err::Code::Unauthorized]を返す
pub(crate) async fn get_refresh(Admin { admin_info }: Admin) -> Result<StatusCode, ErrResp> {
    // NOTE:
    // Admin構造体を受け取る際のリクエストのプリプロセスで認証 (ログインセッションの延長) を実施済
    // そのため、ここまで到達した場合、OKを返すのみで良い
    info!("refresh (admin account id: {})", admin_info.account_id);
    Ok(StatusCode::INTERNAL_SERVER_ERROR) // TODO: アップデートの動作確認のために一時的に変更。動作確認後すぐにもとに戻す。
                                          // Ok(StatusCode::OK)
}
