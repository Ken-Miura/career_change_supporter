// Copyright 2022 Ken Miura

use std::io::Cursor;

pub(super) type FileNameAndBinary = (String, Cursor<Vec<u8>>);

/// 引数が存在する場合、ファイル名のみ複製を行う
pub(super) fn clone_file_name_if_exists(
    file_name_and_binary_option: Option<FileNameAndBinary>,
) -> (Option<FileNameAndBinary>, Option<String>) {
    if let Some(file_name_and_binary) = file_name_and_binary_option {
        let image2 = Some((file_name_and_binary.0.clone(), file_name_and_binary.1));
        let image2_file_name_without_ext = Some(file_name_and_binary.0);
        return (image2, image2_file_name_without_ext);
    };
    (None, None)
}

#[cfg(test)]
mod tests {

    use std::io::Cursor;

    use image::{ImageBuffer, ImageOutputFormat, RgbImage};

    use crate::handlers::authenticated_handlers::personal_info::profile::multipart::clone_file_name_if_exists;

    #[test]
    fn clone_file_name_if_exists_returns_none_if_none_is_passed() {
        let (ret1, ret2) = clone_file_name_if_exists(None);
        assert_eq!(None, ret1);
        assert_eq!(None, ret2);
    }

    #[test]
    fn clone_file_name_if_exists_returns_arg_and_file_name_if_value_is_passed() {
        let file_name = "c89bfd885f6df5fd-345306a47b7dd758";
        let binary = create_dummy_jpeg_image();
        let file_name_and_binary = (file_name.to_string(), binary);

        let (ret1, ret2) = clone_file_name_if_exists(Some(file_name_and_binary.clone()));

        assert_eq!(Some(file_name_and_binary), ret1);
        assert_eq!(Some(file_name.to_string()), ret2);
    }

    fn create_dummy_jpeg_image() -> Cursor<Vec<u8>> {
        let img: RgbImage = ImageBuffer::new(128, 128);
        let mut bytes = Cursor::new(Vec::with_capacity(50 * 1024));
        img.write_to(&mut bytes, ImageOutputFormat::Jpeg(85))
            .expect("failed to get Ok");
        bytes
    }
}
