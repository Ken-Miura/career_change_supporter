// Copyright 2022 Ken Miura

use std::{collections::HashSet, fmt::Display};

use once_cell::sync::Lazy;

use super::search::SortParam;

static KEY_SET: Lazy<HashSet<String>> = Lazy::new(|| {
    let mut set: HashSet<String> = HashSet::with_capacity(2);
    set.insert("fee_per_hour_in_yen".to_string());
    set.insert("rating".to_string());
    set
});

static ORDER_SET: Lazy<HashSet<String>> = Lazy::new(|| {
    let mut set: HashSet<String> = HashSet::with_capacity(2);
    set.insert("asc".to_string());
    set.insert("desc".to_string());
    set
});

pub(super) fn validate_sort_param(sort_param: &SortParam) -> Result<(), SortParamError> {
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
pub(super) enum SortParamError {
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

    use crate::handlers::authenticated_handlers::{
        consultation::consultant::search::SortParam,
        tests::{CONTROL_CHAR_SET, SPACE_SET, SYMBOL_SET},
    };

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
                name: "fee_per_hour_in_yen asc".to_string(),
                input: SortParam {
                    key: "fee_per_hour_in_yen".to_string(),
                    order: "asc".to_string(),
                },
                expected: Ok(()),
            },
            TestCase {
                name: "fee_per_hour_in_yen desc".to_string(),
                input: SortParam {
                    key: "fee_per_hour_in_yen".to_string(),
                    order: "desc".to_string(),
                },
                expected: Ok(()),
            },
            TestCase {
                name: "rating asc".to_string(),
                input: SortParam {
                    key: "rating".to_string(),
                    order: "asc".to_string(),
                },
                expected: Ok(()),
            },
            TestCase {
                name: "rating desc".to_string(),
                input: SortParam {
                    key: "rating".to_string(),
                    order: "desc".to_string(),
                },
                expected: Ok(()),
            },
            TestCase {
                name: "invalid key".to_string(),
                input: SortParam {
                    key: "1' or '1' = '1';--".to_string(),
                    order: "asc".to_string(),
                },
                expected: Err(SortParamError::InvalidKey("1' or '1' = '1';--".to_string())),
            },
            TestCase {
                name: "invalid order".to_string(),
                input: SortParam {
                    key: "rating".to_string(),
                    order: "1' or '1' = '1';--".to_string(),
                },
                expected: Err(SortParamError::InvalidOrder(
                    "1' or '1' = '1';--".to_string(),
                )),
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

    #[test]
    fn validate_sort_param_returns_err_if_key_is_symbol() {
        for s in SYMBOL_SET.iter() {
            let param = SortParam {
                key: s.to_string(),
                order: "desc".to_string(),
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidKey(param.key)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_order_is_symbol() {
        for s in SYMBOL_SET.iter() {
            let param = SortParam {
                key: "fee_per_hour_in_yen".to_string(),
                order: s.to_string(),
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidOrder(param.order)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_key_starts_with_symbol() {
        for s in SYMBOL_SET.iter() {
            let param = SortParam {
                key: s.to_string() + "fee_per_hour_in_yen",
                order: "desc".to_string(),
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidKey(param.key)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_order_starts_with_symbol() {
        for s in SYMBOL_SET.iter() {
            let param = SortParam {
                key: "fee_per_hour_in_yen".to_string(),
                order: s.to_string() + "desc",
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidOrder(param.order)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_key_ends_with_symbol() {
        for s in SYMBOL_SET.iter() {
            let param = SortParam {
                key: "fee_per_hour_in_yen".to_string() + s,
                order: "desc".to_string(),
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidKey(param.key)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_order_ends_with_symbol() {
        for s in SYMBOL_SET.iter() {
            let param = SortParam {
                key: "fee_per_hour_in_yen".to_string(),
                order: "desc".to_string() + s,
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidOrder(param.order)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_key_includes_symbol() {
        for s in SYMBOL_SET.iter() {
            let param = SortParam {
                key: "fee_per_hour".to_string() + s + "_in_yen",
                order: "desc".to_string(),
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidKey(param.key)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_order_includes_symbol() {
        for s in SYMBOL_SET.iter() {
            let param = SortParam {
                key: "fee_per_hour_in_yen".to_string(),
                order: "desc".to_string() + s + "asc",
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidOrder(param.order)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_key_is_control_char() {
        for s in CONTROL_CHAR_SET.iter() {
            let param = SortParam {
                key: s.to_string(),
                order: "desc".to_string(),
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidKey(param.key)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_order_is_control_char() {
        for s in CONTROL_CHAR_SET.iter() {
            let param = SortParam {
                key: "fee_per_hour_in_yen".to_string(),
                order: s.to_string(),
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidOrder(param.order)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_key_starts_with_control_char() {
        for s in CONTROL_CHAR_SET.iter() {
            let param = SortParam {
                key: s.to_string() + "fee_per_hour_in_yen",
                order: "desc".to_string(),
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidKey(param.key)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_order_starts_with_control_char() {
        for s in CONTROL_CHAR_SET.iter() {
            let param = SortParam {
                key: "fee_per_hour_in_yen".to_string(),
                order: s.to_string() + "desc",
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidOrder(param.order)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_key_ends_with_control_char() {
        for s in CONTROL_CHAR_SET.iter() {
            let param = SortParam {
                key: "fee_per_hour_in_yen".to_string() + s,
                order: "desc".to_string(),
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidKey(param.key)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_order_ends_with_control_char() {
        for s in CONTROL_CHAR_SET.iter() {
            let param = SortParam {
                key: "fee_per_hour_in_yen".to_string(),
                order: "desc".to_string() + s,
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidOrder(param.order)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_key_includes_control_char() {
        for s in CONTROL_CHAR_SET.iter() {
            let param = SortParam {
                key: "fee_per_hour".to_string() + s + "_in_yen",
                order: "desc".to_string(),
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidKey(param.key)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_order_includes_control_char() {
        for s in CONTROL_CHAR_SET.iter() {
            let param = SortParam {
                key: "fee_per_hour_in_yen".to_string(),
                order: "desc".to_string() + s + "asc",
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidOrder(param.order)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_key_is_space() {
        for s in SPACE_SET.iter() {
            let param = SortParam {
                key: s.to_string(),
                order: "desc".to_string(),
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidKey(param.key)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_order_is_space() {
        for s in SPACE_SET.iter() {
            let param = SortParam {
                key: "fee_per_hour_in_yen".to_string(),
                order: s.to_string(),
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidOrder(param.order)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_key_starts_with_space() {
        for s in SPACE_SET.iter() {
            let param = SortParam {
                key: s.to_string() + "fee_per_hour_in_yen",
                order: "desc".to_string(),
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidKey(param.key)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_order_starts_with_space() {
        for s in SPACE_SET.iter() {
            let param = SortParam {
                key: "fee_per_hour_in_yen".to_string(),
                order: s.to_string() + "desc",
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidOrder(param.order)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_key_ends_with_space() {
        for s in SPACE_SET.iter() {
            let param = SortParam {
                key: "fee_per_hour_in_yen".to_string() + s,
                order: "desc".to_string(),
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidKey(param.key)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_order_ends_with_space() {
        for s in SPACE_SET.iter() {
            let param = SortParam {
                key: "fee_per_hour_in_yen".to_string(),
                order: "desc".to_string() + s,
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidOrder(param.order)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_key_includes_space() {
        for s in SPACE_SET.iter() {
            let param = SortParam {
                key: "fee_per_hour".to_string() + s + "_in_yen",
                order: "desc".to_string(),
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidKey(param.key)))
        }
    }

    #[test]
    fn validate_sort_param_returns_err_if_order_includes_space() {
        for s in SPACE_SET.iter() {
            let param = SortParam {
                key: "fee_per_hour_in_yen".to_string(),
                order: "desc".to_string() + s + "asc",
            };
            let result = validate_sort_param(&param);
            assert_eq!(result, Err(SortParamError::InvalidOrder(param.order)))
        }
    }
}
