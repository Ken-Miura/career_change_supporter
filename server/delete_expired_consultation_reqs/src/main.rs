// Copyright 2023 Ken Miura

use chrono::{DateTime, Duration, FixedOffset};
use dotenv::dotenv;
use entity::sea_orm::{
    prelude::async_trait::async_trait, ColumnTrait, ConnectOptions, Database, DatabaseConnection,
    EntityTrait, QueryFilter, QuerySelect, TransactionError, TransactionTrait,
};
use std::{error::Error, process::exit};

use common::{
    admin::{
        wait_for, TransactionExecutionError,
        DURATION_WAITING_FOR_DEPENDENT_SERVICE_RATE_LIMIT_IN_MILLI_SECONDS, KEY_TO_DB_ADMIN_NAME,
        KEY_TO_DB_ADMIN_PASSWORD, NUM_OF_MAX_TARGET_RECORDS,
    },
    db::{construct_db_url, KEY_TO_DB_HOST, KEY_TO_DB_NAME, KEY_TO_DB_PORT},
    payment_platform::{
        charge::{ChargeOperation, ChargeOperationImpl, RefundQuery},
        construct_access_info, AccessInfo, KEY_TO_PAYMENT_PLATFORM_API_PASSWORD,
        KEY_TO_PAYMENT_PLATFORM_API_URL, KEY_TO_PAYMENT_PLATFORM_API_USERNAME,
    },
    smtp::{
        SendMail, SmtpClient, ADMIN_EMAIL_ADDRESS, AWS_SES_ACCESS_KEY_ID, AWS_SES_ENDPOINT_URI,
        AWS_SES_REGION, AWS_SES_SECRET_ACCESS_KEY, KEY_TO_ADMIN_EMAIL_ADDRESS,
        KEY_TO_AWS_SES_ACCESS_KEY_ID, KEY_TO_AWS_SES_ENDPOINT_URI, KEY_TO_AWS_SES_REGION,
        KEY_TO_AWS_SES_SECRET_ACCESS_KEY, KEY_TO_SYSTEM_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS,
    },
    util::check_env_vars,
    JAPANESE_TIME_ZONE, WEB_SITE_NAME,
};

const SUCCESS: i32 = 0;
const ENV_VAR_CAPTURE_FAILURE: i32 = 1;
const CONNECTION_ERROR: i32 = 2;
const APPLICATION_ERR: i32 = 3;

fn main() {
    let _ = dotenv().ok();
    let result = check_env_vars(vec![
        KEY_TO_DB_HOST.to_string(),
        KEY_TO_DB_PORT.to_string(),
        KEY_TO_DB_NAME.to_string(),
        KEY_TO_DB_ADMIN_NAME.to_string(),
        KEY_TO_DB_ADMIN_PASSWORD.to_string(),
        KEY_TO_ADMIN_EMAIL_ADDRESS.to_string(),
        KEY_TO_SYSTEM_EMAIL_ADDRESS.to_string(),
        KEY_TO_AWS_SES_REGION.to_string(),
        KEY_TO_AWS_SES_ACCESS_KEY_ID.to_string(),
        KEY_TO_AWS_SES_SECRET_ACCESS_KEY.to_string(),
        KEY_TO_AWS_SES_ENDPOINT_URI.to_string(),
        KEY_TO_PAYMENT_PLATFORM_API_URL.to_string(),
        KEY_TO_PAYMENT_PLATFORM_API_USERNAME.to_string(),
        KEY_TO_PAYMENT_PLATFORM_API_PASSWORD.to_string(),
    ]);
    if result.is_err() {
        println!("failed to resolve mandatory env vars (following env vars are needed)");
        println!("{:?}", result.unwrap_err());
        exit(ENV_VAR_CAPTURE_FAILURE);
    }

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .expect("failed to build Runtime")
        .block_on(main_internal())
}

