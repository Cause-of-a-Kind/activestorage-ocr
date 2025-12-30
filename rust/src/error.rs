use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OcrError {
    #[error("Failed to initialize OCR engine: {0}")]
    InitializationError(String),

    #[error("Failed to process image: {0}")]
    ProcessingError(String),

    #[error("Preprocessing failed: {0}")]
    PreprocessingError(String),

    #[error("Unsupported image format: {0}")]
    #[allow(dead_code)]
    UnsupportedFormat(String),

    #[error("Image too large: {size} bytes (max: {max} bytes)")]
    ImageTooLarge { size: usize, max: usize },

    #[error("Missing file in request")]
    MissingFile,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

impl IntoResponse for OcrError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            OcrError::InitializationError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INIT_ERROR"),
            OcrError::ProcessingError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "PROCESSING_ERROR"),
            OcrError::PreprocessingError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "PREPROCESSING_ERROR")
            }
            OcrError::UnsupportedFormat(_) => (StatusCode::BAD_REQUEST, "UNSUPPORTED_FORMAT"),
            OcrError::ImageTooLarge { .. } => (StatusCode::PAYLOAD_TOO_LARGE, "IMAGE_TOO_LARGE"),
            OcrError::MissingFile => (StatusCode::BAD_REQUEST, "MISSING_FILE"),
            OcrError::InvalidRequest(_) => (StatusCode::BAD_REQUEST, "INVALID_REQUEST"),
            OcrError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        };

        let body = Json(ErrorResponse {
            error: self.to_string(),
            code: code.to_string(),
        });

        (status, body).into_response()
    }
}
