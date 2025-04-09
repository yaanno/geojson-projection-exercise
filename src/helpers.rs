use crate::{ProcessedGeometry, ProjectionError};
use geo::{LineString, Point, Polygon};
use geojson::{Feature, Geometry};
use proj::Proj;

#[derive(Clone, Debug)]
pub struct TransformerConfig {
    from: String,
    to: String,
    inverse: bool,
}

impl TransformerConfig {
    pub fn new(from: String, to: String, inverse: bool) -> Self {
        Self { from, to, inverse }
    }
    pub fn get_transformer(&self) -> Result<Proj, ProjectionError> {
        let transformer = Proj::new_known_crs(&self.from, &self.to, None)?;
        Ok(transformer)
    }
    pub fn default() -> Self {
        Self {
            from: "EPSG:4326".to_string(),
            to: "EPSG:25832".to_string(),
            inverse: false,
        }
    }
}

pub struct GeometryProcessor {
    geometry: Geometry,
    config: TransformerConfig,
}

impl GeometryProcessor {
    pub fn new(geometry: Geometry, config: TransformerConfig) -> Self {
        Self { geometry, config }
    }
    pub fn convert(&self) -> Result<ProcessedGeometry, ProjectionError> {
        match &self.geometry.value {
            geojson::Value::Point(point) => convert_point(point.to_vec(), self.config.clone()),
            geojson::Value::LineString(line_string) => {
                convert_line_string(line_string.to_vec(), self.config.clone())
            }
            geojson::Value::Polygon(polygon) => {
                convert_polygon(polygon.to_vec(), self.config.clone())
            }
            geojson::Value::MultiPoint(_items) => todo!(),
            geojson::Value::MultiLineString(_items) => todo!(),
            geojson::Value::MultiPolygon(_items) => todo!(),
            geojson::Value::GeometryCollection(_items) => todo!(),
        }
    }
    pub fn invert(&self) -> Result<ProcessedGeometry, ProjectionError> {
        match &self.geometry.value {
            geojson::Value::Point(point) => convert_point(point.to_vec(), self.config.clone()),
            geojson::Value::LineString(line_string) => {
                convert_line_string(line_string.to_vec(), self.config.clone())
            }
            geojson::Value::Polygon(polygon) => {
                convert_polygon(polygon.to_vec(), self.config.clone())
            }
            geojson::Value::MultiPoint(_items) => todo!(),
            geojson::Value::MultiLineString(_items) => todo!(),
            geojson::Value::MultiPolygon(_items) => todo!(),
            geojson::Value::GeometryCollection(_items) => todo!(),
        }
    }
}

// ------------------------
// Helper functions
// ------------------------

/// Get a transformer
///
/// # Returns
///
/// * `Proj` - A transformer
pub fn get_transformer(config: TransformerConfig) -> Result<Proj, ProjectionError> {
    let transformer = config.get_transformer()?;
    Ok(transformer)
}

pub fn convert_point(
    p: Vec<f64>,
    config: TransformerConfig,
) -> Result<ProcessedGeometry, ProjectionError> {
    if p.len() != 2 {
        return Err(ProjectionError::InvalidCoordinates(
            "Point must have exactly 2 coordinates".to_string(),
        ));
    }
    let transformer = config.get_transformer()?;
    let point = Point::new(p[0], p[1]);
    let projected = transformer.convert(point)?;
    Ok(ProcessedGeometry::Point(projected.into()))
}

pub fn convert_line_string(
    ls: Vec<Vec<f64>>,
    config: TransformerConfig,
) -> Result<ProcessedGeometry, ProjectionError> {
    let transformer = config.get_transformer()?;
    let mut projected_coords = Vec::<Point<f64>>::new();
    for coord_pair in ls {
        if coord_pair.len() != 2 {
            return Err(ProjectionError::InvalidCoordinates(
                "LineString coordinates must be pairs of numbers".to_string(),
            ));
        }
        let point = Point::new(coord_pair[0], coord_pair[1]);
        let projected_point = transformer.convert(point)?;
        projected_coords.push(projected_point.into());
    }
    let line_string = LineString::from(projected_coords);
    Ok(ProcessedGeometry::LineString(line_string))
}

pub fn convert_linear_ring(
    ring: Vec<Vec<f64>>,
    config: TransformerConfig,
) -> Result<LineString<f64>, ProjectionError> {
    let transformer = config.get_transformer()?;
    let mut projected_coords = Vec::<Point<f64>>::new();
    for coord_pair in ring {
        if coord_pair.len() != 2 {
            return Err(ProjectionError::InvalidCoordinates(
                "Linear ring coordinates must be pairs of numbers".to_string(),
            ));
        }
        let point = Point::new(coord_pair[0], coord_pair[1]);
        let projected_point = transformer.convert(point)?;
        projected_coords.push(projected_point.into());
    }
    Ok(LineString::from(projected_coords))
}

/// Convert a polygon
///
/// # Arguments
///
/// * `p` - A vector of vectors of vectors of f64, representing the coordinates of the polygon
///
/// # Returns
///
/// * `ProcessedGeometry::Polygon` - A polygon with the coordinates projected
pub fn convert_polygon(
    polygon: Vec<Vec<Vec<f64>>>,
    config: TransformerConfig,
) -> Result<ProcessedGeometry, ProjectionError> {
    if polygon.is_empty() {
        return Err(ProjectionError::InvalidCoordinates(
            "Polygon must have at least one linear ring".to_string(),
        ));
    }

    // The first linear ring is the exterior ring
    let exterior_ring = convert_linear_ring(polygon[0].clone(), config.clone())?;

    // Any subsequent linear rings are interior rings (holes)
    let mut interior_rings = Vec::new();
    for inner_ring in polygon.iter().skip(1) {
        let transformed_ring = convert_linear_ring(inner_ring.clone(), config.clone())?;
        interior_rings.push(transformed_ring);
    }

    let poly = Polygon::new(exterior_ring, interior_rings);
    Ok(ProcessedGeometry::Polygon(poly))
}

/// Process a feature
///
/// # Arguments
///
/// * `feature` - A feature with a geometry
///
/// # Returns
///
/// * `ProcessedGeometry` - A processed geometry
pub fn process_feature_geometry(
    feature: Feature,
    config: TransformerConfig,
) -> Result<ProcessedGeometry, ProjectionError> {
    match feature.geometry {
        Some(geometry) => process_geometry(geometry, config),
        None => Err(ProjectionError::InvalidGeometryType),
    }
}

/// Process a geometry
///
/// # Arguments
///
/// * `geometry` - A geometry
///
/// # Returns
///
/// * `ProcessedGeometry` - A processed geometry
pub fn process_geometry(
    geometry: Geometry,
    config: TransformerConfig,
) -> Result<ProcessedGeometry, ProjectionError> {
    let processor = GeometryProcessor::new(geometry, config.clone());
    if config.inverse {
        processor.invert()
    } else {
        processor.convert()
    }
}
