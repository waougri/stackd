use axum::Json;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StackdMessage {
    pub message: String,
}

pub enum AppError {
    SqlError(sqlx::Error),
    UUIDParseError(uuid::Error),
    NotFound(String),
    NotAuthorized(String),
    BadRequest(String),
}
impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Self::SqlError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DB error"),
            Self::NotFound(_id) => (StatusCode::NOT_FOUND, "Not found"),
            Self::NotAuthorized(_id) => (StatusCode::UNAUTHORIZED, "Not authorized"),
            Self::BadRequest(_message) => (StatusCode::BAD_REQUEST, "Bad request"),
            Self::UUIDParseError(_id) => (
                StatusCode::BAD_REQUEST,
                "UUID parse error",
            ),
        };

        (
            status,
            Json(StackdMessage {
                message: message.into(),
            }),
        )
            .into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        Self::SqlError(error)
    }
}

pub type HttpResponse<T> = Result<(StatusCode, Json<T>), AppError>;
