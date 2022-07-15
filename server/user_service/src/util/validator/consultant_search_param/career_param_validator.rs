// Copyright 2022 Ken Miura

use std::{collections::HashSet, fmt::Display};

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
                CareerParamValidationError::InvalidEqualOrLessInAnnualIncomInManYen {
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
                CareerParamValidationError::InvalidEqualOrMoreInAnnualIncomInManYen {
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
