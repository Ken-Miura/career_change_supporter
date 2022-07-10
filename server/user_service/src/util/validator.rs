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
}
