use crate::coordinates::{Coordinate, Line, Polygon};
use crate::error::ProjectionError;
use crate::helpers::ProcessedGeometry;
use crate::pool::CoordinateBufferPool;
use crate::transformer::TransformerConfig;
use geo::{LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon as GeoPolygon};
use geojson::Geometry;

// Trait for geometry-specific processing
pub(crate) trait GeometryProcessorTrait {
    fn process(
        &self,
        config: &mut TransformerConfig,
        buffer_pool: &mut CoordinateBufferPool,
    ) -> Result<ProcessedGeometry, ProjectionError>;
}

// Specialized processor for points
struct PointProcessor {
    point: Point<f64>,
}

impl PointProcessor {
    fn new(point: Point<f64>) -> Self {
        Self { point }
    }
}

impl GeometryProcessorTrait for PointProcessor {
    fn process(
        &self,
        config: &mut TransformerConfig,
        _buffer_pool: &mut CoordinateBufferPool,
    ) -> Result<ProcessedGeometry, ProjectionError> {
        let transformer = config.get_transformer()?;
        let projected = transformer.convert(self.point)?;
        Ok(ProcessedGeometry::Point(projected))
    }
}

// Specialized processor for line strings
struct LineStringProcessor {
    coordinates: Vec<Coordinate>,
}

impl LineStringProcessor {
    fn new(coordinates: Vec<Coordinate>) -> Self {
        Self { coordinates }
    }
}

