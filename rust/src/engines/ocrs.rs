//! OCRS engine implementation
//!
//! Pure Rust OCR engine using the ocrs library. No system dependencies required.
//! Downloads neural network models automatically on first use.

use crate::config::Config;
use crate::engine::{OcrEngine, OcrResult};
use crate::error::OcrError;
use image::DynamicImage;
use ocrs::{DecodeMethod, ImageSource, OcrEngine as OcrsOcrEngine, OcrEngineParams};
use rten::Model;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;

/// Default model URLs from the ocrs project
const DETECTION_MODEL_URL: &str =
    "https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten";
const RECOGNITION_MODEL_URL: &str =
    "https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten";

/// OCR Engine wrapping the ocrs library
pub struct OcrsEngine {
    engine: Arc<OcrsOcrEngine>,
}

impl OcrsEngine {
    /// Create a new OCR processor, downloading models if needed
    pub fn new(_config: &Config) -> Result<Self, OcrError> {
        tracing::info!("Initializing ocrs OCR engine...");

        // Load models (will download if not cached)
        let detection_model_path =
            ensure_model_downloaded(DETECTION_MODEL_URL, "text-detection.rten")?;
        let recognition_model_path =
            ensure_model_downloaded(RECOGNITION_MODEL_URL, "text-recognition.rten")?;

        // Load models using rten::Model::load_file
        let detection_model = Model::load_file(&detection_model_path).map_err(|e| {
            OcrError::InitializationError(format!("Failed to load detection model: {}", e))
        })?;
        let recognition_model = Model::load_file(&recognition_model_path).map_err(|e| {
            OcrError::InitializationError(format!("Failed to load recognition model: {}", e))
        })?;

        let engine = OcrsOcrEngine::new(OcrEngineParams {
            detection_model: Some(detection_model),
            recognition_model: Some(recognition_model),
            decode_method: DecodeMethod::Greedy,
            ..Default::default()
        })
        .map_err(|e| {
            OcrError::InitializationError(format!("Failed to create OCR engine: {}", e))
        })?;

        tracing::info!("ocrs engine initialized successfully");

        Ok(Self {
            engine: Arc::new(engine),
        })
    }

