// Copyright 2021 Ken Miura

use axum::async_trait;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{Datelike, NaiveDate};
use common::util::Ymd;
use common::{ApiError, ErrResp, RespResult};
use entity::identity;
use entity::prelude::Identity;
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::unexpected_err_resp;
use crate::err::Code::IllegalDate;
use crate::handlers::session::authentication::authenticated_handlers::admin::Admin;
use crate::handlers::session::authentication::authenticated_handlers::user_account_operation::{
    FindUserAccountInfoOperation, FindUserAccountInfoOperationImpl, UserAccountInfo,
};

pub(crate) async fn get_users_by_date_of_birth(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<DateOfBirth>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<Vec<User>> {
    let query = query.0;
    let op = UsersByDateOfBirthOperationImpl { pool };
    get_users_by_date_of_birth_internal(query.year, query.month, query.day, op).await
}

#[derive(Deserialize)]
pub(crate) struct DateOfBirth {
    year: i32,
    month: u32,
    day: u32,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
struct UserWithoutAccountStatus {
    user_account_id: i64,
    last_name: String,
    first_name: String,
    last_name_furigana: String,
    first_name_furigana: String,
    date_of_birth: Ymd,
    prefecture: String,
    city: String,
    address_line1: String,
    address_line2: Option<String>,
    telephone_number: String,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct User {
    user_account_id: i64,
    last_name: String,
    first_name: String,
    last_name_furigana: String,
    first_name_furigana: String,
    date_of_birth: Ymd,
    prefecture: String,
    city: String,
    address_line1: String,
    address_line2: Option<String>,
    telephone_number: String,
    account_status: AccountStatus,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
enum AccountStatus {
    Enabled,
    Disabled,
    Deleted,
}

async fn get_users_by_date_of_birth_internal(
    year: i32,
    month: u32,
    day: u32,
    op: impl UsersByDateOfBirthOperation,
) -> RespResult<Vec<User>> {
    let date_of_birth_option = NaiveDate::from_ymd_opt(year, month, day);
    let date_of_birth = date_of_birth_option.ok_or({
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: IllegalDate as u32,
            }),
        )
    })?;

    let users_without_account_status = op.get_users_by_date_of_birth(date_of_birth).await?;

    let mut users = Vec::with_capacity(users_without_account_status.capacity());
    // NOTE:
    //   同じ生年月日のユーザーは少ないと考え、forでSQLクエリを繰り返す。
    //   運用中、性能問題が発生した場合（同じ生年月日のユーザーが多数いるケースが珍しくないと判明した場合）、JOINを使い一度のクエリで取得できるように書き直す。
    for user_without_account_status in users_without_account_status {
        let user_info = op
            .find_user_info_by_user_account_id(user_without_account_status.user_account_id)
            .await?;
        let user = create_user(user_info, user_without_account_status);
        users.push(user);
    }

    Ok((StatusCode::OK, Json(users)))
}

fn create_user(
    user_info: Option<UserAccountInfo>,
    user_without_account_status: UserWithoutAccountStatus,
) -> User {
    let account_status = match user_info {
        Some(ui) => {
            if ui.disabled_at.is_some() {
                AccountStatus::Disabled
            } else {
                AccountStatus::Enabled
            }
        }
        None => AccountStatus::Deleted,
    };
    User {
        user_account_id: user_without_account_status.user_account_id,
        last_name: user_without_account_status.last_name,
        first_name: user_without_account_status.first_name,
        last_name_furigana: user_without_account_status.last_name_furigana,
        first_name_furigana: user_without_account_status.first_name_furigana,
        date_of_birth: user_without_account_status.date_of_birth,
        prefecture: user_without_account_status.prefecture,
        city: user_without_account_status.city,
        address_line1: user_without_account_status.address_line1,
        address_line2: user_without_account_status.address_line2,
        telephone_number: user_without_account_status.telephone_number,
        account_status,
    }
}

#[async_trait]
trait UsersByDateOfBirthOperation {
    async fn get_users_by_date_of_birth(
        &self,
        date_of_birth: NaiveDate,
    ) -> Result<Vec<UserWithoutAccountStatus>, ErrResp>;

    async fn find_user_info_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Option<UserAccountInfo>, ErrResp>;
}

struct UsersByDateOfBirthOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl UsersByDateOfBirthOperation for UsersByDateOfBirthOperationImpl {
    async fn get_users_by_date_of_birth(
        &self,
        date_of_birth: NaiveDate,
    ) -> Result<Vec<UserWithoutAccountStatus>, ErrResp> {
        let models = Identity::find()
            .filter(identity::Column::DateOfBirth.eq(date_of_birth))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter identity (date_of_birth: {}): {}",
                    date_of_birth, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| UserWithoutAccountStatus {
                user_account_id: m.user_account_id,
                last_name: m.last_name,
                first_name: m.first_name,
                last_name_furigana: m.last_name_furigana,
                first_name_furigana: m.first_name_furigana,
                date_of_birth: Ymd {
                    year: m.date_of_birth.year(),
                    month: m.date_of_birth.month(),
                    day: m.date_of_birth.day(),
                },
                prefecture: m.prefecture,
                city: m.city,
                address_line1: m.address_line1,
                address_line2: m.address_line2,
                telephone_number: m.telephone_number,
            })
            .collect::<Vec<UserWithoutAccountStatus>>())
    }

    async fn find_user_info_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Option<UserAccountInfo>, ErrResp> {
        let op = FindUserAccountInfoOperationImpl::new(&self.pool);
        op.find_user_account_info_by_account_id(user_account_id)
            .await
    }
}

