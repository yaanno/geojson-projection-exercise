use crate::coordinates::{Line, Polygon};
use crate::geometry_processor::GeometryProcessor;
use crate::pool::CoordinateBufferPool;
use crate::transformer::TransformerConfig;
use crate::{coordinates::Coordinate, error::ProjectionError};

use geo::{
    CoordsIter, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon, Point,
    Polygon as GeoPolygon,
};
use geojson::{Feature, Geometry};

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
    pub fn to_geojson_geometry(self) -> geojson::Geometry {
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
            ProcessedGeometry::MultiPoint(multi_point) => {
                let coords = multi_point.iter().map(|p| vec![p.x(), p.y()]).collect();
                geojson::Geometry::new(geojson::Value::MultiPoint(coords))
            }
            ProcessedGeometry::MultiLineString(multi_line_string) => {
                let lines = multi_line_string
                    .iter()
                    .map(|ls| {
                        ls.coords_iter()
                            .map(|coord| vec![coord.x, coord.y])
                            .collect()
                    })
                    .collect();
                geojson::Geometry::new(geojson::Value::MultiLineString(lines))
            }
            ProcessedGeometry::MultiPolygon(multi_polygon) => {
                let polygons = multi_polygon
                    .iter()
                    .map(|poly| {
                        let mut rings = vec![poly
                            .exterior()
                            .coords_iter()
                            .map(|coord| vec![coord.x, coord.y])
                            .collect()];
                        rings.extend(poly.interiors().iter().map(|ring| {
                            ring.coords_iter()
                                .map(|coord| vec![coord.x, coord.y])
                                .collect()
                        }));
                        rings
                    })
                    .collect();
                geojson::Geometry::new(geojson::Value::MultiPolygon(polygons))
            }
            ProcessedGeometry::GeometryCollection(collection) => {
                let geometries = collection
                    .iter()
                    .map(|geom| match geom {
                        geo::Geometry::Point(p) => {
                            let coord = Coordinate::from(*p);
                            geojson::Geometry::new(geojson::Value::Point(vec![coord.x, coord.y]))
                        }
                        geo::Geometry::LineString(ls) => {
                            let coords: Vec<Coordinate> = ls
                                .coords_iter()
                                .map(|coord| Coordinate::new(coord.x, coord.y))
                                .collect();
                            let line = Line::new(coords);
                            geojson::Geometry::new(line.to_geojson())
                        }
                        geo::Geometry::Polygon(poly) => {
                            let exterior = Line::new(
                                poly.exterior()
                                    .coords_iter()
                                    .map(|coord| Coordinate::new(coord.x, coord.y))
                                    .collect(),
                            );
                            let interiors = poly
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
                        geo::Geometry::MultiPoint(mp) => {
                            let coords = mp.iter().map(|p| vec![p.x(), p.y()]).collect();
                            geojson::Geometry::new(geojson::Value::MultiPoint(coords))
                        }
                        geo::Geometry::MultiLineString(mls) => {
                            let lines = mls
                                .iter()
                                .map(|ls| {
                                    ls.coords_iter()
                                        .map(|coord| vec![coord.x, coord.y])
                                        .collect()
                                })
                                .collect();
                            geojson::Geometry::new(geojson::Value::MultiLineString(lines))
                        }
                        geo::Geometry::MultiPolygon(mp) => {
                            let polygons = mp
                                .iter()
                                .map(|poly| {
                                    let mut rings = vec![poly
                                        .exterior()
                                        .coords_iter()
                                        .map(|coord| vec![coord.x, coord.y])
                                        .collect()];
                                    rings.extend(poly.interiors().iter().map(|ring| {
                                        ring.coords_iter()
                                            .map(|coord| vec![coord.x, coord.y])
                                            .collect()
                                    }));
                                    rings
                                })
                                .collect();
                            geojson::Geometry::new(geojson::Value::MultiPolygon(polygons))
                        }
                        geo::Geometry::GeometryCollection(_) => {
                            // Nested geometry collections are not supported in GeoJSON
                            panic!("Nested geometry collections are not supported")
                        }
                        geo::Geometry::Line(line) => {
                            let coords: Vec<Coordinate> = vec![
                                Coordinate::new(line.start.x, line.start.y),
                                Coordinate::new(line.end.x, line.end.y),
                            ];
                            let line = Line::new(coords);
                            geojson::Geometry::new(line.to_geojson())
                        }
                        geo::Geometry::Rect(rect) => {
                            let coords: Vec<Coordinate> = vec![
                                Coordinate::new(rect.min().x, rect.min().y),
                                Coordinate::new(rect.max().x, rect.min().y),
                                Coordinate::new(rect.max().x, rect.max().y),
                                Coordinate::new(rect.min().x, rect.max().y),
                                Coordinate::new(rect.min().x, rect.min().y), // Close the polygon
                            ];
                            let line = Line::new(coords);
                            let polygon = Polygon::new(line, vec![]);
                            geojson::Geometry::new(polygon.to_geojson())
                        }
                        geo::Geometry::Triangle(triangle) => {
                            let coords: Vec<Coordinate> = vec![
                                Coordinate::new(triangle.0.x, triangle.0.y),
                                Coordinate::new(triangle.1.x, triangle.1.y),
                                Coordinate::new(triangle.2.x, triangle.2.y),
                                Coordinate::new(triangle.0.x, triangle.0.y), // Close the polygon
                            ];
                            let line = Line::new(coords);
                            let polygon = Polygon::new(line, vec![]);
                            geojson::Geometry::new(polygon.to_geojson())
                        }
                    })
                    .collect();
                geojson::Geometry::new(geojson::Value::GeometryCollection(geometries))
            }
        }
    }
}

