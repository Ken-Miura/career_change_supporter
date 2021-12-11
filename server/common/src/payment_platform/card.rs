// Copyright 2021 Ken Miura

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// [Charge] 内で利用される型
/// 支払いに利用されたクレジットカードを示す
/// <https://pay.jp/docs/api/#card%E3%82%AA%E3%83%96%E3%82%B8%E3%82%A7%E3%82%AF%E3%83%88>
#[derive(Serialize, Deserialize, Debug)]
pub struct Card {
    pub object: String,
    pub id: String,
    pub created: i64,
    pub name: Option<String>,
    pub last4: String,
    pub exp_month: i32,
    pub exp_year: i32,
    pub brand: String,
    pub cvc_check: String,
    pub fingerprint: String,
    pub address_state: Option<String>,
    pub address_city: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub country: Option<String>,
    pub address_zip: Option<String>,
    pub address_zip_check: String,
    /// 一つのオブジェクトには最大20キーまで保存でき、キーは40文字まで、バリューは500文字までの文字列
    /// <https://pay.jp/docs/api/?java#metadata>
    pub metadata: Option<HashMap<String, String>>,
}
