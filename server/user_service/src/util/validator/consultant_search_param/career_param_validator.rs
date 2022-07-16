// Copyright 2022 Ken Miura

use std::{collections::HashSet, error::Error, fmt::Display};

use once_cell::sync::Lazy;

use crate::{
    consultants_search::{AnnualInComeInManYenParam, CareerParam},
    util::{
        validator::MAX_ANNUAL_INCOME_IN_MAN_YEN, YEARS_OF_SERVICE_FIFTEEN_YEARS_OR_MORE,
        YEARS_OF_SERVICE_FIVE_YEARS_OR_MORE, YEARS_OF_SERVICE_TEN_YEARS_OR_MORE,
        YEARS_OF_SERVICE_THREE_YEARS_OR_MORE, YEARS_OF_SERVICE_TWENTY_YEARS_OR_MORE,
    },
};

static YEARS_OF_SERVICE_SET: Lazy<HashSet<String>> = Lazy::new(|| {
    let mut set: HashSet<String> = HashSet::with_capacity(5);
    set.insert(YEARS_OF_SERVICE_THREE_YEARS_OR_MORE.to_string());
    set.insert(YEARS_OF_SERVICE_FIVE_YEARS_OR_MORE.to_string());
    set.insert(YEARS_OF_SERVICE_TEN_YEARS_OR_MORE.to_string());
    set.insert(YEARS_OF_SERVICE_FIFTEEN_YEARS_OR_MORE.to_string());
    set.insert(YEARS_OF_SERVICE_TWENTY_YEARS_OR_MORE.to_string());
    set
});

pub(crate) fn validate_career_param(
    career_param: &CareerParam,
) -> Result<(), CareerParamValidationError> {
    if let Some(company_name) = &career_param.company_name {
        let _ = validate_company(company_name.as_str())?;
    };
    if let Some(department_name) = &career_param.department_name {
        let _ = validate_department_name(department_name.as_str())?;
    };
    if let Some(office) = &career_param.office {
        let _ = validate_office(office.as_str())?;
    };
    if let Some(years_of_service) = &career_param.years_of_service {
        let _ = validate_years_of_service(years_of_service.as_str())?;
    };
    if let Some(contract_type) = &career_param.contract_type {
        let _ = validate_contract_type(contract_type.as_str())?;
    };
    if let Some(profession) = &career_param.profession {
        let _ = validate_profession(profession.as_str())?;
    };
    let _ = validate_annual_income_in_man_yen_param(&career_param.annual_income_in_man_yen)?;

    if let Some(position_name) = &career_param.position_name {
        let _ = validate_position_name(position_name.as_str())?;
    };
    if let Some(note) = &career_param.note {
        let _ = validate_note(note.as_str())?;
    };
    Ok(())
}

fn validate_company(company_name: &str) -> Result<(), CareerParamValidationError> {
    let _ = crate::util::validator::validate_company_name(company_name).map_err(|e| match e {
        crate::util::validator::CompanyNameValidationError::InvalidCompanyNameLength {
            length,
            min_length,
            max_length,
        } => CareerParamValidationError::InvalidCompanyNameLength {
            length,
            min_length,
            max_length,
        },
        crate::util::validator::CompanyNameValidationError::IllegalCharInCompanyName(
            company_name,
        ) => CareerParamValidationError::IllegalCharInCompanyName(company_name),
    })?;
    Ok(())
}

fn validate_department_name(department_name: &str) -> Result<(), CareerParamValidationError> {
    let _ =
        crate::util::validator::validate_department_name(department_name).map_err(|e| match e {
            crate::util::validator::DepartmentNameValidationError::InvalidDepartmentNameLength {
                length,
                min_length,
                max_length,
            } => CareerParamValidationError::InvalidDepartmentNameLength { length, min_length, max_length },
            crate::util::validator::DepartmentNameValidationError::IllegalCharInDepartmentName(department_name) => CareerParamValidationError::IllegalCharInDepartmentName(department_name),
        })?;
    Ok(())
}

