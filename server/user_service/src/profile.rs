// Copyright 2021 Ken Miura

use axum::async_trait;
use axum::{extract::Extension, http::StatusCode, Json};
use chrono::Datelike;
use common::util::{Identity, Ymd};
use common::{ApiError, ErrResp, RespResult, MAX_NUM_OF_CAREER_PER_USER_ACCOUNT};
use entity::prelude::{ConsultingFee, UserAccount};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};
use entity::{career, identity};
use serde::Serialize;
use tracing::error;

use crate::{
    err::{unexpected_err_resp, Code::NoAccountFound},
    util::session::User,
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
        error!("no email address (account id: {}) found", account_id);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: NoAccountFound as u32,
            }),
        )
    })?;
    let identity_option = profile_op.find_identity_by_account_id(account_id).await?;
    let identity = match identity_option {
        Some(i) => i,
        None => {
            return Ok((
                StatusCode::OK,
                Json(ProfileResult::email_address(email_address).finish()),
            ));
        }
    };
    let career_descriptions = profile_op
        .filter_career_descriptions_by_account_id(account_id)
        .await?;
    let fee_per_hour_in_yen = profile_op
        .find_fee_per_hour_in_yen_by_account_id(account_id)
        .await?;
    Ok((
        StatusCode::OK,
        Json(
            ProfileResult::email_address(email_address)
                .identity(Some(identity))
                .career_descriptions(career_descriptions)
                .fee_per_hour_in_yen(fee_per_hour_in_yen)
                .finish(),
        ),
    ))
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub(crate) struct CareerDescription {
    pub career_id: i64,
    pub company_name: String,
    pub contract_type: String,
    pub career_start_date: Ymd,
    pub career_end_date: Option<Ymd>,
}

#[derive(Serialize, Debug)]
pub(crate) struct ProfileResult {
    pub email_address: String,
    pub identity: Option<Identity>,
    pub career_descriptions: Vec<CareerDescription>,
    pub fee_per_hour_in_yen: Option<i32>,
}

impl ProfileResult {
    fn email_address(email_address: String) -> ProfileResultBuilder {
        ProfileResultBuilder {
            email_address,
            identity: None,
            career_descriptions: vec![],
            fee_per_hour_in_yen: None,
        }
    }
}

struct ProfileResultBuilder {
    email_address: String,
    identity: Option<Identity>,
    career_descriptions: Vec<CareerDescription>,
    fee_per_hour_in_yen: Option<i32>,
}

impl ProfileResultBuilder {
    fn identity(mut self, identity: Option<Identity>) -> ProfileResultBuilder {
        self.identity = identity;
        self
    }

