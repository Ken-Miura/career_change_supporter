// Copyright 2021 Ken Miura

use std::{collections::HashSet, error::Error, fmt::Display};

use common::util::validator::{has_control_char, SYMBOL_CHAR_RE};
use common::util::Career;
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

/// Error related to [validate_career()]
#[derive(Debug, PartialEq)]
pub(crate) enum CareerValidationError {
    InvalidCompanyNameLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInCompanyName(String),
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
        }
    }
}

impl Error for CareerValidationError {}

#[cfg(test)]
mod tests {}
