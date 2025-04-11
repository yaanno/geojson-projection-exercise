use crate::coordinates::{Coordinate, Line, Polygon};
use geo::{
    CoordsIter, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon, Point,
    Polygon as GeoPolygon,
};
use geojson::{Feature, Geometry};
use proj::Proj;
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

#[derive(Debug)]
pub enum ProcessedGeometry {
    Point(Point<f64>),
    LineString(LineString<f64>),
    Polygon(GeoPolygon<f64>),
    MultiPoint(MultiPoint<f64>),
    MultiLineString(MultiLineString<f64>),
    MultiPolygon(MultiPolygon<f64>),
    GeometryCollection(GeometryCollection<f64>),
}

impl ProcessedGeometry {
    /// Convert a processed geometry to a geojson geometry
    ///
    /// # Returns
    ///
    /// * `geojson::Geometry` - A geojson geometry
    fn to_geojson_geometry(self) -> geojson::Geometry {
        match self {
            ProcessedGeometry::Point(point) => {
                let coord = Coordinate::from(point);
                geojson::Geometry::new(geojson::Value::Point(vec![coord.x, coord.y]))
            }
            ProcessedGeometry::LineString(line_string) => {
                let coords: Vec<Coordinate> = line_string
                    .coords_iter()
                    .map(|coord| Coordinate::new(coord.x, coord.y))
                    .collect();
                let line = Line::new(coords);
                geojson::Geometry::new(line.to_geojson())
            }
            ProcessedGeometry::Polygon(polygon) => {
                let exterior = Line::new(
                    polygon
                        .exterior()
                        .coords_iter()
                        .map(|coord| Coordinate::new(coord.x, coord.y))
                        .collect(),
                );
                let interiors = polygon
                    .interiors()
                    .iter()
                    .map(|ring| {
                        Line::new(
                            ring.coords_iter()
                                .map(|coord| Coordinate::new(coord.x, coord.y))
                                .collect(),
                        )
                    })
                    .collect();
                let polygon = Polygon::new(exterior, interiors);
                geojson::Geometry::new(polygon.to_geojson())
            }
            ProcessedGeometry::MultiPoint(_multi_point) => todo!(),
            ProcessedGeometry::MultiLineString(_multi_line_string) => todo!(),
            ProcessedGeometry::MultiPolygon(_multi_polygon) => todo!(),
            ProcessedGeometry::GeometryCollection(_geometry_collection) => todo!(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TransformerConfig {
    from: String,
    to: String,
}

/// Default transformer config
///
/// # Returns
///
/// * `TransformerConfig` - A default transformer config
///
/// # Example
///
/// ```rust
/// let config = TransformerConfig::default();
/// ```
impl Default for TransformerConfig {
    fn default() -> Self {
        Self {
            from: "EPSG:4326".to_string(),
            to: "EPSG:25832".to_string(),
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
    pub fn new(from: String, to: String) -> Self {
        Self { from, to }
    }
    /// Get a transformer
    ///
    /// # Returns
    ///
    /// * `Proj` - A transformer
    pub fn get_transformer(&self) -> Result<Proj, ProjectionError> {
        let transformer = Proj::new_known_crs(&self.from, &self.to, None)?;
        Ok(transformer)
    }
}

pub struct GeometryProcessor {
    geometry: Geometry,
    config: TransformerConfig,
}

impl GeometryProcessor {
    /// Create a new GeometryProcessor
    ///
    /// # Arguments
    ///
    /// * `geometry` - A geometry
    /// * `config` - A transformer config
    pub fn new(geometry: Geometry, config: TransformerConfig) -> Self {
        Self { geometry, config }
    }
    pub fn convert(&self) -> Result<ProcessedGeometry, ProjectionError> {
        match &self.geometry.value {
            geojson::Value::Point(point) => {
                let coord = Coordinate::new(point[0], point[1]);
                convert_point(coord, &self.config)
            }
            geojson::Value::LineString(line_string) => {
                let coords = line_string
                    .iter()
                    .map(|p| Coordinate::new(p[0], p[1]))
                    .collect();
                convert_line_string(coords, &self.config)
            }
            geojson::Value::Polygon(polygon) => {
                let exterior = polygon[0]
                    .iter()
                    .map(|p| Coordinate::new(p[0], p[1]))
                    .collect();
                let interiors = polygon[1..]
                    .iter()
                    .map(|ring| {
                        Line::new(ring.iter().map(|p| Coordinate::new(p[0], p[1])).collect())
                    })
                    .collect();
                convert_polygon(Polygon::new(Line::new(exterior), interiors), &self.config)
            }
            geojson::Value::MultiPoint(points) => {
                let coords = points.iter().map(|p| Coordinate::new(p[0], p[1])).collect();
                convert_multi_point(coords, &self.config)
            }
            geojson::Value::MultiLineString(line_strings) => {
                let lines = line_strings
                    .iter()
                    .map(|ls| Line::new(ls.iter().map(|p| Coordinate::new(p[0], p[1])).collect()))
                    .collect();
                convert_multi_line_string(lines, &self.config)
            }
            geojson::Value::MultiPolygon(polygons) => {
                let polys = polygons
                    .iter()
                    .map(|poly| {
                        let exterior = Line::new(
                            poly[0]
                                .iter()
                                .map(|p| Coordinate::new(p[0], p[1]))
                                .collect(),
                        );
                        let interiors = poly[1..]
                            .iter()
                            .map(|ring| {
                                Line::new(
                                    ring.iter().map(|p| Coordinate::new(p[0], p[1])).collect(),
                                )
                            })
                            .collect();
                        Polygon::new(exterior, interiors)
                    })
                    .collect();
                convert_multi_polygon(polys, &self.config)
            }
            geojson::Value::GeometryCollection(_items) => todo!(),
        }
    }
}

pub fn convert_multi_line_string(
    lines: Vec<Line>,
    config: &TransformerConfig,
) -> Result<ProcessedGeometry, ProjectionError> {
    let mut projected_line_strings = Vec::with_capacity(lines.len());
    for line in lines {
        let line_string = convert_line_string(line.coordinates, config)?;
        match line_string {
            ProcessedGeometry::LineString(ls) => projected_line_strings.push(ls),
            _ => {
                return Err(ProjectionError::InvalidCoordinates(
                    "Expected LineString geometry".to_string(),
                ));
            }
        }
    }
    let multi_line_string = MultiLineString::new(projected_line_strings);
    Ok(ProcessedGeometry::MultiLineString(multi_line_string))
}

/// Convert a multi point
///
/// # Arguments
///
/// * `items` - A vector of vectors of f64, representing the coordinates of the multi point
/// * `config` - A transformer config
///
/// # Returns
///
/// * `ProcessedGeometry::MultiPoint` - A projected multi point
pub fn convert_multi_point(
    points: Vec<Coordinate>,
    config: &TransformerConfig,
) -> Result<ProcessedGeometry, ProjectionError> {
    let mut projected_points = Vec::with_capacity(points.len());
    for point in points {
        let point = convert_point(point, config)?;
        match point {
            ProcessedGeometry::Point(p) => projected_points.push(p),
            _ => {
                return Err(ProjectionError::InvalidCoordinates(
                    "Expected Point geometry".to_string(),
                ));
            }
        }
    }
    let multi_point = MultiPoint::from(projected_points);
    Ok(ProcessedGeometry::MultiPoint(multi_point))
}

/// Convert a point
///
/// # Arguments
///
/// * `p` - A vector of f64, representing the coordinates of the point
/// * `config` - A transformer config
///
/// # Returns
///
/// * `ProcessedGeometry::Point` - A projected point
pub fn convert_point(
    point: Coordinate,
    config: &TransformerConfig,
) -> Result<ProcessedGeometry, ProjectionError> {
    let transformer = config.get_transformer()?;
    let geo_point = Point::new(point.x, point.y);
    let projected = transformer.convert(geo_point)?;
    Ok(ProcessedGeometry::Point(projected.into()))
}

/// Convert a line string
///
/// # Arguments
///
/// * `ls` - A vector of vectors of f64, representing the coordinates of the line string
/// * `config` - A transformer config
///
/// # Returns
///
/// * `ProcessedGeometry::LineString` - A projected line string
pub fn convert_line_string(
    coordinates: Vec<Coordinate>,
    config: &TransformerConfig,
) -> Result<ProcessedGeometry, ProjectionError> {
    let transformer = config.get_transformer()?;
    let mut projected_coords: Vec<Point<f64>> = Vec::with_capacity(coordinates.len());

    // Convert each coordinate individually
    for coord in coordinates {
        let point = Point::new(coord.x, coord.y);
        let projected = transformer.convert(point)?;
        projected_coords.push(projected.into());
    }

    let line_string = LineString::from(projected_coords);
    Ok(ProcessedGeometry::LineString(line_string))
}

/// Convert a polygon
///
/// # Arguments
///
/// * `p` - A vector of vectors of vectors of f64, representing the coordinates of the polygon
/// * `config` - A transformer config
///
/// # Returns
///
/// * `ProcessedGeometry::Polygon` - A polygon with the coordinates projected
pub fn convert_polygon(
    polygon: Polygon,
    config: &TransformerConfig,
) -> Result<ProcessedGeometry, ProjectionError> {
    let transformer = config.get_transformer()?;

    // Convert exterior ring
    let mut projected_exterior: Vec<Point<f64>> =
        Vec::with_capacity(polygon.exterior.coordinates.len());
    for coord in &polygon.exterior.coordinates {
        let point = Point::new(coord.x, coord.y);
        let projected = transformer.convert(point)?;
        projected_exterior.push(projected.into());
    }
    let exterior = LineString::from(projected_exterior);

    // Convert interior rings
    let mut projected_interiors: Vec<LineString<f64>> = Vec::with_capacity(polygon.interiors.len());
    for interior in &polygon.interiors {
        let mut projected_ring: Vec<Point<f64>> = Vec::with_capacity(interior.coordinates.len());
        for coord in &interior.coordinates {
            let point = Point::new(coord.x, coord.y);
            let projected = transformer.convert(point)?;
            projected_ring.push(projected.into());
        }
        projected_interiors.push(LineString::from(projected_ring));
    }

    let geo_polygon = GeoPolygon::new(exterior, projected_interiors);
    Ok(ProcessedGeometry::Polygon(geo_polygon))
}

/// Convert a multi polygon
///
/// # Arguments
///
/// * `polygons` - A vector of vectors of vectors of f64, representing the coordinates of the multi polygon
/// * `config` - A transformer config
///
/// # Returns
///
/// * `ProcessedGeometry::MultiPolygon` - A projected multi polygon
pub fn convert_multi_polygon(
    polygons: Vec<Polygon>,
    config: &TransformerConfig,
) -> Result<ProcessedGeometry, ProjectionError> {
    let mut projected_polygons = Vec::with_capacity(polygons.len());
    for polygon in polygons {
        let polygon = convert_polygon(polygon, config)?;
        match polygon {
            ProcessedGeometry::Polygon(p) => projected_polygons.push(p),
            _ => {
                return Err(ProjectionError::InvalidCoordinates(
                    "Expected Polygon geometry".to_string(),
                ));
            }
        }
    }
    Ok(ProcessedGeometry::MultiPolygon(MultiPolygon::from(
        projected_polygons,
    )))
}

/// Process a feature
///
/// # Arguments
///
/// * `feature` - A feature with a geometry
/// * `config` - A transformer config
///
/// # Returns
///
/// * `ProcessedGeometry` - A processed geometry
pub fn process_feature_geometry(
    feature: Feature,
    config: &TransformerConfig,
) -> Result<ProcessedGeometry, ProjectionError> {
    if let Some(geometry) = feature.geometry {
        process_geometry(geometry, config)
    } else {
        Err(ProjectionError::InvalidGeometryType)
    }
}

/// Process a geometry
///
/// # Arguments
///
/// * `geometry` - A geometry
/// * `config` - A transformer config
///
/// # Returns
///
/// * `ProcessedGeometry` - A processed geometry
pub fn process_geometry(
    geometry: Geometry,
    config: &TransformerConfig,
) -> Result<ProcessedGeometry, ProjectionError> {
    let processor = GeometryProcessor::new(geometry, config.clone());
    processor.convert()
}

/// Process a feature collection
///
/// # Arguments
///
/// * `json_value` - A JSON value
///
/// # Returns
///
/// * `geojson::GeoJson` - A processed feature collection
pub fn process_feature_collection(
    json_value: serde_json::Value,
) -> Result<geojson::GeoJson, ProjectionError> {
    let geojson = geojson::GeoJson::from_json_value(json_value)?;
    let config = TransformerConfig::default();
    match geojson {
        geojson::GeoJson::Feature(feature) => {
            let geometry = process_feature_geometry(feature, &config)?;
            Ok(geojson::GeoJson::Feature(geojson::Feature {
                bbox: None,
                geometry: Some(geometry.to_geojson_geometry()),
                id: None,
                properties: None,
                foreign_members: None,
            }))
        }
        geojson::GeoJson::FeatureCollection(feature_collection) => {
            let mut features = Vec::with_capacity(feature_collection.features.len());
            for feature in feature_collection.features {
                let geometry = process_feature_geometry(feature, &config)?;
                features.push(geojson::Feature {
                    bbox: None,
                    geometry: Some(geometry.to_geojson_geometry()),
                    id: None,
                    properties: None,
                    foreign_members: None,
                });
            }
            Ok(geojson::GeoJson::FeatureCollection(
                geojson::FeatureCollection {
                    bbox: None,
                    features,
                    foreign_members: None,
                },
            ))
        }
        geojson::GeoJson::Geometry(geometry) => {
            let geometry = process_geometry(geometry, &config)?;
            Ok(geojson::GeoJson::Geometry(geometry.to_geojson_geometry()))
        }
    }
}
