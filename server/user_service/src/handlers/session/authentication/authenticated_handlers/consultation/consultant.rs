// Copyright 2023 Ken Miura

mod career_param_validator;
pub(crate) mod detail;
mod fee_per_hour_in_yen_param_validator;
pub(crate) mod search;
mod sort_param_validator;

const VALID_YEARS_OF_SERVICE_PERIOD_THREE: i32 = 3;
const VALID_YEARS_OF_SERVICE_PERIOD_FIVE: i32 = 5;
const VALID_YEARS_OF_SERVICE_PERIOD_TEN: i32 = 10;
const VALID_YEARS_OF_SERVICE_PERIOD_FIFTEEN: i32 = 15;
const VALID_YEARS_OF_SERVICE_PERIOD_TWENTY: i32 = 20;
