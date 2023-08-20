// Copyright 2023 Ken Miura

use chrono::{DateTime, Duration, FixedOffset};
use dotenv::dotenv;
use entity::sea_orm::{
    prelude::async_trait::async_trait, ColumnTrait, ConnectOptions, Database, DatabaseConnection,
    EntityTrait, QueryFilter, QuerySelect,
};
use std::{env::var, error::Error, process::exit};

use common::{
    db::{create_db_url, KEY_TO_DB_HOST, KEY_TO_DB_NAME, KEY_TO_DB_PORT},
    smtp::{
        SendMail, SmtpClient, ADMIN_EMAIL_ADDRESS, AWS_SES_ACCESS_KEY_ID, AWS_SES_ENDPOINT_URI,
        AWS_SES_REGION, AWS_SES_SECRET_ACCESS_KEY, KEY_TO_ADMIN_EMAIL_ADDRESS,
        KEY_TO_AWS_SES_ACCESS_KEY_ID, KEY_TO_AWS_SES_ENDPOINT_URI, KEY_TO_AWS_SES_REGION,
        KEY_TO_AWS_SES_SECRET_ACCESS_KEY, KEY_TO_SYSTEM_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS,
    },
    util::check_env_vars,
    JAPANESE_TIME_ZONE, VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE, WEB_SITE_NAME,
};

const KEY_TO_DB_ADMIN_NAME: &str = "DB_ADMIN_NAME";
const KEY_TO_DB_ADMIN_PASSWORD: &str = "DB_ADMIN_PASSWORD";
const KEY_TO_NUM_OF_MAX_TARGET_RECORDS: &str = "NUM_OF_MAX_TARGET_RECORDS";

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
        KEY_TO_NUM_OF_MAX_TARGET_RECORDS.to_string(),
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

    let num_of_max_target_records = var(KEY_TO_NUM_OF_MAX_TARGET_RECORDS)
        .unwrap_or_else(|_| {
            panic!(
                "Not environment variable found: environment variable \"{}\" must be set",
                KEY_TO_NUM_OF_MAX_TARGET_RECORDS
            )
        })
        .parse()
        .expect("failed to get Ok");

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
    let op = DeleteExpiredPwdChangeReqOperationImpl { pool };

    let smtp_client = SmtpClient::new(
        AWS_SES_REGION.as_str(),
        AWS_SES_ACCESS_KEY_ID.as_str(),
        AWS_SES_SECRET_ACCESS_KEY.as_str(),
        AWS_SES_ENDPOINT_URI.as_str(),
    )
    .await;

    let result = delete_expired_pwd_change_reqs(
        current_date_time,
        num_of_max_target_records,
        &op,
        &smtp_client,
    )
    .await;

    let deleted_num = result.unwrap_or_else(|e| {
        println!("failed to delete expired pwd change reqs: {}", e);
        exit(APPLICATION_ERR)
    });

    println!("{} pwd change reqs were deleted successfully", deleted_num);
    exit(SUCCESS)
}

fn construct_db_url(
    key_to_db_host: &str,
    key_to_db_port: &str,
    key_to_db_name: &str,
    key_to_db_admin_name: &str,
    key_to_db_admin_password: &str,
) -> String {
    let db_host = var(key_to_db_host).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            key_to_db_host
        )
    });
    let db_port = var(key_to_db_port).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            key_to_db_port
        )
    });
    let db_name = var(key_to_db_name).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            key_to_db_name
        )
    });
    let db_admin_name = var(key_to_db_admin_name).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            key_to_db_admin_name
        )
    });
    let db_admin_password = var(key_to_db_admin_password).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            key_to_db_admin_password
        )
    });
    create_db_url(
        &db_host,
        &db_port,
        &db_name,
        &db_admin_name,
        &db_admin_password,
    )
}

