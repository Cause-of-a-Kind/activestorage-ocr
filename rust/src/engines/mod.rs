//! OCR engine implementations
//!
//! This module contains implementations of the OcrEngine trait for different
//! OCR backends. Engines are conditionally compiled based on feature flags.

#[cfg(feature = "engine-ocrs")]
pub mod ocrs;

#[cfg(feature = "engine-leptess")]
pub mod leptess;

use crate::config::Config;
use crate::engine::OcrEngine;
use crate::error::OcrError;
use std::sync::Arc;

/// Information about an available engine
#[derive(Debug, Clone)]
pub struct EngineInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub supported_formats: Vec<String>,
    pub supported_languages: Vec<String>,
}

/// Registry of available OCR engines
pub struct EngineRegistry {
    engines: Vec<Arc<dyn OcrEngine>>,
    default_engine: String,
}

impl EngineRegistry {
    /// Create a new engine registry with all available engines initialized
    pub fn new(config: &Config) -> Result<Self, OcrError> {
        let mut engines: Vec<Arc<dyn OcrEngine>> = Vec::new();
        let mut default_engine = String::new();

        #[cfg(feature = "engine-ocrs")]
        {
            tracing::info!("Initializing ocrs engine...");
            let ocrs_engine = ocrs::OcrsEngine::new(config)?;
            if default_engine.is_empty() {
                default_engine = ocrs_engine.name().to_string();
            }
            engines.push(Arc::new(ocrs_engine));
        }

        #[cfg(feature = "engine-leptess")]
        {
            tracing::info!("Initializing leptess engine...");
            let leptess_engine = leptess::LeptessEngine::new(config)?;
            if default_engine.is_empty() {
                default_engine = leptess_engine.name().to_string();
            }
            engines.push(Arc::new(leptess_engine));
        }

        if engines.is_empty() {
            return Err(OcrError::InitializationError(
                "No OCR engines available. Build with --features engine-ocrs or --features engine-leptess".to_string()
            ));
        }

        Ok(Self {
            engines,
            default_engine,
        })
    }

    /// Get an engine by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn OcrEngine>> {
        self.engines.iter().find(|e| e.name() == name).cloned()
    }

    /// Get the default engine
    pub fn default(&self) -> Option<Arc<dyn OcrEngine>> {
        self.get(&self.default_engine)
    }

    /// Get the default engine name
    pub fn default_name(&self) -> &str {
        &self.default_engine
    }

    /// List all available engine names
    pub fn list(&self) -> Vec<&str> {
        self.engines.iter().map(|e| e.name()).collect()
    }

    /// Get info about all available engines
    pub fn info(&self) -> Vec<EngineInfo> {
        self.engines
            .iter()
            .map(|e| EngineInfo {
                name: e.name(),
                description: e.description(),
                supported_formats: e.supported_formats(),
                supported_languages: e.supported_languages(),
            })
            .collect()
    }
}
