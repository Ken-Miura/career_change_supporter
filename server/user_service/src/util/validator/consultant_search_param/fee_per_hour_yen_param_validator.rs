// Copyright 2022 Ken Miura

use std::fmt::Display;

use crate::{
    consultants_search::FeePerHourYenParam,
    util::{MAX_FEE_PER_HOUR_IN_YEN, MIN_FEE_PER_HOUR_IN_YEN},
};

pub(crate) fn validate_fee_per_hour_yen_param(
    fee_per_hour_in_yen: &FeePerHourYenParam,
) -> Result<(), FeePerHourYenParamError> {
    let equal_or_more_option = fee_per_hour_in_yen.equal_or_more;
    if let Some(equal_or_more) = equal_or_more_option {
        if !(MIN_FEE_PER_HOUR_IN_YEN..=MAX_FEE_PER_HOUR_IN_YEN).contains(&equal_or_more) {
            return Err(FeePerHourYenParamError::InvalidEqualOrMore {
                value: equal_or_more,
                min: MIN_FEE_PER_HOUR_IN_YEN,
                max: MAX_FEE_PER_HOUR_IN_YEN,
            });
        }
    }
    let equal_or_less_option = fee_per_hour_in_yen.equal_or_less;
    if let Some(equal_or_less) = equal_or_less_option {
        if !(MIN_FEE_PER_HOUR_IN_YEN..=MAX_FEE_PER_HOUR_IN_YEN).contains(&equal_or_less) {
            return Err(FeePerHourYenParamError::InvalidEqualOrLess {
                value: equal_or_less,
                min: MIN_FEE_PER_HOUR_IN_YEN,
                max: MAX_FEE_PER_HOUR_IN_YEN,
            });
        }
    }
    if let Some(equal_or_more) = equal_or_more_option {
        if let Some(equal_or_less) = equal_or_less_option {
            if equal_or_more > equal_or_less {
                return Err(FeePerHourYenParamError::EqualOrMoreExceedsEqualOrLess {
                    equal_or_more,
                    equal_or_less,
                });
            }
        }
    }
    Ok(())
}

/// Error related to [validate_fee_per_hour_yen_param()]
#[derive(Debug, PartialEq)]
pub(crate) enum FeePerHourYenParamError {
    InvalidEqualOrMore {
        value: i32,
        min: i32,
        max: i32,
    },
    InvalidEqualOrLess {
        value: i32,
        min: i32,
        max: i32,
    },
    EqualOrMoreExceedsEqualOrLess {
        equal_or_more: i32,
        equal_or_less: i32,
    },
}

impl Display for FeePerHourYenParamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeePerHourYenParamError::InvalidEqualOrMore { value, min, max } => write!(
                f,
                "invalid fee_per_hour_yen_param: equal_or_more (value: {}, min: {}, max {})",
                value, min, max
            ),
            FeePerHourYenParamError::InvalidEqualOrLess { value, min, max } => write!(
                f,
                "invalid fee_per_hour_yen_param: equal_or_less (value: {}, min: {}, max {})",
                value, min, max
            ),
            FeePerHourYenParamError::EqualOrMoreExceedsEqualOrLess { equal_or_more, equal_or_less } => write!(
              f,
              "invalid fee_per_hour_yen_param: equal_or_more exceeds equal_or_less (equal_or_more: {}, equal_or_less: {})",
              equal_or_more, equal_or_less
          ),
        }
    }
}

