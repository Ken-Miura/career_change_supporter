// Copyright 2021 Ken Miura

use common::ErrResp;

use axum::http::StatusCode;

use crate::util::session::User;

/// 利用規約に同意しているかどうかを確認する<br>
/// 利用規約に同意している場合、ステータスコード200を返す。<br>
/// <br>
/// # Errors
/// ログインしていない場合、ステータスコード401、エラーコード[crate::err_code::UNAUTHORIZED]を返す。<br>
/// 利用規約にまだ同意していない場合、ステータスコード400、エラーコード[crate::err_code::NOT_TERMS_OF_USE_AGREED_YET]を返す。<br>
pub(crate) async fn get_agreement_status(User { account_id }: User) -> Result<StatusCode, ErrResp> {
    // NOTE:
    // User構造体を受け取る際のリクエストのプリプロセスで認証と利用規約の同意を実施済
    // そのため、ここまで到達した場合、OKを返すのみで良い
    tracing::debug!(
        "user account id ({}) has already agreed terms of use",
        account_id
    );
    Ok(StatusCode::OK)
}
