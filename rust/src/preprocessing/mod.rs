//! Image preprocessing module for OCR enhancement
//!
//! Provides configurable preprocessing pipelines to improve OCR accuracy.

pub mod pipeline;
pub mod steps;

pub use pipeline::{Pipeline, Preset, StepTiming};
