// Copyright 2023 Ken Miura

use chrono::{DateTime, Duration, FixedOffset};
use dotenv::dotenv;
use entity::sea_orm::{
    prelude::async_trait::async_trait, ColumnTrait, ConnectOptions, Database, DatabaseConnection,
    EntityTrait, QueryFilter, QuerySelect,
};
use std::{error::Error, process::exit};

use common::{
    admin::{KEY_TO_DB_ADMIN_NAME, KEY_TO_DB_ADMIN_PASSWORD, NUM_OF_MAX_TARGET_RECORDS},
    db::{construct_db_url, KEY_TO_DB_HOST, KEY_TO_DB_NAME, KEY_TO_DB_PORT},
    smtp::{
        SendMail, SmtpClient, ADMIN_EMAIL_ADDRESS, AWS_SES_ACCESS_KEY_ID, AWS_SES_ENDPOINT_URI,
        AWS_SES_REGION, AWS_SES_SECRET_ACCESS_KEY, KEY_TO_ADMIN_EMAIL_ADDRESS,
        KEY_TO_AWS_SES_ACCESS_KEY_ID, KEY_TO_AWS_SES_ENDPOINT_URI, KEY_TO_AWS_SES_REGION,
        KEY_TO_AWS_SES_SECRET_ACCESS_KEY, KEY_TO_SYSTEM_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS,
    },
    util::check_env_vars,
    JAPANESE_TIME_ZONE, VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR, WEB_SITE_NAME,
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
    let op = DeleteExpiredTempAccountsOperationImpl { pool };

    let smtp_client = SmtpClient::new(
        AWS_SES_REGION.as_str(),
        AWS_SES_ACCESS_KEY_ID.as_str(),
        AWS_SES_SECRET_ACCESS_KEY.as_str(),
        AWS_SES_ENDPOINT_URI.as_str(),
    )
    .await;

    let result = delete_expired_temp_accounts(
        current_date_time,
        *NUM_OF_MAX_TARGET_RECORDS,
        &op,
        &smtp_client,
    )
    .await;

    let deleted_num = result.unwrap_or_else(|e| {
        println!("failed to delete expired temp accounts: {}", e);
        exit(APPLICATION_ERR)
    });

    println!("{} temp accounts were deleted successfully", deleted_num);
    exit(SUCCESS)
}

