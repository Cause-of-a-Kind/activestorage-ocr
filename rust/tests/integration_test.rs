use reqwest::multipart::{Form, Part};
use serde::Deserialize;
use std::fs;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

// Use atomic counter to give each test a unique port
static PORT_COUNTER: AtomicU16 = AtomicU16::new(9400);

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OcrResponse {
    text: String,
    confidence: f32,
    processing_time_ms: u64,
    warnings: Vec<String>,
    engine: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct EngineInfo {
    name: String,
    description: String,
    supported_formats: Vec<String>,
    supported_languages: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct InfoResponse {
    version: String,
    default_engine: String,
    available_engines: Vec<EngineInfo>,
    max_file_size_bytes: usize,
    default_language: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HealthResponse {
    status: String,
    version: String,
}

struct TestServer {
    child: Child,
    port: u16,
}

impl TestServer {
    fn start() -> Self {
        let port = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);

        let child = Command::new(env!("CARGO_BIN_EXE_activestorage-ocr-server"))
            .args(["--host", "127.0.0.1", "--port", &port.to_string()])
            .spawn()
            .expect("Failed to start server");

        // Wait for server to be ready
        std::thread::sleep(Duration::from_secs(4));

        Self { child, port }
    }

    fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

fn test_fixture_path(filename: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/tests/fixtures/{}", manifest_dir, filename)
}

async fn test_ocr_file(
    client: &reqwest::Client,
    base_url: &str,
    filename: &str,
    mime_type: &str,
) -> OcrResponse {
    let path = test_fixture_path(filename);
    let file_bytes = fs::read(&path).expect(&format!("Failed to read {}", path));

    let part = Part::bytes(file_bytes)
        .file_name(filename.to_string())
        .mime_str(mime_type)
        .unwrap();

    let form = Form::new().part("file", part);

    let response = client
        .post(&format!("{}/ocr", base_url))
        .multipart(form)
        .send()
        .await
        .expect("Failed to send request");

    response.json().await.expect("Failed to parse response")
}

#[tokio::test]
async fn test_health_endpoint() {
    let server = TestServer::start();
    let client = reqwest::Client::new();

    let response: HealthResponse = client
        .get(&format!("{}/health", server.base_url()))
        .send()
        .await
        .expect("Failed to send request")
        .json()
        .await
        .expect("Failed to parse response");

    assert_eq!(response.status, "ok");
}

#[tokio::test]
async fn test_ocr_png() {
    let server = TestServer::start();
    let client = reqwest::Client::new();

    let result = test_ocr_file(&client, &server.base_url(), "sample_text.png", "image/png").await;

    assert!(result.text.contains("Hello"));
    assert!(result.text.contains("World"));
    assert!(result.text.contains("OCR"));
    assert!(result.text.contains("12345"));
    assert!(result.confidence > 0.0);
    assert!(result.processing_time_ms > 0);
}

#[tokio::test]
async fn test_ocr_jpeg() {
    let server = TestServer::start();
    let client = reqwest::Client::new();

    let result = test_ocr_file(&client, &server.base_url(), "sample_text.jpg", "image/jpeg").await;

    assert!(result.text.contains("Hello"));
    assert!(result.text.contains("World"));
    assert!(result.confidence > 0.0);
}

#[tokio::test]
async fn test_ocr_gif() {
    let server = TestServer::start();
    let client = reqwest::Client::new();

    let result = test_ocr_file(&client, &server.base_url(), "sample_text.gif", "image/gif").await;

    assert!(result.text.contains("Hello"));
    assert!(result.text.contains("World"));
    assert!(result.confidence > 0.0);
}

#[tokio::test]
async fn test_ocr_bmp() {
    let server = TestServer::start();
    let client = reqwest::Client::new();

    let result = test_ocr_file(&client, &server.base_url(), "sample_text.bmp", "image/bmp").await;

    assert!(result.text.contains("Hello"));
    assert!(result.text.contains("World"));
    assert!(result.confidence > 0.0);
}

#[tokio::test]
async fn test_ocr_webp() {
    let server = TestServer::start();
    let client = reqwest::Client::new();

    let result = test_ocr_file(
        &client,
        &server.base_url(),
        "sample_text.webp",
        "image/webp",
    )
    .await;

    assert!(result.text.contains("Hello"));
    assert!(result.text.contains("World"));
    assert!(result.confidence > 0.0);
}

#[tokio::test]
async fn test_ocr_tiff() {
    let server = TestServer::start();
    let client = reqwest::Client::new();

    let result = test_ocr_file(
        &client,
        &server.base_url(),
        "sample_text.tiff",
        "image/tiff",
    )
    .await;

    assert!(result.text.contains("Hello"));
    assert!(result.text.contains("World"));
    assert!(result.confidence > 0.0);
}

#[tokio::test]
async fn test_ocr_pdf() {
    let server = TestServer::start();
    let client = reqwest::Client::new();

    let result = test_ocr_file(
        &client,
        &server.base_url(),
        "sample_text.pdf",
        "application/pdf",
    )
    .await;

    assert!(result.text.contains("Hello"));
    assert!(result.text.contains("World"));
    assert!(result.text.contains("12345"));
    assert!(result.confidence > 0.0);
}

#[tokio::test]
async fn test_ocr_returns_engine_field() {
    let server = TestServer::start();
    let client = reqwest::Client::new();

    let result = test_ocr_file(&client, &server.base_url(), "sample_text.png", "image/png").await;

    // Should return the engine name in the response
    assert!(result.engine.is_some());
    let engine = result.engine.unwrap();
    // Default engine is ocrs
    assert_eq!(engine, "ocrs");
}

#[tokio::test]
async fn test_info_endpoint() {
    let server = TestServer::start();
    let client = reqwest::Client::new();

    let response: InfoResponse = client
        .get(&format!("{}/info", server.base_url()))
        .send()
        .await
        .expect("Failed to send request")
        .json()
        .await
        .expect("Failed to parse response");

    // Check version is present
    assert!(!response.version.is_empty());

    // Check default engine
    assert_eq!(response.default_engine, "ocrs");

    // Check available engines includes at least ocrs
    let engine_names: Vec<&str> = response
        .available_engines
        .iter()
        .map(|e| e.name.as_str())
        .collect();
    assert!(engine_names.contains(&"ocrs"));

    // Check supported formats are available on the ocrs engine
    let ocrs_engine = response
        .available_engines
        .iter()
        .find(|e| e.name == "ocrs")
        .expect("ocrs engine should be available");
    assert!(ocrs_engine
        .supported_formats
        .contains(&"image/png".to_string()));
    assert!(ocrs_engine
        .supported_formats
        .contains(&"application/pdf".to_string()));
}

async fn test_ocr_file_with_engine(
    client: &reqwest::Client,
    base_url: &str,
    filename: &str,
    mime_type: &str,
    engine: &str,
) -> OcrResponse {
    let path = test_fixture_path(filename);
    let file_bytes = fs::read(&path).expect(&format!("Failed to read {}", path));

    let part = Part::bytes(file_bytes)
        .file_name(filename.to_string())
        .mime_str(mime_type)
        .unwrap();

    let form = Form::new().part("file", part);

    let response = client
        .post(&format!("{}/ocr/{}", base_url, engine))
        .multipart(form)
        .send()
        .await
        .expect("Failed to send request");

    assert!(
        response.status().is_success(),
        "Expected success status, got {} for /ocr/{}",
        response.status(),
        engine
    );

    response.json().await.expect("Failed to parse OCR response")
}

#[tokio::test]
async fn test_ocr_with_explicit_ocrs_engine() {
    let server = TestServer::start();
    let client = reqwest::Client::new();

    let result = test_ocr_file_with_engine(
        &client,
        &server.base_url(),
        "sample_text.png",
        "image/png",
        "ocrs",
    )
    .await;

    assert!(result.text.contains("Hello"));
    assert_eq!(result.engine, Some("ocrs".to_string()));
}
