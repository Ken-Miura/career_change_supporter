// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::mfa::generate_base64_encoded_qr_code;
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

use super::super::{
    extract_first_temp_mfa_secret, filter_temp_mfa_secret_order_by_dsc, TempMfaSecret,
};
use crate::handlers::session::authentication::authenticated_handlers::authenticated_users::user::User;
use crate::handlers::session::authentication::authenticated_handlers::mfs_setting::ensure_mfa_is_not_enabled;
use crate::handlers::session::authentication::mfa::USER_TOTP_ISSUER;

pub(crate) async fn get_temp_mfa_secret(
    User { user_info }: User,
    State(pool): State<DatabaseConnection>,
) -> RespResult<GetTempMfaSecretResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = TempMfaSecretResultOperationImpl { pool };
    handle_temp_mfp_secret(
        user_info.account_id,
        user_info.mfa_enabled_at.is_some(),
        USER_TOTP_ISSUER.as_str(),
        current_date_time,
        op,
    )
    .await
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct GetTempMfaSecretResult {
    // QRコード
    base64_encoded_image: String,
    // QRコードを読み込めない場合に使うシークレットキー
    base32_encoded_secret: String,
}

async fn handle_temp_mfp_secret(
    account_id: i64,
    mfa_enabled: bool,
    issuer: &str,
    current_date_time: DateTime<FixedOffset>,
    op: impl TempMfaSecretResultOperation,
) -> RespResult<GetTempMfaSecretResult> {
    ensure_mfa_is_not_enabled(mfa_enabled)?;

    let temp_mfa_secrets = op
        .filter_temp_mfa_secret_order_by_dsc(account_id, current_date_time)
        .await?;
    // temp_mfa_secretsが登録された日付に降順でソートされているため、1つ目のエントリが最新
    let temp_mfa_secret = extract_first_temp_mfa_secret(temp_mfa_secrets)?;

    let qr_code = generate_base64_encoded_qr_code(
        account_id,
        temp_mfa_secret.base32_encoded_secret.as_str(),
        issuer,
    )?;

    Ok((
        StatusCode::OK,
        Json(GetTempMfaSecretResult {
            base64_encoded_image: qr_code,
            base32_encoded_secret: temp_mfa_secret.base32_encoded_secret,
        }),
    ))
}

#[async_trait]
trait TempMfaSecretResultOperation {
    async fn filter_temp_mfa_secret_order_by_dsc(
        &self,
        account_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<TempMfaSecret>, ErrResp>;
}

struct TempMfaSecretResultOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl TempMfaSecretResultOperation for TempMfaSecretResultOperationImpl {
    async fn filter_temp_mfa_secret_order_by_dsc(
        &self,
        account_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<TempMfaSecret>, ErrResp> {
        filter_temp_mfa_secret_order_by_dsc(account_id, current_date_time, &self.pool).await
    }
}

#[cfg(test)]
mod tests {

    use chrono::TimeZone;
    use common::ApiError;
    use once_cell::sync::Lazy;

    use crate::err::Code;

    use super::*;

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<GetTempMfaSecretResult>,
    }

    #[derive(Debug)]
    struct Input {
        account_id: i64,
        mfa_enabled: bool,
        issuer: String,
        current_date_time: DateTime<FixedOffset>,
        op: TempMfaSecretResultOperationMock,
    }

    impl Input {
        fn new(
            account_id: i64,
            mfa_enabled: bool,
            issuer: String,
            current_date_time: DateTime<FixedOffset>,
            temp_mfa_secrets: Vec<TempMfaSecret>,
        ) -> Self {
            Input {
                account_id,
                mfa_enabled,
                issuer,
                current_date_time,
                op: TempMfaSecretResultOperationMock {
                    account_id,
                    current_date_time,
                    temp_mfa_secrets,
                },
            }
        }
    }

