use crate::error::OcrError;
use image::DynamicImage;
use serde::Serialize;
use std::time::Instant;

use super::steps;

/// Preprocessing preset names
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Preset {
    /// Skip all preprocessing (0ms overhead)
    None,
    /// Minimal processing for clean scans (~30-50ms)
    /// Steps: grayscale only
    Minimal,
    /// Default balanced processing (~100-150ms)
    /// Steps: grayscale, resize, normalize, sharpen
    #[default]
    Default,
    /// Aggressive processing for poor quality images (~200-300ms)
    /// Steps: grayscale, resize, denoise, normalize, sharpen, deskew, threshold
    Aggressive,
}

impl Preset {
    /// Parse from query parameter string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" => Some(Self::None),
            "minimal" => Some(Self::Minimal),
            "default" => Some(Self::Default),
            "aggressive" => Some(Self::Aggressive),
            _ => None,
        }
    }

    /// Get the preset name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Minimal => "minimal",
            Self::Default => "default",
            Self::Aggressive => "aggressive",
        }
    }
}

/// Timing information for a single preprocessing step
#[derive(Debug, Clone, Serialize)]
pub struct StepTiming {
    pub name: String,
    pub time_ms: u64,
}

/// Result of preprocessing including timing stats
#[derive(Debug, Clone, Serialize)]
pub struct PreprocessingResult {
    /// Preprocessed image (not serialized)
    #[serde(skip)]
    pub image: DynamicImage,
    /// Total preprocessing time in milliseconds
    pub total_time_ms: u64,
    /// Preset used
    pub preset: String,
    /// Individual step timings
    pub steps: Vec<StepTiming>,
}

/// Preprocessing pipeline that applies steps based on preset
pub struct Pipeline {
    preset: Preset,
}

impl Pipeline {
    pub fn new(preset: Preset) -> Self {
        Self { preset }
    }

    /// Process an image according to the configured preset
    pub fn process(&self, image: DynamicImage) -> Result<PreprocessingResult, OcrError> {
        let start = Instant::now();
        let mut steps_timing = Vec::new();

        if self.preset == Preset::None {
            return Ok(PreprocessingResult {
                image,
                total_time_ms: 0,
                preset: "none".to_string(),
                steps: vec![],
            });
        }

        let mut img = image;

        // All presets except None do grayscale
        img = self.run_step("grayscale", img, &mut steps_timing, steps::grayscale::apply)?;

        if self.preset == Preset::Minimal {
            return Ok(PreprocessingResult {
                image: img,
                total_time_ms: start.elapsed().as_millis() as u64,
                preset: "minimal".to_string(),
                steps: steps_timing,
            });
        }

        // Default and Aggressive: resize for optimal OCR
        img = self.run_step("resize", img, &mut steps_timing, steps::resize::apply)?;

        // Aggressive only: denoise before normalize
        if self.preset == Preset::Aggressive {
            img = self.run_step("denoise", img, &mut steps_timing, steps::denoise::apply)?;
        }

        // Default and Aggressive: normalize contrast
        img = self.run_step("normalize", img, &mut steps_timing, steps::normalize::apply)?;

        // Default and Aggressive: sharpen
        img = self.run_step("sharpen", img, &mut steps_timing, steps::sharpen::apply)?;

        // Aggressive only: deskew and threshold
        if self.preset == Preset::Aggressive {
            img = self.run_step("deskew", img, &mut steps_timing, steps::deskew::apply)?;
            img = self.run_step("threshold", img, &mut steps_timing, steps::threshold::apply)?;
        }

        Ok(PreprocessingResult {
            image: img,
            total_time_ms: start.elapsed().as_millis() as u64,
            preset: self.preset.as_str().to_string(),
            steps: steps_timing,
        })
    }

    fn run_step<F>(
        &self,
        name: &str,
        img: DynamicImage,
        timings: &mut Vec<StepTiming>,
        step_fn: F,
    ) -> Result<DynamicImage, OcrError>
    where
        F: FnOnce(DynamicImage) -> Result<DynamicImage, OcrError>,
    {
        let step_start = Instant::now();
        let result = step_fn(img)?;
        timings.push(StepTiming {
            name: name.to_string(),
            time_ms: step_start.elapsed().as_millis() as u64,
        });
        Ok(result)
    }
}
