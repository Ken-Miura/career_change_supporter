// Copyright 2022 Ken Miura

use std::{error::Error, fmt::Display};

use chrono::NaiveDate;
use common::util::{Career, Ymd};

pub(super) fn validate_career(career: &Career) -> Result<(), CareerValidationError> {
    validate_company_name(&career.company_name)?;
    if let Some(department_name) = career.department_name.clone() {
        validate_department_name(department_name.as_str())?;
    }
    if let Some(office) = career.office.clone() {
        validate_office(office.as_str())?;
    }
    validate_career_start_date(&career.career_start_date)?;
    if let Some(career_end_date) = career.career_end_date.clone() {
        validate_career_end_date(&career_end_date)?;
        ensure_career_start_date_does_not_exceed_career_end_date(
            &career.career_start_date,
            &career_end_date,
        )?;
    }
    validate_contract_type(&career.contract_type)?;
    if let Some(profession) = career.profession.clone() {
        validate_profession(profession.as_str())?;
    }
    if let Some(annual_income_in_man_yen) = career.annual_income_in_man_yen {
        validate_annual_income_in_man_yen(annual_income_in_man_yen)?;
    }
    if let Some(position_name) = career.position_name.clone() {
        validate_position_name(position_name.as_str())?;
    }
    if let Some(note) = career.note.clone() {
        validate_note(note.as_str())?;
    }
    Ok(())
}

fn validate_company_name(company_name: &str) -> Result<(), CareerValidationError> {
    crate::handlers::authenticated_handlers::personal_info::profile::career::validate_company_name(company_name).map_err(|e| match e {
        crate::handlers::authenticated_handlers::personal_info::profile::career::CompanyNameValidationError::InvalidCompanyNameLength {
            length,
            min_length,
            max_length,
        } => CareerValidationError::InvalidCompanyNameLength {
            length,
            min_length,
            max_length,
        },
        crate::handlers::authenticated_handlers::personal_info::profile::career::CompanyNameValidationError::IllegalCharInCompanyName(
            company_name,
        ) => CareerValidationError::IllegalCharInCompanyName(company_name),
    })?;
    Ok(())
}

fn validate_department_name(department_name: &str) -> Result<(), CareerValidationError> {
    crate::handlers::authenticated_handlers::personal_info::profile::career::validate_department_name(department_name).map_err(|e| match e {
        crate::handlers::authenticated_handlers::personal_info::profile::career::DepartmentNameValidationError::InvalidDepartmentNameLength {
            length,
            min_length,
            max_length,
        } => CareerValidationError::InvalidDepartmentNameLength {
            length,
            min_length,
            max_length,
        },
        crate::handlers::authenticated_handlers::personal_info::profile::career::DepartmentNameValidationError::IllegalCharInDepartmentName(
            department_name,
        ) => CareerValidationError::IllegalCharInDepartmentName(department_name),
    })?;
    Ok(())
}

fn validate_office(office: &str) -> Result<(), CareerValidationError> {
    crate::handlers::authenticated_handlers::personal_info::profile::career::validate_office(office).map_err(|e| match e {
        crate::handlers::authenticated_handlers::personal_info::profile::career::OfficeValidationError::InvalidOfficeLength {
            length,
            min_length,
            max_length,
        } => CareerValidationError::InvalidOfficeLength {
            length,
            min_length,
            max_length,
        },
        crate::handlers::authenticated_handlers::personal_info::profile::career::OfficeValidationError::IllegalCharInOffice(office) => {
            CareerValidationError::IllegalCharInOffice(office)
        }
    })?;
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
    let start_date = NaiveDate::from_ymd_opt(
        career_start_date.year,
        career_start_date.month,
        career_start_date.day,
    )
    .ok_or_else(
        || CareerValidationError::CareerStartDateExceedsCareerEndDate {
            career_start_date: career_start_date.clone(),
            career_end_date: career_end_date.clone(),
        },
    )?;
    let end_date = NaiveDate::from_ymd_opt(
        career_end_date.year,
        career_end_date.month,
        career_end_date.day,
    )
    .ok_or_else(
        || CareerValidationError::CareerStartDateExceedsCareerEndDate {
            career_start_date: career_start_date.clone(),
            career_end_date: career_end_date.clone(),
        },
    )?;
    if start_date > end_date {
        return Err(CareerValidationError::CareerStartDateExceedsCareerEndDate {
            career_start_date: career_start_date.clone(),
            career_end_date: career_end_date.clone(),
        });
    }
    Ok(())
}

fn validate_contract_type(contract_type: &str) -> Result<(), CareerValidationError> {
    crate::handlers::authenticated_handlers::personal_info::profile::career::validate_contract_type(contract_type).map_err(|e| match e {
        crate::handlers::authenticated_handlers::personal_info::profile::career::ContractTypeValidationError::IllegalContractType(contract_type) => {
            CareerValidationError::IllegalContractType(contract_type)
        }
    })?;
    Ok(())
}