async fn main_internal() {
    let current_date_time = chrono::Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));

    let database_url = construct_db_url(
        KEY_TO_DB_HOST,
        KEY_TO_DB_PORT,
        KEY_TO_DB_NAME,
        KEY_TO_DB_ADMIN_NAME,
        KEY_TO_DB_ADMIN_PASSWORD,
    );
    let mut opt = ConnectOptions::new(database_url.clone());
    opt.max_connections(1).min_connections(1).sqlx_logging(true);
    let pool = Database::connect(opt).await.unwrap_or_else(|e| {
        println!("failed to connect database: {}", e);
        exit(CONNECTION_ERROR)
    });

    let access_info = match construct_access_info(
        KEY_TO_PAYMENT_PLATFORM_API_URL,
        KEY_TO_PAYMENT_PLATFORM_API_USERNAME,
        KEY_TO_PAYMENT_PLATFORM_API_PASSWORD,
    ) {
        Ok(ai) => ai,
        Err(e) => {
            println!("invalid PAYJP access info: {}", e);
            exit(ENV_VAR_CAPTURE_FAILURE);
        }
    };

    let op = DeleteExpiredConsultationReqsOperationImpl {
        pool,
        access_info,
        duration_per_iteration_in_milli_seconds:
            *DURATION_WAITING_FOR_DEPENDENT_SERVICE_RATE_LIMIT_IN_MILLI_SECONDS,
    };

    let smtp_client = SmtpClient::new(
        AWS_SES_REGION.as_str(),
        AWS_SES_ACCESS_KEY_ID.as_str(),
        AWS_SES_SECRET_ACCESS_KEY.as_str(),
        AWS_SES_ENDPOINT_URI.as_str(),
    )
    .await;

    let result = delete_expired_consultation_reqs(
        current_date_time,
        *NUM_OF_MAX_TARGET_RECORDS,
        &op,
        &smtp_client,
    )
    .await;

    let deleted_num = result.unwrap_or_else(|e| {
        println!("failed to delete expired consultation reqs: {}", e);
        exit(APPLICATION_ERR)
    });

    println!(
        "{} consultation req(s) were (was) deleted and its credit facility was released successfully",
        deleted_num
    );
    exit(SUCCESS)
}

async fn delete_expired_consultation_reqs(
    current_date_time: DateTime<FixedOffset>,
    num_of_max_target_records: u64,
    op: &impl DeleteExpiredConsultationReqsOperation,
    send_mail: &impl SendMail,
) -> Result<usize, Box<dyn Error>> {
    let criteria = current_date_time
        + Duration::seconds(common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64);
    let limit = if num_of_max_target_records != 0 {
        Some(num_of_max_target_records)
    } else {
        None
    };

    let expired_consultation_reqs = op.get_expired_consultation_reqs(criteria, limit).await?;
    let num_of_expired_consultation_reqs = expired_consultation_reqs.len();

    let mut delete_failed: Vec<ConsultationReq> =
        Vec::with_capacity(expired_consultation_reqs.len());
    for expired_consultation_req in expired_consultation_reqs {
        let req_id = expired_consultation_req.consultation_req_id;
        let charge_id = expired_consultation_req.charge_id.as_str();
        let result = op.delete_consultation_req(req_id, charge_id).await;
        if result.is_err() {
            println!("failed delete_consultation_req: {:?}", result);
            delete_failed.push(expired_consultation_req);
        }
        op.wait_for_dependent_service_rate_limit().await;
    }

    if !delete_failed.is_empty() {
        let subject = format!(
            "[{}] 定期実行ツール (delete_expired_consultation_reqs) 失敗通知",
            WEB_SITE_NAME
        );
        let num_of_delete_failed = delete_failed.len();
        let text = create_text(
            num_of_expired_consultation_reqs,
            num_of_delete_failed,
            &delete_failed,
        );
        let err_message = format!(
            "{} processed, {} failed (detail: {:?})",
            num_of_expired_consultation_reqs, num_of_delete_failed, delete_failed
        );
        send_mail
            .send_mail(
                ADMIN_EMAIL_ADDRESS.as_str(),
                SYSTEM_EMAIL_ADDRESS.as_str(),
                subject.as_str(),
                text.as_str(),
            )
            .await
            .map_err(|e| {
                format!(
                    "failed to send mail (status code: {}, response body: {:?}): {}",
                    e.0, e.1, err_message
                )
            })?;
        return Err(err_message.into());
    }

    Ok(num_of_expired_consultation_reqs)
}

