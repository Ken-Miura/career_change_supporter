// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::util::validator::{has_control_char, has_non_new_line_control_char, SYMBOL_CHAR_RE};
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::ActiveValue::NotSet;
use entity::sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::handlers::session::authentication::authenticated_handlers::admin::Admin;

const MIN_TITLE_SIZE: usize = 1;
const MAX_TITLE_SIZE: usize = 256;

const MIN_BODY_SIZE: usize = 1;
const MAX_BODY_SIZE: usize = 16384;

pub(crate) async fn post_set_news_req(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    Json(req): Json<SetNewsReq>,
) -> RespResult<SetNewsReqResult> {
    let op = SetNewsReqOperationImpl { pool };
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    handle_set_news_req(req.title, req.body, current_date_time, &op).await
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct SetNewsReq {
    title: String,
    body: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct SetNewsReqResult {}

#[async_trait]
trait SetNewsReqOperation {
    async fn set_news(
        &self,
        title: String,
        body: String,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct SetNewsReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl SetNewsReqOperation for SetNewsReqOperationImpl {
    async fn set_news(
        &self,
        title: String,
        body: String,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        let am = entity::news::ActiveModel {
            news_id: NotSet,
            title: Set(title.clone()),
            body: Set(body.clone()),
            published_at: Set(current_date_time),
        };
        am.insert(&self.pool).await.map_err(|e| {
            error!(
                "failed to insert news (title: {}, body: {}, current_date_time: {}): {}",
                title, body, current_date_time, e
            );
            unexpected_err_resp()
        })?;
        Ok(())
    }
}

async fn handle_set_news_req(
    title: String,
    body: String,
    current_date_time: DateTime<FixedOffset>,
    op: &impl SetNewsReqOperation,
) -> RespResult<SetNewsReqResult> {
    validate_title(&title)?;
    validate_body(&body)?;

    op.set_news(title, body, current_date_time).await?;

    Ok((StatusCode::OK, Json(SetNewsReqResult {})))
}

fn validate_title(title: &str) -> Result<(), ErrResp> {
    let length = title.chars().count();
    if !(MIN_TITLE_SIZE..=MAX_TITLE_SIZE).contains(&length) {
        error!("invalid title length ({})", length);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidTitleLength as u32,
            }),
        ));
    }
    if has_control_char(title) {
        error!("title has control char ({:?})", title.as_bytes());
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::IllegalTitle as u32,
            }),
        ));
    }
    if SYMBOL_CHAR_RE.is_match(title) {
        error!("title has control char ({:?})", title.as_bytes());
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::IllegalTitle as u32,
            }),
        ));
    }
    Ok(())
}

