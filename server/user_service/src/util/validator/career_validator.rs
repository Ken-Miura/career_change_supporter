// Copyright 2022 Ken Miura

use std::{collections::HashSet, error::Error, fmt::Display};

use chrono::NaiveDate;
use common::util::validator::{has_control_char, SPACE_RE, SYMBOL_CHAR_RE};
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
        }
    }
}

impl Error for CareerValidationError {}

#[cfg(test)]
mod tests {}
