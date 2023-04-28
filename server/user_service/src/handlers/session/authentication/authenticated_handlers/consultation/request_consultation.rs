// Copyright 2023 Ken Miura

pub(crate) mod begin;
mod consultation_date_time_validator;
pub(crate) mod fee_per_hour_in_yen_for_application;
pub(crate) mod finish;

const KEY_TO_CONSULTAND_ID_ON_CHARGE_OBJ: &str = "consultant_id";
const KEY_TO_FIRST_CANDIDATE_IN_JST_ON_CHARGE_OBJ: &str = "first_candidate_in_jst";
const KEY_TO_SECOND_CANDIDATE_IN_JST_ON_CHARGE_OBJ: &str = "second_candidate_in_jst";
const KEY_TO_THIRD_CANDIDATE_IN_JST_ON_CHARGE_OBJ: &str = "third_candidate_in_jst";