fn validate_office(office: &str) -> Result<(), CareerParamValidationError> {
    let _ = crate::util::validator::validate_office(office).map_err(|e| match e {
        crate::util::validator::OfficeValidationError::InvalidOfficeLength {
            length,
            min_length,
            max_length,
        } => CareerParamValidationError::InvalidOfficeLength {
            length,
            min_length,
            max_length,
        },
        crate::util::validator::OfficeValidationError::IllegalCharInOffice(office) => {
            CareerParamValidationError::IllegalCharInOffice(office)
        }
    })?;
    Ok(())
}

fn validate_years_of_service(years_of_service: &str) -> Result<(), CareerParamValidationError> {
    if !YEARS_OF_SERVICE_SET.contains(years_of_service) {
        return Err(CareerParamValidationError::IllegalYearsOfService(
            years_of_service.to_string(),
        ));
    }
    Ok(())
}

fn validate_contract_type(contract_type: &str) -> Result<(), CareerParamValidationError> {
    let _ = crate::util::validator::validate_contract_type(contract_type).map_err(|e| match e {
        crate::util::validator::ContractTypeValidationError::IllegalContractType(contract_type) => {
            CareerParamValidationError::IllegalContractType(contract_type)
        }
    })?;
    Ok(())
}

fn validate_profession(profession: &str) -> Result<(), CareerParamValidationError> {
    let _ = crate::util::validator::validate_profession(profession).map_err(|e| match e {
        crate::util::validator::ProfessionValidationError::InvalidProfessionLength {
            length,
            min_length,
            max_length,
        } => CareerParamValidationError::InvalidProfessionLength {
            length,
            min_length,
            max_length,
        },
        crate::util::validator::ProfessionValidationError::IllegalCharInProfession(profession) => {
            CareerParamValidationError::IllegalCharInProfession(profession)
        }
    })?;
    Ok(())
}

fn validate_annual_income_in_man_yen_param(
    annual_income_in_man_yen_param: &AnnualInComeInManYenParam,
) -> Result<(), CareerParamValidationError> {
    if let Some(equal_or_more) = annual_income_in_man_yen_param.equal_or_more {
        if !(0..=MAX_ANNUAL_INCOME_IN_MAN_YEN).contains(&equal_or_more) {
            return Err(
                CareerParamValidationError::InvalidEqualOrMoreInAnnualIncomInManYen {
                    value: equal_or_more,
                    min: 0,
                    max: MAX_ANNUAL_INCOME_IN_MAN_YEN,
                },
            );
        }
    }
    if let Some(equal_or_less) = annual_income_in_man_yen_param.equal_or_less {
        if !(0..=MAX_ANNUAL_INCOME_IN_MAN_YEN).contains(&equal_or_less) {
            return Err(
                CareerParamValidationError::InvalidEqualOrLessInAnnualIncomInManYen {
                    value: equal_or_less,
                    min: 0,
                    max: MAX_ANNUAL_INCOME_IN_MAN_YEN,
                },
            );
        }
    }
    if let Some(equal_or_more) = annual_income_in_man_yen_param.equal_or_more {
        if let Some(equal_or_less) = annual_income_in_man_yen_param.equal_or_less {
            if equal_or_more > equal_or_less {
                return Err(CareerParamValidationError::EqualOrMoreExceedsEqualOrLessInAnnualIncomInManYen { equal_or_more, equal_or_less });
            }
        }
    }
    Ok(())
}

fn validate_position_name(position_name: &str) -> Result<(), CareerParamValidationError> {
    let _ = crate::util::validator::validate_position_name(position_name).map_err(|e| match e {
        crate::util::validator::PositionNameValidationError::InvalidPositionNameLength {
            length,
            min_length,
            max_length,
        } => CareerParamValidationError::InvalidPositionNameLength {
            length,
            min_length,
            max_length,
        },
        crate::util::validator::PositionNameValidationError::IllegalCharInPositionName(
            position_name,
        ) => CareerParamValidationError::IllegalCharInPositionName(position_name),
    })?;
    Ok(())
}

