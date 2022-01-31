// Copyright 2021 Ken Miura

use std::{collections::HashSet, error::Error, fmt::Display};

use chrono::{Datelike, NaiveDate};
use once_cell::sync::Lazy;
use regex::Regex;

use super::{Identity, Ymd};

const LAST_NAME_MIN_LENGTH: usize = 1;
const LAST_NAME_MAX_LENGTH: usize = 128;
const FIRST_NAME_MIN_LENGTH: usize = 1;
const FIRST_NAME_MAX_LENGTH: usize = 128;
const LAST_NAME_FURIGANA_MIN_LENGTH: usize = 1;
const LAST_NAME_FURIGANA_MAX_LENGTH: usize = 128;
const FIRST_NAME_FURIGANA_MIN_LENGTH: usize = 1;
const FIRST_NAME_FURIGANA_MAX_LENGTH: usize = 128;
const MIN_AGE_REQUIREMENT: i32 = 18;
const CITY_MIN_LENGTH: usize = 1;
const CITY_MAX_LENGTH: usize = 32;
const ADDRESS_LINE1_MIN_LENGTH: usize = 1;
const ADDRESS_LINE1_MAX_LENGTH: usize = 128;
const ADDRESS_LINE2_MIN_LENGTH: usize = 1;
const ADDRESS_LINE2_MAX_LENGTH: usize = 128;

static PREFECTURE_SET: Lazy<HashSet<String>> = Lazy::new(|| {
    let mut set: HashSet<String> = HashSet::with_capacity(47);
    set.insert("北海道".to_string());
    set.insert("青森県".to_string());
    set.insert("秋田県".to_string());
    set.insert("岩手県".to_string());
    set.insert("山形県".to_string());
    set.insert("宮城県".to_string());
    set.insert("新潟県".to_string());
    set.insert("福島県".to_string());
    set.insert("群馬県".to_string());
    set.insert("栃木県".to_string());
    set.insert("茨城県".to_string());
    set.insert("埼玉県".to_string());
    set.insert("東京都".to_string());
    set.insert("千葉県".to_string());
    set.insert("神奈川県".to_string());
    set.insert("石川県".to_string());
    set.insert("富山県".to_string());
    set.insert("福井県".to_string());
    set.insert("岐阜県".to_string());
    set.insert("長野県".to_string());
    set.insert("愛知県".to_string());
    set.insert("静岡県".to_string());
    set.insert("山梨県".to_string());
    set.insert("兵庫県".to_string());
    set.insert("京都府".to_string());
    set.insert("滋賀県".to_string());
    set.insert("大阪府".to_string());
    set.insert("奈良県".to_string());
    set.insert("三重県".to_string());
    set.insert("和歌山県".to_string());
    set.insert("香川県".to_string());
    set.insert("愛媛県".to_string());
    set.insert("徳島県".to_string());
    set.insert("高知県".to_string());
    set.insert("山口県".to_string());
    set.insert("島根県".to_string());
    set.insert("鳥取県".to_string());
    set.insert("広島県".to_string());
    set.insert("岡山県".to_string());
    set.insert("福岡県".to_string());
    set.insert("佐賀県".to_string());
    set.insert("長崎県".to_string());
    set.insert("大分県".to_string());
    set.insert("熊本県".to_string());
    set.insert("宮崎県".to_string());
    set.insert("鹿児島県".to_string());
    set.insert("沖縄県".to_string());
    set
});

/// 全角カタカナのみのケース
// 参考: https://qiita.com/nasuB7373/items/17adc4b808a8bd39624d
// \p{katakana}は、半角カタカナも含むので使わない
const ZENKAKU_KATAKANA_REGEXP: &str = r"^[ァ-ヴー]+$";
static ZENKAKU_KATAKANA_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(ZENKAKU_KATAKANA_REGEXP).expect("failed to compile zenkaku katakana regexp")
});

/// 国内の電話番号を示す正規表現 (10桁から13桁の数字のみにケース)
const TEL_NUM_REGEXP: &str = "^[0-9]{10,13}$";
static TEL_NUM_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(TEL_NUM_REGEXP).expect("failed to compile telephone number regexp"));

