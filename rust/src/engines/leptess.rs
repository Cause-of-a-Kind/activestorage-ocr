//! Leptess/Tesseract engine implementation
//!
//! Tesseract-based OCR engine. Better for noisy/messy images like phone photos.
//! Uses tesseract-static crate for static linking (no system dependencies).
//! Downloads tessdata (training data) automatically on first use.

use crate::config::Config;
use crate::engine::{OcrEngine, OcrResult};
use crate::error::OcrError;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use tesseract_static::tesseract::Tesseract;

/// Tesseract OCR Engine
pub struct LeptessEngine {
    /// Path to tessdata directory
    tessdata_path: String,
    /// Default language for OCR
    default_language: String,
}

impl LeptessEngine {
    /// Create a new Tesseract-based OCR engine
    pub fn new(config: &Config) -> Result<Self, OcrError> {
        let default_language = config.default_language.clone();

        // Ensure tessdata is available (download if needed)
        let tessdata_path = ensure_tessdata_available(&default_language)?;

        // Validate that tessdata is accessible by doing a test initialization
        let test_tess =
            Tesseract::new(Some(&tessdata_path), Some(&default_language)).map_err(|e| {
                OcrError::InitializationError(format!(
                    "Failed to initialize Tesseract: {}",
                    e
                ))
            })?;

        // Drop the test instance
        drop(test_tess);

        tracing::info!(
            "Leptess engine initialized (tessdata: {}, language: {})",
            tessdata_path,
            default_language
        );

        Ok(Self {
            tessdata_path,
            default_language,
        })
    }

    /// Process an image file
    fn process_image(&self, path: &Path) -> Result<OcrResult, OcrError> {
        // Load image using the image crate
        let img = image::open(path)
            .map_err(|e| OcrError::ProcessingError(format!("Failed to load image: {}", e)))?;

        self.process_dynamic_image(&img)
    }

    /// Process a DynamicImage directly (used by both process_image and process_pdf)
    fn process_dynamic_image(&self, img: &image::DynamicImage) -> Result<OcrResult, OcrError> {
        // Convert to RGB8 for consistent handling
        let rgb_img = img.to_rgb8();
        let (width, height) = rgb_img.dimensions();

        // Convert to BMP in memory (BMP is always supported by leptonica)
        let mut bmp_data = Vec::new();
        {
            let mut cursor = std::io::Cursor::new(&mut bmp_data);
            rgb_img
                .write_to(&mut cursor, image::ImageFormat::Bmp)
                .map_err(|e| {
                    OcrError::ProcessingError(format!("Failed to convert to BMP: {}", e))
                })?;
        }

        tracing::debug!(
            "Processing image: {}x{}, BMP size: {} bytes",
            width,
            height,
            bmp_data.len()
        );

        let mut tess = Tesseract::new(Some(&self.tessdata_path), Some(&self.default_language))
            .map_err(|e| OcrError::ProcessingError(format!("Failed to create Tesseract: {}", e)))?;

        // Use set_image_from_mem with BMP data
        tess = tess.set_image_from_mem(&bmp_data).map_err(|e| {
            OcrError::ProcessingError(format!(
                "Failed to set image ({}x{}, {} bytes): {}",
                width,
                height,
                bmp_data.len(),
                e
            ))
        })?;

        tess = tess
            .recognize()
            .map_err(|e| OcrError::ProcessingError(format!("Failed to recognize text: {}", e)))?;

        let text = tess
            .get_text()
            .map_err(|e| OcrError::ProcessingError(format!("Failed to get text: {}", e)))?;

        // Get confidence score (0-100 scale, convert to 0.0-1.0)
        let confidence = tess.mean_text_conf() as f32 / 100.0;

        Ok(OcrResult {
            text: text.trim().to_string(),
            confidence,
            warnings: Vec::new(),
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
        let mut total_confidence = 0.0;
        let mut confidence_count = 0;

        for (i, img) in images.iter().enumerate() {
            tracing::info!("Processing image {} of {} from PDF", i + 1, images.len());

            // Process the image directly without saving to temp file
            match self.process_dynamic_image(img) {
                Ok(result) => {
                    if !result.text.is_empty() {
                        all_text.push(result.text);
                        total_confidence += result.confidence;
                        confidence_count += 1;
                    }
                }
                Err(e) => {
                    warnings.push(format!("Failed to OCR image {}: {}", i + 1, e));
                }
            }
        }

        let combined_text = all_text.join("\n\n");
        let avg_confidence = if confidence_count > 0 {
            total_confidence / confidence_count as f32
        } else {
            0.0
        };

        Ok(OcrResult {
            text: combined_text,
            confidence: avg_confidence,
            warnings,
        })
    }
}

impl OcrEngine for LeptessEngine {
    fn name(&self) -> &'static str {
        "leptess"
    }

    fn description(&self) -> &'static str {
        "Tesseract OCR engine - better for noisy/messy images like phone photos"
    }

    fn process(&self, path: &Path) -> Result<OcrResult, OcrError> {
        // Check if the file is a PDF
        if is_pdf(path)? {
            return self.process_pdf(path);
        }

        self.process_image(path)
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
        // Tesseract supports many languages - return common ones
        // Users can install additional language packs
        vec![
            "eng".to_string(),     // English
            "deu".to_string(),     // German
            "fra".to_string(),     // French
            "spa".to_string(),     // Spanish
            "ita".to_string(),     // Italian
            "por".to_string(),     // Portuguese
            "nld".to_string(),     // Dutch
            "jpn".to_string(),     // Japanese
            "chi_sim".to_string(), // Chinese Simplified
            "chi_tra".to_string(), // Chinese Traditional
            "kor".to_string(),     // Korean
            "ara".to_string(),     // Arabic
            "rus".to_string(),     // Russian
        ]
    }
}

