// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use common::{ApiError, ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE};
use entity::consultation;
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, Set, TransactionError,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use crate::err::{unexpected_err_resp, Code};
use crate::handlers::session::authentication::authenticated_handlers::authenticated_users::verified_user::VerifiedUser;
use crate::util::user_info::{FindUserInfoOperationImpl, UserInfo};

use super::{
    create_sky_way_auth_token, create_sky_way_auth_token_payload, ensure_audio_test_is_done,
    ensure_consultation_room_can_be_opened, get_consultation_with_exclusive_lock,
    validate_consultation_id_is_positive, Consultation, SkyWayIdentification,
    SKY_WAY_APPLICATION_ID, SKY_WAY_SECRET_KEY, VALID_TOKEN_DURATION_IN_SECONDS,
};

pub(crate) async fn get_consultant_side_info(
    VerifiedUser { user_info }: VerifiedUser,
    query: Query<ConsultantSideInfoQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<ConsultantSideInfoResult> {
    let consultation_id = query.0.consultation_id;
    let audio_test_done = query.0.audio_test_done;
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let identification = SkyWayIdentification {
        application_id: (*SKY_WAY_APPLICATION_ID).to_string(),
        secret: (*SKY_WAY_SECRET_KEY).to_string(),
    };
    let token_id = Uuid::new_v4().to_string();
    let op = ConsultantSideInfoOperationImpl { pool };
    handle_consultant_side_info(
        user_info.account_id,
        consultation_id,
        &current_date_time,
        identification,
        token_id.as_str(),
        audio_test_done,
        op,
    )
    .await
}

#[derive(Deserialize)]
pub(crate) struct ConsultantSideInfoQuery {
    consultation_id: i64,
    audio_test_done: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultantSideInfoResult {
    token: String,
    room_name: String,
    member_name: String,
}

async fn handle_consultant_side_info(
    account_id: i64,
    consultation_id: i64,
    current_date_time: &DateTime<FixedOffset>,
    identification: SkyWayIdentification,
    token_id: &str,
    audio_test_done: bool,
    op: impl ConsultantSideInfoOperation,
) -> RespResult<ConsultantSideInfoResult> {
    validate_consultation_id_is_positive(consultation_id)?;
    ensure_audio_test_is_done(audio_test_done)?;
    let result = get_consultation_by_consultation_id(consultation_id, &op).await?;
    ensure_consultant_id_is_valid(result.consultant_id, account_id)?;
    // 操作者（コンサルタント）のアカウントが無効化されているかどうかは個々のURLを示すハンドラに来る前の共通箇所でチェックする
    // 従って、アカウントが無効化されているかどうかは相談相手のみ確認する
    let _ = get_user_account_if_available(result.user_account_id, &op).await?;
    ensure_consultation_room_can_be_opened(
        current_date_time,
        &result.consultation_date_time_in_jst,
    )?;

    let expiration_date_time =
        *current_date_time + Duration::seconds(VALID_TOKEN_DURATION_IN_SECONDS);
    let payload = create_sky_way_auth_token_payload(
        token_id.to_string(),
        *current_date_time,
        expiration_date_time,
        identification.application_id,
        result.room_name.clone(),
        account_id.to_string(),
    )?;
    let token = create_sky_way_auth_token(&payload, identification.secret.as_bytes())?;

    op.update_consultant_entered_at_if_needed(consultation_id, *current_date_time)
        .await?;

    Ok((
        StatusCode::OK,
        Json(ConsultantSideInfoResult {
            token,
            room_name: result.room_name,
            member_name: account_id.to_string(),
        }),
    ))
}

#[async_trait]
trait ConsultantSideInfoOperation {
    async fn find_consultation_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Option<Consultation>, ErrResp>;

    async fn get_user_account_if_available(
        &self,
        user_account_id: i64,
    ) -> Result<Option<UserInfo>, ErrResp>;

    async fn update_consultant_entered_at_if_needed(
        &self,
        consultation_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct ConsultantSideInfoOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultantSideInfoOperation for ConsultantSideInfoOperationImpl {
    async fn find_consultation_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Option<Consultation>, ErrResp> {
        super::find_consultation_by_consultation_id(consultation_id, &self.pool).await
    }

    async fn get_user_account_if_available(
        &self,
        user_account_id: i64,
    ) -> Result<Option<UserInfo>, ErrResp> {
        let op = FindUserInfoOperationImpl::new(&self.pool);
        super::super::find_user_info_if_available(user_account_id, &op).await
    }

    async fn update_consultant_entered_at_if_needed(
        &self,
        consultation_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let result = get_consultation_with_exclusive_lock(consultation_id, txn).await?;
                    if result.consultant_entered_at.is_some() {
                        return Ok(());
                    }
                    update_consultant_entered_at(current_date_time, result, txn).await?;
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
                    error!(
                        "failed to update_consultant_entered_at_if_needed: {}",
                        err_resp_struct
                    );
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}

async fn get_consultation_by_consultation_id(
    consultation_id: i64,
    op: &impl ConsultantSideInfoOperation,
) -> Result<Consultation, ErrResp> {
    let consultation_option = op
        .find_consultation_by_consultation_id(consultation_id)
        .await?;
    if let Some(c) = consultation_option {
        Ok(c)
    } else {
        error!(
            "no consultation (consultation_id: {}) found",
            consultation_id
        );
        Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoConsultationFound as u32,
            }),
        ))
    }
}

fn ensure_consultant_id_is_valid(
    consultant_id_in_consultation: i64,
    consultant_id: i64,
) -> Result<(), ErrResp> {
    if consultant_id_in_consultation != consultant_id {
        error!(
            "consultant_id in consultation ({}) is not same as passed consultant_id ({})",
            consultant_id_in_consultation, consultant_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoConsultationFound as u32,
            }),
        ));
    }
    Ok(())
}

