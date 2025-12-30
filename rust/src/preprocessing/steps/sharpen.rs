use crate::error::OcrError;
use image::DynamicImage;
use imageproc::filter::filter3x3;

/// Apply Laplacian-based sharpening
/// Enhances edges to make text more distinct
pub fn apply(image: DynamicImage) -> Result<DynamicImage, OcrError> {
    let gray = image.to_luma8();

    // Laplacian-based sharpening kernel
    // Center weight 5, neighbors -1 each = edge enhancement
    let kernel: [f32; 9] = [0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0];

    let sharpened = filter3x3(&gray, &kernel);
    Ok(DynamicImage::ImageLuma8(sharpened))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GrayImage, Luma};

    #[test]
    fn test_sharpen_enhances_edges() {
        // Create image with an edge (left half dark, right half light)
        let img = GrayImage::from_fn(20, 10, |x, _| {
            if x < 10 {
                Luma([50])
            } else {
                Luma([200])
            }
        });

        let result = apply(DynamicImage::ImageLuma8(img.clone())).unwrap();
        let result_gray = result.to_luma8();

        // Edge pixels should have enhanced contrast
        let edge_left = result_gray.get_pixel(9, 5).0[0];
        let edge_right = result_gray.get_pixel(10, 5).0[0];

        // The difference at the edge should be at least as large as original
        let original_diff = 200i32 - 50;
        let result_diff = (edge_right as i32 - edge_left as i32).abs();

        assert!(
            result_diff >= original_diff,
            "Edge should be enhanced: {} >= {}",
            result_diff,
            original_diff
        );
    }
}
