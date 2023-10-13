// Copyright 2023 Ken Miura

use chrono::{DateTime, Duration, FixedOffset};
use dotenv::dotenv;
use entity::sea_orm::{
    prelude::async_trait::async_trait, ColumnTrait, ConnectOptions, Database, DatabaseConnection,
    DatabaseTransaction, EntityTrait, ModelTrait, QueryFilter, QuerySelect, TransactionError,
    TransactionTrait,
};
use std::{env::set_var, error::Error, process::exit};
use tracing::{error, info};

use common::{
    admin::{
        TransactionExecutionError, KEY_TO_DB_ADMIN_NAME, KEY_TO_DB_ADMIN_PASSWORD,
        NUM_OF_MAX_TARGET_RECORDS,
    },
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

const VALID_PERIOD_OF_DELETED_USER_ACCOUNT_IN_DAYS: i64 = 90;

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
        "delete_expired_deleted_user_accounts={},common={},sea_orm={}",
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

    let op = DeleteExpiredDeletedUserAccountsOperationImpl { pool };

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

    let result = delete_expired_deleted_user_accounts(
        current_date_time,
        *NUM_OF_MAX_TARGET_RECORDS,
        &op,
        &smtp_client,
    )
    .await;

    let deleted_num = result.unwrap_or_else(|e| {
        error!("failed to delete expired deleted user accounts: {}", e);
        exit(APPLICATION_ERR)
    });

    info!("{} deleted user account(s) were (was) deleted", deleted_num);
    exit(SUCCESS)
}