/// 記号 (ASCIIの0x21(!)から0x2f(/)、0x3a(:)から0x40(@)、0x5b([)から0x60(`)、0x7b({)から0x7e(~)) を一つ以上含むケース
const SYMBOL_CHAR_REGEXP: &str = r"[!-/:-@\[-`\{-~]+";
static SYMBOL_CHAR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(SYMBOL_CHAR_REGEXP).expect("failed to compile symbol char regexp"));

/// 数字を一つ以上含むケース
const NUM_CHAR_REGEXP: &str = r"[0-9]+";
static NUM_CHAR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(NUM_CHAR_REGEXP).expect("failed to compile num char regexp"));

/// 0x2d(-)以外の記号を一つ以上含むケース
const SYMBOL_CHAR_WITHOUT_HYPHEN_REGEXP: &str = r"[!-,\./:-@\[-`\{-~]+";
static SYMBOL_CHAR_WITHOUT_HYPHEN_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(SYMBOL_CHAR_WITHOUT_HYPHEN_REGEXP)
        .expect("failed to compile symbol char without hyphen regexp")
});

pub(crate) fn validate_identity(
    identity: &Identity,
    current_date: &NaiveDate,
) -> Result<(), IdentityValidationError> {
    let _ = validate_last_name(&identity.last_name)?;
    let _ = validate_first_name(&identity.first_name)?;
    let _ = validate_last_name_furigana(&identity.last_name_furigana)?;
    let _ = validate_first_name_furigana(&identity.first_name_furigana)?;
    let _ = validate_date_of_birth(&identity.date_of_birth, current_date)?;
    let _ = validate_prefecture(&identity.prefecture)?;
    let _ = validate_city(&identity.city)?;
    let _ = validate_address_line1(&identity.address_line1)?;
    if let Some(address_line2) = identity.address_line2.clone() {
        let _ = validate_address_line2(&address_line2)?;
    }
    let _ = validate_telephone_number(&identity.telephone_number)?;
    Ok(())
}

fn validate_last_name(last_name: &str) -> Result<(), IdentityValidationError> {
    let last_name_length = last_name.chars().count();
    if !(LAST_NAME_MIN_LENGTH..=LAST_NAME_MAX_LENGTH).contains(&last_name_length) {
        return Err(IdentityValidationError::InvalidLastNameLength {
            length: last_name_length,
            min_length: LAST_NAME_MIN_LENGTH,
            max_length: LAST_NAME_MAX_LENGTH,
        });
    }
    if has_control_char(last_name) {
        return Err(IdentityValidationError::IllegalCharInLastName(
            last_name.to_string(),
        ));
    }
    if SYMBOL_CHAR_RE.is_match(last_name) || NUM_CHAR_RE.is_match(last_name) {
        return Err(IdentityValidationError::IllegalCharInLastName(
            last_name.to_string(),
        ));
    }
    Ok(())
}

fn validate_first_name(first_name: &str) -> Result<(), IdentityValidationError> {
    let first_name_length = first_name.chars().count();
    if !(FIRST_NAME_MIN_LENGTH..=FIRST_NAME_MAX_LENGTH).contains(&first_name_length) {
        return Err(IdentityValidationError::InvalidFirstNameLength {
            length: first_name_length,
            min_length: LAST_NAME_MIN_LENGTH,
            max_length: LAST_NAME_MAX_LENGTH,
        });
    }
    if has_control_char(first_name) {
        return Err(IdentityValidationError::IllegalCharInFirstName(
            first_name.to_string(),
        ));
    }
    if SYMBOL_CHAR_RE.is_match(first_name) || NUM_CHAR_RE.is_match(first_name) {
        return Err(IdentityValidationError::IllegalCharInFirstName(
            first_name.to_string(),
        ));
    }
    Ok(())
}

