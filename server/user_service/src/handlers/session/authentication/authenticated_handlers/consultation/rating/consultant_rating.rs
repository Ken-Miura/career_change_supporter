// Copyright 2023 Ken Miura

use async_session::serde_json::json;
use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::opensearch::{update_document, INDEX_NAME};
use common::rating::calculate_average_rating;
use common::{ApiError, ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    QueryFilter, Set, TransactionError, TransactionTrait,
};
use opensearch::OpenSearch;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::err::{unexpected_err_resp, Code};
use crate::handlers::session::authentication::authenticated_handlers::consultation::{validate_consultation_id_is_positive};
use crate::handlers::session::authentication::authenticated_handlers::document_operation::find_document_model_by_user_account_id_with_exclusive_lock;
use crate::handlers::session::authentication::authenticated_handlers::authenticated_users::verified_user::VerifiedUser;
use crate::handlers::session::authentication::user_operation::find_user_account_by_user_account_id_with_exclusive_lock;

use super::{
    ensure_end_of_consultation_date_time_has_passed, ensure_rating_is_in_valid_range,
    ConsultationInfo,
};

pub(crate) async fn post_consultant_rating(
    VerifiedUser { user_info }: VerifiedUser,
    State(pool): State<DatabaseConnection>,
    State(index_client): State<OpenSearch>,
    Json(req): Json<ConsultantRatingParam>,
) -> RespResult<ConsultantRatingResult> {
    let op = ConsultantRatingOperationImpl { pool, index_client };
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    handle_consultant_rating(
        user_info.account_id,
        req.consultation_id,
        req.rating,
        &current_date_time,
        op,
    )
    .await
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ConsultantRatingParam {
    consultation_id: i64,
    rating: i16,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct ConsultantRatingResult {}

#[async_trait]
trait ConsultantRatingOperation {
    async fn find_consultation_info(
        &self,
        consultation_id: i64,
    ) -> Result<Option<ConsultationInfo>, ErrResp>;

    async fn update_consultant_rating(
        &self,
        consultant_id: i64,
        consultation_id: i64,
        rating: i16,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;

    async fn filter_consultant_rating_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Vec<i16>, ErrResp>;

    async fn update_rating_on_document_if_not_disabled(
        &self,
        consultant_id: i64,
        averate_rating: f64,
        num_of_rated: i32,
    ) -> Result<(), ErrResp>;
}

struct ConsultantRatingOperationImpl {
    pool: DatabaseConnection,
    index_client: OpenSearch,
}

#[async_trait]
impl ConsultantRatingOperation for ConsultantRatingOperationImpl {
    async fn find_consultation_info(
        &self,
        consultation_id: i64,
    ) -> Result<Option<ConsultationInfo>, ErrResp> {
        super::find_consultation_info(&self.pool, consultation_id).await
    }

    async fn update_consultant_rating(
        &self,
        consultant_id: i64,
        consultation_id: i64,
        rating: i16,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    // 同じコンサルタントに対する複数のconsultant_ratingの更新が来た場合に備えて
                    // また、consultant_rating更新中にコンサルタントが自身のアカウントを削除する場合に備えてuser_accountで排他ロックを取得しておく
                    let consultant_option =
                        find_user_account_by_user_account_id_with_exclusive_lock(
                            txn,
                            consultant_id,
                        )
                        .await?;
                    // コンサルタントのアカウントが削除されていた場合でも評価用のレコードは残っているので評価は行う
                    // （評価時に対象のコンサルタントのアカウントが削除済みかどうかは記録しておく）
                    if consultant_option.is_none() {
                        info!(
                            "no consultant (consultant_id: {}) found when recording rating on database",
                            consultant_id
                        );
                    }
                    let model_option =
                        entity::consultant_rating::Entity::find_by_id(consultation_id)
                            .one(txn)
                            .await
                            .map_err(|e| {
                                error!(
                                    "failed to find consultant_rating (consultation_id: {}): {}",
                                    consultation_id, e
                                );
                                ErrRespStruct {
                                    err_resp: unexpected_err_resp(),
                                }
                            })?;
                    let model = match model_option {
                        Some(m) => m,
                        None => {
                            error!(
                                "no consultant_rating (consultation_id: {}) found on rating",
                                consultation_id
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
                                    code: Code::ConsultantHasAlreadyBeenRated as u32,
                                }),
                            ),
                        });
                    }
                    update_consultant_rating(model, txn, rating, current_date_time).await?;
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
                    error!("failed to update_consultant_rating: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }

    async fn filter_consultant_rating_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Vec<i16>, ErrResp> {
        // 評価数がメモリ容量を圧迫するほど貯まるとは考えづらく、複数回に分けてフェッチするような実装とはしていない
        // NOTE: 実際に問題（特定のコンサルタントへの評価に時間がかかる問題）が発生した際、ここを確認して必要なら修正する
        let models = entity::consultant_rating::Entity::find()
            .filter(entity::consultant_rating::Column::ConsultantId.eq(consultant_id))
            .filter(entity::consultant_rating::Column::Rating.is_not_null())
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter consultant_rating (consultant_id: {}): {}",
                    consultant_id, e
                );
                unexpected_err_resp()
            })?;
        models
            .into_iter()
            .map(|m| {
                let r = m.rating.ok_or_else(|| {
                    // NOT NULL 条件で検索しているのでNULLの場合（＝ない場合）はエラー
                    error!(
                        "rating is null (consultation_id: {}, consultant_id: {})",
                        m.consultation_id, m.consultant_id
                    );
                    unexpected_err_resp()
                })?;
                Ok(r)
            })
            .collect::<Result<Vec<i16>, ErrResp>>()
    }

    async fn update_rating_on_document_if_not_disabled(
        &self,
        consultant_id: i64,
        averate_rating: f64,
        num_of_rated: i32,
    ) -> Result<(), ErrResp> {
        let index_client = self.index_client.clone();
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    // 管理者がコンサルタントをDisabledにしている途中に
                    // ユーザーがコンサルタントのratingの更新をした場合に備えて、user_accountで排他ロックを取得しておく
                    let consultant_option =
                        find_user_account_by_user_account_id_with_exclusive_lock(
                            txn,
                            consultant_id,
                        )
                        .await?;
                    let consultant = match consultant_option {
                        Some(c) => c,
                        None => {
                            // コンサルタントのアカウントが削除されていた場合、そのコンサルタント用のインデックスも削除済みとなっている
                            // 従って、ログに残すだけで評価を登録せず即リターンする（一方でDBのレコードは残しているので評価を行っている）
                            info!(
                                "no consultant (consultant_id: {}) found when recording rating on index",
                                consultant_id
                            );
                            return Ok(());
                        }
                    };
                    if consultant.disabled_at.is_some() {
                        info!("do not update rating on document because consultant (consultant_id: {}) is disabled", consultant_id);
                        return Ok(());
                    }

                    let doc_option = find_document_model_by_user_account_id_with_exclusive_lock(txn, consultant_id).await?;
                    let doc = match doc_option {
                        Some(d) => d,
                        None => {
                            // アカウントを排他ロックし、削除されていないことを確認済みのため、documentが存在しないことはないはず。従ってエラーログとして記録する。
                            // 一方で、ユーザーにまでこのエラーを返すのは適切でないため、Okとして処理する。
                            // エラーを返すのが適切ではないと考えたのは次の通り
                            // - このエラーを解消しないとユーザーは正しく操作を終了できないわけではない
                            // - 偶発的に起きた問題の場合、次回の評価時に正しく平均評価が反映される
                            error!("no document found on rate update(consultant_id: {})", consultant_id);
                            return Ok(());
                        }
                    };
                    update_rating_info_on_document(INDEX_NAME, doc.document_id.to_string().as_str(), averate_rating, num_of_rated, index_client).await?;
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
                        "failed to update_rating_on_document_if_not_disabled: {}",
                        err_resp_struct
                    );
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}

