use proj::Proj;
use std::sync::{Arc, Mutex};

use crate::error::ProjectionError;

#[derive(Debug, Clone)]
pub struct TransformerConfig {
    from: String,
    to: String,
    transformer: Arc<Mutex<Option<Proj>>>,
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
    /// use proj_exercise_simple::helpers::TransformerConfig;
    /// let config = TransformerConfig::new("EPSG:4326".to_string(), "EPSG:3857".to_string());
    /// ```
    pub fn new(from: String, to: String) -> Self {
        Self {
            from,
            to,
            transformer: Arc::new(Mutex::new(None)),
        }
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
    /// use proj_exercise_simple::helpers::TransformerConfig;
    /// let config = TransformerConfig::new("EPSG:4326".to_string(), "EPSG:3857".to_string());
    /// let transformer = config.get_transformer();
    /// ```
    pub fn get_transformer(&self) -> Result<Arc<Proj>, ProjectionError> {
        let mut transformer = self.transformer.lock().unwrap();
        if transformer.is_none() {
            let new_transformer = Proj::new_known_crs(&self.from, &self.to, None)?;
            *transformer = Some(new_transformer);
        }
        Ok(Arc::new(transformer.take().unwrap()))
    }

    // Clear the cached transformer (useful if config changes)
    pub fn clear_cache(&self) {
        let mut transformer = self.transformer.lock().unwrap();
        *transformer = None;
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
    /// use proj_exercise_simple::helpers::TransformerConfig;
    /// let mut config = TransformerConfig::new("EPSG:4326".to_string(), "EPSG:3857".to_string());
    /// config.update_crs("EPSG:4326".to_string(), "EPSG:3857".to_string());
    /// ```
    pub fn update_crs(&mut self, from: String, to: String) {
        self.from = from;
        self.to = to;
        self.clear_cache();
    }
}
