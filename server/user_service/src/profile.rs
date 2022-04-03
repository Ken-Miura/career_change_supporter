// Copyright 2021 Ken Miura

use axum::async_trait;
use axum::{extract::Extension, http::StatusCode, Json};
use chrono::Datelike;
use common::util::Ymd;
use common::{ApiError, ErrResp, RespResult, MAX_NUM_OF_CAREER_PER_USER_ACCOUNT};
use entity::prelude::{ConsultingFee, UserAccount};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};
use entity::{career, identity};
use serde::Serialize;

use crate::{
    err::{unexpected_err_resp, Code::NoAccountFound},
    util::{session::User, Career, Identity},
};

pub(crate) async fn get_profile(
    User { account_id }: User,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<ProfileResult> {
    let profile_op = ProfileOperationImpl::new(pool);
    handle_profile_req(account_id, profile_op).await
}

async fn handle_profile_req(
    account_id: i64,
    profile_op: impl ProfileOperation,
) -> RespResult<ProfileResult> {
    let email_address_option = profile_op
        .find_email_address_by_account_id(account_id)
        .await?;
    let email_address = email_address_option.ok_or_else(|| {
        tracing::error!("no email address (account id: {}) found", account_id);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: NoAccountFound as u32,
            }),
        )
    })?;
    let identity_option = profile_op
        .find_identity_by_user_account_id(account_id)
        .await?;
    let identity = match identity_option {
        Some(i) => i,
        None => {
            return Ok((
                StatusCode::OK,
                Json(ProfileResult::email_address(email_address).finish()),
            ));
        }
    };
    let careers = profile_op.filter_career_by_account_id(account_id).await?;
    let fee_per_hour_in_yen = profile_op
        .find_fee_per_hour_in_yen_by_account_id(account_id)
        .await?;
    Ok((
        StatusCode::OK,
        Json(
            ProfileResult::email_address(email_address)
                .identity(Some(identity))
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

#[async_trait]
trait ProfileOperation {
    async fn find_email_address_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<String>, ErrResp>;
    async fn find_identity_by_user_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<Identity>, ErrResp>;
    async fn filter_career_by_account_id(&self, account_id: i64) -> Result<Vec<Career>, ErrResp>;
    async fn find_fee_per_hour_in_yen_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<i32>, ErrResp>;
}

struct ProfileOperationImpl {
    pool: DatabaseConnection,
}

impl ProfileOperationImpl {
    fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProfileOperation for ProfileOperationImpl {
    async fn find_email_address_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<String>, ErrResp> {
        let model = UserAccount::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("failed to find account (account id: {}): {}", account_id, e);
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| m.email_address))
    }

    async fn find_identity_by_user_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<Identity>, ErrResp> {
        let model = entity::prelude::Identity::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(
                    "failed to find IndentityInfo (account id: {}): {}",
                    account_id,
                    e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(ProfileOperationImpl::convert_identity_model_to_identity))
    }

    async fn filter_career_by_account_id(&self, account_id: i64) -> Result<Vec<Career>, ErrResp> {
        let models = entity::prelude::Career::find()
            .filter(career::Column::UserAccountId.eq(account_id))
            .limit(MAX_NUM_OF_CAREER_PER_USER_ACCOUNT)
            .all(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(
                    "failed to filter career (account id: {}): {}",
                    account_id,
                    e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(ProfileOperationImpl::convert_career_model_to_career)
            .collect::<Vec<Career>>())
    }

    async fn find_fee_per_hour_in_yen_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<i32>, ErrResp> {
        let model = ConsultingFee::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(
                    "failed to find consulting fee (account id: {}): {}",
                    account_id,
                    e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| m.fee_per_hour_in_yen))
    }
}

impl ProfileOperationImpl {
    fn convert_identity_model_to_identity(identity_model: identity::Model) -> Identity {
        let date = identity_model.date_of_birth;
        let ymd = Ymd {
            year: date.year(),
            month: date.month(),
            day: date.day(),
        };
        Identity {
            last_name: identity_model.last_name,
            first_name: identity_model.first_name,
            last_name_furigana: identity_model.last_name_furigana,
            first_name_furigana: identity_model.first_name_furigana,
            date_of_birth: ymd,
            prefecture: identity_model.prefecture,
            city: identity_model.city,
            address_line1: identity_model.address_line1,
            address_line2: identity_model.address_line2,
            telephone_number: identity_model.telephone_number,
        }
    }

    fn convert_career_model_to_career(career_model: career::Model) -> Career {
        let career_start_date = Ymd {
            year: career_model.career_start_date.year(),
            month: career_model.career_start_date.month(),
            day: career_model.career_start_date.day(),
        };
        let career_end_date = career_model.career_end_date.map(|end_date| Ymd {
            year: end_date.year(),
            month: end_date.month(),
            day: end_date.day(),
        });
        Career {
            career_id: career_model.career_id,
            company_name: career_model.company_name,
            department_name: career_model.department_name,
            office: career_model.office,
            career_start_date,
            career_end_date,
            contract_type: career_model.contract_type,
            profession: career_model.profession,
            annual_income_in_man_yen: career_model.annual_income_in_man_yen,
            is_manager: career_model.is_manager,
            position_name: career_model.position_name,
            is_new_graduate: career_model.is_new_graduate,
            note: career_model.note,
        }
    }
}

// TODO: 事前準備に用意するデータ (Career、fee_per_hour_in_yen) に関して、データの追加、編集でvalidatorを実装した後、それを使ってチェックを行うよう修正する
#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use chrono::{Datelike, NaiveDate};
    use common::util::Ymd;
    use common::ErrResp;
    use common::MAX_NUM_OF_CAREER_PER_USER_ACCOUNT;

    use crate::util::validator::identity_validator::{validate_identity, MIN_AGE_REQUIREMENT};
    use crate::util::Identity;
    use crate::{err::Code::NoAccountFound, util::Career};

    use super::{handle_profile_req, ProfileOperation};

    struct ProfileOperationMock {
        email_address_option: Option<String>,
        identity_option: Option<Identity>,
        careers: Vec<Career>,
        fee_per_hour_in_yen_option: Option<i32>,
    }

    #[async_trait]
    impl ProfileOperation for ProfileOperationMock {
        async fn find_email_address_by_account_id(
            &self,
            _account_id: i64,
        ) -> Result<Option<String>, ErrResp> {
            Ok(self.email_address_option.clone())
        }

        async fn find_identity_by_user_account_id(
            &self,
            _account_id: i64,
        ) -> Result<Option<Identity>, ErrResp> {
            Ok(self.identity_option.clone())
        }

        async fn filter_career_by_account_id(
            &self,
            _account_id: i64,
        ) -> Result<Vec<Career>, ErrResp> {
            Ok(self.careers.clone())
        }

        async fn find_fee_per_hour_in_yen_by_account_id(
            &self,
            _account_id: i64,
        ) -> Result<Option<i32>, ErrResp> {
            Ok(self.fee_per_hour_in_yen_option)
        }
    }

    fn create_dummy_identity(date_of_birth: Ymd) -> Identity {
        Identity {
            last_name: "田中".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "タナカ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth,
            prefecture: "東京都".to_string(),
            city: "町田市".to_string(),
            address_line1: "森野2-2-22".to_string(),
            address_line2: None,
            telephone_number: "12345678901".to_string(),
        }
    }

    fn create_dummy_career() -> Career {
        let career_start_date = Ymd {
            year: 2013,
            month: 4,
            day: 1,
        };
        Career {
            career_id: 1,
            company_name: "テスト株式会社".to_string(),
            department_name: None,
            office: None,
            career_start_date,
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

    fn create_max_num_of_dummy_careers() -> Vec<Career> {
        let mut careers = Vec::with_capacity(MAX_NUM_OF_CAREER_PER_USER_ACCOUNT as usize);
        let mut start_date = NaiveDate::from_ymd(2013, 4, 1);
        let mut end_date = Some(start_date + chrono::Duration::days(365));
        for i in 0..MAX_NUM_OF_CAREER_PER_USER_ACCOUNT {
            let career_start_date = Ymd {
                year: start_date.year(),
                month: start_date.month(),
                day: start_date.day(),
            };
            let career_end_date = end_date.map(|date| Ymd {
                year: date.year(),
                month: date.month(),
                day: date.day(),
            });
            let career = Career {
                career_id: (i + 1) as i64,
                company_name: format!("テスト{}株式会社", i + 1),
                department_name: Some(format!("部署{}", i + 1)),
                office: Some(format!("事業所{}", i + 1)),
                career_start_date,
                career_end_date,
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
            careers.push(career);
        }
        careers
    }

    #[tokio::test]
    async fn handle_profile_req_success_return_profile_when_there_is_no_identity() {
        let account_id = 51351;
        let email_address = "profile.test@test.com".to_string();
        let email_address_option = Some(email_address.clone());
        let profile_op = ProfileOperationMock {
            email_address_option,
            identity_option: None,
            careers: vec![],
            fee_per_hour_in_yen_option: None,
        };

        let result = handle_profile_req(account_id, profile_op)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert_eq!(email_address, result.1 .0.email_address);
        assert_eq!(None, result.1 .0.identity);
        let careers: Vec<Career> = vec![];
        assert_eq!(careers, result.1 .0.careers);
        assert_eq!(None, result.1 .0.fee_per_hour_in_yen);
    }

    #[tokio::test]
    async fn handle_profile_req_success_return_profile_with_identity_1career_fee() {
        let account_id = 51351;
        let email_address = "profile.test@test.com".to_string();
        let email_address_option = Some(email_address.clone());

        let current_date = NaiveDate::from_ymd(2022, 2, 25);
        let date_of_birth = Ymd {
            year: current_date.year() - MIN_AGE_REQUIREMENT,
            month: current_date.month(),
            day: current_date.day(),
        };
        let identity = create_dummy_identity(date_of_birth);
        let _ = validate_identity(&identity, &current_date).expect("failed to get Ok");

        let career = create_dummy_career();
        let careers = vec![career];

        let fee_per_hour_in_yen_option = Some(3000);

        let profile_op = ProfileOperationMock {
            email_address_option,
            identity_option: Some(identity.clone()),
            careers: careers.clone(),
            fee_per_hour_in_yen_option,
        };

        let result = handle_profile_req(account_id, profile_op)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert_eq!(email_address, result.1 .0.email_address);
        assert_eq!(Some(identity), result.1 .0.identity);
        assert_eq!(careers, result.1 .0.careers);
        assert_eq!(fee_per_hour_in_yen_option, result.1 .0.fee_per_hour_in_yen);
    }

    #[tokio::test]
    async fn handle_profile_req_success_return_profile_with_identity_max_num_of_careers_fee() {
        let account_id = 51351;
        let email_address = "profile.test@test.com".to_string();
        let email_address_option = Some(email_address.clone());

        let current_date = NaiveDate::from_ymd(2022, 2, 25);
        let date_of_birth = Ymd {
            year: current_date.year() - MIN_AGE_REQUIREMENT,
            month: current_date.month(),
            day: current_date.day(),
        };
        let identity = create_dummy_identity(date_of_birth);
        let _ = validate_identity(&identity, &current_date).expect("failed to get Ok");

        let careers = create_max_num_of_dummy_careers();

        let fee_per_hour_in_yen_option = Some(3000);

        let profile_op = ProfileOperationMock {
            email_address_option,
            identity_option: Some(identity.clone()),
            careers: careers.clone(),
            fee_per_hour_in_yen_option,
        };

        let result = handle_profile_req(account_id, profile_op)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert_eq!(email_address, result.1 .0.email_address);
        assert_eq!(Some(identity), result.1 .0.identity);
        assert_eq!(careers, result.1 .0.careers);
        assert_eq!(fee_per_hour_in_yen_option, result.1 .0.fee_per_hour_in_yen);
    }

    #[tokio::test]
    async fn handle_profile_req_success_return_profile_with_identity_without_career_fee() {
        let account_id = 51351;
        let email_address = "profile.test@test.com".to_string();
        let email_address_option = Some(email_address.clone());

        let current_date = NaiveDate::from_ymd(2022, 2, 25);
        let date_of_birth = Ymd {
            year: current_date.year() - MIN_AGE_REQUIREMENT,
            month: current_date.month(),
            day: current_date.day(),
        };
        let identity = create_dummy_identity(date_of_birth);
        let _ = validate_identity(&identity, &current_date).expect("failed to get Ok");

        let profile_op = ProfileOperationMock {
            email_address_option,
            identity_option: Some(identity.clone()),
            careers: vec![],
            fee_per_hour_in_yen_option: None,
        };

        let result = handle_profile_req(account_id, profile_op)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert_eq!(email_address, result.1 .0.email_address);
        assert_eq!(Some(identity), result.1 .0.identity);
        let careers: Vec<Career> = vec![];
        assert_eq!(careers, result.1 .0.careers);
        assert_eq!(None, result.1 .0.fee_per_hour_in_yen);
    }

    #[tokio::test]
    async fn handle_profile_req_fail_return_no_email_address_found() {
        let non_existing_id = 51351;
        let profile_op = ProfileOperationMock {
            email_address_option: None,
            identity_option: None,
            careers: vec![],
            fee_per_hour_in_yen_option: None,
        };

        let result = handle_profile_req(non_existing_id, profile_op)
            .await
            .expect_err("failed to get Err");

        assert_eq!(StatusCode::BAD_REQUEST, result.0);
        assert_eq!(NoAccountFound as u32, result.1.code);
    }
}
