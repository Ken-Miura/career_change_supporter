// Copyright 2021 Ken Miura

pub mod validator;

use std::env::var;

use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

/// 入力された環境変数が定義されているかチェックする<br>
/// <br>
/// 入力された環境変数がすべて定義されている場合、Okを返す。<br>
/// 入力された環境変数の内、どれか一つでも定義されていなければ、Errを返す。
/// Errには定義されていない環境変数を含む。
pub fn check_env_vars(env_vars: Vec<String>) -> Result<(), Vec<String>> {
    let not_found_vars = env_vars
        .iter()
        .map(|env_var| (env_var.clone(), var(env_var)))
        .filter(|env_var_and_result| env_var_and_result.1.is_err())
        .map(|env_var_and_err| env_var_and_err.0)
        .collect::<Vec<String>>();
    if !not_found_vars.is_empty() {
        return Err(not_found_vars);
    }
    Ok(())
}

/// SameSiteがStrict、Secure、HttpOnlyのセッションCookie（ブラウザが閉じられたら消えるCookie）を返す。
pub fn create_session_cookie<'a>(name: String, value: String, path: String) -> Cookie<'a> {
    Cookie::build(name, value)
        .same_site(SameSite::Strict)
        .path(path)
        .secure(true)
        .http_only(true)
        .finish()
}

/// タイムゾーンを含まない日付（西暦、月（1-12）、日付（1-31））
///
/// [chrono::naive::NaiveDate]をそのままSerializeしてJavascriptに渡すと
/// YYYY-MM-DDというJavascriptのDateオブジェクトとしてそのまま扱えない形になるため、
/// 必要に応じてこちらを利用する。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Ymd {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

/// ユーザーの身元情報
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    pub last_name: String,
    pub first_name: String,
    pub last_name_furigana: String,
    pub first_name_furigana: String,
    pub date_of_birth: Ymd,
    pub prefecture: String,
    pub city: String,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub telephone_number: String,
}

/// 職務経歴情報
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Career {
    pub company_name: String,
    pub department_name: Option<String>,
    pub office: Option<String>,
    pub career_start_date: Ymd,
    pub career_end_date: Option<Ymd>,
    pub contract_type: String,
    pub profession: Option<String>,
    pub annual_income_in_man_yen: Option<i32>,
    pub is_manager: bool,
    pub position_name: Option<String>,
    pub is_new_graduate: bool,
    pub note: Option<String>,
}

/// 口座情報
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BankAccount {
    pub bank_code: String,
    pub branch_code: String,
    pub account_type: String,
    pub account_number: String,
    pub account_holder_name: String,
}

/// メンテナンス情報
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Maintenance {
    pub maintenance_id: i64,
    pub maintenance_start_at_in_jst: DateTime<FixedOffset>,
    pub maintenance_end_at_in_jst: DateTime<FixedOffset>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_generated_by_create_session_cookie_has_same_site_strict_secure_and_http_only() {
        let name = String::from("cookie-name");
        let value = String::from("cookie-value");
        let path = String::from("/path");

        let cookie = create_session_cookie(name.clone(), value.clone(), path.clone());

        assert_eq!(name, cookie.name());
        assert_eq!(value, cookie.value());
        assert_eq!(path, cookie.path().expect("failed to get Ok"));
        assert!(cookie.secure().expect("failed to get Ok"));
        assert!(cookie.http_only().expect("failed to get Ok"));
        assert_eq!(
            SameSite::Strict,
            cookie.same_site().expect("failed to get Ok")
        );
        // セッションCookie (ブラウザを閉じたら消えるCookie) = 期限が記載されていないCookie
        assert_eq!(None, cookie.expires());
        assert_eq!(None, cookie.max_age());
    }
}
