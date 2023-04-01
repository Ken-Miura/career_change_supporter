// Copyright 2023 Ken Miura

use serde::Serialize;

const LOGIN_STATUS_FINISH: &str = "Finish";
const LOGIN_STATUS_NEED_MORE_VERIFICATION: &str = "NeedMoreVerification";

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) enum LoginStatus {
    Finish,
    NeedMoreVerification,
}

impl From<String> for LoginStatus {
    fn from(ls: String) -> Self {
        if ls == LOGIN_STATUS_FINISH {
            LoginStatus::Finish
        } else if ls == LOGIN_STATUS_NEED_MORE_VERIFICATION {
            LoginStatus::NeedMoreVerification
        } else {
            panic!("never reach here!")
        }
    }
}

impl From<LoginStatus> for String {
    fn from(ls: LoginStatus) -> Self {
        match ls {
            LoginStatus::Finish => LOGIN_STATUS_FINISH.to_string(),
            LoginStatus::NeedMoreVerification => LOGIN_STATUS_NEED_MORE_VERIFICATION.to_string(),
        }
    }
}