fn validate_profession(profession: &str) -> Result<(), CareerValidationError> {
    crate::handlers::authenticated_handlers::personal_info::profile::career::validate_profession(profession).map_err(|e| match e {
        crate::handlers::authenticated_handlers::personal_info::profile::career::ProfessionValidationError::InvalidProfessionLength {
            length,
            min_length,
            max_length,
        } => CareerValidationError::InvalidProfessionLength {
            length,
            min_length,
            max_length,
        },
        crate::handlers::authenticated_handlers::personal_info::profile::career::ProfessionValidationError::IllegalCharInProfession(profession) => {
            CareerValidationError::IllegalCharInProfession(profession)
        }
    })?;
    Ok(())
}

fn validate_annual_income_in_man_yen(
    annual_income_in_man_yen: i32,
) -> Result<(), CareerValidationError> {
    crate::handlers::authenticated_handlers::personal_info::profile::career::validate_annual_income_in_man_yen(annual_income_in_man_yen).map_err(|e| match e {
        crate::handlers::authenticated_handlers::personal_info::profile::career::AnnualIncomInManYenValidationError::IllegalAnnualIncomeInManYen(annual_income_in_man_yen) => CareerValidationError::IllegalAnnualIncomeInManYen(annual_income_in_man_yen),
    })?;
    Ok(())
}

fn validate_position_name(position_name: &str) -> Result<(), CareerValidationError> {
    crate::handlers::authenticated_handlers::personal_info::profile::career::validate_position_name(position_name).map_err(|e| match e {
        crate::handlers::authenticated_handlers::personal_info::profile::career::PositionNameValidationError::InvalidPositionNameLength {
            length,
            min_length,
            max_length,
        } => CareerValidationError::InvalidPositionNameLength {
            length,
            min_length,
            max_length,
        },
        crate::handlers::authenticated_handlers::personal_info::profile::career::PositionNameValidationError::IllegalCharInPositionName(
            position_name,
        ) => CareerValidationError::IllegalCharInPositionName(position_name),
    })?;
    Ok(())
}

fn validate_note(note: &str) -> Result<(), CareerValidationError> {
    crate::handlers::authenticated_handlers::personal_info::profile::career::validate_note(note).map_err(|e| match e {
        crate::handlers::authenticated_handlers::personal_info::profile::career::NoteValidationError::InvalidNoteLength {
            length,
            min_length,
            max_length,
        } => CareerValidationError::InvalidNoteLength {
            length,
            min_length,
            max_length,
        },
        crate::handlers::authenticated_handlers::personal_info::profile::career::NoteValidationError::IllegalCharInNote(note) => {
            CareerValidationError::IllegalCharInNote(note)
        }
    })?;
    Ok(())
}

/// Error related to [validate_career()]
#[derive(Debug, PartialEq)]
pub(super) enum CareerValidationError {
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
    IllegalAnnualIncomeInManYen(i32),
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
            CareerValidationError::IllegalAnnualIncomeInManYen(annual_income_in_man_yen) => {
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

    use crate::handlers::authenticated_handlers::{
        personal_info::profile::career::{
            career_validator::CareerValidationError, COMPANY_NAME_MAX_LENGTH,
            COMPANY_NAME_MIN_LENGTH, CONTRACT_TYPE_SET, DEPARTMENT_NAME_MAX_LENGTH,
            DEPARTMENT_NAME_MIN_LENGTH, MAX_ANNUAL_INCOME_IN_MAN_YEN, NOTE_MAX_LENGTH,
            NOTE_MIN_LENGTH, OFFICE_MAX_LENGTH, OFFICE_MIN_LENGTH, POSITION_NAME_MAX_LENGTH,
            POSITION_NAME_MIN_LENGTH, PROFESSION_MAX_LENGTH, PROFESSION_MIN_LENGTH,
        },
        tests::{
            CONTROL_CHAR_SET, NEW_LINE_CONTROL_CHAR_SET, NON_NEW_LINE_CONTROL_CHAR_SET, SPACE_SET,
            SYMBOL_SET,
        },
    };

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

        validate_career(&career).expect("failed to get Ok");
    }

    #[test]
    fn validate_career_returns_ok_if_valid_career_with_only_mandatory_input_is_passed() {
        let career = Career {
            company_name: String::from("Tanaka automotive co．"),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2006,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: String::from("contract"),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: None,
        };

        validate_career(&career).expect("failed to get Ok");
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
            contract_type: String::from("other"),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: None,
        };

