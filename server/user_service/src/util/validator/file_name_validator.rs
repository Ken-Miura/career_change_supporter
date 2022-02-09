// Copyright 2021 Ken Miura

use std::{error::Error, fmt::Display};

use once_cell::sync::Lazy;
use regex::Regex;

// JPEGを示すファイル名（任意の一文字以上＋拡張子）
// 拡張子として.jfif, pjpeg, pjpはサポートしない
const JPEG_FILE_NAME_REGEXP: &str = r"^.+[\.jpg|\.jpeg|\.JPG|\.JPEG|\.jpe]$";
static JPEG_FILE_NAME_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(JPEG_FILE_NAME_REGEXP).expect("failed to compile jpeg file name regexp")
});

pub(crate) fn validate_extension_is_jpeg(file_name: &str) -> Result<(), FileNameValidationError> {
    if !JPEG_FILE_NAME_RE.is_match(file_name) {
        return Err(FileNameValidationError::NotJpegExtension(
            file_name.to_string(),
        ));
    };
    Ok(())
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
mod tests {
    use super::validate_extension_is_jpeg;

    #[test]
    fn validate_extension_is_jpeg_returns_ok_if_file_name_ends_with_dot_jpg() {
        let _ = validate_extension_is_jpeg("test.jpg").expect("failed to get Ok");
    }

    #[test]
    fn validate_extension_is_jpeg_returns_ok_if_file_name_ends_with_dot_jpeg() {
        let _ = validate_extension_is_jpeg("test.jpeg").expect("failed to get Ok");
    }

    #[test]
    fn validate_extension_is_jpeg_returns_ok_if_file_name_ends_with_dot_upper_case_jpg() {
        let _ = validate_extension_is_jpeg("test.JPG").expect("failed to get Ok");
    }

    #[test]
    fn validate_extension_is_jpeg_returns_ok_if_file_name_ends_with_dot_upper_case_jpeg() {
        let _ = validate_extension_is_jpeg("test.JPEG").expect("failed to get Ok");
    }

    #[test]
    fn validate_extension_is_jpeg_returns_ok_if_file_name_ends_with_dot_jpe() {
        let _ = validate_extension_is_jpeg("test.jpe").expect("failed to get Ok");
    }
}
