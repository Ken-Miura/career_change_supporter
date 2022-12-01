// Copyright 2022 Ken Miura

use async_session::serde_json::{json, Value};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::opensearch::{search_documents, INDEX_NAME};
use common::{ApiError, ErrResp, RespResult, MAX_NUM_OF_CAREER_PER_USER_ACCOUNT};
use entity::sea_orm::DatabaseConnection;
use opensearch::OpenSearch;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::err::Code;
use crate::util::session::User;
use crate::util::{
    self, round_to_one_decimal_places, VALID_YEARS_OF_SERVICE_PERIOD_FIFTEEN,
    VALID_YEARS_OF_SERVICE_PERIOD_FIVE, VALID_YEARS_OF_SERVICE_PERIOD_TEN,
    VALID_YEARS_OF_SERVICE_PERIOD_THREE, VALID_YEARS_OF_SERVICE_PERIOD_TWENTY,
};

const YEARS_OF_SERVICE_LESS_THAN_THREE_YEARS: &str = "LESS_THAN_THREE_YEARS";
const YEARS_OF_SERVICE_THREE_YEARS_OR_MORE_LESS_THAN_FIVE_YEARS: &str =
    "THREE_YEARS_OR_MORE_LESS_THAN_FIVE_YEARS";
const YEARS_OF_SERVICE_FIVE_YEARS_OR_MORE_LESS_THAN_TEN_YEARS: &str =
    "FIVE_YEARS_OR_MORE_LESS_THAN_TEN_YEARS";
const YEARS_OF_SERVICE_TEN_YEARS_OR_MORE_LESS_THAN_FIFTEEN_YEARS: &str =
    "TEN_YEARS_OR_MORE_LESS_THAN_FIFTEEN_YEARS";
const YEARS_OF_SERVICE_FIFTEEN_YEARS_OR_MORE_LESS_THAN_TWENTY_YEARS: &str =
    "FIFTEEN_YEARS_OR_MORE_LESS_THAN_TWENTY_YEARS";
const YEARS_OF_SERVICE_TWENTY_YEARS_OR_MORE: &str = "TWENTY_YEARS_OR_MORE";

pub(crate) async fn get_consultant_detail(
    User { account_id }: User,
    query: Query<ConsultantDetailQuery>,
    State(pool): State<DatabaseConnection>,
    State(index_client): State<OpenSearch>,
) -> RespResult<ConsultantDetail> {
    let query = query.0;
    let op = ConsultantDetailOperationImpl { pool, index_client };
    handle_consultant_detail(account_id, query.consultant_id, op).await
}

#[derive(Deserialize)]
pub(crate) struct ConsultantDetailQuery {
    pub(crate) consultant_id: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultantDetail {
    pub(crate) consultant_id: i64,
    pub(crate) fee_per_hour_in_yen: i32,
    pub(crate) rating: Option<String>, // 適切な型は浮動少数だが、PartialEqの==を正しく動作させるために文字列として処理する
    pub(crate) num_of_rated: i32,
    pub(crate) careers: Vec<ConsultantCareerDetail>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultantCareerDetail {
    pub(crate) company_name: String,
    pub(crate) department_name: Option<String>,
    pub(crate) office: Option<String>,
    pub(crate) years_of_service: String,
    pub(crate) employed: bool,
    pub(crate) contract_type: String,
    pub(crate) profession: Option<String>,
    pub(crate) annual_income_in_man_yen: Option<i32>,
    pub(crate) is_manager: bool,
    pub(crate) position_name: Option<String>,
    pub(crate) is_new_graduate: bool,
    pub(crate) note: Option<String>,
}

#[async_trait]
trait ConsultantDetailOperation {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
    async fn check_if_consultant_is_available(&self, consultant_id: i64) -> Result<bool, ErrResp>;
    async fn search_consultant(&self, index_name: &str, query: &Value) -> Result<Value, ErrResp>;
}

struct ConsultantDetailOperationImpl {
    pool: DatabaseConnection,
    index_client: OpenSearch,
}

#[async_trait]
impl ConsultantDetailOperation for ConsultantDetailOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::check_if_identity_exists(&self.pool, account_id).await
    }

    async fn check_if_consultant_is_available(&self, consultant_id: i64) -> Result<bool, ErrResp> {
        util::check_if_user_account_is_available(&self.pool, consultant_id).await
    }

    async fn search_consultant(&self, index_name: &str, query: &Value) -> Result<Value, ErrResp> {
        let result = search_documents(index_name, 0, 1, None, query, &self.index_client).await?;
        Ok(result)
    }
}

