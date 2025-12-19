use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod ocr;
mod server;

#[derive(Parser, Debug)]
#[command(name = "activestorage-ocr-server")]
#[command(about = "High-performance OCR server for ActiveStorage-OCR")]
#[command(version)]
pub struct Args {
    /// Host address to bind to
    #[arg(long, env = "OCR_HOST", default_value = "127.0.0.1")]
    pub host: String,

    /// Port to listen on
    #[arg(long, env = "OCR_PORT", default_value = "9292")]
    pub port: u16,

    /// Default language for OCR (e.g., "eng", "deu", "fra")
    #[arg(long, env = "OCR_DEFAULT_LANGUAGE", default_value = "eng")]
    pub default_language: String,

    /// Maximum file size in bytes (default: 50MB)
    #[arg(long, env = "OCR_MAX_FILE_SIZE", default_value = "52428800")]
    pub max_file_size: usize,

    /// Path to tessdata directory (uses TESSDATA_PREFIX env var if not set)
    #[arg(long, env = "TESSDATA_PREFIX")]
    pub tessdata_path: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, env = "RUST_LOG", default_value = "info")]
    pub log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| args.log_level.clone().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::from(args);

    tracing::info!(
        "Starting activestorage-ocr-server v{}",
        env!("CARGO_PKG_VERSION")
    );
    tracing::info!("Binding to {}:{}", config.host, config.port);

    server::run(config).await
}