async fn update_consultant_rating(
    model: entity::consultant_rating::Model,
    txn: &DatabaseTransaction,
    rating: i16,
    current_date_time: DateTime<FixedOffset>,
) -> Result<(), ErrRespStruct> {
    let consultation_id = model.consultation_id;
    let mut active_model: entity::consultant_rating::ActiveModel = model.into();
    active_model.rating = Set(Some(rating));
    active_model.rated_at = Set(Some(current_date_time));
    let _ = active_model.update(txn).await.map_err(|e| {
        error!(
            "failed to update consultant_rating (consultation_id: {}, rating: {}, current_date_time: {}): {}",
            consultation_id, rating, current_date_time, e
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}

async fn update_rating_info_on_document(
    index_name: &str,
    document_id: &str,
    averate_rating: f64,
    num_of_rated: i32,
    client: OpenSearch,
) -> Result<(), ErrRespStruct> {
    let script = json!({
        "doc": {
            "rating": averate_rating,
            "num_of_rated": num_of_rated
        }
    });
    update_document(index_name, document_id, &script, &client)
        .await
        .map_err(|e| {
            error!(
                "failed to update rating info into document (document_id: {}, averate_rating: {}, num_of_rated: {})",
                document_id, averate_rating, num_of_rated
            );
            ErrRespStruct { err_resp: e }
        })?;
    Ok(())
}

async fn handle_consultant_rating(
    account_id: i64,
    consultation_id: i64,
    rating: i16,
    current_date_time: &DateTime<FixedOffset>,
    op: impl ConsultantRatingOperation,
) -> RespResult<ConsultantRatingResult> {
    validate_consultation_id_is_positive(consultation_id)?;
    ensure_rating_is_in_valid_range(rating)?;

    let cl = get_consultation_info(consultation_id, &op).await?;
    ensure_user_account_ids_are_same(account_id, cl.user_account_id)?;
    ensure_end_of_consultation_date_time_has_passed(
        &cl.consultation_date_time_in_jst,
        current_date_time,
    )?;

    op.update_consultant_rating(
        cl.consultant_id,
        consultation_id,
        rating,
        *current_date_time,
    )
    .await?;

    let ratings = op
        .filter_consultant_rating_by_consultant_id(cl.consultant_id)
        .await?;
    let num_of_rated = ratings.len() as i32;
    // Noneの場合は評価数0を意味するので、現在の評価は0として扱う
    let average_rating = calculate_average_rating(ratings).unwrap_or(0.0);
    // ユーザーに見せる評価は小数点一桁まで。ただ、表示するときに小数点一桁に丸めるだけで、データとしては計算結果をそのまま保管しておく
    op.update_rating_on_document_if_not_disabled(cl.consultant_id, average_rating, num_of_rated)
        .await?;

    Ok((StatusCode::OK, Json(ConsultantRatingResult {})))
}

async fn get_consultation_info(
    consultation_rating_id: i64,
    op: &impl ConsultantRatingOperation,
) -> Result<ConsultationInfo, ErrResp> {
    let cl = op.find_consultation_info(consultation_rating_id).await?;
    match cl {
        Some(c) => Ok(c),
        None => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoConsultantRatingFound as u32,
            }),
        )),
    }
}