impl GeometryProcessorTrait for LineStringProcessor {
    fn process(
        &self,
        config: &mut TransformerConfig,
        buffer_pool: &mut CoordinateBufferPool,
    ) -> Result<ProcessedGeometry, ProjectionError> {
        let transformer = config.get_transformer()?;
        let mut projected_coords = buffer_pool.get_point_buffer()?;
        projected_coords.clear();
        projected_coords.reserve(self.coordinates.len());

        // Process coordinates in batches of 1000
        let mut batch_buffer = Vec::with_capacity(1000);
        for chunk in self.coordinates.chunks(1000) {
            batch_buffer.clear();
            batch_buffer.reserve(chunk.len());
            for coord in chunk {
                let point = Point::new(coord.x, coord.y);
                let projected = transformer.convert(point)?;
                batch_buffer.push(projected.into());
            }
            projected_coords.extend_from_slice(&batch_buffer);
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
}

// Specialized processor for polygons
struct PolygonProcessor {
    polygon: Polygon,
}

impl PolygonProcessor {
    fn new(polygon: Polygon) -> Self {
        Self { polygon }
    }
}

/// A specialized processor for polygons
///
/// This processor is responsible for processing polygons. It iterates over each coordinate,
/// projecting each coordinate and constructing the resulting polygon.
///
/// # Arguments
///
/// * `config` - A mutable reference to the transformer configuration.
/// * `buffer_pool` - A mutable reference to the coordinate buffer pool.
///
/// # Returns
///
/// * `Result<ProcessedGeometry, ProjectionError>` - The processed geometry or an error if projection fails.
///
/// # Errors
///
/// * `ProjectionError` - If there is an error during projection.
impl GeometryProcessorTrait for PolygonProcessor {
    fn process(
        &self,
        config: &mut TransformerConfig,
        buffer_pool: &mut CoordinateBufferPool,
    ) -> Result<ProcessedGeometry, ProjectionError> {
        let transformer = config.get_transformer()?;

        // Process exterior ring
        let mut projected_exterior = buffer_pool.get_point_buffer()?;
        projected_exterior.clear();
        projected_exterior.reserve(self.polygon.exterior.coordinates.len());

        let mut batch_buffer = Vec::with_capacity(1000);
        for chunk in self.polygon.exterior.coordinates.chunks(1000) {
            batch_buffer.clear();
            batch_buffer.reserve(chunk.len());
            for coord in chunk {
                let point = Point::new(coord.x, coord.y);
                let projected = transformer.convert(point)?;
                batch_buffer.push(projected.into());
            }
            projected_exterior.extend_from_slice(&batch_buffer);
        }

        let exterior = LineString::from(
            projected_exterior
                .iter()
                .map(|c| geo::Coord::from((c.x, c.y)))
                .collect::<Vec<_>>(),
        );
        buffer_pool.return_point_buffer(projected_exterior)?;

        // Process interior rings
        let mut projected_interiors = buffer_pool.get_polygon_buffer()?;
        projected_interiors.clear();
        projected_interiors.reserve(self.polygon.interiors.len());

        let mut ring_buffer = buffer_pool.get_point_buffer()?;
        for interior in &self.polygon.interiors {
            ring_buffer.clear();
            ring_buffer.reserve(interior.coordinates.len());

            for chunk in interior.coordinates.chunks(1000) {
                batch_buffer.clear();
                batch_buffer.reserve(chunk.len());
                for coord in chunk {
                    let point = Point::new(coord.x, coord.y);
                    let projected = transformer.convert(point)?;
                    batch_buffer.push(projected.into());
                }
                ring_buffer.extend_from_slice(&batch_buffer);
            }

            let line_string = LineString::from(
                ring_buffer
                    .iter()
                    .map(|c| geo::Coord::from((c.x, c.y)))
                    .collect::<Vec<_>>(),
            );
            projected_interiors.push(Line::from_geo(&line_string));
        }
        buffer_pool.return_point_buffer(ring_buffer)?;

        let geo_polygon = GeoPolygon::new(
            exterior,
            projected_interiors.iter().map(|ls| ls.to_geo()).collect(),
        );
        buffer_pool.return_polygon_buffer(projected_interiors)?;
        Ok(ProcessedGeometry::Polygon(geo_polygon))
    }
}

struct MultiPointProcessor {
    coordinates: Vec<Coordinate>,
}

impl MultiPointProcessor {
    fn new(coordinates: Vec<Coordinate>) -> Self {
        Self { coordinates }
    }
}

/// A specialized processor for multi points
///
/// This processor is responsible for processing multi points. It iterates over each coordinate,
/// projecting each coordinate and constructing the resulting multi point.
///
/// # Arguments
///
/// * `config` - A mutable reference to the transformer configuration.
/// * `buffer_pool` - A mutable reference to the coordinate buffer pool.
///
/// # Returns
///
/// * `Result<ProcessedGeometry, ProjectionError>` - The processed geometry or an error if projection fails.
///
/// # Errors
///
/// * `ProjectionError` - If there is an error during projection.
impl GeometryProcessorTrait for MultiPointProcessor {
    fn process(
        &self,
        config: &mut TransformerConfig,
        buffer_pool: &mut CoordinateBufferPool,
    ) -> Result<ProcessedGeometry, ProjectionError> {
        let transformer = config.get_transformer()?;
        let mut projected_coords = buffer_pool.get_point_buffer()?;

        for coord in &self.coordinates {
            let point = Point::new(coord.x, coord.y);
            let projected = transformer.convert(point)?;
            projected_coords.push(projected.into());
        }
        buffer_pool.return_point_buffer(projected_coords.clone())?;

        let multi_point = MultiPoint::from(
            projected_coords
                .iter()
                .map(|c| geo::Coord::from((c.x, c.y)))
                .collect::<Vec<_>>(),
        );
        Ok(ProcessedGeometry::MultiPoint(multi_point))
    }
}

struct MultiLineStringProcessor {
    coordinates: Vec<Coordinate>,
}

impl MultiLineStringProcessor {
    fn new(coordinates: Vec<Coordinate>) -> Self {
        Self { coordinates }
    }
}

/// A specialized processor for multi line strings
///
/// This processor is responsible for processing multi line strings. It iterates over each coordinate,
/// projecting each coordinate and constructing the resulting multi line string.
///
/// # Arguments
///
/// * `config` - A mutable reference to the transformer configuration.
/// * `buffer_pool` - A mutable reference to the coordinate buffer pool.
///
/// # Returns
///
/// * `Result<ProcessedGeometry, ProjectionError>` - The processed geometry or an error if projection fails.
///
/// # Errors
///
/// * `ProjectionError` - If there is an error during projection.
impl GeometryProcessorTrait for MultiLineStringProcessor {
    fn process(
        &self,
        config: &mut TransformerConfig,
        buffer_pool: &mut CoordinateBufferPool,
    ) -> Result<ProcessedGeometry, ProjectionError> {
        let transformer = config.get_transformer()?;
        let mut projected_coords = buffer_pool.get_point_buffer()?;

        for coord in &self.coordinates {
            let point = Point::new(coord.x, coord.y);
            let projected = transformer.convert(point)?;
            projected_coords.push(projected.into());
        }
        buffer_pool.return_point_buffer(projected_coords.clone())?;

        let multi_line_string = MultiLineString::from(
            projected_coords
                .iter()
                .map(|c| geo::Coord::from((c.x, c.y)))
                .collect::<Vec<_>>(),
        );
        Ok(ProcessedGeometry::MultiLineString(multi_line_string))
    }
}

struct MultiPolygonProcessor {
    polygons: Vec<Polygon>,
}

impl MultiPolygonProcessor {
    fn new(polygons: Vec<Polygon>) -> Self {
        Self { polygons }
    }
}

/// A specialized processor for multi polygons
///
/// This processor is responsible for processing multi polygons. It iterates over each polygon,
/// projecting each coordinate and constructing the resulting multi polygon.
///
/// # Arguments
///
/// * `config` - A mutable reference to the transformer configuration.
/// * `buffer_pool` - A mutable reference to the coordinate buffer pool.
///
/// # Returns
///
/// * `Result<ProcessedGeometry, ProjectionError>` - The processed geometry or an error if projection fails.
///
/// # Errors
///
/// * `ProjectionError` - If there is an error during projection.
impl GeometryProcessorTrait for MultiPolygonProcessor {
    fn process(
        &self,
        config: &mut TransformerConfig,
        buffer_pool: &mut CoordinateBufferPool,
    ) -> Result<ProcessedGeometry, ProjectionError> {
        let transformer = config.get_transformer()?;
        let mut projected_polygons = buffer_pool.get_polygon_buffer()?;
        projected_polygons.clear();
        projected_polygons.reserve(self.polygons.len());

        let mut batch_buffer = Vec::with_capacity(1000);
        let mut ring_buffer = buffer_pool.get_point_buffer()?;
        let mut projected_exterior = buffer_pool.get_point_buffer()?;

        for polygon in &self.polygons {
            // Process exterior ring
            projected_exterior.clear();
            projected_exterior.reserve(polygon.exterior.coordinates.len());

            for chunk in polygon.exterior.coordinates.chunks(1000) {
                batch_buffer.clear();
                batch_buffer.reserve(chunk.len());
                for coord in chunk {
                    let point = Point::new(coord.x, coord.y);
                    let projected = transformer.convert(point)?;
                    batch_buffer.push(projected.into());
                }
                projected_exterior.extend_from_slice(&batch_buffer);
            }

            let exterior = LineString::from(
                projected_exterior
                    .iter()
                    .map(|c| geo::Coord::from((c.x, c.y)))
                    .collect::<Vec<_>>(),
            );

            // Process interior rings
            let mut projected_interiors = Vec::new();
            projected_interiors.reserve(polygon.interiors.len());

            for interior in &polygon.interiors {
                ring_buffer.clear();
                ring_buffer.reserve(interior.coordinates.len());

                for chunk in interior.coordinates.chunks(1000) {
                    batch_buffer.clear();
                    batch_buffer.reserve(chunk.len());
                    for coord in chunk {
                        let point = Point::new(coord.x, coord.y);
                        let projected = transformer.convert(point)?;
                        batch_buffer.push(projected.into());
                    }
                    ring_buffer.extend_from_slice(&batch_buffer);
                }

                let line_string = LineString::from(
                    ring_buffer
                        .iter()
                        .map(|c| geo::Coord::from((c.x, c.y)))
                        .collect::<Vec<_>>(),
                );
                projected_interiors.push(Line::from_geo(&line_string));
            }

            let geo_polygon = GeoPolygon::new(
                exterior,
                projected_interiors.iter().map(|ls| ls.to_geo()).collect(),
            );
            projected_polygons.push(Line::from_geo(&geo_polygon.exterior()));
        }

        buffer_pool.return_point_buffer(ring_buffer)?;
        buffer_pool.return_point_buffer(projected_exterior)?;

        let multi_polygon = MultiPolygon::from(
            projected_polygons
                .iter()
                .map(|ls| GeoPolygon::new(ls.to_geo(), vec![]))
                .collect::<Vec<_>>(),
        );
        buffer_pool.return_polygon_buffer(projected_polygons)?;
        Ok(ProcessedGeometry::MultiPolygon(multi_polygon))
    }
}

/// Main geometry processor that uses specialized processors
///
/// This processor uses specialized processors for each geometry type. It validates the coordinates,
/// and then delegates the processing to the appropriate specialized processor.
///
/// # Arguments
///
/// * `geometry` - A reference to the geometry to be processed.
/// * `config` - A mutable reference to the transformer configuration.
/// * `buffer_pool` - A mutable reference to the coordinate buffer pool.
pub struct GeometryProcessor<'a> {
    geometry: &'a Geometry,
    config: &'a mut TransformerConfig,
}

impl<'a> GeometryProcessor<'a> {
    pub fn new(geometry: &'a Geometry, config: &'a mut TransformerConfig) -> Self {
        Self { geometry, config }
    }

