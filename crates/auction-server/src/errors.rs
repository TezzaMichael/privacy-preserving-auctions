use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use auction_core::{errors::AuctionError, responses::ErrorResponse};

#[derive(Debug)]
pub struct ApiError(pub AuctionError);

impl From<AuctionError> for ApiError {
    fn from(e: AuctionError) -> Self { Self(e) }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let code = self.0.status_code();
        let status = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(ErrorResponse::new(self.0.to_string(), code))).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        Self(AuctionError::Internal(format!("JSON error: {}", e)))
    }
}