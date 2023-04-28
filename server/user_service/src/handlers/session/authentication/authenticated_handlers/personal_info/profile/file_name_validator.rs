// Copyright 2021 Ken Miura

use std::{error::Error, fmt::Display};

use once_cell::sync::Lazy;
use regex::Regex;

// JPEGを示すファイル名（任意の一文字以上＋拡張子）
// 拡張子として.jfif, pjpeg, pjpはサポートしない
const JPEG_FILE_NAME_REGEXP: &str = r"^.+(\.jpg|\.jpeg|\.JPG|\.JPEG|\.jpe)$";
static JPEG_FILE_NAME_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(JPEG_FILE_NAME_REGEXP).expect("failed to compile jpeg file name regexp")
});

pub(super) fn validate_extension_is_jpeg(file_name: &str) -> Result<(), FileNameValidationError> {
    if !JPEG_FILE_NAME_RE.is_match(file_name) {
        return Err(FileNameValidationError::NotJpegExtension(
            file_name.to_string(),
        ));
    };
    Ok(())
}

/// Error related to file name
#[derive(Debug, PartialEq)]
pub(super) enum FileNameValidationError {
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

    use std::collections::HashSet;

    use super::*;

    static JPEG_EXTENTSION_SET: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(5);
        set.insert(".jpg".to_string());
        set.insert(".jpeg".to_string());
        set.insert(".JPG".to_string());
        set.insert(".JPEG".to_string());
        set.insert(".jpe".to_string());
        set
    });

    static JPEG_EXTENTSION_SET_WITHOUT_PERIOD: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(5);
        set.insert("jpg".to_string());
        set.insert("jpeg".to_string());
        set.insert("JPG".to_string());
        set.insert("JPEG".to_string());
        set.insert("jpe".to_string());
        set
    });

    // JPEG以外の拡張子を列挙する。必要に応じて追加する。
    static EXTENTSION_SET_EXCEPT_JPEG: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(11);
        set.insert(".txt".to_string());
        set.insert(".png".to_string());
        set.insert(".bmp".to_string());
        set.insert(".exe".to_string());
        set.insert(".htm".to_string());
        set.insert(".html".to_string());
        set.insert(".css".to_string());
        set.insert(".js".to_string());
        set.insert(".py".to_string());
        set.insert(".rb".to_string());
        set.insert(".sql".to_string());
        set
    });

    #[test]
    fn validate_extension_is_jpeg_returns_ok_if_file_name_ends_with_supported_ext() {
        for ext in JPEG_EXTENTSION_SET.iter() {
            let file_name = "test".to_string() + ext;
            validate_extension_is_jpeg(&file_name).expect("failed to get Ok");
        }
    }

    #[test]
    fn validate_extension_is_jpeg_returns_err_if_file_name_is_only_ext() {
        for ext in JPEG_EXTENTSION_SET.iter() {
            let err = validate_extension_is_jpeg(ext).expect_err("failed to get Err");
            assert_eq!(
                FileNameValidationError::NotJpegExtension(ext.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_extension_is_jpeg_returns_ok_if_hidden_file_name_ends_with_supported_ext() {
        for ext in JPEG_EXTENTSION_SET.iter() {
            let file_name = ".test".to_string() + ext;
            validate_extension_is_jpeg(&file_name).expect("failed to get Ok");
        }
    }

    #[test]
    fn validate_extension_is_jpeg_returns_err_if_file_name_ends_with_ext_other_than_jpeg() {
        for ext in EXTENTSION_SET_EXCEPT_JPEG.iter() {
            let file_name = "test".to_string() + ext;
            let err = validate_extension_is_jpeg(&file_name).expect_err("failed to get Err");
            assert_eq!(
                FileNameValidationError::NotJpegExtension(file_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_extension_is_jpeg_returns_err_if_file_name_does_not_end_with_jpeg_ext() {
        for ext in JPEG_EXTENTSION_SET.iter() {
            let file_name = "a".to_string() + ext + "b";
            let err = validate_extension_is_jpeg(&file_name).expect_err("failed to get Err");
            assert_eq!(
                FileNameValidationError::NotJpegExtension(file_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_extension_is_jpeg_returns_err_if_file_name_starts_with_jpeg_ext() {
        for ext in JPEG_EXTENTSION_SET.iter() {
            let file_name = ext.to_string() + "a";
            let err = validate_extension_is_jpeg(&file_name).expect_err("failed to get Err");
            assert_eq!(
                FileNameValidationError::NotJpegExtension(file_name.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_extension_is_jpeg_returns_err_if_file_name_ends_with_jpeg_ext_without_period() {
        for ext_without_period in JPEG_EXTENTSION_SET_WITHOUT_PERIOD.iter() {
            let file_name = "test".to_string() + ext_without_period;
            let err = validate_extension_is_jpeg(&file_name).expect_err("failed to get Err");
            assert_eq!(
                FileNameValidationError::NotJpegExtension(file_name.to_string()),
                err
            );
        }
    }
}
