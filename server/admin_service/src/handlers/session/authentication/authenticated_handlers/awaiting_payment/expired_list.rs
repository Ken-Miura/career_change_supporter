// Copyright 2023 Ken Miura

use async_session::async_trait;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use serde::Serialize;
use tracing::error;

use crate::{
    err::unexpected_err_resp,
    handlers::session::authentication::authenticated_handlers::{
        admin::Admin, generate_sender_name, pagination::Pagination,
    },
};

use super::{convert_date_time_to_rfc3339_string, AwaitingPayment};

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

    let awaiting_payments = op
        .get_expired_awaiting_payments(page, per_page, current_date_time)
        .await?;

    Ok((
        StatusCode::OK,
        Json(ExpiredAwaitingPaymentResult { awaiting_payments }),
    ))
}

#[async_trait]
trait ExpiredAwaitingPaymentsOperation {
    async fn get_expired_awaiting_payments(
        &self,
        page: u64,
        per_page: u64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<AwaitingPayment>, ErrResp>;
}

struct ExpiredAwaitingPaymentsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ExpiredAwaitingPaymentsOperation for ExpiredAwaitingPaymentsOperationImpl {
    async fn get_expired_awaiting_payments(
        &self,
        page: u64,
        per_page: u64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<AwaitingPayment>, ErrResp> {
        let models = entity::awaiting_payment::Entity::find()
            .filter(entity::awaiting_payment::Column::MeetingAt.lte(current_date_time))
            .find_also_related(entity::identity::Entity)
            .order_by_asc(entity::awaiting_payment::Column::MeetingAt)
            .paginate(&self.pool, per_page)
            .fetch_page(page)
            .await
            .map_err(|e| {
                error!(
                    "failed to find awaiting_payment (page: {}, per_page: {}, current_date_time: {}): {}",
                    page, per_page, current_date_time, e
                );
                unexpected_err_resp()
            })?;
        models
            .into_iter()
            .map(|m| {
                let ap = m.0;
                // 身分情報が削除されるのはアカウントが削除された後
                // アカウントは予定されている相談が終わっていれば（現在時刻が相談終了時刻を超えていると）削除できる
                // 従って、相談開始日時後でフィルターしているこの時点では存在しないことはありえるため、そのハンドリングを行う
                let id = m.1;
                let meeting_at = ap.meeting_at.with_timezone(&(*JAPANESE_TIME_ZONE));
                let meeting_at_str = convert_date_time_to_rfc3339_string(
                    ap.meeting_at.with_timezone(&(*JAPANESE_TIME_ZONE)),
                );
                let sender_name = if let Some(i) = id {
                    let result = generate_sender_name(
                        i.last_name_furigana,
                        i.first_name_furigana,
                        meeting_at,
                    )?;
                    Some(result)
                } else {
                    None
                };
                Ok(AwaitingPayment {
                    consultation_id: ap.consultation_id,
                    consultant_id: ap.consultant_id,
                    user_account_id: ap.user_account_id,
                    meeting_at: meeting_at_str,
                    fee_per_hour_in_yen: ap.fee_per_hour_in_yen,
                    sender_name,
                })
            })
            .collect::<Result<Vec<AwaitingPayment>, ErrResp>>()
    }
}

#[cfg(test)]
mod tests {

    use chrono::TimeZone;

    use crate::err::Code;

    use super::*;

    struct ExpiredAwaitingPaymentsOperationMock {
        page: u64,
        per_page: u64,
        current_date_time: DateTime<FixedOffset>,
        awaiting_payments: Vec<AwaitingPayment>,
    }