async fn handle_consultant_detail(
    account_id: i64,
    consultant_id: i64,
    op: impl ConsultantDetailOperation,
) -> RespResult<ConsultantDetail> {
    if !consultant_id.is_positive() {
        error!("consultant_id ({}) is not positive", consultant_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NonPositiveConsultantId as u32,
            }),
        ));
    }
    let identity_exists = op.check_if_identity_exists(account_id).await?;
    if !identity_exists {
        error!("identity is not registered (account_id: {})", account_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoIdentityRegistered as u32,
            }),
        ));
    }
    let consultant_available = op.check_if_consultant_is_available(consultant_id).await?;
    if !consultant_available {
        error!(
            "consultant is not available (consultant_id: {})",
            consultant_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ConsultantIsNotAvailable as u32,
            }),
        ));
    }
    info!(
        "query param (account_id (for consultant): {}, account_id: {})",
        consultant_id, account_id
    );
    let query = create_query_json(consultant_id, account_id);
    let result = op.search_consultant(INDEX_NAME, &query).await?;
    parse_query_result(result)
}

fn create_query_json(account_id_for_consultant: i64, account_id: i64) -> Value {
    json!({
        "query": {
            "bool": {
                "must": [
                    {
                        "term": {
                            "user_account_id": account_id_for_consultant
                        }
                    }
                ],
                "filter": [
                    {
                        "range": {
                            "num_of_careers": {
                                "gt": 0
                            }
                        }
                    },
                    {
                        "exists": {
                            "field": "fee_per_hour_in_yen"
                        }
                    },
                    {
                        "term": {
                            "is_bank_account_registered": true
                        }
                    }
                ],
                "must_not": [
                    {
                        "term": {
                            "user_account_id": account_id
                        }
                    }
                ]
            }
        }
    })
}

fn parse_query_result(query_result: Value) -> RespResult<ConsultantDetail> {
    let took = query_result["took"].as_i64().ok_or_else(|| {
        error!("failed to get processing time: {}", query_result);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    info!("opensearch took {} milliseconds", took);

    let total = query_result["hits"]["total"]["value"]
        .as_i64()
        .ok_or_else(|| {
            error!("failed to get total value: {}", query_result);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: Code::UnexpectedErr as u32,
                }),
            )
        })?;
    // コンサルタントをIDで指定してる検索しているので1より大きなヒット数はありえない。0または1のみがありえる
    if total > 1 {
        error!("found multiple consultants (total: {})", total);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        ));
    }

    let hits = query_result["hits"]["hits"].as_array().ok_or_else(|| {
        error!("failed to get hits: {}", query_result);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    let hit_num = hits.len();
    if hit_num != 1 {
        error!("no consultant found (hit_num: {})", hit_num);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ConsultantDoesNotExist as u32,
            }),
        ));
    }
    let consultant_detail = create_consultant_detail(&hits[0])?;
    // NOTE: 将来的にパフォーマンスに影響する場合、ログに出力する内容を制限する
    info!("consultant_detail: {:?}", consultant_detail);
    Ok((StatusCode::OK, Json(consultant_detail)))
}