async fn delete_expired_pwd_change_reqs(
    current_date_time: DateTime<FixedOffset>,
    num_of_max_target_records: u64,
    op: &impl DeleteExpiredPwdChangeReqOperation,
    send_mail: &impl SendMail,
) -> Result<usize, Box<dyn Error>> {
    let criteria =
        current_date_time - Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE);
    let limit = if num_of_max_target_records != 0 {
        Some(num_of_max_target_records)
    } else {
        None
    };

    let expired_pwd_change_reqs = op.get_expired_pwd_change_reqs(criteria, limit).await?;
    let num_of_expired_pwd_change_reqs = expired_pwd_change_reqs.len();

    let mut delete_failed: Vec<PwdChangeReq> = Vec::with_capacity(expired_pwd_change_reqs.len());
    for expired_pwd_change_req in expired_pwd_change_reqs {
        let result = op
            .delete_pwd_change_req(&expired_pwd_change_req.pwd_change_req_id)
            .await;
        if result.is_err() {
            delete_failed.push(expired_pwd_change_req);
        }
    }

    if !delete_failed.is_empty() {
        let subject = format!(
            "[{}] 定期実行ツール (delete_expired_pwd_change_reqs) 失敗通知",
            WEB_SITE_NAME
        );
        let num_of_delete_failed = delete_failed.len();
        let text = create_text(
            num_of_expired_pwd_change_reqs,
            num_of_delete_failed,
            &delete_failed,
        );
        let err_message = format!(
            "{} were processed, {} were failed (detail: {:?})",
            num_of_expired_pwd_change_reqs, num_of_delete_failed, delete_failed
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

    Ok(num_of_expired_pwd_change_reqs)
}

#[async_trait]
trait DeleteExpiredPwdChangeReqOperation {
    async fn get_expired_pwd_change_reqs(
        &self,
        criteria: DateTime<FixedOffset>,
        limit: Option<u64>,
    ) -> Result<Vec<PwdChangeReq>, Box<dyn Error>>;

    async fn delete_pwd_change_req(&self, pwd_change_req_id: &str) -> Result<(), Box<dyn Error>>;
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct PwdChangeReq {
    pwd_change_req_id: String,
    email_address: String,
    requested_at: DateTime<FixedOffset>,
}

struct DeleteExpiredPwdChangeReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl DeleteExpiredPwdChangeReqOperation for DeleteExpiredPwdChangeReqOperationImpl {
    async fn get_expired_pwd_change_reqs(
        &self,
        criteria: DateTime<FixedOffset>,
        limit: Option<u64>,
    ) -> Result<Vec<PwdChangeReq>, Box<dyn Error>> {
        let models = entity::pwd_change_req::Entity::find()
            .filter(entity::pwd_change_req::Column::RequestedAt.lt(criteria))
            .limit(limit)
            .all(&self.pool)
            .await
            .map_err(|e| format!("failed to get pwd_change_req: {}", e))?;
        Ok(models
            .into_iter()
            .map(|m| PwdChangeReq {
                pwd_change_req_id: m.pwd_change_req_id,
                email_address: m.email_address,
                requested_at: m.requested_at,
            })
            .collect())
    }

    async fn delete_pwd_change_req(&self, pwd_change_req_id: &str) -> Result<(), Box<dyn Error>> {
        let _ = entity::pwd_change_req::Entity::delete_by_id(pwd_change_req_id)
            .exec(&self.pool)
            .await
            .map_err(|e| {
                format!(
                    "failed to delete pwd_change_req (pwd_change_req_id: {}): {}",
                    pwd_change_req_id, e
                )
            })?;
        Ok(())
    }
}

fn create_text(
    num_of_expired_pwd_change_reqs: usize,
    num_of_delete_failed: usize,
    delete_failed: &[PwdChangeReq],
) -> String {
    format!(
        r"pwd_change_reqの期限切れレコード{}個の内、{}個の削除に失敗しました。

【詳細】
{:?}",
        num_of_expired_pwd_change_reqs, num_of_delete_failed, delete_failed
    )
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use chrono::TimeZone;
    use common::ErrResp;

    use super::*;

    struct DeleteExpiredPwdChangeReqOperationMock {
        pwd_change_reqs: HashMap<String, (PwdChangeReq, bool)>,
        current_date_time: DateTime<FixedOffset>,
        limit: u64,
    }

    #[async_trait]
    impl DeleteExpiredPwdChangeReqOperation for DeleteExpiredPwdChangeReqOperationMock {
        async fn get_expired_pwd_change_reqs(
            &self,
            criteria: DateTime<FixedOffset>,
            limit: Option<u64>,
        ) -> Result<Vec<PwdChangeReq>, Box<dyn Error>> {
            assert_eq!(
                self.current_date_time
                    - Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE),
                criteria
            );
            if self.limit != 0 {
                assert_eq!(Some(self.limit), limit);
            } else {
                assert_eq!(None, limit);
            }
            let expired_pwd_change_reqs = self
                .pwd_change_reqs
                .values()
                .clone()
                .filter(|m| m.0.requested_at < criteria)
                .map(|m| m.0.clone())
                .collect();
            Ok(expired_pwd_change_reqs)
        }

        async fn delete_pwd_change_req(
            &self,
            pwd_change_req_id: &str,
        ) -> Result<(), Box<dyn Error>> {
            let pwd_change_req = self
                .pwd_change_reqs
                .get(pwd_change_req_id)
                .expect("assert that pwd_change_req has value!");
            if !pwd_change_req.1 {
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
    async fn delete_expired_pwd_change_reqs_success0() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredPwdChangeReqOperationMock {
            pwd_change_reqs: HashMap::with_capacity(0),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_pwd_change_reqs(
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
    async fn delete_expired_pwd_change_reqs_success1() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredPwdChangeReqOperationMock {
            pwd_change_reqs: create_dummy_1_non_expired_pwd_change_req(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_pwd_change_reqs(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 0);
    }

    fn create_dummy_1_non_expired_pwd_change_req(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<String, (PwdChangeReq, bool)> {
        let pwd_change_req_id = "b860dc5138d146ac8127b0780fabce7d";
        let pwd_change_req = PwdChangeReq {
            pwd_change_req_id: pwd_change_req_id.to_string(),
            email_address: "test1@test.com".to_string(),
            requested_at: current_date_time
                - Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(pwd_change_req_id.to_string(), (pwd_change_req, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_pwd_change_reqs_success2() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredPwdChangeReqOperationMock {
            pwd_change_reqs: create_dummy_1_expired_pwd_change_req(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_pwd_change_reqs(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 1);
    }

    fn create_dummy_1_expired_pwd_change_req(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<String, (PwdChangeReq, bool)> {
        let pwd_change_req_id = "b860dc5138d146ac8127b0780fabce7d";
        let pwd_change_req = PwdChangeReq {
            pwd_change_req_id: pwd_change_req_id.to_string(),
            email_address: "test1@test.com".to_string(),
            requested_at: current_date_time
                - Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE)
                - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(pwd_change_req_id.to_string(), (pwd_change_req, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_pwd_change_reqs_success3() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 1;
        let op = DeleteExpiredPwdChangeReqOperationMock {
            pwd_change_reqs: create_dummy_1_expired_pwd_change_req(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_pwd_change_reqs(
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
    async fn delete_expired_pwd_change_reqs_success4() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 2;
        let op = DeleteExpiredPwdChangeReqOperationMock {
            pwd_change_reqs: create_dummy_1_expired_pwd_change_req(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_pwd_change_reqs(
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
    async fn delete_expired_pwd_change_reqs_success5() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredPwdChangeReqOperationMock {
            pwd_change_reqs: create_dummy_2_expired_pwd_change_reqs(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_pwd_change_reqs(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 2);
    }

    fn create_dummy_2_expired_pwd_change_reqs(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<String, (PwdChangeReq, bool)> {
        let pwd_change_req_id1 = "b860dc5138d146ac8127b0780fabce7d";
        let pwd_change_req1 = PwdChangeReq {
            pwd_change_req_id: pwd_change_req_id1.to_string(),
            email_address: "test1@test.com".to_string(),
            requested_at: current_date_time
                - Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE)
                - Duration::seconds(1),
        };
        let pwd_change_req_id2 = "c860dc5138d146ac8127b0780fabce7e";
        let pwd_change_req2 = PwdChangeReq {
            pwd_change_req_id: pwd_change_req_id2.to_string(),
            email_address: "test2@test.com".to_string(),
            requested_at: current_date_time
                - Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE)
                - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(pwd_change_req_id1.to_string(), (pwd_change_req1, true));
        map.insert(pwd_change_req_id2.to_string(), (pwd_change_req2, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_pwd_change_reqs_success6() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 1;
        let op = DeleteExpiredPwdChangeReqOperationMock {
            pwd_change_reqs: create_dummy_2_expired_pwd_change_reqs(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_pwd_change_reqs(
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
    async fn delete_expired_pwd_change_reqs_success7() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 2;
        let op = DeleteExpiredPwdChangeReqOperationMock {
            pwd_change_reqs: create_dummy_2_expired_pwd_change_reqs(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_pwd_change_reqs(
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
    async fn delete_expired_pwd_change_reqs_success8() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 3;
        let op = DeleteExpiredPwdChangeReqOperationMock {
            pwd_change_reqs: create_dummy_2_expired_pwd_change_reqs(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_pwd_change_reqs(
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
    async fn delete_expired_pwd_change_reqs_success9() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredPwdChangeReqOperationMock {
            pwd_change_reqs: create_dummy_1_non_expired_and_1_expired_pwd_change_req(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_pwd_change_reqs(
            current_date_time,
            max_num_of_target_records,
            &op,
            &send_mail_mock,
        )
        .await;

        let num_deleted = result.expect("failed to get Ok");
        assert_eq!(num_deleted, 1);
    }

    fn create_dummy_1_non_expired_and_1_expired_pwd_change_req(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<String, (PwdChangeReq, bool)> {
        let pwd_change_req_id1 = "b860dc5138d146ac8127b0780fabce7d";
        let pwd_change_req1 = PwdChangeReq {
            pwd_change_req_id: pwd_change_req_id1.to_string(),
            email_address: "test1@test.com".to_string(),
            requested_at: current_date_time
                - Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE),
        };
        let pwd_change_req_id2 = "c860dc5138d146ac8127b0780fabce7e";
        let pwd_change_req2 = PwdChangeReq {
            pwd_change_req_id: pwd_change_req_id2.to_string(),
            email_address: "test2@test.com".to_string(),
            requested_at: current_date_time
                - Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE)
                - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(pwd_change_req_id1.to_string(), (pwd_change_req1, true));
        map.insert(pwd_change_req_id2.to_string(), (pwd_change_req2, true));
        map
    }

    #[tokio::test]
    async fn delete_expired_pwd_change_reqs_success10() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 1;
        let op = DeleteExpiredPwdChangeReqOperationMock {
            pwd_change_reqs: create_dummy_1_non_expired_and_1_expired_pwd_change_req(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_pwd_change_reqs(
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
    async fn delete_expired_pwd_change_reqs_success11() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 2;
        let op = DeleteExpiredPwdChangeReqOperationMock {
            pwd_change_reqs: create_dummy_1_non_expired_and_1_expired_pwd_change_req(
                current_date_time,
            ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        // 成功時はメールを送らないので、わざと失敗するような内容でモックを生成する
        let send_mail_mock =
            SendMailMock::new("".to_string(), "".to_string(), "".to_string(), vec![]);

        let result = delete_expired_pwd_change_reqs(
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
    async fn delete_expired_pwd_change_reqs_fail1() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredPwdChangeReqOperationMock {
            pwd_change_reqs: create_dummy_1_failed_expired_pwd_change_req(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            format!(
                "[{}] 定期実行ツール (delete_expired_pwd_change_reqs) 失敗通知",
                WEB_SITE_NAME
            ),
            vec![
                "pwd_change_reqの期限切れレコード1個の内、1個の削除に失敗しました。".to_string(),
                "b860dc5138d146ac8127b0780fabce7d".to_string(),
            ],
        );

        let result = delete_expired_pwd_change_reqs(
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

    fn create_dummy_1_failed_expired_pwd_change_req(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<String, (PwdChangeReq, bool)> {
        let pwd_change_req_id = "b860dc5138d146ac8127b0780fabce7d";
        let pwd_change_req = PwdChangeReq {
            pwd_change_req_id: pwd_change_req_id.to_string(),
            email_address: "test1@test.com".to_string(),
            requested_at: current_date_time
                - Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE)
                - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(pwd_change_req_id.to_string(), (pwd_change_req, false));
        map
    }

    #[tokio::test]
    async fn delete_expired_pwd_change_reqs_fail2() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredPwdChangeReqOperationMock {
            pwd_change_reqs: create_dummy_2_failed_expired_pwd_change_reqs(current_date_time),
            current_date_time,
            limit: max_num_of_target_records,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            format!(
                "[{}] 定期実行ツール (delete_expired_pwd_change_reqs) 失敗通知",
                WEB_SITE_NAME
            ),
            vec![
                "pwd_change_reqの期限切れレコード2個の内、2個の削除に失敗しました。".to_string(),
                "b860dc5138d146ac8127b0780fabce7d".to_string(),
                "a860dc5138d146ac8127b0780fbbce7g".to_string(),
            ],
        );

        let result = delete_expired_pwd_change_reqs(
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

    fn create_dummy_2_failed_expired_pwd_change_reqs(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<String, (PwdChangeReq, bool)> {
        let pwd_change_req_id1 = "b860dc5138d146ac8127b0780fabce7d";
        let pwd_change_req1 = PwdChangeReq {
            pwd_change_req_id: pwd_change_req_id1.to_string(),
            email_address: "test1@test.com".to_string(),
            requested_at: current_date_time
                - Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE)
                - Duration::seconds(1),
        };
        let pwd_change_req_id2 = "a860dc5138d146ac8127b0780fbbce7g";
        let pwd_change_req2 = PwdChangeReq {
            pwd_change_req_id: pwd_change_req_id2.to_string(),
            email_address: "test1@test.com".to_string(),
            requested_at: current_date_time
                - Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE)
                - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(pwd_change_req_id1.to_string(), (pwd_change_req1, false));
        map.insert(pwd_change_req_id2.to_string(), (pwd_change_req2, false));
        map
    }

    #[tokio::test]
    async fn delete_expired_pwd_change_reqs_fail3() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 8, 5, 21, 00, 40)
            .unwrap();
        let max_num_of_target_records = 0;
        let op = DeleteExpiredPwdChangeReqOperationMock {
            pwd_change_reqs:
                create_dummy_1_failed_expired_pwd_change_req_and_1_expired_pwd_change_req(
                    current_date_time,
                ),
            current_date_time,
            limit: max_num_of_target_records,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            format!(
                "[{}] 定期実行ツール (delete_expired_pwd_change_reqs) 失敗通知",
                WEB_SITE_NAME
            ),
            vec![
                "pwd_change_reqの期限切れレコード2個の内、1個の削除に失敗しました。".to_string(),
                "b860dc5138d146ac8127b0780fabce7d".to_string(),
            ],
        );

        let result = delete_expired_pwd_change_reqs(
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

    fn create_dummy_1_failed_expired_pwd_change_req_and_1_expired_pwd_change_req(
        current_date_time: DateTime<FixedOffset>,
    ) -> HashMap<String, (PwdChangeReq, bool)> {
        let pwd_change_req_id1 = "b860dc5138d146ac8127b0780fabce7d";
        let pwd_change_req1 = PwdChangeReq {
            pwd_change_req_id: pwd_change_req_id1.to_string(),
            email_address: "test1@test.com".to_string(),
            requested_at: current_date_time
                - Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE)
                - Duration::seconds(1),
        };
        let pwd_change_req_id2 = "a860dc5138d146ac8127b0780fbbce7g";
        let pwd_change_req2 = PwdChangeReq {
            pwd_change_req_id: pwd_change_req_id2.to_string(),
            email_address: "test1@test.com".to_string(),
            requested_at: current_date_time
                - Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE)
                - Duration::seconds(1),
        };
        let mut map = HashMap::with_capacity(1);
        map.insert(pwd_change_req_id1.to_string(), (pwd_change_req1, false));
        map.insert(pwd_change_req_id2.to_string(), (pwd_change_req2, true));
        map
    }
}
