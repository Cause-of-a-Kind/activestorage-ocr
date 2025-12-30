use crate::error::OcrError;
use image::{DynamicImage, GrayImage, Luma};

/// Normalize image contrast using histogram stretching
/// Maps pixel values to use full 0-255 range
pub fn apply(image: DynamicImage) -> Result<DynamicImage, OcrError> {
    let gray = image.to_luma8();
    let (min_val, max_val) = find_min_max(&gray);

    // Avoid division by zero
    if max_val <= min_val {
        return Ok(DynamicImage::ImageLuma8(gray));
    }

    let range = (max_val - min_val) as f32;
    let normalized = GrayImage::from_fn(gray.width(), gray.height(), |x, y| {
        let pixel = gray.get_pixel(x, y).0[0];
        let normalized = ((pixel - min_val) as f32 / range * 255.0) as u8;
        Luma([normalized])
    });

    Ok(DynamicImage::ImageLuma8(normalized))
}

fn find_min_max(img: &GrayImage) -> (u8, u8) {
    let mut min = 255u8;
    let mut max = 0u8;

    for pixel in img.pixels() {
        let val = pixel.0[0];
        min = min.min(val);
        max = max.max(val);
    }

    (min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_stretches_histogram() {
        // Create a low-contrast image (values 50-200)
        let img = GrayImage::from_fn(10, 10, |x, _| {
            let val = 50 + (x as u8 * 15).min(150);
            Luma([val])
        });

        let result = apply(DynamicImage::ImageLuma8(img)).unwrap();
        let result_gray = result.to_luma8();

        let (min, max) = find_min_max(&result_gray);

        // After normalization, min should be 0 and max should be 255
        assert_eq!(min, 0);
        assert_eq!(max, 255);
    }

    #[test]
    fn test_normalize_handles_uniform_image() {
        // Uniform image (all same value)
        let img = GrayImage::from_pixel(10, 10, Luma([128]));

        let result = apply(DynamicImage::ImageLuma8(img.clone())).unwrap();
        let result_gray = result.to_luma8();

        // Should return unchanged (no division by zero)
        assert_eq!(result_gray.get_pixel(0, 0).0[0], 128);
    }
}