        validate_career(&career).expect("failed to get Ok");
    }

    #[test]
    fn validate_career_returns_ok_if_256_char_company_name_is_passed() {
        let career = Career {
            company_name: String::from("ああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ"),
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

        validate_career(&career).expect("failed to get Ok");
    }

    #[test]
    fn validate_career_returns_err_if_empty_char_company_name_is_passed() {
        let career = Career {
            company_name: String::from(""),
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

        let result = validate_career(&career).expect_err("failed to get Err");

        assert_eq!(
            CareerValidationError::InvalidCompanyNameLength {
                length: career.company_name.chars().count(),
                min_length: COMPANY_NAME_MIN_LENGTH,
                max_length: COMPANY_NAME_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_career_returns_err_if_257_char_company_name_is_passed() {
        let career = Career {
            company_name: String::from("あああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ"),
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

        let result = validate_career(&career).expect_err("failed to get Err");

        assert_eq!(
            CareerValidationError::InvalidCompanyNameLength {
                length: career.company_name.chars().count(),
                min_length: COMPANY_NAME_MIN_LENGTH,
                max_length: COMPANY_NAME_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_career_returns_err_if_company_name_is_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: s.to_string(),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInCompanyName(career.company_name),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_company_name_starts_with_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: s.to_string() + "山田工業",
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInCompanyName(career.company_name),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_company_name_ends_with_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: "山田工業".to_string() + s,
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInCompanyName(career.company_name),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_company_name_includes_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: "山田".to_string() + s + "工業",
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInCompanyName(career.company_name),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_company_name_is_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: s.to_string(),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInCompanyName(career.company_name),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_company_name_starts_with_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: s.to_string() + "山田工業",
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInCompanyName(career.company_name),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_company_name_ends_with_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: "山田工業".to_string() + s,
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInCompanyName(career.company_name),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_company_name_includes_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: "山田".to_string() + s + "工業",
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInCompanyName(career.company_name),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_ok_if_1_char_department_name_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
            department_name: Some(String::from("あ")),
            office: None,
            career_start_date: Ymd {
                year: 2006,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: String::from("other"),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: None,
        };

        validate_career(&career).expect("failed to get Ok");
    }

    #[test]
    fn validate_career_returns_ok_if_256_char_department_name_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
            department_name: Some(String::from("ああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ")),
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

        validate_career(&career).expect("failed to get Ok");
    }

    #[test]
    fn validate_career_returns_err_if_empty_char_department_name_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
            department_name: Some(String::from("")),
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

        let result = validate_career(&career).expect_err("failed to get Err");

        assert_eq!(
            CareerValidationError::InvalidDepartmentNameLength {
                length: career
                    .department_name
                    .expect("failed to get department_name")
                    .chars()
                    .count(),
                min_length: DEPARTMENT_NAME_MIN_LENGTH,
                max_length: DEPARTMENT_NAME_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_career_returns_err_if_257_char_department_name_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
            department_name: Some(String::from("あああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ")),
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

        let result = validate_career(&career).expect_err("failed to get Err");

        assert_eq!(
            CareerValidationError::InvalidDepartmentNameLength {
                length: career
                    .department_name
                    .expect("failed to get department_name")
                    .chars()
                    .count(),
                min_length: DEPARTMENT_NAME_MIN_LENGTH,
                max_length: DEPARTMENT_NAME_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_career_returns_err_if_department_name_is_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: Some(s.to_string()),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInDepartmentName(
                    career
                        .department_name
                        .expect("failed to get deparment_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_department_name_starts_with_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: Some(s.to_string() + "第二営業部"),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInDepartmentName(
                    career
                        .department_name
                        .expect("failed to get deparment_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_department_name_ends_with_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: Some("第二営業部".to_string() + s),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInDepartmentName(
                    career
                        .department_name
                        .expect("failed to get deparment_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_department_name_includes_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: "佐藤商事".to_string(),
                department_name: Some("第二".to_string() + s + "営業部"),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInDepartmentName(
                    career
                        .department_name
                        .expect("failed to get deparment_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_department_name_is_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: Some(s.to_string()),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInDepartmentName(
                    career
                        .department_name
                        .expect("failed to get deparment_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_department_name_starts_with_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: Some(s.to_string() + "第二営業部"),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInDepartmentName(
                    career
                        .department_name
                        .expect("failed to get deparment_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_department_name_ends_with_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: Some("第二営業部".to_string() + s),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInDepartmentName(
                    career
                        .department_name
                        .expect("failed to get deparment_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_department_name_includes_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: "佐藤商事".to_string(),
                department_name: Some("第二".to_string() + s + "営業部"),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInDepartmentName(
                    career
                        .department_name
                        .expect("failed to get deparment_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_ok_if_1_char_office_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
            department_name: None,
            office: Some(String::from("あ")),
            career_start_date: Ymd {
                year: 2006,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: String::from("other"),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: None,
        };

        validate_career(&career).expect("failed to get Ok");
    }

    #[test]
    fn validate_career_returns_ok_if_256_char_office_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
            department_name: None,
            office: Some(String::from("ああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ")),
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

        validate_career(&career).expect("failed to get Ok");
    }

    #[test]
    fn validate_career_returns_err_if_empty_char_office_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
            department_name: None,
            office: Some(String::from("")),
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

        let result = validate_career(&career).expect_err("failed to get Err");

        assert_eq!(
            CareerValidationError::InvalidOfficeLength {
                length: career.office.expect("failed to get office").chars().count(),
                min_length: OFFICE_MIN_LENGTH,
                max_length: OFFICE_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_career_returns_err_if_257_char_office_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
            department_name: None,
            office: Some(String::from("あああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ")),
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

        let result = validate_career(&career).expect_err("failed to get Err");

        assert_eq!(
            CareerValidationError::InvalidOfficeLength {
                length: career.office.expect("failed to get office").chars().count(),
                min_length: OFFICE_MIN_LENGTH,
                max_length: OFFICE_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_career_returns_err_if_office_is_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: None,
                office: Some(s.to_string()),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInOffice(
                    career.office.expect("failed to get office")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_office_starts_with_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: None,
                office: Some(s.to_string() + "松山事業所"),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInOffice(
                    career.office.expect("failed to get office")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_office_ends_with_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: None,
                office: Some("松山事業所".to_string() + s),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInOffice(
                    career.office.expect("failed to get office")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_office_includes_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: "佐藤商事".to_string(),
                department_name: None,
                office: Some("松山".to_string() + s + "事業所"),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInOffice(
                    career.office.expect("failed to get office")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_office_is_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: None,
                office: Some(s.to_string()),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInOffice(
                    career.office.expect("failed to get office")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_office_starts_with_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: None,
                office: Some(s.to_string() + "松山事業所"),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInOffice(
                    career.office.expect("failed to get office")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_office_ends_with_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: None,
                office: Some("松山事業所".to_string() + s),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInOffice(
                    career.office.expect("failed to get deparment_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_office_includes_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: "佐藤商事".to_string(),
                department_name: None,
                office: Some("松山".to_string() + s + "事業所"),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInOffice(
                    career.office.expect("failed to get office")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_office_is_space() {
        let mut career_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: None,
                office: Some(s.to_string()),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInOffice(
                    career.office.expect("failed to get office")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_office_starts_with_space() {
        let mut career_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: None,
                office: Some(s.to_string() + "松山事業所"),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInOffice(
                    career.office.expect("failed to get office")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_office_ends_with_space() {
        let mut career_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: None,
                office: Some("松山事業所".to_string() + s),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInOffice(
                    career.office.expect("failed to get deparment_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_office_includes_space() {
        let mut career_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let career = Career {
                company_name: "佐藤商事".to_string(),
                department_name: None,
                office: Some("松山".to_string() + s + "事業所"),
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
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInOffice(
                    career.office.expect("failed to get office")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_career_start_date_is_illegal() {
        let career = Career {
            company_name: "佐藤商事".to_string(),
            department_name: None,
            office: Some("松山事業所".to_string()),
            career_start_date: Ymd {
                year: 2006,
                month: 2,
                day: 30,
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

        let err = validate_career(&career).expect_err("failed to get Err");

        assert_eq!(
            CareerValidationError::IllegalCareerStartDate {
                year: career.career_start_date.year,
                month: career.career_start_date.month,
                day: career.career_start_date.day
            },
            err
        );
    }

    #[test]
    fn validate_career_returns_err_if_career_end_date_is_illegal() {
        let career = Career {
            company_name: "佐藤商事".to_string(),
            department_name: None,
            office: Some("松山事業所".to_string()),
            career_start_date: Ymd {
                year: 2006,
                month: 4,
                day: 1,
            },
            career_end_date: Some(Ymd {
                year: 2008,
                month: 12,
                day: 32,
            }),
            contract_type: String::from("regular"),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: None,
        };

        let err = validate_career(&career).expect_err("failed to get Err");

        let career_end_date = career
            .career_end_date
            .expect("failed to get career_end_day");
        assert_eq!(
            CareerValidationError::IllegalCareerEndDate {
                year: career_end_date.year,
                month: career_end_date.month,
                day: career_end_date.day
            },
            err
        );
    }

    #[test]
    fn validate_career_returns_ok_if_career_start_date_is_career_end_date() {
        let career = Career {
            company_name: "佐藤商事".to_string(),
            department_name: None,
            office: Some("松山事業所".to_string()),
            career_start_date: Ymd {
                year: 2006,
                month: 4,
                day: 1,
            },
            career_end_date: Some(Ymd {
                year: 2006,
                month: 4,
                day: 1,
            }),
            contract_type: String::from("regular"),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: None,
        };

        validate_career(&career).expect("failed to get Ok");
    }

    #[test]
    fn validate_career_returns_err_if_career_end_date_exceeds_career_start_date() {
        let career = Career {
            company_name: "佐藤商事".to_string(),
            department_name: None,
            office: Some("松山事業所".to_string()),
            career_start_date: Ymd {
                year: 2010,
                month: 4,
                day: 2,
            },
            career_end_date: Some(Ymd {
                year: 2010,
                month: 4,
                day: 1,
            }),
            contract_type: String::from("regular"),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: None,
        };

        let err = validate_career(&career).expect_err("failed to get Err");

        let career_start_date = career.career_start_date;
        let career_end_date = career
            .career_end_date
            .expect("failed to get career_end_day");
        assert_eq!(
            CareerValidationError::CareerStartDateExceedsCareerEndDate {
                career_start_date,
                career_end_date
            },
            err
        );
    }

    #[test]
    fn validate_career_returns_ok_if_valid_contract_type_is_passed() {
        let mut career_list = Vec::with_capacity(CONTRACT_TYPE_SET.len());
        for s in CONTRACT_TYPE_SET.iter() {
            let career = Career {
                company_name: "佐藤商事".to_string(),
                department_name: None,
                office: Some("松山事業所".to_string()),
                career_start_date: Ymd {
                    year: 2006,
                    month: 4,
                    day: 1,
                },
                career_end_date: None,
                contract_type: s.to_string(),
                profession: None,
                annual_income_in_man_yen: None,
                is_manager: true,
                position_name: None,
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            validate_career(&career).expect("failed to get Ok");
        }
    }

    #[test]
    fn validate_career_returns_err_if_illegal_contract_type_is_passed() {
        let career = Career {
            company_name: "佐藤商事".to_string(),
            department_name: None,
            office: Some("松山事業所".to_string()),
            career_start_date: Ymd {
                year: 2006,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: "1' or '1' = '1';--".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: None,
        };
        let err = validate_career(&career).expect_err("failed to get Err");
        assert_eq!(
            CareerValidationError::IllegalContractType(career.contract_type),
            err
        );
    }

    #[test]
    fn validate_career_returns_ok_if_1_char_profession_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2006,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: String::from("other"),
            profession: Some("あ".to_string()),
            annual_income_in_man_yen: None,
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: None,
        };

        validate_career(&career).expect("failed to get Ok");
    }

    #[test]
    fn validate_career_returns_ok_if_128_char_profession_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2006,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: String::from("regular"),
            profession: Some("ああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ".to_string()),
            annual_income_in_man_yen: None,
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: None,
        };

        validate_career(&career).expect("failed to get Ok");
    }

    #[test]
    fn validate_career_returns_err_if_empty_char_profession_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2006,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: String::from("regular"),
            profession: Some(String::from("")),
            annual_income_in_man_yen: None,
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: None,
        };

        let result = validate_career(&career).expect_err("failed to get Err");

        assert_eq!(
            CareerValidationError::InvalidProfessionLength {
                length: career
                    .profession
                    .expect("failed to get profession")
                    .chars()
                    .count(),
                min_length: PROFESSION_MIN_LENGTH,
                max_length: PROFESSION_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_career_returns_err_if_129_char_profession_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2006,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: String::from("regular"),
            profession: Some("あああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ".to_string()),
            annual_income_in_man_yen: None,
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: None,
        };
        let result = validate_career(&career).expect_err("failed to get Err");

        assert_eq!(
            CareerValidationError::InvalidProfessionLength {
                length: career
                    .profession
                    .expect("failed to get profession")
                    .chars()
                    .count(),
                min_length: PROFESSION_MIN_LENGTH,
                max_length: PROFESSION_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_career_returns_err_if_profession_is_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: None,
                office: None,
                career_start_date: Ymd {
                    year: 2006,
                    month: 4,
                    day: 1,
                },
                career_end_date: None,
                contract_type: String::from("regular"),
                profession: Some(s.to_string()),
                annual_income_in_man_yen: None,
                is_manager: true,
                position_name: None,
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInProfession(
                    career.profession.expect("failed to get profession")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_profession_starts_with_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: None,
                office: None,
                career_start_date: Ymd {
                    year: 2006,
                    month: 4,
                    day: 1,
                },
                career_end_date: None,
                contract_type: String::from("regular"),
                profession: Some(s.to_string() + "営業"),
                annual_income_in_man_yen: None,
                is_manager: true,
                position_name: None,
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInProfession(
                    career.profession.expect("failed to get profession")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_profession_ends_with_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: None,
                office: None,
                career_start_date: Ymd {
                    year: 2006,
                    month: 4,
                    day: 1,
                },
                career_end_date: None,
                contract_type: String::from("regular"),
                profession: Some("営業".to_string() + s),
                annual_income_in_man_yen: None,
                is_manager: true,
                position_name: None,
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInProfession(
                    career.profession.expect("failed to get profession")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_profession_includes_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: "佐藤商事".to_string(),
                department_name: None,
                office: None,
                career_start_date: Ymd {
                    year: 2006,
                    month: 4,
                    day: 1,
                },
                career_end_date: None,
                contract_type: String::from("regular"),
                profession: Some("営".to_string() + s + "業"),
                annual_income_in_man_yen: None,
                is_manager: true,
                position_name: None,
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInProfession(
                    career.profession.expect("failed to get profession")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_profession_is_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: None,
                office: None,
                career_start_date: Ymd {
                    year: 2006,
                    month: 4,
                    day: 1,
                },
                career_end_date: None,
                contract_type: String::from("regular"),
                profession: Some(s.to_string()),
                annual_income_in_man_yen: None,
                is_manager: true,
                position_name: None,
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInProfession(
                    career.profession.expect("failed to get profession")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_profession_starts_with_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: None,
                office: None,
                career_start_date: Ymd {
                    year: 2006,
                    month: 4,
                    day: 1,
                },
                career_end_date: None,
                contract_type: String::from("regular"),
                profession: Some(s.to_string() + "営業"),
                annual_income_in_man_yen: None,
                is_manager: true,
                position_name: None,
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInProfession(
                    career.profession.expect("failed to get profession")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_profession_ends_with_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: None,
                office: None,
                career_start_date: Ymd {
                    year: 2006,
                    month: 4,
                    day: 1,
                },
                career_end_date: None,
                contract_type: String::from("regular"),
                profession: Some("営業".to_string() + s),
                annual_income_in_man_yen: None,
                is_manager: true,
                position_name: None,
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInProfession(
                    career.profession.expect("failed to get profession")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_profession_includes_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: "佐藤商事".to_string(),
                department_name: None,
                office: None,
                career_start_date: Ymd {
                    year: 2006,
                    month: 4,
                    day: 1,
                },
                career_end_date: None,
                contract_type: String::from("regular"),
                profession: Some("営".to_string() + s + "業"),
                annual_income_in_man_yen: None,
                is_manager: true,
                position_name: None,
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInProfession(
                    career.profession.expect("failed to get profession")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_profession_is_space() {
        let mut career_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: None,
                office: None,
                career_start_date: Ymd {
                    year: 2006,
                    month: 4,
                    day: 1,
                },
                career_end_date: None,
                contract_type: String::from("regular"),
                profession: Some(s.to_string()),
                annual_income_in_man_yen: None,
                is_manager: true,
                position_name: None,
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInProfession(
                    career.profession.expect("failed to get profession")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_profession_starts_with_space() {
        let mut career_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: None,
                office: None,
                career_start_date: Ymd {
                    year: 2006,
                    month: 4,
                    day: 1,
                },
                career_end_date: None,
                contract_type: String::from("regular"),
                profession: Some(s.to_string() + "営業"),
                annual_income_in_man_yen: None,
                is_manager: true,
                position_name: None,
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInProfession(
                    career.profession.expect("failed to get profession")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_profession_ends_with_space() {
        let mut career_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
                department_name: None,
                office: None,
                career_start_date: Ymd {
                    year: 2006,
                    month: 4,
                    day: 1,
                },
                career_end_date: None,
                contract_type: String::from("regular"),
                profession: Some("営業".to_string() + s),
                annual_income_in_man_yen: None,
                is_manager: true,
                position_name: None,
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInProfession(
                    career.profession.expect("failed to get profession")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_profession_includes_space() {
        let mut career_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let career = Career {
                company_name: "佐藤商事".to_string(),
                department_name: None,
                office: None,
                career_start_date: Ymd {
                    year: 2006,
                    month: 4,
                    day: 1,
                },
                career_end_date: None,
                contract_type: String::from("regular"),
                profession: Some("営".to_string() + s + "業"),
                annual_income_in_man_yen: None,
                is_manager: true,
                position_name: None,
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInProfession(
                    career.profession.expect("failed to get profession")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_ok_if_annual_imcom_in_man_yen_0_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
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
            annual_income_in_man_yen: Some(0),
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: None,
        };
        validate_career(&career).expect("failed to get Ok");
    }

    #[test]
    fn validate_career_returns_ok_if_max_annual_imcom_in_man_yen_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
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
            annual_income_in_man_yen: Some(MAX_ANNUAL_INCOME_IN_MAN_YEN),
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: None,
        };
        validate_career(&career).expect("failed to get Ok");
    }

    #[test]
    fn validate_career_returns_err_if_over_max_annual_imcom_in_man_yen_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
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
            annual_income_in_man_yen: Some(MAX_ANNUAL_INCOME_IN_MAN_YEN + 1),
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: None,
        };

        let err = validate_career(&career).expect_err("failed to get Err");

        assert_eq!(
            CareerValidationError::IllegalAnnualIncomeInManYen(
                career
                    .annual_income_in_man_yen
                    .expect("failed to get annual_income_in_man_yen")
            ),
            err
        );
    }

    #[test]
    fn validate_career_returns_err_if_negative_annual_imcom_in_man_yen_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
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
            annual_income_in_man_yen: Some(-1),
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: None,
        };

        let err = validate_career(&career).expect_err("failed to get Err");

        assert_eq!(
            CareerValidationError::IllegalAnnualIncomeInManYen(
                career
                    .annual_income_in_man_yen
                    .expect("failed to get annual_income_in_man_yen")
            ),
            err
        );
    }

    #[test]
    fn validate_career_returns_err_if_i32_min_annual_imcom_in_man_yen_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
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
            annual_income_in_man_yen: Some(i32::MIN),
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: None,
        };

        let err = validate_career(&career).expect_err("failed to get Err");

        assert_eq!(
            CareerValidationError::IllegalAnnualIncomeInManYen(
                career
                    .annual_income_in_man_yen
                    .expect("failed to get annual_income_in_man_yen")
            ),
            err
        );
    }

    #[test]
    fn validate_career_returns_ok_if_1_char_position_name_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2006,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: String::from("other"),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: true,
            position_name: Some("あ".to_string()),
            is_new_graduate: false,
            note: None,
        };

        validate_career(&career).expect("failed to get Ok");
    }

    #[test]
    fn validate_career_returns_ok_if_128_char_position_name_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
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
            position_name: Some("ああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ".to_string()),
            is_new_graduate: false,
            note: None,
        };

        validate_career(&career).expect("failed to get Ok");
    }

    #[test]
    fn validate_career_returns_err_if_empty_char_position_name_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
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
            position_name: Some(String::from("")),
            is_new_graduate: false,
            note: None,
        };

        let result = validate_career(&career).expect_err("failed to get Err");

        assert_eq!(
            CareerValidationError::InvalidPositionNameLength {
                length: career
                    .position_name
                    .expect("failed to get position_name")
                    .chars()
                    .count(),
                min_length: POSITION_NAME_MIN_LENGTH,
                max_length: POSITION_NAME_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_career_returns_err_if_129_char_position_name_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
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
            position_name: Some("あああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ".to_string()),
            is_new_graduate: false,
            note: None,
        };

        let result = validate_career(&career).expect_err("failed to get Err");

        assert_eq!(
            CareerValidationError::InvalidPositionNameLength {
                length: career
                    .position_name
                    .expect("failed to get position_name")
                    .chars()
                    .count(),
                min_length: POSITION_NAME_MIN_LENGTH,
                max_length: POSITION_NAME_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_career_returns_err_if_position_name_is_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
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
                position_name: Some(s.to_string()),
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInPositionName(
                    career.position_name.expect("failed to get position_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_position_name_starts_with_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
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
                position_name: Some(s.to_string() + "係長"),
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInPositionName(
                    career.position_name.expect("failed to get position_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_position_name_ends_with_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
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
                position_name: Some("係長".to_string() + s),
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInPositionName(
                    career.position_name.expect("failed to get position_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_position_name_includes_control_char() {
        let mut career_list = Vec::with_capacity(CONTROL_CHAR_SET.len());
        for s in CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: "佐藤商事".to_string(),
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
                position_name: Some("係".to_string() + s + "長"),
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInPositionName(
                    career.position_name.expect("failed to get position_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_position_name_is_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
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
                position_name: Some(s.to_string()),
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInPositionName(
                    career.position_name.expect("failed to get position_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_position_name_starts_with_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
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
                position_name: Some(s.to_string() + "係長"),
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInPositionName(
                    career.position_name.expect("failed to get position_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_position_name_ends_with_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
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
                position_name: Some("係長".to_string() + s),
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInPositionName(
                    career.position_name.expect("failed to get position_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_position_name_includes_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: "佐藤商事".to_string(),
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
                position_name: Some("係".to_string() + s + "長"),
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInPositionName(
                    career.position_name.expect("failed to get position_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_position_name_is_space() {
        let mut career_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
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
                position_name: Some(s.to_string()),
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInPositionName(
                    career.position_name.expect("failed to get position_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_position_name_starts_with_space() {
        let mut career_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
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
                position_name: Some(s.to_string() + "係長"),
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInPositionName(
                    career.position_name.expect("failed to get position_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_position_name_ends_with_space() {
        let mut career_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
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
                position_name: Some("係長".to_string() + s),
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInPositionName(
                    career.position_name.expect("failed to get position_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_position_name_includes_space() {
        let mut career_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let career = Career {
                company_name: "佐藤商事".to_string(),
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
                position_name: Some("係".to_string() + s + "長"),
                is_new_graduate: false,
                note: None,
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInPositionName(
                    career.position_name.expect("failed to get position_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_ok_if_1_char_note_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2006,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: String::from("other"),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: Some("あ".to_string()),
        };

        validate_career(&career).expect("failed to get Ok");
    }

    #[test]
    fn validate_career_returns_ok_if_2048_char_note_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
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
            note: Some("ああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ".to_string()),
        };

        validate_career(&career).expect("failed to get Ok");
    }

    #[test]
    fn validate_career_returns_err_if_empty_char_note_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
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
            note: Some(String::from("")),
        };

        let result = validate_career(&career).expect_err("failed to get Err");

        assert_eq!(
            CareerValidationError::InvalidNoteLength {
                length: career.note.expect("failed to get note").chars().count(),
                min_length: NOTE_MIN_LENGTH,
                max_length: NOTE_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_career_returns_err_if_2045_char_note_is_passed() {
        let career = Career {
            company_name: String::from("佐藤商事"),
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
            note: Some("あああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ".to_string()),
        };

        let result = validate_career(&career).expect_err("failed to get Err");

        assert_eq!(
            CareerValidationError::InvalidNoteLength {
                length: career.note.expect("failed to get note").chars().count(),
                min_length: NOTE_MIN_LENGTH,
                max_length: NOTE_MAX_LENGTH
            },
            result
        );
    }

    #[test]
    fn validate_career_returns_err_if_note_is_non_new_line_control_char() {
        let mut career_list = Vec::with_capacity(NON_NEW_LINE_CONTROL_CHAR_SET.len());
        for s in NON_NEW_LINE_CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
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
                note: Some(s.to_string()),
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInNote(career.note.expect("failed to get note")),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_note_starts_with_non_new_line_control_char() {
        let mut career_list = Vec::with_capacity(NON_NEW_LINE_CONTROL_CHAR_SET.len());
        for s in NON_NEW_LINE_CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
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
                note: Some(s.to_string() + "備考"),
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInNote(career.note.expect("failed to get note")),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_note_ends_with_non_new_line_control_char() {
        let mut career_list = Vec::with_capacity(NON_NEW_LINE_CONTROL_CHAR_SET.len());
        for s in NON_NEW_LINE_CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
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
                note: Some("備考".to_string() + s),
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInNote(
                    career.note.expect("failed to get position_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_note_includes_non_new_line_control_char() {
        let mut career_list = Vec::with_capacity(NON_NEW_LINE_CONTROL_CHAR_SET.len());
        for s in NON_NEW_LINE_CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: "佐藤商事".to_string(),
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
                note: Some("備".to_string() + s + "考"),
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInNote(career.note.expect("failed to get note")),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_ok_if_note_is_new_line_control_char() {
        let mut career_list = Vec::with_capacity(NEW_LINE_CONTROL_CHAR_SET.len());
        for s in NEW_LINE_CONTROL_CHAR_SET.iter() {
            let career = Career {
                company_name: "佐藤商事".to_string(),
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
                note: Some(s.to_string()),
            };
            career_list.push(career);
        }
        for career in career_list {
            validate_career(&career).expect("failed to get Ok");
        }
    }

    #[test]
    fn validate_career_returns_ok_if_note_is_space() {
        let mut career_list = Vec::with_capacity(SPACE_SET.len());
        for s in SPACE_SET.iter() {
            let career = Career {
                company_name: "佐藤商事".to_string(),
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
                note: Some(s.to_string()),
            };
            career_list.push(career);
        }
        for career in career_list {
            validate_career(&career).expect("failed to get Ok");
        }
    }

    #[test]
    fn validate_career_returns_ok_if_note_includes_space_and_new_line() {
        let career = Career {
            company_name: String::from("佐藤商事"),
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
            note: Some(String::from(
                "備考は、
            
            改行や\n
             　空白を\r
             受け入れます。\r\n
             
             ",
            )),
        };

        validate_career(&career).expect("failed to get Ok");
    }

    #[test]
    fn validate_career_returns_err_if_note_is_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
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
                note: Some(s.to_string()),
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInNote(career.note.expect("failed to get note")),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_note_starts_with_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
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
                note: Some(s.to_string() + "備考"),
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInNote(career.note.expect("failed to get note")),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_note_ends_with_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: String::from("佐藤商事"),
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
                note: Some("備考".to_string() + s),
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInNote(
                    career.note.expect("failed to get position_name")
                ),
                err
            );
        }
    }

    #[test]
    fn validate_career_returns_err_if_note_includes_symbol() {
        let mut career_list = Vec::with_capacity(SYMBOL_SET.len());
        for s in SYMBOL_SET.iter() {
            let career = Career {
                company_name: "佐藤商事".to_string(),
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
                note: Some("備".to_string() + s + "考"),
            };
            career_list.push(career);
        }
        for career in career_list {
            let err = validate_career(&career).expect_err("failed to get Err");
            assert_eq!(
                CareerValidationError::IllegalCharInNote(career.note.expect("failed to get note")),
                err
            );
        }
    }
}