#[allow(dead_code)]
fn convert_multi_line_string(
    lines: Vec<Line>,
    config: &mut TransformerConfig,
    buffer_pool: &mut CoordinateBufferPool,
) -> Result<ProcessedGeometry, ProjectionError> {
    let mut projected_line_strings = buffer_pool.get_line_buffer()?;
    for line in lines {
        let line_string = convert_line_string(line.coordinates, config, buffer_pool)?;
        match line_string {
            ProcessedGeometry::LineString(ls) => projected_line_strings.push(Line::from_geo(&ls)),
            _ => {
                return Err(ProjectionError::InvalidCoordinates(
                    "Expected LineString geometry".to_string(),
                ));
            }
        }
    }
    let multi_line_string = MultiLineString::new(
        projected_line_strings
            .iter()
            .map(|ls| ls.to_geo())
            .collect(),
    );
    let line_strings = projected_line_strings;
    buffer_pool.return_line_buffer(line_strings)?;
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
#[allow(dead_code)]
fn convert_multi_point(
    points: Vec<Coordinate>,
    config: &mut TransformerConfig,
    buffer_pool: &mut CoordinateBufferPool,
) -> Result<ProcessedGeometry, ProjectionError> {
    let mut projected_points = buffer_pool.get_point_buffer()?;
    for point in points {
        let point = convert_point(point, config)?;
        match point {
            ProcessedGeometry::Point(p) => projected_points.push(p.into()),
            _ => {
                buffer_pool.return_point_buffer(projected_points)?;
                return Err(ProjectionError::InvalidCoordinates(
                    "Expected Point geometry".to_string(),
                ));
            }
        }
    }
    let multi_point = MultiPoint::from(projected_points.clone());
    buffer_pool.return_point_buffer(projected_points)?;
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
#[allow(dead_code)]
fn convert_point(
    point: Coordinate,
    config: &mut TransformerConfig,
) -> Result<ProcessedGeometry, ProjectionError> {
    if point.x.is_nan() || point.y.is_nan() {
        return Err(ProjectionError::InvalidCoordinates(
            "Invalid coordinates: NaN values".to_string(),
        ));
    }
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
fn convert_line_string(
    coordinates: Vec<Coordinate>,
    config: &mut TransformerConfig,
    buffer_pool: &mut CoordinateBufferPool,
) -> Result<ProcessedGeometry, ProjectionError> {
    let transformer = config.get_transformer()?;
    let mut projected_coords = buffer_pool.get_point_buffer()?;

    for coord in coordinates {
        let point = Point::new(coord.x, coord.y);
        let projected = transformer.convert(point)?;
        projected_coords.push(projected.into());
    }

    let line_string = LineString::from(
        projected_coords
            .iter()
            .map(|c| geo::Coord::from((c.x, c.y)))
            .collect::<Vec<_>>(),
    );
    buffer_pool.return_point_buffer(projected_coords)?;
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
fn convert_polygon(
    polygon: Polygon,
    config: &mut TransformerConfig,
    buffer_pool: &mut CoordinateBufferPool,
) -> Result<ProcessedGeometry, ProjectionError> {
    let transformer = config.get_transformer()?;

    // Convert exterior ring
    let mut projected_exterior = buffer_pool.get_point_buffer()?;
    for coord in &polygon.exterior.coordinates {
        let point = Point::new(coord.x, coord.y);
        let projected = transformer.convert(point)?;
        projected_exterior.push(projected.into());
    }
    let exterior = LineString::from(
        projected_exterior
            .iter()
            .map(|c| geo::Coord::from((c.x, c.y)))
            .collect::<Vec<_>>(),
    );
    buffer_pool.return_point_buffer(projected_exterior)?;

    // Convert interior rings
    let mut projected_interiors = buffer_pool.get_polygon_buffer()?;
    for interior in &polygon.interiors {
        let mut projected_ring = buffer_pool.get_point_buffer()?;
        for coord in &interior.coordinates {
            let point = Point::new(coord.x, coord.y);
            let projected = transformer.convert(point)?;
            projected_ring.push(projected.into());
        }
        let line_string = LineString::from(
            projected_ring
                .iter()
                .map(|c| geo::Coord::from((c.x, c.y)))
                .collect::<Vec<_>>(),
        );
        projected_interiors.push(Line::from_geo(&line_string));
        buffer_pool.return_point_buffer(projected_ring)?;
    }

    let geo_polygon = GeoPolygon::new(
        exterior,
        projected_interiors.iter().map(|ls| ls.to_geo()).collect(),
    );
    buffer_pool.return_polygon_buffer(projected_interiors)?;
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
#[allow(dead_code)]
fn convert_multi_polygon(
    polygons: Vec<Polygon>,
    config: &mut TransformerConfig,
    buffer_pool: &mut CoordinateBufferPool,
) -> Result<ProcessedGeometry, ProjectionError> {
    let mut projected_polygons = Vec::with_capacity(polygons.len());
    for polygon in polygons {
        let polygon = convert_polygon(polygon, config, buffer_pool)?;
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
fn process_feature_geometry(
    feature: Feature,
    config: &mut TransformerConfig,
    buffer_pool: &mut CoordinateBufferPool,
) -> Result<ProcessedGeometry, ProjectionError> {
    if let Some(geometry) = feature.geometry {
        process_geometry(geometry, config, buffer_pool)
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
fn process_geometry(
    geometry: Geometry,
    config: &mut TransformerConfig,
    buffer_pool: &mut CoordinateBufferPool,
) -> Result<ProcessedGeometry, ProjectionError> {
    let mut processor = GeometryProcessor::new(&geometry, config);
    processor.process(buffer_pool)
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
    let mut config = TransformerConfig::default();
    let mut buffer_pool = CoordinateBufferPool::new(10, 100);
    match geojson {
        geojson::GeoJson::Feature(feature) => {
            let geometry = process_feature_geometry(feature, &mut config, &mut buffer_pool)?;
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
                let geometry = process_feature_geometry(feature, &mut config, &mut buffer_pool)?;
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
            let geometry = process_geometry(geometry, &mut config, &mut buffer_pool)?;
            Ok(geojson::GeoJson::Geometry(geometry.to_geojson_geometry()))
        }
    }
}
