// Copyright 2023 Ken Miura

use chrono::Datelike;
use common::{
    util::{Identity, Ymd},
    ErrResp,
};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::Serialize;
use tracing::error;

use crate::err::unexpected_err_resp;

pub(crate) mod admin;
pub(crate) mod career_request;
pub(crate) mod consultation;
mod document_operation;
pub(crate) mod identity_by_user_account_id;
pub(crate) mod identity_request;
pub(crate) mod maintenance;
pub(crate) mod pagination;
mod reason_validator;
pub(crate) mod refresh;
pub(crate) mod user_account;
mod user_account_operation;

async fn find_identity_by_user_account_id(
    pool: &DatabaseConnection,
    user_account_id: i64,
) -> Result<Option<Identity>, ErrResp> {
    let model = entity::identity::Entity::find_by_id(user_account_id)
        .one(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to find identity (user_account_id: {}): {}",
                user_account_id, e
            );
            unexpected_err_resp()
        })?;
    Ok(model.map(convert_identity))
}

fn convert_identity(identity_model: entity::identity::Model) -> Identity {
    let date = identity_model.date_of_birth;
    let ymd = Ymd {
        year: date.year(),
        month: date.month(),
        day: date.day(),
    };
    Identity {
        last_name: identity_model.last_name,
        first_name: identity_model.first_name,
        last_name_furigana: identity_model.last_name_furigana,
        first_name_furigana: identity_model.first_name_furigana,
        date_of_birth: ymd,
        prefecture: identity_model.prefecture,
        city: identity_model.city,
        address_line1: identity_model.address_line1,
        address_line2: identity_model.address_line2,
        telephone_number: identity_model.telephone_number,
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct Consultation {
    consultation_id: i64,
    user_account_id: i64,
    consultant_id: i64,
    meeting_at: String, // RFC 3339形式の文字列
    room_name: String,
    user_account_entered_at: Option<String>, // RFC 3339形式の文字列
    consultant_entered_at: Option<String>,   // RFC 3339形式の文字列
}

#[cfg(test)]
pub(super) mod tests {

    use axum::async_trait;

    use common::{smtp::SendMail, ErrResp};

    pub(super) struct SendMailMock {
        to: String,
        from: String,
        subject: String,
        text: String,
    }

    impl SendMailMock {
        pub(super) fn new(to: String, from: String, subject: String, text: String) -> Self {
            Self {
                to,
                from,
                subject,
                text,
            }
        }
    }

    #[async_trait]
    impl SendMail for SendMailMock {
        async fn send_mail(
            &self,
            to: &str,
            from: &str,
            subject: &str,
            text: &str,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.to, to);
            assert_eq!(self.from, from);
            assert_eq!(self.subject, subject);
            assert_eq!(self.text, text);
            Ok(())
        }
    }
}
