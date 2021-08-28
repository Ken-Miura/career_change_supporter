// Copyright 2021 Ken Miura

use common::{DatabaseConnection, ValidCred};

/// 一時アカウントを作成する。<br>
/// <br>
///
/// # Errors
///
async fn _post_temp_accounts(
    ValidCred(_cred): ValidCred,
    DatabaseConnection(_conn): DatabaseConnection,
) {
}
