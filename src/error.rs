use crate::pool::BufferPoolError;
use crate::transformer::TransformerError;
use geojson::Error as GeoJsonError;
use proj::ProjCreateError;
use proj::ProjError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectionError {
    #[error("Invalid geometry type")]
    InvalidGeometryType,
    #[error("Invalid coordinates: {0}")]
    InvalidCoordinates(String),
    #[error("Projection error: {0}")]
    ProjError(#[from] ProjError),
    #[error("Projection creation error: {0}")]
    ProjCreateError(#[from] ProjCreateError),
    #[error("GeoJSON error: {0}")]
    GeoJsonError(#[from] GeoJsonError),
    #[error("Transformer error: {0}")]
    TransformerError(#[from] TransformerError),
    #[error("Buffer pool error: {0}")]
    BufferPoolError(#[from] BufferPoolError),
}
