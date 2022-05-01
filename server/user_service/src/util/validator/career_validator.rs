// Copyright 2022 Ken Miura

use std::{collections::HashSet, error::Error, fmt::Display};

use chrono::NaiveDate;
use common::util::validator::{
    has_control_char, has_non_new_line_control_char, SPACE_RE, SYMBOL_CHAR_RE,
};
use common::util::{Career, Ymd};
use once_cell::sync::Lazy;

pub(crate) const COMPANY_NAME_MIN_LENGTH: usize = 1;
pub(crate) const COMPANY_NAME_MAX_LENGTH: usize = 256;
pub(crate) const DEPARTMENT_NAME_MIN_LENGTH: usize = 1;
pub(crate) const DEPARTMENT_NAME_MAX_LENGTH: usize = 256;
pub(crate) const OFFICE_MIN_LENGTH: usize = 1;
pub(crate) const OFFICE_MAX_LENGTH: usize = 256;
pub(crate) const PROFESSION_MIN_LENGTH: usize = 1;
pub(crate) const PROFESSION_MAX_LENGTH: usize = 128;
pub(crate) const POSITION_NAME_MIN_LENGTH: usize = 1;
pub(crate) const POSITION_NAME_MAX_LENGTH: usize = 128;
pub(crate) const NOTE_MIN_LENGTH: usize = 1;
pub(crate) const NOTE_MAX_LENGTH: usize = 2048;

static CONTRACT_TYPE_SET: Lazy<HashSet<String>> = Lazy::new(|| {
    let mut set: HashSet<String> = HashSet::with_capacity(3);
    set.insert("regular".to_string());
    set.insert("contract".to_string());
    set.insert("other".to_string());
    set
});

pub(crate) fn validate_career(career: &Career) -> Result<(), CareerValidationError> {
    let _ = validate_company_name(&career.company_name)?;
    if let Some(department_name) = career.department_name.clone() {
        let _ = validate_department_name(department_name.as_str())?;
    }
    if let Some(office) = career.office.clone() {
        let _ = validate_office(office.as_str())?;
    }
    let _ = validate_career_start_date(&career.career_start_date)?;
    if let Some(career_end_date) = career.career_end_date.clone() {
        let _ = validate_career_end_date(&career_end_date)?;
        let _ = ensure_career_start_date_does_not_exceed_career_end_date(
            &career.career_start_date,
            &career_end_date,
        )?;
    }
    let _ = validate_contract_type(&career.contract_type)?;
    if let Some(profession) = career.profession.clone() {
        let _ = validate_profession(profession.as_str())?;
    }
    if let Some(annual_income_in_man_yen) = career.annual_income_in_man_yen {
        let _ = validate_annual_income_in_man_yen(annual_income_in_man_yen)?;
    }
    if let Some(position_name) = career.position_name.clone() {
        let _ = validate_position_name(position_name.as_str())?;
    }
    if let Some(note) = career.note.clone() {
        let _ = validate_note(note.as_str())?;
    }
    Ok(())
}

// NOTE:
// - 英単語の区切りに空白が許可されているので、空白のチェックはしない
// - 脆弱性の作り込みをさけるため、半角記号は許可しない。記号が必要な場合、全角を用いてもらう
// 補足：
//   日本の会社名は、仕様としていくつかの記号が許可されている（※1）
//   今後サービスを改善し、半角記号を利用可能にする場合、アプリのみでなく、ORMとDBを含めた自動の結合テストを必ず用意する。
//   そして用意されたテストでは、SQLインジェクションが発生しないことを必ずテストする（※2）
//   （'（アポストロフィー）が、会社名の仕様として許可されているので、特にそれが問題ないことは確認する）
// （※1）https://vs-group.jp/tax/startup/48check/10check/
// （※2）ORMがアポストロフィー含め、エスケープが必要な文字をすべてエスケープしていること、
//        DBがORMの実装しているエスケープ方法（DBやそのDBのバージョンによってエスケープが必要な文字に対するエスケープ方法が異なるケースがある）に対応していること、
//        は、ぞれぞれORMとDBを含めた結合テストまで実施しないと確認できない。重要なセキュリティインシデントにつながる可能性があるため、
//        必ず自動化されたテストとして実装し、テストの実施漏れによる問題の検出漏れをさけるようにする必要がある。
fn validate_company_name(company_name: &str) -> Result<(), CareerValidationError> {
    let company_name_length = company_name.chars().count();
    if !(COMPANY_NAME_MIN_LENGTH..=COMPANY_NAME_MAX_LENGTH).contains(&company_name_length) {
        return Err(CareerValidationError::InvalidCompanyNameLength {
            length: company_name_length,
            min_length: COMPANY_NAME_MIN_LENGTH,
            max_length: COMPANY_NAME_MAX_LENGTH,
        });
    }
    if has_control_char(company_name) {
        return Err(CareerValidationError::IllegalCharInCompanyName(
            company_name.to_string(),
        ));
    }
    if SYMBOL_CHAR_RE.is_match(company_name) {
        return Err(CareerValidationError::IllegalCharInCompanyName(
            company_name.to_string(),
        ));
    }
    Ok(())
}

