use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectionError {
    #[error("Failed to parse GeoJSON: {0}")]
    GeoJsonParseError(#[from] geojson::Error),
    #[error("Invalid geometry type: expected Point")]
    InvalidGeometryType,
    #[error("Projection error: {0}")]
    ProjError(#[from] proj::ProjError),
    #[error("Projection creation error: {0}")]
    ProjCreateError(#[from] proj::ProjCreateError),
    #[error("Invalid coordinates: {0}")]
    InvalidCoordinates(String),
}