// ロジックはDBへのクエリのみでテストは必要ないかもしれないが、
// DBへのクエリ（ORMのAPI）への期待する動作を記すためにテストを記載しておく。
#[cfg(test)]
mod tests {
    use crate::err::Code::IllegalDate;
    use axum::async_trait;
    use axum::http::StatusCode;
    use chrono::{Datelike, NaiveDate, TimeZone};
    use common::{util::Ymd, ErrResp, JAPANESE_TIME_ZONE};

    use super::*;

    struct UsersByDateOfBirthOperationMock {
        users: Vec<User>,
    }

    #[async_trait]
    impl UsersByDateOfBirthOperation for UsersByDateOfBirthOperationMock {
        async fn get_users_by_date_of_birth(
            &self,
            date_of_birth: NaiveDate,
        ) -> Result<Vec<UserWithoutAccountStatus>, ErrResp> {
            let ymd = Ymd {
                year: date_of_birth.year(),
                month: date_of_birth.month(),
                day: date_of_birth.day(),
            };
            let mut users_without_account_status = Vec::with_capacity(self.users.capacity());
            for user in &self.users {
                if user.date_of_birth != ymd {
                    continue;
                }
                let user_without_account_status = UserWithoutAccountStatus {
                    user_account_id: user.user_account_id,
                    last_name: user.last_name.clone(),
                    first_name: user.first_name.clone(),
                    last_name_furigana: user.last_name_furigana.clone(),
                    first_name_furigana: user.first_name_furigana.clone(),
                    date_of_birth: user.date_of_birth.clone(),
                    prefecture: user.prefecture.clone(),
                    city: user.city.clone(),
                    address_line1: user.address_line1.clone(),
                    address_line2: user.address_line2.clone(),
                    telephone_number: user.telephone_number.clone(),
                };
                users_without_account_status.push(user_without_account_status);
            }
            Ok(users_without_account_status)
        }