    fn career_descriptions(
        mut self,
        career_descriptions: Vec<CareerDescription>,
    ) -> ProfileResultBuilder {
        self.career_descriptions = career_descriptions;
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
            career_descriptions: self.career_descriptions,
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
    async fn find_identity_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<Identity>, ErrResp>;
    async fn filter_career_descriptions_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Vec<CareerDescription>, ErrResp>;
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
                error!(
                    "failed to find user_account (user_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| m.email_address))
    }

    async fn find_identity_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<Identity>, ErrResp> {
        let model = entity::prelude::Identity::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find identity (user_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(ProfileOperationImpl::convert_identity_model_to_identity))
    }

    async fn filter_career_descriptions_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Vec<CareerDescription>, ErrResp> {
        let models = entity::prelude::Career::find()
            .filter(career::Column::UserAccountId.eq(account_id))
            .limit(MAX_NUM_OF_CAREER_PER_USER_ACCOUNT)
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter career (user_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(ProfileOperationImpl::convert_career_model_to_career_description)
            .collect::<Vec<CareerDescription>>())
    }

    async fn find_fee_per_hour_in_yen_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<i32>, ErrResp> {
        let model = ConsultingFee::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find consulting_fee (user_account_id: {}): {}",
                    account_id, e
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

    fn convert_career_model_to_career_description(
        career_model: career::Model,
    ) -> CareerDescription {
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
        CareerDescription {
            career_id: career_model.career_id,
            company_name: career_model.company_name,
            contract_type: career_model.contract_type,
            career_start_date,
            career_end_date,
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use chrono::{Datelike, NaiveDate};
    use common::util::Identity;
    use common::util::Ymd;
    use common::ErrResp;
    use common::MAX_NUM_OF_CAREER_PER_USER_ACCOUNT;

    use crate::err::Code::NoAccountFound;
    use crate::util::validator::identity_validator::{validate_identity, MIN_AGE_REQUIREMENT};
    use crate::util::MAX_FEE_PER_HOUR_IN_YEN;
    use crate::util::MIN_FEE_PER_HOUR_IN_YEN;

    use super::CareerDescription;
    use super::{handle_profile_req, ProfileOperation};

    struct ProfileOperationMock {
        email_address_option: Option<String>,
        identity_option: Option<Identity>,
        career_descriptions: Vec<CareerDescription>,
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

        async fn find_identity_by_account_id(
            &self,
            _account_id: i64,
        ) -> Result<Option<Identity>, ErrResp> {
            Ok(self.identity_option.clone())
        }

        async fn filter_career_descriptions_by_account_id(
            &self,
            _account_id: i64,
        ) -> Result<Vec<CareerDescription>, ErrResp> {
            Ok(self.career_descriptions.clone())
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

    fn create_dummy_career_description() -> CareerDescription {
        let career_start_date = Ymd {
            year: 2013,
            month: 4,
            day: 1,
        };
        CareerDescription {
            career_id: 1,
            company_name: "テスト株式会社".to_string(),
            contract_type: "regular".to_string(),
            career_start_date,
            career_end_date: None,
        }
    }

    fn create_max_num_of_dummy_career_descriptions() -> Vec<CareerDescription> {
        let mut career_descriptions =
            Vec::with_capacity(MAX_NUM_OF_CAREER_PER_USER_ACCOUNT as usize);
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
            let career_description = CareerDescription {
                career_id: (i + 1) as i64,
                company_name: format!("テスト{}株式会社", i + 1),
                contract_type: "contract".to_string(),
                career_start_date,
                career_end_date,
            };
            start_date = end_date.expect("failed to get Ok") + chrono::Duration::days(1);
            end_date = Some(start_date + chrono::Duration::days(30));
            career_descriptions.push(career_description);
        }
        career_descriptions
    }

    #[tokio::test]
    async fn handle_profile_req_success_return_profile_when_there_is_no_identity() {
        let account_id = 51351;
        let email_address = "profile.test@test.com".to_string();
        let email_address_option = Some(email_address.clone());
        let profile_op = ProfileOperationMock {
            email_address_option,
            identity_option: None,
            career_descriptions: vec![],
            fee_per_hour_in_yen_option: None,
        };

        let result = handle_profile_req(account_id, profile_op)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert_eq!(email_address, result.1 .0.email_address);
        assert_eq!(None, result.1 .0.identity);
        let career_descriptions: Vec<CareerDescription> = vec![];
        assert_eq!(career_descriptions, result.1 .0.career_descriptions);
        assert_eq!(None, result.1 .0.fee_per_hour_in_yen);
    }

    #[tokio::test]
    async fn handle_profile_req_success_return_profile_with_identity_1career_description_fee() {
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
        validate_identity(&identity, &current_date).expect("failed to get Ok");

        let career_description = create_dummy_career_description();
        let career_descriptions = vec![career_description];

        let fee_per_hour_in_yen = 3000;
        assert!((MIN_FEE_PER_HOUR_IN_YEN..=MAX_FEE_PER_HOUR_IN_YEN).contains(&fee_per_hour_in_yen));
        let fee_per_hour_in_yen_option = Some(fee_per_hour_in_yen);

        let profile_op = ProfileOperationMock {
            email_address_option,
            identity_option: Some(identity.clone()),
            career_descriptions: career_descriptions.clone(),
            fee_per_hour_in_yen_option,
        };

        let result = handle_profile_req(account_id, profile_op)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert_eq!(email_address, result.1 .0.email_address);
        assert_eq!(Some(identity), result.1 .0.identity);
        assert_eq!(career_descriptions, result.1 .0.career_descriptions);
        assert_eq!(fee_per_hour_in_yen_option, result.1 .0.fee_per_hour_in_yen);
    }

    #[tokio::test]
    async fn handle_profile_req_success_return_profile_with_identity_max_num_of_career_descriptions_fee(
    ) {
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
        validate_identity(&identity, &current_date).expect("failed to get Ok");

        let career_descriptions = create_max_num_of_dummy_career_descriptions();

        let fee_per_hour_in_yen = 3000;
        assert!((MIN_FEE_PER_HOUR_IN_YEN..=MAX_FEE_PER_HOUR_IN_YEN).contains(&fee_per_hour_in_yen));
        let fee_per_hour_in_yen_option = Some(fee_per_hour_in_yen);

        let profile_op = ProfileOperationMock {
            email_address_option,
            identity_option: Some(identity.clone()),
            career_descriptions: career_descriptions.clone(),
            fee_per_hour_in_yen_option,
        };

        let result = handle_profile_req(account_id, profile_op)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert_eq!(email_address, result.1 .0.email_address);
        assert_eq!(Some(identity), result.1 .0.identity);
        assert_eq!(career_descriptions, result.1 .0.career_descriptions);
        assert_eq!(fee_per_hour_in_yen_option, result.1 .0.fee_per_hour_in_yen);
    }

    #[tokio::test]
    async fn handle_profile_req_success_return_profile_with_identity_without_career_description_fee(
    ) {
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
        validate_identity(&identity, &current_date).expect("failed to get Ok");

        let profile_op = ProfileOperationMock {
            email_address_option,
            identity_option: Some(identity.clone()),
            career_descriptions: vec![],
            fee_per_hour_in_yen_option: None,
        };

        let result = handle_profile_req(account_id, profile_op)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert_eq!(email_address, result.1 .0.email_address);
        assert_eq!(Some(identity), result.1 .0.identity);
        let career_descriptions: Vec<CareerDescription> = vec![];
        assert_eq!(career_descriptions, result.1 .0.career_descriptions);
        assert_eq!(None, result.1 .0.fee_per_hour_in_yen);
    }

    #[tokio::test]
    async fn handle_profile_req_fail_return_no_email_address_found() {
        let non_existing_id = 51351;
        let profile_op = ProfileOperationMock {
            email_address_option: None,
            identity_option: None,
            career_descriptions: vec![],
            fee_per_hour_in_yen_option: None,
        };

        let result = handle_profile_req(non_existing_id, profile_op)
            .await
            .expect_err("failed to get Err");

        assert_eq!(StatusCode::BAD_REQUEST, result.0);
        assert_eq!(NoAccountFound as u32, result.1.code);
    }
}
