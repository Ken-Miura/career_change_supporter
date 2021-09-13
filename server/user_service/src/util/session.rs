// Copyright 2021 Ken Miura

use crate::util::ROOT_PATH;

pub(crate) const COOKIE_NAME: &str = "session";
pub(crate) const KEY_TO_USER_ACCOUNT_ID: &str = "user_account_id";

pub(crate) fn create_cookie_format(cookie_name_value: &str) -> String {
    format!(
        // TODO: SSLのセットアップが完了し次第、Secureを追加する
        //"{}={}; SameSite=Strict; Path={}/; Secure; HttpOnly",
        "{}={}; SameSite=Strict; Path={}/; HttpOnly",
        COOKIE_NAME,
        cookie_name_value,
        ROOT_PATH
    )
}

pub(crate) fn create_expired_cookie_format(cookie_name_value: &str) -> String {
    format!(
        // TODO: SSLのセットアップが完了し次第、Secureを追加する
        //"{}={}; SameSite=Strict; Path={}/; Max-Age=-1; Secure; HttpOnly",
        "{}={}; SameSite=Strict; Path={}/; Max-Age=-1; HttpOnly",
        COOKIE_NAME,
        cookie_name_value,
        ROOT_PATH
    )
}