async fn delete_expired_temp_accounts(
    current_date_time: DateTime<FixedOffset>,
    num_of_max_target_records: u64,
    op: &impl DeleteExpiredTempAccountsOperation,
    send_mail: &impl SendMail,
) -> Result<usize, Box<dyn Error>> {
    let criteria = current_date_time - Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR);
    let limit = if num_of_max_target_records != 0 {
        Some(num_of_max_target_records)
    } else {
        None
    };

    let expired_temp_accounts = op.get_expired_temp_accounts(criteria, limit).await?;
    let num_of_expired_temp_accounts = expired_temp_accounts.len();

    let mut delete_failed: Vec<TempAccount> = Vec::with_capacity(expired_temp_accounts.len());
    for expired_temp_account in expired_temp_accounts {
        let result = op
            .delete_temp_account(&expired_temp_account.temp_account_id)
            .await;
        if result.is_err() {
            println!("failed delete_temp_account: {:?}", result);
            delete_failed.push(expired_temp_account);
        }
    }

    if !delete_failed.is_empty() {
        let subject = format!(
            "[{}] 定期実行ツール (delete_expired_temp_accounts) 失敗通知",
            WEB_SITE_NAME
        );
        let num_of_delete_failed = delete_failed.len();
        let text = create_text(
            num_of_expired_temp_accounts,
            num_of_delete_failed,
            &delete_failed,
        );
        let err_message = format!(
            "{} were processed, {} were failed (detail: {:?})",
            num_of_expired_temp_accounts, num_of_delete_failed, delete_failed
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

    Ok(num_of_expired_temp_accounts)
}

#[async_trait]
trait DeleteExpiredTempAccountsOperation {
    async fn get_expired_temp_accounts(
        &self,
        criteria: DateTime<FixedOffset>,
        limit: Option<u64>,
    ) -> Result<Vec<TempAccount>, Box<dyn Error>>;

    async fn delete_temp_account(&self, temp_account_id: &str) -> Result<(), Box<dyn Error>>;
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct TempAccount {
    temp_account_id: String,
    email_address: String,
    created_at: DateTime<FixedOffset>,
}

struct DeleteExpiredTempAccountsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl DeleteExpiredTempAccountsOperation for DeleteExpiredTempAccountsOperationImpl {
    async fn get_expired_temp_accounts(
        &self,
        criteria: DateTime<FixedOffset>,
        limit: Option<u64>,
    ) -> Result<Vec<TempAccount>, Box<dyn Error>> {
        let models = entity::user_temp_account::Entity::find()
            .filter(entity::user_temp_account::Column::CreatedAt.lt(criteria))
            .limit(limit)
            .all(&self.pool)
            .await
            .map_err(|e| format!("failed to get user_temp_account: {}", e))?;
        Ok(models
            .into_iter()
            .map(|m| TempAccount {
                temp_account_id: m.user_temp_account_id,
                email_address: m.email_address,
                created_at: m.created_at,
            })
            .collect())
    }

    async fn delete_temp_account(&self, temp_account_id: &str) -> Result<(), Box<dyn Error>> {
        let _ = entity::user_temp_account::Entity::delete_by_id(temp_account_id)
            .exec(&self.pool)
            .await
            .map_err(|e| {
                format!(
                    "failed to delete user_temp_account (temp_account_id: {}): {}",
                    temp_account_id, e
                )
            })?;
        Ok(())
    }
}

fn create_text(
    num_of_expired_temp_accounts: usize,
    num_of_delete_failed: usize,
    delete_failed: &[TempAccount],
) -> String {
    format!(
        r"user_temp_accountの期限切れレコード{}個の内、{}個の削除に失敗しました。

【詳細】
{:?}",
        num_of_expired_temp_accounts, num_of_delete_failed, delete_failed
    )
}

#[cfg(test)]
mod tests {

    use std::{cmp::min, collections::HashMap};

    use chrono::TimeZone;
    use common::ErrResp;

    use super::*;

    struct DeleteExpiredTempAccountsOperationMock {
        temp_accounts: HashMap<String, (TempAccount, bool)>,
        current_date_time: DateTime<FixedOffset>,
        limit: u64,
    }

    #[async_trait]
    impl DeleteExpiredTempAccountsOperation for DeleteExpiredTempAccountsOperationMock {
        async fn get_expired_temp_accounts(
            &self,
            criteria: DateTime<FixedOffset>,
            limit: Option<u64>,
        ) -> Result<Vec<TempAccount>, Box<dyn Error>> {
            assert_eq!(
                self.current_date_time - Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR),
                criteria
            );
            if self.limit != 0 {
                assert_eq!(Some(self.limit), limit);
            } else {
                assert_eq!(None, limit);
            }
            let expired_temp_accounts: Vec<TempAccount> = self
                .temp_accounts
                .values()
                .clone()
                .filter(|m| m.0.created_at < criteria)
                .map(|m| m.0.clone())
                .collect();
            let results = if let Some(limit) = limit {
                let limit = min(limit as usize, expired_temp_accounts.len());
                let mut expired_temp_accounts_limited = Vec::with_capacity(limit);
                (0..limit).for_each(|i| {
                    expired_temp_accounts_limited.push(expired_temp_accounts[i].clone())
                });
                expired_temp_accounts_limited
            } else {
                expired_temp_accounts
            };
            Ok(results)
        }

        async fn delete_temp_account(&self, temp_account_id: &str) -> Result<(), Box<dyn Error>> {
            let temp_account = self
                .temp_accounts
                .get(temp_account_id)
                .expect("assert that temp_account has value!");
            if !temp_account.1 {
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
    async fn delete_expired_temp_accounts_success0() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredTempAccountsOperationMock {
            temp_accounts: HashMap::with_capacity(0),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_accounts(
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
    async fn delete_expired_temp_accounts_success1() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredTempAccountsOperationMock {
            temp_accounts: create_dummy_1_non_expired_temp_account(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_accounts(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 0);
    }

    fn create_dummy_1_non_expired_temp_account(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<String, (TempAccount, bool)> {
        let temp_account_id = "b860dc5138d146ac8127b0780fabce7d";
        let temp_account = TempAccount {
            temp_account_id: temp_account_id.to_string(),
            email_address: "test1@test.com".to_string(),
            created_at: current_date_time - Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(temp_account_id.to_string(), (temp_account, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_temp_accounts_success2() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredTempAccountsOperationMock {
            temp_accounts: create_dummy_1_expired_temp_account(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_accounts(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 1);
    }

    fn create_dummy_1_expired_temp_account(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<String, (TempAccount, bool)> {
        let temp_account_id = "b860dc5138d146ac8127b0780fabce7d";
        let temp_account = TempAccount {
            temp_account_id: temp_account_id.to_string(),
            email_address: "test1@test.com".to_string(),
            created_at: current_date_time
                - Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR)
                - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(temp_account_id.to_string(), (temp_account, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_temp_accounts_success3() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 1;
        let op = DeleteExpiredTempAccountsOperationMock {
            temp_accounts: create_dummy_1_expired_temp_account(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_accounts(
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
    async fn delete_expired_temp_accounts_success4() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 2;
        let op = DeleteExpiredTempAccountsOperationMock {
            temp_accounts: create_dummy_1_expired_temp_account(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_accounts(
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
    async fn delete_expired_temp_accounts_success5() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredTempAccountsOperationMock {
            temp_accounts: create_dummy_2_expired_temp_accounts(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_accounts(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 2);
    }

    fn create_dummy_2_expired_temp_accounts(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<String, (TempAccount, bool)> {
        let temp_account_id1 = "b860dc5138d146ac8127b0780fabce7d";
        let temp_account1 = TempAccount {
            temp_account_id: temp_account_id1.to_string(),
            email_address: "test1@test.com".to_string(),
            created_at: current_date_time
                - Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR)
                - Duration::seconds(1),
        };
        let temp_account_id2 = "c860dc5138d146ac8127b0780fabce7e";
        let temp_account2 = TempAccount {
            temp_account_id: temp_account_id2.to_string(),
            email_address: "test2@test.com".to_string(),
            created_at: current_date_time
                - Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR)
                - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(2);
        map.insert(temp_account_id1.to_string(), (temp_account1, true));
        map.insert(temp_account_id2.to_string(), (temp_account2, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_temp_accounts_success6() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 1;
        let op = DeleteExpiredTempAccountsOperationMock {
            temp_accounts: create_dummy_2_expired_temp_accounts(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_accounts(
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
    async fn delete_expired_temp_accounts_success7() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 2;
        let op = DeleteExpiredTempAccountsOperationMock {
            temp_accounts: create_dummy_2_expired_temp_accounts(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_accounts(
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
    async fn delete_expired_temp_accounts_success8() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 3;
        let op = DeleteExpiredTempAccountsOperationMock {
            temp_accounts: create_dummy_2_expired_temp_accounts(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_accounts(
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
    async fn delete_expired_temp_accounts_success9() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredTempAccountsOperationMock {
            temp_accounts: create_dummy_1_non_expired_and_1_expired_temp_account(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_accounts(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 1);
    }

    fn create_dummy_1_non_expired_and_1_expired_temp_account(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<String, (TempAccount, bool)> {
        let temp_account_id1 = "b860dc5138d146ac8127b0780fabce7d";
        let temp_account1 = TempAccount {
            temp_account_id: temp_account_id1.to_string(),
            email_address: "test1@test.com".to_string(),
            created_at: current_date_time - Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR),
        };
        let temp_account_id2 = "c860dc5138d146ac8127b0780fabce7e";
        let temp_account2 = TempAccount {
            temp_account_id: temp_account_id2.to_string(),
            email_address: "test2@test.com".to_string(),
            created_at: current_date_time
                - Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR)
                - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(2);
        map.insert(temp_account_id1.to_string(), (temp_account1, true));
        map.insert(temp_account_id2.to_string(), (temp_account2, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_temp_accounts_success10() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 1;
        let op = DeleteExpiredTempAccountsOperationMock {
            temp_accounts: create_dummy_1_non_expired_and_1_expired_temp_account(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_accounts(
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
    async fn delete_expired_temp_accounts_success11() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 2;
        let op = DeleteExpiredTempAccountsOperationMock {
            temp_accounts: create_dummy_1_non_expired_and_1_expired_temp_account(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_temp_accounts(
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
    async fn delete_expired_temp_accounts_fail1() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredTempAccountsOperationMock {
            temp_accounts: create_dummy_1_failed_expired_temp_account(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            format!(
                "[{}] 定期実行ツール (delete_expired_temp_accounts) 失敗通知",
                WEB_SITE_NAME
            ),
            vec![
                "user_temp_accountの期限切れレコード1個の内、1個の削除に失敗しました。".to_string(),
                "b860dc5138d146ac8127b0780fabce7d".to_string(),
            ],
        );

        let result = delete_expired_temp_accounts(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let err = result.expect_err("failed to get Err");
        let err_message = err.to_string();
        assert!(err_message.contains("1 were processed, 1 were failed"));
        assert!(err_message.contains("b860dc5138d146ac8127b0780fabce7d"));
    }

    fn create_dummy_1_failed_expired_temp_account(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<String, (TempAccount, bool)> {
        let temp_account_id = "b860dc5138d146ac8127b0780fabce7d";
        let temp_account = TempAccount {
            temp_account_id: temp_account_id.to_string(),
            email_address: "test1@test.com".to_string(),
            created_at: current_date_time
                - Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR)
                - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(temp_account_id.to_string(), (temp_account, false));
        map
    }

    #[tokio::test]
    async fn delete_expired_temp_accounts_fail2() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredTempAccountsOperationMock {
            temp_accounts: create_dummy_2_failed_expired_temp_accounts(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            format!(
                "[{}] 定期実行ツール (delete_expired_temp_accounts) 失敗通知",
                WEB_SITE_NAME
            ),
            vec![
                "user_temp_accountの期限切れレコード2個の内、2個の削除に失敗しました。".to_string(),
                "b860dc5138d146ac8127b0780fabce7d".to_string(),
                "a860dc5138d146ac8127b0780fbbce7g".to_string(),
            ],
        );

        let result = delete_expired_temp_accounts(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let err = result.expect_err("failed to get Err");
        let err_message = err.to_string();
        assert!(err_message.contains("2 were processed, 2 were failed"));
        assert!(err_message.contains("b860dc5138d146ac8127b0780fabce7d"));
        assert!(err_message.contains("a860dc5138d146ac8127b0780fbbce7g"));
    }

    fn create_dummy_2_failed_expired_temp_accounts(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<String, (TempAccount, bool)> {
        let temp_account_id1 = "b860dc5138d146ac8127b0780fabce7d";
        let temp_account1 = TempAccount {
            temp_account_id: temp_account_id1.to_string(),
            email_address: "test1@test.com".to_string(),
            created_at: current_date_time
                - Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR)
                - Duration::seconds(1),
        };
        let temp_account_id2 = "a860dc5138d146ac8127b0780fbbce7g";
        let temp_account2 = TempAccount {
            temp_account_id: temp_account_id2.to_string(),
            email_address: "test1@test.com".to_string(),
            created_at: current_date_time
                - Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR)
                - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(2);
        map.insert(temp_account_id1.to_string(), (temp_account1, false));
        map.insert(temp_account_id2.to_string(), (temp_account2, false));
        map
    }

    #[tokio::test]
    async fn delete_expired_temp_accounts_fail3() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredTempAccountsOperationMock {
            temp_accounts: create_dummy_1_failed_expired_temp_account_and_1_expired_temp_account(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            format!(
                "[{}] 定期実行ツール (delete_expired_temp_accounts) 失敗通知",
                WEB_SITE_NAME
            ),
            vec![
                "user_temp_accountの期限切れレコード2個の内、1個の削除に失敗しました。".to_string(),
                "b860dc5138d146ac8127b0780fabce7d".to_string(),
            ],
        );

        let result = delete_expired_temp_accounts(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let err = result.expect_err("failed to get Err");
        let err_message = err.to_string();
        assert!(err_message.contains("2 were processed, 1 were failed"));
        assert!(err_message.contains("b860dc5138d146ac8127b0780fabce7d"));
        assert!(!err_message.contains("a860dc5138d146ac8127b0780fbbce7g"));
    }

    fn create_dummy_1_failed_expired_temp_account_and_1_expired_temp_account(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<String, (TempAccount, bool)> {
        let temp_account_id1 = "b860dc5138d146ac8127b0780fabce7d";
        let temp_account1 = TempAccount {
            temp_account_id: temp_account_id1.to_string(),
            email_address: "test1@test.com".to_string(),
            created_at: current_date_time
                - Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR)
                - Duration::seconds(1),
        };
        let temp_account_id2 = "a860dc5138d146ac8127b0780fbbce7g";
        let temp_account2 = TempAccount {
            temp_account_id: temp_account_id2.to_string(),
            email_address: "test1@test.com".to_string(),
            created_at: current_date_time
                - Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR)
                - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(temp_account_id1.to_string(), (temp_account1, false));
        map.insert(temp_account_id2.to_string(), (temp_account2, true));
        map
    }
}
