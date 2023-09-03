// Copyright 2023 Ken Miura

use chrono::{DateTime, FixedOffset};
use dotenv::dotenv;
use entity::sea_orm::{
    prelude::async_trait::async_trait, ColumnTrait, ConnectOptions, Database, DatabaseConnection,
    EntityTrait, QueryFilter, QuerySelect,
};
use std::{env::set_var, error::Error, process::exit};
use tracing::{error, info};

use common::{
    admin::{KEY_TO_DB_ADMIN_NAME, KEY_TO_DB_ADMIN_PASSWORD, NUM_OF_MAX_TARGET_RECORDS},
    db::{construct_db_url, KEY_TO_DB_HOST, KEY_TO_DB_NAME, KEY_TO_DB_PORT},
    log::{init_log, LOG_LEVEL},
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
    let log_conf = format!(
        "delete_expired_stopped_settlements={},common={},sea_orm={}",
        LOG_LEVEL.as_str(),
        LOG_LEVEL.as_str(),
        LOG_LEVEL.as_str()
    );
    set_var("RUST_LOG", log_conf);
    init_log();

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
        error!("failed to connect database: {}", e);
        exit(CONNECTION_ERROR)
    });
    let op = DeleteExpiredStoppedSettlementsOperationImpl { pool };

    let smtp_client = SmtpClient::new(
        AWS_SES_REGION.as_str(),
        AWS_SES_ACCESS_KEY_ID.as_str(),
        AWS_SES_SECRET_ACCESS_KEY.as_str(),
        AWS_SES_ENDPOINT_URI.as_str(),
    )
    .await;

    let result = delete_expired_stopped_settlements(
        current_date_time,
        *NUM_OF_MAX_TARGET_RECORDS,
        &op,
        &smtp_client,
    )
    .await;

    let deleted_num = result.unwrap_or_else(|e| {
        error!("failed to delete expired stopped settlements: {}", e);
        exit(APPLICATION_ERR)
    });

    info!(
        "{} stopped settlement(s) were (was) deleted successfully",
        deleted_num
    );
    exit(SUCCESS)
}

