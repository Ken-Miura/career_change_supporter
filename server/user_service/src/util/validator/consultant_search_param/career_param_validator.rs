// Copyright 2022 Ken Miura

use std::fmt::Display;

use crate::consultants_search::CareerParam;

pub(crate) fn validate_career_param(career_param: &CareerParam) -> Result<(), CareerParamError> {
    todo!()
}

/// Error related to [validate_career_param()]
#[derive(Debug, PartialEq)]
pub(crate) enum CareerParamError {
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

impl Display for CareerParamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CareerParamError::InvalidCompanyNameLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid company_name length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            CareerParamError::IllegalCharInCompanyName(company_name) => {
                write!(
                    f,
                    "company_name: illegal charcter included: {} (binary: {:X?})",
                    company_name,
                    company_name.as_bytes().to_vec()
                )
            }
            CareerParamError::InvalidDepartmentNameLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid department_name length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            CareerParamError::IllegalCharInDepartmentName(department_name) => {
                write!(
                    f,
                    "department_name: illegal charcter included: {} (binary: {:X?})",
                    department_name,
                    department_name.as_bytes().to_vec()
                )
            }
            CareerParamError::InvalidOfficeLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid office length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            CareerParamError::IllegalCharInOffice(office) => {
                write!(
                    f,
                    "office: illegal charcter included: {} (binary: {:X?})",
                    office,
                    office.as_bytes().to_vec()
                )
            }
            CareerParamError::IllegalYearsOfService(years_of_service) => {
                write!(f, "illegal years_of_service ({})", years_of_service)
            }
            CareerParamError::IllegalContractType(contract_type) => {
                write!(f, "illegal contract_type ({})", contract_type)
            }
            CareerParamError::InvalidProfessionLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid profession length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            CareerParamError::IllegalCharInProfession(profession) => {
                write!(
                    f,
                    "profession: illegal charcter included: {} (binary: {:X?})",
                    profession,
                    profession.as_bytes().to_vec()
                )
            }
            CareerParamError::InvalidEqualOrMoreInAnnualIncomInManYen { value, min, max } => {
                write!(
                    f,
                    "invalid equal_or_more (value: {}, min: {}, max {})",
                    value, min, max
                )
            }
            CareerParamError::InvalidEqualOrLessInAnnualIncomInManYen { value, min, max } => {
                write!(
                    f,
                    "invalid equal_or_less (value: {}, min: {}, max {})",
                    value, min, max
                )
            }
            CareerParamError::EqualOrMoreExceedsEqualOrLessInAnnualIncomInManYen {
                equal_or_more,
                equal_or_less,
            } => write!(
                f,
                "equal_or_more exceeds equal_or_less (equal_or_more: {}, equal_or_less: {})",
                equal_or_more, equal_or_less
            ),
            CareerParamError::InvalidPositionNameLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid position_name length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            CareerParamError::IllegalCharInPositionName(position_name) => {
                write!(
                    f,
                    "position_name: illegal charcter included: {} (binary: {:X?})",
                    position_name,
                    position_name.as_bytes().to_vec()
                )
            }
            CareerParamError::InvalidNoteLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid note length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            CareerParamError::IllegalCharInNote(note) => {
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
