// Copyright 2021 Ken Miura

use axum::{http::StatusCode, Json};
use chrono::Datelike;
use common::{
    model::user::{Account, CareerInfo, ConsultingFee, IdentityInfo},
    schema::ccs_schema::{
        career_info::{dsl::career_info as career_info_table, user_account_id},
        consulting_fee::dsl::consulting_fee as consulting_fee_table,
        identity_info::dsl::identity_info as identity_info_table,
        user_account::dsl::user_account,
    },
    ApiError, DatabaseConnection, ErrResp, RespResult, MAX_NUM_OF_CAREER_INFO_PER_USER_ACCOUNT,
};
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    result::Error::NotFound,
    ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl,
};
use serde::Serialize;

use crate::{
    err_code::NO_ACCOUNT_FOUND,
    util::{session::User, unexpected_err_resp, Career, Identity, Ymd},
};

pub(crate) async fn get_profile(
    User { account_id }: User,
    DatabaseConnection(conn): DatabaseConnection,
) -> RespResult<ProfileResult> {
    let profile_op = ProfileOperationImpl::new(conn);
    get_profile_internal(account_id, profile_op).await
}

async fn get_profile_internal(
    account_id: i32,
    profile_op: impl ProfileOperation,
) -> RespResult<ProfileResult> {
    let account = profile_op.find_user_account_by_user_account_id(account_id)?;
    let identity_info_option = profile_op.find_identity_info_by_user_account_id(account_id)?;
    if identity_info_option.is_none() {
        return Ok((
            StatusCode::OK,
            Json(ProfileResult::email_address(account.email_address).finish()),
        ));
    };
    let identity = identity_info_option.map(convert_identity_info_to_identity);
    let careers_info = profile_op.filter_career_info_by_user_account_id(account_id)?;
    let careers = careers_info
        .into_iter()
        .map(convert_career_info_to_career)
        .collect::<Vec<Career>>();
    let consulting_fee_option = profile_op.find_consulting_fee_by_user_account_id(account_id)?;
    let fee_per_hour_in_yen = consulting_fee_option.map(|c| c.fee_per_hour_in_yen);
    Ok((
        StatusCode::OK,
        Json(
            ProfileResult::email_address(account.email_address)
                .identity(identity)
                .careers(careers)
                .fee_per_hour_in_yen(fee_per_hour_in_yen)
                .finish(),
        ),
    ))
}

#[derive(Serialize, Debug)]
pub(crate) struct ProfileResult {
    pub email_address: String,
    pub identity: Option<Identity>,
    pub careers: Vec<Career>,
    pub fee_per_hour_in_yen: Option<i32>,
}

impl ProfileResult {
    fn email_address(email_address: String) -> ProfileResultBuilder {
        ProfileResultBuilder {
            email_address,
            identity: None,
            careers: vec![],
            fee_per_hour_in_yen: None,
        }
    }
}

struct ProfileResultBuilder {
    email_address: String,
    identity: Option<Identity>,
    careers: Vec<Career>,
    fee_per_hour_in_yen: Option<i32>,
}

impl ProfileResultBuilder {
    fn identity(mut self, identity: Option<Identity>) -> ProfileResultBuilder {
        self.identity = identity;
        self
    }

    fn careers(mut self, careers: Vec<Career>) -> ProfileResultBuilder {
        self.careers = careers;
        self
    }

    fn fee_per_hour_in_yen(mut self, fee_per_hour_in_yen: Option<i32>) -> ProfileResultBuilder {
        self.fee_per_hour_in_yen = fee_per_hour_in_yen;
        self
    }

    fn finish(self) -> ProfileResult {
        ProfileResult {
            email_address: self.email_address,
            identity: self.identity,
            careers: self.careers,
            fee_per_hour_in_yen: self.fee_per_hour_in_yen,
        }
    }
}

