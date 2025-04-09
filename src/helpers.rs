use geo::{CoordsIter, LineString, Point, Polygon};
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
    Polygon(Polygon<f64>),
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
                geojson::Geometry::new(geojson::Value::Point(vec![point.x(), point.y()]))
            }
            ProcessedGeometry::LineString(line_string) => {
                let coordinates: Vec<Vec<f64>> = line_string
                    .coords_iter()
                    .map(|coord| vec![coord.x, coord.y])
                    .collect();
                geojson::Geometry::new(geojson::Value::LineString(coordinates))
            }
            ProcessedGeometry::Polygon(polygon) => {
                let mut rings: Vec<Vec<Vec<f64>>> = Vec::new();

                // Process the exterior ring
                let exterior_ring: Vec<Vec<f64>> = polygon
                    .exterior()
                    .coords_iter()
                    .map(|coord| vec![coord.x, coord.y])
                    .collect();
                rings.push(exterior_ring);

                // Process the interior rings (holes)
                for interior in polygon.interiors() {
                    let interior_ring: Vec<Vec<f64>> = interior
                        .coords_iter()
                        .map(|coord| vec![coord.x, coord.y])
                        .collect();
                    rings.push(interior_ring);
                }

                geojson::Geometry::new(geojson::Value::Polygon(rings))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct TransformerConfig {
    from: String,
    to: String,
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
    /// Get a default TransformerConfig
    ///
    /// # Returns
    ///
    /// * `TransformerConfig` - A default transformer config
    pub fn default() -> Self {
        Self {
            from: "EPSG:4326".to_string(),
            to: "EPSG:25832".to_string(),
        }
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

/// Get a transformer
///
/// # Returns
///
/// * `Proj` - A transformer
pub fn get_transformer(config: TransformerConfig) -> Result<Proj, ProjectionError> {
    let transformer = config.get_transformer()?;
    Ok(transformer)
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

/// Convert a linear ring
///
/// # Arguments
///
/// * `ring` - A vector of vectors of f64, representing the coordinates of the linear ring
/// * `config` - A transformer config
///
/// # Returns
///
/// * `LineString<f64>` - A projected linear ring
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
/// * `config` - A transformer config
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
/// * `config` - A transformer config
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
/// * `config` - A transformer config
///
/// # Returns
///
/// * `ProcessedGeometry` - A processed geometry
pub fn process_geometry(
    geometry: Geometry,
    config: TransformerConfig,
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
    let geojson_data = geojson::GeoJson::from_json_value(json_value)?;
    let mut transformed_features: Vec<Feature> = Vec::new();
    // TODO: Make this configurable
    let config = TransformerConfig::new("EPSG:4326".to_string(), "EPSG:25832".to_string());

    match geojson_data {
        geojson::GeoJson::FeatureCollection(fc) => {
            for original_feature in fc.features {
                let processed_geometry =
                    process_feature_geometry(original_feature.clone(), config.clone())?;
                let transformed_feature = match processed_geometry {
                    ProcessedGeometry::Point(point) => Feature {
                        bbox: None,
                        geometry: Some(ProcessedGeometry::Point(point).to_geojson_geometry()),
                        id: original_feature.id,
                        properties: original_feature.properties,
                        foreign_members: original_feature.foreign_members,
                    },
                    ProcessedGeometry::LineString(line_string) => Feature {
                        bbox: None,
                        geometry: Some(
                            ProcessedGeometry::LineString(line_string).to_geojson_geometry(),
                        ),
                        id: original_feature.id,
                        properties: original_feature.properties,
                        foreign_members: original_feature.foreign_members,
                    },
                    ProcessedGeometry::Polygon(polygon) => Feature {
                        bbox: None,
                        geometry: Some(ProcessedGeometry::Polygon(polygon).to_geojson_geometry()),
                        id: original_feature.id,
                        properties: original_feature.properties,
                        foreign_members: original_feature.foreign_members,
                    },
                };
                transformed_features.push(transformed_feature);
            }
            // Create the new FeatureCollection
            let new_feature_collection = geojson::FeatureCollection {
                bbox: None,
                features: transformed_features,
                foreign_members: None,
            };

            return Ok(geojson::GeoJson::FeatureCollection(new_feature_collection));
        }
        geojson::GeoJson::Feature(feature) => {
            let processed_geometry = process_feature_geometry(feature.clone(), config)?;
            let transformed_feature = Feature {
                bbox: None,
                geometry: Some(processed_geometry.to_geojson_geometry()),
                id: feature.id,
                properties: feature.properties,
                foreign_members: feature.foreign_members,
            };
            return Ok(geojson::GeoJson::Feature(transformed_feature));
        }
        geojson::GeoJson::Geometry(geometry) => {
            let processed_geometry = process_geometry(geometry.clone(), config)?;
            return Ok(geojson::GeoJson::Geometry(
                processed_geometry.to_geojson_geometry(),
            ));
        }
    }
}