fn create_consultant_detail(hit: &Value) -> Result<ConsultantDetail, ErrResp> {
    let account_id = hit["_source"]["user_account_id"].as_i64().ok_or_else(|| {
        error!("failed to find account id in _source: {:?}", hit);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    let fee_per_hour_in_yen = hit["_source"]["fee_per_hour_in_yen"]
        .as_i64()
        .ok_or_else(|| {
            error!(
                "failed to find fee_per_hour_in_yen id in _source: {:?}",
                hit
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: Code::UnexpectedErr as u32,
                }),
            )
        })?;
    let rating = hit["_source"]["rating"].as_f64();
    let num_of_rated = hit["_source"]["num_of_rated"].as_i64().unwrap_or(0);
    // 検索条件で num_of_careers > 0 を指定しているので、careersがないケースはエラーとして扱う
    let careers = hit["_source"]["careers"].as_array().ok_or_else(|| {
        error!("failed to find careers id in _source: {:?}", hit);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    let mut consultant_career_details =
        Vec::with_capacity(MAX_NUM_OF_CAREER_PER_USER_ACCOUNT as usize);
    for career in careers {
        let consultant_career_detail = create_consultant_career_detail(career)?;
        consultant_career_details.push(consultant_career_detail);
    }
    Ok(ConsultantDetail {
        consultant_id: account_id,
        fee_per_hour_in_yen: fee_per_hour_in_yen as i32,
        rating: rating.map(round_to_one_decimal_places),
        num_of_rated: num_of_rated as i32,
        careers: consultant_career_details,
    })
}

fn create_consultant_career_detail(career: &Value) -> Result<ConsultantCareerDetail, ErrResp> {
    let company_name = career["company_name"].as_str().ok_or_else(|| {
        error!("failed to find company_name in career: {:?}", career);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    let department_name = career["department_name"].as_str();
    let office = career["office"].as_str();
    let years_of_service = career["years_of_service"].as_i64().ok_or_else(|| {
        error!("failed to find years_of_service in career: {:?}", career);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    let years_of_service = convert_years_of_service(years_of_service)?;
    let employed = career["employed"].as_bool().ok_or_else(|| {
        error!("failed to find employed in career: {:?}", career);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    let contract_type = career["contract_type"].as_str().ok_or_else(|| {
        error!("failed to find contract_type in career: {:?}", career);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    let profession = career["profession"].as_str();
    let annual_income_in_man_yen = career["annual_income_in_man_yen"].as_i64();
    let is_manager = career["is_manager"].as_bool().ok_or_else(|| {
        error!("failed to find is_manager in career: {:?}", career);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    let position_name = career["position_name"].as_str();
    let is_new_graduate = career["is_new_graduate"].as_bool().ok_or_else(|| {
        error!("failed to find is_new_graduate in career: {:?}", career);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    let note = career["note"].as_str();
    Ok(ConsultantCareerDetail {
        company_name: company_name.to_string(),
        department_name: department_name.map(|s| s.to_string()),
        office: office.map(|s| s.to_string()),
        years_of_service,
        employed,
        contract_type: contract_type.to_string(),
        profession: profession.map(|s| s.to_string()),
        annual_income_in_man_yen: annual_income_in_man_yen.map(|i| i as i32),
        is_manager,
        position_name: position_name.map(|s| s.to_string()),
        is_new_graduate,
        note: note.map(|s| s.to_string()),
    })
}

fn convert_years_of_service(years_of_service: i64) -> Result<String, ErrResp> {
    if (0..VALID_YEARS_OF_SERVICE_PERIOD_THREE as i64).contains(&years_of_service) {
        Ok(YEARS_OF_SERVICE_LESS_THAN_THREE_YEARS.to_string())
    } else if (VALID_YEARS_OF_SERVICE_PERIOD_THREE as i64
        ..VALID_YEARS_OF_SERVICE_PERIOD_FIVE as i64)
        .contains(&years_of_service)
    {
        Ok(YEARS_OF_SERVICE_THREE_YEARS_OR_MORE_LESS_THAN_FIVE_YEARS.to_string())
    } else if (VALID_YEARS_OF_SERVICE_PERIOD_FIVE as i64..VALID_YEARS_OF_SERVICE_PERIOD_TEN as i64)
        .contains(&years_of_service)
    {
        Ok(YEARS_OF_SERVICE_FIVE_YEARS_OR_MORE_LESS_THAN_TEN_YEARS.to_string())
    } else if (VALID_YEARS_OF_SERVICE_PERIOD_TEN as i64
        ..VALID_YEARS_OF_SERVICE_PERIOD_FIFTEEN as i64)
        .contains(&years_of_service)
    {
        Ok(YEARS_OF_SERVICE_TEN_YEARS_OR_MORE_LESS_THAN_FIFTEEN_YEARS.to_string())
    } else if (VALID_YEARS_OF_SERVICE_PERIOD_FIFTEEN as i64
        ..VALID_YEARS_OF_SERVICE_PERIOD_TWENTY as i64)
        .contains(&years_of_service)
    {
        Ok(YEARS_OF_SERVICE_FIFTEEN_YEARS_OR_MORE_LESS_THAN_TWENTY_YEARS.to_string())
    } else if years_of_service >= VALID_YEARS_OF_SERVICE_PERIOD_TWENTY as i64 {
        Ok(YEARS_OF_SERVICE_TWENTY_YEARS_OR_MORE.to_string())
    } else {
        error!("invalid years_of_service: {}", years_of_service);
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use async_session::serde_json::{json, Value};
    use axum::http::StatusCode;
    use axum::{async_trait, Json};
    use common::{ApiError, ErrResp, RespResult};
    use once_cell::sync::Lazy;

    use crate::err::Code;

    use super::{
        handle_consultant_detail, ConsultantCareerDetail, ConsultantDetail,
        ConsultantDetailOperation, YEARS_OF_SERVICE_FIFTEEN_YEARS_OR_MORE_LESS_THAN_TWENTY_YEARS,
        YEARS_OF_SERVICE_FIVE_YEARS_OR_MORE_LESS_THAN_TEN_YEARS,
        YEARS_OF_SERVICE_LESS_THAN_THREE_YEARS,
        YEARS_OF_SERVICE_TEN_YEARS_OR_MORE_LESS_THAN_FIFTEEN_YEARS,
        YEARS_OF_SERVICE_THREE_YEARS_OR_MORE_LESS_THAN_FIVE_YEARS,
        YEARS_OF_SERVICE_TWENTY_YEARS_OR_MORE,
    };

    #[derive(Clone, Debug)]
    struct ConsultantDetailOperationMock {
        account_id: i64,
        consultant_id: i64,
        query_result: Value,
    }

    #[async_trait]
    impl ConsultantDetailOperation for ConsultantDetailOperationMock {
        async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
            if self.account_id != account_id {
                return Ok(false);
            };
            Ok(true)
        }

        async fn check_if_consultant_is_available(
            &self,
            consultant_id: i64,
        ) -> Result<bool, ErrResp> {
            if self.consultant_id != consultant_id {
                return Ok(false);
            };
            Ok(true)
        }

        async fn search_consultant(
            &self,
            _index_name: &str,
            _query: &Value,
        ) -> Result<Value, ErrResp> {
            Ok(self.query_result.clone())
        }
    }

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<ConsultantDetail>,
    }

    #[derive(Debug)]
    struct Input {
        account_id: i64,
        consultant_id: i64,
        op: ConsultantDetailOperationMock,
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        vec![
            TestCase {
                name: "consultant id is not positive".to_string(),
                input: Input {
                    account_id: 1,
                    consultant_id: 0,
                    op: ConsultantDetailOperationMock {
                        account_id: 1,
                        consultant_id: 1,
                        query_result: json!({
                          "took" : 6,
                          "timed_out" : false,
                          "_shards" : {
                            "total" : 1,
                            "successful" : 1,
                            "skipped" : 0,
                            "failed" : 0
                          },
                          "hits" : {
                            "total" : {
                              "value" : 0,
                              "relation" : "eq"
                            },
                            "max_score" : null,
                            "hits" : [ ]
                          }
                        }),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NonPositiveConsultantId as u32,
                    }),
                )),
            },
            TestCase {
                name: "no identity found".to_string(),
                input: Input {
                    account_id: 1,
                    consultant_id: 2,
                    op: ConsultantDetailOperationMock {
                        account_id: 3,
                        consultant_id: 2,
                        query_result: json!({
                          "took" : 6,
                          "timed_out" : false,
                          "_shards" : {
                            "total" : 1,
                            "successful" : 1,
                            "skipped" : 0,
                            "failed" : 0
                          },
                          "hits" : {
                            "total" : {
                              "value" : 0,
                              "relation" : "eq"
                            },
                            "max_score" : null,
                            "hits" : [ ]
                          }
                        }),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoIdentityRegistered as u32,
                    }),
                )),
            },
            TestCase {
                name: "no counsultant found 1".to_string(),
                input: Input {
                    account_id: 3,
                    consultant_id: 4,
                    op: ConsultantDetailOperationMock {
                        account_id: 3,
                        consultant_id: 5,
                        query_result: json!({
                          "took" : 5,
                          "timed_out" : false,
                          "_shards" : {
                            "total" : 1,
                            "successful" : 1,
                            "skipped" : 0,
                            "failed" : 0
                          },
                          "hits" : {
                            "total" : {
                              "value" : 1,
                              "relation" : "eq"
                            },
                            "max_score" : 1.0,
                            "hits" : [
                              {
                                "_index" : "users",
                                "_id" : "3",
                                "_score" : 1.0,
                                "_source" : {
                                  "careers" : [
                                    {
                                      "annual_income_in_man_yen" : null,
                                      "career_id" : 2,
                                      "company_name" : "テスト５（株）",
                                      "contract_type" : "regular",
                                      "department_name" : null,
                                      "employed" : true,
                                      "is_manager" : false,
                                      "is_new_graduate" : false,
                                      "note" : null,
                                      "office" : null,
                                      "position_name" : null,
                                      "profession" : null,
                                      "years_of_service" : 2
                                    }
                                  ],
                                  "fee_per_hour_in_yen" : 3000,
                                  "is_bank_account_registered" : true,
                                  "num_of_careers" : 1,
                                  "rating" : null,
                                  "user_account_id" : 3
                                }
                              }
                            ]
                          }
                        }),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::ConsultantIsNotAvailable as u32,
                    }),
                )),
            },
            TestCase {
                name: "no counsultant found 2".to_string(),
                input: Input {
                    account_id: 1,
                    consultant_id: 2,
                    op: ConsultantDetailOperationMock {
                        account_id: 1,
                        consultant_id: 2,
                        query_result: json!({
                          "took" : 6,
                          "timed_out" : false,
                          "_shards" : {
                            "total" : 1,
                            "successful" : 1,
                            "skipped" : 0,
                            "failed" : 0
                          },
                          "hits" : {
                            "total" : {
                              "value" : 0,
                              "relation" : "eq"
                            },
                            "max_score" : null,
                            "hits" : [ ]
                          }
                        }),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::ConsultantDoesNotExist as u32,
                    }),
                )),
            },
            TestCase {
                name: "succeed in getting consultant 1".to_string(),
                input: Input {
                    account_id: 1,
                    consultant_id: 3,
                    op: ConsultantDetailOperationMock {
                        account_id: 1,
                        consultant_id: 3,
                        query_result: json!({
                          "took" : 5,
                          "timed_out" : false,
                          "_shards" : {
                            "total" : 1,
                            "successful" : 1,
                            "skipped" : 0,
                            "failed" : 0
                          },
                          "hits" : {
                            "total" : {
                              "value" : 1,
                              "relation" : "eq"
                            },
                            "max_score" : 1.0,
                            "hits" : [
                              {
                                "_index" : "users",
                                "_id" : "3",
                                "_score" : 1.0,
                                "_source" : {
                                  "careers" : [
                                    {
                                      "annual_income_in_man_yen" : 400,
                                      "career_id" : 2,
                                      "company_name" : "テスト５（株）",
                                      "contract_type" : "regular",
                                      "department_name" : "開発部",
                                      "employed" : true,
                                      "is_manager" : false,
                                      "is_new_graduate" : false,
                                      "note" : "備考",
                                      "office" : "東京事業所",
                                      "position_name" : "主任",
                                      "profession" : "ITエンジニア",
                                      "years_of_service" : 5
                                    }
                                  ],
                                  "fee_per_hour_in_yen" : 3000,
                                  "is_bank_account_registered" : true,
                                  "num_of_careers" : 1,
                                  "num_of_rated" : 43,
                                  "rating" : 4.2,
                                  "user_account_id" : 3
                                }
                              }
                            ]
                          }
                        }),
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultantDetail {
                        consultant_id: 3,
                        fee_per_hour_in_yen: 3000,
                        rating: Some("4.2".to_string()),
                        num_of_rated: 43,
                        careers: vec![ConsultantCareerDetail {
                            company_name: "テスト５（株）".to_string(),
                            department_name: Some("開発部".to_string()),
                            office: Some("東京事業所".to_string()),
                            years_of_service:
                                YEARS_OF_SERVICE_FIVE_YEARS_OR_MORE_LESS_THAN_TEN_YEARS.to_string(),
                            employed: true,
                            contract_type: "regular".to_string(),
                            profession: Some("ITエンジニア".to_string()),
                            annual_income_in_man_yen: Some(400),
                            is_manager: false,
                            position_name: Some("主任".to_string()),
                            is_new_graduate: false,
                            note: Some("備考".to_string()),
                        }],
                    }),
                )),
            },
            TestCase {
                name: "succeed in getting consultant 2".to_string(),
                input: Input {
                    account_id: 1,
                    consultant_id: 5,
                    op: ConsultantDetailOperationMock {
                        account_id: 1,
                        consultant_id: 5,
                        query_result: json!({
                          "took" : 6,
                          "timed_out" : false,
                          "_shards" : {
                            "total" : 1,
                            "successful" : 1,
                            "skipped" : 0,
                            "failed" : 0
                          },
                          "hits" : {
                            "total" : {
                              "value" : 1,
                              "relation" : "eq"
                            },
                            "max_score" : 1.0,
                            "hits" : [
                              {
                                "_index" : "users",
                                "_id" : "5",
                                "_score" : 1.0,
                                "_source" : {
                                  "careers" : [
                                    {
                                      "profession" : null,
                                      "note" : null,
                                      "department_name" : null,
                                      "position_name" : null,
                                      "is_new_graduate" : false,
                                      "office" : null,
                                      "is_manager" : false,
                                      "annual_income_in_man_yen" : null,
                                      "employed" : true,
                                      "career_id" : 5,
                                      "contract_type" : "regular",
                                      "years_of_service" : 10,
                                      "company_name" : "タナカ株式会社"
                                    }
                                  ],
                                  "fee_per_hour_in_yen" : 4500,
                                  "is_bank_account_registered" : true,
                                  "rating" : null,
                                  "user_account_id" : 5,
                                  "num_of_careers" : 1
                                }
                              }
                            ]
                          }
                        }),
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultantDetail {
                        consultant_id: 5,
                        fee_per_hour_in_yen: 4500,
                        rating: None,
                        num_of_rated: 0,
                        careers: vec![ConsultantCareerDetail {
                            company_name: "タナカ株式会社".to_string(),
                            department_name: None,
                            office: None,
                            years_of_service:
                                YEARS_OF_SERVICE_TEN_YEARS_OR_MORE_LESS_THAN_FIFTEEN_YEARS
                                    .to_string(),
                            employed: true,
                            contract_type: "regular".to_string(),
                            profession: None,
                            annual_income_in_man_yen: None,
                            is_manager: false,
                            position_name: None,
                            is_new_graduate: false,
                            note: None,
                        }],
                    }),
                )),
            },
            TestCase {
                name: "succeed in getting consultant 3".to_string(),
                input: Input {
                    account_id: 1,
                    consultant_id: 5,
                    op: ConsultantDetailOperationMock {
                        account_id: 1,
                        consultant_id: 5,
                        query_result: json!({
                          "took" : 6,
                          "timed_out" : false,
                          "_shards" : {
                            "total" : 1,
                            "successful" : 1,
                            "skipped" : 0,
                            "failed" : 0
                          },
                          "hits" : {
                            "total" : {
                              "value" : 1,
                              "relation" : "eq"
                            },
                            "max_score" : 1.0,
                            "hits" : [
                              {
                                "_index" : "users",
                                "_id" : "5",
                                "_score" : 1.0,
                                "_source" : {
                                  "careers" : [
                                    {
                                      "profession" : null,
                                      "note" : null,
                                      "department_name" : null,
                                      "position_name" : null,
                                      "is_new_graduate" : false,
                                      "office" : null,
                                      "is_manager" : false,
                                      "annual_income_in_man_yen" : null,
                                      "employed" : true,
                                      "career_id" : 5,
                                      "contract_type" : "regular",
                                      "years_of_service" : 2,
                                      "company_name" : "テスト１株式会社"
                                    },
                                    {
                                        "profession" : null,
                                        "note" : null,
                                        "department_name" : null,
                                        "position_name" : null,
                                        "is_new_graduate" : false,
                                        "office" : null,
                                        "is_manager" : false,
                                        "annual_income_in_man_yen" : null,
                                        "employed" : true,
                                        "career_id" : 6,
                                        "contract_type" : "regular",
                                        "years_of_service" : 3,
                                        "company_name" : "テスト２株式会社"
                                      },
                                      {
                                        "profession" : null,
                                        "note" : null,
                                        "department_name" : null,
                                        "position_name" : null,
                                        "is_new_graduate" : false,
                                        "office" : null,
                                        "is_manager" : false,
                                        "annual_income_in_man_yen" : null,
                                        "employed" : true,
                                        "career_id" : 7,
                                        "contract_type" : "regular",
                                        "years_of_service" : 5,
                                        "company_name" : "テスト３株式会社"
                                      },
                                      {
                                        "profession" : null,
                                        "note" : null,
                                        "department_name" : null,
                                        "position_name" : null,
                                        "is_new_graduate" : false,
                                        "office" : null,
                                        "is_manager" : false,
                                        "annual_income_in_man_yen" : null,
                                        "employed" : true,
                                        "career_id" : 8,
                                        "contract_type" : "regular",
                                        "years_of_service" : 10,
                                        "company_name" : "テスト４株式会社"
                                      },
                                      {
                                        "profession" : null,
                                        "note" : null,
                                        "department_name" : null,
                                        "position_name" : null,
                                        "is_new_graduate" : false,
                                        "office" : null,
                                        "is_manager" : false,
                                        "annual_income_in_man_yen" : null,
                                        "employed" : true,
                                        "career_id" : 9,
                                        "contract_type" : "regular",
                                        "years_of_service" : 15,
                                        "company_name" : "テスト５株式会社"
                                      },
                                      {
                                        "profession" : null,
                                        "note" : null,
                                        "department_name" : null,
                                        "position_name" : null,
                                        "is_new_graduate" : false,
                                        "office" : null,
                                        "is_manager" : false,
                                        "annual_income_in_man_yen" : null,
                                        "employed" : true,
                                        "career_id" : 10,
                                        "contract_type" : "regular",
                                        "years_of_service" : 20,
                                        "company_name" : "テスト６株式会社"
                                      },
                                      {
                                        "profession" : null,
                                        "note" : null,
                                        "department_name" : null,
                                        "position_name" : null,
                                        "is_new_graduate" : false,
                                        "office" : null,
                                        "is_manager" : false,
                                        "annual_income_in_man_yen" : null,
                                        "employed" : true,
                                        "career_id" : 11,
                                        "contract_type" : "regular",
                                        "years_of_service" : 19,
                                        "company_name" : "テスト７株式会社"
                                      },
                                      {
                                        "profession" : null,
                                        "note" : null,
                                        "department_name" : null,
                                        "position_name" : null,
                                        "is_new_graduate" : false,
                                        "office" : null,
                                        "is_manager" : false,
                                        "annual_income_in_man_yen" : null,
                                        "employed" : true,
                                        "career_id" : 12,
                                        "contract_type" : "regular",
                                        "years_of_service" : 14,
                                        "company_name" : "テスト８株式会社"
                                      },
                                  ],
                                  "fee_per_hour_in_yen" : 4500,
                                  "is_bank_account_registered" : true,
                                  "rating" : null,
                                  "user_account_id" : 5,
                                  "num_of_careers" : 8
                                }
                              }
                            ]
                          }
                        }),
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultantDetail {
                        consultant_id: 5,
                        fee_per_hour_in_yen: 4500,
                        rating: None,
                        num_of_rated: 0,
                        careers: vec![
                            ConsultantCareerDetail {
                                company_name: "テスト１株式会社".to_string(),
                                department_name: None,
                                office: None,
                                years_of_service: YEARS_OF_SERVICE_LESS_THAN_THREE_YEARS
                                    .to_string(),
                                employed: true,
                                contract_type: "regular".to_string(),
                                profession: None,
                                annual_income_in_man_yen: None,
                                is_manager: false,
                                position_name: None,
                                is_new_graduate: false,
                                note: None,
                            },
                            ConsultantCareerDetail {
                                company_name: "テスト２株式会社".to_string(),
                                department_name: None,
                                office: None,
                                years_of_service:
                                    YEARS_OF_SERVICE_THREE_YEARS_OR_MORE_LESS_THAN_FIVE_YEARS
                                        .to_string(),
                                employed: true,
                                contract_type: "regular".to_string(),
                                profession: None,
                                annual_income_in_man_yen: None,
                                is_manager: false,
                                position_name: None,
                                is_new_graduate: false,
                                note: None,
                            },
                            ConsultantCareerDetail {
                                company_name: "テスト３株式会社".to_string(),
                                department_name: None,
                                office: None,
                                years_of_service:
                                    YEARS_OF_SERVICE_FIVE_YEARS_OR_MORE_LESS_THAN_TEN_YEARS
                                        .to_string(),
                                employed: true,
                                contract_type: "regular".to_string(),
                                profession: None,
                                annual_income_in_man_yen: None,
                                is_manager: false,
                                position_name: None,
                                is_new_graduate: false,
                                note: None,
                            },
                            ConsultantCareerDetail {
                                company_name: "テスト４株式会社".to_string(),
                                department_name: None,
                                office: None,
                                years_of_service:
                                    YEARS_OF_SERVICE_TEN_YEARS_OR_MORE_LESS_THAN_FIFTEEN_YEARS
                                        .to_string(),
                                employed: true,
                                contract_type: "regular".to_string(),
                                profession: None,
                                annual_income_in_man_yen: None,
                                is_manager: false,
                                position_name: None,
                                is_new_graduate: false,
                                note: None,
                            },
                            ConsultantCareerDetail {
                                company_name: "テスト５株式会社".to_string(),
                                department_name: None,
                                office: None,
                                years_of_service:
                                    YEARS_OF_SERVICE_FIFTEEN_YEARS_OR_MORE_LESS_THAN_TWENTY_YEARS
                                        .to_string(),
                                employed: true,
                                contract_type: "regular".to_string(),
                                profession: None,
                                annual_income_in_man_yen: None,
                                is_manager: false,
                                position_name: None,
                                is_new_graduate: false,
                                note: None,
                            },
                            ConsultantCareerDetail {
                                company_name: "テスト６株式会社".to_string(),
                                department_name: None,
                                office: None,
                                years_of_service: YEARS_OF_SERVICE_TWENTY_YEARS_OR_MORE.to_string(),
                                employed: true,
                                contract_type: "regular".to_string(),
                                profession: None,
                                annual_income_in_man_yen: None,
                                is_manager: false,
                                position_name: None,
                                is_new_graduate: false,
                                note: None,
                            },
                            ConsultantCareerDetail {
                                company_name: "テスト７株式会社".to_string(),
                                department_name: None,
                                office: None,
                                years_of_service:
                                    YEARS_OF_SERVICE_FIFTEEN_YEARS_OR_MORE_LESS_THAN_TWENTY_YEARS
                                        .to_string(),
                                employed: true,
                                contract_type: "regular".to_string(),
                                profession: None,
                                annual_income_in_man_yen: None,
                                is_manager: false,
                                position_name: None,
                                is_new_graduate: false,
                                note: None,
                            },
                            ConsultantCareerDetail {
                                company_name: "テスト８株式会社".to_string(),
                                department_name: None,
                                office: None,
                                years_of_service:
                                    YEARS_OF_SERVICE_TEN_YEARS_OR_MORE_LESS_THAN_FIFTEEN_YEARS
                                        .to_string(),
                                employed: true,
                                contract_type: "regular".to_string(),
                                profession: None,
                                annual_income_in_man_yen: None,
                                is_manager: false,
                                position_name: None,
                                is_new_graduate: false,
                                note: None,
                            },
                        ],
                    }),
                )),
            },
            TestCase {
                name: "succeed in getting consultant 4".to_string(),
                input: Input {
                    account_id: 1,
                    consultant_id: 5,
                    op: ConsultantDetailOperationMock {
                        account_id: 1,
                        consultant_id: 5,
                        query_result: json!({
                          "took" : 6,
                          "timed_out" : false,
                          "_shards" : {
                            "total" : 1,
                            "successful" : 1,
                            "skipped" : 0,
                            "failed" : 0
                          },
                          "hits" : {
                            "total" : {
                              "value" : 1,
                              "relation" : "eq"
                            },
                            "max_score" : 1.0,
                            "hits" : [
                              {
                                "_index" : "users",
                                "_id" : "5",
                                "_score" : 1.0,
                                "_source" : {
                                  "careers" : [
                                    {
                                      "profession" : null,
                                      "note" : null,
                                      "department_name" : null,
                                      "position_name" : null,
                                      "is_new_graduate" : false,
                                      "office" : null,
                                      "is_manager" : false,
                                      "annual_income_in_man_yen" : null,
                                      "employed" : true,
                                      "career_id" : 5,
                                      "contract_type" : "regular",
                                      "years_of_service" : 0,
                                      "company_name" : "テスト１株式会社"
                                    },
                                    {
                                        "profession" : null,
                                        "note" : null,
                                        "department_name" : null,
                                        "position_name" : null,
                                        "is_new_graduate" : false,
                                        "office" : null,
                                        "is_manager" : false,
                                        "annual_income_in_man_yen" : null,
                                        "employed" : true,
                                        "career_id" : 6,
                                        "contract_type" : "regular",
                                        "years_of_service" : 4,
                                        "company_name" : "テスト２株式会社"
                                      },
                                      {
                                        "profession" : null,
                                        "note" : null,
                                        "department_name" : null,
                                        "position_name" : null,
                                        "is_new_graduate" : false,
                                        "office" : null,
                                        "is_manager" : false,
                                        "annual_income_in_man_yen" : null,
                                        "employed" : true,
                                        "career_id" : 7,
                                        "contract_type" : "regular",
                                        "years_of_service" : 9,
                                        "company_name" : "テスト３株式会社"
                                      },
                                  ],
                                  "fee_per_hour_in_yen" : 4500,
                                  "is_bank_account_registered" : true,
                                  "rating" : null,
                                  "user_account_id" : 5,
                                  "num_of_careers" : 3
                                }
                              }
                            ]
                          }
                        }),
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultantDetail {
                        consultant_id: 5,
                        fee_per_hour_in_yen: 4500,
                        rating: None,
                        num_of_rated: 0,
                        careers: vec![
                            ConsultantCareerDetail {
                                company_name: "テスト１株式会社".to_string(),
                                department_name: None,
                                office: None,
                                years_of_service: YEARS_OF_SERVICE_LESS_THAN_THREE_YEARS
                                    .to_string(),
                                employed: true,
                                contract_type: "regular".to_string(),
                                profession: None,
                                annual_income_in_man_yen: None,
                                is_manager: false,
                                position_name: None,
                                is_new_graduate: false,
                                note: None,
                            },
                            ConsultantCareerDetail {
                                company_name: "テスト２株式会社".to_string(),
                                department_name: None,
                                office: None,
                                years_of_service:
                                    YEARS_OF_SERVICE_THREE_YEARS_OR_MORE_LESS_THAN_FIVE_YEARS
                                        .to_string(),
                                employed: true,
                                contract_type: "regular".to_string(),
                                profession: None,
                                annual_income_in_man_yen: None,
                                is_manager: false,
                                position_name: None,
                                is_new_graduate: false,
                                note: None,
                            },
                            ConsultantCareerDetail {
                                company_name: "テスト３株式会社".to_string(),
                                department_name: None,
                                office: None,
                                years_of_service:
                                    YEARS_OF_SERVICE_FIVE_YEARS_OR_MORE_LESS_THAN_TEN_YEARS
                                        .to_string(),
                                employed: true,
                                contract_type: "regular".to_string(),
                                profession: None,
                                annual_income_in_man_yen: None,
                                is_manager: false,
                                position_name: None,
                                is_new_graduate: false,
                                note: None,
                            },
                        ],
                    }),
                )),
            },
        ]
    });

    #[tokio::test]
    async fn test_handle_consultants_search() {
        for test_case in TEST_CASE_SET.iter() {
            let resp = handle_consultant_detail(
                test_case.input.account_id,
                test_case.input.consultant_id,
                test_case.input.op.clone(),
            )
            .await;

            let message = format!("test case \"{}\" failed", test_case.name.clone());
            if test_case.expected.is_ok() {
                let result = resp.expect("failed to get Ok");
                let expected_result = test_case.expected.as_ref().expect("failed to get Ok");
                assert_eq!(expected_result.0, result.0, "{}", message);
                assert_eq!(expected_result.1 .0, result.1 .0, "{}", message);
            } else {
                let result = resp.expect_err("failed to get Err");
                let expected_result = test_case.expected.as_ref().expect_err("failed to get Err");
                assert_eq!(expected_result.0, result.0, "{}", message);
                assert_eq!(expected_result.1 .0, result.1 .0, "{}", message);
            }
        }
    }
}
