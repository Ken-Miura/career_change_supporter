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
mod tests {}
