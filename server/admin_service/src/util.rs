// Copyright 2021 Ken Miura

pub(crate) mod session;

pub(crate) const ROOT_PATH: &str = "/admin/api";

/// PAY.JPにアクセスするための情報を保持する変数
// pub(crate) static ACCESS_INFO: Lazy<AccessInfo> = Lazy::new(|| {
//     let url_without_path = var(KEY_TO_PAYMENT_PLATFORM_API_URL).unwrap_or_else(|_| {
//         panic!(
//             "Not environment variable found: environment variable \"{}\" must be set",
//             KEY_TO_PAYMENT_PLATFORM_API_URL
//         )
//     });
//     let username = var(KEY_TO_PAYMENT_PLATFORM_API_USERNAME).unwrap_or_else(|_| {
//         panic!(
//             "Not environment variable found: environment variable \"{}\" must be set",
//             KEY_TO_PAYMENT_PLATFORM_API_USERNAME
//         )
//     });
//     let password = var(KEY_TO_PAYMENT_PLATFORM_API_PASSWORD).unwrap_or_else(|_| {
//         panic!(
//             "Not environment variable found: environment variable \"{}\" must be set",
//             KEY_TO_PAYMENT_PLATFORM_API_PASSWORD
//         )
//     });
//     let access_info = AccessInfo::new(url_without_path, username, password);
//     access_info.expect("failed to get Ok")
// });

/// テストコードで共通で使うコードをまとめるモジュール
#[cfg(test)]
pub(crate) mod tests {
    // use common::{smtp::SendMail, ErrResp};

    // pub(crate) struct SendMailMock {
    //     to: String,
    //     from: String,
    //     subject: String,
    //     text: String,
    // }

    // impl SendMailMock {
    //     pub(crate) fn new(to: String, from: String, subject: String, text: String) -> Self {
    //         Self {
    //             to,
    //             from,
    //             subject,
    //             text,
    //         }
    //     }
    // }

    // impl SendMail for SendMailMock {
    //     fn send_mail(
    //         &self,
    //         to: &str,
    //         from: &str,
    //         subject: &str,
    //         text: &str,
    //     ) -> Result<(), ErrResp> {
    //         assert_eq!(self.to, to);
    //         assert_eq!(self.from, from);
    //         assert_eq!(self.subject, subject);
    //         assert_eq!(self.text, text);
    //         Ok(())
    //     }
    // }
}
