// Copyright 2021 Ken Miura

use crate::util::session::User;

pub(crate) async fn get_profile(User { account_id }: User) {
    tracing::info!("id: {}", account_id);
}