#[async_trait]
trait DeleteExpiredConsultationReqsOperation {
    async fn get_expired_consultation_reqs(
        &self,
        criteria: DateTime<FixedOffset>,
        limit: Option<u64>,
    ) -> Result<Vec<ConsultationReq>, Box<dyn Error>>;

    async fn delete_consultation_req(
        &self,
        consultation_req_id: i64,
        charge_id: &str,
    ) -> Result<(), Box<dyn Error>>;

    async fn wait_for_dependent_service_rate_limit(&self);
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct ConsultationReq {
    consultation_req_id: i64,
    user_account_id: i64,
    consultant_id: i64,
    first_candidate_date_time: DateTime<FixedOffset>,
    second_candidate_date_time: DateTime<FixedOffset>,
    third_candidate_date_time: DateTime<FixedOffset>,
    latest_candidate_date_time: DateTime<FixedOffset>,
    charge_id: String,
    fee_per_hour_in_yen: i32,
    platform_fee_rate_in_percentage: String,
    credit_facilities_expired_at: DateTime<FixedOffset>,
}

struct DeleteExpiredConsultationReqsOperationImpl {
    pool: DatabaseConnection,
    access_info: AccessInfo,
    duration_per_iteration_in_milli_seconds: u64,
}

#[async_trait]
impl DeleteExpiredConsultationReqsOperation for DeleteExpiredConsultationReqsOperationImpl {
    async fn get_expired_consultation_reqs(
        &self,
        criteria: DateTime<FixedOffset>,
        limit: Option<u64>,
    ) -> Result<Vec<ConsultationReq>, Box<dyn Error>> {
        let models = entity::consultation_req::Entity::find()
            .filter(entity::consultation_req::Column::LatestCandidateDateTime.lte(criteria)) // 受け付け時の仕様と合わせて <= とする
            .limit(limit)
            .all(&self.pool)
            .await
            .map_err(|e| format!("failed to get consultation_req: {}", e))?;
        Ok(models
            .into_iter()
            .map(|m| ConsultationReq {
                consultation_req_id: m.consultation_req_id,
                user_account_id: m.user_account_id,
                consultant_id: m.consultant_id,
                first_candidate_date_time: m.first_candidate_date_time,
                second_candidate_date_time: m.second_candidate_date_time,
                third_candidate_date_time: m.third_candidate_date_time,
                latest_candidate_date_time: m.latest_candidate_date_time,
                charge_id: m.charge_id,
                fee_per_hour_in_yen: m.fee_per_hour_in_yen,
                platform_fee_rate_in_percentage: m.platform_fee_rate_in_percentage,
                credit_facilities_expired_at: m.credit_facilities_expired_at,
            })
            .collect())
    }