async fn get_user_account_if_available(
    user_account_id: i64,
    op: &impl ConsultantSideInfoOperation,
) -> Result<UserInfo, ErrResp> {
    let user = op.get_user_account_if_available(user_account_id).await?;
    user.ok_or_else(|| {
        error!("user ({}) is not available", user_account_id);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::TheOtherPersonAccountIsNotAvailable as u32,
            }),
        )
    })
}

async fn update_consultant_entered_at(
    current_date_time: DateTime<FixedOffset>,
    model: consultation::Model,
    txn: &DatabaseTransaction,
) -> Result<(), ErrRespStruct> {
    let consultation_id = model.consultation_id;
    let mut active_model: consultation::ActiveModel = model.into();
    active_model.consultant_entered_at = Set(Some(current_date_time));
    let _ = active_model.update(txn).await.map_err(|e| {
        error!("failed to update consultant_entered_at consultation (consultation_id: {}, current_date_time: {}): {}",
        consultation_id, current_date_time, e);
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum::{async_trait, Json};
    use chrono::{DateTime, Duration, FixedOffset, TimeZone};
    use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
    use once_cell::sync::Lazy;

    use super::super::tests::{
        CURRENT_DATE_TIME, DUMMY_APPLICATION_ID, DUMMY_SECRET, MEMBER_NAME, ROOM_NAME, TOKEN,
        TOKEN_ID,
    };
    use super::super::LEEWAY_IN_MINUTES;
    use super::super::{Consultation, SkyWayIdentification};
    use crate::err::Code;
    use crate::handlers::session::LENGTH_OF_MEETING_IN_MINUTE;
    use crate::util::user_info::UserInfo;

    use super::{
        handle_consultant_side_info, ConsultantSideInfoOperation, ConsultantSideInfoResult,
    };

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<ConsultantSideInfoResult>,
    }

    #[derive(Debug)]
    struct Input {
        account_id: i64,
        consultation_id: i64,
        current_date_time: DateTime<FixedOffset>,
        identification: SkyWayIdentification,
        token_id: String,
        audio_test_done: bool,
        op: ConsultantSideInfoOperationMock,
    }

    #[derive(Clone, Debug)]
    struct ConsultantSideInfoOperationMock {
        consultation_id: i64,
        consultation: Consultation,
        user_account: UserInfo,
        current_date_time: DateTime<FixedOffset>,
    }

    #[async_trait]
    impl ConsultantSideInfoOperation for ConsultantSideInfoOperationMock {
        async fn find_consultation_by_consultation_id(
            &self,
            consultation_id: i64,
        ) -> Result<Option<Consultation>, ErrResp> {
            if self.consultation_id != consultation_id {
                return Ok(None);
            }
            Ok(Some(self.consultation.clone()))
        }

        async fn get_user_account_if_available(
            &self,
            user_account_id: i64,
        ) -> Result<Option<UserInfo>, ErrResp> {
            if self.user_account.account_id != user_account_id {
                return Ok(None);
            }
            if self.user_account.disabled_at.is_some() {
                return Ok(None);
            }
            Ok(Some(self.user_account.clone()))
        }

        async fn update_consultant_entered_at_if_needed(
            &self,
            consultation_id: i64,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.consultation_id, consultation_id);
            assert_eq!(self.current_date_time, current_date_time);
            Ok(())
        }
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        let account_id_of_consultant = MEMBER_NAME.parse::<i64>().expect("failed to get Ok");
        let account_id_of_user = account_id_of_consultant + 6007;
        let consultation_id = 4134;
        let consultation_date_time_in_jst =
            *CURRENT_DATE_TIME + Duration::minutes(LEEWAY_IN_MINUTES); // LEEWAY_IN_MINUTES分前丁度はミーティングルームへ入れる
        let user_account_email_address = "test1@test.com";
        vec![
            TestCase {
                name: "success case 1".to_string(),
                input: Input {
                    account_id: account_id_of_consultant,
                    consultation_id,
                    current_date_time: *CURRENT_DATE_TIME,
                    identification: SkyWayIdentification {
                        application_id: DUMMY_APPLICATION_ID.to_string(),
                        secret: DUMMY_SECRET.to_string(),
                    },
                    token_id: TOKEN_ID.to_string(),
                    audio_test_done: true,
                    op: ConsultantSideInfoOperationMock {
                        consultation_id,
                        consultation: Consultation {
                            user_account_id: account_id_of_user,
                            consultant_id: account_id_of_consultant,
                            consultation_date_time_in_jst,
                            room_name: ROOM_NAME.to_string(),
                        },
                        user_account: UserInfo {
                            account_id: account_id_of_user,
                            email_address: user_account_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        },
                        current_date_time: *CURRENT_DATE_TIME,
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultantSideInfoResult {
                        token: TOKEN.to_string(),
                        room_name: ROOM_NAME.to_string(),
                        member_name: MEMBER_NAME.to_string(),
                    }),
                )),
            },
            TestCase {
                name: "success case 2 (consultation_date_time_in_jst is same as current_date_time)"
                    .to_string(),
                input: Input {
                    account_id: account_id_of_consultant,
                    consultation_id,
                    current_date_time: *CURRENT_DATE_TIME,
                    identification: SkyWayIdentification {
                        application_id: DUMMY_APPLICATION_ID.to_string(),
                        secret: DUMMY_SECRET.to_string(),
                    },
                    token_id: TOKEN_ID.to_string(),
                    audio_test_done: true,
                    op: ConsultantSideInfoOperationMock {
                        consultation_id,
                        consultation: Consultation {
                            user_account_id: account_id_of_user,
                            consultant_id: account_id_of_consultant,
                            consultation_date_time_in_jst: *CURRENT_DATE_TIME,
                            room_name: ROOM_NAME.to_string(),
                        },
                        user_account: UserInfo {
                            account_id: account_id_of_user,
                            email_address: user_account_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        },
                        current_date_time: *CURRENT_DATE_TIME,
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultantSideInfoResult {
                        token: TOKEN.to_string(),
                        room_name: ROOM_NAME.to_string(),
                        member_name: MEMBER_NAME.to_string(),
                    }),
                )),
            },
            TestCase {
                name: "success case 3 (current_date_time passed consultation_date_time_in_jst)"
                    .to_string(),
                input: Input {
                    account_id: account_id_of_consultant,
                    consultation_id,
                    current_date_time: *CURRENT_DATE_TIME,
                    identification: SkyWayIdentification {
                        application_id: DUMMY_APPLICATION_ID.to_string(),
                        secret: DUMMY_SECRET.to_string(),
                    },
                    token_id: TOKEN_ID.to_string(),
                    audio_test_done: true,
                    op: ConsultantSideInfoOperationMock {
                        consultation_id,
                        consultation: Consultation {
                            user_account_id: account_id_of_user,
                            consultant_id: account_id_of_consultant,
                            consultation_date_time_in_jst: *CURRENT_DATE_TIME
                                - Duration::minutes(10), // 現在時刻が相談開始時刻を過ぎていることを表したいだけで10分は適当な数字
                            room_name: ROOM_NAME.to_string(),
                        },
                        user_account: UserInfo {
                            account_id: account_id_of_user,
                            email_address: user_account_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        },
                        current_date_time: *CURRENT_DATE_TIME,
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultantSideInfoResult {
                        token: TOKEN.to_string(),
                        room_name: ROOM_NAME.to_string(),
                        member_name: MEMBER_NAME.to_string(),
                    }),
                )),
            },
            TestCase {
                name: "success case 4 (current_date_time is just end of consultation)".to_string(),
                input: Input {
                    account_id: account_id_of_consultant,
                    consultation_id,
                    current_date_time: *CURRENT_DATE_TIME,
                    identification: SkyWayIdentification {
                        application_id: DUMMY_APPLICATION_ID.to_string(),
                        secret: DUMMY_SECRET.to_string(),
                    },
                    token_id: TOKEN_ID.to_string(),
                    audio_test_done: true,
                    op: ConsultantSideInfoOperationMock {
                        consultation_id,
                        consultation: Consultation {
                            user_account_id: account_id_of_user,
                            consultant_id: account_id_of_consultant,
                            consultation_date_time_in_jst: *CURRENT_DATE_TIME
                                - Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64), // 相談終了時刻丁度は許容
                            room_name: ROOM_NAME.to_string(),
                        },
                        user_account: UserInfo {
                            account_id: account_id_of_user,
                            email_address: user_account_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        },
                        current_date_time: *CURRENT_DATE_TIME,
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultantSideInfoResult {
                        token: TOKEN.to_string(),
                        room_name: ROOM_NAME.to_string(),
                        member_name: MEMBER_NAME.to_string(),
                    }),
                )),
            },
            TestCase {
                name: "fail NonPositiveConsultationId (zero)".to_string(),
                input: Input {
                    account_id: account_id_of_consultant,
                    consultation_id: 0,
                    current_date_time: *CURRENT_DATE_TIME,
                    identification: SkyWayIdentification {
                        application_id: DUMMY_APPLICATION_ID.to_string(),
                        secret: DUMMY_SECRET.to_string(),
                    },
                    token_id: TOKEN_ID.to_string(),
                    audio_test_done: true,
                    op: ConsultantSideInfoOperationMock {
                        consultation_id: 0,
                        consultation: Consultation {
                            user_account_id: account_id_of_user,
                            consultant_id: account_id_of_consultant,
                            consultation_date_time_in_jst,
                            room_name: ROOM_NAME.to_string(),
                        },
                        user_account: UserInfo {
                            account_id: account_id_of_user,
                            email_address: user_account_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        },
                        current_date_time: *CURRENT_DATE_TIME,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NonPositiveConsultationId as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NonPositiveConsultationId (negative value)".to_string(),
                input: Input {
                    account_id: account_id_of_consultant,
                    consultation_id: -1,
                    current_date_time: *CURRENT_DATE_TIME,
                    identification: SkyWayIdentification {
                        application_id: DUMMY_APPLICATION_ID.to_string(),
                        secret: DUMMY_SECRET.to_string(),
                    },
                    token_id: TOKEN_ID.to_string(),
                    audio_test_done: true,
                    op: ConsultantSideInfoOperationMock {
                        consultation_id: -1,
                        consultation: Consultation {
                            user_account_id: account_id_of_user,
                            consultant_id: account_id_of_consultant,
                            consultation_date_time_in_jst,
                            room_name: ROOM_NAME.to_string(),
                        },
                        user_account: UserInfo {
                            account_id: account_id_of_user,
                            email_address: user_account_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        },
                        current_date_time: *CURRENT_DATE_TIME,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NonPositiveConsultationId as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail AudioTestIsNotDone".to_string(),
                input: Input {
                    account_id: account_id_of_consultant,
                    consultation_id,
                    current_date_time: *CURRENT_DATE_TIME,
                    identification: SkyWayIdentification {
                        application_id: DUMMY_APPLICATION_ID.to_string(),
                        secret: DUMMY_SECRET.to_string(),
                    },
                    token_id: TOKEN_ID.to_string(),
                    audio_test_done: false,
                    op: ConsultantSideInfoOperationMock {
                        consultation_id,
                        consultation: Consultation {
                            user_account_id: account_id_of_user,
                            consultant_id: account_id_of_consultant,
                            consultation_date_time_in_jst,
                            room_name: ROOM_NAME.to_string(),
                        },
                        user_account: UserInfo {
                            account_id: account_id_of_user,
                            email_address: user_account_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        },
                        current_date_time: *CURRENT_DATE_TIME,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::AudioTestIsNotDone as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NoConsultationFound (really not found)".to_string(),
                input: Input {
                    account_id: account_id_of_consultant,
                    consultation_id,
                    current_date_time: *CURRENT_DATE_TIME,
                    identification: SkyWayIdentification {
                        application_id: DUMMY_APPLICATION_ID.to_string(),
                        secret: DUMMY_SECRET.to_string(),
                    },
                    token_id: TOKEN_ID.to_string(),
                    audio_test_done: true,
                    op: ConsultantSideInfoOperationMock {
                        consultation_id: consultation_id + 1,
                        consultation: Consultation {
                            user_account_id: account_id_of_user,
                            consultant_id: account_id_of_consultant,
                            consultation_date_time_in_jst,
                            room_name: ROOM_NAME.to_string(),
                        },
                        user_account: UserInfo {
                            account_id: account_id_of_user,
                            email_address: user_account_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        },
                        current_date_time: *CURRENT_DATE_TIME,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoConsultationFound as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NoConsultationFound (consultant id does not match)".to_string(),
                input: Input {
                    account_id: account_id_of_consultant,
                    consultation_id,
                    current_date_time: *CURRENT_DATE_TIME,
                    identification: SkyWayIdentification {
                        application_id: DUMMY_APPLICATION_ID.to_string(),
                        secret: DUMMY_SECRET.to_string(),
                    },
                    token_id: TOKEN_ID.to_string(),
                    audio_test_done: true,
                    op: ConsultantSideInfoOperationMock {
                        consultation_id,
                        consultation: Consultation {
                            user_account_id: account_id_of_user,
                            consultant_id: account_id_of_consultant + 42,
                            consultation_date_time_in_jst,
                            room_name: ROOM_NAME.to_string(),
                        },
                        user_account: UserInfo {
                            account_id: account_id_of_user,
                            email_address: user_account_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        },
                        current_date_time: *CURRENT_DATE_TIME,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoConsultationFound as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail TheOtherPersonAccountIsNotAvailable (user not found)".to_string(),
                input: Input {
                    account_id: account_id_of_consultant,
                    consultation_id,
                    current_date_time: *CURRENT_DATE_TIME,
                    identification: SkyWayIdentification {
                        application_id: DUMMY_APPLICATION_ID.to_string(),
                        secret: DUMMY_SECRET.to_string(),
                    },
                    token_id: TOKEN_ID.to_string(),
                    audio_test_done: true,
                    op: ConsultantSideInfoOperationMock {
                        consultation_id,
                        consultation: Consultation {
                            user_account_id: account_id_of_user,
                            consultant_id: account_id_of_consultant,
                            consultation_date_time_in_jst,
                            room_name: ROOM_NAME.to_string(),
                        },
                        user_account: UserInfo {
                            account_id: account_id_of_user + 6501,
                            email_address: user_account_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        },
                        current_date_time: *CURRENT_DATE_TIME,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::TheOtherPersonAccountIsNotAvailable as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail TheOtherPersonAccountIsNotAvailable (user disabled)".to_string(),
                input: Input {
                    account_id: account_id_of_consultant,
                    consultation_id,
                    current_date_time: *CURRENT_DATE_TIME,
                    identification: SkyWayIdentification {
                        application_id: DUMMY_APPLICATION_ID.to_string(),
                        secret: DUMMY_SECRET.to_string(),
                    },
                    token_id: TOKEN_ID.to_string(),
                    audio_test_done: true,
                    op: ConsultantSideInfoOperationMock {
                        consultation_id,
                        consultation: Consultation {
                            user_account_id: account_id_of_user,
                            consultant_id: account_id_of_consultant,
                            consultation_date_time_in_jst,
                            room_name: ROOM_NAME.to_string(),
                        },
                        user_account: UserInfo {
                            account_id: account_id_of_user,
                            email_address: user_account_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: Some(
                                JAPANESE_TIME_ZONE
                                    .with_ymd_and_hms(2023, 1, 30, 6, 2, 30)
                                    .unwrap(),
                            ),
                        },
                        current_date_time: *CURRENT_DATE_TIME,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::TheOtherPersonAccountIsNotAvailable as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail ConsultationRoomHasNotOpenedYet".to_string(),
                input: Input {
                    account_id: account_id_of_consultant,
                    consultation_id,
                    current_date_time: *CURRENT_DATE_TIME,
                    identification: SkyWayIdentification {
                        application_id: DUMMY_APPLICATION_ID.to_string(),
                        secret: DUMMY_SECRET.to_string(),
                    },
                    token_id: TOKEN_ID.to_string(),
                    audio_test_done: true,
                    op: ConsultantSideInfoOperationMock {
                        consultation_id,
                        consultation: Consultation {
                            user_account_id: account_id_of_user,
                            consultant_id: account_id_of_consultant,
                            consultation_date_time_in_jst: *CURRENT_DATE_TIME
                                + Duration::minutes(LEEWAY_IN_MINUTES)
                                + Duration::seconds(1),
                            room_name: ROOM_NAME.to_string(),
                        },
                        user_account: UserInfo {
                            account_id: account_id_of_user,
                            email_address: user_account_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        },
                        current_date_time: *CURRENT_DATE_TIME,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::ConsultationRoomHasNotOpenedYet as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail ConsultationRoomHasAlreadyClosed".to_string(),
                input: Input {
                    account_id: account_id_of_consultant,
                    consultation_id,
                    current_date_time: *CURRENT_DATE_TIME,
                    identification: SkyWayIdentification {
                        application_id: DUMMY_APPLICATION_ID.to_string(),
                        secret: DUMMY_SECRET.to_string(),
                    },
                    token_id: TOKEN_ID.to_string(),
                    audio_test_done: true,
                    op: ConsultantSideInfoOperationMock {
                        consultation_id,
                        consultation: Consultation {
                            user_account_id: account_id_of_user,
                            consultant_id: account_id_of_consultant,
                            consultation_date_time_in_jst: *CURRENT_DATE_TIME
                                - Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
                                - Duration::seconds(1),
                            room_name: ROOM_NAME.to_string(),
                        },
                        user_account: UserInfo {
                            account_id: account_id_of_user,
                            email_address: user_account_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        },
                        current_date_time: *CURRENT_DATE_TIME,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::ConsultationRoomHasAlreadyClosed as u32,
                    }),
                )),
            },
        ]
    });

    #[tokio::test]
    async fn handle_consultant_side_info_tests() {
        for test_case in TEST_CASE_SET.iter() {
            let account_id = test_case.input.account_id;
            let consultation_id = test_case.input.consultation_id;
            let current_date_time = test_case.input.current_date_time;
            let identification = test_case.input.identification.clone();
            let token_id = test_case.input.token_id.clone();
            let audio_test_done = test_case.input.audio_test_done;
            let op = test_case.input.op.clone();

            let result = handle_consultant_side_info(
                account_id,
                consultation_id,
                &current_date_time,
                identification,
                token_id.as_str(),
                audio_test_done,
                op,
            )
            .await;

            let message = format!("test case \"{}\" failed", test_case.name.clone());
            if test_case.expected.is_ok() {
                let resp = result.expect("failed to get Ok");
                let expected = test_case.expected.as_ref().expect("failed to get Ok");
                assert_eq!(expected.0, resp.0, "{}", message);
                assert_eq!(expected.1 .0, resp.1 .0, "{}", message);
            } else {
                let resp = result.expect_err("failed to get Err");
                let expected = test_case.expected.as_ref().expect_err("failed to get Err");
                assert_eq!(expected.0, resp.0, "{}", message);
                assert_eq!(expected.1 .0, resp.1 .0, "{}", message);
            }
        }
    }
}
