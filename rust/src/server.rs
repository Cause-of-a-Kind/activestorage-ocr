use crate::config::Config;
use crate::engine::OcrEngine;
use crate::engines::EngineRegistry;
use crate::error::OcrError;
use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Multipart, Path, State},
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
    pub registry: Arc<EngineRegistry>,
    pub config: Arc<Config>,
}

/// OCR response
#[derive(Serialize)]
pub struct OcrResponse {
    pub text: String,
    pub confidence: f32,
    pub processing_time_ms: u64,
    pub warnings: Vec<String>,
    pub engine: String,
}

/// Engine info for /info response
#[derive(Serialize)]
pub struct EngineInfoResponse {
    pub name: String,
    pub description: String,
    pub supported_formats: Vec<String>,
    pub supported_languages: Vec<String>,
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
    pub available_engines: Vec<EngineInfoResponse>,
    pub default_engine: String,
    pub max_file_size_bytes: usize,
    pub default_language: String,
}

/// Run the HTTP server
pub async fn run(config: Config) -> anyhow::Result<()> {
    let registry = EngineRegistry::new(&config)?;
    let addr = format!("{}:{}", config.host, config.port);
    let max_file_size = config.max_file_size;

    tracing::info!(
        "Available engines: {:?}",
        registry.list()
    );

    let state = AppState {
        registry: Arc::new(registry),
        config: Arc::new(config),
    };

    let app = Router::new()
        .route("/ocr", post(handle_ocr))
        .route("/ocr/{engine}", post(handle_ocr_with_engine))
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

/// Handle OCR requests (uses default engine)
async fn handle_ocr(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<Json<OcrResponse>, OcrError> {
    let engine = state.registry.default().ok_or_else(|| {
        OcrError::InitializationError("No default engine available".to_string())
    })?;

    process_ocr_request(state, engine, multipart).await
}

/// Handle OCR requests with specific engine
async fn handle_ocr_with_engine(
    State(state): State<AppState>,
    Path(engine_name): Path<String>,
    multipart: Multipart,
) -> Result<Json<OcrResponse>, OcrError> {
    let engine = state.registry.get(&engine_name).ok_or_else(|| {
        OcrError::InvalidRequest(format!(
            "Unknown engine '{}'. Available engines: {:?}",
            engine_name,
            state.registry.list()
        ))
    })?;

    process_ocr_request(state, engine, multipart).await
}

/// Common OCR processing logic
async fn process_ocr_request(
    state: AppState,
    engine: Arc<dyn OcrEngine>,
    mut multipart: Multipart,
) -> Result<Json<OcrResponse>, OcrError> {
    let start = Instant::now();
    let engine_name = engine.name().to_string();

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
    if !engine.supported_formats().contains(&mime) && !mime.starts_with("image/") {
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

    // Perform OCR
    let _ = languages; // TODO: Pass to engine if supported
    let result = engine.process(temp_file.path())?;

    let processing_time_ms = start.elapsed().as_millis() as u64;

    tracing::info!(
        "[{}] OCR completed in {}ms, confidence: {:.2}, text length: {}",
        engine_name,
        processing_time_ms,
        result.confidence,
        result.text.len()
    );

    Ok(Json(OcrResponse {
        text: result.text,
        confidence: result.confidence,
        processing_time_ms,
        warnings: result.warnings,
        engine: engine_name,
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
    let engines: Vec<EngineInfoResponse> = state
        .registry
        .info()
        .into_iter()
        .map(|e| EngineInfoResponse {
            name: e.name.to_string(),
            description: e.description.to_string(),
            supported_formats: e.supported_formats,
            supported_languages: e.supported_languages,
        })
        .collect();

    Json(InfoResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        available_engines: engines,
        default_engine: state.registry.default_name().to_string(),
        max_file_size_bytes: state.config.max_file_size,
        default_language: state.config.default_language.clone(),
    })
}
