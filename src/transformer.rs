use proj::Proj;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransformerError {
    #[error("Mutex was poisoned: {0}")]
    MutexPoisoned(String),
    #[error("Invalid CRS: {0}")]
    InvalidCrs(String),
    #[error("Projection error: {0}")]
    ProjError(#[from] proj::ProjCreateError),
}

#[derive(Debug, Clone)]
pub struct TransformerConfig {
    from: String,
    to: String,
    transformer: Arc<Mutex<Option<Arc<Proj>>>>,
}

impl Default for TransformerConfig {
    fn default() -> Self {
        Self::new("EPSG:4326".to_string(), "EPSG:3857".to_string())
            .expect("Default CRS should be valid")
    }
}

impl TransformerConfig {
    /// Create a new TransformerConfig
    ///
    /// # Arguments
    ///
    /// * `from` - A string, representing the source coordinate reference system
    /// * `to` - A string, representing the target coordinate reference system
    ///
    /// # Example
    ///
    /// ```rust
    /// use proj_exercise_simple::transformer::TransformerConfig;
    /// let config = TransformerConfig::new("EPSG:4326".to_string(), "EPSG:3857".to_string()).unwrap();
    /// ```
    pub fn new(from: String, to: String) -> Result<Self, TransformerError> {
        validate_crs(&from)?;
        validate_crs(&to)?;

        Ok(Self {
            from,
            to,
            transformer: Arc::new(Mutex::new(None)),
        })
    }

    /// Get a transformer
    ///
    /// # Returns
    ///
    /// * `Arc<Proj>` - A transformer
    ///
    /// # Example
    ///
    /// ```rust
    /// use proj_exercise_simple::transformer::TransformerConfig;
    /// let config = TransformerConfig::new("EPSG:4326".to_string(), "EPSG:3857".to_string()).unwrap();
    /// let transformer = config.get_transformer();
    /// ```
    pub fn get_transformer(&self) -> Result<Arc<Proj>, TransformerError> {
        let mut transformer = self
            .transformer
            .lock()
            .map_err(|e| TransformerError::MutexPoisoned(e.to_string()))?;

        if transformer.is_none() {
            let new_transformer = Proj::new_known_crs(&self.from, &self.to, None)?;
            *transformer = Some(Arc::new(new_transformer));
        }

        Ok(transformer.as_ref().unwrap().clone())
    }

    // Clear the cached transformer (useful if config changes)
    pub fn clear_cache(&self) -> Result<(), TransformerError> {
        let mut transformer = self
            .transformer
            .lock()
            .map_err(|e| TransformerError::MutexPoisoned(e.to_string()))?;
        *transformer = None;
        Ok(())
    }

    /// Update the CRS
    ///
    /// # Arguments
    ///
    /// * `from` - A string, representing the source coordinate reference system
    /// * `to` - A string, representing the target coordinate reference system
    ///
    /// # Example
    ///
    /// ```rust
    /// use proj_exercise_simple::transformer::TransformerConfig;
    /// let mut config = TransformerConfig::new("EPSG:4326".to_string(), "EPSG:3857".to_string()).unwrap();
    /// config.update_crs("EPSG:4326".to_string(), "EPSG:3857".to_string());
    /// ```
    pub fn update_crs(&mut self, from: String, to: String) -> Result<(), TransformerError> {
        validate_crs(&from)?;
        validate_crs(&to)?;

        self.from = from;
        self.to = to;
        self.clear_cache()
    }

    pub fn is_transformer_available(&self) -> Result<bool, TransformerError> {
        Ok(self
            .transformer
            .lock()
            .map_err(|e| TransformerError::MutexPoisoned(e.to_string()))?
            .is_some())
    }
}

fn validate_crs(crs: &str) -> Result<(), TransformerError> {
    if crs.is_empty() {
        return Err(TransformerError::InvalidCrs(
            "CRS string cannot be empty".to_string(),
        ));
    }

    // Try to create a temporary transformer to validate the CRS
    let _ = Proj::new_known_crs(crs, crs, None)
        .map_err(|e| TransformerError::InvalidCrs(format!("Invalid CRS {}: {}", crs, e)))?;

    Ok(())
}
