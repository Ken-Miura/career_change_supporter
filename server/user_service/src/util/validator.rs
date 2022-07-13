// Copyright 2021 Ken Miura

use std::collections::HashSet;

use common::util::validator::{
    has_control_char, has_non_new_line_control_char, SPACE_RE, SYMBOL_CHAR_RE,
};
use once_cell::sync::Lazy;

pub(crate) mod bank_account_validator;
pub(crate) mod career_validator;
pub(crate) mod consultant_search_param;
pub(crate) mod file_name_validator;
pub(crate) mod identity_validator;

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

/// 99999万円（9億9999万円）が最大値
pub(crate) const MAX_ANNUAL_INCOME_IN_MAN_YEN: i32 = 99999;

static CONTRACT_TYPE_SET: Lazy<HashSet<String>> = Lazy::new(|| {
    let mut set: HashSet<String> = HashSet::with_capacity(3);
    set.insert("regular".to_string());
    set.insert("contract".to_string());
    set.insert("other".to_string());
    set
});

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
fn validate_company_name(company_name: &str) -> Result<(), CompanyNameValidationError> {
    let company_name_length = company_name.chars().count();
    if !(COMPANY_NAME_MIN_LENGTH..=COMPANY_NAME_MAX_LENGTH).contains(&company_name_length) {
        return Err(CompanyNameValidationError::InvalidCompanyNameLength {
            length: company_name_length,
            min_length: COMPANY_NAME_MIN_LENGTH,
            max_length: COMPANY_NAME_MAX_LENGTH,
        });
    }
    if has_control_char(company_name) {
        return Err(CompanyNameValidationError::IllegalCharInCompanyName(
            company_name.to_string(),
        ));
    }
    if SYMBOL_CHAR_RE.is_match(company_name) {
        return Err(CompanyNameValidationError::IllegalCharInCompanyName(
            company_name.to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
enum CompanyNameValidationError {
    InvalidCompanyNameLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInCompanyName(String),
}

fn validate_department_name(department_name: &str) -> Result<(), DepartmentNameValidationError> {
    let department_name_length = department_name.chars().count();
    if !(DEPARTMENT_NAME_MIN_LENGTH..=DEPARTMENT_NAME_MAX_LENGTH).contains(&department_name_length)
    {
        return Err(DepartmentNameValidationError::InvalidDepartmentNameLength {
            length: department_name_length,
            min_length: DEPARTMENT_NAME_MIN_LENGTH,
            max_length: DEPARTMENT_NAME_MAX_LENGTH,
        });
    }
    if has_control_char(department_name) {
        return Err(DepartmentNameValidationError::IllegalCharInDepartmentName(
            department_name.to_string(),
        ));
    }
    if SYMBOL_CHAR_RE.is_match(department_name) {
        return Err(DepartmentNameValidationError::IllegalCharInDepartmentName(
            department_name.to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
enum DepartmentNameValidationError {
    InvalidDepartmentNameLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInDepartmentName(String),
}

fn validate_office(office: &str) -> Result<(), OfficeValidationError> {
    let office_length = office.chars().count();
    if !(OFFICE_MIN_LENGTH..=OFFICE_MAX_LENGTH).contains(&office_length) {
        return Err(OfficeValidationError::InvalidOfficeLength {
            length: office_length,
            min_length: OFFICE_MIN_LENGTH,
            max_length: OFFICE_MAX_LENGTH,
        });
    }
    if has_control_char(office) {
        return Err(OfficeValidationError::IllegalCharInOffice(
            office.to_string(),
        ));
    }
    if SYMBOL_CHAR_RE.is_match(office) || SPACE_RE.is_match(office) {
        return Err(OfficeValidationError::IllegalCharInOffice(
            office.to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
enum OfficeValidationError {
    InvalidOfficeLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInOffice(String),
}

fn validate_contract_type(contract_type: &str) -> Result<(), ContractTypeValidationError> {
    if !CONTRACT_TYPE_SET.contains(contract_type) {
        return Err(ContractTypeValidationError::IllegalContractType(
            contract_type.to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
enum ContractTypeValidationError {
    IllegalContractType(String),
}

fn validate_profession(profession: &str) -> Result<(), ProfessionValidationError> {
    let profession_length = profession.chars().count();
    if !(PROFESSION_MIN_LENGTH..=PROFESSION_MAX_LENGTH).contains(&profession_length) {
        return Err(ProfessionValidationError::InvalidProfessionLength {
            length: profession_length,
            min_length: PROFESSION_MIN_LENGTH,
            max_length: PROFESSION_MAX_LENGTH,
        });
    }
    if has_control_char(profession) {
        return Err(ProfessionValidationError::IllegalCharInProfession(
            profession.to_string(),
        ));
    }
    if SYMBOL_CHAR_RE.is_match(profession) || SPACE_RE.is_match(profession) {
        return Err(ProfessionValidationError::IllegalCharInProfession(
            profession.to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
enum ProfessionValidationError {
    InvalidProfessionLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInProfession(String),
}

fn validate_annual_income_in_man_yen(
    annual_income_in_man_yen: i32,
) -> Result<(), AnnualIncomInManYenValidationError> {
    if annual_income_in_man_yen.is_negative() {
        return Err(
            AnnualIncomInManYenValidationError::IllegalAnnualIncomInManYen(
                annual_income_in_man_yen,
            ),
        );
    }
    if annual_income_in_man_yen > MAX_ANNUAL_INCOME_IN_MAN_YEN {
        return Err(
            AnnualIncomInManYenValidationError::IllegalAnnualIncomInManYen(
                annual_income_in_man_yen,
            ),
        );
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
enum AnnualIncomInManYenValidationError {
    IllegalAnnualIncomInManYen(i32),
}

fn validate_position_name(position_name: &str) -> Result<(), PositionNameValidationError> {
    let position_name_length = position_name.chars().count();
    if !(POSITION_NAME_MIN_LENGTH..=POSITION_NAME_MAX_LENGTH).contains(&position_name_length) {
        return Err(PositionNameValidationError::InvalidPositionNameLength {
            length: position_name_length,
            min_length: POSITION_NAME_MIN_LENGTH,
            max_length: POSITION_NAME_MAX_LENGTH,
        });
    }
    if has_control_char(position_name) {
        return Err(PositionNameValidationError::IllegalCharInPositionName(
            position_name.to_string(),
        ));
    }
    if SYMBOL_CHAR_RE.is_match(position_name) || SPACE_RE.is_match(position_name) {
        return Err(PositionNameValidationError::IllegalCharInPositionName(
            position_name.to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
enum PositionNameValidationError {
    InvalidPositionNameLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInPositionName(String),
}

fn validate_note(note: &str) -> Result<(), NoteValidationError> {
    let note_length = note.chars().count();
    if !(NOTE_MIN_LENGTH..=NOTE_MAX_LENGTH).contains(&note_length) {
        return Err(NoteValidationError::InvalidNoteLength {
            length: note_length,
            min_length: NOTE_MIN_LENGTH,
            max_length: NOTE_MAX_LENGTH,
        });
    }
    if has_non_new_line_control_char(note) {
        return Err(NoteValidationError::IllegalCharInNote(note.to_string()));
    }
    if SYMBOL_CHAR_RE.is_match(note) {
        return Err(NoteValidationError::IllegalCharInNote(note.to_string()));
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
enum NoteValidationError {
    InvalidNoteLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInNote(String),
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use once_cell::sync::Lazy;

    use crate::util::validator::{
        AnnualIncomInManYenValidationError, CompanyNameValidationError,
        ContractTypeValidationError, DepartmentNameValidationError, NoteValidationError,
        OfficeValidationError, PositionNameValidationError, ProfessionValidationError,
        COMPANY_NAME_MAX_LENGTH, COMPANY_NAME_MIN_LENGTH, DEPARTMENT_NAME_MAX_LENGTH,
        DEPARTMENT_NAME_MIN_LENGTH, NOTE_MAX_LENGTH, NOTE_MIN_LENGTH, OFFICE_MAX_LENGTH,
        OFFICE_MIN_LENGTH, POSITION_NAME_MAX_LENGTH, POSITION_NAME_MIN_LENGTH,
        PROFESSION_MAX_LENGTH, PROFESSION_MIN_LENGTH,
    };

    use super::{
        validate_annual_income_in_man_yen, validate_company_name, validate_contract_type,
        validate_department_name, validate_note, validate_office, validate_position_name,
        validate_profession, CONTRACT_TYPE_SET, MAX_ANNUAL_INCOME_IN_MAN_YEN,
    };

    pub(in crate::util::validator) static SYMBOL_SET: Lazy<HashSet<String>> = Lazy::new(|| {
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

    pub(in crate::util::validator) static CONTROL_CHAR_SET: Lazy<HashSet<String>> =
        Lazy::new(|| {
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

    pub(in crate::util::validator) static NON_NEW_LINE_CONTROL_CHAR_SET: Lazy<HashSet<String>> =
        Lazy::new(|| {
            let mut set: HashSet<String> = HashSet::with_capacity(32 - 2 + 1 + 32);
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
            // set.insert('\u{000A}'.to_string());
            set.insert('\u{000B}'.to_string());
            set.insert('\u{000C}'.to_string());
            // set.insert('\u{000D}'.to_string());
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

    pub(in crate::util::validator) static NEW_LINE_CONTROL_CHAR_SET: Lazy<HashSet<String>> =
        Lazy::new(|| {
            let mut set: HashSet<String> = HashSet::with_capacity(2);
            set.insert('\u{000A}'.to_string());
            set.insert('\u{000D}'.to_string());
            set
        });

    pub(in crate::util::validator) static SPACE_SET: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(2);
        // 半角スペース
        set.insert(" ".to_string());
        // 全角スペース
        set.insert("　".to_string());
        set
    });

    pub(in crate::util::validator) static NUMBER_SET: Lazy<HashSet<String>> = Lazy::new(|| {
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

    #[test]
    fn validate_company_name_returns_ok_if_1_char_company_name_is_passed() {
        let company_name = "あ";
        let _ = validate_company_name(company_name).expect("failed to get Ok");
    }

    #[test]
    fn validate_company_name_returns_ok_if_256_char_company_name_is_passed() {
        let company_name = "ああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ";
        let _ = validate_company_name(company_name).expect("failed to get Ok");
    }

    #[test]
    fn validate_company_name_returns_err_if_empty_char_company_name_is_passed() {
        let company_name = "";

        let result = validate_company_name(company_name).expect_err("failed to get Err");

        assert_eq!(
            CompanyNameValidationError::InvalidCompanyNameLength {
                length: company_name.chars().count(),
                min_length: COMPANY_NAME_MIN_LENGTH,
                max_length: COMPANY_NAME_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_company_name_returns_err_if_257_char_company_name_is_passed() {
        let company_name = "あああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ";

        let result = validate_company_name(company_name).expect_err("failed to get Err");

        assert_eq!(
            CompanyNameValidationError::InvalidCompanyNameLength {
                length: company_name.chars().count(),
                min_length: COMPANY_NAME_MIN_LENGTH,
                max_length: COMPANY_NAME_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_company_name_returns_err_if_company_name_is_control_char() {
        let mut company_names = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            company_names.push(s.to_string());
        }
        for company_name in company_names {
            let err = validate_company_name(company_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                CompanyNameValidationError::IllegalCharInCompanyName(company_name),
                err
            );
        }
    }

    #[test]
    fn validate_company_name_returns_err_if_company_name_starts_with_control_char() {
        let mut company_names = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            company_names.push(s.to_string() + "山田工業");
        }
        for company_name in company_names {
            let err = validate_company_name(company_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                CompanyNameValidationError::IllegalCharInCompanyName(company_name),
                err
            );
        }
    }

    #[test]
    fn validate_company_name_returns_err_if_company_name_ends_with_control_char() {
        let mut company_names = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            company_names.push("山田工業".to_string() + s);
        }
        for company_name in company_names {
            let err = validate_company_name(company_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                CompanyNameValidationError::IllegalCharInCompanyName(company_name),
                err
            );
        }
    }

    #[test]
    fn validate_company_name_returns_err_if_company_name_includes_control_char() {
        let mut company_names = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            company_names.push("山田".to_string() + s + "工業");
        }
        for company_name in company_names {
            let err = validate_company_name(company_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                CompanyNameValidationError::IllegalCharInCompanyName(company_name),
                err
            );
        }
    }

    #[test]
    fn validate_company_name_returns_err_if_company_name_is_symbol() {
        let mut company_names = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            company_names.push(s.to_string());
        }
        for company_name in company_names {
            let err = validate_company_name(company_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                CompanyNameValidationError::IllegalCharInCompanyName(company_name),
                err
            );
        }
    }

    #[test]
    fn validate_company_name_returns_err_if_company_name_starts_with_symbol() {
        let mut company_names = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            company_names.push(s.to_string() + "山田工業");
        }
        for company_name in company_names {
            let err = validate_company_name(company_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                CompanyNameValidationError::IllegalCharInCompanyName(company_name),
                err
            );
        }
    }

    #[test]
    fn validate_company_name_returns_err_if_company_name_ends_with_symbol() {
        let mut company_names = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            company_names.push("山田工業".to_string() + s);
        }
        for company_name in company_names {
            let err = validate_company_name(company_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                CompanyNameValidationError::IllegalCharInCompanyName(company_name),
                err
            );
        }
    }

    #[test]
    fn validate_company_name_returns_err_if_company_name_includes_symbol() {
        let mut company_names = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            company_names.push("山田".to_string() + s + "工業");
        }
        for company_name in company_names {
            let err = validate_company_name(company_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                CompanyNameValidationError::IllegalCharInCompanyName(company_name),
                err
            );
        }
    }

    #[test]
    fn validate_department_name_returns_ok_if_1_char_department_name_is_passed() {
        let department_name = "あ";
        let _ = validate_department_name(department_name).expect("failed to get Ok");
    }

    #[test]
    fn validate_department_name_returns_ok_if_256_char_department_name_is_passed() {
        let department_name = "ああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ";
        let _ = validate_department_name(department_name).expect("failed to get Ok");
    }

    #[test]
    fn validate_department_name_returns_err_if_empty_char_department_name_is_passed() {
        let department_name = "";

        let result = validate_department_name(department_name).expect_err("failed to get Err");

        assert_eq!(
            DepartmentNameValidationError::InvalidDepartmentNameLength {
                length: department_name.chars().count(),
                min_length: DEPARTMENT_NAME_MIN_LENGTH,
                max_length: DEPARTMENT_NAME_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_department_name_returns_err_if_257_char_department_name_is_passed() {
        let department_name = "あああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ";

        let result = validate_department_name(department_name).expect_err("failed to get Err");

        assert_eq!(
            DepartmentNameValidationError::InvalidDepartmentNameLength {
                length: department_name.chars().count(),
                min_length: DEPARTMENT_NAME_MIN_LENGTH,
                max_length: DEPARTMENT_NAME_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_department_name_returns_err_if_department_name_is_control_char() {
        let mut department_names = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let department_name = s.to_string();
            department_names.push(department_name);
        }
        for department_name in department_names {
            let err =
                validate_department_name(department_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                DepartmentNameValidationError::IllegalCharInDepartmentName(department_name),
                err
            );
        }
    }

    #[test]
    fn validate_department_name_returns_err_if_department_name_starts_with_control_char() {
        let mut department_names = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let department_name = s.to_string() + "第二営業部";
            department_names.push(department_name);
        }
        for department_name in department_names {
            let err =
                validate_department_name(department_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                DepartmentNameValidationError::IllegalCharInDepartmentName(department_name),
                err
            );
        }
    }

    #[test]
    fn validate_department_name_returns_err_if_department_name_ends_with_control_char() {
        let mut department_names = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let department_name = "第二営業部".to_string() + s;
            department_names.push(department_name);
        }
        for department_name in department_names {
            let err =
                validate_department_name(department_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                DepartmentNameValidationError::IllegalCharInDepartmentName(department_name),
                err
            );
        }
    }

    #[test]
    fn validate_department_name_returns_err_if_department_name_includes_control_char() {
        let mut department_names = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let department_name = "第二".to_string() + s + "営業部";
            department_names.push(department_name);
        }
        for department_name in department_names {
            let err =
                validate_department_name(department_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                DepartmentNameValidationError::IllegalCharInDepartmentName(department_name),
                err
            );
        }
    }

    #[test]
    fn validate_department_name_returns_err_if_department_name_is_symbol() {
        let mut department_names = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let department_name = s.to_string();
            department_names.push(department_name);
        }
        for department_name in department_names {
            let err =
                validate_department_name(department_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                DepartmentNameValidationError::IllegalCharInDepartmentName(department_name),
                err
            );
        }
    }

    #[test]
    fn validate_department_name_returns_err_if_department_name_starts_with_symbol() {
        let mut department_names = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let department_name = s.to_string() + "第二営業部";
            department_names.push(department_name);
        }
        for department_name in department_names {
            let err =
                validate_department_name(department_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                DepartmentNameValidationError::IllegalCharInDepartmentName(department_name),
                err
            );
        }
    }

    #[test]
    fn validate_department_name_returns_err_if_department_name_ends_with_symbol() {
        let mut department_names = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let department_name = "第二営業部".to_string() + s;
            department_names.push(department_name);
        }
        for department_name in department_names {
            let err =
                validate_department_name(department_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                DepartmentNameValidationError::IllegalCharInDepartmentName(department_name),
                err
            );
        }
    }

    #[test]
    fn validate_department_name_returns_err_if_department_name_includes_symbol() {
        let mut department_names = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let department_name = "第二".to_string() + s + "営業部";
            department_names.push(department_name);
        }
        for department_name in department_names {
            let err =
                validate_department_name(department_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                DepartmentNameValidationError::IllegalCharInDepartmentName(department_name),
                err
            );
        }
    }

    #[test]
    fn validate_office_returns_ok_if_1_char_office_is_passed() {
        let office = "あ";
        let _ = validate_office(office).expect("failed to get Ok");
    }

    #[test]
    fn validate_office_returns_ok_if_256_char_office_is_passed() {
        let office = "ああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ";
        let _ = validate_office(office).expect("failed to get Ok");
    }

    #[test]
    fn validate_office_returns_err_if_empty_char_office_is_passed() {
        let office = "";

        let result = validate_office(office).expect_err("failed to get Err");

        assert_eq!(
            OfficeValidationError::InvalidOfficeLength {
                length: office.chars().count(),
                min_length: OFFICE_MIN_LENGTH,
                max_length: OFFICE_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_office_returns_err_if_257_char_office_is_passed() {
        let office = "あああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ";

        let result = validate_office(office).expect_err("failed to get Err");

        assert_eq!(
            OfficeValidationError::InvalidOfficeLength {
                length: office.chars().count(),
                min_length: OFFICE_MIN_LENGTH,
                max_length: OFFICE_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_office_returns_err_if_office_is_control_char() {
        let mut offices = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let office = s.to_string();
            offices.push(office);
        }
        for office in offices {
            let err = validate_office(office.as_str()).expect_err("failed to get Err");
            assert_eq!(OfficeValidationError::IllegalCharInOffice(office), err);
        }
    }

    #[test]
    fn validate_office_returns_err_if_office_starts_with_control_char() {
        let mut offices = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let office = s.to_string() + "松山事業所";
            offices.push(office);
        }
        for office in offices {
            let err = validate_office(office.as_str()).expect_err("failed to get Err");
            assert_eq!(OfficeValidationError::IllegalCharInOffice(office), err);
        }
    }

    #[test]
    fn validate_office_returns_err_if_office_ends_with_control_char() {
        let mut offices = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let office = "松山事業所".to_string() + s;
            offices.push(office);
        }
        for office in offices {
            let err = validate_office(office.as_str()).expect_err("failed to get Err");
            assert_eq!(OfficeValidationError::IllegalCharInOffice(office), err);
        }
    }

    #[test]
    fn validate_office_returns_err_if_office_includes_control_char() {
        let mut offices = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let office = "松山".to_string() + s + "事業所";
            offices.push(office);
        }
        for office in offices {
            let err = validate_office(office.as_str()).expect_err("failed to get Err");
            assert_eq!(OfficeValidationError::IllegalCharInOffice(office), err);
        }
    }

    #[test]
    fn validate_office_returns_err_if_office_is_symbol() {
        let mut offices = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let office = s.to_string();
            offices.push(office);
        }
        for office in offices {
            let err = validate_office(office.as_str()).expect_err("failed to get Err");
            assert_eq!(OfficeValidationError::IllegalCharInOffice(office), err);
        }
    }

    #[test]
    fn validate_office_returns_err_if_office_starts_with_symbol() {
        let mut offices = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let office = s.to_string() + "松山事業所";
            offices.push(office);
        }
        for office in offices {
            let err = validate_office(office.as_str()).expect_err("failed to get Err");
            assert_eq!(OfficeValidationError::IllegalCharInOffice(office), err);
        }
    }

    #[test]
    fn validate_office_returns_err_if_office_ends_with_symbol() {
        let mut offices = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let office = "松山事業所".to_string() + s;
            offices.push(office);
        }
        for office in offices {
            let err = validate_office(office.as_str()).expect_err("failed to get Err");
            assert_eq!(OfficeValidationError::IllegalCharInOffice(office), err);
        }
    }

    #[test]
    fn validate_office_returns_err_if_office_includes_symbol() {
        let mut offices = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let office = "松山".to_string() + s + "事業所";
            offices.push(office);
        }
        for office in offices {
            let err = validate_office(office.as_str()).expect_err("failed to get Err");
            assert_eq!(OfficeValidationError::IllegalCharInOffice(office), err);
        }
    }

    #[test]
    fn validate_office_returns_err_if_office_is_space() {
        let mut offices = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let office = s.to_string();
            offices.push(office);
        }
        for office in offices {
            let err = validate_office(office.as_str()).expect_err("failed to get Err");
            assert_eq!(OfficeValidationError::IllegalCharInOffice(office), err);
        }
    }

    #[test]
    fn validate_office_returns_err_if_office_starts_with_space() {
        let mut offices = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let office = s.to_string() + "松山事業所";
            offices.push(office);
        }
        for office in offices {
            let err = validate_office(office.as_str()).expect_err("failed to get Err");
            assert_eq!(OfficeValidationError::IllegalCharInOffice(office), err);
        }
    }

    #[test]
    fn validate_office_returns_err_if_office_ends_with_space() {
        let mut offices = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let office = "松山事業所".to_string() + s;
            offices.push(office);
        }
        for office in offices {
            let err = validate_office(office.as_str()).expect_err("failed to get Err");
            assert_eq!(OfficeValidationError::IllegalCharInOffice(office), err);
        }
    }

    #[test]
    fn validate_office_returns_err_if_office_includes_space() {
        let mut offices = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let office = "松山".to_string() + s + "事業所";
            offices.push(office);
        }
        for office in offices {
            let err = validate_office(office.as_str()).expect_err("failed to get Err");
            assert_eq!(OfficeValidationError::IllegalCharInOffice(office), err);
        }
    }

    #[test]
    fn validate_contract_type_returns_ok_if_valid_contract_type_is_passed() {
        let mut contract_types = Vec::with_capacity(CONTRACT_TYPE_SET.len());
        for s in CONTRACT_TYPE_SET.iter() {
            let contract_type = s.to_string();
            contract_types.push(contract_type);
        }
        for contract_type in contract_types {
            let _ = validate_contract_type(contract_type.as_str()).expect("failed to get Ok");
        }
    }

    #[test]
    fn validate_contract_type_returns_err_if_illegal_contract_type_is_passed() {
        let contract_type = "1' or '1' = '1';--";
        let err = validate_contract_type(contract_type).expect_err("failed to get Err");
        assert_eq!(
            ContractTypeValidationError::IllegalContractType(contract_type.to_string()),
            err
        );
    }

    #[test]
    fn validate_profession_returns_ok_if_1_char_profession_is_passed() {
        let profession = "あ";
        let _ = validate_profession(profession).expect("failed to get Ok");
    }

    #[test]
    fn validate_profession_returns_ok_if_128_char_profession_is_passed() {
        let profession = "ああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ";
        let _ = validate_profession(profession).expect("failed to get Ok");
    }

    #[test]
    fn validate_profession_returns_err_if_empty_char_profession_is_passed() {
        let profession = "";

        let result = validate_profession(profession).expect_err("failed to get Err");

        assert_eq!(
            ProfessionValidationError::InvalidProfessionLength {
                length: profession.chars().count(),
                min_length: PROFESSION_MIN_LENGTH,
                max_length: PROFESSION_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_profession_returns_err_if_129_char_profession_is_passed() {
        let profession = "あああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ";

        let result = validate_profession(profession).expect_err("failed to get Err");

        assert_eq!(
            ProfessionValidationError::InvalidProfessionLength {
                length: profession.chars().count(),
                min_length: PROFESSION_MIN_LENGTH,
                max_length: PROFESSION_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_profession_returns_err_if_profession_is_control_char() {
        let mut professions = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let profession = s.to_string();
            professions.push(profession);
        }
        for profession in professions {
            let err = validate_profession(profession.as_str()).expect_err("failed to get Err");
            assert_eq!(
                ProfessionValidationError::IllegalCharInProfession(profession),
                err
            );
        }
    }

    #[test]
    fn validate_profession_returns_err_if_profession_starts_with_control_char() {
        let mut professions = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let profession = s.to_string() + "営業";
            professions.push(profession);
        }
        for profession in professions {
            let err = validate_profession(profession.as_str()).expect_err("failed to get Err");
            assert_eq!(
                ProfessionValidationError::IllegalCharInProfession(profession),
                err
            );
        }
    }

    #[test]
    fn validate_profession_returns_err_if_profession_ends_with_control_char() {
        let mut professions = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let profession = "営業".to_string() + s;
            professions.push(profession);
        }
        for profession in professions {
            let err = validate_profession(profession.as_str()).expect_err("failed to get Err");
            assert_eq!(
                ProfessionValidationError::IllegalCharInProfession(profession),
                err
            );
        }
    }

    #[test]
    fn validate_profession_returns_err_if_profession_includes_control_char() {
        let mut professions = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let profession = "営".to_string() + s + "業";
            professions.push(profession);
        }
        for profession in professions {
            let err = validate_profession(profession.as_str()).expect_err("failed to get Err");
            assert_eq!(
                ProfessionValidationError::IllegalCharInProfession(profession),
                err
            );
        }
    }

    #[test]
    fn validate_profession_returns_err_if_profession_is_symbol() {
        let mut professions = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let profession = s.to_string();
            professions.push(profession);
        }
        for profession in professions {
            let err = validate_profession(profession.as_str()).expect_err("failed to get Err");
            assert_eq!(
                ProfessionValidationError::IllegalCharInProfession(profession),
                err
            );
        }
    }

    #[test]
    fn validate_profession_returns_err_if_profession_starts_with_symbol() {
        let mut professions = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let profession = s.to_string() + "営業";
            professions.push(profession);
        }
        for profession in professions {
            let err = validate_profession(profession.as_str()).expect_err("failed to get Err");
            assert_eq!(
                ProfessionValidationError::IllegalCharInProfession(profession),
                err
            );
        }
    }

    #[test]
    fn validate_profession_returns_err_if_profession_ends_with_symbol() {
        let mut professions = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let profession = "営業".to_string() + s;
            professions.push(profession);
        }
        for profession in professions {
            let err = validate_profession(profession.as_str()).expect_err("failed to get Err");
            assert_eq!(
                ProfessionValidationError::IllegalCharInProfession(profession),
                err
            );
        }
    }

    #[test]
    fn validate_profession_returns_err_if_profession_includes_symbol() {
        let mut professions = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let profession = "営".to_string() + s + "業";
            professions.push(profession);
        }
        for profession in professions {
            let err = validate_profession(profession.as_str()).expect_err("failed to get Err");
            assert_eq!(
                ProfessionValidationError::IllegalCharInProfession(profession),
                err
            );
        }
    }

    #[test]
    fn validate_profession_returns_err_if_profession_is_space() {
        let mut professions = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let profession = s.to_string();
            professions.push(profession);
        }
        for profession in professions {
            let err = validate_profession(profession.as_str()).expect_err("failed to get Err");
            assert_eq!(
                ProfessionValidationError::IllegalCharInProfession(profession),
                err
            );
        }
    }

    #[test]
    fn validate_profession_returns_err_if_profession_starts_with_space() {
        let mut professions = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let profession = s.to_string() + "営業";
            professions.push(profession);
        }
        for profession in professions {
            let err = validate_profession(profession.as_str()).expect_err("failed to get Err");
            assert_eq!(
                ProfessionValidationError::IllegalCharInProfession(profession),
                err
            );
        }
    }

    #[test]
    fn validate_profession_returns_err_if_profession_ends_with_space() {
        let mut professions = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let profession = "営業".to_string() + s;
            professions.push(profession);
        }
        for profession in professions {
            let err = validate_profession(profession.as_str()).expect_err("failed to get Err");
            assert_eq!(
                ProfessionValidationError::IllegalCharInProfession(profession),
                err
            );
        }
    }

    #[test]
    fn validate_profession_returns_err_if_profession_includes_space() {
        let mut professions = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let profession = "営".to_string() + s + "業";
            professions.push(profession);
        }
        for profession in professions {
            let err = validate_profession(profession.as_str()).expect_err("failed to get Err");
            assert_eq!(
                ProfessionValidationError::IllegalCharInProfession(profession),
                err
            );
        }
    }

    #[test]
    fn validate_annual_income_in_man_yen_returns_ok_if_annual_imcom_in_man_yen_0_is_passed() {
        let annual_income_in_man_yen = 0;
        let _ =
            validate_annual_income_in_man_yen(annual_income_in_man_yen).expect("failed to get Ok");
    }

    #[test]
    fn validate_annual_income_in_man_yen_returns_ok_if_max_annual_imcom_in_man_yen_is_passed() {
        let annual_income_in_man_yen = MAX_ANNUAL_INCOME_IN_MAN_YEN;
        let _ =
            validate_annual_income_in_man_yen(annual_income_in_man_yen).expect("failed to get Ok");
    }

    #[test]
    fn validate_annual_income_in_man_yen_returns_err_if_over_max_annual_imcom_in_man_yen_is_passed()
    {
        let annual_income_in_man_yen = MAX_ANNUAL_INCOME_IN_MAN_YEN + 1;

        let err = validate_annual_income_in_man_yen(annual_income_in_man_yen)
            .expect_err("failed to get Err");

        assert_eq!(
            AnnualIncomInManYenValidationError::IllegalAnnualIncomInManYen(
                annual_income_in_man_yen
            ),
            err
        );
    }

    #[test]
    fn validate_annual_income_in_man_yen_returns_err_if_negative_annual_imcom_in_man_yen_is_passed()
    {
        let annual_income_in_man_yen = -1;

        let err = validate_annual_income_in_man_yen(annual_income_in_man_yen)
            .expect_err("failed to get Err");

        assert_eq!(
            AnnualIncomInManYenValidationError::IllegalAnnualIncomInManYen(
                annual_income_in_man_yen
            ),
            err
        );
    }

    #[test]
    fn validate_annual_income_in_man_yen_returns_err_if_i32_min_annual_imcom_in_man_yen_is_passed()
    {
        let annual_income_in_man_yen = i32::MIN;

        let err = validate_annual_income_in_man_yen(annual_income_in_man_yen)
            .expect_err("failed to get Err");

        assert_eq!(
            AnnualIncomInManYenValidationError::IllegalAnnualIncomInManYen(
                annual_income_in_man_yen
            ),
            err
        );
    }

    #[test]
    fn validate_position_name_returns_ok_if_1_char_position_name_is_passed() {
        let position_name = "あ";
        let _ = validate_position_name(position_name).expect("failed to get Ok");
    }

    #[test]
    fn validate_position_name_returns_ok_if_128_char_position_name_is_passed() {
        let position_name = "ああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ";
        let _ = validate_position_name(position_name).expect("failed to get Ok");
    }

    #[test]
    fn validate_position_name_returns_err_if_empty_char_position_name_is_passed() {
        let position_name = "";

        let result = validate_position_name(position_name).expect_err("failed to get Err");

        assert_eq!(
            PositionNameValidationError::InvalidPositionNameLength {
                length: position_name.chars().count(),
                min_length: POSITION_NAME_MIN_LENGTH,
                max_length: POSITION_NAME_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_position_name_returns_err_if_129_char_position_name_is_passed() {
        let position_name = "あああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ";

        let result = validate_position_name(position_name).expect_err("failed to get Err");

        assert_eq!(
            PositionNameValidationError::InvalidPositionNameLength {
                length: position_name.chars().count(),
                min_length: POSITION_NAME_MIN_LENGTH,
                max_length: POSITION_NAME_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_position_name_returns_err_if_position_name_is_control_char() {
        let mut position_name_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let position_name = s.to_string();
            position_name_list.push(position_name);
        }
        for position_name in position_name_list {
            let err =
                validate_position_name(position_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                PositionNameValidationError::IllegalCharInPositionName(position_name),
                err
            );
        }
    }

    #[test]
    fn validate_position_name_returns_err_if_position_name_starts_with_control_char() {
        let mut position_name_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let position_name = s.to_string() + "係長";
            position_name_list.push(position_name);
        }
        for position_name in position_name_list {
            let err =
                validate_position_name(position_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                PositionNameValidationError::IllegalCharInPositionName(position_name),
                err
            );
        }
    }

    #[test]
    fn validate_position_name_returns_err_if_position_name_ends_with_control_char() {
        let mut position_name_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let position_name = "係長".to_string() + s;
            position_name_list.push(position_name);
        }
        for position_name in position_name_list {
            let err =
                validate_position_name(position_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                PositionNameValidationError::IllegalCharInPositionName(position_name),
                err
            );
        }
    }

    #[test]
    fn validate_position_name_returns_err_if_position_name_includes_control_char() {
        let mut position_name_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let position_name = "係".to_string() + s + "長";
            position_name_list.push(position_name);
        }
        for position_name in position_name_list {
            let err =
                validate_position_name(position_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                PositionNameValidationError::IllegalCharInPositionName(position_name),
                err
            );
        }
    }

    #[test]
    fn validate_position_name_returns_err_if_position_name_is_symbol() {
        let mut position_name_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let position_name = s.to_string();
            position_name_list.push(position_name);
        }
        for position_name in position_name_list {
            let err =
                validate_position_name(position_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                PositionNameValidationError::IllegalCharInPositionName(position_name),
                err
            );
        }
    }

    #[test]
    fn validate_position_name_returns_err_if_position_name_starts_with_symbol() {
        let mut position_name_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let position_name = s.to_string() + "係長";
            position_name_list.push(position_name);
        }
        for position_name in position_name_list {
            let err =
                validate_position_name(position_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                PositionNameValidationError::IllegalCharInPositionName(position_name),
                err
            );
        }
    }

    #[test]
    fn validate_position_name_returns_err_if_position_name_ends_with_symbol() {
        let mut position_name_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let position_name = "係長".to_string() + s;
            position_name_list.push(position_name);
        }
        for position_name in position_name_list {
            let err =
                validate_position_name(position_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                PositionNameValidationError::IllegalCharInPositionName(position_name),
                err
            );
        }
    }

    #[test]
    fn validate_position_name_returns_err_if_position_name_includes_symbol() {
        let mut position_name_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let position_name = "係".to_string() + s + "長";
            position_name_list.push(position_name);
        }
        for position_name in position_name_list {
            let err =
                validate_position_name(position_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                PositionNameValidationError::IllegalCharInPositionName(position_name),
                err
            );
        }
    }

    #[test]
    fn validate_position_name_returns_err_if_position_name_is_space() {
        let mut position_name_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let position_name = s.to_string();
            position_name_list.push(position_name);
        }
        for position_name in position_name_list {
            let err =
                validate_position_name(position_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                PositionNameValidationError::IllegalCharInPositionName(position_name),
                err
            );
        }
    }

    #[test]
    fn validate_position_name_returns_err_if_position_name_starts_with_space() {
        let mut position_name_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let position_name = s.to_string() + "係長";
            position_name_list.push(position_name);
        }
        for position_name in position_name_list {
            let err =
                validate_position_name(position_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                PositionNameValidationError::IllegalCharInPositionName(position_name),
                err
            );
        }
    }

    #[test]
    fn validate_position_name_returns_err_if_position_name_ends_with_space() {
        let mut position_name_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let position_name = "係長".to_string() + s;
            position_name_list.push(position_name);
        }
        for position_name in position_name_list {
            let err =
                validate_position_name(position_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                PositionNameValidationError::IllegalCharInPositionName(position_name),
                err
            );
        }
    }

    #[test]
    fn validate_position_name_returns_err_if_position_name_includes_space() {
        let mut position_name_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let position_name = "係".to_string() + s + "長";
            position_name_list.push(position_name);
        }
        for position_name in position_name_list {
            let err =
                validate_position_name(position_name.as_str()).expect_err("failed to get Err");
            assert_eq!(
                PositionNameValidationError::IllegalCharInPositionName(position_name),
                err
            );
        }
    }

    #[test]
    fn validate_note_returns_ok_if_1_char_note_is_passed() {
        let note = "あ";
        let _ = validate_note(note).expect("failed to get Ok");
    }

    #[test]
    fn validate_note_returns_ok_if_2048_char_note_is_passed() {
        let note = "ああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ";
        let _ = validate_note(note).expect("failed to get Ok");
    }

    #[test]
    fn validate_note_returns_err_if_empty_char_note_is_passed() {
        let note = "";

        let result = validate_note(note).expect_err("failed to get Err");

        assert_eq!(
            NoteValidationError::InvalidNoteLength {
                length: note.chars().count(),
                min_length: NOTE_MIN_LENGTH,
                max_length: NOTE_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_note_returns_err_if_2045_char_note_is_passed() {
        let note = "あああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ";

        let result = validate_note(note).expect_err("failed to get Err");

        assert_eq!(
            NoteValidationError::InvalidNoteLength {
                length: note.chars().count(),
                min_length: NOTE_MIN_LENGTH,
                max_length: NOTE_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_note_returns_err_if_note_is_non_new_line_control_char() {
        let mut note_list = Vec::with_capacity(NON_NEW_LINE_CONTROL_CHAR_SET.len());
        for s in NON_NEW_LINE_CONTROL_CHAR_SET.iter() {
            let note = s.to_string();
            note_list.push(note);
        }
        for note in note_list {
            let err = validate_note(note.as_str()).expect_err("failed to get Err");
            assert_eq!(NoteValidationError::IllegalCharInNote(note), err);
        }
    }

    #[test]
    fn validate_note_returns_err_if_note_starts_with_non_new_line_control_char() {
        let mut note_list = Vec::with_capacity(NON_NEW_LINE_CONTROL_CHAR_SET.len());
        for s in NON_NEW_LINE_CONTROL_CHAR_SET.iter() {
            let note = s.to_string() + "備考";
            note_list.push(note);
        }
        for note in note_list {
            let err = validate_note(note.as_str()).expect_err("failed to get Err");
            assert_eq!(NoteValidationError::IllegalCharInNote(note), err);
        }
    }

    #[test]
    fn validate_note_returns_err_if_note_ends_with_non_new_line_control_char() {
        let mut note_list = Vec::with_capacity(NON_NEW_LINE_CONTROL_CHAR_SET.len());
        for s in NON_NEW_LINE_CONTROL_CHAR_SET.iter() {
            let note = "備考".to_string() + s;
            note_list.push(note);
        }
        for note in note_list {
            let err = validate_note(note.as_str()).expect_err("failed to get Err");
            assert_eq!(NoteValidationError::IllegalCharInNote(note), err);
        }
    }

    #[test]
    fn validate_note_returns_err_if_note_includes_non_new_line_control_char() {
        let mut note_list = Vec::with_capacity(NON_NEW_LINE_CONTROL_CHAR_SET.len());
        for s in NON_NEW_LINE_CONTROL_CHAR_SET.iter() {
            let note = "備".to_string() + s + "考";
            note_list.push(note);
        }
        for note in note_list {
            let err = validate_note(note.as_str()).expect_err("failed to get Err");
            assert_eq!(NoteValidationError::IllegalCharInNote(note), err);
        }
    }

    #[test]
    fn validate_note_returns_ok_if_note_is_new_line_control_char() {
        let mut note_list = Vec::with_capacity(NEW_LINE_CONTROL_CHAR_SET.len());
        for s in NEW_LINE_CONTROL_CHAR_SET.iter() {
            let note = s.to_string();
            note_list.push(note);
        }
        for note in note_list {
            let _ = validate_note(note.as_str()).expect("failed to get Ok");
        }
    }

    #[test]
    fn validate_note_returns_ok_if_note_is_space() {
        let mut note_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let note = s.to_string();
            note_list.push(note);
        }
        for note in note_list {
            let _ = validate_note(note.as_str()).expect("failed to get Ok");
        }
    }

    #[test]
    fn validate_note_returns_ok_if_note_includes_space_and_new_line() {
        let note = "備考は、
            
        改行や\n
         　空白を\r
         受け入れます。\r\n
         
         ";
        let _ = validate_note(note).expect("failed to get Ok");
    }

    #[test]
    fn validate_note_returns_err_if_note_is_symbol() {
        let mut note_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let note = s.to_string();
            note_list.push(note);
        }
        for note in note_list {
            let err = validate_note(note.as_str()).expect_err("failed to get Err");
            assert_eq!(NoteValidationError::IllegalCharInNote(note), err);
        }
    }

    #[test]
    fn validate_note_returns_err_if_note_starts_with_symbol() {
        let mut note_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let note = s.to_string() + "備考";
            note_list.push(note);
        }
        for note in note_list {
            let err = validate_note(note.as_str()).expect_err("failed to get Err");
            assert_eq!(NoteValidationError::IllegalCharInNote(note), err);
        }
    }

    #[test]
    fn validate_note_returns_err_if_note_ends_with_symbol() {
        let mut note_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let note = "備考".to_string() + s;
            note_list.push(note);
        }
        for note in note_list {
            let err = validate_note(note.as_str()).expect_err("failed to get Err");
            assert_eq!(NoteValidationError::IllegalCharInNote(note), err);
        }
    }

    #[test]
    fn validate_note_returns_err_if_note_includes_symbol() {
        let mut note_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let note = "備".to_string() + s + "考";
            note_list.push(note);
        }
        for note in note_list {
            let err = validate_note(note.as_str()).expect_err("failed to get Err");
            assert_eq!(NoteValidationError::IllegalCharInNote(note), err);
        }
    }
}
