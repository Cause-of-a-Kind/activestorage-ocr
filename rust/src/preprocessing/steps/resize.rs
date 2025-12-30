use crate::error::OcrError;
use image::{imageops::FilterType, DynamicImage, GenericImageView};

/// Target DPI for OCR (300 DPI is generally optimal)
const TARGET_DPI: u32 = 300;
/// Assume input images are 72 DPI if no metadata available
const ASSUMED_INPUT_DPI: u32 = 72;
/// Maximum dimension to avoid memory issues
const MAX_DIMENSION: u32 = 4000;
/// Minimum dimension for reasonable OCR
const MIN_DIMENSION: u32 = 300;

/// Resize image to optimal size for OCR
/// Scales up low-res images and constrains very large ones
pub fn apply(image: DynamicImage) -> Result<DynamicImage, OcrError> {
    let (width, height) = image.dimensions();

    // Calculate scale factor (assume 72 DPI source, target 300 DPI)
    let scale = TARGET_DPI as f32 / ASSUMED_INPUT_DPI as f32;

    let mut new_width = (width as f32 * scale) as u32;
    let mut new_height = (height as f32 * scale) as u32;

    // Clamp to max dimension
    if new_width > MAX_DIMENSION || new_height > MAX_DIMENSION {
        let max_dim = new_width.max(new_height);
        let scale_down = MAX_DIMENSION as f32 / max_dim as f32;
        new_width = (new_width as f32 * scale_down) as u32;
        new_height = (new_height as f32 * scale_down) as u32;
    }

    // Ensure minimum dimension
    if new_width < MIN_DIMENSION && new_height < MIN_DIMENSION {
        let min_dim = new_width.min(new_height);
        let scale_up = MIN_DIMENSION as f32 / min_dim as f32;
        new_width = (new_width as f32 * scale_up) as u32;
        new_height = (new_height as f32 * scale_up) as u32;
    }

    // Skip resize if dimensions are similar (within 5%)
    let width_ratio = new_width as f32 / width as f32;
    let height_ratio = new_height as f32 / height as f32;
    if (0.95..=1.05).contains(&width_ratio) && (0.95..=1.05).contains(&height_ratio) {
        return Ok(image);
    }

    Ok(image.resize(new_width, new_height, FilterType::Lanczos3))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::GrayImage;

    #[test]
    fn test_resize_upscales_small_image() {
        // 100x100 at 72 DPI should be scaled to ~416x416 at 300 DPI
        let img = GrayImage::new(100, 100);
        let result = apply(DynamicImage::ImageLuma8(img)).unwrap();
        assert!(result.width() > 100);
        assert!(result.height() > 100);
    }

    #[test]
    fn test_resize_limits_large_image() {
        // Very large image should be constrained to MAX_DIMENSION
        let img = GrayImage::new(2000, 2000);
        let result = apply(DynamicImage::ImageLuma8(img)).unwrap();
        assert!(result.width() <= MAX_DIMENSION);
        assert!(result.height() <= MAX_DIMENSION);
    }
}