    #[async_trait]
    impl ExpiredAwaitingPaymentsOperation for ExpiredAwaitingPaymentsOperationMock {
        async fn get_expired_awaiting_payments(
            &self,
            page: u64,
            per_page: u64,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<Vec<AwaitingPayment>, ErrResp> {
            assert_eq!(self.page, page);
            assert_eq!(self.per_page, per_page);
            assert_eq!(self.current_date_time, current_date_time);
            let awaiting_payments: Vec<AwaitingPayment> = self
                .awaiting_payments
                .clone()
                .into_iter()
                .filter(|aw| {
                    DateTime::parse_from_rfc3339(&aw.meeting_at).expect("failed to get Ok")
                        <= current_date_time
                })
                .collect();
            let length = awaiting_payments.len();
            let page = page as usize;
            let per_page = per_page as usize;
            let start_index = page * per_page;
            let num = if length > per_page { per_page } else { length };
            let end_index = start_index + num;
            Ok(if length <= start_index {
                vec![]
            } else {
                awaiting_payments[start_index..end_index].to_vec()
            })
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
            awaiting_payments: vec![],
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
        let awaiting_payment1 = AwaitingPayment {
            consultation_id,
            consultant_id,
            user_account_id,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at),
            fee_per_hour_in_yen,
            sender_name: Some(
                generate_sender_name("タナカ".to_string(), "タロウ".to_string(), meeting_at)
                    .expect("failed to get Ok"),
            ),
        };

        let op = ExpiredAwaitingPaymentsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_payments: vec![awaiting_payment1.clone()],
        };

        let result = handle_expired_awaiting_payments(page, per_page, current_date_time, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            ExpiredAwaitingPaymentResult {
                awaiting_payments: vec![awaiting_payment1]
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
        let awaiting_payment1 = AwaitingPayment {
            consultation_id: consultation_id1,
            consultant_id: consultant_id1,
            user_account_id: user_account_id1,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: fee_per_hour_in_yen1,
            sender_name: Some(
                generate_sender_name("タナカ".to_string(), "タロウ".to_string(), meeting_at1)
                    .expect("failed to get Ok"),
            ),
        };

        let consultation_id2 = 4;
        let consultant_id2 = 5;
        let user_account_id2 = 6;
        let meeting_at2 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 29, 17, 0, 0)
            .unwrap();
        let fee_per_hour_in_yen2 = 7000;
        let awaiting_payment2 = AwaitingPayment {
            consultation_id: consultation_id2,
            consultant_id: consultant_id2,
            user_account_id: user_account_id2,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: fee_per_hour_in_yen2,
            sender_name: Some(
                generate_sender_name("スズキ".to_string(), "ジロウ".to_string(), meeting_at2)
                    .expect("failed to get Ok"),
            ),
        };

        let op = ExpiredAwaitingPaymentsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_payments: vec![awaiting_payment1.clone(), awaiting_payment2.clone()],
        };

        let result = handle_expired_awaiting_payments(page, per_page, current_date_time, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            ExpiredAwaitingPaymentResult {
                awaiting_payments: vec![awaiting_payment1, awaiting_payment2]
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
        let awaiting_payment1 = AwaitingPayment {
            consultation_id: consultation_id1,
            consultant_id: consultant_id1,
            user_account_id: user_account_id1,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: fee_per_hour_in_yen1,
            sender_name: Some(
                generate_sender_name("タナカ".to_string(), "タロウ".to_string(), meeting_at1)
                    .expect("failed to get Ok"),
            ),
        };

        let consultation_id2 = 4;
        let consultant_id2 = 5;
        let user_account_id2 = 6;
        let meeting_at2 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 29, 17, 0, 0)
            .unwrap();
        let fee_per_hour_in_yen2 = 7000;
        let awaiting_payment2 = AwaitingPayment {
            consultation_id: consultation_id2,
            consultant_id: consultant_id2,
            user_account_id: user_account_id2,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: fee_per_hour_in_yen2,
            sender_name: Some(
                generate_sender_name("スズキ".to_string(), "ジロウ".to_string(), meeting_at2)
                    .expect("failed to get Ok"),
            ),
        };

        let op = ExpiredAwaitingPaymentsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_payments: vec![awaiting_payment1.clone(), awaiting_payment2.clone()],
        };

