// Copyright 2021 Ken Miura

pub(crate) mod session;
pub(crate) mod terms_of_use;
pub(crate) mod validator;

use std::env::var;

use chrono::FixedOffset;
use common::payment_platform::AccessInfo;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub(crate) const WEB_SITE_NAME: &str = "就職先・転職先を見極めるためのサイト";

pub(crate) const ROOT_PATH: &str = "/api";

pub(crate) const KEY_TO_PAYMENT_PLATFORM_API_URL: &str = "PAYMENT_PLATFORM_API_URL";
pub(crate) const KEY_TO_PAYMENT_PLATFORM_API_USERNAME: &str = "PAYMENT_PLATFORM_API_USERNAME";
pub(crate) const KEY_TO_PAYMENT_PLATFORM_API_PASSWORD: &str = "PAYMENT_PLATFORM_API_PASSWORD";
/// PAY.JPにアクセスするための情報を保持する変数
pub(crate) static ACCESS_INFO: Lazy<AccessInfo> = Lazy::new(|| {
    let url_without_path = var(KEY_TO_PAYMENT_PLATFORM_API_URL).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_PAYMENT_PLATFORM_API_URL
        )
    });
    let username = var(KEY_TO_PAYMENT_PLATFORM_API_USERNAME).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_PAYMENT_PLATFORM_API_USERNAME
        )
    });
    let password = var(KEY_TO_PAYMENT_PLATFORM_API_PASSWORD).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_PAYMENT_PLATFORM_API_PASSWORD
        )
    });
    let access_info = AccessInfo::new(url_without_path, username, password);
    access_info.expect("failed to get Ok")
});

/// UTCにおける日本のタイムゾーン（正確には、UTCで日本時間を表すためのオフセットだが、タイムゾーンと同等の意味で利用）
/// [chrono::DateTime] で日本時間を扱う際に利用する。
pub(crate) static JAPANESE_TIME_ZONE: Lazy<FixedOffset> = Lazy::new(|| FixedOffset::east(9 * 3600));

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct Identity {
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct Ymd {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct Career {
    pub id: i32,
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct BankAccount {
    pub bank_code: String, // 明確な仕様は見つからなかったが数字4桁が最も普及しているように見える
    pub branch_code: String,
    pub account_type: String,
    pub account_number: String,
    pub account_holder_name: String,
}

/// テストコードで共通で使うコードをまとめるモジュール
#[cfg(test)]
pub(crate) mod tests {
    use common::{smtp::SendMail, ErrResp};

    pub(crate) struct SendMailMock {
        to: String,
        from: String,
        subject: String,
        text: String,
    }

    impl SendMailMock {
        pub(crate) fn new(to: String, from: String, subject: String, text: String) -> Self {
            Self {
                to,
                from,
                subject,
                text,
            }
        }
    }

    impl SendMail for SendMailMock {
        fn send_mail(
            &self,
            to: &str,
            from: &str,
            subject: &str,
            text: &str,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.to, to);
            assert_eq!(self.from, from);
            assert_eq!(self.subject, subject);
            assert_eq!(self.text, text);
            Ok(())
        }
    }
}