fn validate_last_name_furigana(last_name_furigana: &str) -> Result<(), IdentityValidationError> {
    let last_name_furigana_length = last_name_furigana.chars().count();
    if !(LAST_NAME_FURIGANA_MIN_LENGTH..=LAST_NAME_FURIGANA_MAX_LENGTH)
        .contains(&last_name_furigana_length)
    {
        return Err(IdentityValidationError::InvalidLastNameFuriganaLength {
            length: last_name_furigana_length,
            min_length: LAST_NAME_FURIGANA_MIN_LENGTH,
            max_length: LAST_NAME_FURIGANA_MAX_LENGTH,
        });
    }
    if !ZENKAKU_KATAKANA_RE.is_match(last_name_furigana) {
        return Err(IdentityValidationError::IllegalCharInLastNameFurigana(
            last_name_furigana.to_string(),
        ));
    }
    Ok(())
}

fn validate_first_name_furigana(first_name_furigana: &str) -> Result<(), IdentityValidationError> {
    let first_name_furigana_length = first_name_furigana.chars().count();
    if !(FIRST_NAME_FURIGANA_MIN_LENGTH..=FIRST_NAME_FURIGANA_MAX_LENGTH)
        .contains(&first_name_furigana_length)
    {
        return Err(IdentityValidationError::InvalidFirstNameFuriganaLength {
            length: first_name_furigana_length,
            min_length: LAST_NAME_MIN_LENGTH,
            max_length: LAST_NAME_MAX_LENGTH,
        });
    }
    if !ZENKAKU_KATAKANA_RE.is_match(first_name_furigana) {
        return Err(IdentityValidationError::IllegalCharInFirstNameFurigana(
            first_name_furigana.to_string(),
        ));
    }
    Ok(())
}

fn has_control_char(s: &str) -> bool {
    let characters = s.chars().collect::<Vec<char>>();
    for c in characters {
        if c.is_control() {
            return true;
        }
    }
    return false;
}

fn validate_date_of_birth(
    date_of_birth: &Ymd,
    current_date: &NaiveDate,
) -> Result<(), IdentityValidationError> {
    match NaiveDate::from_ymd_opt(date_of_birth.year, date_of_birth.month, date_of_birth.day) {
        Some(ymd) => {
            let _ = validate_age_satisfies_min_age_requirement(&ymd, current_date)?;
        }
        None => {
            return Err(IdentityValidationError::IllegalDate {
                year: date_of_birth.year,
                month: date_of_birth.month,
                day: date_of_birth.day,
            })
        }
    };
    Ok(())
}

fn validate_age_satisfies_min_age_requirement(
    date_of_birth: &NaiveDate,
    current_date: &NaiveDate,
) -> Result<(), IdentityValidationError> {
    let year_diff = current_date.year() - date_of_birth.year();
    if year_diff > MIN_AGE_REQUIREMENT {
        return Ok(());
    } else if year_diff == MIN_AGE_REQUIREMENT {
        return validate_current_day_passes_birthday(date_of_birth, current_date);
    } else {
        return Err(IdentityValidationError::IllegalAge {
            birth_year: date_of_birth.year(),
            birth_month: date_of_birth.month(),
            birth_day: date_of_birth.day(),
            current_year: current_date.year(),
            current_month: current_date.month(),
            current_day: current_date.day(),
        });
    };
}

fn validate_current_day_passes_birthday(
    date_of_birth: &NaiveDate,
    current_date: &NaiveDate,
) -> Result<(), IdentityValidationError> {
    if current_date.month() > date_of_birth.month() {
        return Ok(());
    } else if current_date.month() == date_of_birth.month() {
        if current_date.day() >= date_of_birth.day() {
            return Ok(());
        } else {
            return Err(IdentityValidationError::IllegalAge {
                birth_year: date_of_birth.year(),
                birth_month: date_of_birth.month(),
                birth_day: date_of_birth.day(),
                current_year: current_date.year(),
                current_month: current_date.month(),
                current_day: current_date.day(),
            });
        }
    } else {
        return Err(IdentityValidationError::IllegalAge {
            birth_year: date_of_birth.year(),
            birth_month: date_of_birth.month(),
            birth_day: date_of_birth.day(),
            current_year: current_date.year(),
            current_month: current_date.month(),
            current_day: current_date.day(),
        });
    }
}