fn validate_department_name(department_name: &str) -> Result<(), CareerValidationError> {
    let department_name_length = department_name.chars().count();
    if !(DEPARTMENT_NAME_MIN_LENGTH..=DEPARTMENT_NAME_MAX_LENGTH).contains(&department_name_length)
    {
        return Err(CareerValidationError::InvalidDepartmentNameLength {
            length: department_name_length,
            min_length: DEPARTMENT_NAME_MIN_LENGTH,
            max_length: DEPARTMENT_NAME_MAX_LENGTH,
        });
    }
    if has_control_char(department_name) {
        return Err(CareerValidationError::IllegalCharInDepartmentName(
            department_name.to_string(),
        ));
    }
    if SYMBOL_CHAR_RE.is_match(department_name) {
        return Err(CareerValidationError::IllegalCharInDepartmentName(
            department_name.to_string(),
        ));
    }
    Ok(())
}

fn validate_office(office: &str) -> Result<(), CareerValidationError> {
    let office_length = office.chars().count();
    if !(OFFICE_MIN_LENGTH..=OFFICE_MAX_LENGTH).contains(&office_length) {
        return Err(CareerValidationError::InvalidOfficeLength {
            length: office_length,
            min_length: OFFICE_MIN_LENGTH,
            max_length: OFFICE_MAX_LENGTH,
        });
    }
    if has_control_char(office) {
        return Err(CareerValidationError::IllegalCharInOffice(
            office.to_string(),
        ));
    }
    if SYMBOL_CHAR_RE.is_match(office) || SPACE_RE.is_match(office) {
        return Err(CareerValidationError::IllegalCharInOffice(
            office.to_string(),
        ));
    }
    Ok(())
}

fn validate_career_start_date(career_start_date: &Ymd) -> Result<(), CareerValidationError> {
    match NaiveDate::from_ymd_opt(
        career_start_date.year,
        career_start_date.month,
        career_start_date.day,
    ) {
        Some(_) => Ok(()),
        None => Err(CareerValidationError::IllegalCareerStartDate {
            year: career_start_date.year,
            month: career_start_date.month,
            day: career_start_date.day,
        }),
    }
}

fn validate_career_end_date(career_end_date: &Ymd) -> Result<(), CareerValidationError> {
    match NaiveDate::from_ymd_opt(
        career_end_date.year,
        career_end_date.month,
        career_end_date.day,
    ) {
        Some(_) => Ok(()),
        None => Err(CareerValidationError::IllegalCareerEndDate {
            year: career_end_date.year,
            month: career_end_date.month,
            day: career_end_date.day,
        }),
    }
}

// 入社日が退社日を超えていないことを確認する。入社日 == 退社日は許容する。
fn ensure_career_start_date_does_not_exceed_career_end_date(
    career_start_date: &Ymd,
    career_end_date: &Ymd,
) -> Result<(), CareerValidationError> {
    let start_date = NaiveDate::from_ymd(
        career_start_date.year,
        career_start_date.month,
        career_start_date.day,
    );
    let end_date = NaiveDate::from_ymd(
        career_end_date.year,
        career_end_date.month,
        career_end_date.day,
    );
    if start_date > end_date {
        return Err(CareerValidationError::CareerStartDateExceedsCareerEndDate {
            career_start_date: career_start_date.clone(),
            career_end_date: career_end_date.clone(),
        });
    }
    Ok(())
}

