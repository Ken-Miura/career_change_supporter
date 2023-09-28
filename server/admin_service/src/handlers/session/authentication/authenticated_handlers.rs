// Copyright 2023 Ken Miura

use chrono::{DateTime, Datelike, FixedOffset};
use common::{
    util::{Identity, Ymd},
    ErrResp, ErrRespStruct,
};
use entity::sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, DatabaseConnection, DatabaseTransaction, EntityTrait,
    QuerySelect, Set, TransactionError, TransactionTrait,
};
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
pub(crate) mod news;
pub(crate) mod pagination;
mod reason_validator;
pub(crate) mod refresh;
pub(crate) mod user_account;
mod user_account_operation;
pub(crate) mod waiting_for_payment;

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

async fn move_to_stopped_settlement(
    pool: &DatabaseConnection,
    settlement_id: i64,
    current_date_time: DateTime<FixedOffset>,
) -> Result<(), ErrResp> {
    pool.transaction::<_, (), ErrRespStruct>(|txn| {
        Box::pin(async move {
            let s = find_settlement_with_exclusive_lock(settlement_id, txn).await?;

            let ss = entity::stopped_settlement::ActiveModel {
                stopped_settlement_id: NotSet,
                consultation_id: Set(s.consultation_id),
                charge_id: Set(s.charge_id.clone()),
                fee_per_hour_in_yen: Set(s.fee_per_hour_in_yen),
                platform_fee_rate_in_percentage: Set(s.platform_fee_rate_in_percentage.clone()),
                credit_facilities_expired_at: Set(s.credit_facilities_expired_at),
                stopped_at: Set(current_date_time),
            };
            let _ = ss.insert(txn).await.map_err(|e| {
                error!(
                    "failed to insert stopped_settlement (settlement: {:?}): {}",
                    s, e,
                );
                ErrRespStruct {
                    err_resp: unexpected_err_resp(),
                }
            })?;

            let _ = entity::settlement::Entity::delete_by_id(settlement_id)
                .exec(txn)
                .await
                .map_err(|e| {
                    error!(
                        "failed to delete settlement (settlement_id: {}): {}",
                        settlement_id, e,
                    );
                    ErrRespStruct {
                        err_resp: unexpected_err_resp(),
                    }
                })?;

            Ok(())
        })
    })
    .await
    .map_err(|e| match e {
        TransactionError::Connection(db_err) => {
            error!("connection error: {}", db_err);
            unexpected_err_resp()
        }
        TransactionError::Transaction(err_resp_struct) => {
            error!("failed to move_to_stopped_settlement: {}", err_resp_struct);
            err_resp_struct.err_resp
        }
    })?;
    Ok(())
}

async fn find_settlement_with_exclusive_lock(
    settlement_id: i64,
    txn: &DatabaseTransaction,
) -> Result<entity::settlement::Model, ErrRespStruct> {
    let result = entity::settlement::Entity::find_by_id(settlement_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find settlement (settlement_id: {}): {}",
                settlement_id, e,
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    let model = result.ok_or_else(|| {
        error!("no settlement (settlement_id: {}) found", settlement_id,);
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(model)
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