fn ensure_user_account_ids_are_same(
    user_account_id: i64,
    user_account_id_in_consultation_info: i64,
) -> Result<(), ErrResp> {
    if user_account_id != user_account_id_in_consultation_info {
        error!(
            "user_account_id ({}) and user_account_id_in_consultation_info ({}) are not same",
            user_account_id, user_account_id_in_consultation_info
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoConsultantRatingFound as u32,
            }),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use chrono::TimeZone;
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<ConsultantRatingResult>,
    }

    #[derive(Debug)]
    struct Input {
        account_id: i64,
        consultation_id: i64,
        rating: i16,
        current_date_time: DateTime<FixedOffset>,
        op: ConsultantRatingOperationMock,
    }

    #[derive(Clone, Debug)]
    struct ConsultantRatingOperationMock {
        consultation_id: i64,
        consultation_info: ConsultationInfo,
        rating: i16,
        current_date_time: DateTime<FixedOffset>,
        already_exists: bool,
        ratings: Vec<i16>,
    }

    #[async_trait]
    impl ConsultantRatingOperation for ConsultantRatingOperationMock {
        async fn find_consultation_info(
            &self,
            consultation_id: i64,
        ) -> Result<Option<ConsultationInfo>, ErrResp> {
            if self.consultation_id != consultation_id {
                return Ok(None);
            }
            Ok(Some(self.consultation_info.clone()))
        }

        async fn update_consultant_rating(
            &self,
            consultant_id: i64,
            consultation_id: i64,
            rating: i16,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<(), ErrResp> {
            if self.already_exists {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::ConsultantHasAlreadyBeenRated as u32,
                    }),
                ));
            }
            assert_eq!(self.consultation_info.consultant_id, consultant_id);
            assert_eq!(self.consultation_id, consultation_id);
            assert_eq!(self.rating, rating);
            assert_eq!(self.current_date_time, current_date_time);
            Ok(())
        }

        async fn filter_consultant_rating_by_consultant_id(
            &self,
            consultant_id: i64,
        ) -> Result<Vec<i16>, ErrResp> {
            assert_eq!(self.consultation_info.consultant_id, consultant_id);
            Ok(self.ratings.clone())
        }

        async fn update_rating_on_document_if_not_disabled(
            &self,
            consultant_id: i64,
            averate_rating: f64,
            num_of_rated: i32,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.consultation_info.consultant_id, consultant_id);
            let ratings = self.ratings.clone();
            assert_eq!(ratings.len() as i32, num_of_rated);
            let average = calculate_average_rating(ratings).unwrap_or(0.0);
            let diff = (averate_rating - average).abs();
            assert!(diff < f64::EPSILON);
            Ok(())
        }
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        let account_id = 166;
        let consultation_id = 5701;
        let rating = 4;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 16, 17, 53, 12)
            .unwrap();
        let user_account_id = account_id;
        let consultant_id = user_account_id + 9761;
        let consultation_date_time_in_jst = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 13, 10, 0, 0)
            .unwrap();
        vec![
            TestCase {
                name: "success 1".to_string(),
                input: Input {
                    account_id,
                    consultation_id,
                    rating,
                    current_date_time,
                    op: ConsultantRatingOperationMock {
                        consultation_id,
                        consultation_info: ConsultationInfo {
                            user_account_id,
                            consultant_id,
                            consultation_date_time_in_jst,
                        },
                        rating,
                        current_date_time,
                        already_exists: false,
                        ratings: vec![rating],
                    },
                },
                expected: Ok((StatusCode::OK, Json(ConsultantRatingResult {}))),
            },
            TestCase {
                name: "success 2".to_string(),
                input: Input {
                    account_id,
                    consultation_id,
                    rating,
                    current_date_time,
                    op: ConsultantRatingOperationMock {
                        consultation_id,
                        consultation_info: ConsultationInfo {
                            user_account_id,
                            consultant_id,
                            consultation_date_time_in_jst,
                        },
                        rating,
                        current_date_time,
                        already_exists: false,
                        ratings: vec![rating, 3],
                    },
                },
                expected: Ok((StatusCode::OK, Json(ConsultantRatingResult {}))),
            },
            TestCase {
                name: "success 3".to_string(),
                input: Input {
                    account_id,
                    consultation_id,
                    rating,
                    current_date_time,
                    op: ConsultantRatingOperationMock {
                        consultation_id,
                        consultation_info: ConsultationInfo {
                            user_account_id,
                            consultant_id,
                            consultation_date_time_in_jst,
                        },
                        rating,
                        current_date_time,
                        already_exists: false,
                        ratings: vec![rating, 3, 2],
                    },
                },
                expected: Ok((StatusCode::OK, Json(ConsultantRatingResult {}))),
            },
            TestCase {
                name: "success 4".to_string(),
                input: Input {
                    account_id,
                    consultation_id,
                    rating,
                    current_date_time,
                    op: ConsultantRatingOperationMock {
                        consultation_id,
                        consultation_info: ConsultationInfo {
                            user_account_id,
                            consultant_id,
                            consultation_date_time_in_jst,
                        },
                        rating,
                        current_date_time,
                        already_exists: false,
                        ratings: vec![rating, 3, 2],
                    },
                },
                expected: Ok((StatusCode::OK, Json(ConsultantRatingResult {}))),
            },
            TestCase {
                name: "fail NonPositiveConsultationId".to_string(),
                input: Input {
                    account_id,
                    consultation_id: -1,
                    rating,
                    current_date_time,
                    op: ConsultantRatingOperationMock {
                        consultation_id,
                        consultation_info: ConsultationInfo {
                            user_account_id,
                            consultant_id,
                            consultation_date_time_in_jst,
                        },
                        rating,
                        current_date_time,
                        already_exists: false,
                        ratings: vec![rating],
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
                name: "fail InvalidRating".to_string(),
                input: Input {
                    account_id,
                    consultation_id,
                    rating: 6,
                    current_date_time,
                    op: ConsultantRatingOperationMock {
                        consultation_id,
                        consultation_info: ConsultationInfo {
                            user_account_id,
                            consultant_id,
                            consultation_date_time_in_jst,
                        },
                        rating,
                        current_date_time,
                        already_exists: false,
                        ratings: vec![rating],
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
                name: "fail NoConsultantRatingFound (really not found)".to_string(),
                input: Input {
                    account_id,
                    consultation_id,
                    rating,
                    current_date_time,
                    op: ConsultantRatingOperationMock {
                        consultation_id: consultation_id + 68,
                        consultation_info: ConsultationInfo {
                            user_account_id,
                            consultant_id,
                            consultation_date_time_in_jst,
                        },
                        rating,
                        current_date_time,
                        already_exists: false,
                        ratings: vec![rating],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoConsultantRatingFound as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NoConsultantRatingFound (user account id does not match)".to_string(),
                input: Input {
                    account_id,
                    consultation_id,
                    rating,
                    current_date_time,
                    op: ConsultantRatingOperationMock {
                        consultation_id,
                        consultation_info: ConsultationInfo {
                            user_account_id: user_account_id + 65010,
                            consultant_id,
                            consultation_date_time_in_jst,
                        },
                        rating,
                        current_date_time,
                        already_exists: false,
                        ratings: vec![rating],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoConsultantRatingFound as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail EndOfConsultationDateTimeHasNotPassedYet".to_string(),
                input: Input {
                    account_id,
                    consultation_id,
                    rating,
                    current_date_time,
                    op: ConsultantRatingOperationMock {
                        consultation_id,
                        consultation_info: ConsultationInfo {
                            user_account_id,
                            consultant_id,
                            consultation_date_time_in_jst: current_date_time, // consultation_date_time_in_jst == current_date_time => まだミーティング時間中,
                        },
                        rating,
                        current_date_time,
                        already_exists: false,
                        ratings: vec![rating],
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
                name: "fail ConsultantHasAlreadyBeenRated".to_string(),
                input: Input {
                    account_id,
                    consultation_id,
                    rating,
                    current_date_time,
                    op: ConsultantRatingOperationMock {
                        consultation_id,
                        consultation_info: ConsultationInfo {
                            user_account_id,
                            consultant_id,
                            consultation_date_time_in_jst,
                        },
                        rating,
                        current_date_time,
                        already_exists: true,
                        ratings: vec![rating],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::ConsultantHasAlreadyBeenRated as u32,
                    }),
                )),
            },
        ]
    });

    #[tokio::test]
    async fn handle_consultant_rating_tests() {
        for test_case in TEST_CASE_SET.iter() {
            let account_id = test_case.input.account_id;
            let consultation_id = test_case.input.consultation_id;
            let rating = test_case.input.rating;
            let current_date_time = test_case.input.current_date_time;
            let op = test_case.input.op.clone();

            let result = handle_consultant_rating(
                account_id,
                consultation_id,
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