// ============================================================================
// Helper functions (shared with ocrs engine, could be moved to common module)
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
fn extract_images_from_pdf(path: &Path) -> Result<Vec<image::DynamicImage>, OcrError> {
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
) -> Result<image::DynamicImage, OcrError> {
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

    // Get color space
    let color_space = get_color_space(doc, stream);

    // Get bits per component
    let bits_per_component = stream
        .dict
        .get(b"BitsPerComponent")
        .ok()
        .and_then(|b| b.as_i64().ok())
        .unwrap_or(8) as u8;

    // Handle different color spaces
    match color_space.as_str() {
        "DeviceGray" => {
            if bits_per_component == 8 && data.len() >= (width * height) as usize {
                let img = image::GrayImage::from_raw(width, height, data).ok_or_else(|| {
                    OcrError::ProcessingError("Invalid grayscale image data".to_string())
                })?;
                Ok(image::DynamicImage::ImageLuma8(img))
            } else {
                Err(OcrError::ProcessingError(format!(
                    "Unsupported grayscale format: {} bits",
                    bits_per_component
                )))
            }
        }
        "DeviceRGB" | "ICCBased" => {
            if bits_per_component == 8 && data.len() >= (width * height * 3) as usize {
                let img = image::RgbImage::from_raw(width, height, data).ok_or_else(|| {
                    OcrError::ProcessingError("Invalid RGB image data".to_string())
                })?;
                Ok(image::DynamicImage::ImageRgb8(img))
            } else {
                Err(OcrError::ProcessingError(format!(
                    "Unsupported RGB format: {} bits",
                    bits_per_component
                )))
            }
        }
        "DeviceCMYK" => {
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
                Ok(image::DynamicImage::ImageRgb8(img))
            } else {
                Err(OcrError::ProcessingError(format!(
                    "Unsupported CMYK format: {} bits",
                    bits_per_component
                )))
            }
        }
        _ => Err(OcrError::ProcessingError(format!(
            "Unsupported color space: {}",
            color_space
        ))),
    }
}

/// Get the color space name from a PDF stream
fn get_color_space(doc: &lopdf::Document, stream: &lopdf::Stream) -> String {
    let cs_obj = match stream.dict.get(b"ColorSpace") {
        Ok(obj) => obj,
        Err(_) => return "DeviceRGB".to_string(),
    };

    if let Ok(name) = cs_obj.as_name() {
        return String::from_utf8_lossy(name).to_string();
    }

    if let Ok(reference) = cs_obj.as_reference() {
        if let Ok(resolved) = doc.get_object(reference) {
            if let Ok(name) = resolved.as_name() {
                return String::from_utf8_lossy(name).to_string();
            }
            if let Ok(array) = resolved.as_array() {
                if let Some(first) = array.first() {
                    if let Ok(name) = first.as_name() {
                        return String::from_utf8_lossy(name).to_string();
                    }
                }
            }
        }
    }

    if let Ok(array) = cs_obj.as_array() {
        if let Some(first) = array.first() {
            if let Ok(name) = first.as_name() {
                return String::from_utf8_lossy(name).to_string();
            }
        }
    }

    "DeviceRGB".to_string()
}

// ============================================================================
// Tessdata download helpers
// ============================================================================

/// Ensure tessdata is available, downloading if needed
fn ensure_tessdata_available(language: &str) -> Result<String, OcrError> {
    // Get cache directory for tessdata
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("activestorage-ocr")
        .join("tessdata");

    std::fs::create_dir_all(&cache_dir).map_err(|e| {
        OcrError::InitializationError(format!("Failed to create tessdata directory: {}", e))
    })?;

    let traineddata_file = format!("{}.traineddata", language);
    let traineddata_path = cache_dir.join(&traineddata_file);

    // Download if not cached
    if !traineddata_path.exists() {
        let url = tessdata_url(language);
        tracing::info!(
            "Downloading tessdata for '{}' (this may take a moment)...",
            language
        );
        download_file(&url, &traineddata_path)?;
        tracing::info!("Downloaded tessdata to {:?}", traineddata_path);
    } else {
        tracing::info!("Using cached tessdata from {:?}", cache_dir);
    }

    // Return the directory path (Tesseract expects the directory, not the file)
    cache_dir
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| OcrError::InitializationError("Invalid tessdata path".to_string()))
}

/// Get tessdata download URL for a language
fn tessdata_url(language: &str) -> String {
    // Use tessdata_fast for smaller, faster downloads
    format!(
        "https://github.com/tesseract-ocr/tessdata_fast/raw/main/{}.traineddata",
        language
    )
}

/// Download a file from URL to path using ureq
fn download_file(url: &str, path: &Path) -> Result<(), OcrError> {
    let response = ureq::get(url)
        .call()
        .map_err(|e| OcrError::InitializationError(format!("Failed to download tessdata: {}", e)))?;

    let mut file = File::create(path).map_err(|e| {
        OcrError::InitializationError(format!("Failed to create tessdata file: {}", e))
    })?;

    // Read response body and write to file
    let buffer = response.into_body().read_to_vec().map_err(|e| {
        OcrError::InitializationError(format!("Failed to read tessdata response: {}", e))
    })?;

    file.write_all(&buffer).map_err(|e| {
        OcrError::InitializationError(format!("Failed to write tessdata file: {}", e))
    })?;

    Ok(())
}
