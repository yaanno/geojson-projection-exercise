use crate::{ProcessedGeometry, ProjectionError};
use geo::{LineString, Point, Polygon};
use geojson::{Feature, FeatureCollection};
use proj::Proj;

// ------------------------
// Helper functions
// ------------------------

/// Get a transformer
///
/// # Returns
///
/// * `Proj` - A transformer
pub fn get_transformer() -> Result<Proj, ProjectionError> {
    let from = "EPSG:4326";
    let to = "EPSG:25832";
    let transformer = Proj::new_known_crs(&from, &to, None)?;
    Ok(transformer)
}

/// Convert a point to projected
///
/// # Arguments
///
/// * `p` - A vector of f64, representing the coordinates of the point
///
/// # Returns
///
/// * `ProcessedGeometry::Point` - A point with the coordinates projected
pub fn convert_point_to_projected(p: Vec<f64>) -> Result<ProcessedGeometry, ProjectionError> {
    if p.len() != 2 {
        return Err(ProjectionError::InvalidCoordinates(
            "Point must have exactly 2 coordinates".to_string(),
        ));
    }
    let transformer = get_transformer()?;
    let point = Point::new(p[0], p[1]);
    let projected = transformer.convert(point)?;
    Ok(ProcessedGeometry::Point(projected.into()))
}

/// Convert a line string to projected
///
/// # Arguments
///
/// * `ls` - A vector of vectors of f64, representing the coordinates of the line string
///
/// # Returns
///
/// * `ProcessedGeometry::LineString` - A line string with the coordinates projected
///
fn convert_line_string_to_projected(
    ls: Vec<Vec<f64>>,
) -> Result<ProcessedGeometry, ProjectionError> {
    let transformer = get_transformer()?;
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

/// Convert a linear ring to projected
///
/// # Arguments
///
/// * `ring` - A vector of vectors of f64, representing the coordinates of the linear ring
///
/// # Returns
///
/// * `ProcessedGeometry::LineString` - A line string with the coordinates projected
pub fn convert_linear_ring_to_projected(
    ring: Vec<Vec<f64>>,
) -> Result<LineString<f64>, ProjectionError> {
    let transformer = get_transformer()?;
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

/// Convert a polygon to projected
///
/// # Arguments
///
/// * `p` - A vector of vectors of vectors of f64, representing the coordinates of the polygon
///
/// # Returns
///
/// * `ProcessedGeometry::Polygon` - A polygon with the coordinates projected
pub fn convert_polygon_to_projected(
    polygon: Vec<Vec<Vec<f64>>>,
) -> Result<ProcessedGeometry, ProjectionError> {
    if polygon.is_empty() {
        return Err(ProjectionError::InvalidCoordinates(
            "Polygon must have at least one linear ring".to_string(),
        ));
    }

    // The first linear ring is the exterior ring
    let exterior_ring = convert_linear_ring_to_projected(polygon[0].clone())?;

    // Any subsequent linear rings are interior rings (holes)
    let mut interior_rings = Vec::new();
    for inner_ring in polygon.iter().skip(1) {
        let transformed_ring = convert_linear_ring_to_projected(inner_ring.clone())?;
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
pub fn process_geometry(feature: Feature) -> Result<ProcessedGeometry, ProjectionError> {
    match feature.geometry {
        Some(geometry) => match geometry.value {
            geojson::Value::Point(point) => convert_point_to_projected(point),
            geojson::Value::LineString(line_string) => {
                convert_line_string_to_projected(line_string)
            }
            geojson::Value::Polygon(polygon) => convert_polygon_to_projected(polygon),
            _ => Err(ProjectionError::InvalidGeometryType),
        },
        None => Err(ProjectionError::InvalidGeometryType),
    }
}

/// Process a feature collection
///
/// # Arguments
///
/// * `fc` - A feature collection
///
/// # Returns
///
/// * `Vec<ProcessedGeometry>` - A vector of processed features
pub fn process_feature_collection(
    fc: FeatureCollection,
) -> Result<Vec<ProcessedGeometry>, ProjectionError> {
    let mut processed_features = Vec::new();
    for feature in fc.features {
        let processed_feature = process_geometry(feature)?;
        processed_features.push(processed_feature);
    }
    Ok(processed_features)
}