    async fn delete_consultation_req(
        &self,
        consultation_req_id: i64,
        charge_id: &str,
    ) -> Result<(), Box<dyn Error>> {
        let charge_id = charge_id.to_string();
        let access_info = self.access_info.clone();
        let _ = self
            .pool
            .transaction::<_, (), TransactionExecutionError>(|txn| {
                Box::pin(async move {
                    let _ = entity::consultation_req::Entity::delete_by_id(consultation_req_id)
                        .exec(txn)
                        .await
                        .map_err(|e| TransactionExecutionError {
                            message: format!(
                                "failed to delete consultation_req (consultation_req_id: {}): {}",
                                consultation_req_id, e
                            ),
                        })?;

                    let charge_op = ChargeOperationImpl::new(&access_info);
                    let refund_reason = "credit_facility_was_released_automatically_because_consultantion_date_has_been_outdated".to_string();
                    let query = RefundQuery::new(refund_reason).map_err(|e|{
                        TransactionExecutionError {
                            message: format!("failed to create RefundQuery: {}", e)
                        }
                    })?;
                    // ここで実施していることは、返金ではなく与信枠の開放のため、refundテーブルへのレコード作成は行わない
                    // 実施していることが与信枠開放にも関わらず、refundというAPI名なのは、PAYJPが提供しているAPIと合わせているため
                    let _ = charge_op.refund(&charge_id, query).await.map_err(|e| {
                        TransactionExecutionError {
                            message: format!("failed to release credit faclity (charge_id: {}): {}", charge_id, e)
                        }
                    })?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(db_err) => {
                    format!("connection error: {}", db_err)
                }
                TransactionError::Transaction(transaction_err) => {
                    format!("transaction error: {}", transaction_err)
                }
            })?;
        Ok(())
    }

    async fn wait_for_dependent_service_rate_limit(&self) {
        wait_for(self.duration_per_iteration_in_milli_seconds).await;
    }
}

fn create_text(
    num_of_expired_consultation_reqs: usize,
    num_of_delete_failed: usize,
    delete_failed: &[ConsultationReq],
) -> String {
    format!(
        r"consultation_reqの期限切れレコード{}個の内、{}個の削除に失敗しました。

【詳細】
{:?}",
        num_of_expired_consultation_reqs, num_of_delete_failed, delete_failed
    )
}

#[cfg(test)]
mod tests {

    use std::{cmp::min, collections::HashMap};

    use chrono::TimeZone;
    use common::ErrResp;

    use super::*;

    struct DeleteExpiredConsultationReqsOperationMock {
        consultation_reqs: HashMap<i64, (ConsultationReq, bool)>,
        current_date_time: DateTime<FixedOffset>,
        limit: u64,
    }

    #[async_trait]
    impl DeleteExpiredConsultationReqsOperation for DeleteExpiredConsultationReqsOperationMock {
        async fn get_expired_consultation_reqs(
            &self,
            criteria: DateTime<FixedOffset>,
            limit: Option<u64>,
        ) -> Result<Vec<ConsultationReq>, Box<dyn Error>> {
            assert_eq!(
                self.current_date_time
                    + Duration::seconds(
                        common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64
                    ),
                criteria
            );
            if self.limit != 0 {
                assert_eq!(Some(self.limit), limit);
            } else {
                assert_eq!(None, limit);
            }
            let expired_consultation_reqs: Vec<ConsultationReq> = self
                .consultation_reqs
                .values()
                .clone()
                .filter(|m| m.0.latest_candidate_date_time <= criteria)
                .map(|m| m.0.clone())
                .collect();
            let results = if let Some(limit) = limit {
                let limit = min(limit as usize, expired_consultation_reqs.len());
                let mut expired_consultation_reqs_limited = Vec::with_capacity(limit);
                (0..limit).for_each(|i| {
                    expired_consultation_reqs_limited.push(expired_consultation_reqs[i].clone())
                });
                expired_consultation_reqs_limited
            } else {
                expired_consultation_reqs
            };
            Ok(results)
        }

        async fn delete_consultation_req(
            &self,
            consultation_req_id: i64,
            charge_id: &str,
        ) -> Result<(), Box<dyn Error>> {
            let consultation_req = self
                .consultation_reqs
                .get(&consultation_req_id)
                .expect("assert that consultation_req has value!");
            assert_eq!(consultation_req.0.charge_id, charge_id);
            if !consultation_req.1 {
                return Err("mock error message".into());
            }
            Ok(())
        }

        async fn wait_for_dependent_service_rate_limit(&self) {
            // テストコードでは待つ必要はないので何もしない
        }
    }

    #[derive(Clone, Debug)]
    pub(super) struct SendMailMock {
        to: String,
        from: String,
        subject: String,
        text_keywords: Vec<String>,
    }

    impl SendMailMock {
        pub(super) fn new(
            to: String,
            from: String,
            subject: String,
            text_keywords: Vec<String>,
        ) -> Self {
            Self {
                to,
                from,
                subject,
                text_keywords,
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
            for text_keyword in self.text_keywords.clone() {
                assert!(text.contains(&text_keyword));
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn delete_expired_consultation_reqs_success0() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredConsultationReqsOperationMock {
            consultation_reqs: HashMap::with_capacity(0),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_consultation_reqs(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 0);
    }

    #[tokio::test]
    async fn delete_expired_consultation_reqs_success1() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredConsultationReqsOperationMock {
            consultation_reqs: create_dummy_1_non_expired_consultation_req(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_consultation_reqs(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 0);
    }

    fn create_dummy_1_non_expired_consultation_req(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (ConsultationReq, bool)> {
        let consultation_req_id = 1234;
        let consultation_req = ConsultationReq {
            consultation_req_id,
            user_account_id: 456,
            consultant_id: 789,
            first_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 25, 13, 0, 0)
                .unwrap(),
            second_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 26, 14, 0, 0)
                .unwrap(),
            third_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
                .unwrap(),
            // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第三希望日時）とは異なるものを入れる。
            latest_candidate_date_time: current_date_time
                + Duration::seconds(
                    common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
                )
                + Duration::seconds(1),
            charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
            fee_per_hour_in_yen: 5000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 10, 19, 15, 0, 0)
                .unwrap(),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(consultation_req_id, (consultation_req, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_consultation_reqs_success2() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredConsultationReqsOperationMock {
            consultation_reqs: create_dummy_1_expired_consultation_req(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_consultation_reqs(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 1);
    }

    fn create_dummy_1_expired_consultation_req(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (ConsultationReq, bool)> {
        let consultation_req_id = 1234;
        let consultation_req = ConsultationReq {
            consultation_req_id,
            user_account_id: 456,
            consultant_id: 789,
            first_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 25, 13, 0, 0)
                .unwrap(),
            second_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 26, 14, 0, 0)
                .unwrap(),
            third_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
                .unwrap(),
            // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第三希望日時）とは異なるものを入れる。
            latest_candidate_date_time: current_date_time
                + Duration::seconds(
                    common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
                ),
            charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
            fee_per_hour_in_yen: 5000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 10, 19, 15, 0, 0)
                .unwrap(),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(consultation_req_id, (consultation_req, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_consultation_reqs_success3() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 00, 40)
            .unwrap();
        let max_num_of_target_records = 1;
        let op = DeleteExpiredConsultationReqsOperationMock {
            consultation_reqs: create_dummy_1_expired_consultation_req(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_consultation_reqs(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 1);
    }

    #[tokio::test]
    async fn delete_expired_consultation_reqs_success4() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 00, 40)
            .unwrap();
        let max_num_of_target_records = 2;
        let op = DeleteExpiredConsultationReqsOperationMock {
            consultation_reqs: create_dummy_1_expired_consultation_req(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_consultation_reqs(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 1);
    }

    #[tokio::test]
    async fn delete_expired_consultation_reqs_success5() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredConsultationReqsOperationMock {
            consultation_reqs: create_dummy_2_expired_consultation_reqs(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_consultation_reqs(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 2);
    }

    fn create_dummy_2_expired_consultation_reqs(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (ConsultationReq, bool)> {
        let consultation_req_id1 = 1234;
        let consultation_req1 = ConsultationReq {
            consultation_req_id: consultation_req_id1,
            user_account_id: 456,
            consultant_id: 789,
            first_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 25, 13, 0, 0)
                .unwrap(),
            second_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 26, 14, 0, 0)
                .unwrap(),
            third_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
                .unwrap(),
            // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第三希望日時）とは異なるものを入れる。
            latest_candidate_date_time: current_date_time
                + Duration::seconds(
                    common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
                ),
            charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
            fee_per_hour_in_yen: 5000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 10, 19, 15, 0, 0)
                .unwrap(),
        };

        let consultation_req_id2 = 56;
        let consultation_req2 = ConsultationReq {
            consultation_req_id: consultation_req_id2,
            user_account_id: 32,
            consultant_id: 87,
            first_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 25, 7, 0, 0)
                .unwrap(),
            second_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
                .unwrap(),
            third_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 23, 21, 0, 0)
                .unwrap(),
            // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第二希望日時）とは異なるものを入れる。
            latest_candidate_date_time: current_date_time
                + Duration::seconds(
                    common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
                ),
            charge_id: "ch_ea990a4c10672a93053a774730b0b".to_string(),
            fee_per_hour_in_yen: 8000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 10, 17, 15, 0, 0)
                .unwrap(),
        };

        let mut map = HashMap::with_capacity(2);
        map.insert(consultation_req_id1, (consultation_req1, true));
        map.insert(consultation_req_id2, (consultation_req2, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_consultation_reqs_success6() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 21, 8, 40)
            .unwrap();
        let max_num_of_target_records = 1;
        let op = DeleteExpiredConsultationReqsOperationMock {
            consultation_reqs: create_dummy_2_expired_consultation_reqs(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_consultation_reqs(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 1);
    }

    #[tokio::test]
    async fn delete_expired_consultation_reqs_success7() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 21, 8, 40)
            .unwrap();
        let max_num_of_target_records = 2;
        let op = DeleteExpiredConsultationReqsOperationMock {
            consultation_reqs: create_dummy_2_expired_consultation_reqs(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_consultation_reqs(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 2);
    }

    #[tokio::test]
    async fn delete_expired_consultation_reqs_success8() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 0, 40)
            .unwrap();
        let max_num_of_target_records = 3;
        let op = DeleteExpiredConsultationReqsOperationMock {
            consultation_reqs: create_dummy_2_expired_consultation_reqs(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_consultation_reqs(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 2);
    }

    #[tokio::test]
    async fn delete_expired_consultation_reqs_success9() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 0, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredConsultationReqsOperationMock {
            consultation_reqs: create_dummy_1_non_expired_and_1_expired_consultation_req(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_consultation_reqs(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 1);
    }

    fn create_dummy_1_non_expired_and_1_expired_consultation_req(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (ConsultationReq, bool)> {
        let consultation_req_id1 = 1234;
        let consultation_req1 = ConsultationReq {
            consultation_req_id: consultation_req_id1,
            user_account_id: 456,
            consultant_id: 789,
            first_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 25, 13, 0, 0)
                .unwrap(),
            second_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 26, 14, 0, 0)
                .unwrap(),
            third_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
                .unwrap(),
            // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第三希望日時）とは異なるものを入れる。
            latest_candidate_date_time: current_date_time
                + Duration::seconds(
                    common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
                )
                + Duration::seconds(1),
            charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
            fee_per_hour_in_yen: 5000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 10, 19, 15, 0, 0)
                .unwrap(),
        };

        let consultation_req_id2 = 56;
        let consultation_req2 = ConsultationReq {
            consultation_req_id: consultation_req_id2,
            user_account_id: 32,
            consultant_id: 87,
            first_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 25, 7, 0, 0)
                .unwrap(),
            second_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
                .unwrap(),
            third_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 23, 21, 0, 0)
                .unwrap(),
            // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第二希望日時）とは異なるものを入れる。
            latest_candidate_date_time: current_date_time
                + Duration::seconds(
                    common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
                ),
            charge_id: "ch_ea990a4c10672a93053a774730b0b".to_string(),
            fee_per_hour_in_yen: 8000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 10, 17, 15, 0, 0)
                .unwrap(),
        };

        let mut map = HashMap::with_capacity(2);
        map.insert(consultation_req_id1, (consultation_req1, true));
        map.insert(consultation_req_id2, (consultation_req2, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_consultation_reqs_success10() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 0, 40)
            .unwrap();
        let max_num_of_target_records = 1;
        let op = DeleteExpiredConsultationReqsOperationMock {
            consultation_reqs: create_dummy_1_non_expired_and_1_expired_consultation_req(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_consultation_reqs(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 1);
    }

    #[tokio::test]
    async fn delete_expired_consultation_reqs_success11() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 0, 40)
            .unwrap();
        let max_num_of_target_records = 2;
        let op = DeleteExpiredConsultationReqsOperationMock {
            consultation_reqs: create_dummy_1_non_expired_and_1_expired_consultation_req(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_consultation_reqs(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 1);
    }

    #[tokio::test]
    async fn delete_expired_consultation_reqs_fail1() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 0, 00)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredConsultationReqsOperationMock {
            consultation_reqs: create_dummy_1_failed_expired_consultation_req(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            format!(
                "[{}] 定期実行ツール (delete_expired_consultation_reqs) 失敗通知",
                WEB_SITE_NAME
            ),
            vec![
                "consultation_reqの期限切れレコード1個の内、1個の削除に失敗しました。".to_string(),
                "1234".to_string(),
                "456".to_string(),
                "789".to_string(),
                "ch_fa990a4c10672a93053a774730b0a".to_string(),
                "2023-08-27T14:00:00+09:00".to_string(),
            ],
        );

        let result = delete_expired_consultation_reqs(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let err = result.expect_err("failed to get Err");
        let err_message = err.to_string();
        assert!(err_message.contains("1 processed, 1 failed"));
        assert!(err_message.contains("1234"));
        assert!(err_message.contains("456"));
        assert!(err_message.contains("789"));
        assert!(err_message.contains("ch_fa990a4c10672a93053a774730b0a"));
        assert!(err_message.contains("2023-08-27T14:00:00+09:00"));
    }

    fn create_dummy_1_failed_expired_consultation_req(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (ConsultationReq, bool)> {
        let consultation_req_id = 1234;
        let consultation_req = ConsultationReq {
            consultation_req_id,
            user_account_id: 456,
            consultant_id: 789,
            first_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 25, 13, 0, 0)
                .unwrap(),
            second_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 26, 14, 0, 0)
                .unwrap(),
            third_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
                .unwrap(),
            // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第三希望日時）とは異なるものを入れる。
            latest_candidate_date_time: current_date_time
                + Duration::seconds(
                    common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
                ),
            charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
            fee_per_hour_in_yen: 5000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 10, 19, 15, 0, 0)
                .unwrap(),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(consultation_req_id, (consultation_req, false));
        map
    }

    #[tokio::test]
    async fn delete_expired_consultation_reqs_fail2() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 0, 00)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredConsultationReqsOperationMock {
            consultation_reqs: create_dummy_2_failed_expired_consultation_reqs(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            format!(
                "[{}] 定期実行ツール (delete_expired_consultation_reqs) 失敗通知",
                WEB_SITE_NAME
            ),
            vec![
                "consultation_reqの期限切れレコード2個の内、2個の削除に失敗しました。".to_string(),
                "1234".to_string(),
                "456".to_string(),
                "789".to_string(),
                "ch_fa990a4c10672a93053a774730b0a".to_string(),
                "2023-08-27T14:00:00+09:00".to_string(),
                "56".to_string(),
                "32".to_string(),
                "87".to_string(),
                "ch_ea990a4c10672a93053a774730b0b".to_string(),
                "2023-08-27T14:00:00+09:00".to_string(),
            ],
        );

        let result = delete_expired_consultation_reqs(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let err = result.expect_err("failed to get Err");
        let err_message = err.to_string();
        assert!(err_message.contains("2 processed, 2 failed"));

        assert!(err_message.contains("1234"));
        assert!(err_message.contains("456"));
        assert!(err_message.contains("789"));
        assert!(err_message.contains("ch_fa990a4c10672a93053a774730b0a"));
        assert!(err_message.contains("2023-08-27T14:00:00+09:00"));

        assert!(err_message.contains("56"));
        assert!(err_message.contains("32"));
        assert!(err_message.contains("87"));
        assert!(err_message.contains("ch_ea990a4c10672a93053a774730b0b"));
        assert!(err_message.contains("2023-08-27T14:00:00+09:00"));
    }

    fn create_dummy_2_failed_expired_consultation_reqs(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (ConsultationReq, bool)> {
        let consultation_req_id1 = 1234;
        let consultation_req1 = ConsultationReq {
            consultation_req_id: consultation_req_id1,
            user_account_id: 456,
            consultant_id: 789,
            first_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 25, 13, 0, 0)
                .unwrap(),
            second_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 26, 14, 0, 0)
                .unwrap(),
            third_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
                .unwrap(),
            // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第三希望日時）とは異なるものを入れる。
            latest_candidate_date_time: current_date_time
                + Duration::seconds(
                    common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
                ),
            charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
            fee_per_hour_in_yen: 5000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 10, 19, 15, 0, 0)
                .unwrap(),
        };

        let consultation_req_id2 = 56;
        let consultation_req2 = ConsultationReq {
            consultation_req_id: consultation_req_id2,
            user_account_id: 32,
            consultant_id: 87,
            first_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 25, 7, 0, 0)
                .unwrap(),
            second_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
                .unwrap(),
            third_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 23, 21, 0, 0)
                .unwrap(),
            // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第二希望日時）とは異なるものを入れる。
            latest_candidate_date_time: current_date_time
                + Duration::seconds(
                    common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
                ),
            charge_id: "ch_ea990a4c10672a93053a774730b0b".to_string(),
            fee_per_hour_in_yen: 8000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 10, 17, 15, 0, 0)
                .unwrap(),
        };

        let mut map = HashMap::with_capacity(2);
        map.insert(consultation_req_id1, (consultation_req1, false));
        map.insert(consultation_req_id2, (consultation_req2, false));
        map
    }

    #[tokio::test]
    async fn delete_expired_consultation_reqs_fail3() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 0, 00)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredConsultationReqsOperationMock {
            consultation_reqs:
                create_dummy_1_failed_expired_consultation_req_and_1_expired_consultation_req(
                    current_date_time,
                ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            format!(
                "[{}] 定期実行ツール (delete_expired_consultation_reqs) 失敗通知",
                WEB_SITE_NAME
            ),
            vec![
                "consultation_reqの期限切れレコード2個の内、1個の削除に失敗しました。".to_string(),
                "56".to_string(),
                "32".to_string(),
                "87".to_string(),
                "ch_ea990a4c10672a93053a774730b0b".to_string(),
                "2023-08-27T14:00:00+09:00".to_string(),
            ],
        );

        let result = delete_expired_consultation_reqs(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let err = result.expect_err("failed to get Err");
        let err_message = err.to_string();
        assert!(err_message.contains("2 processed, 1 failed"));

        assert!(!err_message.contains("1234"));
        assert!(!err_message.contains("456"));
        assert!(!err_message.contains("789"));
        assert!(!err_message.contains("ch_fa990a4c10672a93053a774730b0a"));
        assert!(!err_message.contains("2023-08-27 13:00:00 +09:00"));

        assert!(err_message.contains("56"));
        assert!(err_message.contains("32"));
        assert!(err_message.contains("87"));
        assert!(err_message.contains("ch_ea990a4c10672a93053a774730b0b"));
        assert!(err_message.contains("2023-08-27T14:00:00+09:00"));
    }

    fn create_dummy_1_failed_expired_consultation_req_and_1_expired_consultation_req(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (ConsultationReq, bool)> {
        let consultation_req_id1 = 1234;
        let consultation_req1 = ConsultationReq {
            consultation_req_id: consultation_req_id1,
            user_account_id: 456,
            consultant_id: 789,
            first_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 25, 13, 0, 0)
                .unwrap(),
            second_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 26, 14, 0, 0)
                .unwrap(),
            third_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
                .unwrap(),
            // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第三希望日時）とは異なるものを入れる。
            latest_candidate_date_time: current_date_time
                + Duration::seconds(
                    common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
                )
                - Duration::hours(1),
            charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
            fee_per_hour_in_yen: 5000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 10, 19, 15, 0, 0)
                .unwrap(),
        };

        let consultation_req_id2 = 56;
        let consultation_req2 = ConsultationReq {
            consultation_req_id: consultation_req_id2,
            user_account_id: 32,
            consultant_id: 87,
            first_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 25, 7, 0, 0)
                .unwrap(),
            second_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
                .unwrap(),
            third_candidate_date_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 23, 21, 0, 0)
                .unwrap(),
            // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第二希望日時）とは異なるものを入れる。
            latest_candidate_date_time: current_date_time
                + Duration::seconds(
                    common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
                ),
            charge_id: "ch_ea990a4c10672a93053a774730b0b".to_string(),
            fee_per_hour_in_yen: 8000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 10, 17, 15, 0, 0)
                .unwrap(),
        };

        let mut map = HashMap::with_capacity(2);
        map.insert(consultation_req_id1, (consultation_req1, true));
        map.insert(consultation_req_id2, (consultation_req2, false));
        map
    }
}
