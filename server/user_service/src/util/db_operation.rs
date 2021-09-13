// Copyright 2021 Ken Miura

use common::{
    schema::ccs_schema::user_account::dsl::{email_address, user_account},
    ErrResp,
};
use diesel::{
    dsl::count_star,
    query_dsl::methods::{FilterDsl, SelectDsl},
    r2d2::{ConnectionManager, PooledConnection},
    ExpressionMethods, PgConnection, RunQueryDsl,
};

use crate::util::unexpected_err_resp;

/// ユーザーが既に存在するか確認する
pub(crate) fn user_exists(
    conn: &PooledConnection<ConnectionManager<PgConnection>>,
    email_addr: &str,
) -> Result<bool, ErrResp> {
    let cnt = user_account
        .filter(email_address.eq(email_addr))
        .select(count_star())
        .get_result::<i64>(conn)
        .map_err(|e| {
            tracing::error!("user ({}) already exists: {}", email_addr, e);
            unexpected_err_resp()
        })?;
    Ok(cnt != 0)
}
