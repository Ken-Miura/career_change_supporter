// Copyright 2021 Ken Miura

use std::{collections::HashSet, error::Error, fmt::Display};

use once_cell::sync::Lazy;

static JPEG_EXTENTSION_SET: Lazy<HashSet<String>> = Lazy::new(|| {
    let mut set: HashSet<String> = HashSet::with_capacity(47);
    set.insert(".jpg".to_string());
    set.insert(".jpeg".to_string());
    set.insert(".JPG".to_string());
    set.insert(".JPEG".to_string());
    set.insert(".jpe".to_string());
    set.insert(".jfif".to_string());
    set.insert(".pjpeg".to_string());
    set.insert(".pjp".to_string());
    set
});

pub(crate) fn validate_extension_is_jpeg(file_name: &str) -> Result<(), FileNameValidationError> {
    for jpeg_ext in JPEG_EXTENTSION_SET.iter() {
        if file_name.ends_with(jpeg_ext) {
            return Ok(());
        };
    }
    Err(FileNameValidationError::NotJpegExtension(
        file_name.to_string(),
    ))
}

/// Error related to file name
#[derive(Debug, PartialEq)]
pub(crate) enum FileNameValidationError {
    NotJpegExtension(String),
}

impl Display for FileNameValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileNameValidationError::NotJpegExtension(file_name) => {
                write!(
                    f,
                    "file name does not end with jpeg extension: {}",
                    file_name,
                )
            }
        }
    }
}

impl Error for FileNameValidationError {}

#[cfg(test)]
mod tests {}
