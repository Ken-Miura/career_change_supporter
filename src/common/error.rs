// Copyright 2021 Ken Miura

pub mod handled;
pub mod unexpected;

use crate::common;
use actix_web::{dev, http};
use derive_more::Display;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct ErrorInformation {
    pub code: i32,
    pub message: String,
}

#[derive(Display, Debug)]
pub(crate) enum Error {
    #[display(fmt = "handled error: {}", _0)]
    Handled(common::error::handled::Error),
    #[display(fmt = "unexpected error: {}", _0)]
    Unexpected(common::error::unexpected::Error),
}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> http::StatusCode {
        match self {
            Error::Handled(e) => match e {
                common::error::handled::Error::InvalidEmailAddressLength(_) => {
                    http::StatusCode::BAD_REQUEST
                }
                common::error::handled::Error::InvalidEmailAddressFormat(_) => {
                    http::StatusCode::BAD_REQUEST
                }
                common::error::handled::Error::InvalidPasswordLength(_) => {
                    http::StatusCode::BAD_REQUEST
                }
                common::error::handled::Error::InvalidPasswordFormat(_) => {
                    http::StatusCode::BAD_REQUEST
                }
                common::error::handled::Error::PasswordConstraintsViolation(_) => {
                    http::StatusCode::BAD_REQUEST
                }
                common::error::handled::Error::PasswordNotMatch(_) => {
                    http::StatusCode::UNAUTHORIZED
                }
                common::error::handled::Error::AccountAlreadyExists(_) => {
                    http::StatusCode::CONFLICT
                }
                common::error::handled::Error::ReachLimitOfTemporaryAccount(_) => {
                    http::StatusCode::BAD_REQUEST
                }
                common::error::handled::Error::NoTemporaryAccountFound(_) => {
                    http::StatusCode::NOT_FOUND
                }
                common::error::handled::Error::TemporaryAccountExpired(_) => {
                    http::StatusCode::BAD_REQUEST
                }
                common::error::handled::Error::InvalidTemporaryAccountId(_) => {
                    http::StatusCode::BAD_REQUEST
                }
            },
            Error::Unexpected(_e) => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> actix_web::HttpResponse<dev::Body> {
        match self {
            Error::Handled(err) => {
                let code;
                let message;
                match err {
                    common::error::handled::Error::InvalidEmailAddressLength(e) => {
                        code = e.code;
                        message = format!("メールアドレスの長さが不正です (入力されたメールアドレスの長さ: {})。メールアドレスは{}文字以下である必要があります。", e.length, e.max_length);
                    }
                    common::error::handled::Error::InvalidEmailAddressFormat(e) => {
                        code = e.code;
                        message = format!("メールアドレスの形式が不正です (入力されたメールアドレス: {})。\"email.address@example.com\"のような形式で入力してください。", e.email_address);
                    }
                    common::error::handled::Error::InvalidPasswordLength(e) => {
                        code = e.code;
                        message = format!("パスワードの長さが不正です。パスワードは{}文字以上、{}文字以下である必要があります。", e.min_length, e.max_length);
                    }
                    common::error::handled::Error::InvalidPasswordFormat(e) => {
                        code = e.code;
                        message = format!("パスワードに使用できない文字が含まれています。パスワードに使用可能な文字は、半角英数字と記号です。");
                    }
                    common::error::handled::Error::PasswordConstraintsViolation(e) => {
                        code = e.code;
                        message = format!("不正な形式のパスワードです。パスワードは小文字、大文字、数字または記号の内、2種類以上を組み合わせる必要があります。");
                    }
                    common::error::handled::Error::PasswordNotMatch(e) => {
                        code = e.code;
                        message = format!("メールアドレス、もしくはパスワードが間違っています。");
                    }
                    common::error::handled::Error::AccountAlreadyExists(e) => {
                        code = e.code;
                        message = format!("{}は既に登録されています。", e.email_address);
                    }
                    common::error::handled::Error::ReachLimitOfTemporaryAccount(e) => {
                        code = e.code;
                        message = format!("アカウント作成を依頼できる回数の上限に達しました。一定の期間が過ぎた後、再度お試しください。");
                    }
                    common::error::handled::Error::NoTemporaryAccountFound(e) => {
                        code = e.code;
                        // TODO: httpsに変更する
                        let url = format!(
                            "http://{}:{}/temporary-accounts?id={}",
                            common::DOMAIN,
                            common::PORT,
                            e.id
                        );
                        message = format!("指定されたURL ({}) は存在しません。ブラウザに入力されているURLと、メール本文に記載されているURLが同じかご確認ください。", url);
                    }
                    common::error::handled::Error::TemporaryAccountExpired(e) => {
                        code = e.code;
                        // TODO: httpsに変更する
                        let url = format!(
                            "http://{}:{}/temporary-accounts?id={}",
                            common::DOMAIN,
                            common::PORT,
                            e.id
                        );
                        message = format!("指定されたURL ({}) は有効期限が過ぎています。お手数ですが、ユーザアカウント作成から再度作成手続きをお願いします。", url);
                    }
                    common::error::handled::Error::InvalidTemporaryAccountId(e) => {
                        code = e.code;
                        // TODO: httpsに変更する
                        let url = format!(
                            "http://{}:{}/temporary-accounts?id={}",
                            common::DOMAIN,
                            common::PORT,
                            e.id
                        );
                        message = format!("不正なURLです ({}) 。ブラウザに入力されているURLと、メール本文に記載されているURLが同じかご確認ください。", url);
                    }
                }
                return actix_web::HttpResponse::build(self.status_code())
                    .content_type("application/problem+json")
                    .json(ErrorInformation { code, message });
            }
            Error::Unexpected(_e) => {
                let code = common::error::unexpected::INTERNAL_SERVER_ERROR;
                let message =
                    String::from("サーバでエラーが発生しました。一定時間後、再度お試しください。");
                actix_web::HttpResponse::build(self.status_code())
                    .content_type("application/problem+json")
                    .json(ErrorInformation { code, message })
            }
        }
    }
}
