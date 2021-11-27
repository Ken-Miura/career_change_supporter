// Copyright 2021 Ken Miura

use common::DatabaseConnection;

use crate::util::session::User;

pub(crate) async fn get_profile(
    User { account_id }: User,
    DatabaseConnection(conn): DatabaseConnection,
) {
    tracing::info!("id: {}", account_id);
}
