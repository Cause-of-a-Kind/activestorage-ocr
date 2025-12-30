use crate::error::OcrError;
use image::{DynamicImage, GrayImage, Luma};

/// Sauvola threshold parameters
const WINDOW_SIZE: u32 = 15;
const K: f32 = 0.2;
const R: f32 = 128.0; // Dynamic range / 2

/// Apply Sauvola adaptive thresholding
/// Better than Otsu for documents with uneven lighting
pub fn apply(image: DynamicImage) -> Result<DynamicImage, OcrError> {
    let gray = image.to_luma8();
    let binarized = sauvola_threshold(&gray, WINDOW_SIZE, K);
    Ok(DynamicImage::ImageLuma8(binarized))
}

/// Sauvola adaptive thresholding
///
/// For each pixel, threshold = mean * (1 + k * (std_dev / R - 1))
/// where R is max standard deviation (128 for 8-bit images)
fn sauvola_threshold(img: &GrayImage, window_size: u32, k: f32) -> GrayImage {
    let (width, height) = img.dimensions();
    let half_window = window_size as i32 / 2;

    // Precompute integral images for efficient window statistics
    let (integral, integral_sq) = compute_integral_images(img);

    GrayImage::from_fn(width, height, |x, y| {
        let x1 = (x as i32 - half_window).max(0) as u32;
        let y1 = (y as i32 - half_window).max(0) as u32;
        let x2 = (x as i32 + half_window).min(width as i32 - 1) as u32;
        let y2 = (y as i32 + half_window).min(height as i32 - 1) as u32;

        let (mean, std_dev) = window_stats(&integral, &integral_sq, x1, y1, x2, y2);

        let threshold = mean * (1.0 + k * (std_dev / R - 1.0));

        let pixel = img.get_pixel(x, y).0[0] as f32;
        if pixel > threshold {
            Luma([255u8])
        } else {
            Luma([0u8])
        }
    })
}

/// Compute integral image and integral of squared values
fn compute_integral_images(img: &GrayImage) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let (width, height) = img.dimensions();
    let mut integral = vec![vec![0.0f64; width as usize + 1]; height as usize + 1];
    let mut integral_sq = vec![vec![0.0f64; width as usize + 1]; height as usize + 1];

    for y in 0..height as usize {
        for x in 0..width as usize {
            let val = img.get_pixel(x as u32, y as u32).0[0] as f64;
            integral[y + 1][x + 1] =
                val + integral[y][x + 1] + integral[y + 1][x] - integral[y][x];
            integral_sq[y + 1][x + 1] =
                val * val + integral_sq[y][x + 1] + integral_sq[y + 1][x] - integral_sq[y][x];
        }
    }

    (integral, integral_sq)
}

/// Compute mean and standard deviation for a window using integral images
fn window_stats(
    integral: &[Vec<f64>],
    integral_sq: &[Vec<f64>],
    x1: u32,
    y1: u32,
    x2: u32,
    y2: u32,
) -> (f32, f32) {
    let (x1, y1, x2, y2) = (x1 as usize, y1 as usize, x2 as usize + 1, y2 as usize + 1);
    let area = ((x2 - x1) * (y2 - y1)) as f64;

    let sum = integral[y2][x2] - integral[y1][x2] - integral[y2][x1] + integral[y1][x1];
    let sum_sq =
        integral_sq[y2][x2] - integral_sq[y1][x2] - integral_sq[y2][x1] + integral_sq[y1][x1];

    let mean = sum / area;
    let variance = (sum_sq / area) - (mean * mean);
    let std_dev = variance.max(0.0).sqrt();

    (mean as f32, std_dev as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_binarizes_image() {
        // Create a simple gradient image
        let img = GrayImage::from_fn(50, 50, |x, _| Luma([(x as u8 * 5).min(255)]));

        let result = apply(DynamicImage::ImageLuma8(img)).unwrap();
        let result_gray = result.to_luma8();

        // Result should only contain 0 or 255
        for pixel in result_gray.pixels() {
            assert!(
                pixel.0[0] == 0 || pixel.0[0] == 255,
                "Expected binary pixel, got {}",
                pixel.0[0]
            );
        }
    }

    #[test]
    fn test_threshold_handles_text_pattern() {
        // Create dark text on light background
        let mut img = GrayImage::from_pixel(50, 20, Luma([240]));
        for x in 10..40 {
            img.put_pixel(x, 10, Luma([20])); // dark text
        }

        let result = apply(DynamicImage::ImageLuma8(img)).unwrap();
        let result_gray = result.to_luma8();

        // Text pixels should be black (0)
        assert_eq!(result_gray.get_pixel(25, 10).0[0], 0);
        // Background should be white (255)
        assert_eq!(result_gray.get_pixel(25, 5).0[0], 255);
    }
}