#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;

    use crate::{
        consultants_search::FeePerHourYenParam,
        util::{MAX_FEE_PER_HOUR_IN_YEN, MIN_FEE_PER_HOUR_IN_YEN},
    };

    use super::{validate_fee_per_hour_yen_param, FeePerHourYenParamError};

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: FeePerHourYenParam,
        expected: Result<(), FeePerHourYenParamError>,
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        vec![
            TestCase {
                name: "no parameters specified".to_string(),
                input: FeePerHourYenParam {
                    equal_or_more: None,
                    equal_or_less: None,
                },
                expected: Ok(()),
            },
            TestCase {
                name: "min equal_or_more".to_string(),
                input: FeePerHourYenParam {
                    equal_or_more: Some(MIN_FEE_PER_HOUR_IN_YEN),
                    equal_or_less: None,
                },
                expected: Ok(()),
            },
            TestCase {
                name: "max equal_or_more".to_string(),
                input: FeePerHourYenParam {
                    equal_or_more: Some(MAX_FEE_PER_HOUR_IN_YEN),
                    equal_or_less: None,
                },
                expected: Ok(()),
            },
            TestCase {
                name: "min equal_or_less".to_string(),
                input: FeePerHourYenParam {
                    equal_or_more: None,
                    equal_or_less: Some(MIN_FEE_PER_HOUR_IN_YEN),
                },
                expected: Ok(()),
            },
            TestCase {
                name: "max equal_or_less".to_string(),
                input: FeePerHourYenParam {
                    equal_or_more: None,
                    equal_or_less: Some(MAX_FEE_PER_HOUR_IN_YEN),
                },
                expected: Ok(()),
            },
            TestCase {
                name: "min equal_or_more and max equal_or_less".to_string(),
                input: FeePerHourYenParam {
                    equal_or_more: Some(MIN_FEE_PER_HOUR_IN_YEN),
                    equal_or_less: Some(MAX_FEE_PER_HOUR_IN_YEN),
                },
                expected: Ok(()),
            },
            TestCase {
                name: "equal_or_more == equal_or_less".to_string(),
                input: FeePerHourYenParam {
                    equal_or_more: Some(MIN_FEE_PER_HOUR_IN_YEN),
                    equal_or_less: Some(MIN_FEE_PER_HOUR_IN_YEN),
                },
                expected: Ok(()),
            },
            TestCase {
                name: "invalid equal_or_more 1".to_string(),
                input: FeePerHourYenParam {
                    equal_or_more: Some(MIN_FEE_PER_HOUR_IN_YEN - 1),
                    equal_or_less: None,
                },
                expected: Err(FeePerHourYenParamError::InvalidEqualOrMore {
                    value: MIN_FEE_PER_HOUR_IN_YEN - 1,
                    min: MIN_FEE_PER_HOUR_IN_YEN,
                    max: MAX_FEE_PER_HOUR_IN_YEN,
                }),
            },
            TestCase {
                name: "invalid equal_or_more 2".to_string(),
                input: FeePerHourYenParam {
                    equal_or_more: Some(MAX_FEE_PER_HOUR_IN_YEN + 1),
                    equal_or_less: None,
                },
                expected: Err(FeePerHourYenParamError::InvalidEqualOrMore {
                    value: MAX_FEE_PER_HOUR_IN_YEN + 1,
                    min: MIN_FEE_PER_HOUR_IN_YEN,
                    max: MAX_FEE_PER_HOUR_IN_YEN,
                }),
            },
            TestCase {
                name: "invalid equal_or_less 1".to_string(),
                input: FeePerHourYenParam {
                    equal_or_more: None,
                    equal_or_less: Some(MIN_FEE_PER_HOUR_IN_YEN - 1),
                },
                expected: Err(FeePerHourYenParamError::InvalidEqualOrLess {
                    value: MIN_FEE_PER_HOUR_IN_YEN - 1,
                    min: MIN_FEE_PER_HOUR_IN_YEN,
                    max: MAX_FEE_PER_HOUR_IN_YEN,
                }),
            },
            TestCase {
                name: "invalid equal_or_less 2".to_string(),
                input: FeePerHourYenParam {
                    equal_or_more: None,
                    equal_or_less: Some(MAX_FEE_PER_HOUR_IN_YEN + 1),
                },
                expected: Err(FeePerHourYenParamError::InvalidEqualOrLess {
                    value: MAX_FEE_PER_HOUR_IN_YEN + 1,
                    min: MIN_FEE_PER_HOUR_IN_YEN,
                    max: MAX_FEE_PER_HOUR_IN_YEN,
                }),
            },
            TestCase {
                name: "equal_or_less exceeds equal_or_less".to_string(),
                input: FeePerHourYenParam {
                    equal_or_more: Some(MIN_FEE_PER_HOUR_IN_YEN + 200),
                    equal_or_less: Some(MIN_FEE_PER_HOUR_IN_YEN + 100),
                },
                expected: Err(FeePerHourYenParamError::EqualOrMoreExceedsEqualOrLess {
                    equal_or_more: MIN_FEE_PER_HOUR_IN_YEN + 200,
                    equal_or_less: MIN_FEE_PER_HOUR_IN_YEN + 100,
                }),
            },
        ]
    });

    #[test]
    fn test_validate_fee_per_hour_yen_param() {
        for test_case in TEST_CASE_SET.iter() {
            let result = validate_fee_per_hour_yen_param(&test_case.input);
            let message = format!("test case \"{}\" failed", test_case.name.clone());
            assert_eq!(test_case.expected, result, "{}", message);
        }
    }
}