fn convert_identity_info_to_identity(identity_info: IdentityInfo) -> Identity {
    let date = identity_info.date_of_birth;
    let ymd = Ymd {
        year: date.year(),
        month: date.month(),
        day: date.day(),
    };
    Identity {
        last_name: identity_info.last_name,
        first_name: identity_info.first_name,
        last_name_furigana: identity_info.last_name_furigana,
        first_name_furigana: identity_info.first_name_furigana,
        sex: identity_info.sex,
        date_of_birth: ymd,
        prefecture: identity_info.prefecture,
        city: identity_info.city,
        address_line1: identity_info.address_line1,
        address_line2: identity_info.address_line2,
        telephone_number: identity_info.telephone_number,
    }
}

fn convert_career_info_to_career(career_info: CareerInfo) -> Career {
    let career_start_date = Ymd {
        year: career_info.career_start_date.year(),
        month: career_info.career_start_date.month(),
        day: career_info.career_start_date.day(),
    };
    let career_end_date = career_info.career_end_date.map(|end_date| Ymd {
        year: end_date.year(),
        month: end_date.month(),
        day: end_date.day(),
    });
    Career {
        id: career_info.career_info_id,
        company_name: career_info.company_name,
        department_name: career_info.department_name,
        office: career_info.office,
        career_start_date,
        career_end_date,
        contract_type: career_info.contract_type,
        profession: career_info.profession,
        annual_income_in_man_yen: career_info.annual_income_in_man_yen,
        is_manager: career_info.is_manager,
        position_name: career_info.position_name,
        is_new_graduate: career_info.is_new_graduate,
        note: career_info.note,
    }
}

trait ProfileOperation {
    fn find_user_account_by_user_account_id(&self, id: i32) -> Result<Account, ErrResp>;
    fn find_identity_info_by_user_account_id(
        &self,
        id: i32,
    ) -> Result<Option<IdentityInfo>, ErrResp>;
    fn filter_career_info_by_user_account_id(&self, id: i32) -> Result<Vec<CareerInfo>, ErrResp>;
    fn find_consulting_fee_by_user_account_id(
        &self,
        id: i32,
    ) -> Result<Option<ConsultingFee>, ErrResp>;
}

struct ProfileOperationImpl {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
}

impl ProfileOperationImpl {
    fn new(conn: PooledConnection<ConnectionManager<PgConnection>>) -> Self {
        Self { conn }
    }
}

impl ProfileOperation for ProfileOperationImpl {
    fn find_user_account_by_user_account_id(&self, id: i32) -> Result<Account, ErrResp> {
        let result = user_account.find(id).first::<Account>(&self.conn);
        match result {
            Ok(account) => Ok(account),
            Err(e) => {
                if e == NotFound {
                    Err((
                        StatusCode::BAD_REQUEST,
                        Json(ApiError {
                            code: NO_ACCOUNT_FOUND,
                        }),
                    ))
                } else {
                    Err(unexpected_err_resp())
                }
            }
        }
    }

    fn find_identity_info_by_user_account_id(
        &self,
        id: i32,
    ) -> Result<Option<IdentityInfo>, ErrResp> {
        let result = identity_info_table
            .find(id)
            .first::<IdentityInfo>(&self.conn);
        match result {
            Ok(identity_info) => Ok(Some(identity_info)),
            Err(e) => {
                if e == NotFound {
                    Ok(None)
                } else {
                    Err(unexpected_err_resp())
                }
            }
        }
    }

    fn filter_career_info_by_user_account_id(&self, id: i32) -> Result<Vec<CareerInfo>, ErrResp> {
        let result = career_info_table
            .filter(user_account_id.eq(id))
            .limit(MAX_NUM_OF_CAREER_INFO_PER_USER_ACCOUNT)
            .load::<CareerInfo>(&self.conn)
            .map_err(|e| {
                tracing::error!("failed to filter career info by id {}: {}", id, e);
                unexpected_err_resp()
            })?;
        Ok(result)
    }

