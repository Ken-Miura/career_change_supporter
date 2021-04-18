// Copyright 2021 Ken Miura

use derive_more::Display;

#[derive(Display, Debug)]
pub(crate) enum Error {
    #[display(fmt = "failed to get connection: {}", _0)]
    R2d2Err(r2d2::Error),

    #[display(fmt = "diesel result error: {}", _0)]
    DieselResultErr(diesel::result::Error),

    #[display(fmt = "{}", _0)]
    AccountDuplicate(AccountDuplicate),

    #[display(fmt = "actix_web::error::BlockingError::Canceled")]
    BlockingErrCanceled,

    #[display(fmt = "{}", _0)]
    FailedToUpdateAccount(FailedToUpdateAccount),

    #[display(fmt = "actix web error: {}", _0)]
    ActixWebErr(String),

    #[display(fmt = "lettre email error: {}", _0)]
    LettreEmailErr(lettre_email::error::Error),

    #[display(fmt = "lettre smtp error: {}", _0)]
    LettreSmtpErr(lettre::smtp::error::Error),

    #[display(fmt = "{}", _0)]
    TemporaryAccountIdDuplicate(TemporaryAccountIdDuplicate),
}

// NOTE: Use negative value because positive value is used for handled error
pub(super) const INTERNAL_SERVER_ERROR: i32 = -1;

#[derive(Display, Debug)]
#[display(fmt = "account duplicate (email_address: {})", email_address)]
pub(crate) struct AccountDuplicate {
    email_address: String,
}

impl AccountDuplicate {
    pub(crate) fn new(email_address: String) -> Self {
        AccountDuplicate { email_address }
    }
}

#[derive(Display, Debug)]
#[display(fmt = "failed to update account (primary_key: {})", primary_key)]
pub(crate) struct FailedToUpdateAccount {
    primary_key: i32,
}

impl FailedToUpdateAccount {
    pub(crate) fn new(primary_key: i32) -> Self {
        FailedToUpdateAccount { primary_key }
    }
}

#[derive(Display, Debug)]
#[display(fmt = "temporary account id duplicate (temp_acc_id: {})", temp_acc_id)]
pub(crate) struct TemporaryAccountIdDuplicate {
    temp_acc_id: String,
}

impl TemporaryAccountIdDuplicate {
    pub(crate) fn new(temp_acc_id: String) -> Self {
        TemporaryAccountIdDuplicate { temp_acc_id }
    }
}
