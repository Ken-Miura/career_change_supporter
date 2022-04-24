// Copyright 2021 Ken Miura

use axum::async_trait;
use axum::{
    extract::{Extension, Query},
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
use crate::util::session::Admin;

pub(crate) async fn get_users_by_date_of_birth(
    Admin { account_id: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<DateOfBirth>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<Vec<User>> {
    let query = query.0;
    let op = UsersByDateOfBirthOperationImpl { pool };
    get_users_by_date_of_birth_internal(query.year, query.month, query.day, op).await
}

#[derive(Deserialize)]
pub(crate) struct DateOfBirth {
    pub(crate) year: i32,
    pub(crate) month: u32,
    pub(crate) day: u32,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct User {
    pub(crate) user_account_id: i64,
    pub(crate) last_name: String,
    pub(crate) first_name: String,
    pub(crate) last_name_furigana: String,
    pub(crate) first_name_furigana: String,
    pub(crate) date_of_birth: Ymd,
    pub(crate) prefecture: String,
    pub(crate) city: String,
    pub(crate) address_line1: String,
    pub(crate) address_line2: Option<String>,
    pub(crate) telephone_number: String,
}

async fn get_users_by_date_of_birth_internal(
    year: i32,
    month: u32,
    day: u32,
    op: impl UsersByDateOfBirthOperation,
) -> RespResult<Vec<User>> {
    let date_of_birth_option = NaiveDate::from_ymd_opt(year, month, day);
    let date_of_birth = date_of_birth_option.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: IllegalDate as u32,
            }),
        )
    })?;
    let users = op.get_users_by_date_of_birth(date_of_birth).await?;
    Ok((StatusCode::OK, Json(users)))
}

#[async_trait]
trait UsersByDateOfBirthOperation {
    async fn get_users_by_date_of_birth(
        &self,
        date_of_birth: NaiveDate,
    ) -> Result<Vec<User>, ErrResp>;
}

struct UsersByDateOfBirthOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl UsersByDateOfBirthOperation for UsersByDateOfBirthOperationImpl {
    async fn get_users_by_date_of_birth(
        &self,
        date_of_birth: NaiveDate,
    ) -> Result<Vec<User>, ErrResp> {
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
            .map(|m| User {
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
            .collect::<Vec<User>>())
    }
}

// ロジックはDBへのクエリのみでテストは必要ないかもしれないが、
// DBへのクエリ（ORMのAPI）への期待する動作を記すためにテストを記載しておく。
#[cfg(test)]
mod tests {
    use crate::err::Code::IllegalDate;
    use axum::async_trait;
    use axum::http::StatusCode;
    use chrono::{Datelike, NaiveDate, TimeZone, Utc};
    use common::{util::Ymd, ErrResp};

    use super::{get_users_by_date_of_birth_internal, User, UsersByDateOfBirthOperation};

    struct UsersByDateOfBirthOperationMock {
        users: Vec<User>,
    }

    #[async_trait]
    impl UsersByDateOfBirthOperation for UsersByDateOfBirthOperationMock {
        async fn get_users_by_date_of_birth(
            &self,
            date_of_birth: NaiveDate,
        ) -> Result<Vec<User>, ErrResp> {
            let mut users = Vec::with_capacity(self.users.len());
            for user in &self.users {
                let ymd = Ymd {
                    year: date_of_birth.year(),
                    month: date_of_birth.month(),
                    day: date_of_birth.day(),
                };
                if user.date_of_birth == ymd {
                    users.push(user.clone());
                }
            }
            Ok(users)
        }
    }

    #[tokio::test]
    async fn get_users_by_date_of_birth_internal_success_one_user_found() {
        let date_of_birth = Utc.ymd(1991, 4, 1).naive_local();
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
    async fn get_users_by_date_of_birth_internal_success_no_user_found() {
        let date_of_birth = Utc.ymd(1991, 4, 1).naive_local();
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
        let date_of_birth1 = Utc.ymd(1991, 4, 1).naive_local();
        let user1 = create_dummy_user1(date_of_birth1);
        let date_of_birth2 = Utc.ymd(1991, date_of_birth1.month() + 1, 1).naive_local();
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
        let date_of_birth1 = Utc.ymd(1991, 4, 1).naive_local();
        let user1 = create_dummy_user1(date_of_birth1);
        let date_of_birth2 = Utc.ymd(1991, date_of_birth1.month() + 1, 1).naive_local();
        let user2 = create_dummy_user2(date_of_birth2);
        let date_of_birth3 = Utc
            .ymd(
                date_of_birth1.year(),
                date_of_birth1.month(),
                date_of_birth1.day(),
            )
            .naive_local();
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
        }
    }
}