async fn delete_expired_deleted_user_accounts(
    current_date_time: DateTime<FixedOffset>,
    num_of_max_target_records: u64,
    op: &impl DeleteExpiredDeletedUserAccountsOperation,
    send_mail: &impl SendMail,
) -> Result<usize, Box<dyn Error>> {
    let criteria = current_date_time - Duration::days(VALID_PERIOD_OF_DELETED_USER_ACCOUNT_IN_DAYS);
    let limit = if num_of_max_target_records != 0 {
        Some(num_of_max_target_records)
    } else {
        None
    };

    let expired_deleted_user_accounts = op
        .get_expired_deleted_user_accounts(criteria, limit)
        .await?;
    let num_of_expired_deleted_user_accounts = expired_deleted_user_accounts.len();

    let mut delete_failed: Vec<DeletedUserAccount> =
        Vec::with_capacity(expired_deleted_user_accounts.len());
    for expired_deleted_user_account in expired_deleted_user_accounts {
        let result = op
            .delete_deleted_user_account(expired_deleted_user_account.user_account_id)
            .await;
        if result.is_err() {
            error!("failed delete_deleted_user_account: {:?}", result);
            delete_failed.push(expired_deleted_user_account);
        }
        op.wait_for_next_iteration().await;
    }

    if !delete_failed.is_empty() {
        let subject = format!(
            "[{}] 定期実行ツール (delete_expired_deleted_user_accounts) 失敗通知",
            WEB_SITE_NAME
        );
        let num_of_delete_failed = delete_failed.len();
        let text = create_text(
            num_of_expired_deleted_user_accounts,
            num_of_delete_failed,
            &delete_failed,
        );
        let err_message = format!(
            "{} processed, {} failed (detail: {:?})",
            num_of_expired_deleted_user_accounts, num_of_delete_failed, delete_failed
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

    Ok(num_of_expired_deleted_user_accounts)
}

#[async_trait]
trait DeleteExpiredDeletedUserAccountsOperation {
    async fn get_expired_deleted_user_accounts(
        &self,
        criteria: DateTime<FixedOffset>,
        limit: Option<u64>,
    ) -> Result<Vec<DeletedUserAccount>, Box<dyn Error>>;

    async fn delete_deleted_user_account(&self, user_account_id: i64)
        -> Result<(), Box<dyn Error>>;

    /// 外部サービスに依存するアクションをする場合、その外部サービスのレートリミットにかからないように一定時間待つ
    async fn wait_for_next_iteration(&self);
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct DeletedUserAccount {
    user_account_id: i64,
    email_address: String,
    last_login_time: Option<DateTime<FixedOffset>>,
    created_at: DateTime<FixedOffset>,
    mfa_enabled_at: Option<DateTime<FixedOffset>>,
    disabled_at: Option<DateTime<FixedOffset>>,
    deleted_at: DateTime<FixedOffset>,
}

struct DeleteExpiredDeletedUserAccountsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl DeleteExpiredDeletedUserAccountsOperation for DeleteExpiredDeletedUserAccountsOperationImpl {
    async fn get_expired_deleted_user_accounts(
        &self,
        criteria: DateTime<FixedOffset>,
        limit: Option<u64>,
    ) -> Result<Vec<DeletedUserAccount>, Box<dyn Error>> {
        let models = entity::deleted_user_account::Entity::find()
            .filter(entity::deleted_user_account::Column::DeletedAt.lt(criteria))
            .limit(limit)
            .all(&self.pool)
            .await
            .map_err(|e| format!("failed to get deleted_user_account: {}", e))?;
        Ok(models
            .into_iter()
            .map(|m| DeletedUserAccount {
                user_account_id: m.user_account_id,
                email_address: m.email_address,
                last_login_time: m.last_login_time,
                created_at: m.created_at,
                mfa_enabled_at: m.mfa_enabled_at,
                disabled_at: m.disabled_at,
                deleted_at: m.deleted_at,
            })
            .collect())
    }

    async fn delete_deleted_user_account(
        &self,
        user_account_id: i64,
    ) -> Result<(), Box<dyn Error>> {
        let result = self
            .pool
            .transaction::<_, (bool, usize, bool, bool, bool), TransactionExecutionError>(|txn| {
                Box::pin(async move {
                    let dua = lock_deleted_user_account_exclusively(user_account_id, txn).await?;

                    let deleted_identity = entity::identity::Entity::delete_by_id(user_account_id)
                        .exec(txn)
                        .await
                        .map_err(|e| TransactionExecutionError {
                            message: format!(
                                "failed to delete identity (user_account_id: {}): {}",
                                user_account_id, e
                            ),
                        })?;

                    let num_of_careers_deleted = delete_careers(user_account_id, txn).await?;

                    // documentレコードとそれに対応したインデックスはユーザーがアカウントを削除するとき
                    // （user_account -> deleted_user_accountに移動するとき）に実施済のためここでは必要ない

                    let deleted_consulting_fee =
                        entity::consulting_fee::Entity::delete_by_id(user_account_id)
                            .exec(txn)
                            .await
                            .map_err(|e| TransactionExecutionError {
                                message: format!(
                                    "failed to delete consulting_fee (user_account_id: {}): {}",
                                    user_account_id, e
                                ),
                            })?;

                    let deleted_mfa_info = entity::mfa_info::Entity::delete_by_id(user_account_id)
                        .exec(txn)
                        .await
                        .map_err(|e| TransactionExecutionError {
                            message: format!(
                                "failed to delete mfa_info (user_account_id: {}): {}",
                                user_account_id, e
                            ),
                        })?;

                    let deleted_bank_account =
                        entity::bank_account::Entity::delete_by_id(user_account_id)
                            .exec(txn)
                            .await
                            .map_err(|e| TransactionExecutionError {
                                message: format!(
                                    "failed to delete bank_account (user_account_id: {}): {}",
                                    user_account_id, e
                                ),
                            })?;

                    let _ = dua
                        .delete(txn)
                        .await
                        .map_err(|e| TransactionExecutionError {
                            message: format!(
                                "failed to delete deleted_user_account (user_account_id: {}): {}",
                                user_account_id, e
                            ),
                        })?;

                    Ok((
                        deleted_identity.rows_affected != 0,
                        num_of_careers_deleted,
                        deleted_consulting_fee.rows_affected != 0,
                        deleted_mfa_info.rows_affected != 0,
                        deleted_bank_account.rows_affected != 0,
                    ))
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
        info!("identity deleted: {}, num of careers deleted: {}, consulting fee deleted: {}, mfa info deleted: {}, bank account deleted: {}",
            result.0, result.1, result.2, result.3, result.4);
        Ok(())
    }

    async fn wait_for_next_iteration(&self) {
        // 特に外部サービスに依存する処理はないため何もしない
    }
}

async fn lock_deleted_user_account_exclusively(
    user_account_id: i64,
    txn: &DatabaseTransaction,
) -> Result<entity::deleted_user_account::Model, TransactionExecutionError> {
    let result = entity::deleted_user_account::Entity::find_by_id(user_account_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| TransactionExecutionError {
            message: format!(
                "failed to find deleted_user_account (user_account_id: {}): {}",
                user_account_id, e
            ),
        })?;
    let result = result.ok_or_else(|| TransactionExecutionError {
        message: format!(
            "no deleted_user_account found (user_account_id: {})",
            user_account_id
        ),
    })?;
    Ok(result)
}

async fn delete_careers(
    user_account_id: i64,
    txn: &DatabaseTransaction,
) -> Result<usize, TransactionExecutionError> {
    let models = entity::career::Entity::find()
        .filter(entity::career::Column::UserAccountId.eq(user_account_id))
        .all(txn)
        .await
        .map_err(|e| TransactionExecutionError {
            message: format!(
                "failed to filter career (user_account_id: {}): {}",
                user_account_id, e
            ),
        })?;
    // ユーザーアカウントひとつあたりの職務経歴数は制限してあるため、繰り返し処理を許容する
    for model in &models {
        let career_id = model.career_id;
        let _ = entity::career::Entity::delete_by_id(career_id)
            .exec(txn)
            .await
            .map_err(|e| TransactionExecutionError {
                message: format!("failed to delete career (career_id: {}): {}", career_id, e),
            })?;
    }
    Ok(models.len())
}

fn create_text(
    num_of_expired_deleted_user_accounts: usize,
    num_of_delete_failed: usize,
    delete_failed: &[DeletedUserAccount],
) -> String {
    format!(
        r"deleted_user_accountの期限切れレコード{}個の内、{}個の削除に失敗しました。

【詳細】
{:?}",
        num_of_expired_deleted_user_accounts, num_of_delete_failed, delete_failed
    )
}

#[cfg(test)]
mod tests {

    use std::{cmp::min, collections::HashMap};

    use chrono::TimeZone;
    use common::ErrResp;

    use super::*;

    struct DeleteExpiredDeletedUserAccountsOperationMock {
        deleted_user_accounts: HashMap<i64, (DeletedUserAccount, bool)>,
        current_date_time: DateTime<FixedOffset>,
        limit: u64,
    }

    #[async_trait]
    impl DeleteExpiredDeletedUserAccountsOperation for DeleteExpiredDeletedUserAccountsOperationMock {
        async fn get_expired_deleted_user_accounts(
            &self,
            criteria: DateTime<FixedOffset>,
            limit: Option<u64>,
        ) -> Result<Vec<DeletedUserAccount>, Box<dyn Error>> {
            assert_eq!(
                self.current_date_time
                    - Duration::days(VALID_PERIOD_OF_DELETED_USER_ACCOUNT_IN_DAYS),
                criteria
            );
            if self.limit != 0 {
                assert_eq!(Some(self.limit), limit);
            } else {
                assert_eq!(None, limit);
            }
            let expired_deleted_user_accounts: Vec<DeletedUserAccount> = self
                .deleted_user_accounts
                .values()
                .clone()
                .filter(|m| m.0.deleted_at < criteria)
                .map(|m| m.0.clone())
                .collect();
            let results = if let Some(limit) = limit {
                let limit = min(limit as usize, expired_deleted_user_accounts.len());
                let mut expired_deleted_user_accounts_limited = Vec::with_capacity(limit);
                (0..limit).for_each(|i| {
                    expired_deleted_user_accounts_limited
                        .push(expired_deleted_user_accounts[i].clone())
                });
                expired_deleted_user_accounts_limited
            } else {
                expired_deleted_user_accounts
            };
            Ok(results)
        }

        async fn delete_deleted_user_account(
            &self,
            user_account_id: i64,
        ) -> Result<(), Box<dyn Error>> {
            let deleted_user_account = self
                .deleted_user_accounts
                .get(&user_account_id)
                .expect("assert that deleted_user_account has value!");
            if !deleted_user_account.1 {
                return Err("mock error message".into());
            }
            Ok(())
        }

        async fn wait_for_next_iteration(&self) {
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
    async fn delete_expired_deleted_user_accounts_success0() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredDeletedUserAccountsOperationMock {
            deleted_user_accounts: HashMap::with_capacity(0),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_deleted_user_accounts(
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
    async fn delete_expired_deleted_user_accounts_success1() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredDeletedUserAccountsOperationMock {
            deleted_user_accounts: create_dummy_1_non_expired_deleted_user_account(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_deleted_user_accounts(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 0);
    }

    fn create_dummy_1_non_expired_deleted_user_account(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (DeletedUserAccount, bool)> {
        let user_account_id = 1234;
        let deleted_user_account = DeletedUserAccount {
            user_account_id,
            email_address: "test1@test.com".to_string(),
            last_login_time: Some(
                JAPANESE_TIME_ZONE
                    .with_ymd_and_hms(2023, 8, 5, 13, 24, 56)
                    .unwrap(),
            ),
            created_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 1, 10, 2, 1)
                .unwrap(),
            mfa_enabled_at: None,
            disabled_at: None,
            deleted_at: current_date_time
                - Duration::days(VALID_PERIOD_OF_DELETED_USER_ACCOUNT_IN_DAYS),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(user_account_id, (deleted_user_account, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_deleted_user_accounts_success2() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredDeletedUserAccountsOperationMock {
            deleted_user_accounts: create_dummy_1_expired_deleted_user_account(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_deleted_user_accounts(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 1);
    }

    fn create_dummy_1_expired_deleted_user_account(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (DeletedUserAccount, bool)> {
        let user_account_id = 1234;
        let deleted_user_account = DeletedUserAccount {
            user_account_id,
            email_address: "test1@test.com".to_string(),
            last_login_time: Some(
                JAPANESE_TIME_ZONE
                    .with_ymd_and_hms(2023, 8, 5, 13, 24, 56)
                    .unwrap(),
            ),
            created_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 1, 10, 2, 1)
                .unwrap(),
            mfa_enabled_at: None,
            disabled_at: None,
            deleted_at: current_date_time
                - Duration::days(VALID_PERIOD_OF_DELETED_USER_ACCOUNT_IN_DAYS)
                - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(user_account_id, (deleted_user_account, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_deleted_user_accounts_success3() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 00, 40)
            .unwrap();
        let max_num_of_target_records = 1;
        let op = DeleteExpiredDeletedUserAccountsOperationMock {
            deleted_user_accounts: create_dummy_1_expired_deleted_user_account(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_deleted_user_accounts(
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
    async fn delete_expired_deleted_user_accounts_success4() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 00, 40)
            .unwrap();
        let max_num_of_target_records = 2;
        let op = DeleteExpiredDeletedUserAccountsOperationMock {
            deleted_user_accounts: create_dummy_1_expired_deleted_user_account(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_deleted_user_accounts(
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
    async fn delete_expired_deleted_user_accounts_success5() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredDeletedUserAccountsOperationMock {
            deleted_user_accounts: create_dummy_2_expired_deleted_user_accounts(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_deleted_user_accounts(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 2);
    }

    fn create_dummy_2_expired_deleted_user_accounts(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (DeletedUserAccount, bool)> {
        let user_account_id1 = 1234;
        let deleted_user_account1 = DeletedUserAccount {
            user_account_id: user_account_id1,
            email_address: "test1@test.com".to_string(),
            last_login_time: Some(
                JAPANESE_TIME_ZONE
                    .with_ymd_and_hms(2023, 8, 5, 13, 24, 56)
                    .unwrap(),
            ),
            created_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 1, 10, 2, 1)
                .unwrap(),
            mfa_enabled_at: None,
            disabled_at: None,
            deleted_at: current_date_time
                - Duration::days(VALID_PERIOD_OF_DELETED_USER_ACCOUNT_IN_DAYS)
                - Duration::seconds(1),
        };

        let user_account_id2 = 4567;
        let deleted_user_account2 = DeletedUserAccount {
            user_account_id: user_account_id2,
            email_address: "test2@test.com".to_string(),
            last_login_time: Some(
                JAPANESE_TIME_ZONE
                    .with_ymd_and_hms(2023, 7, 15, 18, 42, 23)
                    .unwrap(),
            ),
            created_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2022, 12, 1, 11, 32, 11)
                .unwrap(),
            mfa_enabled_at: None,
            disabled_at: None,
            deleted_at: current_date_time
                - Duration::days(VALID_PERIOD_OF_DELETED_USER_ACCOUNT_IN_DAYS)
                - Duration::seconds(1),
        };

        let mut map = HashMap::with_capacity(2);
        map.insert(user_account_id1, (deleted_user_account1, true));
        map.insert(user_account_id2, (deleted_user_account2, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_deleted_user_accounts_success6() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 21, 8, 40)
            .unwrap();
        let max_num_of_target_records = 1;
        let op = DeleteExpiredDeletedUserAccountsOperationMock {
            deleted_user_accounts: create_dummy_2_expired_deleted_user_accounts(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_deleted_user_accounts(
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
    async fn delete_expired_deleted_user_accounts_success7() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 21, 8, 40)
            .unwrap();
        let max_num_of_target_records = 2;
        let op = DeleteExpiredDeletedUserAccountsOperationMock {
            deleted_user_accounts: create_dummy_2_expired_deleted_user_accounts(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_deleted_user_accounts(
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
    async fn delete_expired_deleted_user_accounts_success8() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 0, 40)
            .unwrap();
        let max_num_of_target_records = 3;
        let op = DeleteExpiredDeletedUserAccountsOperationMock {
            deleted_user_accounts: create_dummy_2_expired_deleted_user_accounts(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_deleted_user_accounts(
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
    async fn delete_expired_deleted_user_accounts_success9() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 0, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredDeletedUserAccountsOperationMock {
            deleted_user_accounts: create_dummy_1_non_expired_and_1_expired_deleted_user_account(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_deleted_user_accounts(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 1);
    }

    fn create_dummy_1_non_expired_and_1_expired_deleted_user_account(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (DeletedUserAccount, bool)> {
        let user_account_id1 = 1234;
        let deleted_user_account1 = DeletedUserAccount {
            user_account_id: user_account_id1,
            email_address: "test1@test.com".to_string(),
            last_login_time: Some(
                JAPANESE_TIME_ZONE
                    .with_ymd_and_hms(2023, 8, 5, 13, 24, 56)
                    .unwrap(),
            ),
            created_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 1, 10, 2, 1)
                .unwrap(),
            mfa_enabled_at: None,
            disabled_at: None,
            deleted_at: current_date_time
                - Duration::days(VALID_PERIOD_OF_DELETED_USER_ACCOUNT_IN_DAYS),
        };

        let user_account_id2 = 4567;
        let deleted_user_account2 = DeletedUserAccount {
            user_account_id: user_account_id2,
            email_address: "test2@test.com".to_string(),
            last_login_time: Some(
                JAPANESE_TIME_ZONE
                    .with_ymd_and_hms(2023, 7, 15, 18, 42, 23)
                    .unwrap(),
            ),
            created_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2022, 12, 1, 11, 32, 11)
                .unwrap(),
            mfa_enabled_at: None,
            disabled_at: None,
            deleted_at: current_date_time
                - Duration::days(VALID_PERIOD_OF_DELETED_USER_ACCOUNT_IN_DAYS)
                - Duration::seconds(1),
        };

        let mut map = HashMap::with_capacity(2);
        map.insert(user_account_id1, (deleted_user_account1, true));
        map.insert(user_account_id2, (deleted_user_account2, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_deleted_user_accounts_success10() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 0, 40)
            .unwrap();
        let max_num_of_target_records = 1;
        let op = DeleteExpiredDeletedUserAccountsOperationMock {
            deleted_user_accounts: create_dummy_1_non_expired_and_1_expired_deleted_user_account(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_deleted_user_accounts(
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
    async fn delete_expired_deleted_user_accounts_success11() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 0, 40)
            .unwrap();
        let max_num_of_target_records = 2;
        let op = DeleteExpiredDeletedUserAccountsOperationMock {
            deleted_user_accounts: create_dummy_1_non_expired_and_1_expired_deleted_user_account(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_deleted_user_accounts(
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
    async fn delete_expired_deleted_user_accounts_fail1() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 0, 00)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredDeletedUserAccountsOperationMock {
            deleted_user_accounts: create_dummy_1_failed_expired_deleted_user_account(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            format!(
                "[{}] 定期実行ツール (delete_expired_deleted_user_accounts) 失敗通知",
                WEB_SITE_NAME
            ),
            vec![
                "deleted_user_accountの期限切れレコード1個の内、1個の削除に失敗しました。"
                    .to_string(),
                "1234".to_string(),
                "test1@test.com".to_string(),
                "2023-08-05T13:24:56+09:00".to_string(),
                "2023-08-01T10:02:01+09:00".to_string(),
                "2023-05-29T07:59:59+09:00".to_string(),
            ],
        );

        let result = delete_expired_deleted_user_accounts(
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
        assert!(err_message.contains("test1@test.com"));
        assert!(err_message.contains("2023-08-05T13:24:56+09:00"));
        assert!(err_message.contains("2023-08-01T10:02:01+09:00"));
        assert!(err_message.contains("2023-05-29T07:59:59+09:00"));
    }

    fn create_dummy_1_failed_expired_deleted_user_account(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (DeletedUserAccount, bool)> {
        let user_account_id = 1234;
        let deleted_user_account = DeletedUserAccount {
            user_account_id,
            email_address: "test1@test.com".to_string(),
            last_login_time: Some(
                JAPANESE_TIME_ZONE
                    .with_ymd_and_hms(2023, 8, 5, 13, 24, 56)
                    .unwrap(),
            ),
            created_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 1, 10, 2, 1)
                .unwrap(),
            mfa_enabled_at: None,
            disabled_at: None,
            deleted_at: current_date_time
                - Duration::days(VALID_PERIOD_OF_DELETED_USER_ACCOUNT_IN_DAYS)
                - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(user_account_id, (deleted_user_account, false));
        map
    }

    #[tokio::test]
    async fn delete_expired_deleted_user_accounts_fail2() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 0, 00)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredDeletedUserAccountsOperationMock {
            deleted_user_accounts: create_dummy_2_failed_expired_deleted_user_accounts(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            format!(
                "[{}] 定期実行ツール (delete_expired_deleted_user_accounts) 失敗通知",
                WEB_SITE_NAME
            ),
            vec![
                "deleted_user_accountの期限切れレコード2個の内、2個の削除に失敗しました。"
                    .to_string(),
                "1234".to_string(),
                "test1@test.com".to_string(),
                "2023-08-05T13:24:56+09:00".to_string(),
                "2023-08-01T10:02:01+09:00".to_string(),
                "2023-05-29T07:59:59+09:00".to_string(),
                "4567".to_string(),
                "test2@test.com".to_string(),
                "2023-07-15T18:42:23+09:00".to_string(),
                "2022-12-01T11:32:11+09:00".to_string(),
                "2023-05-29T07:59:58+09:00".to_string(),
            ],
        );

        let result = delete_expired_deleted_user_accounts(
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
        assert!(err_message.contains("test1@test.com"));
        assert!(err_message.contains("2023-08-05T13:24:56+09:00"));
        assert!(err_message.contains("2023-08-01T10:02:01+09:00"));
        assert!(err_message.contains("2023-05-29T07:59:59+09:00"));

        assert!(err_message.contains("4567"));
        assert!(err_message.contains("test2@test.com"));
        assert!(err_message.contains("2023-07-15T18:42:23+09:00"));
        assert!(err_message.contains("2022-12-01T11:32:11+09:00"));
        assert!(err_message.contains("2023-05-29T07:59:58+09:00"));
    }

    fn create_dummy_2_failed_expired_deleted_user_accounts(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (DeletedUserAccount, bool)> {
        let user_account_id1 = 1234;
        let deleted_user_account1 = DeletedUserAccount {
            user_account_id: user_account_id1,
            email_address: "test1@test.com".to_string(),
            last_login_time: Some(
                JAPANESE_TIME_ZONE
                    .with_ymd_and_hms(2023, 8, 5, 13, 24, 56)
                    .unwrap(),
            ),
            created_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 1, 10, 2, 1)
                .unwrap(),
            mfa_enabled_at: None,
            disabled_at: None,
            deleted_at: current_date_time
                - Duration::days(VALID_PERIOD_OF_DELETED_USER_ACCOUNT_IN_DAYS)
                - Duration::seconds(1),
        };

        let user_account_id2 = 4567;
        let deleted_user_account2 = DeletedUserAccount {
            user_account_id: user_account_id2,
            email_address: "test2@test.com".to_string(),
            last_login_time: Some(
                JAPANESE_TIME_ZONE
                    .with_ymd_and_hms(2023, 7, 15, 18, 42, 23)
                    .unwrap(),
            ),
            created_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2022, 12, 1, 11, 32, 11)
                .unwrap(),
            mfa_enabled_at: None,
            disabled_at: None,
            deleted_at: current_date_time
                - Duration::days(VALID_PERIOD_OF_DELETED_USER_ACCOUNT_IN_DAYS)
                - Duration::seconds(2),
        };

        let mut map = HashMap::with_capacity(2);
        map.insert(user_account_id1, (deleted_user_account1, false));
        map.insert(user_account_id2, (deleted_user_account2, false));
        map
    }

    #[tokio::test]
    async fn delete_expired_deleted_user_accounts_fail3() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 27, 8, 0, 00)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredDeletedUserAccountsOperationMock {
            deleted_user_accounts:
                create_dummy_1_failed_expired_deleted_user_account_and_1_expired_deleted_user_account(
                    current_date_time,
                ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            format!(
                "[{}] 定期実行ツール (delete_expired_deleted_user_accounts) 失敗通知",
                WEB_SITE_NAME
            ),
            vec![
                "deleted_user_accountの期限切れレコード2個の内、1個の削除に失敗しました。"
                    .to_string(),
                "4567".to_string(),
                "test2@test.com".to_string(),
                "2023-07-15T18:42:23+09:00".to_string(),
                "2022-12-01T11:32:11+09:00".to_string(),
                "2023-05-29T07:59:58+09:00".to_string(),
            ],
        );

        let result = delete_expired_deleted_user_accounts(
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
        assert!(!err_message.contains("test1@test.com"));
        assert!(!err_message.contains("2023-08-05T13:24:56+09:00"));
        assert!(!err_message.contains("2023-08-01T10:02:01+09:00"));
        assert!(!err_message.contains("2023-05-29T07:59:59+09:00"));

        assert!(err_message.contains("4567"));
        assert!(err_message.contains("test2@test.com"));
        assert!(err_message.contains("2023-07-15T18:42:23+09:00"));
        assert!(err_message.contains("2022-12-01T11:32:11+09:00"));
        assert!(err_message.contains("2023-05-29T07:59:58+09:00"));
    }

    fn create_dummy_1_failed_expired_deleted_user_account_and_1_expired_deleted_user_account(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<i64, (DeletedUserAccount, bool)> {
        let user_account_id1 = 1234;
        let deleted_user_account1 = DeletedUserAccount {
            user_account_id: user_account_id1,
            email_address: "test1@test.com".to_string(),
            last_login_time: Some(
                JAPANESE_TIME_ZONE
                    .with_ymd_and_hms(2023, 8, 5, 13, 24, 56)
                    .unwrap(),
            ),
            created_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 8, 1, 10, 2, 1)
                .unwrap(),
            mfa_enabled_at: None,
            disabled_at: None,
            deleted_at: current_date_time
                - Duration::days(VALID_PERIOD_OF_DELETED_USER_ACCOUNT_IN_DAYS)
                - Duration::seconds(1),
        };

        let user_account_id2 = 4567;
        let deleted_user_account2 = DeletedUserAccount {
            user_account_id: user_account_id2,
            email_address: "test2@test.com".to_string(),
            last_login_time: Some(
                JAPANESE_TIME_ZONE
                    .with_ymd_and_hms(2023, 7, 15, 18, 42, 23)
                    .unwrap(),
            ),
            created_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2022, 12, 1, 11, 32, 11)
                .unwrap(),
            mfa_enabled_at: None,
            disabled_at: None,
            deleted_at: current_date_time
                - Duration::days(VALID_PERIOD_OF_DELETED_USER_ACCOUNT_IN_DAYS)
                - Duration::seconds(2),
        };

        let mut map = HashMap::with_capacity(2);
        map.insert(user_account_id1, (deleted_user_account1, true));
        map.insert(user_account_id2, (deleted_user_account2, false));
        map
    }
}
