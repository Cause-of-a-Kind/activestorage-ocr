use crate::error::OcrError;
use image::DynamicImage;

/// Convert image to grayscale
/// This is the foundation for most other preprocessing steps
pub fn apply(image: DynamicImage) -> Result<DynamicImage, OcrError> {
    Ok(DynamicImage::ImageLuma8(image.to_luma8()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    #[test]
    fn test_grayscale_converts_color() {
        let mut img = RgbImage::new(10, 10);
        img.put_pixel(0, 0, Rgb([255, 0, 0])); // Red
        img.put_pixel(1, 0, Rgb([0, 255, 0])); // Green
        img.put_pixel(2, 0, Rgb([0, 0, 255])); // Blue

        let result = apply(DynamicImage::ImageRgb8(img)).unwrap();
        let gray = result.to_luma8();

        // All pixels should have some value (within tolerance)
        assert!(gray.get_pixel(0, 0).0[0] > 0);
        assert!(gray.get_pixel(1, 0).0[0] > 0);
        assert!(gray.get_pixel(2, 0).0[0] > 0);
    }

    #[test]
    fn test_grayscale_preserves_dimensions() {
        let img = RgbImage::new(100, 50);
        let result = apply(DynamicImage::ImageRgb8(img)).unwrap();
        assert_eq!(result.width(), 100);
        assert_eq!(result.height(), 50);
    }
}
