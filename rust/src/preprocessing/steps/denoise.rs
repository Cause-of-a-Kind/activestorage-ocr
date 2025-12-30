use crate::error::OcrError;
use image::DynamicImage;
use imageproc::filter::median_filter;

/// Apply median filter to reduce noise
/// Median filter preserves edges better than Gaussian blur
pub fn apply(image: DynamicImage) -> Result<DynamicImage, OcrError> {
    let gray = image.to_luma8();
    // 3x3 median filter (radius 1) - effective for salt-and-pepper noise
    let denoised = median_filter(&gray, 1, 1);
    Ok(DynamicImage::ImageLuma8(denoised))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GrayImage, Luma};

    #[test]
    fn test_denoise_reduces_salt_pepper_noise() {
        // Create image with salt-and-pepper noise pattern
        let mut img = GrayImage::from_pixel(10, 10, Luma([128]));
        img.put_pixel(5, 5, Luma([0])); // "pepper" noise
        img.put_pixel(6, 5, Luma([255])); // "salt" noise

        let result = apply(DynamicImage::ImageLuma8(img.clone())).unwrap();
        let result_gray = result.to_luma8();

        // Median filter should smooth out isolated noise pixels
        // The result should be closer to the surrounding values
        let original_variance = calculate_variance(&img);
        let result_variance = calculate_variance(&result_gray);

        // Variance should be reduced after denoising
        assert!(result_variance <= original_variance);
    }

    fn calculate_variance(img: &GrayImage) -> f64 {
        let pixels: Vec<f64> = img.pixels().map(|p| p.0[0] as f64).collect();
        let mean = pixels.iter().sum::<f64>() / pixels.len() as f64;
        pixels.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / pixels.len() as f64
    }
}
