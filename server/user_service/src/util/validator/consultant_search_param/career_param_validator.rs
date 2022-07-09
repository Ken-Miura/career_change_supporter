// Copyright 2022 Ken Miura

use std::fmt::Display;

use crate::consultants_search::CareerParam;

pub(crate) fn validate_career_param(career_param: &CareerParam) -> Result<(), CareerParamError> {
    todo!()
}

/// Error related to [validate_career_param()]
#[derive(Debug, PartialEq)]
pub(crate) enum CareerParamError {}

impl Display for CareerParamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