fn validate_prefecture(prefecture: &str) -> Result<(), IdentityValidationError> {
    if !PREFECTURE_SET.contains(prefecture) {
        return Err(IdentityValidationError::InvalidPrefecture(
            prefecture.to_string(),
        ));
    }
    Ok(())
}

fn validate_city(city: &str) -> Result<(), IdentityValidationError> {
    let city_length = city.chars().count();
    if !(CITY_MIN_LENGTH..=CITY_MAX_LENGTH).contains(&city_length) {
        return Err(IdentityValidationError::InvalidCityLength {
            length: city_length,
            min_length: CITY_MIN_LENGTH,
            max_length: CITY_MAX_LENGTH,
        });
    }
    if has_control_char(city) {
        return Err(IdentityValidationError::IllegalCharInCity(city.to_string()));
    }
    if SYMBOL_CHAR_RE.is_match(city) || NUM_CHAR_RE.is_match(city) {
        return Err(IdentityValidationError::IllegalCharInCity(city.to_string()));
    }
    Ok(())
}

fn validate_address_line1(address_line1: &str) -> Result<(), IdentityValidationError> {
    let address_line1_length = address_line1.chars().count();
    if !(ADDRESS_LINE1_MIN_LENGTH..=ADDRESS_LINE1_MAX_LENGTH).contains(&address_line1_length) {
        return Err(IdentityValidationError::InvalidAddressLine1Length {
            length: address_line1_length,
            min_length: ADDRESS_LINE1_MIN_LENGTH,
            max_length: ADDRESS_LINE1_MAX_LENGTH,
        });
    }
    if has_control_char(address_line1) {
        return Err(IdentityValidationError::IllegalCharInAddressLine1(
            address_line1.to_string(),
        ));
    }
    if SYMBOL_CHAR_WITHOUT_HYPHEN_RE.is_match(address_line1) {
        return Err(IdentityValidationError::IllegalCharInAddressLine1(
            address_line1.to_string(),
        ));
    }
    Ok(())
}

fn validate_address_line2(address_line2: &str) -> Result<(), IdentityValidationError> {
    let address_line2_length = address_line2.chars().count();
    if !(ADDRESS_LINE2_MIN_LENGTH..=ADDRESS_LINE2_MAX_LENGTH).contains(&address_line2_length) {
        return Err(IdentityValidationError::InvalidAddressLine2Length {
            length: address_line2_length,
            min_length: ADDRESS_LINE2_MIN_LENGTH,
            max_length: ADDRESS_LINE2_MAX_LENGTH,
        });
    }
    if has_control_char(address_line2) {
        return Err(IdentityValidationError::IllegalCharInAddressLine2(
            address_line2.to_string(),
        ));
    }
    if SYMBOL_CHAR_WITHOUT_HYPHEN_RE.is_match(address_line2) {
        return Err(IdentityValidationError::IllegalCharInAddressLine2(
            address_line2.to_string(),
        ));
    }
    Ok(())
}

fn validate_telephone_number(telephone_number: &str) -> Result<(), IdentityValidationError> {
    if !TEL_NUM_RE.is_match(telephone_number) {
        return Err(IdentityValidationError::InvalidTelNumFormat(
            telephone_number.to_string(),
        ));
    }
    Ok(())
}