fn validate_note(note: &str) -> Result<(), CareerParamValidationError> {
    let _ = crate::util::validator::validate_note(note).map_err(|e| match e {
        crate::util::validator::NoteValidationError::InvalidNoteLength {
            length,
            min_length,
            max_length,
        } => CareerParamValidationError::InvalidNoteLength {
            length,
            min_length,
            max_length,
        },
        crate::util::validator::NoteValidationError::IllegalCharInNote(note) => {
            CareerParamValidationError::IllegalCharInNote(note)
        }
    })?;
    Ok(())
}

/// Error related to [validate_career_param()]
#[derive(Debug, PartialEq)]
pub(crate) enum CareerParamValidationError {
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
    IllegalYearsOfService(String),
    IllegalContractType(String),
    InvalidProfessionLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInProfession(String),
    InvalidEqualOrMoreInAnnualIncomInManYen {
        value: i32,
        min: i32,
        max: i32,
    },
    InvalidEqualOrLessInAnnualIncomInManYen {
        value: i32,
        min: i32,
        max: i32,
    },
    EqualOrMoreExceedsEqualOrLessInAnnualIncomInManYen {
        equal_or_more: i32,
        equal_or_less: i32,
    },
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

impl Display for CareerParamValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CareerParamValidationError::InvalidCompanyNameLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid company_name length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            CareerParamValidationError::IllegalCharInCompanyName(company_name) => {
                write!(
                    f,
                    "company_name: illegal charcter included: {} (binary: {:X?})",
                    company_name,
                    company_name.as_bytes().to_vec()
                )
            }
            CareerParamValidationError::InvalidDepartmentNameLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid department_name length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            CareerParamValidationError::IllegalCharInDepartmentName(department_name) => {
                write!(
                    f,
                    "department_name: illegal charcter included: {} (binary: {:X?})",
                    department_name,
                    department_name.as_bytes().to_vec()
                )
            }
            CareerParamValidationError::InvalidOfficeLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid office length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            CareerParamValidationError::IllegalCharInOffice(office) => {
                write!(
                    f,
                    "office: illegal charcter included: {} (binary: {:X?})",
                    office,
                    office.as_bytes().to_vec()
                )
            }
            CareerParamValidationError::IllegalYearsOfService(years_of_service) => {
                write!(f, "illegal years_of_service ({})", years_of_service)
            }
            CareerParamValidationError::IllegalContractType(contract_type) => {
                write!(f, "illegal contract_type ({})", contract_type)
            }
            CareerParamValidationError::InvalidProfessionLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid profession length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            CareerParamValidationError::IllegalCharInProfession(profession) => {
                write!(
                    f,
                    "profession: illegal charcter included: {} (binary: {:X?})",
                    profession,
                    profession.as_bytes().to_vec()
                )
            }
            CareerParamValidationError::InvalidEqualOrMoreInAnnualIncomInManYen {
                value,
                min,
                max,
            } => {
                write!(
                    f,
                    "invalid equal_or_more (value: {}, min: {}, max {})",
                    value, min, max
                )
            }
            CareerParamValidationError::InvalidEqualOrLessInAnnualIncomInManYen {
                value,
                min,
                max,
            } => {
                write!(
                    f,
                    "invalid equal_or_less (value: {}, min: {}, max {})",
                    value, min, max
                )
            }
            CareerParamValidationError::EqualOrMoreExceedsEqualOrLessInAnnualIncomInManYen {
                equal_or_more,
                equal_or_less,
            } => write!(
                f,
                "equal_or_more exceeds equal_or_less (equal_or_more: {}, equal_or_less: {})",
                equal_or_more, equal_or_less
            ),
            CareerParamValidationError::InvalidPositionNameLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid position_name length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            CareerParamValidationError::IllegalCharInPositionName(position_name) => {
                write!(
                    f,
                    "position_name: illegal charcter included: {} (binary: {:X?})",
                    position_name,
                    position_name.as_bytes().to_vec()
                )
            }
            CareerParamValidationError::InvalidNoteLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid note length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            CareerParamValidationError::IllegalCharInNote(note) => {
                write!(
                    f,
                    "note: illegal charcter included: {} (binary: {:X?})",
                    note,
                    note.as_bytes().to_vec()
                )
            }
        }
    }
}

