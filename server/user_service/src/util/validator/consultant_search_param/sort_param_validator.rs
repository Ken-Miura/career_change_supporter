// Copyright 2022 Ken Miura

use std::{collections::HashSet, fmt::Display};

use once_cell::sync::Lazy;

use crate::consultants_search::SortParam;

static KEY_SET: Lazy<HashSet<String>> = Lazy::new(|| {
    let mut set: HashSet<String> = HashSet::with_capacity(2);
    set.insert("fee_per_hour_in_yen".to_string());
    set.insert("rating".to_string());
    set
});

static ORDER_SET: Lazy<HashSet<String>> = Lazy::new(|| {
    let mut set: HashSet<String> = HashSet::with_capacity(2);
    set.insert("ascending".to_string());
    set.insert("descending".to_string());
    set
});

pub(crate) fn validate_sort_param(sort_param: &SortParam) -> Result<(), SortParamError> {
    if !KEY_SET.contains(sort_param.key.as_str()) {
        return Err(SortParamError::InvalidKey(sort_param.key.clone()));
    }
    if !ORDER_SET.contains(sort_param.order.as_str()) {
        return Err(SortParamError::InvalidOrder(sort_param.order.clone()));
    }
    Ok(())
}

/// Error related to [validate_sort_param()]
#[derive(Debug, PartialEq)]
pub(crate) enum SortParamError {
    InvalidKey(String),
    InvalidOrder(String),
}

impl Display for SortParamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortParamError::InvalidKey(key) => write!(f, "invalid sort_param (key: {})", key),
            SortParamError::InvalidOrder(order) => {
                write!(f, "invalid sort_param (order: {})", order)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;

    use crate::consultants_search::SortParam;

    use super::{validate_sort_param, SortParamError};

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: SortParam,
        expected: Result<(), SortParamError>,
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        vec![
            TestCase {
                name: "fee_per_hour_in_yen ascending".to_string(),
                input: SortParam {
                    key: "fee_per_hour_in_yen".to_string(),
                    order: "ascending".to_string(),
                },
                expected: Ok(()),
            },
            TestCase {
                name: "fee_per_hour_in_yen descending".to_string(),
                input: SortParam {
                    key: "fee_per_hour_in_yen".to_string(),
                    order: "descending".to_string(),
                },
                expected: Ok(()),
            },
            TestCase {
                name: "rating ascending".to_string(),
                input: SortParam {
                    key: "rating".to_string(),
                    order: "ascending".to_string(),
                },
                expected: Ok(()),
            },
            TestCase {
                name: "rating descending".to_string(),
                input: SortParam {
                    key: "rating".to_string(),
                    order: "descending".to_string(),
                },
                expected: Ok(()),
            },
        ]
    });

    #[test]
    fn test_validate_sort_param() {
        for test_case in TEST_CASE_SET.iter() {
            let result = validate_sort_param(&test_case.input);
            let message = format!("test case \"{}\" failed", test_case.name.clone());
            assert_eq!(test_case.expected, result, "{}", message);
        }
    }
}