    fn find_consulting_fee_by_user_account_id(
        &self,
        id: i32,
    ) -> Result<Option<ConsultingFee>, ErrResp> {
        let result = consulting_fee_table
            .find(id)
            .first::<ConsultingFee>(&self.conn);
        match result {
            Ok(consulting_fee) => Ok(Some(consulting_fee)),
            Err(e) => {
                if e == NotFound {
                    Ok(None)
                } else {
                    Err(unexpected_err_resp())
                }
            }
        }
    }
}

// TODO: 事前準備に用意するデータ (IdentitiInfo、CareerInfo、ConsultingFee) に関して、データの追加、編集でvalidatorを実装した後、それを使ってチェックを行うよう修正する
#[cfg(test)]
mod tests {
    use axum::{http::StatusCode, Json};
    use chrono::{NaiveDate, TimeZone, Utc};
    use common::{
        model::user::{Account, CareerInfo, ConsultingFee, IdentityInfo},
        util::hash_password,
        ApiError, MAX_NUM_OF_CAREER_INFO_PER_USER_ACCOUNT,
    };

    use crate::{
        err_code::{self, NO_ACCOUNT_FOUND},
        profile::{convert_career_info_to_career, convert_identity_info_to_identity},
        util::Career,
    };

    use super::{get_profile_internal, ProfileOperation};

    struct ProfileOperationMock {
        account: Account,
        identity_info_option: Option<IdentityInfo>,
        careers_info: Vec<CareerInfo>,
        consulting_fee_option: Option<ConsultingFee>,
    }

