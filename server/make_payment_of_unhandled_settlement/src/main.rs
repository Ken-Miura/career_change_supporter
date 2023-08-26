// Copyright 2023 Ken Miura

use chrono::{DateTime, Duration, FixedOffset};
use dotenv::dotenv;
use entity::sea_orm::{
    prelude::async_trait::async_trait, ActiveModelTrait, ActiveValue::NotSet, ColumnTrait,
    ConnectOptions, Database, DatabaseConnection, DatabaseTransaction, EntityTrait, ModelTrait,
    QueryFilter, QuerySelect, Set, TransactionError, TransactionTrait,
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
        charge::{ChargeOperation, ChargeOperationImpl},
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
    JAPANESE_TIME_ZONE, LENGTH_OF_MEETING_IN_MINUTE, WEB_SITE_NAME,
};

const SUCCESS: i32 = 0;
const ENV_VAR_CAPTURE_FAILURE: i32 = 1;
const CONNECTION_ERROR: i32 = 2;
const APPLICATION_ERR: i32 = 3;

const DURATION_ALLOWED_AS_UNHANDLED_IN_DAYS: i64 = 14;

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

    let op = MakePaymentOfUnhandledSettlementOperationImpl {
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

    let result = make_payment_of_unhandled_settlement(
        current_date_time,
        *NUM_OF_MAX_TARGET_RECORDS,
        &op,
        &smtp_client,
    )
    .await;

    let num_of_handled = result.unwrap_or_else(|e| {
        println!("failed to make payment of unhandled settlement: {}", e);
        exit(APPLICATION_ERR)
    });

    println!("{} payments were made successfully", num_of_handled);
    exit(SUCCESS)
}

