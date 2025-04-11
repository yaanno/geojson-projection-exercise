pub mod coordinates;
pub mod helpers;

// Re-export commonly used types
pub use coordinates::Coordinate;
pub use helpers::{GeometryProcessor, ProcessedGeometry, ProjectionError, TransformerConfig};
