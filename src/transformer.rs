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
        Self {
            from: "EPSG:4326".to_string(),
            to: "EPSG:3857".to_string(),
            transformer: Arc::new(Mutex::new(None)),
        }
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
        // Validate CRS strings
        if !is_valid_crs(&from) || !is_valid_crs(&to) {
            return Err(TransformerError::InvalidCrs(format!(
                "Invalid CRS: from={}, to={}",
                from, to
            )));
        }

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
            let new_transformer = Proj::new_known_crs(&self.from, &self.to, None)
                .map_err(|e| TransformerError::ProjError(e))?;
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
        if !is_valid_crs(&from) || !is_valid_crs(&to) {
            return Err(TransformerError::InvalidCrs(format!(
                "Invalid CRS: from={}, to={}",
                from, to
            )));
        }

        self.from = from;
        self.to = to;
        self.clear_cache()
    }

    pub fn is_transformer_available(&self) -> bool {
        self.transformer
            .lock()
            .map(|t| t.is_some())
            .unwrap_or(false)
    }
}

fn is_valid_crs(crs: &str) -> bool {
    // Add CRS validation logic here
    // Could check against a list of known CRS or use proj's validation
    !crs.is_empty()
}
