// Copyright 2021 Ken Miura

use crate::common;
use crate::common::error::handled;
use crate::common::error::unexpected;
use crate::common::util;
use serde::Deserialize;

// TODO: パスワードのストレッチングが2^BCRYPT_COST回実行される。リリース前に値を調整する
const BCRYPT_COST: u32 = 7;

#[derive(Deserialize)]
pub(crate) struct Credential {
    pub(crate) email_address: String,
    pub(crate) password: String,
}

impl Credential {
    pub(crate) fn validate(&self) -> Result<(), handled::Error> {
        let _ = util::validate_email_address(&self.email_address)?;
        let _ = util::validate_password(&self.password)?;
        Ok(())
    }
}

pub(crate) fn hash_password(password: &str) -> Result<Vec<u8>, unexpected::Error> {
    let result = bcrypt::hash(password, BCRYPT_COST);
    match result {
        Ok(hashed_pwd_str) => {
            let binary = hashed_pwd_str.as_bytes();
            Ok(Vec::from(binary))
        }
        Err(e) => Err(unexpected::Error::BcryptErr(e)),
    }
}

pub(crate) fn verify_password(
    password: &str,
    hashed_password: &[u8],
) -> Result<(), common::error::Error> {
    let pwd_str = String::from_utf8(Vec::from(hashed_password))?;
    let verified = bcrypt::verify(password, &pwd_str)?;
    if !verified {
        let e = handled::Error::PasswordNotMatch(handled::PasswordNotMatch::new());
        return Err(common::error::Error::Handled(e));
    }
    Ok(())
}