    fn validate_coordinate(x: f64, y: f64) -> Result<(), ProjectionError> {
        if x.is_nan() || y.is_nan() {
            return Err(ProjectionError::InvalidCoordinates(
                "NaN coordinates are not allowed".to_string(),
            ));
        }
        if !(-180.0..=180.0).contains(&x) || !(-90.0..=90.0).contains(&y) {
            return Err(ProjectionError::InvalidCoordinates(
                "Coordinates out of valid range".to_string(),
            ));
        }
        Ok(())
    }

    pub fn process(
        &mut self,
        buffer_pool: &mut CoordinateBufferPool,
    ) -> Result<ProcessedGeometry, ProjectionError> {
        match &self.geometry.value {
            geojson::Value::Point(point) => {
                Self::validate_coordinate(point[0], point[1])?;
                let processor = PointProcessor::new(Point::new(point[0], point[1]));
                processor.process(self.config, buffer_pool)
            }
            geojson::Value::LineString(line_string) => {
                for point in line_string {
                    Self::validate_coordinate(point[0], point[1])?;
                }
                let coords = line_string
                    .iter()
                    .map(|p| Coordinate::new(p[0], p[1]))
                    .collect();
                let processor = LineStringProcessor::new(coords);
                processor.process(self.config, buffer_pool)
            }
            geojson::Value::Polygon(polygon) => {
                for ring in polygon {
                    for point in ring {
                        Self::validate_coordinate(point[0], point[1])?;
                    }
                }
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
                let processor = PolygonProcessor::new(Polygon::new(Line::new(exterior), interiors));
                processor.process(self.config, buffer_pool)
            }
            geojson::Value::MultiPoint(points) => {
                for point in points {
                    Self::validate_coordinate(point[0], point[1])?;
                }
                let coords = points.iter().map(|p| Coordinate::new(p[0], p[1])).collect();
                let processor = MultiPointProcessor::new(coords);
                processor.process(self.config, buffer_pool)
            }
            geojson::Value::MultiLineString(lines) => {
                for line in lines {
                    for point in line {
                        Self::validate_coordinate(point[0], point[1])?;
                    }
                }
                let coords = lines
                    .iter()
                    .flat_map(|line| line.iter().map(|p| Coordinate::new(p[0], p[1])))
                    .collect();
                let processor = MultiLineStringProcessor::new(coords);
                processor.process(self.config, buffer_pool)
            }
            geojson::Value::MultiPolygon(polygons) => {
                let mut processed_polygons = Vec::new();
                for polygon in polygons {
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
                    processed_polygons.push(Polygon::new(Line::new(exterior), interiors));
                }
                let processor = MultiPolygonProcessor::new(processed_polygons);
                processor.process(self.config, buffer_pool)
            }
            geojson::Value::GeometryCollection(geometries) => {
                let mut processed_geometries: Vec<ProcessedGeometry> = Vec::new();
                for geometry in geometries {
                    let mut processor = GeometryProcessor::new(geometry, self.config);
                    let result = processor.process(buffer_pool)?;
                    processed_geometries.push(result);
                }
                let geometries: Vec<geo::Geometry<f64>> = processed_geometries
                    .into_iter()
                    .map(|g| match g {
                        ProcessedGeometry::Point(p) => geo::Geometry::Point(p),
                        ProcessedGeometry::LineString(ls) => geo::Geometry::LineString(ls),
                        ProcessedGeometry::Polygon(p) => geo::Geometry::Polygon(p),
                        ProcessedGeometry::MultiPoint(mp) => geo::Geometry::MultiPoint(mp),
                        ProcessedGeometry::MultiLineString(mls) => {
                            geo::Geometry::MultiLineString(mls)
                        }
                        ProcessedGeometry::MultiPolygon(mp) => geo::Geometry::MultiPolygon(mp),
                        ProcessedGeometry::GeometryCollection(_) => unreachable!(),
                    })
                    .collect();
                Ok(ProcessedGeometry::GeometryCollection(
                    geo::GeometryCollection::from(geometries),
                ))
            }
        }
    }
}