async fn make_payment_of_unhandled_settlement(
    current_date_time: DateTime<FixedOffset>,
    num_of_max_target_records: u64,
    op: &impl MakePaymentOfUnhandledSettlementOperation,
    send_mail: &impl SendMail,
) -> Result<usize, Box<dyn Error>> {
    let criteria = current_date_time
        - Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
        - Duration::days(DURATION_ALLOWED_AS_UNHANDLED_IN_DAYS);
    let limit = if num_of_max_target_records != 0 {
        Some(num_of_max_target_records)
    } else {
        None
    };

    let unhandled_settlements = op.get_unhandled_settlements(criteria, limit).await?;
    let num_of_unhandled_settlements = unhandled_settlements.len();

    let mut make_payment_failed: Vec<Settlement> = Vec::with_capacity(num_of_unhandled_settlements);
    for unhandled_settlement in unhandled_settlements {
        let settlement_id = unhandled_settlement.settlement_id;
        let result = op.make_payment(settlement_id, current_date_time).await;
        if result.is_err() {
            println!("failed make_payment: {:?}", result);
            make_payment_failed.push(unhandled_settlement);
        }
        op.wait_for_dependent_service_rate_limit().await;
    }

    if !make_payment_failed.is_empty() {
        let subject = format!(
            "[{}] 定期実行ツール (make_payment_of_unhandled_settlement) 失敗通知",
            WEB_SITE_NAME
        );
        let num_of_make_payment_failed = make_payment_failed.len();
        let text = create_text(
            num_of_unhandled_settlements,
            num_of_make_payment_failed,
            &make_payment_failed,
        );
        let err_message = format!(
            "{} were processed, {} were failed (detail: {:?})",
            num_of_unhandled_settlements, num_of_make_payment_failed, make_payment_failed
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

    Ok(num_of_unhandled_settlements)
}

#[async_trait]
trait MakePaymentOfUnhandledSettlementOperation {
    async fn get_unhandled_settlements(
        &self,
        criteria: DateTime<FixedOffset>,
        limit: Option<u64>,
    ) -> Result<Vec<Settlement>, Box<dyn Error>>;

    async fn make_payment(
        &self,
        settlement_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), Box<dyn Error>>;

    async fn wait_for_dependent_service_rate_limit(&self);
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct Settlement {
    settlement_id: i64,
    consultation_id: i64,
    charge_id: String,
    fee_per_hour_in_yen: i32,
    platform_fee_rate_in_percentage: String,
    credit_facilities_expired_at: DateTime<FixedOffset>,
}

struct MakePaymentOfUnhandledSettlementOperationImpl {
    pool: DatabaseConnection,
    access_info: AccessInfo,
    duration_per_iteration_in_milli_seconds: u64,
}

#[async_trait]
impl MakePaymentOfUnhandledSettlementOperation for MakePaymentOfUnhandledSettlementOperationImpl {
    async fn get_unhandled_settlements(
        &self,
        criteria: DateTime<FixedOffset>,
        limit: Option<u64>,
    ) -> Result<Vec<Settlement>, Box<dyn Error>> {
        let models = entity::settlement::Entity::find()
            .find_also_related(entity::consultation::Entity)
            .filter(entity::consultation::Column::MeetingAt.lt(criteria))
            .limit(limit)
            .all(&self.pool)
            .await
            .map_err(|e| format!("failed to get settlement and consultation: {}", e))?;
        Ok(models
            .into_iter()
            .map(|m| Settlement {
                settlement_id: m.0.settlement_id,
                consultation_id: m.0.consultation_id,
                charge_id: m.0.charge_id,
                fee_per_hour_in_yen: m.0.fee_per_hour_in_yen,
                platform_fee_rate_in_percentage: m.0.platform_fee_rate_in_percentage,
                credit_facilities_expired_at: m.0.credit_facilities_expired_at,
            })
            .collect())
    }

    async fn make_payment(
        &self,
        settlement_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), Box<dyn Error>> {
        let access_info = self.access_info.clone();
        let _ = self
            .pool
            .transaction::<_, (), TransactionExecutionError>(|txn| {
                Box::pin(async move {
                    let settlement_model = entity::settlement::Entity::find_by_id(settlement_id)
                        .lock_exclusive()
                        .one(txn)
                        .await
                        .map_err(|e| TransactionExecutionError {
                            message: format!(
                                "failed to find settlement (settlement_id: {}): {}",
                                settlement_id, e
                            ),
                        })?;
                    let settlement_model = match settlement_model {
                        Some(m) => m,
                        None => {
                            println!("no settlement (settlement_id: {}) found (maybe it has been already handled)", settlement_id);
                            return Ok(());
                        }
                    };

                    insert_receipt(txn, &settlement_model, current_date_time).await?;

                    let charge_id = settlement_model.charge_id.clone();
                    let _ = settlement_model.delete(txn).await.map_err(|e| {
                        TransactionExecutionError { message: format!("failed to delete settlement (settlement_id {}): {}", settlement_id, e) }
                    })?;

                    let charge_op = ChargeOperationImpl::new(&access_info);
                    let _ = charge_op
                        .capture_the_charge(charge_id.as_str())
                        .await
                        .map_err(|e| {
                            TransactionExecutionError { message: format!("failed to capture the charge (charge_id {}): {}", charge_id, e) }
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

async fn insert_receipt(
    txn: &DatabaseTransaction,
    model: &entity::settlement::Model,
    current_date_time: DateTime<FixedOffset>,
) -> Result<(), TransactionExecutionError> {
    let active_model = entity::receipt::ActiveModel {
        receipt_id: NotSet,
        consultation_id: Set(model.consultation_id),
        charge_id: Set(model.charge_id.clone()),
        fee_per_hour_in_yen: Set(model.fee_per_hour_in_yen),
        platform_fee_rate_in_percentage: Set(model.platform_fee_rate_in_percentage.clone()),
        settled_at: Set(current_date_time),
    };
    let _ = active_model
        .insert(txn)
        .await
        .map_err(|e| TransactionExecutionError {
            message: format!("failed to insert receipt (settlement: {:?}): {}", model, e),
        })?;
    Ok(())
}

fn create_text(
    num_of_unhandled_settlements: usize,
    num_of_make_payment_failed: usize,
    make_payment_failed: &[Settlement],
) -> String {
    format!(
        r"処理されていないsettlementレコード{}個の内、{}個の処理に失敗しました。

【詳細】
{:?}",
        num_of_unhandled_settlements, num_of_make_payment_failed, make_payment_failed
    )
}

#[cfg(test)]
mod tests {

    use std::{cmp::min, collections::HashMap};

    use chrono::TimeZone;
    use common::ErrResp;

    use super::*;

    struct MakePaymentOfUnhandledSettlementOperationMock {
        settlements: HashMap<i64, (Settlement, DateTime<FixedOffset>, bool)>,
        current_date_time: DateTime<FixedOffset>,
        limit: u64,
    }

    #[async_trait]
    impl MakePaymentOfUnhandledSettlementOperation for MakePaymentOfUnhandledSettlementOperationMock {
        async fn get_unhandled_settlements(
            &self,
            criteria: DateTime<FixedOffset>,
            limit: Option<u64>,
        ) -> Result<Vec<Settlement>, Box<dyn Error>> {
            assert_eq!(
                self.current_date_time
                    - Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
                    - Duration::days(DURATION_ALLOWED_AS_UNHANDLED_IN_DAYS),
                criteria
            );
            if self.limit != 0 {
                assert_eq!(Some(self.limit), limit);
            } else {
                assert_eq!(None, limit);
            }
            let unhandled_settlements: Vec<Settlement> = self
                .settlements
                .values()
                .clone()
                .filter(|m| m.1 < criteria)
                .map(|m| m.0.clone())
                .collect();
            let results = if let Some(limit) = limit {
                let limit = min(limit as usize, unhandled_settlements.len());
                let mut unhandled_settlements_limited = Vec::with_capacity(limit);
                (0..limit).for_each(|i| {
                    unhandled_settlements_limited.push(unhandled_settlements[i].clone())
                });
                unhandled_settlements_limited
            } else {
                unhandled_settlements
            };
            Ok(results)
        }

        async fn make_payment(
            &self,
            settlement_id: i64,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<(), Box<dyn Error>> {
            let settlement = self
                .settlements
                .get(&settlement_id)
                .expect("assert that settlement has value!");
            assert_eq!(self.current_date_time, current_date_time);
            if !settlement.2 {
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
    async fn make_payment_of_unhandled_settlement_success0() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = MakePaymentOfUnhandledSettlementOperationMock {
            settlements: HashMap::with_capacity(0),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = make_payment_of_unhandled_settlement(
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
    async fn make_payment_of_unhandled_settlement_success1() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = MakePaymentOfUnhandledSettlementOperationMock {
            settlements: create_dummy_1_settlement(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = make_payment_of_unhandled_settlement(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 0);
    }

    fn create_dummy_1_settlement(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (Settlement, DateTime<FixedOffset>, bool)> {
        let settlement_id = 1234;
        let settlement = Settlement {
            settlement_id,
            consultation_id: 34,
            charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
            fee_per_hour_in_yen: 5000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 10, 19, 15, 0, 0)
                .unwrap(),
        };
        let meeting_at = current_date_time
            - Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            - Duration::days(DURATION_ALLOWED_AS_UNHANDLED_IN_DAYS);

        let mut map = HashMap::with_capacity(1);
        map.insert(settlement_id, (settlement, meeting_at, true));
        map
    }

    #[tokio::test]
    async fn make_payment_of_unhandled_settlement_success2() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = MakePaymentOfUnhandledSettlementOperationMock {
            settlements: create_dummy_1_unhandled_settlement(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = make_payment_of_unhandled_settlement(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 1);
    }

    fn create_dummy_1_unhandled_settlement(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (Settlement, DateTime<FixedOffset>, bool)> {
        let settlement_id = 1234;
        let settlement = Settlement {
            settlement_id,
            consultation_id: 34,
            charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
            fee_per_hour_in_yen: 5000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 10, 19, 15, 0, 0)
                .unwrap(),
        };
        let meeting_at = current_date_time
            - Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            - Duration::days(DURATION_ALLOWED_AS_UNHANDLED_IN_DAYS)
            - Duration::seconds(1);

        let mut map = HashMap::with_capacity(1);
        map.insert(settlement_id, (settlement, meeting_at, true));
        map
    }

    #[tokio::test]
    async fn make_payment_of_unhandled_settlement_success3() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 00, 40)
            .unwrap();
        let max_num_of_target_records = 1;
        let op = MakePaymentOfUnhandledSettlementOperationMock {
            settlements: create_dummy_1_unhandled_settlement(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = make_payment_of_unhandled_settlement(
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
    async fn make_payment_of_unhandled_settlement_success4() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 00, 40)
            .unwrap();
        let max_num_of_target_records = 2;
        let op = MakePaymentOfUnhandledSettlementOperationMock {
            settlements: create_dummy_1_unhandled_settlement(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = make_payment_of_unhandled_settlement(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 1);
    }

    // #[tokio::test]
    // async fn make_payment_of_unhandled_settlement_success5() {
    //     let current_date_time = JAPANESE_TIME_ZONE
    //         .with_ymd_and_hms(2023, 8, 27, 8, 00, 40)
    //         .unwrap();
    //     let max_num_of_target_records = 0;
    //     let op = MakePaymentOfUnhandledSettlementOperationMock {
    //         settlements: create_dummy_2_expired_settlements(current_date_time),
    //         current_date_time,
    //         limit: max_num_of_target_records,
    //     };
    //     // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
    //     let send_mail_mock =
    //         SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

    //     let result = make_payment_of_unhandled_settlement(
    //         current_date_time,
    //         max_num_of_target_records,
    //         &op,
    //         &send_mail_mock,
    //     )
    //     .await;

    //     let num_deleted = result.expect("failed to get Ok");
    //     assert_eq!(num_deleted, 2);
    // }

    // fn create_dummy_2_expired_settlements(
    //     current_date_time: DateTime<FixedOffset>,
    // ) -> HashMap<i64, (Settlement, bool)> {
    //     let settlement_id1 = 1234;
    //     let settlement1 = Settlement {
    //         settlement_id: settlement_id1,
    //         user_account_id: 456,
    //         consultant_id: 789,
    //         first_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 25, 13, 0, 0)
    //             .unwrap(),
    //         second_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 26, 14, 0, 0)
    //             .unwrap(),
    //         third_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
    //             .unwrap(),
    //         // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第三希望日時）とは異なるものを入れる。
    //         latest_candidate_date_time: current_date_time
    //             + Duration::seconds(
    //                 common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
    //             ),
    //         charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
    //         fee_per_hour_in_yen: 5000,
    //         platform_fee_rate_in_percentage: "30.0".to_string(),
    //         credit_facilities_expired_at: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 10, 19, 15, 0, 0)
    //             .unwrap(),
    //     };

    //     let settlement_id2 = 56;
    //     let settlement2 = Settlement {
    //         settlement_id: settlement_id2,
    //         user_account_id: 32,
    //         consultant_id: 87,
    //         first_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 25, 7, 0, 0)
    //             .unwrap(),
    //         second_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
    //             .unwrap(),
    //         third_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 23, 21, 0, 0)
    //             .unwrap(),
    //         // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第二希望日時）とは異なるものを入れる。
    //         latest_candidate_date_time: current_date_time
    //             + Duration::seconds(
    //                 common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
    //             ),
    //         charge_id: "ch_ea990a4c10672a93053a774730b0b".to_string(),
    //         fee_per_hour_in_yen: 8000,
    //         platform_fee_rate_in_percentage: "30.0".to_string(),
    //         credit_facilities_expired_at: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 10, 17, 15, 0, 0)
    //             .unwrap(),
    //     };

    //     let mut map = HashMap::with_capacity(2);
    //     map.insert(settlement_id1, (settlement1, true));
    //     map.insert(settlement_id2, (settlement2, true));
    //     map
    // }

    // #[tokio::test]
    // async fn make_payment_of_unhandled_settlement_success6() {
    //     let current_date_time = JAPANESE_TIME_ZONE
    //         .with_ymd_and_hms(2023, 8, 27, 21, 8, 40)
    //         .unwrap();
    //     let max_num_of_target_records = 1;
    //     let op = MakePaymentOfUnhandledSettlementOperationMock {
    //         settlements: create_dummy_2_expired_settlements(current_date_time),
    //         current_date_time,
    //         limit: max_num_of_target_records,
    //     };
    //     // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
    //     let send_mail_mock =
    //         SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

    //     let result = make_payment_of_unhandled_settlement(
    //         current_date_time,
    //         max_num_of_target_records,
    //         &op,
    //         &send_mail_mock,
    //     )
    //     .await;

    //     let num_deleted = result.expect("failed to get Ok");
    //     assert_eq!(num_deleted, 1);
    // }

    // #[tokio::test]
    // async fn make_payment_of_unhandled_settlement_success7() {
    //     let current_date_time = JAPANESE_TIME_ZONE
    //         .with_ymd_and_hms(2023, 8, 27, 21, 8, 40)
    //         .unwrap();
    //     let max_num_of_target_records = 2;
    //     let op = MakePaymentOfUnhandledSettlementOperationMock {
    //         settlements: create_dummy_2_expired_settlements(current_date_time),
    //         current_date_time,
    //         limit: max_num_of_target_records,
    //     };
    //     // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
    //     let send_mail_mock =
    //         SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

    //     let result = make_payment_of_unhandled_settlement(
    //         current_date_time,
    //         max_num_of_target_records,
    //         &op,
    //         &send_mail_mock,
    //     )
    //     .await;

    //     let num_deleted = result.expect("failed to get Ok");
    //     assert_eq!(num_deleted, 2);
    // }

    // #[tokio::test]
    // async fn make_payment_of_unhandled_settlement_success8() {
    //     let current_date_time = JAPANESE_TIME_ZONE
    //         .with_ymd_and_hms(2023, 8, 27, 8, 0, 40)
    //         .unwrap();
    //     let max_num_of_target_records = 3;
    //     let op = MakePaymentOfUnhandledSettlementOperationMock {
    //         settlements: create_dummy_2_expired_settlements(current_date_time),
    //         current_date_time,
    //         limit: max_num_of_target_records,
    //     };
    //     // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
    //     let send_mail_mock =
    //         SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

    //     let result = make_payment_of_unhandled_settlement(
    //         current_date_time,
    //         max_num_of_target_records,
    //         &op,
    //         &send_mail_mock,
    //     )
    //     .await;

    //     let num_deleted = result.expect("failed to get Ok");
    //     assert_eq!(num_deleted, 2);
    // }

    // #[tokio::test]
    // async fn make_payment_of_unhandled_settlement_success9() {
    //     let current_date_time = JAPANESE_TIME_ZONE
    //         .with_ymd_and_hms(2023, 8, 27, 8, 0, 40)
    //         .unwrap();
    //     let max_num_of_target_records = 0;
    //     let op = MakePaymentOfUnhandledSettlementOperationMock {
    //         settlements: create_dummy_1_non_expired_and_1_expired_settlement(
    //             current_date_time,
    //         ),
    //         current_date_time,
    //         limit: max_num_of_target_records,
    //     };
    //     // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
    //     let send_mail_mock =
    //         SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

    //     let result = make_payment_of_unhandled_settlement(
    //         current_date_time,
    //         max_num_of_target_records,
    //         &op,
    //         &send_mail_mock,
    //     )
    //     .await;

    //     let num_deleted = result.expect("failed to get Ok");
    //     assert_eq!(num_deleted, 1);
    // }

    // fn create_dummy_1_non_expired_and_1_expired_settlement(
    //     current_date_time: DateTime<FixedOffset>,
    // ) -> HashMap<i64, (Settlement, bool)> {
    //     let settlement_id1 = 1234;
    //     let settlement1 = Settlement {
    //         settlement_id: settlement_id1,
    //         user_account_id: 456,
    //         consultant_id: 789,
    //         first_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 25, 13, 0, 0)
    //             .unwrap(),
    //         second_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 26, 14, 0, 0)
    //             .unwrap(),
    //         third_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
    //             .unwrap(),
    //         // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第三希望日時）とは異なるものを入れる。
    //         latest_candidate_date_time: current_date_time
    //             + Duration::seconds(
    //                 common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
    //             )
    //             + Duration::seconds(1),
    //         charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
    //         fee_per_hour_in_yen: 5000,
    //         platform_fee_rate_in_percentage: "30.0".to_string(),
    //         credit_facilities_expired_at: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 10, 19, 15, 0, 0)
    //             .unwrap(),
    //     };

    //     let settlement_id2 = 56;
    //     let settlement2 = Settlement {
    //         settlement_id: settlement_id2,
    //         user_account_id: 32,
    //         consultant_id: 87,
    //         first_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 25, 7, 0, 0)
    //             .unwrap(),
    //         second_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
    //             .unwrap(),
    //         third_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 23, 21, 0, 0)
    //             .unwrap(),
    //         // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第二希望日時）とは異なるものを入れる。
    //         latest_candidate_date_time: current_date_time
    //             + Duration::seconds(
    //                 common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
    //             ),
    //         charge_id: "ch_ea990a4c10672a93053a774730b0b".to_string(),
    //         fee_per_hour_in_yen: 8000,
    //         platform_fee_rate_in_percentage: "30.0".to_string(),
    //         credit_facilities_expired_at: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 10, 17, 15, 0, 0)
    //             .unwrap(),
    //     };

    //     let mut map = HashMap::with_capacity(2);
    //     map.insert(settlement_id1, (settlement1, true));
    //     map.insert(settlement_id2, (settlement2, true));
    //     map
    // }

    // #[tokio::test]
    // async fn make_payment_of_unhandled_settlement_success10() {
    //     let current_date_time = JAPANESE_TIME_ZONE
    //         .with_ymd_and_hms(2023, 8, 27, 8, 0, 40)
    //         .unwrap();
    //     let max_num_of_target_records = 1;
    //     let op = MakePaymentOfUnhandledSettlementOperationMock {
    //         settlements: create_dummy_1_non_expired_and_1_expired_settlement(
    //             current_date_time,
    //         ),
    //         current_date_time,
    //         limit: max_num_of_target_records,
    //     };
    //     // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
    //     let send_mail_mock =
    //         SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

    //     let result = make_payment_of_unhandled_settlement(
    //         current_date_time,
    //         max_num_of_target_records,
    //         &op,
    //         &send_mail_mock,
    //     )
    //     .await;

    //     let num_deleted = result.expect("failed to get Ok");
    //     assert_eq!(num_deleted, 1);
    // }

    // #[tokio::test]
    // async fn make_payment_of_unhandled_settlement_success11() {
    //     let current_date_time = JAPANESE_TIME_ZONE
    //         .with_ymd_and_hms(2023, 8, 27, 8, 0, 40)
    //         .unwrap();
    //     let max_num_of_target_records = 2;
    //     let op = MakePaymentOfUnhandledSettlementOperationMock {
    //         settlements: create_dummy_1_non_expired_and_1_expired_settlement(
    //             current_date_time,
    //         ),
    //         current_date_time,
    //         limit: max_num_of_target_records,
    //     };
    //     // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
    //     let send_mail_mock =
    //         SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

    //     let result = make_payment_of_unhandled_settlement(
    //         current_date_time,
    //         max_num_of_target_records,
    //         &op,
    //         &send_mail_mock,
    //     )
    //     .await;

    //     let num_deleted = result.expect("failed to get Ok");
    //     assert_eq!(num_deleted, 1);
    // }

    // #[tokio::test]
    // async fn make_payment_of_unhandled_settlement_fail1() {
    //     let current_date_time = JAPANESE_TIME_ZONE
    //         .with_ymd_and_hms(2023, 8, 27, 8, 0, 00)
    //         .unwrap();
    //     let max_num_of_target_records = 0;
    //     let op = MakePaymentOfUnhandledSettlementOperationMock {
    //         settlements: create_dummy_1_failed_expired_settlement(current_date_time),
    //         current_date_time,
    //         limit: max_num_of_target_records,
    //     };
    //     let send_mail_mock = SendMailMock::new(
    //         ADMIN_EMAIL_ADDRESS.to_string(),
    //         SYSTEM_EMAIL_ADDRESS.to_string(),
    //         format!(
    //             "[{}] 定期実行ツール (make_payment_of_unhandled_settlement) 失敗通知",
    //             WEB_SITE_NAME
    //         ),
    //         vec![
    //             "settlementの期限切れレコード1個の内、1個の削除に失敗しました。".to_string(),
    //             "1234".to_string(),
    //             "456".to_string(),
    //             "789".to_string(),
    //             "ch_fa990a4c10672a93053a774730b0a".to_string(),
    //             "2023-08-27T14:00:00+09:00".to_string(),
    //         ],
    //     );

    //     let result = make_payment_of_unhandled_settlement(
    //         current_date_time,
    //         max_num_of_target_records,
    //         &op,
    //         &send_mail_mock,
    //     )
    //     .await;

    //     let err = result.expect_err("failed to get Err");
    //     let err_message = err.to_string();
    //     assert!(err_message.contains("1 were processed, 1 were failed"));
    //     assert!(err_message.contains("1234"));
    //     assert!(err_message.contains("456"));
    //     assert!(err_message.contains("789"));
    //     assert!(err_message.contains("ch_fa990a4c10672a93053a774730b0a"));
    //     assert!(err_message.contains("2023-08-27T14:00:00+09:00"));
    // }

    // fn create_dummy_1_failed_expired_settlement(
    //     current_date_time: DateTime<FixedOffset>,
    // ) -> HashMap<i64, (Settlement, bool)> {
    //     let settlement_id = 1234;
    //     let settlement = Settlement {
    //         settlement_id,
    //         user_account_id: 456,
    //         consultant_id: 789,
    //         first_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 25, 13, 0, 0)
    //             .unwrap(),
    //         second_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 26, 14, 0, 0)
    //             .unwrap(),
    //         third_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
    //             .unwrap(),
    //         // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第三希望日時）とは異なるものを入れる。
    //         latest_candidate_date_time: current_date_time
    //             + Duration::seconds(
    //                 common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
    //             ),
    //         charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
    //         fee_per_hour_in_yen: 5000,
    //         platform_fee_rate_in_percentage: "30.0".to_string(),
    //         credit_facilities_expired_at: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 10, 19, 15, 0, 0)
    //             .unwrap(),
    //     };
    //     let mut map = HashMap::with_capacity(1);
    //     map.insert(settlement_id, (settlement, false));
    //     map
    // }

    // #[tokio::test]
    // async fn make_payment_of_unhandled_settlement_fail2() {
    //     let current_date_time = JAPANESE_TIME_ZONE
    //         .with_ymd_and_hms(2023, 8, 27, 8, 0, 00)
    //         .unwrap();
    //     let max_num_of_target_records = 0;
    //     let op = MakePaymentOfUnhandledSettlementOperationMock {
    //         settlements: create_dummy_2_failed_expired_settlements(current_date_time),
    //         current_date_time,
    //         limit: max_num_of_target_records,
    //     };
    //     let send_mail_mock = SendMailMock::new(
    //         ADMIN_EMAIL_ADDRESS.to_string(),
    //         SYSTEM_EMAIL_ADDRESS.to_string(),
    //         format!(
    //             "[{}] 定期実行ツール (make_payment_of_unhandled_settlement) 失敗通知",
    //             WEB_SITE_NAME
    //         ),
    //         vec![
    //             "settlementの期限切れレコード2個の内、2個の削除に失敗しました。".to_string(),
    //             "1234".to_string(),
    //             "456".to_string(),
    //             "789".to_string(),
    //             "ch_fa990a4c10672a93053a774730b0a".to_string(),
    //             "2023-08-27T14:00:00+09:00".to_string(),
    //             "56".to_string(),
    //             "32".to_string(),
    //             "87".to_string(),
    //             "ch_ea990a4c10672a93053a774730b0b".to_string(),
    //             "2023-08-27T14:00:00+09:00".to_string(),
    //         ],
    //     );

    //     let result = make_payment_of_unhandled_settlement(
    //         current_date_time,
    //         max_num_of_target_records,
    //         &op,
    //         &send_mail_mock,
    //     )
    //     .await;

    //     let err = result.expect_err("failed to get Err");
    //     let err_message = err.to_string();
    //     assert!(err_message.contains("2 were processed, 2 were failed"));

    //     assert!(err_message.contains("1234"));
    //     assert!(err_message.contains("456"));
    //     assert!(err_message.contains("789"));
    //     assert!(err_message.contains("ch_fa990a4c10672a93053a774730b0a"));
    //     assert!(err_message.contains("2023-08-27T14:00:00+09:00"));

    //     assert!(err_message.contains("56"));
    //     assert!(err_message.contains("32"));
    //     assert!(err_message.contains("87"));
    //     assert!(err_message.contains("ch_ea990a4c10672a93053a774730b0b"));
    //     assert!(err_message.contains("2023-08-27T14:00:00+09:00"));
    // }

    // fn create_dummy_2_failed_expired_settlements(
    //     current_date_time: DateTime<FixedOffset>,
    // ) -> HashMap<i64, (Settlement, bool)> {
    //     let settlement_id1 = 1234;
    //     let settlement1 = Settlement {
    //         settlement_id: settlement_id1,
    //         user_account_id: 456,
    //         consultant_id: 789,
    //         first_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 25, 13, 0, 0)
    //             .unwrap(),
    //         second_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 26, 14, 0, 0)
    //             .unwrap(),
    //         third_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
    //             .unwrap(),
    //         // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第三希望日時）とは異なるものを入れる。
    //         latest_candidate_date_time: current_date_time
    //             + Duration::seconds(
    //                 common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
    //             ),
    //         charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
    //         fee_per_hour_in_yen: 5000,
    //         platform_fee_rate_in_percentage: "30.0".to_string(),
    //         credit_facilities_expired_at: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 10, 19, 15, 0, 0)
    //             .unwrap(),
    //     };

    //     let settlement_id2 = 56;
    //     let settlement2 = Settlement {
    //         settlement_id: settlement_id2,
    //         user_account_id: 32,
    //         consultant_id: 87,
    //         first_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 25, 7, 0, 0)
    //             .unwrap(),
    //         second_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
    //             .unwrap(),
    //         third_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 23, 21, 0, 0)
    //             .unwrap(),
    //         // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第二希望日時）とは異なるものを入れる。
    //         latest_candidate_date_time: current_date_time
    //             + Duration::seconds(
    //                 common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
    //             ),
    //         charge_id: "ch_ea990a4c10672a93053a774730b0b".to_string(),
    //         fee_per_hour_in_yen: 8000,
    //         platform_fee_rate_in_percentage: "30.0".to_string(),
    //         credit_facilities_expired_at: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 10, 17, 15, 0, 0)
    //             .unwrap(),
    //     };

    //     let mut map = HashMap::with_capacity(2);
    //     map.insert(settlement_id1, (settlement1, false));
    //     map.insert(settlement_id2, (settlement2, false));
    //     map
    // }

    // #[tokio::test]
    // async fn make_payment_of_unhandled_settlement_fail3() {
    //     let current_date_time = JAPANESE_TIME_ZONE
    //         .with_ymd_and_hms(2023, 8, 27, 8, 0, 00)
    //         .unwrap();
    //     let max_num_of_target_records = 0;
    //     let op = MakePaymentOfUnhandledSettlementOperationMock {
    //         settlements:
    //             create_dummy_1_failed_expired_settlement_and_1_expired_settlement(
    //                 current_date_time,
    //             ),
    //         current_date_time,
    //         limit: max_num_of_target_records,
    //     };
    //     let send_mail_mock = SendMailMock::new(
    //         ADMIN_EMAIL_ADDRESS.to_string(),
    //         SYSTEM_EMAIL_ADDRESS.to_string(),
    //         format!(
    //             "[{}] 定期実行ツール (make_payment_of_unhandled_settlement) 失敗通知",
    //             WEB_SITE_NAME
    //         ),
    //         vec![
    //             "settlementの期限切れレコード2個の内、1個の削除に失敗しました。".to_string(),
    //             "56".to_string(),
    //             "32".to_string(),
    //             "87".to_string(),
    //             "ch_ea990a4c10672a93053a774730b0b".to_string(),
    //             "2023-08-27T14:00:00+09:00".to_string(),
    //         ],
    //     );

    //     let result = make_payment_of_unhandled_settlement(
    //         current_date_time,
    //         max_num_of_target_records,
    //         &op,
    //         &send_mail_mock,
    //     )
    //     .await;

    //     let err = result.expect_err("failed to get Err");
    //     let err_message = err.to_string();
    //     assert!(err_message.contains("2 were processed, 1 were failed"));

    //     assert!(!err_message.contains("1234"));
    //     assert!(!err_message.contains("456"));
    //     assert!(!err_message.contains("789"));
    //     assert!(!err_message.contains("ch_fa990a4c10672a93053a774730b0a"));
    //     assert!(!err_message.contains("2023-08-27 13:00:00 +09:00"));

    //     assert!(err_message.contains("56"));
    //     assert!(err_message.contains("32"));
    //     assert!(err_message.contains("87"));
    //     assert!(err_message.contains("ch_ea990a4c10672a93053a774730b0b"));
    //     assert!(err_message.contains("2023-08-27T14:00:00+09:00"));
    // }

    // fn create_dummy_1_failed_expired_settlement_and_1_expired_settlement(
    //     current_date_time: DateTime<FixedOffset>,
    // ) -> HashMap<i64, (Settlement, bool)> {
    //     let settlement_id1 = 1234;
    //     let settlement1 = Settlement {
    //         settlement_id: settlement_id1,
    //         user_account_id: 456,
    //         consultant_id: 789,
    //         first_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 25, 13, 0, 0)
    //             .unwrap(),
    //         second_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 26, 14, 0, 0)
    //             .unwrap(),
    //         third_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
    //             .unwrap(),
    //         // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第三希望日時）とは異なるものを入れる。
    //         latest_candidate_date_time: current_date_time
    //             + Duration::seconds(
    //                 common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
    //             )
    //             - Duration::hours(1),
    //         charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
    //         fee_per_hour_in_yen: 5000,
    //         platform_fee_rate_in_percentage: "30.0".to_string(),
    //         credit_facilities_expired_at: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 10, 19, 15, 0, 0)
    //             .unwrap(),
    //     };

    //     let settlement_id2 = 56;
    //     let settlement2 = Settlement {
    //         settlement_id: settlement_id2,
    //         user_account_id: 32,
    //         consultant_id: 87,
    //         first_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 25, 7, 0, 0)
    //             .unwrap(),
    //         second_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 27, 15, 0, 0)
    //             .unwrap(),
    //         third_candidate_date_time: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 8, 23, 21, 0, 0)
    //             .unwrap(),
    //         // latest_candidate_date_timeが削除するかどうかの基準となる。UTでは境界値のテストをしたいので実際の値（このケースでは第二希望日時）とは異なるものを入れる。
    //         latest_candidate_date_time: current_date_time
    //             + Duration::seconds(
    //                 common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
    //             ),
    //         charge_id: "ch_ea990a4c10672a93053a774730b0b".to_string(),
    //         fee_per_hour_in_yen: 8000,
    //         platform_fee_rate_in_percentage: "30.0".to_string(),
    //         credit_facilities_expired_at: JAPANESE_TIME_ZONE
    //             .with_ymd_and_hms(2023, 10, 17, 15, 0, 0)
    //             .unwrap(),
    //     };

    //     let mut map = HashMap::with_capacity(2);
    //     map.insert(settlement_id1, (settlement1, true));
    //     map.insert(settlement_id2, (settlement2, false));
    //     map
    // }
}
