// Copyright 2021 Ken Miura

use headers::Cookie;

use crate::util::ROOT_PATH;

const COOKIE_NAME: &str = "session_id";
pub(crate) const KEY_TO_USER_ACCOUNT_ID: &str = "user_account_id";

pub(crate) fn create_cookie_format(session_id_value: &str) -> String {
    format!(
        // TODO: SSLのセットアップが完了し次第、Secureを追加する
        //"{}={}; SameSite=Strict; Path={}/; Secure; HttpOnly",
        "{}={}; SameSite=Strict; Path={}/; HttpOnly",
        COOKIE_NAME,
        session_id_value,
        ROOT_PATH
    )
}

pub(crate) fn create_expired_cookie_format(session_id_value: &str) -> String {
    format!(
        // TODO: SSLのセットアップが完了し次第、Secureを追加する
        //"{}={}; SameSite=Strict; Path={}/; Max-Age=-1; Secure; HttpOnly",
        "{}={}; SameSite=Strict; Path={}/; Max-Age=-1; HttpOnly",
        COOKIE_NAME,
        session_id_value,
        ROOT_PATH
    )
}

pub(crate) fn extract_session_id(option_cookie: Option<Cookie>) -> Option<String> {
    let cookie = match option_cookie {
        Some(c) => c,
        None => {
            tracing::debug!("no cookie");
            return None;
        }
    };
    match cookie.get(COOKIE_NAME) {
        Some(value) => Some(value.to_string()),
        None => {
            tracing::debug!("no {} in cookie", COOKIE_NAME);
            None
        }
    }
}

/// テストコードで共通で使うコードをまとめるモジュール
#[cfg(test)]
pub(crate) mod tests {
    use headers::HeaderValue;

    use super::COOKIE_NAME;

    pub(crate) fn extract_session_id_value(header_value: &HeaderValue) -> String {
        let set_cookie = header_value.to_str().expect("failed to get value");
        let cookie_name = set_cookie
            .split(";")
            .find(|s| s.contains(COOKIE_NAME))
            .expect("failed to get session")
            .trim()
            .split_once("=")
            .expect("failed to get value");
        cookie_name.1.to_string()
    }

    pub(crate) fn extract_cookie_max_age_value(header_value: &HeaderValue) -> String {
        let set_cookie = header_value.to_str().expect("failed to get value");
        let cookie_max_age = set_cookie
            .split(";")
            .find(|s| s.contains("Max-Age"))
            .expect("failed to get Max-Age")
            .trim()
            .split_once("=")
            .expect("failed to get value");
        cookie_max_age.1.to_string()
    }
}