    impl ProfileOperation for ProfileOperationMock {
        fn find_user_account_by_user_account_id(
            &self,
            id: i32,
        ) -> Result<Account, common::ErrResp> {
            if self.account.user_account_id != id {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: NO_ACCOUNT_FOUND,
                    }),
                ));
            }
            Ok(self.account.clone())
        }

        fn find_identity_info_by_user_account_id(
            &self,
            _id: i32,
        ) -> Result<Option<IdentityInfo>, common::ErrResp> {
            Ok(self.identity_info_option.clone())
        }

        fn filter_career_info_by_user_account_id(
            &self,
            _id: i32,
        ) -> Result<Vec<CareerInfo>, common::ErrResp> {
            Ok(self.careers_info.clone())
        }

        fn find_consulting_fee_by_user_account_id(
            &self,
            _id: i32,
        ) -> Result<Option<ConsultingFee>, common::ErrResp> {
            Ok(self.consulting_fee_option.clone())
        }
    }

    fn create_dummy_identity_info(account_id: i32) -> IdentityInfo {
        let date = NaiveDate::from_ymd(1990, 4, 5);
        IdentityInfo {
            user_account_id: account_id,
            last_name: "田中".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "タナカ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            sex: "male".to_string(),
            date_of_birth: date,
            prefecture: "東京都".to_string(),
            city: "町田市".to_string(),
            address_line1: "森野2-2-22".to_string(),
            address_line2: None,
            telephone_number: "12345678901".to_string(),
        }
    }

    fn create_dummy_career_info_1(account_id: i32) -> CareerInfo {
        let start_date = NaiveDate::from_ymd(2013, 4, 1);
        CareerInfo {
            career_info_id: 1,
            user_account_id: account_id,
            company_name: "テスト株式会社".to_string(),
            department_name: None,
            office: None,
            career_start_date: start_date,
            career_end_date: None,
            contract_type: "regular".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: None,
            is_new_graduate: true,
            note: Some("備考テスト".to_string()),
        }
    }

    fn create_max_num_of_dummy_careers_info(account_id: i32) -> Vec<CareerInfo> {
        let mut careers = Vec::with_capacity(MAX_NUM_OF_CAREER_INFO_PER_USER_ACCOUNT as usize);
        let mut start_date = NaiveDate::from_ymd(2013, 4, 1);
        let mut end_date = Some(start_date + chrono::Duration::days(365));
        for i in 0..MAX_NUM_OF_CAREER_INFO_PER_USER_ACCOUNT {
            let career_info = CareerInfo {
                career_info_id: (i + 1) as i32,
                user_account_id: account_id,
                company_name: format!("テスト{}株式会社", i + 1),
                department_name: Some(format!("部署{}", i + 1)),
                office: Some(format!("事業所{}", i + 1)),
                career_start_date: start_date.clone(),
                career_end_date: end_date.clone(),
                contract_type: "contract".to_string(),
                profession: Some(format!("職種{}", i + 1)),
                annual_income_in_man_yen: Some(((i + 1) * 100) as i32),
                is_manager: true,
                position_name: None,
                is_new_graduate: false,
                note: None,
            };
            start_date = end_date.expect("failed to get Ok") + chrono::Duration::days(1);
            end_date = Some(start_date + chrono::Duration::days(30));
            careers.push(career_info);
        }
        careers
    }

    fn create_dummy_consulting_fee(account_id: i32) -> ConsultingFee {
        ConsultingFee {
            user_account_id: account_id,
            fee_per_hour_in_yen: 3000,
        }
    }

    #[tokio::test]
    async fn success_return_profile_when_there_is_no_identity() {
        let account_id = 51351;
        let email = "profile.test@test.com";
        let pwd = "vvvvvvvvvV";
        let hashed_pwd = hash_password(pwd).expect("failed to get Ok");
        let creation_time = Utc.ymd(2021, 9, 11).and_hms(15, 30, 45);
        let last_login = creation_time + chrono::Duration::days(1);
        let account = Account {
            user_account_id: account_id,
            email_address: email.to_string(),
            hashed_password: hashed_pwd,
            last_login_time: Some(last_login),
            created_at: creation_time,
        };
        let profile_op = ProfileOperationMock {
            account: account.clone(),
            identity_info_option: None,
            // ユーザー情報 (IdentityInfo) がない場合、職務経歴と相談料の登録は許可しないので、必ず空VecとNoneとなる
            careers_info: vec![],
            consulting_fee_option: None,
        };

        let result = get_profile_internal(account_id, profile_op)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert_eq!(account.email_address, result.1 .0.email_address);
        assert_eq!(None, result.1 .0.identity);
        let careers: Vec<Career> = vec![];
        assert_eq!(careers, result.1 .0.careers);
        assert_eq!(None, result.1 .0.fee_per_hour_in_yen);
    }

    #[tokio::test]
    async fn success_return_profile_with_identity_1career_fee() {
        let account_id = 51351;
        let email = "profile.test@test.com";
        let pwd = "vvvvvvvvvV";
        let hashed_pwd = hash_password(pwd).expect("failed to get Ok");
        let creation_time = Utc.ymd(2021, 9, 11).and_hms(15, 30, 45);
        let last_login = creation_time + chrono::Duration::days(1);
        let account = Account {
            user_account_id: account_id,
            email_address: email.to_string(),
            hashed_password: hashed_pwd,
            last_login_time: Some(last_login),
            created_at: creation_time,
        };
        let identity_info = create_dummy_identity_info(account_id);
        let career_info = create_dummy_career_info_1(account_id);
        let fee = create_dummy_consulting_fee(account_id);
        let profile_op = ProfileOperationMock {
            account: account.clone(),
            identity_info_option: Some(identity_info.clone()),
            careers_info: vec![career_info.clone()],
            consulting_fee_option: Some(fee.clone()),
        };

        let result = get_profile_internal(account_id, profile_op)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert_eq!(account.email_address, result.1 .0.email_address);
        assert_eq!(
            Some(convert_identity_info_to_identity(identity_info)),
            result.1 .0.identity
        );
        let careers = vec![career_info.clone()]
            .into_iter()
            .map(|c| convert_career_info_to_career(c))
            .collect::<Vec<Career>>();
        assert_eq!(careers, result.1 .0.careers);
        assert_eq!(
            Some(fee.fee_per_hour_in_yen),
            result.1 .0.fee_per_hour_in_yen
        );
    }

    #[tokio::test]
    async fn success_return_profile_with_identity_max_num_of_careers_fee() {
        let account_id = 51351;
        let email = "profile.test@test.com";
        let pwd = "vvvvvvvvvV";
        let hashed_pwd = hash_password(pwd).expect("failed to get Ok");
        let creation_time = Utc.ymd(2021, 9, 11).and_hms(15, 30, 45);
        let last_login = creation_time + chrono::Duration::days(1);
        let account = Account {
            user_account_id: account_id,
            email_address: email.to_string(),
            hashed_password: hashed_pwd,
            last_login_time: Some(last_login),
            created_at: creation_time,
        };
        let identity_info = create_dummy_identity_info(account_id);
        let careers_info = create_max_num_of_dummy_careers_info(account_id);
        let fee = create_dummy_consulting_fee(account_id);
        let profile_op = ProfileOperationMock {
            account: account.clone(),
            identity_info_option: Some(identity_info.clone()),
            careers_info: careers_info.clone(),
            consulting_fee_option: Some(fee.clone()),
        };

        let result = get_profile_internal(account_id, profile_op)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert_eq!(account.email_address, result.1 .0.email_address);
        assert_eq!(
            Some(convert_identity_info_to_identity(identity_info)),
            result.1 .0.identity
        );
        let careers = careers_info
            .into_iter()
            .map(|c| convert_career_info_to_career(c))
            .collect::<Vec<Career>>();
        assert_eq!(careers, result.1 .0.careers);
        assert_eq!(
            Some(fee.fee_per_hour_in_yen),
            result.1 .0.fee_per_hour_in_yen
        );
    }

    #[tokio::test]
    async fn success_return_profile_with_identity_without_career_fee() {
        let account_id = 51351;
        let email = "profile.test@test.com";
        let pwd = "vvvvvvvvvV";
        let hashed_pwd = hash_password(pwd).expect("failed to get Ok");
        let creation_time = Utc.ymd(2021, 9, 11).and_hms(15, 30, 45);
        let last_login = creation_time + chrono::Duration::days(1);
        let account = Account {
            user_account_id: account_id,
            email_address: email.to_string(),
            hashed_password: hashed_pwd,
            last_login_time: Some(last_login),
            created_at: creation_time,
        };
        let identity_info = create_dummy_identity_info(account_id);
        let profile_op = ProfileOperationMock {
            account: account.clone(),
            identity_info_option: Some(identity_info.clone()),
            careers_info: vec![],
            consulting_fee_option: None,
        };

        let result = get_profile_internal(account_id, profile_op)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert_eq!(account.email_address, result.1 .0.email_address);
        assert_eq!(
            Some(convert_identity_info_to_identity(identity_info)),
            result.1 .0.identity
        );
        let careers: Vec<Career> = vec![];
        assert_eq!(careers, result.1 .0.careers);
        assert_eq!(None, result.1 .0.fee_per_hour_in_yen);
    }

    #[tokio::test]
    async fn fail_return_no_account_found() {
        let account_id = 51351;
        let email = "profile.test@test.com";
        let pwd = "vvvvvvvvvV";
        let hashed_pwd = hash_password(pwd).expect("failed to get Ok");
        let creation_time = Utc.ymd(2021, 9, 11).and_hms(15, 30, 45);
        let last_login = creation_time + chrono::Duration::days(1);
        let account = Account {
            user_account_id: account_id,
            email_address: email.to_string(),
            hashed_password: hashed_pwd,
            last_login_time: Some(last_login),
            created_at: creation_time,
        };
        let profile_op = ProfileOperationMock {
            account: account.clone(),
            identity_info_option: None,
            careers_info: vec![],
            consulting_fee_option: None,
        };
        let non_existing_id = account_id + 1;

        let result = get_profile_internal(non_existing_id, profile_op)
            .await
            .expect_err("failed to get Err");

        assert_eq!(StatusCode::BAD_REQUEST, result.0);
        assert_eq!(err_code::NO_ACCOUNT_FOUND, result.1.code);
    }
}