fn validate_contract_type(contract_type: &str) -> Result<(), CareerValidationError> {
    if !CONTRACT_TYPE_SET.contains(contract_type) {
        return Err(CareerValidationError::IllegalContractType(
            contract_type.to_string(),
        ));
    }
    Ok(())
}

fn validate_profession(profession: &str) -> Result<(), CareerValidationError> {
    let profession_length = profession.chars().count();
    if !(PROFESSION_MIN_LENGTH..=PROFESSION_MAX_LENGTH).contains(&profession_length) {
        return Err(CareerValidationError::InvalidProfessionLength {
            length: profession_length,
            min_length: PROFESSION_MIN_LENGTH,
            max_length: PROFESSION_MAX_LENGTH,
        });
    }
    if has_control_char(profession) {
        return Err(CareerValidationError::IllegalCharInProfession(
            profession.to_string(),
        ));
    }
    if SYMBOL_CHAR_RE.is_match(profession) || SPACE_RE.is_match(profession) {
        return Err(CareerValidationError::IllegalCharInProfession(
            profession.to_string(),
        ));
    }
    Ok(())
}

fn validate_annual_income_in_man_yen(
    annual_income_in_man_yen: i32,
) -> Result<(), CareerValidationError> {
    if annual_income_in_man_yen.is_negative() {
        return Err(CareerValidationError::IllegalAnnualIncomInManYen(
            annual_income_in_man_yen,
        ));
    }
    Ok(())
}

fn validate_position_name(position_name: &str) -> Result<(), CareerValidationError> {
    let position_name_length = position_name.chars().count();
    if !(POSITION_NAME_MIN_LENGTH..=POSITION_NAME_MAX_LENGTH).contains(&position_name_length) {
        return Err(CareerValidationError::InvalidPositionNameLength {
            length: position_name_length,
            min_length: POSITION_NAME_MIN_LENGTH,
            max_length: POSITION_NAME_MAX_LENGTH,
        });
    }
    if has_control_char(position_name) {
        return Err(CareerValidationError::IllegalCharInPositionName(
            position_name.to_string(),
        ));
    }
    if SYMBOL_CHAR_RE.is_match(position_name) || SPACE_RE.is_match(position_name) {
        return Err(CareerValidationError::IllegalCharInPositionName(
            position_name.to_string(),
        ));
    }
    Ok(())
}

fn validate_note(note: &str) -> Result<(), CareerValidationError> {
    let note_length = note.chars().count();
    if !(NOTE_MIN_LENGTH..=NOTE_MAX_LENGTH).contains(&note_length) {
        return Err(CareerValidationError::InvalidNoteLength {
            length: note_length,
            min_length: NOTE_MIN_LENGTH,
            max_length: NOTE_MAX_LENGTH,
        });
    }
    if has_non_new_line_control_char(note) {
        return Err(CareerValidationError::IllegalCharInNote(note.to_string()));
    }
    if SYMBOL_CHAR_RE.is_match(note) {
        return Err(CareerValidationError::IllegalCharInNote(note.to_string()));
    }
    Ok(())
}

/// Error related to [validate_career()]
#[derive(Debug, PartialEq)]
pub(crate) enum CareerValidationError {
    InvalidCompanyNameLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInCompanyName(String),
    InvalidDepartmentNameLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInDepartmentName(String),
    InvalidOfficeLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInOffice(String),
    IllegalCareerStartDate {
        year: i32,
        month: u32,
        day: u32,
    },
    IllegalCareerEndDate {
        year: i32,
        month: u32,
        day: u32,
    },
    CareerStartDateExceedsCareerEndDate {
        career_start_date: Ymd,
        career_end_date: Ymd,
    },
    IllegalContractType(String),
    InvalidProfessionLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInProfession(String),
    IllegalAnnualIncomInManYen(i32),
    InvalidPositionNameLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInPositionName(String),
    InvalidNoteLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInNote(String),
}