    /// Process an image file and return the extracted text
    fn process_image_file(&self, path: &Path) -> Result<OcrResult, OcrError> {
        let warnings = Vec::new();

        // Load the image using the image crate
        let img = image::open(path)
            .map_err(|e| OcrError::ProcessingError(format!("Failed to load image: {}", e)))?;

        // Convert to RGB8 (HWC format, which is what ImageSource::from_bytes expects)
        let rgb_img = img.into_rgb8();
        let dimensions = rgb_img.dimensions();

        // Create image source from raw bytes (HWC format)
        let img_source = ImageSource::from_bytes(rgb_img.as_raw(), dimensions).map_err(|e| {
            OcrError::ProcessingError(format!("Failed to create image source: {}", e))
        })?;

        // Prepare input for OCR
        let ocr_input = self
            .engine
            .prepare_input(img_source)
            .map_err(|e| OcrError::ProcessingError(format!("Failed to prepare input: {}", e)))?;

        // Detect words
        let word_rects = self
            .engine
            .detect_words(&ocr_input)
            .map_err(|e| OcrError::ProcessingError(format!("Failed to detect words: {}", e)))?;

        // Group words into lines
        let line_rects = self.engine.find_text_lines(&ocr_input, &word_rects);

        // Recognize text in each line
        let line_texts = self
            .engine
            .recognize_text(&ocr_input, &line_rects)
            .map_err(|e| OcrError::ProcessingError(format!("Failed to recognize text: {}", e)))?;

        // Combine all lines into a single string
        let text: String = line_texts
            .iter()
            .filter_map(|line| line.as_ref())
            .map(|line| {
                line.words()
                    .map(|word| word.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Calculate confidence using text quality heuristics
        let confidence = calculate_confidence(&text);

        Ok(OcrResult {
            text,
            confidence,
            warnings,
        })
    }

    /// Process a PDF file
    fn process_pdf(&self, path: &Path) -> Result<OcrResult, OcrError> {
        let mut warnings = Vec::new();

        // First, try to extract text directly from the PDF
        let direct_text = pdf_extract::extract_text(path)
            .map_err(|e| OcrError::ProcessingError(format!("Failed to parse PDF: {}", e)))?;

        // If we got meaningful text, return it
        let trimmed_text = direct_text.trim();
        if !trimmed_text.is_empty() && trimmed_text.len() > 10 {
            tracing::info!(
                "Extracted {} chars of text directly from PDF",
                trimmed_text.len()
            );
            return Ok(OcrResult {
                text: trimmed_text.to_string(),
                confidence: 0.95, // High confidence for direct text extraction
                warnings,
            });
        }

        // If direct extraction yielded little/no text, try to extract and OCR images
        tracing::info!("PDF has no embedded text, attempting to extract images for OCR");
        warnings
            .push("PDF appears to be scanned/image-based, extracting images for OCR".to_string());

        let images = extract_images_from_pdf(path)?;

        if images.is_empty() {
            return Ok(OcrResult {
                text: String::new(),
                confidence: 0.0,
                warnings: vec!["No text or images found in PDF".to_string()],
            });
        }

        // OCR each image and combine results
        let mut all_text = Vec::new();
        for (i, img) in images.iter().enumerate() {
            tracing::info!("Processing image {} of {} from PDF", i + 1, images.len());
            match self.process_dynamic_image(img) {
                Ok(result) => {
                    if !result.text.is_empty() {
                        all_text.push(result.text);
                    }
                }
                Err(e) => {
                    warnings.push(format!("Failed to OCR image {}: {}", i + 1, e));
                }
            }
        }

        let combined_text = all_text.join("\n\n");
        let confidence = calculate_confidence(&combined_text);

        Ok(OcrResult {
            text: combined_text,
            confidence,
            warnings,
        })
    }

    /// Process a DynamicImage directly (used for extracted PDF images)
    fn process_dynamic_image(&self, img: &DynamicImage) -> Result<OcrResult, OcrError> {
        let rgb_img = img.to_rgb8();
        let dimensions = rgb_img.dimensions();

        let img_source = ImageSource::from_bytes(rgb_img.as_raw(), dimensions).map_err(|e| {
            OcrError::ProcessingError(format!("Failed to create image source: {}", e))
        })?;

        let ocr_input = self
            .engine
            .prepare_input(img_source)
            .map_err(|e| OcrError::ProcessingError(format!("Failed to prepare input: {}", e)))?;

        let word_rects = self
            .engine
            .detect_words(&ocr_input)
            .map_err(|e| OcrError::ProcessingError(format!("Failed to detect words: {}", e)))?;

        let line_rects = self.engine.find_text_lines(&ocr_input, &word_rects);

        let line_texts = self
            .engine
            .recognize_text(&ocr_input, &line_rects)
            .map_err(|e| OcrError::ProcessingError(format!("Failed to recognize text: {}", e)))?;

        let text: String = line_texts
            .iter()
            .filter_map(|line| line.as_ref())
            .map(|line| {
                line.words()
                    .map(|word| word.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n");

        let confidence = calculate_confidence(&text);

        Ok(OcrResult {
            text,
            confidence,
            warnings: Vec::new(),
        })
    }
}

impl OcrEngine for OcrsEngine {
    fn name(&self) -> &'static str {
        "ocrs"
    }

    fn description(&self) -> &'static str {
        "Pure Rust OCR engine - fast, no system dependencies required"
    }

    fn process(&self, path: &Path) -> Result<OcrResult, OcrError> {
        // Check if the file is a PDF
        if is_pdf(path)? {
            return self.process_pdf(path);
        }

        self.process_image_file(path)
    }

    fn process_image(&self, image: &DynamicImage) -> Result<OcrResult, OcrError> {
        self.process_dynamic_image(image)
    }

    fn supported_formats(&self) -> Vec<String> {
        vec![
            "image/png".to_string(),
            "image/jpeg".to_string(),
            "image/gif".to_string(),
            "image/bmp".to_string(),
            "image/webp".to_string(),
            "image/tiff".to_string(),
            "application/pdf".to_string(),
        ]
    }

    fn supported_languages(&self) -> Vec<String> {
        // ocrs currently only supports English/Latin alphabet
        vec!["eng".to_string()]
    }
}

// ============================================================================
// Confidence scoring heuristics
// ============================================================================

/// Calculate confidence score based on text quality heuristics.
///
/// Since ocrs doesn't provide per-character confidence scores, we analyze
/// the recognized text for patterns that indicate OCR quality.
fn calculate_confidence(text: &str) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    if text.len() < 5 {
        return 0.5; // Too short to judge accurately
    }

    let char_score = analyze_char_frequency(text);
    let word_score = analyze_word_lengths(text);
    let whitespace_score = analyze_whitespace(text);
    let repetition_score = detect_repetition(text);

    let confidence =
        0.40 * char_score + 0.30 * word_score + 0.15 * whitespace_score + 0.15 * repetition_score;

    confidence.clamp(0.0, 1.0)
}

/// Analyze character frequency for signs of garbled OCR.
///
/// Penalizes text with too many special/control characters or too few letters.
fn analyze_char_frequency(text: &str) -> f32 {
    let total = text.chars().count();
    if total == 0 {
        return 0.0;
    }

    let letters = text.chars().filter(|c| c.is_alphabetic()).count();
    let special = text
        .chars()
        .filter(|c| !c.is_alphanumeric() && !c.is_whitespace() && !c.is_ascii_punctuation())
        .count();

    // Penalize high special char ratio
    let special_ratio = special as f32 / total as f32;
    let special_penalty = 1.0 - (special_ratio * 10.0).min(1.0);

    // Penalize very low letter content (unless it's a numeric document)
    let letter_ratio = letters as f32 / total as f32;
    let letter_score = (letter_ratio * 1.5).min(1.0);

    special_penalty * 0.6 + letter_score * 0.4
}

/// Analyze word length distribution.
///
/// Garbled OCR often produces single-character "words" or very long sequences.
fn analyze_word_lengths(text: &str) -> f32 {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return 0.5;
    }

    let total_len: usize = words.iter().map(|w| w.len()).sum();
    let avg_len = total_len as f32 / words.len() as f32;

    // Ideal average word length: 4-8 chars
    let avg_score = match avg_len as usize {
        0..=1 => 0.3,
        2..=3 => 0.7,
        4..=8 => 1.0,
        9..=12 => 0.8,
        _ => 0.4,
    };

    // Penalize too many single-char "words"
    let single_count = words.iter().filter(|w| w.len() == 1).count();
    let single_ratio = single_count as f32 / words.len() as f32;
    let single_penalty = 1.0 - (single_ratio * 1.5).min(0.5);

    avg_score * single_penalty
}

/// Analyze whitespace ratio.
///
/// Normal text has ~10-25% whitespace. Too dense or too sparse indicates issues.
fn analyze_whitespace(text: &str) -> f32 {
    let total = text.chars().count();
    if total == 0 {
        return 0.0;
    }

    let whitespace = text.chars().filter(|c| c.is_whitespace()).count();
    let ratio = (whitespace as f32 / total as f32) * 100.0;

    match ratio as usize {
        0..=5 => 0.5,   // Too dense
        6..=10 => 0.8,  // Slightly dense
        11..=25 => 1.0, // Ideal
        26..=40 => 0.7, // Slightly sparse
        _ => 0.3,       // Too sparse
    }
}

/// Detect repeated character sequences.
///
/// Patterns like "aaaa" or "####" often indicate OCR confusion.
fn detect_repetition(text: &str) -> f32 {
    let mut max_repeat = 1;
    let mut current = 1;
    let mut prev: Option<char> = None;

    for c in text.chars() {
        if Some(c) == prev && !c.is_whitespace() {
            current += 1;
            max_repeat = max_repeat.max(current);
        } else {
            current = 1;
        }
        prev = Some(c);
    }

    match max_repeat {
        1..=3 => 1.0,
        4..=5 => 0.8,
        6..=10 => 0.5,
        _ => 0.2,
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Check if a file is a PDF by reading its magic bytes
fn is_pdf(path: &Path) -> Result<bool, OcrError> {
    // Check file extension first
    if let Some(ext) = path.extension() {
        if ext.to_string_lossy().to_lowercase() == "pdf" {
            return Ok(true);
        }
    }

    // Also check magic bytes (%PDF-)
    let mut file = File::open(path)
        .map_err(|e| OcrError::ProcessingError(format!("Failed to open file: {}", e)))?;

    let mut magic = [0u8; 5];
    if file.read_exact(&mut magic).is_ok() {
        return Ok(&magic == b"%PDF-");
    }

    Ok(false)
}

/// Extract images from a PDF using lopdf
fn extract_images_from_pdf(path: &Path) -> Result<Vec<DynamicImage>, OcrError> {
    use lopdf::Document;

    let doc = Document::load(path)
        .map_err(|e| OcrError::ProcessingError(format!("Failed to load PDF: {}", e)))?;

    let mut images = Vec::new();

    // Iterate through all objects looking for image XObjects
    for (object_id, object) in doc.objects.iter() {
        if let Ok(stream) = object.as_stream() {
            // Check if this is an image XObject
            if let Ok(subtype) = stream.dict.get(b"Subtype") {
                if let Ok(name) = subtype.as_name() {
                    if name == b"Image" {
                        // Try to extract the image data
                        match extract_image_from_stream(&doc, stream) {
                            Ok(img) => images.push(img),
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to extract image from object {:?}: {}",
                                    object_id,
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(images)
}

/// Extract an image from a PDF stream
fn extract_image_from_stream(
    doc: &lopdf::Document,
    stream: &lopdf::Stream,
) -> Result<DynamicImage, OcrError> {
    // Get image dimensions
    let width = stream
        .dict
        .get(b"Width")
        .ok()
        .and_then(|w| w.as_i64().ok())
        .ok_or_else(|| OcrError::ProcessingError("Missing image width".to_string()))?
        as u32;

    let height = stream
        .dict
        .get(b"Height")
        .ok()
        .and_then(|h| h.as_i64().ok())
        .ok_or_else(|| OcrError::ProcessingError("Missing image height".to_string()))?
        as u32;

    // Get the image data (decompressed)
    let data = stream
        .decompressed_content()
        .map_err(|e| OcrError::ProcessingError(format!("Failed to decompress image: {}", e)))?;

    // Get color space - handle both direct names and indirect references
    let color_space = get_color_space(doc, stream);

    // Get bits per component
    let bits_per_component = stream
        .dict
        .get(b"BitsPerComponent")
        .ok()
        .and_then(|b| b.as_i64().ok())
        .unwrap_or(8) as u8;

    tracing::debug!(
        "PDF image: {}x{}, {} bits, color_space={}, data_len={}",
        width,
        height,
        bits_per_component,
        color_space,
        data.len()
    );

    // Handle different color spaces
    match color_space.as_str() {
        "DeviceGray" => {
            if bits_per_component == 8 && data.len() >= (width * height) as usize {
                let img = image::GrayImage::from_raw(width, height, data).ok_or_else(|| {
                    OcrError::ProcessingError("Invalid grayscale image data".to_string())
                })?;
                Ok(DynamicImage::ImageLuma8(img))
            } else {
                Err(OcrError::ProcessingError(format!(
                    "Unsupported grayscale format: {} bits, data_len={}, expected={}",
                    bits_per_component,
                    data.len(),
                    width * height
                )))
            }
        }
        "DeviceRGB" | "ICCBased" => {
            // ICCBased with 3 components is typically RGB
            if bits_per_component == 8 && data.len() >= (width * height * 3) as usize {
                let img = image::RgbImage::from_raw(width, height, data).ok_or_else(|| {
                    OcrError::ProcessingError("Invalid RGB image data".to_string())
                })?;
                Ok(DynamicImage::ImageRgb8(img))
            } else {
                Err(OcrError::ProcessingError(format!(
                    "Unsupported RGB format: {} bits, data_len={}, expected={}",
                    bits_per_component,
                    data.len(),
                    width * height * 3
                )))
            }
        }
        "DeviceCMYK" => {
            // Convert CMYK to RGB
            if bits_per_component == 8 && data.len() >= (width * height * 4) as usize {
                let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);
                for chunk in data.chunks(4) {
                    if chunk.len() == 4 {
                        let c = chunk[0] as f32 / 255.0;
                        let m = chunk[1] as f32 / 255.0;
                        let y = chunk[2] as f32 / 255.0;
                        let k = chunk[3] as f32 / 255.0;
                        let r = ((1.0 - c) * (1.0 - k) * 255.0) as u8;
                        let g = ((1.0 - m) * (1.0 - k) * 255.0) as u8;
                        let b = ((1.0 - y) * (1.0 - k) * 255.0) as u8;
                        rgb_data.push(r);
                        rgb_data.push(g);
                        rgb_data.push(b);
                    }
                }
                let img = image::RgbImage::from_raw(width, height, rgb_data).ok_or_else(|| {
                    OcrError::ProcessingError("Invalid CMYK->RGB conversion".to_string())
                })?;
                Ok(DynamicImage::ImageRgb8(img))
            } else {
                Err(OcrError::ProcessingError(format!(
                    "Unsupported CMYK format: {} bits, data_len={}, expected={}",
                    bits_per_component,
                    data.len(),
                    width * height * 4
                )))
            }
        }
        _ => Err(OcrError::ProcessingError(format!(
            "Unsupported color space: {}",
            color_space
        ))),
    }
}

/// Get the color space name from a PDF stream, resolving indirect references
fn get_color_space(doc: &lopdf::Document, stream: &lopdf::Stream) -> String {
    let cs_obj = match stream.dict.get(b"ColorSpace") {
        Ok(obj) => obj,
        Err(_) => return "DeviceRGB".to_string(),
    };

    // Handle direct name
    if let Ok(name) = cs_obj.as_name() {
        return String::from_utf8_lossy(name).to_string();
    }

    // Handle indirect reference
    if let Ok(reference) = cs_obj.as_reference() {
        if let Ok(resolved) = doc.get_object(reference) {
            // Could be a name
            if let Ok(name) = resolved.as_name() {
                return String::from_utf8_lossy(name).to_string();
            }
            // Could be an array like [/ICCBased ref]
            if let Ok(array) = resolved.as_array() {
                if let Some(first) = array.first() {
                    if let Ok(name) = first.as_name() {
                        return String::from_utf8_lossy(name).to_string();
                    }
                }
            }
        }
    }

    // Handle array directly (like [/ICCBased ref])
    if let Ok(array) = cs_obj.as_array() {
        if let Some(first) = array.first() {
            if let Ok(name) = first.as_name() {
                return String::from_utf8_lossy(name).to_string();
            }
        }
    }

    "DeviceRGB".to_string()
}

/// Ensure model is downloaded and return its path
fn ensure_model_downloaded(url: &str, filename: &str) -> Result<std::path::PathBuf, OcrError> {
    // Get cache directory
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("activestorage-ocr");

    std::fs::create_dir_all(&cache_dir).map_err(|e| {
        OcrError::InitializationError(format!("Failed to create cache directory: {}", e))
    })?;

    let model_path = cache_dir.join(filename);

    // Download if not cached
    if !model_path.exists() {
        tracing::info!("Downloading {} (this may take a moment)...", filename);
        download_file(url, &model_path)?;
        tracing::info!("Downloaded {} to {:?}", filename, model_path);
    } else {
        tracing::info!("Using cached model from {:?}", model_path);
    }

    Ok(model_path)
}

/// Download a file from URL to path using ureq
fn download_file(url: &str, path: &Path) -> Result<(), OcrError> {
    let response = ureq::get(url)
        .call()
        .map_err(|e| OcrError::InitializationError(format!("Failed to download model: {}", e)))?;

    let mut file = File::create(path).map_err(|e| {
        OcrError::InitializationError(format!("Failed to create model file: {}", e))
    })?;

    // Read response body and write to file
    let buffer = response.into_body().read_to_vec().map_err(|e| {
        OcrError::InitializationError(format!("Failed to read response body: {}", e))
    })?;

    file.write_all(&buffer)
        .map_err(|e| OcrError::InitializationError(format!("Failed to write model file: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_text_returns_zero() {
        assert_eq!(calculate_confidence(""), 0.0);
    }

    #[test]
    fn test_short_text_returns_half() {
        assert_eq!(calculate_confidence("Hi"), 0.5);
        assert_eq!(calculate_confidence("Test"), 0.5);
    }

    #[test]
    fn test_clean_text_high_confidence() {
        let text = "Hello World OCR Test 12345";
        let confidence = calculate_confidence(text);
        assert!(confidence > 0.7, "Expected > 0.7, got {}", confidence);
    }

    #[test]
    fn test_garbled_text_low_confidence() {
        // Lots of special characters indicates bad OCR
        let text = "§±®©¥€£¢¤";
        let confidence = calculate_confidence(text);
        assert!(confidence < 0.5, "Expected < 0.5, got {}", confidence);
    }

    #[test]
    fn test_repeated_chars_lower_confidence() {
        let text = "Hello aaaaaaaaaaaa World";
        let confidence = calculate_confidence(text);
        // Should be lower than clean text due to repetition
        assert!(confidence < 0.9, "Expected < 0.9, got {}", confidence);
    }

    #[test]
    fn test_single_char_words_lower_confidence() {
        // Many single-char "words" suggests garbled OCR
        let text = "a b c d e f g h i j k l m n o p";
        let confidence = calculate_confidence(text);
        assert!(confidence < 0.7, "Expected < 0.7, got {}", confidence);
    }

    #[test]
    fn test_normal_sentence_good_confidence() {
        let text = "The quick brown fox jumps over the lazy dog.";
        let confidence = calculate_confidence(text);
        assert!(confidence > 0.75, "Expected > 0.75, got {}", confidence);
    }

    #[test]
    fn test_analyze_char_frequency_normal() {
        let score = analyze_char_frequency("Hello World");
        assert!(score > 0.8, "Expected > 0.8, got {}", score);
    }

    #[test]
    fn test_analyze_char_frequency_special() {
        let score = analyze_char_frequency("§±®©¥€£¢¤ƒ");
        assert!(score < 0.5, "Expected < 0.5, got {}", score);
    }

    #[test]
    fn test_analyze_word_lengths_normal() {
        let score = analyze_word_lengths("Hello World Test");
        assert!(score > 0.8, "Expected > 0.8, got {}", score);
    }

    #[test]
    fn test_analyze_whitespace_normal() {
        let text = "Hello World Test String";
        let score = analyze_whitespace(text);
        assert!(score > 0.7, "Expected > 0.7, got {}", score);
    }

    #[test]
    fn test_detect_repetition_none() {
        assert_eq!(detect_repetition("Hello World"), 1.0);
    }

    #[test]
    fn test_detect_repetition_some() {
        let score = detect_repetition("Hellooooo World");
        assert!(score < 1.0, "Expected < 1.0, got {}", score);
    }
}
