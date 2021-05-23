// Copyright 2021 Ken Miura

use derive_more::Display;

#[derive(Display, Debug)]
pub(crate) enum Error {
    #[display(fmt = "failed to get connection: {}", _0)]
    R2d2Err(r2d2::Error),

    #[display(fmt = "diesel result error: {}", _0)]
    DieselResultErr(diesel::result::Error),

    #[display(fmt = "{}", _0)]
    UserAccountDuplicate(UserAccountDuplicate),

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

    #[display(fmt = "bcrypt error: {}", _0)]
    BcryptErr(bcrypt::BcryptError),

    #[display(fmt = "from utf-8 error: {}", _0)]
    FromUtf8Err(std::string::FromUtf8Error),

    #[display(fmt = "{}", _0)]
    AdvisorAccountDuplicate(AdvisorAccountDuplicate),

    #[display(fmt = "{}", _0)]
    RegistrationRequestIdDuplicate(RegistrationRequestIdDuplicate),
}

// NOTE: Use negative value because positive value is used for handled error
pub(super) const INTERNAL_SERVER_ERROR: i32 = -1;

#[derive(Display, Debug)]
#[display(fmt = "user account duplicate (email_address: {})", email_address)]
pub(crate) struct UserAccountDuplicate {
    email_address: String,
}

impl UserAccountDuplicate {
    pub(crate) fn new(email_address: String) -> Self {
        UserAccountDuplicate { email_address }
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

#[derive(Display, Debug)]
#[display(fmt = "advisor account duplicate (email_address: {})", email_address)]
pub(crate) struct AdvisorAccountDuplicate {
    email_address: String,
}

impl AdvisorAccountDuplicate {
    pub(crate) fn new(email_address: String) -> Self {
        AdvisorAccountDuplicate { email_address }
    }
}

#[derive(Display, Debug)]
#[display(fmt = "registration request id duplicate (request_id: {})", request_id)]
pub(crate) struct RegistrationRequestIdDuplicate {
    request_id: String,
}

impl RegistrationRequestIdDuplicate {
    pub(crate) fn new(request_id: String) -> Self {
        RegistrationRequestIdDuplicate { request_id }
    }
}