impl Display for CareerValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CareerValidationError::InvalidCompanyNameLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid company_name length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            CareerValidationError::IllegalCharInCompanyName(company_name) => {
                write!(
                    f,
                    "company_name: illegal charcter included: {} (binary: {:X?})",
                    company_name,
                    company_name.as_bytes().to_vec()
                )
            }
            CareerValidationError::InvalidDepartmentNameLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid department_name length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            CareerValidationError::IllegalCharInDepartmentName(department_name) => {
                write!(
                    f,
                    "department_name: illegal charcter included: {} (binary: {:X?})",
                    department_name,
                    department_name.as_bytes().to_vec()
                )
            }
            CareerValidationError::InvalidOfficeLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid office length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            CareerValidationError::IllegalCharInOffice(office) => {
                write!(
                    f,
                    "office: illegal charcter included: {} (binary: {:X?})",
                    office,
                    office.as_bytes().to_vec()
                )
            }
            CareerValidationError::IllegalCareerStartDate { year, month, day } => write!(
                f,
                "illegal career_start_date (year: {}, month: {}, day: {})",
                year, month, day
            ),
            CareerValidationError::IllegalCareerEndDate { year, month, day } => write!(
                f,
                "illegal career_end_date (year: {}, month: {}, day: {})",
                year, month, day
            ),
            CareerValidationError::CareerStartDateExceedsCareerEndDate {
                career_start_date,
                career_end_date,
            } => write!(
                f,
                "career_start_date (year: {}, month: {}, day: {}) exceeds career_end_date (year: {}, month: {}, day: {})",
                career_start_date.year, career_start_date.month, career_start_date.day,
                career_end_date.year, career_end_date.month, career_end_date.day
            ),
            CareerValidationError::IllegalContractType(contract_type) => write!(
                f,
                "illegal contract_type ({})",
                contract_type
            ),
            CareerValidationError::InvalidProfessionLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid profession length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            CareerValidationError::IllegalCharInProfession(profession) => {
                write!(
                    f,
                    "profession: illegal charcter included: {} (binary: {:X?})",
                    profession,
                    profession.as_bytes().to_vec()
                )
            }
            CareerValidationError::IllegalAnnualIncomInManYen(annual_income_in_man_yen) => {
                write!(
                    f,
                    "illegal annual_income_in_man_yen: {}",
                    annual_income_in_man_yen
                )
            }
            CareerValidationError::InvalidPositionNameLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid position_name length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            CareerValidationError::IllegalCharInPositionName(position_name) => {
                write!(
                    f,
                    "position_name: illegal charcter included: {} (binary: {:X?})",
                    position_name,
                    position_name.as_bytes().to_vec()
                )
            }
            CareerValidationError::InvalidNoteLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid note length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            CareerValidationError::IllegalCharInNote(note) => {
                write!(
                    f,
                    "note: illegal charcter included: {} (binary: {:X?})",
                    note, note.as_bytes().to_vec()
                )
            }
        }
    }
}

impl Error for CareerValidationError {}

#[cfg(test)]
mod tests {
    use common::util::{Career, Ymd};

    use super::validate_career;

    #[test]
    fn validate_career_returns_ok_if_valid_career_is_passed() {
        let career = Career {
            company_name: String::from("田中自動車"),
            department_name: Some(String::from("開発部　第一開発部")),
            office: Some(String::from("名古屋事業所")),
            career_start_date: Ymd {
                year: 2006,
                month: 4,
                day: 1,
            },
            career_end_date: Some(Ymd {
                year: 2016,
                month: 7,
                day: 1,
            }),
            contract_type: String::from("regular"),
            profession: Some(String::from("ITエンジニア")),
            annual_income_in_man_yen: Some(800),
            is_manager: false,
            position_name: Some(String::from("主任")),
            is_new_graduate: true,
            note: Some(String::from("田中自動車の名古屋事業所で１０年ほどエンジン制御のソフトウェア開発に携わってきました。そのため、下記の点についてご相談を受け付けられるかと思います。
            ・田中自動車の給与、福利厚生
            ・田中自動車　開発部の雰囲気
            ・名古屋事業所での働きやすさ
            ・田中自動車での組み込みエンジニアの仕事について")),
        };

        let _ = validate_career(&career).expect("failed to get Ok");
    }

    #[test]
    fn validate_career_returns_ok_if_valid_career_with_only_mandatory_input_is_passed() {
        let career = Career {
            company_name: String::from("田中自動車"),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2006,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: String::from("regular"),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: None,
        };

        let _ = validate_career(&career).expect("failed to get Ok");
    }

    #[test]
    fn validate_career_returns_ok_if_1_char_company_name_is_passed() {
        let career = Career {
            company_name: String::from("あ"),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2006,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: String::from("regular"),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: None,
        };

        let _ = validate_career(&career).expect("failed to get Ok");
    }
}