/// Error related to [validate_identity()]
#[derive(Debug, PartialEq)]
pub(crate) enum IdentityValidationError {
    InvalidLastNameLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInLastName(String),
    InvalidFirstNameLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInFirstName(String),
    InvalidLastNameFuriganaLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInLastNameFurigana(String),
    InvalidFirstNameFuriganaLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInFirstNameFurigana(String),
    IllegalDate {
        year: i32,
        month: u32,
        day: u32,
    },
    IllegalAge {
        birth_year: i32,
        birth_month: u32,
        birth_day: u32,
        current_year: i32,
        current_month: u32,
        current_day: u32,
    },
    InvalidPrefecture(String),
    InvalidCityLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInCity(String),
    InvalidAddressLine1Length {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInAddressLine1(String),
    InvalidAddressLine2Length {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInAddressLine2(String),
    InvalidTelNumFormat(String),
}

impl Display for IdentityValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentityValidationError::InvalidLastNameLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid last_name length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            IdentityValidationError::IllegalCharInLastName(last_name) => {
                write!(
                    f,
                    "last_name: illegal charcter included: {} (binary: {:X?})",
                    last_name,
                    last_name.as_bytes().to_vec()
                )
            }
            IdentityValidationError::InvalidFirstNameLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid first_name length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            IdentityValidationError::IllegalCharInFirstName(first_name) => {
                write!(
                    f,
                    "first_name: illegal charcter included: {} (binary: {:X?})",
                    first_name,
                    first_name.as_bytes().to_vec()
                )
            }
            IdentityValidationError::InvalidLastNameFuriganaLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid last_name_furigana length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            IdentityValidationError::IllegalCharInLastNameFurigana(last_name_furigana) => {
                write!(
                    f,
                    "last_name_furigana: illegal charcter included: {} (binary: {:X?})",
                    last_name_furigana,
                    last_name_furigana.as_bytes().to_vec()
                )
            }
            IdentityValidationError::InvalidFirstNameFuriganaLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid first_name_furigana length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            IdentityValidationError::IllegalCharInFirstNameFurigana(first_name_furigana) => {
                write!(
                    f,
                    "first_name_furigana: illegal charcter included: {} (binary: {:X?})",
                    first_name_furigana,
                    first_name_furigana.as_bytes().to_vec()
                )
            }
            IdentityValidationError::IllegalDate { year, month, day } => write!(
                f,
                "illegal date (year: {}, month: {}, day: {})",
                year, month, day
            ),
            IdentityValidationError::IllegalAge {
                birth_year,
                birth_month,
                birth_day,
                current_year,
                current_month,
                current_day,
            } => write!(
                f,
                "illegal age (birthday = year: {}, month: {}, day: {}, current date = year: {}, month: {}, day: {})",
                birth_year, birth_month, birth_day, current_year, current_month, current_day
            ),
            IdentityValidationError::InvalidPrefecture(prefecture) => {
                write!(
                    f,
                    "invalid prefecture: {} (binary: {:X?})",
                    prefecture,
                    prefecture.as_bytes().to_vec()
                )
            }
            IdentityValidationError::InvalidCityLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid city length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            IdentityValidationError::IllegalCharInCity(city) => {
                write!(
                    f,
                    "city: illegal charcter included: {} (binary: {:X?})",
                    city,
                    city.as_bytes().to_vec()
                )
            }
            IdentityValidationError::InvalidAddressLine1Length {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid address_line1 length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            IdentityValidationError::IllegalCharInAddressLine1(address_line1) => {
                write!(
                    f,
                    "address_line1: illegal charcter included: {} (binary: {:X?})",
                    address_line1,
                    address_line1.as_bytes().to_vec()
                )
            }
            IdentityValidationError::InvalidAddressLine2Length {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid address_line2 length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            IdentityValidationError::IllegalCharInAddressLine2(address_line2) => {
                write!(
                    f,
                    "address_line2: illegal charcter included: {} (binary: {:X?})",
                    address_line2,
                    address_line2.as_bytes().to_vec()
                )
            }
            IdentityValidationError::InvalidTelNumFormat(tel_num) => {
                write!(f, "invalid tel_num format: {}", tel_num)
            }
        }
    }
}

impl Error for IdentityValidationError {}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use chrono::NaiveDate;
    use once_cell::sync::Lazy;

    use crate::util::{
        validator::{
            validate_identity, IdentityValidationError, LAST_NAME_MAX_LENGTH, LAST_NAME_MIN_LENGTH,
        },
        Identity, Ymd,
    };

