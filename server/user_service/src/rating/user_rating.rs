// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ApiError, ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE};
use entity::prelude::Consultation;
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, Set, TransactionError,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::err::{unexpected_err_resp, Code};
use crate::util::find_user_account_by_user_account_id_with_exclusive_lock;
use crate::util::session::verified_user::VerifiedUser;

use super::{
    ensure_end_of_consultation_date_time_has_passed, ensure_rating_id_is_positive,
    ensure_rating_is_in_valid_range, ConsultationInfo,
};

pub(crate) async fn post_user_rating(
    VerifiedUser { account_id }: VerifiedUser,
    State(pool): State<DatabaseConnection>,
    Json(req): Json<UserRatingParam>,
) -> RespResult<UserRatingResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = UserRatingOperationImpl { pool };
    handle_user_rating(
        account_id,
        req.user_rating_id,
        req.rating,
        &current_date_time,
        op,
    )
    .await
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct UserRatingParam {
    user_rating_id: i64,
    rating: i16,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct UserRatingResult {}

#[async_trait]
trait UserRatingOperation {
    async fn find_consultation_info_from_user_rating(
        &self,
        user_rating_id: i64,
    ) -> Result<Option<ConsultationInfo>, ErrResp>;

    async fn update_user_rating(
        &self,
        user_account_id: i64,
        user_rating_id: i64,
        rating: i16,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct UserRatingOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl UserRatingOperation for UserRatingOperationImpl {
    async fn find_consultation_info_from_user_rating(
        &self,
        user_rating_id: i64,
    ) -> Result<Option<ConsultationInfo>, ErrResp> {
        let model = entity::user_rating::Entity::find_by_id(user_rating_id)
            .find_also_related(Consultation)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find user_rating and consultation (user_rating_id: {}): {}",
                    user_rating_id, e
                );
                unexpected_err_resp()
            })?;
        let converted_result = model.map(|m| {
            let c = m.1.ok_or_else(|| {
                error!(
                    "failed to find consultation (user_rating_id: {}, consultation_id: {})",
                    user_rating_id, m.0.consultation_id
                );
                unexpected_err_resp()
            })?;
            Ok(ConsultationInfo {
                consultation_id: c.consultation_id,
                user_account_id: c.user_account_id,
                consultant_id: c.consultant_id,
                consultation_date_time_in_jst: c.meeting_at.with_timezone(&(*JAPANESE_TIME_ZONE)),
            })
        });
        Ok(match converted_result {
            Some(r) => Some(r?),
            None => None,
        })
    }

    async fn update_user_rating(
        &self,
        user_account_id: i64,
        user_rating_id: i64,
        rating: i16,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    // 同じユーザーに対する複数のuser_ratingの更新が来た場合に備えて
                    // また、user_rating更新中にユーザーが自身のアカウントを削除する場合に備えてuser_accountで排他ロックを取得しておく
                    let user_account_option =
                        find_user_account_by_user_account_id_with_exclusive_lock(
                            txn,
                            user_account_id,
                        )
                        .await?;
                    if user_account_option.is_none() {
                        info!(
                            "no user (user_account_id: {}) found on rating",
                            user_account_id
                        );
                        return Ok(());
                    }
                    let model_option = entity::user_rating::Entity::find_by_id(user_rating_id)
                        .one(txn)
                        .await
                        .map_err(|e| {
                            error!(
                                "failed to find user_rating (user_rating_id: {}): {}",
                                user_rating_id, e
                            );
                            ErrRespStruct {
                                err_resp: unexpected_err_resp(),
                            }
                        })?;
                    let model = match model_option {
                        Some(m) => m,
                        None => {
                            error!(
                                "no user_rating (user_rating_id: {}) found on rating",
                                user_rating_id
                            );
                            return Err(ErrRespStruct {
                                err_resp: unexpected_err_resp(),
                            });
                        }
                    };
                    if model.rating.is_some() {
                        return Err(ErrRespStruct {
                            err_resp: (
                                StatusCode::BAD_REQUEST,
                                Json(ApiError {
                                    code: Code::UserAccountHasAlreadyBeenRated as u32,
                                }),
                            ),
                        });
                    }
                    update_user_rating(model, txn, rating, current_date_time).await?;
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
                    error!("failed to update_user_rating: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}

async fn update_user_rating(
    model: entity::user_rating::Model,
    txn: &DatabaseTransaction,
    rating: i16,
    current_date_time: DateTime<FixedOffset>,
) -> Result<(), ErrRespStruct> {
    let user_rating_id = model.user_rating_id;
    let mut active_model: entity::user_rating::ActiveModel = model.into();
    active_model.rating = Set(Some(rating));
    active_model.rated_at = Set(Some(current_date_time));
    let _ = active_model.update(txn).await.map_err(|e| {
        error!(
            "failed to update user_rating (user_rating_id: {}, rating: {}, current_date_time: {}): {}",
            user_rating_id, rating, current_date_time, e
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}

async fn handle_user_rating(
    consultant_id: i64,
    user_rating_id: i64,
    rating: i16,
    current_date_time: &DateTime<FixedOffset>,
    op: impl UserRatingOperation,
) -> RespResult<UserRatingResult> {
    ensure_rating_id_is_positive(user_rating_id)?;
    ensure_rating_is_in_valid_range(rating)?;

    let cl = get_consultation_info_from_user_rating(user_rating_id, &op).await?;
    ensure_consultant_ids_are_same(consultant_id, cl.consultant_id)?;
    ensure_end_of_consultation_date_time_has_passed(
        &cl.consultation_date_time_in_jst,
        current_date_time,
    )?;

    op.update_user_rating(
        cl.user_account_id,
        user_rating_id,
        rating,
        *current_date_time,
    )
    .await?;

    Ok((StatusCode::OK, Json(UserRatingResult {})))
}

async fn get_consultation_info_from_user_rating(
    user_rating_id: i64,
    op: &impl UserRatingOperation,
) -> Result<ConsultationInfo, ErrResp> {
    let cl = op
        .find_consultation_info_from_user_rating(user_rating_id)
        .await?;
    match cl {
        Some(c) => Ok(c),
        None => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoUserRatingFound as u32,
            }),
        )),
    }
}

fn ensure_consultant_ids_are_same(
    consultant_id: i64,
    consultant_id_in_consultation_info: i64,
) -> Result<(), ErrResp> {
    if consultant_id != consultant_id_in_consultation_info {
        error!(
            "consultant_id ({}) and consultant_id_in_consultation_info ({}) are not same",
            consultant_id, consultant_id_in_consultation_info
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoUserRatingFound as u32,
            }),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum::{async_trait, Json};
    use chrono::{DateTime, FixedOffset, TimeZone};
    use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
    use once_cell::sync::Lazy;

    use crate::err::Code;

    use super::{handle_user_rating, ConsultationInfo, UserRatingOperation, UserRatingResult};

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<UserRatingResult>,
    }

    #[derive(Debug)]
    struct Input {
        consultant_id: i64,
        user_rating_id: i64,
        rating: i16,
        current_date_time: DateTime<FixedOffset>,
        op: UserRatingOperationMock,
    }

    #[derive(Clone, Debug)]
    struct UserRatingOperationMock {
        user_rating_id: i64,
        consultation_info: ConsultationInfo,
        rating: i16,
        current_date_time: DateTime<FixedOffset>,
        already_exists: bool,
    }

    #[async_trait]
    impl UserRatingOperation for UserRatingOperationMock {
        async fn find_consultation_info_from_user_rating(
            &self,
            user_rating_id: i64,
        ) -> Result<Option<ConsultationInfo>, ErrResp> {
            if self.user_rating_id != user_rating_id {
                return Ok(None);
            }
            Ok(Some(self.consultation_info.clone()))
        }

        async fn update_user_rating(
            &self,
            user_account_id: i64,
            user_rating_id: i64,
            rating: i16,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<(), ErrResp> {
            if self.already_exists {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::UserAccountHasAlreadyBeenRated as u32,
                    }),
                ));
            }
            assert_eq!(self.consultation_info.user_account_id, user_account_id);
            assert_eq!(self.user_rating_id, user_rating_id);
            assert_eq!(self.rating, rating);
            assert_eq!(self.current_date_time, current_date_time);
            Ok(())
        }
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        let consultant_id = 5123;
        let user_rating_id = 51604;
        let rating = 3;
        let consultation_id = 515;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 5, 17, 53, 12)
            .unwrap();
        let user_account_id = consultant_id + 640;
        let consultation_date_time_in_jst = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 3, 17, 0, 0)
            .unwrap();
        vec![
            TestCase {
                name: "success".to_string(),
                input: Input {
                    consultant_id,
                    user_rating_id,
                    rating,
                    current_date_time,
                    op: UserRatingOperationMock {
                        user_rating_id,
                        consultation_info: ConsultationInfo {
                            consultation_id,
                            user_account_id,
                            consultant_id,
                            consultation_date_time_in_jst,
                        },
                        rating,
                        current_date_time,
                        already_exists: false,
                    },
                },
                expected: Ok((StatusCode::OK, Json(UserRatingResult {}))),
            },
            TestCase {
                name: "fail RatingIdIsNotPositive".to_string(),
                input: Input {
                    consultant_id,
                    user_rating_id: -1,
                    rating,
                    current_date_time,
                    op: UserRatingOperationMock {
                        user_rating_id,
                        consultation_info: ConsultationInfo {
                            consultation_id,
                            user_account_id,
                            consultant_id,
                            consultation_date_time_in_jst,
                        },
                        rating,
                        current_date_time,
                        already_exists: false,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::RatingIdIsNotPositive as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail InvalidRating".to_string(),
                input: Input {
                    consultant_id,
                    user_rating_id,
                    rating: 0,
                    current_date_time,
                    op: UserRatingOperationMock {
                        user_rating_id,
                        consultation_info: ConsultationInfo {
                            consultation_id,
                            user_account_id,
                            consultant_id,
                            consultation_date_time_in_jst,
                        },
                        rating,
                        current_date_time,
                        already_exists: false,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidRating as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NoUserRatingFound (really not found)".to_string(),
                input: Input {
                    consultant_id,
                    user_rating_id,
                    rating,
                    current_date_time,
                    op: UserRatingOperationMock {
                        user_rating_id: user_rating_id + 3,
                        consultation_info: ConsultationInfo {
                            consultation_id,
                            user_account_id,
                            consultant_id,
                            consultation_date_time_in_jst,
                        },
                        rating,
                        current_date_time,
                        already_exists: false,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoUserRatingFound as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NoUserRatingFound (consultant id does not match)".to_string(),
                input: Input {
                    consultant_id,
                    user_rating_id,
                    rating,
                    current_date_time,
                    op: UserRatingOperationMock {
                        user_rating_id,
                        consultation_info: ConsultationInfo {
                            consultation_id,
                            user_account_id,
                            consultant_id: consultant_id + 60,
                            consultation_date_time_in_jst,
                        },
                        rating,
                        current_date_time,
                        already_exists: false,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoUserRatingFound as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail EndOfConsultationDateTimeHasNotPassedYet".to_string(),
                input: Input {
                    consultant_id,
                    user_rating_id,
                    rating,
                    current_date_time,
                    op: UserRatingOperationMock {
                        user_rating_id,
                        consultation_info: ConsultationInfo {
                            consultation_id,
                            user_account_id,
                            consultant_id,
                            consultation_date_time_in_jst: current_date_time, // consultation_date_time_in_jst == current_date_time => まだミーティング時間中
                        },
                        rating,
                        current_date_time,
                        already_exists: false,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::EndOfConsultationDateTimeHasNotPassedYet as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail UserAccountHasAlreadyBeenRated".to_string(),
                input: Input {
                    consultant_id,
                    user_rating_id,
                    rating,
                    current_date_time,
                    op: UserRatingOperationMock {
                        user_rating_id,
                        consultation_info: ConsultationInfo {
                            consultation_id,
                            user_account_id,
                            consultant_id,
                            consultation_date_time_in_jst,
                        },
                        rating,
                        current_date_time,
                        already_exists: true,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::UserAccountHasAlreadyBeenRated as u32,
                    }),
                )),
            },
        ]
    });

    #[tokio::test]
    async fn handle_user_rating_tests() {
        for test_case in TEST_CASE_SET.iter() {
            let consultant_id = test_case.input.consultant_id;
            let user_rating_id = test_case.input.user_rating_id;
            let rating = test_case.input.rating;
            let current_date_time = test_case.input.current_date_time;
            let op = test_case.input.op.clone();

            let result = handle_user_rating(
                consultant_id,
                user_rating_id,
                rating,
                &current_date_time,
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
