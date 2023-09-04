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
        KEY_TO_AWS_SES_ENDPOINT_URI, KEY_TO_AWS_SES_REGION, KEY_TO_SYSTEM_EMAIL_ADDRESS,
        SYSTEM_EMAIL_ADDRESS,
    },
    util::check_env_vars,
    JAPANESE_TIME_ZONE, KEY_TO_USE_ECS_TASK_ROLE, USE_ECS_TASK_ROLE, WEB_SITE_NAME,
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
        KEY_TO_AWS_SES_ENDPOINT_URI.to_string(),
        KEY_TO_USE_ECS_TASK_ROLE.to_string(),
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
        "delete_expired_temp_mfa_secrets={},common={},sea_orm={}",
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
    let op = DeleteExpiredTempMfaSecretsOperationImpl { pool };

    let smtp_client = if *USE_ECS_TASK_ROLE {
        SmtpClient::new_with_ecs_task_role(AWS_SES_REGION.as_str(), AWS_SES_ENDPOINT_URI.as_str())
            .await
    } else {
        SmtpClient::new(
            AWS_SES_REGION.as_str(),
            AWS_SES_ACCESS_KEY_ID.as_str(),
            AWS_SES_SECRET_ACCESS_KEY.as_str(),
            AWS_SES_ENDPOINT_URI.as_str(),
        )
        .await
    };

    let result = delete_expired_temp_mfa_secrets(
        current_date_time,
        *NUM_OF_MAX_TARGET_RECORDS,
        &op,
        &smtp_client,
    )
    .await;

    let deleted_num = result.unwrap_or_else(|e| {
        error!("failed to delete expired temp mfa secrets: {}", e);
        exit(APPLICATION_ERR)
    });

    info!(
        "{} temp mfa secret(s) were (was) deleted successfully",
        deleted_num
    );
    exit(SUCCESS)
}