        let result = handle_expired_awaiting_payments(page, per_page, current_date_time, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            ExpiredAwaitingPaymentResult {
                awaiting_payments: vec![awaiting_payment1]
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
        let awaiting_payment1 = AwaitingPayment {
            consultation_id: consultation_id1,
            consultant_id: consultant_id1,
            user_account_id: user_account_id1,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: fee_per_hour_in_yen1,
            sender_name: Some(
                generate_sender_name("タナカ".to_string(), "タロウ".to_string(), meeting_at1)
                    .expect("failed to get Ok"),
            ),
        };

        let consultation_id2 = 4;
        let consultant_id2 = 5;
        let user_account_id2 = 6;
        let meeting_at2 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 29, 17, 0, 0)
            .unwrap();
        let fee_per_hour_in_yen2 = 7000;
        let awaiting_payment2 = AwaitingPayment {
            consultation_id: consultation_id2,
            consultant_id: consultant_id2,
            user_account_id: user_account_id2,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: fee_per_hour_in_yen2,
            sender_name: Some(
                generate_sender_name("スズキ".to_string(), "ジロウ".to_string(), meeting_at2)
                    .expect("failed to get Ok"),
            ),
        };

        let op = ExpiredAwaitingPaymentsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_payments: vec![awaiting_payment1.clone(), awaiting_payment2.clone()],
        };

        let result = handle_expired_awaiting_payments(page, per_page, current_date_time, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            ExpiredAwaitingPaymentResult {
                awaiting_payments: vec![awaiting_payment2]
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
        let awaiting_payment1 = AwaitingPayment {
            consultation_id: consultation_id1,
            consultant_id: consultant_id1,
            user_account_id: user_account_id1,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: fee_per_hour_in_yen1,
            sender_name: Some(
                generate_sender_name("タナカ".to_string(), "タロウ".to_string(), meeting_at1)
                    .expect("failed to get Ok"),
            ),
        };

        let consultation_id2 = 4;
        let consultant_id2 = 5;
        let user_account_id2 = 6;
        let meeting_at2 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 29, 17, 0, 0)
            .unwrap();
        let fee_per_hour_in_yen2 = 7000;
        let awaiting_payment2 = AwaitingPayment {
            consultation_id: consultation_id2,
            consultant_id: consultant_id2,
            user_account_id: user_account_id2,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: fee_per_hour_in_yen2,
            sender_name: Some(
                generate_sender_name("スズキ".to_string(), "ジロウ".to_string(), meeting_at2)
                    .expect("failed to get Ok"),
            ),
        };

        let op = ExpiredAwaitingPaymentsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_payments: vec![awaiting_payment1, awaiting_payment2],
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
    async fn handle_expired_awaiting_payments_success_case7() {
        let page = 0;
        let per_page = VALID_PAGE_SIZE;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 27, 21, 0, 40)
            .unwrap();

        let consultation_id1 = 1;
        let consultant_id1 = 2;
        let user_account_id1 = 3;
        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let fee_per_hour_in_yen1 = 5000;
        let awaiting_payment1 = AwaitingPayment {
            consultation_id: consultation_id1,
            consultant_id: consultant_id1,
            user_account_id: user_account_id1,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: fee_per_hour_in_yen1,
            sender_name: Some(
                generate_sender_name("タナカ".to_string(), "タロウ".to_string(), meeting_at1)
                    .expect("failed to get Ok"),
            ),
        };

        let consultation_id2 = 4;
        let consultant_id2 = 5;
        let user_account_id2 = 6;
        let meeting_at2 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 29, 17, 0, 0)
            .unwrap();
        let fee_per_hour_in_yen2 = 7000;
        let awaiting_payment2 = AwaitingPayment {
            consultation_id: consultation_id2,
            consultant_id: consultant_id2,
            user_account_id: user_account_id2,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: fee_per_hour_in_yen2,
            sender_name: Some(
                generate_sender_name("スズキ".to_string(), "ジロウ".to_string(), meeting_at2)
                    .expect("failed to get Ok"),
            ),
        };

        let op = ExpiredAwaitingPaymentsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_payments: vec![awaiting_payment1.clone(), awaiting_payment2],
        };

        let result = handle_expired_awaiting_payments(page, per_page, current_date_time, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            ExpiredAwaitingPaymentResult {
                awaiting_payments: vec![awaiting_payment1]
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
            awaiting_payments: vec![],
        };

        let result = handle_expired_awaiting_payments(page, per_page, current_date_time, op).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.0);
        assert_eq!(Code::UnexpectedErr as u32, resp.1.code);
    }
}
