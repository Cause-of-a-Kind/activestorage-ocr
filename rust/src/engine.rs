use crate::error::OcrError;
use std::path::Path;

/// OCR processing result
#[derive(Debug, Clone)]
pub struct OcrResult {
    pub text: String,
    pub confidence: f32,
    pub warnings: Vec<String>,
}

/// Trait that all OCR engines must implement
pub trait OcrEngine: Send + Sync {
    /// Returns the engine identifier (e.g., "ocrs", "leptess")
    fn name(&self) -> &'static str;

    /// Returns a human-readable description of the engine
    fn description(&self) -> &'static str;

    /// Process a file (image or PDF) and return the extracted text
    fn process(&self, path: &Path) -> Result<OcrResult, OcrError>;

    /// Get supported MIME types
    fn supported_formats(&self) -> Vec<String>;

    /// Get supported languages
    fn supported_languages(&self) -> Vec<String>;
}