async fn delete_expired_stopped_settlements(
    current_date_time: DateTime<FixedOffset>,
    num_of_max_target_records: u64,
    op: &impl DeleteExpiredStoppedSettlementsOperation,
    send_mail: &impl SendMail,
) -> Result<usize, Box<dyn Error>> {
    let limit = if num_of_max_target_records != 0 {
        Some(num_of_max_target_records)
    } else {
        None
    };

    let expired_stopped_settlements = op
        .get_expired_stopped_settlements(current_date_time, limit)
        .await?;
    let num_of_expired_stopped_settlements = expired_stopped_settlements.len();

    let mut delete_failed: Vec<StoppedSettlement> =
        Vec::with_capacity(num_of_expired_stopped_settlements);
    for expired_stopped_settlement in expired_stopped_settlements {
        let result = op
            .delete_stopped_settlement(expired_stopped_settlement.stopped_settlement_id)
            .await;
        if result.is_err() {
            error!("failed delete_stopped_settlement: {:?}", result);
            delete_failed.push(expired_stopped_settlement);
        }
    }

    if !delete_failed.is_empty() {
        let subject = format!(
            "[{}] 定期実行ツール (delete_expired_stopped_settlements) 失敗通知",
            WEB_SITE_NAME
        );
        let num_of_delete_failed = delete_failed.len();
        let text = create_text(
            num_of_expired_stopped_settlements,
            num_of_delete_failed,
            &delete_failed,
        );
        let err_message = format!(
            "{} processed, {} failed (detail: {:?})",
            num_of_expired_stopped_settlements, num_of_delete_failed, delete_failed
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

    Ok(num_of_expired_stopped_settlements)
}

#[async_trait]
trait DeleteExpiredStoppedSettlementsOperation {
    async fn get_expired_stopped_settlements(
        &self,
        current_date_time: DateTime<FixedOffset>,
        limit: Option<u64>,
    ) -> Result<Vec<StoppedSettlement>, Box<dyn Error>>;

    async fn delete_stopped_settlement(
        &self,
        stopped_settlement_id: i64,
    ) -> Result<(), Box<dyn Error>>;
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct StoppedSettlement {
    stopped_settlement_id: i64,
    consultation_id: i64,
    charge_id: String,
    fee_per_hour_in_yen: i32,
    platform_fee_rate_in_percentage: String,
    credit_facilities_expired_at: DateTime<FixedOffset>,
    stopped_at: DateTime<FixedOffset>,
}

struct DeleteExpiredStoppedSettlementsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl DeleteExpiredStoppedSettlementsOperation for DeleteExpiredStoppedSettlementsOperationImpl {
    async fn get_expired_stopped_settlements(
        &self,
        current_date_time: DateTime<FixedOffset>,
        limit: Option<u64>,
    ) -> Result<Vec<StoppedSettlement>, Box<dyn Error>> {
        let models = entity::stopped_settlement::Entity::find()
            .filter(
                entity::stopped_settlement::Column::CreditFacilitiesExpiredAt.lt(current_date_time),
            )
            .limit(limit)
            .all(&self.pool)
            .await
            .map_err(|e| format!("failed to get stopped_settlement: {}", e))?;
        Ok(models
            .into_iter()
            .map(|m| StoppedSettlement {
                stopped_settlement_id: m.stopped_settlement_id,
                consultation_id: m.consultation_id,
                charge_id: m.charge_id,
                fee_per_hour_in_yen: m.fee_per_hour_in_yen,
                platform_fee_rate_in_percentage: m.platform_fee_rate_in_percentage,
                credit_facilities_expired_at: m.credit_facilities_expired_at,
                stopped_at: m.stopped_at,
            })
            .collect())
    }

    async fn delete_stopped_settlement(
        &self,
        stopped_settlement_id: i64,
    ) -> Result<(), Box<dyn Error>> {
        let _ = entity::stopped_settlement::Entity::delete_by_id(stopped_settlement_id)
            .exec(&self.pool)
            .await
            .map_err(|e| {
                format!(
                    "failed to delete stopped_settlement (stopped_settlement_id: {}): {}",
                    stopped_settlement_id, e
                )
            })?;
        Ok(())
    }
}

fn create_text(
    num_of_expired_stopped_settlements: usize,
    num_of_delete_failed: usize,
    delete_failed: &[StoppedSettlement],
) -> String {
    format!(
        r"stopped_settlementの期限切れレコード{}個の内、{}個の削除に失敗しました。

【詳細】
{:?}",
        num_of_expired_stopped_settlements, num_of_delete_failed, delete_failed
    )
}

#[cfg(test)]
mod tests {

    use std::{cmp::min, collections::HashMap};

    use chrono::{Duration, TimeZone};
    use common::ErrResp;

    use super::*;

    struct DeleteExpiredStoppedSettlementsOperationMock {
        stopped_settlements: HashMap<i64, (StoppedSettlement, bool)>,
        current_date_time: DateTime<FixedOffset>,
        limit: u64,
    }

    #[async_trait]
    impl DeleteExpiredStoppedSettlementsOperation for DeleteExpiredStoppedSettlementsOperationMock {
        async fn get_expired_stopped_settlements(
            &self,
            current_date_time: DateTime<FixedOffset>,
            limit: Option<u64>,
        ) -> Result<Vec<StoppedSettlement>, Box<dyn Error>> {
            assert_eq!(self.current_date_time, current_date_time);
            if self.limit != 0 {
                assert_eq!(Some(self.limit), limit);
            } else {
                assert_eq!(None, limit);
            }
            let expired_stopped_settlements: Vec<StoppedSettlement> = self
                .stopped_settlements
                .values()
                .clone()
                .filter(|m| m.0.credit_facilities_expired_at < current_date_time)
                .map(|m| m.0.clone())
                .collect();
            let results = if let Some(limit) = limit {
                let limit = min(limit as usize, expired_stopped_settlements.len());
                let mut expired_stopped_settlements_limited = Vec::with_capacity(limit);
                (0..limit).for_each(|i| {
                    expired_stopped_settlements_limited.push(expired_stopped_settlements[i].clone())
                });
                expired_stopped_settlements_limited
            } else {
                expired_stopped_settlements
            };
            Ok(results)
        }

        async fn delete_stopped_settlement(
            &self,
            stopped_settlement_id: i64,
        ) -> Result<(), Box<dyn Error>> {
            let stopped_settlement = self
                .stopped_settlements
                .get(&stopped_settlement_id)
                .expect("assert that stopped_settlement has value!");
            if !stopped_settlement.1 {
                return Err("mock error message".into());
            }
            Ok(())
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
    async fn delete_expired_stopped_settlements_success0() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredStoppedSettlementsOperationMock {
            stopped_settlements: HashMap::with_capacity(0),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_stopped_settlements(
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
    async fn delete_expired_stopped_settlements_success1() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredStoppedSettlementsOperationMock {
            stopped_settlements: create_dummy_1_non_expired_stopped_settlement(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_stopped_settlements(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 0);
    }

    fn create_dummy_1_non_expired_stopped_settlement(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (StoppedSettlement, bool)> {
        let stopped_settlement_id = 1;
        let stopped_settlement = StoppedSettlement {
            stopped_settlement_id,
            consultation_id: 123,
            charge_id: "ch_ea990a4c10672a93053a774730b0b".to_string(),
            fee_per_hour_in_yen: 4500,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: current_date_time,
            stopped_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 21, 0, 0)
                .unwrap(),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(stopped_settlement_id, (stopped_settlement, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_stopped_settlements_success2() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredStoppedSettlementsOperationMock {
            stopped_settlements: create_dummy_1_expired_stopped_settlement(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_stopped_settlements(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 1);
    }

    fn create_dummy_1_expired_stopped_settlement(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (StoppedSettlement, bool)> {
        let stopped_settlement_id = 1;
        let stopped_settlement = StoppedSettlement {
            stopped_settlement_id,
            consultation_id: 123,
            charge_id: "ch_ea990a4c10672a93053a774730b0b".to_string(),
            fee_per_hour_in_yen: 4500,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: current_date_time - Duration::seconds(1),
            stopped_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 21, 0, 0)
                .unwrap(),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(stopped_settlement_id, (stopped_settlement, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_stopped_settlements_success3() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 1;
        let op = DeleteExpiredStoppedSettlementsOperationMock {
            stopped_settlements: create_dummy_1_expired_stopped_settlement(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_stopped_settlements(
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
    async fn delete_expired_stopped_settlements_success4() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 2;
        let op = DeleteExpiredStoppedSettlementsOperationMock {
            stopped_settlements: create_dummy_1_expired_stopped_settlement(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_stopped_settlements(
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
    async fn delete_expired_stopped_settlements_success5() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredStoppedSettlementsOperationMock {
            stopped_settlements: create_dummy_2_expired_stopped_settlements(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_stopped_settlements(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 2);
    }

    fn create_dummy_2_expired_stopped_settlements(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (StoppedSettlement, bool)> {
        let stopped_settlement_id1 = 233;
        let stopped_settlement1 = StoppedSettlement {
            stopped_settlement_id: stopped_settlement_id1,
            consultation_id: 123,
            charge_id: "ch_ea990a4c10672a93053a774730b0b".to_string(),
            fee_per_hour_in_yen: 4500,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: current_date_time - Duration::seconds(1),
            stopped_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 21, 0, 0)
                .unwrap(),
        };

        let stopped_settlement_id2 = 77;
        let stopped_settlement2 = StoppedSettlement {
            stopped_settlement_id: stopped_settlement_id2,
            consultation_id: 98,
            charge_id: "ch_ea990a4c10672a93053a774730b0b".to_string(),
            fee_per_hour_in_yen: 3000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: current_date_time - Duration::seconds(1),
            stopped_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 21, 0, 0)
                .unwrap(),
        };

        let mut map = HashMap::with_capacity(2);
        map.insert(stopped_settlement_id1, (stopped_settlement1, true));
        map.insert(stopped_settlement_id2, (stopped_settlement2, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_stopped_settlements_success6() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 1;
        let op = DeleteExpiredStoppedSettlementsOperationMock {
            stopped_settlements: create_dummy_2_expired_stopped_settlements(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_stopped_settlements(
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
    async fn delete_expired_stopped_settlements_success7() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 2;
        let op = DeleteExpiredStoppedSettlementsOperationMock {
            stopped_settlements: create_dummy_2_expired_stopped_settlements(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_stopped_settlements(
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
    async fn delete_expired_stopped_settlements_success8() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 3;
        let op = DeleteExpiredStoppedSettlementsOperationMock {
            stopped_settlements: create_dummy_2_expired_stopped_settlements(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_stopped_settlements(
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
    async fn delete_expired_stopped_settlements_success9() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredStoppedSettlementsOperationMock {
            stopped_settlements: create_dummy_1_non_expired_and_1_expired_stopped_settlement(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_stopped_settlements(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 1);
    }

    fn create_dummy_1_non_expired_and_1_expired_stopped_settlement(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (StoppedSettlement, bool)> {
        let stopped_settlement_id1 = 233;
        let stopped_settlement1 = StoppedSettlement {
            stopped_settlement_id: stopped_settlement_id1,
            consultation_id: 123,
            charge_id: "ch_ea990a4c10672a93053a774730b0b".to_string(),
            fee_per_hour_in_yen: 4500,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: current_date_time,
            stopped_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 21, 0, 0)
                .unwrap(),
        };

        let stopped_settlement_id2 = 77;
        let stopped_settlement2 = StoppedSettlement {
            stopped_settlement_id: stopped_settlement_id2,
            consultation_id: 98,
            charge_id: "ch_ea990a4c10672a93053a774730b0b".to_string(),
            fee_per_hour_in_yen: 3000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: current_date_time - Duration::seconds(1),
            stopped_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 21, 0, 0)
                .unwrap(),
        };

        let mut map = HashMap::with_capacity(2);
        map.insert(stopped_settlement_id1, (stopped_settlement1, true));
        map.insert(stopped_settlement_id2, (stopped_settlement2, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_stopped_settlements_success10() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 1;
        let op = DeleteExpiredStoppedSettlementsOperationMock {
            stopped_settlements: create_dummy_1_non_expired_and_1_expired_stopped_settlement(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_stopped_settlements(
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
    async fn delete_expired_stopped_settlements_success11() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 2;
        let op = DeleteExpiredStoppedSettlementsOperationMock {
            stopped_settlements: create_dummy_1_non_expired_and_1_expired_stopped_settlement(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_stopped_settlements(
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
    async fn delete_expired_stopped_settlements_fail1() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredStoppedSettlementsOperationMock {
            stopped_settlements: create_dummy_1_failed_expired_stopped_settlement(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            format!(
                "[{}] 定期実行ツール (delete_expired_stopped_settlements) 失敗通知",
                WEB_SITE_NAME
            ),
            vec![
                "stopped_settlementの期限切れレコード1個の内、1個の削除に失敗しました。"
                    .to_string(),
                "111".to_string(),
                "123".to_string(),
                "ch_ea990a4c10672a93053a774730b0b".to_string(),
                "4500".to_string(),
                "30.0".to_string(),
                "2023-08-05T21:00:39+09:00".to_string(),
                "2023-06-23T21:00:00+09:00".to_string(),
            ],
        );

        let result = delete_expired_stopped_settlements(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let err = result.expect_err("failed to get Err");
        let err_message = err.to_string();
        assert!(err_message.contains("1 processed, 1 failed"));
        assert!(err_message.contains("111"));
        assert!(err_message.contains("123"));
        assert!(err_message.contains("ch_ea990a4c10672a93053a774730b0b"));
        assert!(err_message.contains("4500"));
        assert!(err_message.contains("30.0"));
        assert!(err_message.contains("2023-08-05T21:00:39+09:00"));
        assert!(err_message.contains("2023-06-23T21:00:00+09:00"));
    }

    fn create_dummy_1_failed_expired_stopped_settlement(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (StoppedSettlement, bool)> {
        let stopped_settlement_id = 111;
        let stopped_settlement = StoppedSettlement {
            stopped_settlement_id,
            consultation_id: 123,
            charge_id: "ch_ea990a4c10672a93053a774730b0b".to_string(),
            fee_per_hour_in_yen: 4500,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: current_date_time - Duration::seconds(1),
            stopped_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 21, 0, 0)
                .unwrap(),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(stopped_settlement_id, (stopped_settlement, false));
        map
    }

    #[tokio::test]
    async fn delete_expired_stopped_settlements_fail2() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredStoppedSettlementsOperationMock {
            stopped_settlements: create_dummy_2_failed_expired_stopped_settlements(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            format!(
                "[{}] 定期実行ツール (delete_expired_stopped_settlements) 失敗通知",
                WEB_SITE_NAME
            ),
            vec![
                "stopped_settlementの期限切れレコード2個の内、2個の削除に失敗しました。"
                    .to_string(),
                "111".to_string(),
                "123".to_string(),
                "ch_ea990a4c10672a93053a774730b0b".to_string(),
                "4500".to_string(),
                "30.0".to_string(),
                "2023-08-05T21:00:39+09:00".to_string(),
                "2023-06-23T21:00:00+09:00".to_string(),
                "222".to_string(),
                "446".to_string(),
                "ch_fa990a4c10672a93053a774730b0c".to_string(),
                "5500".to_string(),
                "30.0".to_string(),
                "2023-08-05T21:00:39+09:00".to_string(),
                "2023-06-23T21:00:00+09:00".to_string(),
            ],
        );

        let result = delete_expired_stopped_settlements(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let err = result.expect_err("failed to get Err");
        let err_message = err.to_string();
        assert!(err_message.contains("2 processed, 2 failed"));

        assert!(err_message.contains("111"));
        assert!(err_message.contains("123"));
        assert!(err_message.contains("ch_ea990a4c10672a93053a774730b0b"));
        assert!(err_message.contains("4500"));
        assert!(err_message.contains("30.0"));
        assert!(err_message.contains("2023-08-05T21:00:39+09:00"));
        assert!(err_message.contains("2023-06-23T21:00:00+09:00"));

        assert!(err_message.contains("222"));
        assert!(err_message.contains("446"));
        assert!(err_message.contains("ch_fa990a4c10672a93053a774730b0c"));
        assert!(err_message.contains("5500"));
        assert!(err_message.contains("30.0"));
        assert!(err_message.contains("2023-08-05T21:00:39+09:00"));
        assert!(err_message.contains("2023-06-23T21:00:00+09:00"));
    }

    fn create_dummy_2_failed_expired_stopped_settlements(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (StoppedSettlement, bool)> {
        let stopped_settlement_id1 = 111;
        let stopped_settlement1 = StoppedSettlement {
            stopped_settlement_id: stopped_settlement_id1,
            consultation_id: 123,
            charge_id: "ch_ea990a4c10672a93053a774730b0b".to_string(),
            fee_per_hour_in_yen: 4500,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: current_date_time - Duration::seconds(1),
            stopped_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 21, 0, 0)
                .unwrap(),
        };

        let stopped_settlement_id2 = 222;
        let stopped_settlement2 = StoppedSettlement {
            stopped_settlement_id: stopped_settlement_id2,
            consultation_id: 446,
            charge_id: "ch_fa990a4c10672a93053a774730b0c".to_string(),
            fee_per_hour_in_yen: 5500,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: current_date_time - Duration::seconds(1),
            stopped_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 21, 0, 0)
                .unwrap(),
        };

        let mut map = HashMap::with_capacity(2);
        map.insert(stopped_settlement_id1, (stopped_settlement1, false));
        map.insert(stopped_settlement_id2, (stopped_settlement2, false));
        map
    }

    #[tokio::test]
    async fn delete_expired_stopped_settlements_fail3() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredStoppedSettlementsOperationMock {
            stopped_settlements:
                create_dummy_1_failed_expired_stopped_settlement_and_1_expired_stopped_settlement(
                    current_date_time,
                ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            format!(
                "[{}] 定期実行ツール (delete_expired_stopped_settlements) 失敗通知",
                WEB_SITE_NAME
            ),
            vec![
                "stopped_settlementの期限切れレコード2個の内、1個の削除に失敗しました。"
                    .to_string(),
                "111".to_string(),
                "123".to_string(),
                "ch_ea990a4c10672a93053a774730b0b".to_string(),
                "4500".to_string(),
                "30.0".to_string(),
                "2023-08-05T21:00:39+09:00".to_string(),
                "2023-06-23T21:00:00+09:00".to_string(),
            ],
        );

        let result = delete_expired_stopped_settlements(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let err = result.expect_err("failed to get Err");
        let err_message = err.to_string();
        assert!(err_message.contains("2 processed, 1 failed"));

        assert!(err_message.contains("111"));
        assert!(err_message.contains("123"));
        assert!(err_message.contains("ch_ea990a4c10672a93053a774730b0b"));
        assert!(err_message.contains("4500"));
        assert!(err_message.contains("30.0"));
        assert!(err_message.contains("2023-08-05T21:00:39+09:00"));
        assert!(err_message.contains("2023-06-23T21:00:00+09:00"));

        assert!(!err_message.contains("222"));
        assert!(!err_message.contains("446"));
        assert!(!err_message.contains("ch_fa990a4c10672a93053a774730b0c"));
        assert!(!err_message.contains("5500"));
    }

    fn create_dummy_1_failed_expired_stopped_settlement_and_1_expired_stopped_settlement(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (StoppedSettlement, bool)> {
        let stopped_settlement_id1 = 111;
        let stopped_settlement1 = StoppedSettlement {
            stopped_settlement_id: stopped_settlement_id1,
            consultation_id: 123,
            charge_id: "ch_ea990a4c10672a93053a774730b0b".to_string(),
            fee_per_hour_in_yen: 4500,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: current_date_time - Duration::seconds(1),
            stopped_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 21, 0, 0)
                .unwrap(),
        };

        let stopped_settlement_id2 = 222;
        let stopped_settlement2 = StoppedSettlement {
            stopped_settlement_id: stopped_settlement_id2,
            consultation_id: 446,
            charge_id: "ch_fa990a4c10672a93053a774730b0c".to_string(),
            fee_per_hour_in_yen: 5500,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: current_date_time - Duration::seconds(1),
            stopped_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 21, 0, 0)
                .unwrap(),
        };

        let mut map = HashMap::with_capacity(2);
        map.insert(stopped_settlement_id1, (stopped_settlement1, false));
        map.insert(stopped_settlement_id2, (stopped_settlement2, true));
        map
    }
}
