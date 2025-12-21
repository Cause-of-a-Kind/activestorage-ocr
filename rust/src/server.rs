use crate::config::Config;
use crate::error::OcrError;
use crate::ocr::OcrProcessor;
use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Multipart, State},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::Serialize;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;
use tower_http::trace::TraceLayer;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<OcrProcessor>,
    pub config: Arc<Config>,
}

/// OCR response
#[derive(Serialize)]
pub struct OcrResponse {
    pub text: String,
    pub confidence: f32,
    pub processing_time_ms: u64,
    pub warnings: Vec<String>,
}

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Server info response
#[derive(Serialize)]
pub struct InfoResponse {
    pub version: String,
    pub supported_formats: Vec<String>,
    pub supported_languages: Vec<String>,
    pub max_file_size_bytes: usize,
    pub default_language: String,
}

/// Run the HTTP server
pub async fn run(config: Config) -> anyhow::Result<()> {
    let engine = OcrProcessor::new(&config)?;
    let addr = format!("{}:{}", config.host, config.port);
    let max_file_size = config.max_file_size;

    let state = AppState {
        engine: Arc::new(engine),
        config: Arc::new(config),
    };

    let app = Router::new()
        .route("/ocr", post(handle_ocr))
        .route("/health", get(handle_health))
        .route("/info", get(handle_info))
        .layer(DefaultBodyLimit::max(max_file_size))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Handle OCR requests
async fn handle_ocr(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<OcrResponse>, OcrError> {
    let start = Instant::now();

    let mut file_data: Option<Bytes> = None;
    let mut content_type: Option<String> = None;
    let mut languages: Option<String> = None;

    // Parse multipart form
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| OcrError::InvalidRequest(format!("Failed to parse multipart: {}", e)))?
    {
        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "file" => {
                content_type = field.content_type().map(|s| s.to_string());
                file_data = Some(field.bytes().await.map_err(|e| {
                    OcrError::InvalidRequest(format!("Failed to read file data: {}", e))
                })?);
            }
            "languages" => {
                languages =
                    Some(field.text().await.map_err(|e| {
                        OcrError::InvalidRequest(format!("Invalid languages: {}", e))
                    })?);
            }
            _ => {
                // Ignore unknown fields
            }
        }
    }

    // Validate file was provided
    let data = file_data.ok_or(OcrError::MissingFile)?;

    // Check file size
    if data.len() > state.config.max_file_size {
        return Err(OcrError::ImageTooLarge {
            size: data.len(),
            max: state.config.max_file_size,
        });
    }

    // Validate content type and get extension
    let mime = content_type.unwrap_or_else(|| "application/octet-stream".to_string());
    if !state.engine.supported_formats().contains(&mime) && !mime.starts_with("image/") {
        tracing::warn!("Received file with content type: {}", mime);
    }

    // Determine file extension from mime type
    let extension = match mime.as_str() {
        "image/png" => ".png",
        "image/jpeg" => ".jpg",
        "image/gif" => ".gif",
        "image/bmp" => ".bmp",
        "image/webp" => ".webp",
        "image/tiff" => ".tiff",
        "application/pdf" => ".pdf",
        _ => ".tmp",
    };

    // Write to temp file with proper extension
    let mut temp_file = tempfile::Builder::new()
        .suffix(extension)
        .tempfile()
        .map_err(|e| OcrError::Internal(format!("Failed to create temp file: {}", e)))?;

    temp_file
        .write_all(&data)
        .map_err(|e| OcrError::Internal(format!("Failed to write temp file: {}", e)))?;

    // Perform OCR (ocrs doesn't support language selection yet, it's English-only)
    let _ = languages; // Ignore languages for now
    let result = state.engine.process(temp_file.path())?;

    let processing_time_ms = start.elapsed().as_millis() as u64;

    tracing::info!(
        "OCR completed in {}ms, confidence: {:.2}, text length: {}",
        processing_time_ms,
        result.confidence,
        result.text.len()
    );

    Ok(Json(OcrResponse {
        text: result.text,
        confidence: result.confidence,
        processing_time_ms,
        warnings: result.warnings,
    }))
}

/// Handle health check requests
async fn handle_health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Handle info requests
async fn handle_info(State(state): State<AppState>) -> impl IntoResponse {
    Json(InfoResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        supported_formats: state.engine.supported_formats(),
        // ocrs currently only supports English/Latin alphabet
        supported_languages: vec!["eng".to_string()],
        max_file_size_bytes: state.config.max_file_size,
        default_language: state.config.default_language.clone(),
    })
}
