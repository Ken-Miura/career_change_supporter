// Copyright 2021 Ken Miura

use std::{collections::HashSet, error::Error, fmt::Display};

use chrono::{Datelike, NaiveDate};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::util::{Identity, Ymd};

const LAST_NAME_MIN_LENGTH: usize = 1;
const LAST_NAME_MAX_LENGTH: usize = 64;
const FIRST_NAME_MIN_LENGTH: usize = 1;
const FIRST_NAME_MAX_LENGTH: usize = 64;
const LAST_NAME_FURIGANA_MIN_LENGTH: usize = 1;
const LAST_NAME_FURIGANA_MAX_LENGTH: usize = 64;
const FIRST_NAME_FURIGANA_MIN_LENGTH: usize = 1;
const FIRST_NAME_FURIGANA_MAX_LENGTH: usize = 64;
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

/// 半角スペース、または全角スペースを一つ以上含むケース
const SPACE_REGEXP: &str = r"[ 　]+";
static SPACE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(SPACE_REGEXP).expect("failed to compile space regexp"));

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
    if SYMBOL_CHAR_RE.is_match(last_name)
        || NUM_CHAR_RE.is_match(last_name)
        || SPACE_RE.is_match(last_name)
    {
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
    if SYMBOL_CHAR_RE.is_match(first_name)
        || NUM_CHAR_RE.is_match(first_name)
        || SPACE_RE.is_match(first_name)
    {
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
    false
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
    match year_diff.cmp(&MIN_AGE_REQUIREMENT) {
        std::cmp::Ordering::Greater => Ok(()),
        std::cmp::Ordering::Equal => {
            validate_current_day_passes_birthday(date_of_birth, current_date)
        }
        std::cmp::Ordering::Less => Err(IdentityValidationError::IllegalAge {
            birth_year: date_of_birth.year(),
            birth_month: date_of_birth.month(),
            birth_day: date_of_birth.day(),
            current_year: current_date.year(),
            current_month: current_date.month(),
            current_day: current_date.day(),
        }),
    }
}

fn validate_current_day_passes_birthday(
    date_of_birth: &NaiveDate,
    current_date: &NaiveDate,
) -> Result<(), IdentityValidationError> {
    let current_month = current_date.month();
    let birth_month = date_of_birth.month();
    match current_month.cmp(&birth_month) {
        std::cmp::Ordering::Greater => Ok(()),
        std::cmp::Ordering::Equal => {
            if current_date.day() >= date_of_birth.day() {
                Ok(())
            } else {
                Err(IdentityValidationError::IllegalAge {
                    birth_year: date_of_birth.year(),
                    birth_month: date_of_birth.month(),
                    birth_day: date_of_birth.day(),
                    current_year: current_date.year(),
                    current_month: current_date.month(),
                    current_day: current_date.day(),
                })
            }
        }
        std::cmp::Ordering::Less => Err(IdentityValidationError::IllegalAge {
            birth_year: date_of_birth.year(),
            birth_month: date_of_birth.month(),
            birth_day: date_of_birth.day(),
            current_year: current_date.year(),
            current_month: current_date.month(),
            current_day: current_date.day(),
        }),
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
    if SYMBOL_CHAR_RE.is_match(city) || NUM_CHAR_RE.is_match(city) || SPACE_RE.is_match(city) {
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

    use chrono::{Datelike, NaiveDate};
    use once_cell::sync::Lazy;

    use crate::util::{
        validator::identity_validator::{
            validate_identity, IdentityValidationError, ADDRESS_LINE1_MAX_LENGTH,
            ADDRESS_LINE1_MIN_LENGTH, CITY_MAX_LENGTH, CITY_MIN_LENGTH,
            FIRST_NAME_FURIGANA_MAX_LENGTH, FIRST_NAME_FURIGANA_MIN_LENGTH, FIRST_NAME_MAX_LENGTH,
            FIRST_NAME_MIN_LENGTH, LAST_NAME_FURIGANA_MAX_LENGTH, LAST_NAME_FURIGANA_MIN_LENGTH,
            LAST_NAME_MAX_LENGTH, LAST_NAME_MIN_LENGTH,
        },
        Identity, Ymd,
    };

    use super::PREFECTURE_SET;

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

    static SYMBOL_WITH_OUT_HYPHEN_SET: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(31);
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

    static HYPHEN_SET: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(2);
        set.insert("-".to_string());
        set.insert("ー".to_string());
        set
    });

    static NUMBER_SET: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(10);
        set.insert("0".to_string());
        set.insert("1".to_string());
        set.insert("2".to_string());
        set.insert("3".to_string());
        set.insert("4".to_string());
        set.insert("5".to_string());
        set.insert("6".to_string());
        set.insert("7".to_string());
        set.insert("8".to_string());
        set.insert("9".to_string());
        set
    });

    static CONTROL_CHAR_SET: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(32 + 1 + 32);
        // C0制御コード
        set.insert('\u{0000}'.to_string());
        set.insert('\u{0001}'.to_string());
        set.insert('\u{0002}'.to_string());
        set.insert('\u{0003}'.to_string());
        set.insert('\u{0004}'.to_string());
        set.insert('\u{0005}'.to_string());
        set.insert('\u{0006}'.to_string());
        set.insert('\u{0007}'.to_string());
        set.insert('\u{0008}'.to_string());
        set.insert('\u{0009}'.to_string());
        set.insert('\u{000A}'.to_string());
        set.insert('\u{000B}'.to_string());
        set.insert('\u{000C}'.to_string());
        set.insert('\u{000D}'.to_string());
        set.insert('\u{000E}'.to_string());
        set.insert('\u{000F}'.to_string());
        set.insert('\u{0010}'.to_string());
        set.insert('\u{0011}'.to_string());
        set.insert('\u{0012}'.to_string());
        set.insert('\u{0013}'.to_string());
        set.insert('\u{0014}'.to_string());
        set.insert('\u{0015}'.to_string());
        set.insert('\u{0016}'.to_string());
        set.insert('\u{0017}'.to_string());
        set.insert('\u{0018}'.to_string());
        set.insert('\u{0019}'.to_string());
        set.insert('\u{001A}'.to_string());
        set.insert('\u{001B}'.to_string());
        set.insert('\u{001C}'.to_string());
        set.insert('\u{001D}'.to_string());
        set.insert('\u{001E}'.to_string());
        set.insert('\u{001F}'.to_string());
        // 削除文字
        set.insert('\u{007F}'.to_string());
        // C1制御コード
        set.insert('\u{0080}'.to_string());
        set.insert('\u{0081}'.to_string());
        set.insert('\u{0082}'.to_string());
        set.insert('\u{0083}'.to_string());
        set.insert('\u{0084}'.to_string());
        set.insert('\u{0085}'.to_string());
        set.insert('\u{0086}'.to_string());
        set.insert('\u{0087}'.to_string());
        set.insert('\u{0088}'.to_string());
        set.insert('\u{0089}'.to_string());
        set.insert('\u{008A}'.to_string());
        set.insert('\u{008B}'.to_string());
        set.insert('\u{008C}'.to_string());
        set.insert('\u{008D}'.to_string());
        set.insert('\u{008E}'.to_string());
        set.insert('\u{008F}'.to_string());
        set.insert('\u{0090}'.to_string());
        set.insert('\u{0091}'.to_string());
        set.insert('\u{0092}'.to_string());
        set.insert('\u{0093}'.to_string());
        set.insert('\u{0094}'.to_string());
        set.insert('\u{0095}'.to_string());
        set.insert('\u{0096}'.to_string());
        set.insert('\u{0097}'.to_string());
        set.insert('\u{0098}'.to_string());
        set.insert('\u{0099}'.to_string());
        set.insert('\u{009A}'.to_string());
        set.insert('\u{009B}'.to_string());
        set.insert('\u{009C}'.to_string());
        set.insert('\u{009D}'.to_string());
        set.insert('\u{009E}'.to_string());
        set.insert('\u{009F}'.to_string());
        set
    });

    static SPACE_SET: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(2);
        // 半角スペース
        set.insert(" ".to_string());
        // 全角スペース
        set.insert("　".to_string());
        set
    });

    static ZENKAKU_KANA_SET_FOR_NAME: Lazy<HashSet<String>> = Lazy::new(|| {
        // http://www.asahi-net.or.jp/~ax2s-kmtn/ref/unicode/u30a0.html
        // から名前の入力に必要な項目（30A1から30F4、30FC）をリストアップ
        // 上記以外の全角カナは名前入力に必要ないので候補に入れない
        let mut set: HashSet<String> = HashSet::with_capacity(85);
        set.insert('\u{30A1}'.to_string());
        set.insert('\u{30A2}'.to_string());
        set.insert('\u{30A3}'.to_string());
        set.insert('\u{30A4}'.to_string());
        set.insert('\u{30A5}'.to_string());
        set.insert('\u{30A6}'.to_string());
        set.insert('\u{30A7}'.to_string());
        set.insert('\u{30A8}'.to_string());
        set.insert('\u{30A9}'.to_string());
        set.insert('\u{30AA}'.to_string());
        set.insert('\u{30AB}'.to_string());
        set.insert('\u{30AC}'.to_string());
        set.insert('\u{30AD}'.to_string());
        set.insert('\u{30AE}'.to_string());
        set.insert('\u{30AF}'.to_string());
        set.insert('\u{30B0}'.to_string());
        set.insert('\u{30B1}'.to_string());
        set.insert('\u{30B2}'.to_string());
        set.insert('\u{30B3}'.to_string());
        set.insert('\u{30B4}'.to_string());
        set.insert('\u{30B5}'.to_string());
        set.insert('\u{30B6}'.to_string());
        set.insert('\u{30B7}'.to_string());
        set.insert('\u{30B8}'.to_string());
        set.insert('\u{30B9}'.to_string());
        set.insert('\u{30BA}'.to_string());
        set.insert('\u{30BB}'.to_string());
        set.insert('\u{30BC}'.to_string());
        set.insert('\u{30BD}'.to_string());
        set.insert('\u{30BE}'.to_string());
        set.insert('\u{30BF}'.to_string());
        set.insert('\u{30C0}'.to_string());
        set.insert('\u{30C1}'.to_string());
        set.insert('\u{30C2}'.to_string());
        set.insert('\u{30C3}'.to_string());
        set.insert('\u{30C4}'.to_string());
        set.insert('\u{30C5}'.to_string());
        set.insert('\u{30C6}'.to_string());
        set.insert('\u{30C7}'.to_string());
        set.insert('\u{30C8}'.to_string());
        set.insert('\u{30C9}'.to_string());
        set.insert('\u{30CA}'.to_string());
        set.insert('\u{30CB}'.to_string());
        set.insert('\u{30CC}'.to_string());
        set.insert('\u{30CD}'.to_string());
        set.insert('\u{30CE}'.to_string());
        set.insert('\u{30CF}'.to_string());
        set.insert('\u{30D0}'.to_string());
        set.insert('\u{30D1}'.to_string());
        set.insert('\u{30D2}'.to_string());
        set.insert('\u{30D3}'.to_string());
        set.insert('\u{30D4}'.to_string());
        set.insert('\u{30D5}'.to_string());
        set.insert('\u{30D6}'.to_string());
        set.insert('\u{30D7}'.to_string());
        set.insert('\u{30D8}'.to_string());
        set.insert('\u{30D9}'.to_string());
        set.insert('\u{30DA}'.to_string());
        set.insert('\u{30DB}'.to_string());
        set.insert('\u{30DC}'.to_string());
        set.insert('\u{30DD}'.to_string());
        set.insert('\u{30DE}'.to_string());
        set.insert('\u{30DF}'.to_string());
        set.insert('\u{30E0}'.to_string());
        set.insert('\u{30E1}'.to_string());
        set.insert('\u{30E2}'.to_string());
        set.insert('\u{30E3}'.to_string());
        set.insert('\u{30E4}'.to_string());
        set.insert('\u{30E5}'.to_string());
        set.insert('\u{30E6}'.to_string());
        set.insert('\u{30E7}'.to_string());
        set.insert('\u{30E8}'.to_string());
        set.insert('\u{30E9}'.to_string());
        set.insert('\u{30EA}'.to_string());
        set.insert('\u{30EB}'.to_string());
        set.insert('\u{30EC}'.to_string());
        set.insert('\u{30ED}'.to_string());
        set.insert('\u{30EE}'.to_string());
        set.insert('\u{30EF}'.to_string());
        set.insert('\u{30F0}'.to_string());
        set.insert('\u{30F1}'.to_string());
        set.insert('\u{30F2}'.to_string());
        set.insert('\u{30F3}'.to_string());
        set.insert('\u{30F4}'.to_string());
        set.insert('\u{30FC}'.to_string());
        set
    });

    static HANKAKU_KANA_SET: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(63);
        set.insert('\u{FF61}'.to_string());
        set.insert('\u{FF62}'.to_string());
        set.insert('\u{FF63}'.to_string());
        set.insert('\u{FF64}'.to_string());
        set.insert('\u{FF65}'.to_string());
        set.insert('\u{FF66}'.to_string());
        set.insert('\u{FF67}'.to_string());
        set.insert('\u{FF68}'.to_string());
        set.insert('\u{FF69}'.to_string());
        set.insert('\u{FF6A}'.to_string());
        set.insert('\u{FF6B}'.to_string());
        set.insert('\u{FF6C}'.to_string());
        set.insert('\u{FF6D}'.to_string());
        set.insert('\u{FF6E}'.to_string());
        set.insert('\u{FF6F}'.to_string());
        set.insert('\u{FF70}'.to_string());
        set.insert('\u{FF71}'.to_string());
        set.insert('\u{FF72}'.to_string());
        set.insert('\u{FF73}'.to_string());
        set.insert('\u{FF74}'.to_string());
        set.insert('\u{FF75}'.to_string());
        set.insert('\u{FF76}'.to_string());
        set.insert('\u{FF77}'.to_string());
        set.insert('\u{FF78}'.to_string());
        set.insert('\u{FF79}'.to_string());
        set.insert('\u{FF7A}'.to_string());
        set.insert('\u{FF7B}'.to_string());
        set.insert('\u{FF7C}'.to_string());
        set.insert('\u{FF7D}'.to_string());
        set.insert('\u{FF7E}'.to_string());
        set.insert('\u{FF7F}'.to_string());
        set.insert('\u{FF80}'.to_string());
        set.insert('\u{FF81}'.to_string());
        set.insert('\u{FF82}'.to_string());
        set.insert('\u{FF83}'.to_string());
        set.insert('\u{FF84}'.to_string());
        set.insert('\u{FF85}'.to_string());
        set.insert('\u{FF86}'.to_string());
        set.insert('\u{FF87}'.to_string());
        set.insert('\u{FF88}'.to_string());
        set.insert('\u{FF89}'.to_string());
        set.insert('\u{FF8A}'.to_string());
        set.insert('\u{FF8B}'.to_string());
        set.insert('\u{FF8C}'.to_string());
        set.insert('\u{FF8D}'.to_string());
        set.insert('\u{FF8E}'.to_string());
        set.insert('\u{FF8F}'.to_string());
        set.insert('\u{FF90}'.to_string());
        set.insert('\u{FF91}'.to_string());
        set.insert('\u{FF92}'.to_string());
        set.insert('\u{FF93}'.to_string());
        set.insert('\u{FF94}'.to_string());
        set.insert('\u{FF95}'.to_string());
        set.insert('\u{FF96}'.to_string());
        set.insert('\u{FF97}'.to_string());
        set.insert('\u{FF98}'.to_string());
        set.insert('\u{FF99}'.to_string());
        set.insert('\u{FF9A}'.to_string());
        set.insert('\u{FF9B}'.to_string());
        set.insert('\u{FF9C}'.to_string());
        set.insert('\u{FF9D}'.to_string());
        set.insert('\u{FF9E}'.to_string());
        set.insert('\u{FF9F}'.to_string());
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
    fn validate_identity_returns_ok_if_64_char_last_name_is_passed() {
        let identity = Identity {
            last_name: "ああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ".to_string(),
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
    fn validate_identity_returns_err_if_65_chars_last_name_is_passed() {
        let identity = Identity {
            last_name: "あああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ".to_string(),
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
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
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
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
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
    fn validate_identity_returns_err_if_last_name_ends_with_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
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
    fn validate_identity_returns_err_if_last_name_starts_with_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
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

    #[test]
    fn validate_identity_returns_err_if_last_name_is_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
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
    fn validate_identity_returns_err_if_last_name_includes_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
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
    fn validate_identity_returns_err_if_last_name_ends_with_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
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
    fn validate_identity_returns_err_if_last_name_starts_with_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
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

    #[test]
    fn validate_identity_returns_err_if_last_name_is_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
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
    fn validate_identity_returns_err_if_last_name_includes_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
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
    fn validate_identity_returns_err_if_last_name_ends_with_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
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
    fn validate_identity_returns_err_if_last_name_starts_with_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
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

    #[test]
    fn validate_identity_returns_err_if_last_name_is_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
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
    fn validate_identity_returns_err_if_last_name_includes_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
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
    fn validate_identity_returns_err_if_last_name_starts_with_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
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

    #[test]
    fn validate_identity_returns_err_if_last_name_ends_with_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
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
    fn validate_identity_returns_ok_if_1_char_first_name_is_passed() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "あ".to_string(),
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
    fn validate_identity_returns_ok_if_64_char_first_name_is_passed() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "ああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ".to_string(),
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
    fn validate_identity_returns_err_if_empty_first_name_is_passed() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "".to_string(),
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
            IdentityValidationError::InvalidFirstNameLength {
                length: identity.first_name.chars().count(),
                min_length: FIRST_NAME_MIN_LENGTH,
                max_length: FIRST_NAME_MAX_LENGTH
            },
            err
        );
    }

    #[test]
    fn validate_identity_returns_err_if_65_chars_first_name_is_passed() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "あああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ".to_string(),
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
            IdentityValidationError::InvalidFirstNameLength {
                length: identity.first_name.chars().count(),
                min_length: FIRST_NAME_MIN_LENGTH,
                max_length: FIRST_NAME_MAX_LENGTH
            },
            err
        );
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_is_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: s.to_string(),
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
                IdentityValidationError::IllegalCharInFirstName(id.first_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_includes_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太".to_string() + s + "郎",
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
                IdentityValidationError::IllegalCharInFirstName(id.first_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_ends_with_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string() + s,
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
                IdentityValidationError::IllegalCharInFirstName(id.first_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_starts_with_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: s.to_string() + "太郎",
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
                IdentityValidationError::IllegalCharInFirstName(id.first_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_is_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: s.to_string(),
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
                IdentityValidationError::IllegalCharInFirstName(id.first_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_includes_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太".to_string() + s + "郎",
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
                IdentityValidationError::IllegalCharInFirstName(id.first_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_ends_with_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string() + s,
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
                IdentityValidationError::IllegalCharInFirstName(id.first_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_starts_with_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: s.to_string() + "太郎",
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
                IdentityValidationError::IllegalCharInFirstName(id.first_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_is_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: s.to_string(),
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
                IdentityValidationError::IllegalCharInFirstName(id.first_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_includes_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太".to_string() + s + "郎",
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
                IdentityValidationError::IllegalCharInFirstName(id.first_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_ends_with_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string() + s,
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
                IdentityValidationError::IllegalCharInFirstName(id.first_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_starts_with_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: s.to_string() + "太郎",
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
                IdentityValidationError::IllegalCharInFirstName(id.first_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_is_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: s.to_string(),
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
                IdentityValidationError::IllegalCharInFirstName(id.first_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_includes_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太".to_string() + s + "郎",
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
                IdentityValidationError::IllegalCharInFirstName(id.first_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_starts_with_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: s.to_string() + "太郎",
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
                IdentityValidationError::IllegalCharInFirstName(id.first_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_ends_with_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string() + s,
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
                IdentityValidationError::IllegalCharInFirstName(id.first_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_accepts_zenkaku_kana_on_last_name_furigana() {
        let mut identity_list = Vec::with_capacity(ZENKAKU_KANA_SET_FOR_NAME.len());
        for s in ZENKAKU_KANA_SET_FOR_NAME.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: s.to_string(),
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
            let _ = validate_identity(&id, &current_date).expect("failed to get Ok");
        }
    }

    #[test]
    fn validate_identity_rejects_hankaku_kana_on_last_name_furigana() {
        let mut identity_list = Vec::with_capacity(HANKAKU_KANA_SET.len());
        for s in HANKAKU_KANA_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: s.to_string(),
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
                IdentityValidationError::IllegalCharInLastNameFurigana(
                    id.last_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_ok_if_1_char_last_name_furigana_is_passed() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ア".to_string(),
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
    fn validate_identity_returns_ok_if_64_char_last_name_furigana_is_passed() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "アアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアア".to_string(),
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
    fn validate_identity_returns_err_if_empty_last_name_furigana_is_passed() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "".to_string(),
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
            IdentityValidationError::InvalidLastNameFuriganaLength {
                length: identity.last_name_furigana.chars().count(),
                min_length: LAST_NAME_FURIGANA_MIN_LENGTH,
                max_length: LAST_NAME_FURIGANA_MAX_LENGTH
            },
            err
        );
    }

    #[test]
    fn validate_identity_returns_err_if_65_chars_last_name_furigana_is_passed() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "アアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアア".to_string(),
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
            IdentityValidationError::InvalidLastNameFuriganaLength {
                length: identity.last_name_furigana.chars().count(),
                min_length: LAST_NAME_FURIGANA_MIN_LENGTH,
                max_length: LAST_NAME_FURIGANA_MAX_LENGTH
            },
            err
        );
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_furigana_is_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: s.to_string(),
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
                IdentityValidationError::IllegalCharInLastNameFurigana(
                    id.last_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_furigana_includes_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤ".to_string() + s + "マダ",
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
                IdentityValidationError::IllegalCharInLastNameFurigana(
                    id.last_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_furigana_ends_with_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string() + s,
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
                IdentityValidationError::IllegalCharInLastNameFurigana(
                    id.last_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_furigana_starts_with_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: s.to_string() + "ヤマダ",
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
                IdentityValidationError::IllegalCharInLastNameFurigana(
                    id.last_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_furigana_is_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: s.to_string(),
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
                IdentityValidationError::IllegalCharInLastNameFurigana(
                    id.last_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_furigana_includes_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤ".to_string() + s + "マダ",
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
                IdentityValidationError::IllegalCharInLastNameFurigana(
                    id.last_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_furigana_ends_with_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string() + s,
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
                IdentityValidationError::IllegalCharInLastNameFurigana(
                    id.last_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_furigana_starts_with_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: s.to_string() + "ヤマダ",
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
                IdentityValidationError::IllegalCharInLastNameFurigana(
                    id.last_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_furigana_is_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: s.to_string(),
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
                IdentityValidationError::IllegalCharInLastNameFurigana(
                    id.last_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_furigana_includes_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤ".to_string() + s + "マダ",
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
                IdentityValidationError::IllegalCharInLastNameFurigana(
                    id.last_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_furigana_ends_with_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string() + s,
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
                IdentityValidationError::IllegalCharInLastNameFurigana(
                    id.last_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_furigana_starts_with_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: s.to_string() + "ヤマダ",
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
                IdentityValidationError::IllegalCharInLastNameFurigana(
                    id.last_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_furigana_is_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: s.to_string(),
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
                IdentityValidationError::IllegalCharInLastNameFurigana(
                    id.last_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_furigana_includes_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤ".to_string() + s + "マダ",
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
                IdentityValidationError::IllegalCharInLastNameFurigana(
                    id.last_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_furigana_starts_with_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: s.to_string() + "ヤマダ",
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
                IdentityValidationError::IllegalCharInLastNameFurigana(
                    id.last_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_last_name_furigana_ends_with_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string() + s,
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
                IdentityValidationError::IllegalCharInLastNameFurigana(
                    id.last_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_accepts_zenkaku_kana_on_first_name_furigana() {
        let mut identity_list = Vec::with_capacity(ZENKAKU_KANA_SET_FOR_NAME.len());
        for s in ZENKAKU_KANA_SET_FOR_NAME.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: s.to_string(),
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
            let _ = validate_identity(&id, &current_date).expect("failed to get Ok");
        }
    }

    #[test]
    fn validate_identity_rejects_hankaku_kana_on_first_name_furigana() {
        let mut identity_list = Vec::with_capacity(HANKAKU_KANA_SET.len());
        for s in HANKAKU_KANA_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: s.to_string(),
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
                IdentityValidationError::IllegalCharInFirstNameFurigana(
                    id.first_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_ok_if_1_char_first_name_furigana_is_passed() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "ア".to_string(),
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
    fn validate_identity_returns_ok_if_64_char_first_name_furigana_is_passed() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "アアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアア".to_string(),
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
    fn validate_identity_returns_err_if_empty_first_name_furigana_is_passed() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "".to_string(),
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
            IdentityValidationError::InvalidFirstNameFuriganaLength {
                length: identity.first_name_furigana.chars().count(),
                min_length: FIRST_NAME_FURIGANA_MIN_LENGTH,
                max_length: FIRST_NAME_FURIGANA_MAX_LENGTH
            },
            err
        );
    }

    #[test]
    fn validate_identity_returns_err_if_65_chars_first_name_furigana_is_passed() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "アアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアアア".to_string(),
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
            IdentityValidationError::InvalidFirstNameFuriganaLength {
                length: identity.first_name_furigana.chars().count(),
                min_length: FIRST_NAME_FURIGANA_MIN_LENGTH,
                max_length: FIRST_NAME_FURIGANA_MAX_LENGTH
            },
            err
        );
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_furigana_is_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: s.to_string(),
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
                IdentityValidationError::IllegalCharInFirstNameFurigana(
                    id.first_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_furigana_includes_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: "タ".to_string() + s + "ロウ",
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
                IdentityValidationError::IllegalCharInFirstNameFurigana(
                    id.first_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_furigana_ends_with_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: "タロウ".to_string() + s,
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
                IdentityValidationError::IllegalCharInFirstNameFurigana(
                    id.first_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_furigana_starts_with_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: s.to_string() + "タロウ",
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
                IdentityValidationError::IllegalCharInFirstNameFurigana(
                    id.first_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_furigana_is_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: s.to_string(),
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
                IdentityValidationError::IllegalCharInFirstNameFurigana(
                    id.first_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_furigana_includes_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: "タ".to_string() + s + "ロウ",
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
                IdentityValidationError::IllegalCharInFirstNameFurigana(
                    id.first_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_furigana_ends_with_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: "タロウ".to_string() + s,
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
                IdentityValidationError::IllegalCharInFirstNameFurigana(
                    id.first_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_furigana_starts_with_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: s.to_string() + "タロウ",
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
                IdentityValidationError::IllegalCharInFirstNameFurigana(
                    id.first_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_furigana_is_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: s.to_string(),
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
                IdentityValidationError::IllegalCharInFirstNameFurigana(
                    id.first_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_furigana_includes_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: "タ".to_string() + s + "ロウ",
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
                IdentityValidationError::IllegalCharInFirstNameFurigana(
                    id.first_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_furigana_ends_with_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: "タロウ".to_string() + s,
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
                IdentityValidationError::IllegalCharInFirstNameFurigana(
                    id.first_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_furigana_starts_with_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: s.to_string() + "タロウ",
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
                IdentityValidationError::IllegalCharInFirstNameFurigana(
                    id.first_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_furigana_is_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: s.to_string(),
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
                IdentityValidationError::IllegalCharInFirstNameFurigana(
                    id.first_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_furigana_includes_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: "タ".to_string() + s + "ロウ",
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
                IdentityValidationError::IllegalCharInFirstNameFurigana(
                    id.first_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_furigana_starts_with_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: s.to_string() + "タロウ",
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
                IdentityValidationError::IllegalCharInFirstNameFurigana(
                    id.first_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_first_name_furigana_ends_with_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let identity = Identity {
                last_name: "山田".to_string(),
                first_name: "太郎".to_string(),
                last_name_furigana: "ヤマダ".to_string(),
                first_name_furigana: "タロウ".to_string() + s,
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
                IdentityValidationError::IllegalCharInFirstNameFurigana(
                    id.first_name_furigana.to_string()
                ),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_ok_if_user_is_19_years_old_or_more() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth: Ymd {
                year: 2003,
                month: 1,
                day: 1,
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
    fn validate_identity_returns_ok_if_user_is_18_years_old_and_already_passed_birth_month() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth: Ymd {
                year: 2004,
                month: 1,
                day: 1,
            },
            prefecture: "東京都".to_string(),
            city: "町田市".to_string(),
            address_line1: "森野２−２−２２".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 2, 1);

        let _ = validate_identity(&identity, &current_date).expect("failed to get Ok");
    }

    #[test]
    fn validate_identity_returns_ok_if_user_is_18_years_old_and_already_passed_birth_day() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth: Ymd {
                year: 2004,
                month: 1,
                day: 1,
            },
            prefecture: "東京都".to_string(),
            city: "町田市".to_string(),
            address_line1: "森野２−２−２２".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 2);

        let _ = validate_identity(&identity, &current_date).expect("failed to get Ok");
    }

    #[test]
    fn validate_identity_returns_ok_if_user_is_just_18_years_old() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth: Ymd {
                year: 2004,
                month: 1,
                day: 2,
            },
            prefecture: "東京都".to_string(),
            city: "町田市".to_string(),
            address_line1: "森野２−２−２２".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 2);

        let _ = validate_identity(&identity, &current_date).expect("failed to get Ok");
    }

    #[test]
    fn validate_identity_returns_err_if_user_is_17_years_old_and_day_before_birth_day1() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth: Ymd {
                year: 2004,
                month: 1,
                day: 2,
            },
            prefecture: "東京都".to_string(),
            city: "町田市".to_string(),
            address_line1: "森野２−２−２２".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 1);

        let err = validate_identity(&identity, &current_date).expect_err("failed to get Err");
        assert_eq!(
            IdentityValidationError::IllegalAge {
                birth_year: identity.date_of_birth.year,
                birth_month: identity.date_of_birth.month,
                birth_day: identity.date_of_birth.day,
                current_year: current_date.year(),
                current_month: current_date.month(),
                current_day: current_date.day()
            },
            err
        );
    }

    #[test]
    fn validate_identity_returns_err_if_user_is_17_years_old_and_day_before_birth_day2() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth: Ymd {
                year: 2004,
                month: 1,
                day: 1,
            },
            prefecture: "東京都".to_string(),
            city: "町田市".to_string(),
            address_line1: "森野２−２−２２".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2021, 12, 31);

        let err = validate_identity(&identity, &current_date).expect_err("failed to get Err");
        assert_eq!(
            IdentityValidationError::IllegalAge {
                birth_year: identity.date_of_birth.year,
                birth_month: identity.date_of_birth.month,
                birth_day: identity.date_of_birth.day,
                current_year: current_date.year(),
                current_month: current_date.month(),
                current_day: current_date.day()
            },
            err
        );
    }

    #[test]
    fn validate_identity_returns_err_if_user_is_17_years_old_and_month_before_birth_month1() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth: Ymd {
                year: 2004,
                month: 2,
                day: 2,
            },
            prefecture: "東京都".to_string(),
            city: "町田市".to_string(),
            address_line1: "森野２−２−２２".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 2);

        let err = validate_identity(&identity, &current_date).expect_err("failed to get Err");
        assert_eq!(
            IdentityValidationError::IllegalAge {
                birth_year: identity.date_of_birth.year,
                birth_month: identity.date_of_birth.month,
                birth_day: identity.date_of_birth.day,
                current_year: current_date.year(),
                current_month: current_date.month(),
                current_day: current_date.day()
            },
            err
        );
    }

    #[test]
    fn validate_identity_returns_err_if_user_is_17_years_old_and_month_before_birth_month2() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth: Ymd {
                year: 2004,
                month: 1,
                day: 2,
            },
            prefecture: "東京都".to_string(),
            city: "町田市".to_string(),
            address_line1: "森野２−２−２２".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2021, 12, 2);

        let err = validate_identity(&identity, &current_date).expect_err("failed to get Err");
        assert_eq!(
            IdentityValidationError::IllegalAge {
                birth_year: identity.date_of_birth.year,
                birth_month: identity.date_of_birth.month,
                birth_day: identity.date_of_birth.day,
                current_year: current_date.year(),
                current_month: current_date.month(),
                current_day: current_date.day()
            },
            err
        );
    }

    #[test]
    fn validate_identity_returns_err_if_user_is_16_years_old_or_less() {
        let identity = Identity {
            last_name: "山田".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "ヤマダ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth: Ymd {
                year: 2006,
                month: 1,
                day: 2,
            },
            prefecture: "東京都".to_string(),
            city: "町田市".to_string(),
            address_line1: "森野２−２−２２".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 20);

        let err = validate_identity(&identity, &current_date).expect_err("failed to get Err");
        assert_eq!(
            IdentityValidationError::IllegalAge {
                birth_year: identity.date_of_birth.year,
                birth_month: identity.date_of_birth.month,
                birth_day: identity.date_of_birth.day,
                current_year: current_date.year(),
                current_month: current_date.month(),
                current_day: current_date.day()
            },
            err
        );
    }

    #[test]
    fn validate_identity_returns_ok_for_valid_prefecture() {
        let mut identity_list = Vec::with_capacity(PREFECTURE_SET.len());
        for s in PREFECTURE_SET.iter() {
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
                prefecture: s.to_string(),
                city: "町田市".to_string(),
                address_line1: "森野２−２−２２".to_string(),
                address_line2: Some("サーパスマンション　１０１号室".to_string()),
                telephone_number: "09012345678".to_string(),
            };
            identity_list.push(identity);
        }
        let current_date = NaiveDate::from_ymd(2022, 1, 30);
        for id in identity_list {
            let _ = validate_identity(&id, &current_date).expect("failed to get Ok");
        }
    }

    #[test]
    fn validate_identity_returns_err_if_prefecture_is_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
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
                prefecture: s.to_string(),
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
                IdentityValidationError::InvalidPrefecture(id.prefecture.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_prefecture_includes_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
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
                prefecture: "東".to_string() + s + "京都",
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
                IdentityValidationError::InvalidPrefecture(id.prefecture.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_prefecture_ends_with_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
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
                prefecture: "東京都".to_string() + s,
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
                IdentityValidationError::InvalidPrefecture(id.prefecture.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_prefecture_starts_with_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
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
                prefecture: s.to_string() + "東京都",
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
                IdentityValidationError::InvalidPrefecture(id.prefecture.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_prefecture_is_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
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
                prefecture: s.to_string(),
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
                IdentityValidationError::InvalidPrefecture(id.prefecture.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_prefecture_includes_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
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
                prefecture: "東".to_string() + s + "京都",
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
                IdentityValidationError::InvalidPrefecture(id.prefecture.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_prefecture_ends_with_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
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
                prefecture: "東京都".to_string() + s,
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
                IdentityValidationError::InvalidPrefecture(id.prefecture.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_prefecture_starts_with_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
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
                prefecture: s.to_string() + "東京都",
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
                IdentityValidationError::InvalidPrefecture(id.prefecture.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_prefecture_is_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
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
                prefecture: s.to_string(),
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
                IdentityValidationError::InvalidPrefecture(id.prefecture.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_prefecture_includes_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
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
                prefecture: "東".to_string() + s + "京都",
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
                IdentityValidationError::InvalidPrefecture(id.prefecture.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_prefecture_ends_with_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
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
                prefecture: "東京都".to_string() + s,
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
                IdentityValidationError::InvalidPrefecture(id.prefecture.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_prefecture_starts_with_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
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
                prefecture: s.to_string() + "東京都",
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
                IdentityValidationError::InvalidPrefecture(id.prefecture.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_prefecture_is_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
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
                prefecture: s.to_string(),
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
                IdentityValidationError::InvalidPrefecture(id.prefecture.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_prefecture_includes_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
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
                prefecture: "東".to_string() + s + "京都",
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
                IdentityValidationError::InvalidPrefecture(id.prefecture.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_prefecture_starts_with_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
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
                prefecture: s.to_string() + "東京都",
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
                IdentityValidationError::InvalidPrefecture(id.prefecture.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_prefecture_ends_with_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
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
                prefecture: "東京都".to_string() + s,
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
                IdentityValidationError::InvalidPrefecture(id.prefecture.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_ok_if_1_char_city_is_passed() {
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
            city: "あ".to_string(),
            address_line1: "森野２−２−２２".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 30);

        let _ = validate_identity(&identity, &current_date).expect("failed to get Ok");
    }

    #[test]
    fn validate_identity_returns_ok_if_32_char_city_is_passed() {
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
            city: "ああああああああああああああああああああああああああああああああ".to_string(),
            address_line1: "森野２−２−２２".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 30);

        let _ = validate_identity(&identity, &current_date).expect("failed to get Ok");
    }

    #[test]
    fn validate_identity_returns_err_if_empty_city_is_passed() {
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
            city: "".to_string(),
            address_line1: "森野２−２−２２".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 30);

        let err = validate_identity(&identity, &current_date).expect_err("failed to get Err");

        assert_eq!(
            IdentityValidationError::InvalidCityLength {
                length: identity.city.chars().count(),
                min_length: CITY_MIN_LENGTH,
                max_length: CITY_MAX_LENGTH
            },
            err
        );
    }

    #[test]
    fn validate_identity_returns_err_if_33_chars_city_is_passed() {
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
            city: "あああああああああああああああああああああああああああああああああ".to_string(),
            address_line1: "森野２−２−２２".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 30);

        let err = validate_identity(&identity, &current_date).expect_err("failed to get Err");

        assert_eq!(
            IdentityValidationError::InvalidCityLength {
                length: identity.city.chars().count(),
                min_length: CITY_MIN_LENGTH,
                max_length: CITY_MAX_LENGTH
            },
            err
        );
    }

    #[test]
    fn validate_identity_returns_err_if_city_is_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
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
                city: s.to_string(),
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
                IdentityValidationError::IllegalCharInCity(id.city.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_city_includes_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
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
                city: "町".to_string() + s + "田市",
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
                IdentityValidationError::IllegalCharInCity(id.city.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_city_ends_with_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
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
                city: "町田市".to_string() + s,
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
                IdentityValidationError::IllegalCharInCity(id.city.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_city_starts_with_symbol() {
        let mut identity_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
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
                city: s.to_string() + "町田市",
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
                IdentityValidationError::IllegalCharInCity(id.city.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_city_is_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
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
                city: s.to_string(),
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
                IdentityValidationError::IllegalCharInCity(id.city.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_city_includes_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
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
                city: "町".to_string() + s + "田市",
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
                IdentityValidationError::IllegalCharInCity(id.city.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_city_ends_with_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
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
                city: "町田市".to_string() + s,
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
                IdentityValidationError::IllegalCharInCity(id.city.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_city_starts_with_number() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
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
                city: s.to_string() + "町田市",
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
                IdentityValidationError::IllegalCharInCity(id.city.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_city_is_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
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
                city: s.to_string(),
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
                IdentityValidationError::IllegalCharInCity(id.city.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_city_includes_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
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
                city: "町".to_string() + s + "田市",
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
                IdentityValidationError::IllegalCharInCity(id.city.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_city_ends_with_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
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
                city: "町田市".to_string() + s,
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
                IdentityValidationError::IllegalCharInCity(id.city.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_city_starts_with_control_char() {
        let mut identity_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
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
                city: s.to_string() + "町田市",
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
                IdentityValidationError::IllegalCharInCity(id.city.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_city_is_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
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
                city: s.to_string(),
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
                IdentityValidationError::IllegalCharInCity(id.city.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_city_name_includes_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
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
                city: "町".to_string() + s + "田市",
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
                IdentityValidationError::IllegalCharInCity(id.city.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_city_starts_with_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
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
                city: s.to_string() + "町田市",
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
                IdentityValidationError::IllegalCharInCity(id.city.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_err_if_city_ends_with_space() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
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
                city: "町田市".to_string() + s,
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
                IdentityValidationError::IllegalCharInCity(id.city.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_identity_returns_ok_if_1_char_address_line1_is_passed() {
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
            address_line1: "あ".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 30);

        let _ = validate_identity(&identity, &current_date).expect("failed to get Ok");
    }

    #[test]
    fn validate_identity_returns_ok_if_128_char_address_line1_is_passed() {
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
            address_line1: "ああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 30);

        let _ = validate_identity(&identity, &current_date).expect("failed to get Ok");
    }

    #[test]
    fn validate_identity_returns_err_if_empty_address_line1_is_passed() {
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
            address_line1: "".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 30);

        let err = validate_identity(&identity, &current_date).expect_err("failed to get Err");

        assert_eq!(
            IdentityValidationError::InvalidAddressLine1Length {
                length: identity.address_line1.chars().count(),
                min_length: ADDRESS_LINE1_MIN_LENGTH,
                max_length: ADDRESS_LINE1_MAX_LENGTH
            },
            err
        );
    }

    #[test]
    fn validate_identity_returns_err_if_129_chars_city_is_passed() {
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
            address_line1: "あああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ".to_string(),
            address_line2: Some("サーパスマンション　１０１号室".to_string()),
            telephone_number: "09012345678".to_string(),
        };
        let current_date = NaiveDate::from_ymd(2022, 1, 30);

        let err = validate_identity(&identity, &current_date).expect_err("failed to get Err");

        assert_eq!(
            IdentityValidationError::InvalidAddressLine1Length {
                length: identity.address_line1.chars().count(),
                min_length: ADDRESS_LINE1_MIN_LENGTH,
                max_length: ADDRESS_LINE1_MAX_LENGTH
            },
            err
        );
    }

    #[test]
    fn validate_identity_accepts_hypen_on_address_line1() {
        let mut identity_list = Vec::with_capacity(HYPHEN_SET.len());
        for s in HYPHEN_SET.iter() {
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
                address_line1: s.to_string(),
                address_line2: Some("サーパスマンション　１０１号室".to_string()),
                telephone_number: "09012345678".to_string(),
            };
            identity_list.push(identity);
        }
        let current_date = NaiveDate::from_ymd(2022, 1, 30);
        for id in identity_list {
            let _ = validate_identity(&id, &current_date).expect("failed to get Ok");
        }
    }

    #[test]
    fn validate_identity_accepts_space_on_address_line1() {
        let mut identity_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
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
                address_line1: s.to_string(),
                address_line2: Some("サーパスマンション　１０１号室".to_string()),
                telephone_number: "09012345678".to_string(),
            };
            identity_list.push(identity);
        }
        let current_date = NaiveDate::from_ymd(2022, 1, 30);
        for id in identity_list {
            let _ = validate_identity(&id, &current_date).expect("failed to get Ok");
        }
    }

    #[test]
    fn validate_identity_accepts_number_on_address_line1() {
        let mut identity_list = Vec::with_capacity(NUMBER_SET.len());
        for s in NUMBER_SET.iter() {
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
                address_line1: s.to_string(),
                address_line2: Some("サーパスマンション　１０１号室".to_string()),
                telephone_number: "09012345678".to_string(),
            };
            identity_list.push(identity);
        }
        let current_date = NaiveDate::from_ymd(2022, 1, 30);
        for id in identity_list {
            let _ = validate_identity(&id, &current_date).expect("failed to get Ok");
        }
    }
}