impl Error for CareerParamValidationError {}

#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;

    use crate::{
        consultants_search::{AnnualInComeInManYenParam, CareerParam},
        util::{
            validator::{
                consultant_search_param::career_param_validator::validate_career_param,
                COMPANY_NAME_MAX_LENGTH, COMPANY_NAME_MIN_LENGTH, DEPARTMENT_NAME_MAX_LENGTH,
                DEPARTMENT_NAME_MIN_LENGTH, MAX_ANNUAL_INCOME_IN_MAN_YEN, OFFICE_MAX_LENGTH,
                OFFICE_MIN_LENGTH, POSITION_NAME_MAX_LENGTH, POSITION_NAME_MIN_LENGTH,
                PROFESSION_MAX_LENGTH, PROFESSION_MIN_LENGTH,
            },
            YEARS_OF_SERVICE_FIFTEEN_YEARS_OR_MORE, YEARS_OF_SERVICE_FIVE_YEARS_OR_MORE,
            YEARS_OF_SERVICE_TEN_YEARS_OR_MORE, YEARS_OF_SERVICE_THREE_YEARS_OR_MORE,
            YEARS_OF_SERVICE_TWENTY_YEARS_OR_MORE,
        },
    };

    use super::CareerParamValidationError;

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: CareerParam,
        expected: Result<(), CareerParamValidationError>,
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        vec![
            TestCase {
                name: "no parameters specified".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Ok(()),
            },
            TestCase {
                name: "all parameters specified".to_string(),
                input: CareerParam {
                    company_name: Some("テスト株式会社".to_string()),
                    department_name: Some("開発部".to_string()),
                    office: Some("山梨事業所".to_string()),
                    years_of_service: Some("THREE_YEARS_OR_MORE".to_string()),
                    employed: Some(true),
                    contract_type: Some("regular".to_string()),
                    profession: Some("ITエンジニア".to_string()),
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: Some(0),
                        equal_or_less: Some(MAX_ANNUAL_INCOME_IN_MAN_YEN),
                    },
                    is_manager: Some(false),
                    position_name: Some("主任".to_string()),
                    is_new_graduate: Some(true),
                    note: Some("備考".to_string()),
                },
                expected: Ok(()),
            },
            TestCase {
                name: "invalid length company_name".to_string(),
                input: CareerParam {
                    company_name: Some("".to_string()),
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Err(CareerParamValidationError::InvalidCompanyNameLength {
                    length: 0,
                    min_length: COMPANY_NAME_MIN_LENGTH,
                    max_length: COMPANY_NAME_MAX_LENGTH,
                }),
            },
            TestCase {
                name: "illegal char company_name".to_string(),
                input: CareerParam {
                    company_name: Some("’ or ‘A’=‘A".to_string()),
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Err(CareerParamValidationError::IllegalCharInCompanyName(
                    "’ or ‘A’=‘A".to_string(),
                )),
            },
            TestCase {
                name: "invalid length department_name".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: Some("".to_string()),
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Err(CareerParamValidationError::InvalidDepartmentNameLength {
                    length: 0,
                    min_length: DEPARTMENT_NAME_MIN_LENGTH,
                    max_length: DEPARTMENT_NAME_MAX_LENGTH,
                }),
            },
            TestCase {
                name: "illegal char department_name".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: Some("’ or ‘A’=‘A".to_string()),
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Err(CareerParamValidationError::IllegalCharInDepartmentName(
                    "’ or ‘A’=‘A".to_string(),
                )),
            },
            TestCase {
                name: "invalid length office".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: Some("".to_string()),
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Err(CareerParamValidationError::InvalidOfficeLength {
                    length: 0,
                    min_length: OFFICE_MIN_LENGTH,
                    max_length: OFFICE_MAX_LENGTH,
                }),
            },
            TestCase {
                name: "illegal char office".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: Some("’ or ‘A’=‘A".to_string()),
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Err(CareerParamValidationError::IllegalCharInOffice(
                    "’ or ‘A’=‘A".to_string(),
                )),
            },
            TestCase {
                name: "valid years_of_service THREE_YEARS_OR_MORE".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: Some(YEARS_OF_SERVICE_THREE_YEARS_OR_MORE.to_string()),
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Ok(()),
            },
            TestCase {
                name: "valid years_of_service FIVE_YEARS_OR_MORE".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: Some(YEARS_OF_SERVICE_FIVE_YEARS_OR_MORE.to_string()),
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Ok(()),
            },
            TestCase {
                name: "valid years_of_service TEN_YEARS_OR_MORE".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: Some(YEARS_OF_SERVICE_TEN_YEARS_OR_MORE.to_string()),
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Ok(()),
            },
            TestCase {
                name: "valid years_of_service FIFTEEN_YEARS_OR_MORE".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: Some(YEARS_OF_SERVICE_FIFTEEN_YEARS_OR_MORE.to_string()),
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Ok(()),
            },
            TestCase {
                name: "valid years_of_service TWENTY_YEARS_OR_MORE".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: Some(YEARS_OF_SERVICE_TWENTY_YEARS_OR_MORE.to_string()),
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Ok(()),
            },
            TestCase {
                name: "invalid years_of_service".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: Some("1' or '1' = '1';--".to_string()),
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Err(CareerParamValidationError::IllegalYearsOfService(
                    "1' or '1' = '1';--".to_string(),
                )),
            },
            TestCase {
                name: "valid contract type regular".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: Some("regular".to_string()),
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Ok(()),
            },
            TestCase {
                name: "valid contract type contract".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: Some("contract".to_string()),
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Ok(()),
            },
            TestCase {
                name: "valid contract type other".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: Some("other".to_string()),
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Ok(()),
            },
            TestCase {
                name: "invalid contract type".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: Some("1' or '1' = '1';--".to_string()),
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Err(CareerParamValidationError::IllegalContractType(
                    "1' or '1' = '1';--".to_string(),
                )),
            },
            TestCase {
                name: "invalid length profession".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: Some("".to_string()),
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Err(CareerParamValidationError::InvalidProfessionLength {
                    length: 0,
                    min_length: PROFESSION_MIN_LENGTH,
                    max_length: PROFESSION_MAX_LENGTH,
                }),
            },
            TestCase {
                name: "illegal char profession".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: Some("’ or ‘A’=‘A".to_string()),
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Err(CareerParamValidationError::IllegalCharInProfession(
                    "’ or ‘A’=‘A".to_string(),
                )),
            },
            TestCase {
                name: "valid equal_or_more in annual_income_in_man_yen 0".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: Some(0),
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Ok(()),
            },
            TestCase {
                name: "valid equal_or_more in annual_income_in_man_yen max value".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: Some(MAX_ANNUAL_INCOME_IN_MAN_YEN),
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Ok(()),
            },
            TestCase {
                name: "invalid equal_or_more in annual_income_in_man_yen negative value"
                    .to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: Some(-1),
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Err(
                    CareerParamValidationError::InvalidEqualOrMoreInAnnualIncomInManYen {
                        value: -1,
                        min: 0,
                        max: MAX_ANNUAL_INCOME_IN_MAN_YEN,
                    },
                ),
            },
            TestCase {
                name: "valid equal_or_more in annual_income_in_man_yen max value".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: Some(MAX_ANNUAL_INCOME_IN_MAN_YEN + 1),
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Err(
                    CareerParamValidationError::InvalidEqualOrMoreInAnnualIncomInManYen {
                        value: MAX_ANNUAL_INCOME_IN_MAN_YEN + 1,
                        min: 0,
                        max: MAX_ANNUAL_INCOME_IN_MAN_YEN,
                    },
                ),
            },
            TestCase {
                name: "valid equal_or_less in annual_income_in_man_yen 0".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: Some(0),
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Ok(()),
            },
            TestCase {
                name: "valid equal_or_less in annual_income_in_man_yen max value".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: Some(MAX_ANNUAL_INCOME_IN_MAN_YEN),
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Ok(()),
            },
            TestCase {
                name: "invalid equal_or_less in annual_income_in_man_yen negative value"
                    .to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: Some(-1),
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Err(
                    CareerParamValidationError::InvalidEqualOrLessInAnnualIncomInManYen {
                        value: -1,
                        min: 0,
                        max: MAX_ANNUAL_INCOME_IN_MAN_YEN,
                    },
                ),
            },
            TestCase {
                name: "valid equal_or_less in annual_income_in_man_yen max value".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: Some(MAX_ANNUAL_INCOME_IN_MAN_YEN + 1),
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Err(
                    CareerParamValidationError::InvalidEqualOrLessInAnnualIncomInManYen {
                        value: MAX_ANNUAL_INCOME_IN_MAN_YEN + 1,
                        min: 0,
                        max: MAX_ANNUAL_INCOME_IN_MAN_YEN,
                    },
                ),
            },
            TestCase {
                name: "valid equal_or_less == equal_or_more".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: Some(MAX_ANNUAL_INCOME_IN_MAN_YEN),
                        equal_or_less: Some(MAX_ANNUAL_INCOME_IN_MAN_YEN),
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Ok(()),
            },
            TestCase {
                name: "invalid equal_or_less exceeds equal_or_more".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: Some(1),
                        equal_or_less: Some(0),
                    },
                    is_manager: None,
                    position_name: None,
                    is_new_graduate: None,
                    note: None,
                },
                expected: Err(CareerParamValidationError::EqualOrMoreExceedsEqualOrLessInAnnualIncomInManYen { equal_or_more: 1, equal_or_less: 0 }),
            },
            TestCase {
                name: "invalid length position name".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: Some("".to_string()),
                    is_new_graduate: None,
                    note: None,
                },
                expected: Err(CareerParamValidationError::InvalidPositionNameLength {
                    length: 0,
                    min_length: POSITION_NAME_MIN_LENGTH,
                    max_length: POSITION_NAME_MAX_LENGTH,
                }),
            },
            TestCase {
                name: "illegal char position name".to_string(),
                input: CareerParam {
                    company_name: None,
                    department_name: None,
                    office: None,
                    years_of_service: None,
                    employed: None,
                    contract_type: None,
                    profession: None,
                    annual_income_in_man_yen: AnnualInComeInManYenParam {
                        equal_or_more: None,
                        equal_or_less: None,
                    },
                    is_manager: None,
                    position_name: Some("’ or ‘A’=‘A".to_string()),
                    is_new_graduate: None,
                    note: None,
                },
                expected: Err(CareerParamValidationError::IllegalCharInPositionName(
                    "’ or ‘A’=‘A".to_string(),
                )),
            },
        ]
    });

    #[test]
    fn test_validate_career_param() {
        for test_case in TEST_CASE_SET.iter() {
            let result = validate_career_param(&test_case.input);
            let message = format!("test case \"{}\" failed", test_case.name.clone());
            assert_eq!(test_case.expected, result, "{}", message);
        }
    }
}