        async fn find_user_info_by_user_account_id(
            &self,
            user_account_id: i64,
        ) -> Result<Option<UserAccountInfo>, ErrResp> {
            for user in &self.users {
                if user.user_account_id == user_account_id {
                    let account_status = user.account_status.clone();
                    let disabled_at = match account_status {
                        AccountStatus::Enabled => None,
                        AccountStatus::Disabled => Some(
                            JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 4, 5, 0, 1, 7) // Someで返せばよいので日時はダミー値
                                .unwrap(),
                        ),
                        AccountStatus::Deleted => return Ok(None),
                    };
                    return Ok(Some(UserAccountInfo {
                        account_id: user.user_account_id,
                        email_address: "test@test.com".to_string(), // 利用しないのでリテラルでダミー値をセット
                        last_login_time: None, // 利用しないのでリテラルでダミー値をセット
                        created_at: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 4, 1, 0, 1, 7) // 利用しないのでダミー値をセット
                            .unwrap(),
                        mfa_enabled_at: None, // 利用しないのでリテラルでダミー値をセット
                        disabled_at,
                    }));
                }
            }
            panic!("never reach here!")
        }
    }

    #[tokio::test]
    async fn get_users_by_date_of_birth_internal_success_one_user_found() {
        let date_of_birth = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(1991, 4, 1, 0, 0, 0)
            .unwrap()
            .date_naive();
        let user = create_dummy_user1(date_of_birth);
        let op_mock = UsersByDateOfBirthOperationMock {
            users: vec![user.clone()],
        };

        let result = get_users_by_date_of_birth_internal(
            date_of_birth.year(),
            date_of_birth.month(),
            date_of_birth.day(),
            op_mock,
        )
        .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(vec![user], resp.1 .0);
    }

    #[tokio::test]
    async fn get_users_by_date_of_birth_internal_success_one_disabled_user_found() {
        let date_of_birth = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(1991, 4, 1, 0, 0, 0)
            .unwrap()
            .date_naive();
        let user = create_dummy_user4(date_of_birth);
        let op_mock = UsersByDateOfBirthOperationMock {
            users: vec![user.clone()],
        };

        let result = get_users_by_date_of_birth_internal(
            date_of_birth.year(),
            date_of_birth.month(),
            date_of_birth.day(),
            op_mock,
        )
        .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(vec![user], resp.1 .0);
    }

    #[tokio::test]
    async fn get_users_by_date_of_birth_internal_success_one_deleted_user_found() {
        let date_of_birth = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(1991, 4, 1, 0, 0, 0)
            .unwrap()
            .date_naive();
        let user = create_dummy_user5(date_of_birth);
        let op_mock = UsersByDateOfBirthOperationMock {
            users: vec![user.clone()],
        };

        let result = get_users_by_date_of_birth_internal(
            date_of_birth.year(),
            date_of_birth.month(),
            date_of_birth.day(),
            op_mock,
        )
        .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(vec![user], resp.1 .0);
    }

    #[tokio::test]
    async fn get_users_by_date_of_birth_internal_success_no_user_found() {
        let date_of_birth = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(1991, 4, 1, 0, 0, 0)
            .unwrap()
            .date_naive();
        let user = create_dummy_user1(date_of_birth);
        let op_mock = UsersByDateOfBirthOperationMock {
            users: vec![user.clone()],
        };

        let result = get_users_by_date_of_birth_internal(
            date_of_birth.year() + 1,
            date_of_birth.month(),
            date_of_birth.day(),
            op_mock,
        )
        .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(Vec::<User>::with_capacity(0), resp.1 .0);
    }

    #[tokio::test]
    async fn get_users_by_date_of_birth_internal_success_one_user_found_one_user_not_found() {
        let date_of_birth1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(1991, 4, 1, 0, 0, 0)
            .unwrap()
            .date_naive();
        let user1 = create_dummy_user1(date_of_birth1);
        let date_of_birth2 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(1991, date_of_birth1.month() + 1, 1, 0, 0, 0)
            .unwrap()
            .date_naive();
        let user2 = create_dummy_user2(date_of_birth2);
        let op_mock = UsersByDateOfBirthOperationMock {
            users: vec![user1.clone(), user2],
        };

        let result = get_users_by_date_of_birth_internal(
            date_of_birth1.year(),
            date_of_birth1.month(),
            date_of_birth1.day(),
            op_mock,
        )
        .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(vec![user1], resp.1 .0);
    }

    #[tokio::test]
    async fn get_users_by_date_of_birth_internal_success_multiple_users_found() {
        let date_of_birth1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(1991, 4, 1, 0, 0, 0)
            .unwrap()
            .date_naive();
        let user1 = create_dummy_user1(date_of_birth1);
        let date_of_birth2 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(1991, date_of_birth1.month() + 1, 1, 0, 0, 0)
            .unwrap()
            .date_naive();
        let user2 = create_dummy_user2(date_of_birth2);
        let date_of_birth3 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(
                date_of_birth1.year(),
                date_of_birth1.month(),
                date_of_birth1.day(),
                0,
                0,
                0,
            )
            .unwrap()
            .date_naive();
        let user3 = create_dummy_user3(date_of_birth3);
        let op_mock = UsersByDateOfBirthOperationMock {
            users: vec![user1.clone(), user2, user3.clone()],
        };

        let result = get_users_by_date_of_birth_internal(
            date_of_birth1.year(),
            date_of_birth1.month(),
            date_of_birth1.day(),
            op_mock,
        )
        .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(vec![user1, user3], resp.1 .0);
    }

    #[tokio::test]
    async fn get_users_by_date_of_birth_internal_fail_illegal_date() {
        let op_mock = UsersByDateOfBirthOperationMock { users: vec![] };

        let result = get_users_by_date_of_birth_internal(1990, 1, 0, op_mock).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(IllegalDate as u32, err_resp.1 .0.code);
    }

    fn create_dummy_user1(date_of_birth: NaiveDate) -> User {
        let ymd = Ymd {
            year: date_of_birth.year(),
            month: date_of_birth.month(),
            day: date_of_birth.day(),
        };
        User {
            user_account_id: 341,
            last_name: String::from("田中"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("タナカ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: ymd,
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            telephone_number: String::from("09012345678"),
            account_status: AccountStatus::Enabled,
        }
    }

    fn create_dummy_user2(date_of_birth: NaiveDate) -> User {
        let ymd = Ymd {
            year: date_of_birth.year(),
            month: date_of_birth.month(),
            day: date_of_birth.day(),
        };
        User {
            user_account_id: 567,
            last_name: String::from("佐藤"),
            first_name: String::from("次郎"),
            last_name_furigana: String::from("サトウ"),
            first_name_furigana: String::from("次郎"),
            date_of_birth: ymd,
            prefecture: String::from("沖縄県"),
            city: String::from("那覇市"),
            address_line1: String::from("泉崎１ー１ー１"),
            address_line2: None,
            telephone_number: String::from("08012345678"),
            account_status: AccountStatus::Enabled,
        }
    }

    fn create_dummy_user3(date_of_birth: NaiveDate) -> User {
        let ymd = Ymd {
            year: date_of_birth.year(),
            month: date_of_birth.month(),
            day: date_of_birth.day(),
        };
        User {
            user_account_id: 5767,
            last_name: String::from("田中"),
            first_name: String::from("三郎"),
            last_name_furigana: String::from("タナカ"),
            first_name_furigana: String::from("サブロウ"),
            date_of_birth: ymd,
            prefecture: String::from("北海道"),
            city: String::from("札幌市"),
            address_line1: String::from("中央区北1条西2丁目"),
            address_line2: None,
            telephone_number: String::from("08087654321"),
            account_status: AccountStatus::Enabled,
        }
    }

    fn create_dummy_user4(date_of_birth: NaiveDate) -> User {
        let ymd = Ymd {
            year: date_of_birth.year(),
            month: date_of_birth.month(),
            day: date_of_birth.day(),
        };
        User {
            user_account_id: 341,
            last_name: String::from("田中"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("タナカ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: ymd,
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            telephone_number: String::from("09012345678"),
            account_status: AccountStatus::Disabled,
        }
    }

    fn create_dummy_user5(date_of_birth: NaiveDate) -> User {
        let ymd = Ymd {
            year: date_of_birth.year(),
            month: date_of_birth.month(),
            day: date_of_birth.day(),
        };
        User {
            user_account_id: 341,
            last_name: String::from("田中"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("タナカ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: ymd,
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            telephone_number: String::from("09012345678"),
            account_status: AccountStatus::Deleted,
        }
    }
}
