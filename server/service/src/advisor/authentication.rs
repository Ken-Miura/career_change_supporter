// Copyright 2021 Ken Miura

use crate::common::error;
use crate::common::error::handled;
use actix_session::Session;
use actix_web::{get, HttpResponse};

// TODO: アドバイザー向けに実装を行う
#[get("/session-state")]
async fn session_state(_session: Session) -> Result<HttpResponse, error::Error> {
    let e = error::Error::Handled(handled::Error::NoSessionFound(
        handled::NoSessionFound::new(),
    ));
    log::error!("failed to get session state {}", e);
    Err(e)
}