    static SYMBOL_SET: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(32);
        set.insert("!".to_string());
        set.insert("\"".to_string());
        set.insert("#".to_string());
        set.insert("$".to_string());
        set.insert("%".to_string());
        set.insert("&".to_string());
        set.insert("'".to_string());
        set.insert("(".to_string());
        set.insert(")".to_string());
        set.insert("*".to_string());
        set.insert("+".to_string());
        set.insert(",".to_string());
        set.insert("-".to_string());
        set.insert(".".to_string());
        set.insert("/".to_string());
        set.insert(":".to_string());
        set.insert(";".to_string());
        set.insert("<".to_string());
        set.insert("=".to_string());
        set.insert(">".to_string());
        set.insert("?".to_string());
        set.insert("@".to_string());
        set.insert("[".to_string());
        set.insert("\\".to_string());
        set.insert("]".to_string());
        set.insert("^".to_string());
        set.insert("_".to_string());
        set.insert("`".to_string());
        set.insert("{".to_string());
        set.insert("|".to_string());
        set.insert("}".to_string());
        set.insert("~".to_string());
        set
    });

    #[test]
    fn validate_identity_returns_ok_if_valid_identity_is_passed() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth: Ymd {
                year: 1990,
                month: 10,
                day: 11,
            },
            prefecture: "東京都".to_string(),
            city: "町田市".to_string(),
            address_line1: "森野２−２−２２".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 30);

        let _ = validate_identity(&identity, &current_date).expect("failed to get Ok");
    }

    #[test]
    fn validate_identity_accepts_hankaku_suuji_space_and_hyphen_on_address_line1_and_address_line2()
    {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth: Ymd {
                year: 1990,
                month: 10,
                day: 11,
            },
            prefecture: "東京都".to_string(),
            city: "町田市".to_string(),
            address_line1: "森野2-2-22".to_string(),
            address_line2: Some("サーパスマンション 101号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 30);

        let _ = validate_identity(&identity, &current_date).expect("failed to get Ok");
    }

    #[test]
    fn validate_identity_accepts_none_on_address_line2() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth: Ymd {
                year: 1990,
                month: 10,
                day: 11,
            },
            prefecture: "東京都".to_string(),
            city: "町田市".to_string(),
            address_line1: "森野２ー２ー２２".to_string(),
            address_line2: None,
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 30);

        let _ = validate_identity(&identity, &current_date).expect("failed to get Ok");
    }

    #[test]
    fn validate_identity_returns_ok_if_1_char_last_name_is_passed() {
        let identity = Identity {
            last_name: "あ".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth: Ymd {
                year: 1990,
                month: 10,
                day: 11,
            },
            prefecture: "東京都".to_string(),
            city: "町田市".to_string(),
            address_line1: "森野２−２−２２".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 30);

        let _ = validate_identity(&identity, &current_date).expect("failed to get Ok");
    }

    #[test]
    fn validate_identity_returns_ok_if_128_char_last_name_is_passed() {
        let identity = Identity {
            last_name: "ああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth: Ymd {
                year: 1990,
                month: 10,
                day: 11,
            },
            prefecture: "東京都".to_string(),
            city: "町田市".to_string(),
            address_line1: "森野２−２−２２".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 30);

        let _ = validate_identity(&identity, &current_date).expect("failed to get Ok");
    }

    #[test]
    fn validate_identity_returns_err_if_empty_last_name_is_passed() {
        let identity = Identity {
            last_name: "".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth: Ymd {
                year: 1990,
                month: 10,
                day: 11,
            },
            prefecture: "東京都".to_string(),
            city: "町田市".to_string(),
            address_line1: "森野２−２−２２".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 30);

        let err = validate_identity(&identity, &current_date).expect_err("failed to get Err");

        assert_eq!(
            IdentityValidationError::InvalidLastNameLength {
                length: identity.last_name.chars().count(),
                min_length: LAST_NAME_MIN_LENGTH,
                max_length: LAST_NAME_MAX_LENGTH
            },
            err
        );
    }

    #[test]
    fn validate_identity_returns_err_if_129_chars_last_name_is_passed() {
        let identity = Identity {
            last_name: "あああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth: Ymd {
                year: 1990,
                month: 10,
                day: 11,
            },
            prefecture: "東京都".to_string(),
            city: "町田市".to_string(),
            address_line1: "森野２−２−２２".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 30);

        let err = validate_identity(&identity, &current_date).expect_err("failed to get Err");

        assert_eq!(
            IdentityValidationError::InvalidLastNameLength {
                length: identity.last_name.chars().count(),
                min_length: LAST_NAME_MIN_LENGTH,
                max_length: LAST_NAME_MAX_LENGTH
            },
            err
        );
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_is_symbol() {
        let mut identity_list = Vec::with_capacity(32);
        for s in SYMBOL_SET.iter() {
            let identity = Identity {
                last_name: s.to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: "タロウ".to_string(),
                date_of_birth: Ymd {
                    year: 1990,
                    month: 10,
                    day: 11,
                },
                prefecture: "東京都".to_string(),
                city: "町田市".to_string(),
                address_line1: "森野２−２−２２".to_string(),
                address_line2: Some("サーパスマンション　１０１号室".to_string()),
                telephone_number: "09012345678".to_string(),
            };
            identity_list.push(identity);
        }
        let current_date = NaiveDate::from_ymd(2022, 1, 30);
        for id in identity_list {
            let err = validate_identity(&id, &current_date).expect_err("failed to get Err");
            assert_eq!(
                IdentityValidationError::IllegalCharInLastName(id.last_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_includes_symbol() {
        let mut identity_list = Vec::with_capacity(32);
        for s in SYMBOL_SET.iter() {
            let identity = Identity {
                last_name: "山".to_string() + s + "田",
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: "タロウ".to_string(),
                date_of_birth: Ymd {
                    year: 1990,
                    month: 10,
                    day: 11,
                },
                prefecture: "東京都".to_string(),
                city: "町田市".to_string(),
                address_line1: "森野２−２−２２".to_string(),
                address_line2: Some("サーパスマンション　１０１号室".to_string()),
                telephone_number: "09012345678".to_string(),
            };
            identity_list.push(identity);
        }
        let current_date = NaiveDate::from_ymd(2022, 1, 30);
        for id in identity_list {
            let err = validate_identity(&id, &current_date).expect_err("failed to get Err");
            assert_eq!(
                IdentityValidationError::IllegalCharInLastName(id.last_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_starts_with_symbol() {
        let mut identity_list = Vec::with_capacity(32);
        for s in SYMBOL_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string() + s,
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: "タロウ".to_string(),
                date_of_birth: Ymd {
                    year: 1990,
                    month: 10,
                    day: 11,
                },
                prefecture: "東京都".to_string(),
                city: "町田市".to_string(),
                address_line1: "森野２−２−２２".to_string(),
                address_line2: Some("サーパスマンション　１０１号室".to_string()),
                telephone_number: "09012345678".to_string(),
            };
            identity_list.push(identity);
        }
        let current_date = NaiveDate::from_ymd(2022, 1, 30);
        for id in identity_list {
            let err = validate_identity(&id, &current_date).expect_err("failed to get Err");
            assert_eq!(
                IdentityValidationError::IllegalCharInLastName(id.last_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_ends_with_symbol() {
        let mut identity_list = Vec::with_capacity(32);
        for s in SYMBOL_SET.iter() {
            let identity = Identity {
                last_name: s.to_string() + "山田",
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: "タロウ".to_string(),
                date_of_birth: Ymd {
                    year: 1990,
                    month: 10,
                    day: 11,
                },
                prefecture: "東京都".to_string(),
                city: "町田市".to_string(),
                address_line1: "森野２−２−２２".to_string(),
                address_line2: Some("サーパスマンション　１０１号室".to_string()),
                telephone_number: "09012345678".to_string(),
            };
            identity_list.push(identity);
        }
        let current_date = NaiveDate::from_ymd(2022, 1, 30);
        for id in identity_list {
            let err = validate_identity(&id, &current_date).expect_err("failed to get Err");
            assert_eq!(
                IdentityValidationError::IllegalCharInLastName(id.last_name.to_string()),
                err
            );
        }
    }
}