async fn delete_expired_temp_mfa_secrets(
    current_date_time: DateTime<FixedOffset>,
    num_of_max_target_records: u64,
    op: &impl DeleteExpiredTempMfaSecretsOperation,
    send_mail: &impl SendMail,
) -> Result<usize, Box<dyn Error>> {
    let limit = if num_of_max_target_records != 0 {
        Some(num_of_max_target_records)
    } else {
        None
    };

    let expired_temp_mfa_secrets = op
        .get_expired_temp_mfa_secrets(current_date_time, limit)
        .await?;
    let num_of_expired_temp_mfa_secrets = expired_temp_mfa_secrets.len();

    let mut delete_failed: Vec<TempMfaSecret> = Vec::with_capacity(expired_temp_mfa_secrets.len());
    for expired_temp_mfa_secret in expired_temp_mfa_secrets {
        let result = op
            .delete_temp_mfa_secret(expired_temp_mfa_secret.temp_mfa_secret_id)
            .await;
        if result.is_err() {
            error!("failed delete_temp_mfa_secret: {:?}", result);
            delete_failed.push(expired_temp_mfa_secret);
        }
    }

    if !delete_failed.is_empty() {
        let subject = format!(
            "[{}] 定期実行ツール (delete_expired_temp_mfa_secrets) 失敗通知",
            WEB_SITE_NAME
        );
        let num_of_delete_failed = delete_failed.len();
        let text = create_text(
            num_of_expired_temp_mfa_secrets,
            num_of_delete_failed,
            &delete_failed,
        );
        let err_message = format!(
            "{} processed, {} failed (detail: {:?})",
            num_of_expired_temp_mfa_secrets, num_of_delete_failed, delete_failed
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

    Ok(num_of_expired_temp_mfa_secrets)
}

#[async_trait]
trait DeleteExpiredTempMfaSecretsOperation {
    async fn get_expired_temp_mfa_secrets(
        &self,
        current_date_time: DateTime<FixedOffset>,
        limit: Option<u64>,
    ) -> Result<Vec<TempMfaSecret>, Box<dyn Error>>;

    async fn delete_temp_mfa_secret(&self, temp_mfa_secret_id: i64) -> Result<(), Box<dyn Error>>;
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct TempMfaSecret {
    temp_mfa_secret_id: i64,
    user_account_id: i64,
    expired_at: DateTime<FixedOffset>,
}

struct DeleteExpiredTempMfaSecretsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl DeleteExpiredTempMfaSecretsOperation for DeleteExpiredTempMfaSecretsOperationImpl {
    async fn get_expired_temp_mfa_secrets(
        &self,
        current_date_time: DateTime<FixedOffset>,
        limit: Option<u64>,
    ) -> Result<Vec<TempMfaSecret>, Box<dyn Error>> {
        let models = entity::temp_mfa_secret::Entity::find()
            .filter(entity::temp_mfa_secret::Column::ExpiredAt.lt(current_date_time))
            .limit(limit)
            .all(&self.pool)
            .await
            .map_err(|e| format!("failed to get temp_mfa_secret: {}", e))?;
        Ok(models
            .into_iter()
            .map(|m| TempMfaSecret {
                temp_mfa_secret_id: m.temp_mfa_secret_id,
                user_account_id: m.user_account_id,
                expired_at: m.expired_at,
            })
            .collect())
    }

    async fn delete_temp_mfa_secret(&self, temp_mfa_secret_id: i64) -> Result<(), Box<dyn Error>> {
        let _ = entity::temp_mfa_secret::Entity::delete_by_id(temp_mfa_secret_id)
            .exec(&self.pool)
            .await
            .map_err(|e| {
                format!(
                    "failed to delete temp_mfa_secret (temp_mfa_secret_id: {}): {}",
                    temp_mfa_secret_id, e
                )
            })?;
        Ok(())
    }
}

fn create_text(
    num_of_expired_temp_mfa_secrets: usize,
    num_of_delete_failed: usize,
    delete_failed: &[TempMfaSecret],
) -> String {
    format!(
        r"temp_mfa_secretの期限切れレコード{}個の内、{}個の削除に失敗しました。

【詳細】
{:?}",
        num_of_expired_temp_mfa_secrets, num_of_delete_failed, delete_failed
    )
}

#[cfg(test)]
mod tests {

    use std::{cmp::min, collections::HashMap};

    use chrono::{Duration, TimeZone};
    use common::ErrResp;

    use super::*;

    struct DeleteExpiredTempMfaSecretsOperationMock {
        temp_mfa_secrets: HashMap<i64, (TempMfaSecret, bool)>,
        current_date_time: DateTime<FixedOffset>,
        limit: u64,
    }

    #[async_trait]
    impl DeleteExpiredTempMfaSecretsOperation for DeleteExpiredTempMfaSecretsOperationMock {
        async fn get_expired_temp_mfa_secrets(
            &self,
            current_date_time: DateTime<FixedOffset>,
            limit: Option<u64>,
        ) -> Result<Vec<TempMfaSecret>, Box<dyn Error>> {
            assert_eq!(self.current_date_time, current_date_time);
            if self.limit != 0 {
                assert_eq!(Some(self.limit), limit);
            } else {
                assert_eq!(None, limit);
            }
            let expired_temp_mfa_secrets: Vec<TempMfaSecret> = self
                .temp_mfa_secrets
                .values()
                .clone()
                .filter(|m| m.0.expired_at < current_date_time)
                .map(|m| m.0.clone())
                .collect();
            let results = if let Some(limit) = limit {
                let limit = min(limit as usize, expired_temp_mfa_secrets.len());
                let mut expired_temp_mfa_secrets_limited = Vec::with_capacity(limit);
                (0..limit).for_each(|i| {
                    expired_temp_mfa_secrets_limited.push(expired_temp_mfa_secrets[i].clone())
                });
                expired_temp_mfa_secrets_limited
            } else {
                expired_temp_mfa_secrets
            };
            Ok(results)
        }

        async fn delete_temp_mfa_secret(
            &self,
            temp_mfa_secret_id: i64,
        ) -> Result<(), Box<dyn Error>> {
            let temp_mfa_secret = self
                .temp_mfa_secrets
                .get(&temp_mfa_secret_id)
                .expect("assert that temp_mfa_secret has value!");
            if !temp_mfa_secret.1 {
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
    async fn delete_expired_temp_mfa_secrets_success0() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredTempMfaSecretsOperationMock {
            temp_mfa_secrets: HashMap::with_capacity(0),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_mfa_secrets(
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
    async fn delete_expired_temp_mfa_secrets_success1() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredTempMfaSecretsOperationMock {
            temp_mfa_secrets: create_dummy_1_non_expired_temp_mfa_secret(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_mfa_secrets(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 0);
    }

    fn create_dummy_1_non_expired_temp_mfa_secret(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (TempMfaSecret, bool)> {
        let temp_mfa_secret_id = 1;
        let temp_mfa_secret = TempMfaSecret {
            temp_mfa_secret_id,
            user_account_id: 10,
            expired_at: current_date_time,
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(temp_mfa_secret_id, (temp_mfa_secret, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_temp_mfa_secrets_success2() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredTempMfaSecretsOperationMock {
            temp_mfa_secrets: create_dummy_1_expired_temp_mfa_secret(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_mfa_secrets(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 1);
    }

    fn create_dummy_1_expired_temp_mfa_secret(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (TempMfaSecret, bool)> {
        let temp_mfa_secret_id = 412;
        let temp_mfa_secret = TempMfaSecret {
            temp_mfa_secret_id,
            user_account_id: 7041,
            expired_at: current_date_time - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(temp_mfa_secret_id, (temp_mfa_secret, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_temp_mfa_secrets_success3() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 1;
        let op = DeleteExpiredTempMfaSecretsOperationMock {
            temp_mfa_secrets: create_dummy_1_expired_temp_mfa_secret(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_mfa_secrets(
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
    async fn delete_expired_temp_mfa_secrets_success4() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 2;
        let op = DeleteExpiredTempMfaSecretsOperationMock {
            temp_mfa_secrets: create_dummy_1_expired_temp_mfa_secret(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_mfa_secrets(
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
    async fn delete_expired_temp_mfa_secrets_success5() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredTempMfaSecretsOperationMock {
            temp_mfa_secrets: create_dummy_2_expired_temp_mfa_secrets(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_mfa_secrets(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 2);
    }

    fn create_dummy_2_expired_temp_mfa_secrets(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (TempMfaSecret, bool)> {
        let temp_mfa_secret_id1 = 55;
        let temp_mfa_secret1 = TempMfaSecret {
            temp_mfa_secret_id: temp_mfa_secret_id1,
            user_account_id: 702,
            expired_at: current_date_time - Duration::seconds(1),
        };
        let temp_mfa_secret_id2 = 777;
        let temp_mfa_secret2 = TempMfaSecret {
            temp_mfa_secret_id: temp_mfa_secret_id2,
            user_account_id: 90,
            expired_at: current_date_time - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(2);
        map.insert(temp_mfa_secret_id1, (temp_mfa_secret1, true));
        map.insert(temp_mfa_secret_id2, (temp_mfa_secret2, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_temp_mfa_secrets_success6() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 1;
        let op = DeleteExpiredTempMfaSecretsOperationMock {
            temp_mfa_secrets: create_dummy_2_expired_temp_mfa_secrets(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_mfa_secrets(
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
    async fn delete_expired_temp_mfa_secrets_success7() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 2;
        let op = DeleteExpiredTempMfaSecretsOperationMock {
            temp_mfa_secrets: create_dummy_2_expired_temp_mfa_secrets(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_mfa_secrets(
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
    async fn delete_expired_temp_mfa_secrets_success8() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 3;
        let op = DeleteExpiredTempMfaSecretsOperationMock {
            temp_mfa_secrets: create_dummy_2_expired_temp_mfa_secrets(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_mfa_secrets(
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
    async fn delete_expired_temp_mfa_secrets_success9() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredTempMfaSecretsOperationMock {
            temp_mfa_secrets: create_dummy_1_non_expired_and_1_expired_temp_mfa_secret(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_mfa_secrets(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 1);
    }

    fn create_dummy_1_non_expired_and_1_expired_temp_mfa_secret(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (TempMfaSecret, bool)> {
        let temp_mfa_secret_id1 = 1915;
        let temp_mfa_secret1 = TempMfaSecret {
            temp_mfa_secret_id: temp_mfa_secret_id1,
            user_account_id: 846,
            expired_at: current_date_time,
        };
        let temp_mfa_secret_id2 = 9999;
        let temp_mfa_secret2 = TempMfaSecret {
            temp_mfa_secret_id: temp_mfa_secret_id2,
            user_account_id: 1234,
            expired_at: current_date_time - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(2);
        map.insert(temp_mfa_secret_id1, (temp_mfa_secret1, true));
        map.insert(temp_mfa_secret_id2, (temp_mfa_secret2, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_temp_mfa_secrets_success10() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 1;
        let op = DeleteExpiredTempMfaSecretsOperationMock {
            temp_mfa_secrets: create_dummy_1_non_expired_and_1_expired_temp_mfa_secret(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_mfa_secrets(
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
    async fn delete_expired_temp_mfa_secrets_success11() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 2;
        let op = DeleteExpiredTempMfaSecretsOperationMock {
            temp_mfa_secrets: create_dummy_1_non_expired_and_1_expired_temp_mfa_secret(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_mfa_secrets(
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
    async fn delete_expired_temp_mfa_secrets_fail1() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredTempMfaSecretsOperationMock {
            temp_mfa_secrets: create_dummy_1_failed_expired_temp_mfa_secret(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            format!(
                "[{}] 定期実行ツール (delete_expired_temp_mfa_secrets) 失敗通知",
                WEB_SITE_NAME
            ),
            vec![
                "temp_mfa_secretの期限切れレコード1個の内、1個の削除に失敗しました。".to_string(),
                "734".to_string(),
            ],
        );

        let result = delete_expired_temp_mfa_secrets(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let err = result.expect_err("failed to get Err");
        let err_message = err.to_string();
        assert!(err_message.contains("1 processed, 1 failed"));
        assert!(err_message.contains("734"));
    }

    fn create_dummy_1_failed_expired_temp_mfa_secret(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (TempMfaSecret, bool)> {
        let temp_mfa_secret_id = 734;
        let temp_mfa_secret = TempMfaSecret {
            temp_mfa_secret_id,
            user_account_id: 231,
            expired_at: current_date_time - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(temp_mfa_secret_id, (temp_mfa_secret, false));
        map
    }

    #[tokio::test]
    async fn delete_expired_temp_mfa_secrets_fail2() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredTempMfaSecretsOperationMock {
            temp_mfa_secrets: create_dummy_2_failed_expired_temp_mfa_secrets(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            format!(
                "[{}] 定期実行ツール (delete_expired_temp_mfa_secrets) 失敗通知",
                WEB_SITE_NAME
            ),
            vec![
                "temp_mfa_secretの期限切れレコード2個の内、2個の削除に失敗しました。".to_string(),
                "45".to_string(),
                "567".to_string(),
            ],
        );

        let result = delete_expired_temp_mfa_secrets(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let err = result.expect_err("failed to get Err");
        let err_message = err.to_string();
        assert!(err_message.contains("2 processed, 2 failed"));
        assert!(err_message.contains("45"));
        assert!(err_message.contains("567"));
    }

    fn create_dummy_2_failed_expired_temp_mfa_secrets(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (TempMfaSecret, bool)> {
        let temp_mfa_secret_id1 = 45;
        let temp_mfa_secret1 = TempMfaSecret {
            temp_mfa_secret_id: temp_mfa_secret_id1,
            user_account_id: 478,
            expired_at: current_date_time - Duration::seconds(1),
        };
        let temp_mfa_secret_id2 = 567;
        let temp_mfa_secret2 = TempMfaSecret {
            temp_mfa_secret_id: temp_mfa_secret_id2,
            user_account_id: 111,
            expired_at: current_date_time - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(2);
        map.insert(temp_mfa_secret_id1, (temp_mfa_secret1, false));
        map.insert(temp_mfa_secret_id2, (temp_mfa_secret2, false));
        map
    }

    #[tokio::test]
    async fn delete_expired_temp_mfa_secrets_fail3() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredTempMfaSecretsOperationMock {
            temp_mfa_secrets:
                create_dummy_1_failed_expired_temp_mfa_secret_and_1_expired_temp_mfa_secret(
                    current_date_time,
                ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            format!(
                "[{}] 定期実行ツール (delete_expired_temp_mfa_secrets) 失敗通知",
                WEB_SITE_NAME
            ),
            vec![
                "temp_mfa_secretの期限切れレコード2個の内、1個の削除に失敗しました。".to_string(),
                "333".to_string(),
            ],
        );

        let result = delete_expired_temp_mfa_secrets(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let err = result.expect_err("failed to get Err");
        let err_message = err.to_string();
        assert!(err_message.contains("2 processed, 1 failed"));
        assert!(err_message.contains("333"));
        assert!(!err_message.contains("987"));
    }

    fn create_dummy_1_failed_expired_temp_mfa_secret_and_1_expired_temp_mfa_secret(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (TempMfaSecret, bool)> {
        let temp_mfa_secret_id1 = 333;
        let temp_mfa_secret1 = TempMfaSecret {
            temp_mfa_secret_id: temp_mfa_secret_id1,
            user_account_id: 455,
            expired_at: current_date_time - Duration::seconds(1),
        };
        let temp_mfa_secret_id2 = 987;
        let temp_mfa_secret2 = TempMfaSecret {
            temp_mfa_secret_id: temp_mfa_secret_id2,
            user_account_id: 387,
            expired_at: current_date_time - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(2);
        map.insert(temp_mfa_secret_id1, (temp_mfa_secret1, false));
        map.insert(temp_mfa_secret_id2, (temp_mfa_secret2, true));
        map
    }
}
