// Copyright 2023 Ken Miura

use async_session::async_trait;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Datelike, FixedOffset, Timelike, Utc};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use serde::Serialize;
use tracing::error;

use crate::{
    err::unexpected_err_resp,
    handlers::session::authentication::authenticated_handlers::{
        admin::Admin, pagination::Pagination,
    },
};

use super::{AwaitingPayment, AwaitingPaymentAndConsultation, Name};

// DBテーブルの設計上、この回数分だけクエリを呼ぶようになるため、他より少なめな一方で運用上閲覧するのに十分な値を設定する
const VALID_PAGE_SIZE: u64 = 20;

pub(crate) async fn get_expired_awaiting_payments(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<Pagination>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<ExpiredAwaitingPaymentResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = ExpiredAwaitingPaymentsOperationImpl { pool };
    handle_expired_awaiting_payments(query.page, query.per_page, current_date_time, op).await
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct ExpiredAwaitingPaymentResult {
    awaiting_payments: Vec<AwaitingPayment>,
}

async fn handle_expired_awaiting_payments(
    page: u64,
    per_page: u64,
    current_date_time: DateTime<FixedOffset>,
    op: impl ExpiredAwaitingPaymentsOperation,
) -> RespResult<ExpiredAwaitingPaymentResult> {
    if per_page > VALID_PAGE_SIZE {
        error!("invalid per_page ({})", per_page);
        return Err(unexpected_err_resp());
    };

    let results = op
        .get_expired_awaiting_payment_and_consultation(page, per_page, current_date_time)
        .await?;
    let mut awaiting_payments = Vec::with_capacity(results.len());
    for result in results {
        // resultsの個数回分だけDBアクセスが発生してしまうが、per_page回以下であることが保証されるため、許容する
        let name = op
            .find_name_by_user_account_id(result.user_account_id)
            .await?;
        awaiting_payments.push(AwaitingPayment {
            consultation_id: result.consultation_id,
            consultant_id: result.consultant_id,
            user_account_id: result.user_account_id,
            meeting_at: result.meeting_at.to_rfc3339(),
            fee_per_hour_in_yen: result.fee_per_hour_in_yen,
            sender_name: format!("{}　{}", name.last_name_furigana, name.first_name_furigana),
            sender_name_suffix: format!(
                "{:0>2}{:0>2}{:0>2}",
                result.meeting_at.month(),
                result.meeting_at.day(),
                result.meeting_at.hour()
            ),
        })
    }

    Ok((
        StatusCode::OK,
        Json(ExpiredAwaitingPaymentResult { awaiting_payments }),
    ))
}

#[async_trait]
trait ExpiredAwaitingPaymentsOperation {
    async fn get_expired_awaiting_payment_and_consultation(
        &self,
        page: u64,
        per_page: u64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<AwaitingPaymentAndConsultation>, ErrResp>;

    async fn find_name_by_user_account_id(&self, user_account_id: i64) -> Result<Name, ErrResp>;
}

struct ExpiredAwaitingPaymentsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ExpiredAwaitingPaymentsOperation for ExpiredAwaitingPaymentsOperationImpl {
    async fn get_expired_awaiting_payment_and_consultation(
        &self,
        page: u64,
        per_page: u64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<AwaitingPaymentAndConsultation>, ErrResp> {
        let models = entity::awaiting_payment::Entity::find()
            .find_also_related(entity::consultation::Entity)
            .filter(entity::consultation::Column::MeetingAt.lte(current_date_time))
            .order_by_asc(entity::consultation::Column::MeetingAt)
            .paginate(&self.pool, per_page)
            .fetch_page(page)
            .await
            .map_err(|e| {
                error!(
                    "failed to find awaiting_payment and consultation (page: {}, per_page: {}, current_date_time: {}): {}",
                    page, per_page, current_date_time, e
                );
                unexpected_err_resp()
            })?;
        models
            .into_iter()
            .map(|m| {
                let a = m.0;
                let c = m.1.ok_or_else(|| {
                    error!("failed to get consultation");
                    unexpected_err_resp()
                })?;
                Ok(AwaitingPaymentAndConsultation {
                    consultation_id: a.consultation_id,
                    consultant_id: c.consultant_id,
                    user_account_id: c.user_account_id,
                    meeting_at: c.meeting_at.with_timezone(&(*JAPANESE_TIME_ZONE)),
                    fee_per_hour_in_yen: a.fee_per_hour_in_yen,
                })
            })
            .collect::<Result<Vec<AwaitingPaymentAndConsultation>, ErrResp>>()
    }

    async fn find_name_by_user_account_id(&self, user_account_id: i64) -> Result<Name, ErrResp> {
        super::find_name_by_user_account_id(&self.pool, user_account_id).await
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use chrono::TimeZone;

    use crate::err::Code;

    use super::*;

    struct ExpiredAwaitingPaymentsOperationMock {
        page: u64,
        per_page: u64,
        current_date_time: DateTime<FixedOffset>,
        awaiting_payment_and_consultations: Vec<AwaitingPaymentAndConsultation>,
        names: HashMap<i64, Name>,
    }

    #[async_trait]
    impl ExpiredAwaitingPaymentsOperation for ExpiredAwaitingPaymentsOperationMock {
        async fn get_expired_awaiting_payment_and_consultation(
            &self,
            page: u64,
            per_page: u64,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<Vec<AwaitingPaymentAndConsultation>, ErrResp> {
            assert_eq!(self.page, page);
            assert_eq!(self.per_page, per_page);
            assert_eq!(self.current_date_time, current_date_time);
            let length = self.awaiting_payment_and_consultations.len();
            let page = page as usize;
            let per_page = per_page as usize;
            let start_index = page * per_page;
            let num = if length > per_page { per_page } else { length };
            let end_index = start_index + num;
            let cloned = self.awaiting_payment_and_consultations.clone();
            Ok(if length <= start_index {
                vec![]
            } else {
                cloned[start_index..end_index].to_vec()
            })
        }

        async fn find_name_by_user_account_id(
            &self,
            user_account_id: i64,
        ) -> Result<Name, ErrResp> {
            let ids = self
                .awaiting_payment_and_consultations
                .clone()
                .into_iter()
                .map(|m| m.user_account_id)
                .collect::<Vec<i64>>();
            assert!(ids.contains(&user_account_id));
            let name = self
                .names
                .get(&user_account_id)
                .expect("failed to get Name");
            Ok(name.clone())
        }
    }

    #[tokio::test]
    async fn handle_expired_awaiting_payments_success_case1() {
        let page = 0;
        let per_page = VALID_PAGE_SIZE;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();
        let op = ExpiredAwaitingPaymentsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_payment_and_consultations: vec![],
            names: HashMap::with_capacity(0),
        };

        let result = handle_expired_awaiting_payments(page, per_page, current_date_time, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            ExpiredAwaitingPaymentResult {
                awaiting_payments: vec![]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn handle_expired_awaiting_payments_success_case2() {
        let page = 0;
        let per_page = VALID_PAGE_SIZE;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 40)
            .unwrap();
        let consultation_id = 1;
        let consultant_id = 2;
        let user_account_id = 3;
        let meeting_at = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let fee_per_hour_in_yen = 5000;
        let name = Name {
            last_name_furigana: "タナカ".to_string(),
            first_name_furigana: "タロウ".to_string(),
        };
        let mut names = HashMap::with_capacity(1);
        names.insert(user_account_id, name.clone());
        let op = ExpiredAwaitingPaymentsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_payment_and_consultations: vec![AwaitingPaymentAndConsultation {
                consultation_id,
                consultant_id,
                user_account_id,
                meeting_at,
                fee_per_hour_in_yen,
            }],
            names,
        };

        let result = handle_expired_awaiting_payments(page, per_page, current_date_time, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            ExpiredAwaitingPaymentResult {
                awaiting_payments: vec![AwaitingPayment {
                    consultation_id,
                    consultant_id,
                    user_account_id,
                    meeting_at: meeting_at.to_rfc3339(),
                    fee_per_hour_in_yen,
                    sender_name: format!(
                        "{}　{}",
                        name.last_name_furigana, name.first_name_furigana
                    ),
                    sender_name_suffix: format!(
                        "{:0>2}{:0>2}{:0>2}",
                        meeting_at.month(),
                        meeting_at.day(),
                        meeting_at.hour()
                    )
                }]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn handle_expired_awaiting_payments_success_case3() {
        let page = 0;
        let per_page = VALID_PAGE_SIZE;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 10, 5, 21, 0, 40)
            .unwrap();

        let consultation_id1 = 1;
        let consultant_id1 = 2;
        let user_account_id1 = 3;
        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let fee_per_hour_in_yen1 = 5000;
        let name1 = Name {
            last_name_furigana: "タナカ".to_string(),
            first_name_furigana: "タロウ".to_string(),
        };

        let consultation_id2 = 4;
        let consultant_id2 = 5;
        let user_account_id2 = 6;
        let meeting_at2 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 29, 17, 0, 0)
            .unwrap();
        let fee_per_hour_in_yen2 = 7000;
        let name2 = Name {
            last_name_furigana: "スズキ".to_string(),
            first_name_furigana: "ジロウ".to_string(),
        };

        let mut names = HashMap::with_capacity(2);
        names.insert(user_account_id1, name1.clone());
        names.insert(user_account_id2, name2.clone());

        let op = ExpiredAwaitingPaymentsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_payment_and_consultations: vec![
                AwaitingPaymentAndConsultation {
                    consultation_id: consultation_id1,
                    consultant_id: consultant_id1,
                    user_account_id: user_account_id1,
                    meeting_at: meeting_at1,
                    fee_per_hour_in_yen: fee_per_hour_in_yen1,
                },
                AwaitingPaymentAndConsultation {
                    consultation_id: consultation_id2,
                    consultant_id: consultant_id2,
                    user_account_id: user_account_id2,
                    meeting_at: meeting_at2,
                    fee_per_hour_in_yen: fee_per_hour_in_yen2,
                },
            ],
            names,
        };

        let result = handle_expired_awaiting_payments(page, per_page, current_date_time, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            ExpiredAwaitingPaymentResult {
                awaiting_payments: vec![
                    AwaitingPayment {
                        consultation_id: consultation_id1,
                        consultant_id: consultant_id1,
                        user_account_id: user_account_id1,
                        meeting_at: meeting_at1.to_rfc3339(),
                        fee_per_hour_in_yen: fee_per_hour_in_yen1,
                        sender_name: format!(
                            "{}　{}",
                            name1.last_name_furigana, name1.first_name_furigana
                        ),
                        sender_name_suffix: format!(
                            "{:0>2}{:0>2}{:0>2}",
                            meeting_at1.month(),
                            meeting_at1.day(),
                            meeting_at1.hour()
                        )
                    },
                    AwaitingPayment {
                        consultation_id: consultation_id2,
                        consultant_id: consultant_id2,
                        user_account_id: user_account_id2,
                        meeting_at: meeting_at2.to_rfc3339(),
                        fee_per_hour_in_yen: fee_per_hour_in_yen2,
                        sender_name: format!(
                            "{}　{}",
                            name2.last_name_furigana, name2.first_name_furigana
                        ),
                        sender_name_suffix: format!(
                            "{:0>2}{:0>2}{:0>2}",
                            meeting_at2.month(),
                            meeting_at2.day(),
                            meeting_at2.hour()
                        )
                    }
                ]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn handle_expired_awaiting_payments_success_case4() {
        let page = 0;
        let per_page = 1;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 10, 5, 21, 0, 40)
            .unwrap();

        let consultation_id1 = 1;
        let consultant_id1 = 2;
        let user_account_id1 = 3;
        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let fee_per_hour_in_yen1 = 5000;
        let name1 = Name {
            last_name_furigana: "タナカ".to_string(),
            first_name_furigana: "タロウ".to_string(),
        };

        let consultation_id2 = 4;
        let consultant_id2 = 5;
        let user_account_id2 = 6;
        let meeting_at2 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 29, 17, 0, 0)
            .unwrap();
        let fee_per_hour_in_yen2 = 7000;
        let name2 = Name {
            last_name_furigana: "スズキ".to_string(),
            first_name_furigana: "ジロウ".to_string(),
        };

        let mut names = HashMap::with_capacity(2);
        names.insert(user_account_id1, name1.clone());
        names.insert(user_account_id2, name2.clone());

        let op = ExpiredAwaitingPaymentsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_payment_and_consultations: vec![
                AwaitingPaymentAndConsultation {
                    consultation_id: consultation_id1,
                    consultant_id: consultant_id1,
                    user_account_id: user_account_id1,
                    meeting_at: meeting_at1,
                    fee_per_hour_in_yen: fee_per_hour_in_yen1,
                },
                AwaitingPaymentAndConsultation {
                    consultation_id: consultation_id2,
                    consultant_id: consultant_id2,
                    user_account_id: user_account_id2,
                    meeting_at: meeting_at2,
                    fee_per_hour_in_yen: fee_per_hour_in_yen2,
                },
            ],
            names,
        };

        let result = handle_expired_awaiting_payments(page, per_page, current_date_time, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            ExpiredAwaitingPaymentResult {
                awaiting_payments: vec![AwaitingPayment {
                    consultation_id: consultation_id1,
                    consultant_id: consultant_id1,
                    user_account_id: user_account_id1,
                    meeting_at: meeting_at1.to_rfc3339(),
                    fee_per_hour_in_yen: fee_per_hour_in_yen1,
                    sender_name: format!(
                        "{}　{}",
                        name1.last_name_furigana, name1.first_name_furigana
                    ),
                    sender_name_suffix: format!(
                        "{:0>2}{:0>2}{:0>2}",
                        meeting_at1.month(),
                        meeting_at1.day(),
                        meeting_at1.hour()
                    )
                }]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn handle_expired_awaiting_payments_success_case5() {
        let page = 1;
        let per_page = 1;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 10, 5, 21, 0, 40)
            .unwrap();

        let consultation_id1 = 1;
        let consultant_id1 = 2;
        let user_account_id1 = 3;
        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let fee_per_hour_in_yen1 = 5000;
        let name1 = Name {
            last_name_furigana: "タナカ".to_string(),
            first_name_furigana: "タロウ".to_string(),
        };

        let consultation_id2 = 4;
        let consultant_id2 = 5;
        let user_account_id2 = 6;
        let meeting_at2 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 29, 17, 0, 0)
            .unwrap();
        let fee_per_hour_in_yen2 = 7000;
        let name2 = Name {
            last_name_furigana: "スズキ".to_string(),
            first_name_furigana: "ジロウ".to_string(),
        };

        let mut names = HashMap::with_capacity(2);
        names.insert(user_account_id1, name1.clone());
        names.insert(user_account_id2, name2.clone());

        let op = ExpiredAwaitingPaymentsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_payment_and_consultations: vec![
                AwaitingPaymentAndConsultation {
                    consultation_id: consultation_id1,
                    consultant_id: consultant_id1,
                    user_account_id: user_account_id1,
                    meeting_at: meeting_at1,
                    fee_per_hour_in_yen: fee_per_hour_in_yen1,
                },
                AwaitingPaymentAndConsultation {
                    consultation_id: consultation_id2,
                    consultant_id: consultant_id2,
                    user_account_id: user_account_id2,
                    meeting_at: meeting_at2,
                    fee_per_hour_in_yen: fee_per_hour_in_yen2,
                },
            ],
            names,
        };

        let result = handle_expired_awaiting_payments(page, per_page, current_date_time, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            ExpiredAwaitingPaymentResult {
                awaiting_payments: vec![AwaitingPayment {
                    consultation_id: consultation_id2,
                    consultant_id: consultant_id2,
                    user_account_id: user_account_id2,
                    meeting_at: meeting_at2.to_rfc3339(),
                    fee_per_hour_in_yen: fee_per_hour_in_yen2,
                    sender_name: format!(
                        "{}　{}",
                        name2.last_name_furigana, name2.first_name_furigana
                    ),
                    sender_name_suffix: format!(
                        "{:0>2}{:0>2}{:0>2}",
                        meeting_at2.month(),
                        meeting_at2.day(),
                        meeting_at2.hour()
                    )
                }]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn handle_expired_awaiting_payments_success_case6() {
        let page = 2;
        let per_page = 1;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 10, 5, 21, 0, 40)
            .unwrap();

        let consultation_id1 = 1;
        let consultant_id1 = 2;
        let user_account_id1 = 3;
        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let fee_per_hour_in_yen1 = 5000;
        let name1 = Name {
            last_name_furigana: "タナカ".to_string(),
            first_name_furigana: "タロウ".to_string(),
        };

        let consultation_id2 = 4;
        let consultant_id2 = 5;
        let user_account_id2 = 6;
        let meeting_at2 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 29, 17, 0, 0)
            .unwrap();
        let fee_per_hour_in_yen2 = 7000;
        let name2 = Name {
            last_name_furigana: "スズキ".to_string(),
            first_name_furigana: "ジロウ".to_string(),
        };

        let mut names = HashMap::with_capacity(2);
        names.insert(user_account_id1, name1.clone());
        names.insert(user_account_id2, name2.clone());

        let op = ExpiredAwaitingPaymentsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_payment_and_consultations: vec![
                AwaitingPaymentAndConsultation {
                    consultation_id: consultation_id1,
                    consultant_id: consultant_id1,
                    user_account_id: user_account_id1,
                    meeting_at: meeting_at1,
                    fee_per_hour_in_yen: fee_per_hour_in_yen1,
                },
                AwaitingPaymentAndConsultation {
                    consultation_id: consultation_id2,
                    consultant_id: consultant_id2,
                    user_account_id: user_account_id2,
                    meeting_at: meeting_at2,
                    fee_per_hour_in_yen: fee_per_hour_in_yen2,
                },
            ],
            names,
        };

        let result = handle_expired_awaiting_payments(page, per_page, current_date_time, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            ExpiredAwaitingPaymentResult {
                awaiting_payments: vec![]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn handle_expired_awaiting_payments_fale_case1() {
        let page = 0;
        let per_page = VALID_PAGE_SIZE + 1;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();
        let op = ExpiredAwaitingPaymentsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_payment_and_consultations: vec![],
            names: HashMap::with_capacity(0),
        };

        let result = handle_expired_awaiting_payments(page, per_page, current_date_time, op).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.0);
        assert_eq!(Code::UnexpectedErr as u32, resp.1.code);
    }
}