fn validate_body(body: &str) -> Result<(), ErrResp> {
    let length = body.chars().count();
    if !(MIN_BODY_SIZE..=MAX_BODY_SIZE).contains(&length) {
        error!("invalid body length ({})", length);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidBodyLength as u32,
            }),
        ));
    }
    if has_non_new_line_control_char(body) {
        error!("body has non new line control char ({:?})", body.as_bytes());
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::IllegalBody as u32,
            }),
        ));
    }
    if SYMBOL_CHAR_RE.is_match(body) {
        error!("body has control char ({:?})", body.as_bytes());
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::IllegalBody as u32,
            }),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use chrono::TimeZone;

    use super::*;

    struct SetNewsReqOperationMock {
        title: String,
        body: String,
        current_date_time: DateTime<FixedOffset>,
    }

    #[async_trait]
    impl SetNewsReqOperation for SetNewsReqOperationMock {
        async fn set_news(
            &self,
            title: String,
            body: String,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<(), ErrResp> {
            assert_eq!(title, self.title);
            assert_eq!(body, self.body);
            assert_eq!(current_date_time, self.current_date_time);
            Ok(())
        }
    }

    #[tokio::test]
    async fn handle_set_news_req_success() {
        let title = "タイトル".to_string();
        let body = r"ライン１
      ライン２
      ライン３"
            .to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 11, 21, 32, 21)
            .unwrap();
        let op = SetNewsReqOperationMock {
            title: title.clone(),
            body: body.clone(),
            current_date_time,
        };

        let result = handle_set_news_req(title, body, current_date_time, &op)
            .await
            .expect("failed to get Ok");

        assert_eq!(result.0, StatusCode::OK);
        assert_eq!(result.1 .0, SetNewsReqResult {});
    }

    #[tokio::test]
    async fn handle_set_news_req_success_title_min() {
        let title = "あ".to_string();
        let body = r"ライン１
      ライン２
      ライン３"
            .to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 11, 21, 32, 21)
            .unwrap();
        let op = SetNewsReqOperationMock {
            title: title.clone(),
            body: body.clone(),
            current_date_time,
        };

        let result = handle_set_news_req(title, body, current_date_time, &op)
            .await
            .expect("failed to get Ok");

        assert_eq!(result.0, StatusCode::OK);
        assert_eq!(result.1 .0, SetNewsReqResult {});
    }

    #[tokio::test]
    async fn handle_set_news_req_success_title_max() {
        let title = "ああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ".to_string();
        let body = r"ライン１
      ライン２
      ライン３"
            .to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 11, 21, 32, 21)
            .unwrap();
        let op = SetNewsReqOperationMock {
            title: title.clone(),
            body: body.clone(),
            current_date_time,
        };

        let result = handle_set_news_req(title, body, current_date_time, &op)
            .await
            .expect("failed to get Ok");

        assert_eq!(result.0, StatusCode::OK);
        assert_eq!(result.1 .0, SetNewsReqResult {});
    }

    #[tokio::test]
    async fn handle_set_news_req_fail_title_empty() {
        let title = "".to_string();
        let body = r"ライン１
      ライン２
      ライン３"
            .to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 11, 21, 32, 21)
            .unwrap();
        let op = SetNewsReqOperationMock {
            title: title.clone(),
            body: body.clone(),
            current_date_time,
        };

        let result = handle_set_news_req(title, body, current_date_time, &op)
            .await
            .expect_err("failed to get Err");

        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(result.1.code, Code::InvalidTitleLength as u32);
    }

    #[tokio::test]
    async fn handle_set_news_req_fail_title_over_max() {
        let title = "あああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああああ".to_string();
        let body = r"ライン１
      ライン２
      ライン３"
            .to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 11, 21, 32, 21)
            .unwrap();
        let op = SetNewsReqOperationMock {
            title: title.clone(),
            body: body.clone(),
            current_date_time,
        };

        let result = handle_set_news_req(title, body, current_date_time, &op)
            .await
            .expect_err("failed to get Err");

        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(result.1.code, Code::InvalidTitleLength as u32);
    }

    #[tokio::test]
    async fn handle_set_news_req_fail_title_control_char() {
        let title = "\u{000A}\u{000D}".to_string();
        let body = r"ライン１
      ライン２
      ライン３"
            .to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 11, 21, 32, 21)
            .unwrap();
        let op = SetNewsReqOperationMock {
            title: title.clone(),
            body: body.clone(),
            current_date_time,
        };

        let result = handle_set_news_req(title, body, current_date_time, &op)
            .await
            .expect_err("failed to get Err");

        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(result.1.code, Code::IllegalTitle as u32);
    }

    #[tokio::test]
    async fn handle_set_news_req_fail_title_symbol() {
        let title = "<script><alert('test')/script>".to_string();
        let body = r"ライン１
      ライン２
      ライン３"
            .to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 11, 21, 32, 21)
            .unwrap();
        let op = SetNewsReqOperationMock {
            title: title.clone(),
            body: body.clone(),
            current_date_time,
        };

        let result = handle_set_news_req(title, body, current_date_time, &op)
            .await
            .expect_err("failed to get Err");

        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(result.1.code, Code::IllegalTitle as u32);
    }

    #[tokio::test]
    async fn handle_set_news_req_success_body_min() {
        let title = "タイトル".to_string();
        let body = r"あ".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 11, 21, 32, 21)
            .unwrap();
        let op = SetNewsReqOperationMock {
            title: title.clone(),
            body: body.clone(),
            current_date_time,
        };

        let result = handle_set_news_req(title, body, current_date_time, &op)
            .await
            .expect("failed to get Ok");

        assert_eq!(result.0, StatusCode::OK);
        assert_eq!(result.1 .0, SetNewsReqResult {});
    }

    #[tokio::test]
    async fn handle_set_news_req_success_body_max() {
        let title = "タイトル".to_string();
        let mut body = String::with_capacity(MAX_BODY_SIZE);
        for _ in 0..MAX_BODY_SIZE {
            body.push('a');
        }
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 11, 21, 32, 21)
            .unwrap();
        let op = SetNewsReqOperationMock {
            title: title.clone(),
            body: body.clone(),
            current_date_time,
        };

        let result = handle_set_news_req(title, body, current_date_time, &op)
            .await
            .expect("failed to get Ok");

        assert_eq!(result.0, StatusCode::OK);
        assert_eq!(result.1 .0, SetNewsReqResult {});
    }

    #[tokio::test]
    async fn handle_set_news_req_fail_body_empty() {
        let title = "タイトル".to_string();
        let body = r"".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 11, 21, 32, 21)
            .unwrap();
        let op = SetNewsReqOperationMock {
            title: title.clone(),
            body: body.clone(),
            current_date_time,
        };

        let result = handle_set_news_req(title, body, current_date_time, &op)
            .await
            .expect_err("failed to get Err");

        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(result.1.code, Code::InvalidBodyLength as u32);
    }

    #[tokio::test]
    async fn handle_set_news_req_fail_body_over_max() {
        let title = "タイトル".to_string();
        let mut body = String::with_capacity(MAX_BODY_SIZE);
        for _ in 0..(MAX_BODY_SIZE + 1) {
            body.push('a');
        }
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 11, 21, 32, 21)
            .unwrap();
        let op = SetNewsReqOperationMock {
            title: title.clone(),
            body: body.clone(),
            current_date_time,
        };

        let result = handle_set_news_req(title, body, current_date_time, &op)
            .await
            .expect_err("failed to get Err");

        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(result.1.code, Code::InvalidBodyLength as u32);
    }

    #[tokio::test]
    async fn handle_set_news_req_fail_body_non_new_line_control_char() {
        let title = "タイトル".to_string();
        let body = "\u{0009}".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 11, 21, 32, 21)
            .unwrap();
        let op = SetNewsReqOperationMock {
            title: title.clone(),
            body: body.clone(),
            current_date_time,
        };

        let result = handle_set_news_req(title, body, current_date_time, &op)
            .await
            .expect_err("failed to get Err");

        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(result.1.code, Code::IllegalBody as u32);
    }

    #[tokio::test]
    async fn handle_set_news_req_fail_body_symbol() {
        let title = "タイトル".to_string();
        let body = "<script><alert('test')/script>".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 11, 21, 32, 21)
            .unwrap();
        let op = SetNewsReqOperationMock {
            title: title.clone(),
            body: body.clone(),
            current_date_time,
        };

        let result = handle_set_news_req(title, body, current_date_time, &op)
            .await
            .expect_err("failed to get Err");

        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(result.1.code, Code::IllegalBody as u32);
    }
}
