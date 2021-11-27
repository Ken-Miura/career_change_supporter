// Copyright 2021 Ken Miura

use axum::{http::StatusCode, Json};
use common::{DatabaseConnection, RespResult};
use serde::Serialize;

use crate::util::session::User;

pub(crate) async fn get_profile(
    User { account_id }: User,
    DatabaseConnection(conn): DatabaseConnection,
) -> RespResult<ProfileResult> {
    tracing::info!("id: {}", account_id);
    let profile_result = ProfileResult {
        email_address: "test@test.com".to_string(),
        identity: None,
        careers: None,
        fee_per_hour_in_yen: None,
        bank_account: None,
        profit: None,
    };
    Ok((StatusCode::OK, Json(profile_result)))
}

#[derive(Serialize, Debug)]
pub(crate) struct ProfileResult {
    email_address: String,
    identity: Option<Identity>,
    careers: Option<Vec<Career>>,
    fee_per_hour_in_yen: Option<i32>,
    bank_account: Option<BankAccount>,
    profit: Option<u32>, // プラットフォーム利用の取り分は引く。振込手数料は引かない。
}

#[derive(Serialize, Debug)]
pub(crate) struct Identity {
    pub last_name: String,
    pub first_name: String,
    pub last_name_furigana: String,
    pub first_name_furigana: String,
    pub sex: String,
    pub date_of_birth: Ymd,
    pub prefecture: String,
    pub city: String,
    pub address_line1: String,
    pub address_line2: Option<String>,
}

#[derive(Serialize, Debug)]
pub(crate) struct Ymd {
    pub year: i32,
    pub month: i32,
    pub day: i32,
}

#[derive(Serialize, Debug)]
pub(crate) struct Career {
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

#[derive(Serialize, Debug)]
pub(crate) struct BankAccount {
    pub bank_code: String, // 明確な仕様は見つからなかったが数字4桁が最も普及しているように見える
    pub branch_code: String,
    pub account_type: String,
    pub account_number: String,
    pub account_holder_name: String,
}
