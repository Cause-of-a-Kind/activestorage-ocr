use crate::error::OcrError;
use image::{DynamicImage, GrayImage, Luma};
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};

/// Deskew image by detecting and correcting rotation
/// Uses projection profile method to find optimal angle
pub fn apply(image: DynamicImage) -> Result<DynamicImage, OcrError> {
    let gray = image.to_luma8();

    // Find optimal rotation angle
    let angle = detect_skew_angle(&gray);

    // Skip if angle is negligible (less than 0.1 degrees)
    if angle.abs() < 0.1_f32.to_radians() {
        return Ok(DynamicImage::ImageLuma8(gray));
    }

    // Rotate to correct skew
    let background = Luma([255u8]); // White background
    let rotated = rotate_about_center(&gray, angle, Interpolation::Bilinear, background);

    Ok(DynamicImage::ImageLuma8(rotated))
}

/// Detect skew angle using projection profile variance
fn detect_skew_angle(img: &GrayImage) -> f32 {
    let mut best_angle = 0.0_f32;
    let mut best_variance = 0.0_f32;

    // Search -5 to +5 degrees in 0.5 degree increments
    let mut angle = -5.0_f32;
    while angle <= 5.0 {
        let variance = compute_projection_variance(img, angle.to_radians());
        if variance > best_variance {
            best_variance = variance;
            best_angle = angle;
        }
        angle += 0.5;
    }

    // Refine search around best angle
    let mut refined_angle = best_angle - 0.5;
    while refined_angle <= best_angle + 0.5 {
        let variance = compute_projection_variance(img, refined_angle.to_radians());
        if variance > best_variance {
            best_variance = variance;
            best_angle = refined_angle;
        }
        refined_angle += 0.1;
    }

    best_angle.to_radians()
}

/// Compute variance of horizontal projection profile
/// Higher variance indicates more aligned text
fn compute_projection_variance(img: &GrayImage, angle: f32) -> f32 {
    let (width, height) = img.dimensions();
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;

    // Project and count dark pixels per row
    let mut row_counts = vec![0u32; height as usize];

    for y in 0..height {
        for x in 0..width {
            // Rotate point around center
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let new_y = (dy * cos_a - dx * sin_a + cy) as i32;

            if new_y >= 0 && new_y < height as i32 {
                let pixel = img.get_pixel(x, y).0[0];
                if pixel < 128 {
                    // Dark pixel (text)
                    row_counts[new_y as usize] += 1;
                }
            }
        }
    }

    // Compute variance
    let mean: f32 = row_counts.iter().sum::<u32>() as f32 / row_counts.len() as f32;
    let variance: f32 = row_counts
        .iter()
        .map(|&c| (c as f32 - mean).powi(2))
        .sum::<f32>()
        / row_counts.len() as f32;

    variance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deskew_detects_zero_angle_for_straight_image() {
        // Create a simple horizontal line pattern (straight text)
        let mut img = GrayImage::from_pixel(100, 50, Luma([255]));
        for x in 10..90 {
            img.put_pixel(x, 25, Luma([0])); // horizontal line
        }

        let angle = detect_skew_angle(&img);

        // Should detect near-zero angle for horizontal text
        assert!(
            angle.abs() < 0.5_f32.to_radians(),
            "Expected near-zero angle, got {} radians",
            angle
        );
    }

    #[test]
    fn test_deskew_preserves_dimensions() {
        let img = GrayImage::new(100, 50);
        let result = apply(DynamicImage::ImageLuma8(img)).unwrap();
        assert_eq!(result.width(), 100);
        assert_eq!(result.height(), 50);
    }
}