    #[derive(Clone, Debug)]
    struct TempMfaSecretResultOperationMock {
        account_id: i64,
        current_date_time: DateTime<FixedOffset>,
        temp_mfa_secrets: Vec<TempMfaSecret>,
    }

    #[async_trait]
    impl TempMfaSecretResultOperation for TempMfaSecretResultOperationMock {
        async fn filter_temp_mfa_secret_order_by_dsc(
            &self,
            account_id: i64,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<Vec<TempMfaSecret>, ErrResp> {
            assert_eq!(self.account_id, account_id);
            assert_eq!(self.current_date_time, current_date_time);
            Ok(self.temp_mfa_secrets.clone())
        }
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        let account_id = 413;
        let mfa_enabled = false;
        let issuer = "Issuer".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 4, 5, 0, 1, 7)
            .unwrap();
        vec![TestCase {
            name: "success".to_string(),
            input: Input::new(
                account_id,
                mfa_enabled,
                issuer.clone(),
                current_date_time,
                vec![TempMfaSecret {
                    temp_mfa_secret_id: 1,
                    base32_encoded_secret: "HU7YU2643SZJMWFW5MUOMWNMHSGLA3S6".to_string(),
                }],
            ),
            expected: Ok((StatusCode::OK, Json(GetTempMfaSecretResult{
                // account_id, issuer, base32_encoded_secretから実際に生成されたものを利用 
                base64_encoded_image: "iVBORw0KGgoAAAANSUhEUgAAAWgAAAFoCAAAAABfjj4JAAAL9ElEQVR4Ae3gAZAkSZIkSRKLqpm7R0REZmZmVlVVVVV3d3d3d/fMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMdHd3d3dXV1VVVVVmZkZGRIS7m5kKz0xmV3d1d3dPz8zMzMxMYjVX/RegctV/BSpX/VegctV/BSpX/VegctV/BSpX/VegctV/BSpX/VegctV/BSpX/VegctV/BSpX/VegctV/BSpX/VegctV/BSpX/VegctV/BSpX/VegctV/BSpX/VegctV/BSpX/VegctV/BSpX/VegctV/BSpX/VegctV/BSpX/VegctV/BSpX/VegctV/BSr/AvFvY64QL5x5TuIK85zEFeaFE8/JvHDiCnOF+LcxLxSVq/4rULnqvwKVq/4rULnqvwKVq/4rULnqvwKVq/4rUHkRmReNeE7mCnGFeeHM82euEM/JXCGuMM+feE7mCvP8mReNeJFQueq/ApWr/itQueq/ApWr/itQueq/ApWr/itQueq/ApV/JfH8mRfOXCGuMM+feOHMFeKFE1eYK8wV4grxnMzzJ54/869C5ar/ClSu+q9A5ar/ClSu+q9A5ar/ClSu+q9A5ar/ClT+k4nnZK4QV5grxBXmCnGFuUJcIa4wz8lcIa4wV4jnz1wh/ktQueq/ApWr/itQueq/ApWr/itQueq/ApWr/itQueq/ApX/IuY5mSvEFeaFM89JXGGeP/H8mf8WVK76r0Dlqv8KVK76r0Dlqv8KVK76r0Dlqv8KVK76r0DlX8n8+4jnT1xhrjBXiCvMCyeuMC+cuMK8aMx/CCpX/VegctV/BSpX/VegctV/BSpX/VegctV/BSpX/Veg8iIS/7HMFeIKc4W4wjx/4gpzhbjCXCGuMFeIK8y/jvgPReWq/wpUrvqvQOWq/wpUrvqvQOWq/wpUrvqvQOWq/wpU/gXmP5e5Qrxw4oUTV5grxBXmhTPPyfynoHLVfwUqV/1XoHLVfwUqV/1XoHLVfwUqV/1XoHLVfwVkXjhxhXn+xBXmCvHCmSvEczLPSVxhrhBXmOdPPCdzhXhO5gpxhblCvGjMcxJXmBeKylX/Fahc9V+BylX/Fahc9V+BylX/Fahc9V+BylX/Faj8K4nnZJ6TuUK8aMzzZ64QV5grxBXmhRNXmCvEFeIKc4V4/swV4gpzhXhO5kVC5ar/ClSu+q9A5ar/ClSu+q9A5ar/ClSu+q9A5ar/ClReROL5Ey+ceU7iCvOcxHMyz0k8f+IK85zE82euEFeY5ySuEFeYK8QV5t+EylX/Fahc9V+BylX/Fahc9V+BylX/Fahc9V+BylX/FZB54cQV5gpxhblCXGGek3jRmCvEFeb5E8+fuUJcYZ6TuMJcIZ6TeU7i+TNXiOdkXiRUrvqvQOWq/wpUrvqvQOWq/wpUrvqvQOWq/wpUrvqvQOVFJF404vkzz0lcIa4wz0n824grzBXmOZkrxIvGPCdzhbhCXGFeKCpX/VegctV/BSpX/VegctV/BSpX/VegctV/BSpX/Veg8i8wz5+4wlwhrjDPSTwn8ZzMcxLPybxw4gpzhbhCPH/mhTNXiCvEFeY5mSvEi4TKVf8VqFz1X4HKVf8VqFz1X4HKVf8VqFz1X4HKVf8VkHnhxHMyL5y4wlwhrjDPSVxhnj/xwpnnTzwnc4V44czzJ54/869C5ar/ClSu+q9A5ar/ClSu+q9A5ar/ClSu+q9A5ar/ClReROY5iSvMczJXiCvMFeIK88KJ52SuEM+feE7mCnGFuMJcIf5tzBXiCvGczAtF5ar/ClSu+q9A5ar/ClSu+q9A5ar/ClSu+q9A5ar/ClT+BeYKcYW5wjwn8fyJK8wLJ56TuUI8J3OFuMJcIa4QV5grxBXiCvP8iX8d869C5ar/ClSu+q9A5ar/ClSu+q9A5ar/ClSu+q9A5ar/Csi8cOL5M1eI52ReOPGczBXiCvPvI1405gpxhblCXGGuEC+ceZFQueq/ApWr/itQueq/ApWr/itQueq/ApWr/itQueq/ApV/gXnhzHMSz595TuYKcYW5Qjwn85zEFeYKcYV5/swV4gpzhXjhxBXmOYkrzL8Klav+K1C56r8Clav+K1C56r8Clav+K1C56r8Clav+K1B5EYnnZJ6TuMJcIZ6TuMI8J3OFeP7EC2euEFeY5ySuMFeIK8x/DHGFeaGoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcpV/xWoXPVfAZkXTjx/5grx72Oek7jCXCGuMC+ceE7mhRMvnHlO4gpzhbjCvEioXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcq/krlCPCfz/InnZK4Qz0k8J3GFedGY5ySek3nRmCvEczLPn7jCvFBUrvqvQOWq/wpUrvqvQOWq/wpUrvqvQOWq/wpUrvqvgMy/jrjCXCFeOPP8iX8d8/yJ52SuEFeY5ySeP3OFuMI8f+IK869C5ar/ClSu+q9A5ar/ClSu+q9A5ar/ClSu+q9A5ar/ClT+BeI5mSvE82euEFeIF425QlxhnpO4wrxozPNnrhBXmBdOXGGeP3GFeaGoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcqLyDwn8/yJK8xzEs/J/OuYK8QLJ56TuUJcYa4wz0k8f+YK8e9C5ar/ClSu+q9A5ar/ClSu+q9A5ar/ClSu+q9A5ar/ClReROIK8/yJK8wV4grznMxzEleY50+8cOb5E1eI5ySuMFeIK8wV4grx/JnnZF4kVK76r0Dlqv8KVK76r0Dlqv8KVK76r0Dlqv8KVK76r4DMv494/swV4gpzhXj+zPMnnpN5TuI5mSvEczJXiOdknj/xnMy/C5Wr/itQueq/ApWr/itQueq/ApWr/itQueq/ApWr/itQ+TcSV5grxBXmCvGcxAsnrjDPyVwhrhBXmBfOvHDmOYkrzHMyz0lcYa4QV5gXispV/xWoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcpV/xWQeeHEczJXiCvMFeKFM89JPCdzhXj+zPMnnj/zohEvGvOcxBXmRULlqv8KVK76r0Dlqv8KVK76r0Dlqv8KVK76r0Dlqv8KyPznEleY5ySuMFeIK8wV4gpzhbjCvGjEv455/sRzMleI52ReKCpX/VegctV/BSpX/VegctV/BSpX/VegctV/BSpX/Veg8i8Q/zbmOYnnZF404gpzhbjCPCdxhXlO5oUTz0lcYZ6TuUJcYf5VqFz1X4HKVf8VqFz1X4HKVf8VqFz1X4HKVf8VqFz1X4HKi8i8aMRzMleIF85cIa4wV4jnZK4QV5jnzzx/4grz/Jl/HXGFeaGoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcq/knj+zPMnrjBXiOck/mOJK8y/jnjhxL8Llav+K1C56r8Clav+K1C56r8Clav+K1C56r8Clav+K1D5T2ZeNOb5M89JXGGuEFeYK8TzJ64wV4gXjblCXGH+Tahc9V+BylX/Fahc9V+BylX/Fahc9V+BylX/Fahc9V+Byn8R8fyZ509cYa4Qz595/sRzMleIK8xzEv825kVC5ar/ClSu+q9A5ar/ClSu+q9A5ar/ClSu+q9A5ar/ClT+lcy/jblCXCGek7lCPCfx/IkrzBXiCvOvI64wV4grzHMyV4jnZF4kVK76r0Dlqv8KVK76r0Dlqv8KVK76r0Dlqv8KVK76r0DlRST+bcQV5vkzV4grzHMS/zriCvP8mSvEFeYK8fyJ52SuEFeIK8wLReWq/wpUrvqvQOWq/wpUrvqvQOWq/wpUrvqvQOWq/wrIXPVfgMpV/xWoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcpV/xWoXPVfgcpV/xX4R0x+TX6Z/07MAAAAAElFTkSuQmCC".to_string(), 
                base32_encoded_secret: "HU7YU2643SZJMWFW5MUOMWNMHSGLA3S6".to_string() }))),
        },
        TestCase {
            name: "fail MfaHasAlreadyBeenEnabled".to_string(),
            input: Input::new(
                account_id,
                true,
                issuer.clone(),
                current_date_time,
                vec![TempMfaSecret {
                    temp_mfa_secret_id: 1,
                    base32_encoded_secret: "HU7YU2643SZJMWFW5MUOMWNMHSGLA3S6".to_string(),
                }],
            ),
            expected: Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::MfaHasAlreadyBeenEnabled as u32,
                }),
            )),
        },
        TestCase {
            name: "fail NoTempMfaSecretFound".to_string(),
            input: Input::new(
                account_id,
                mfa_enabled,
                issuer,
                current_date_time,
                vec![],
            ),
            expected: Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::NoTempMfaSecretFound as u32,
                }),
            )),
        }]
    });

    #[tokio::test]
    async fn handle_temp_mfp_secret_tests() {
        for test_case in TEST_CASE_SET.iter() {
            let account_id = test_case.input.account_id;
            let mfa_enabled = test_case.input.mfa_enabled;
            let issuer = test_case.input.issuer.clone();
            let current_date_time = test_case.input.current_date_time;
            let op = test_case.input.op.clone();

            let result = handle_temp_mfp_secret(
                account_id,
                mfa_enabled,
                issuer.as_str(),
                current_date_time,
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
