use crate::coordinates::{Coordinate, Line, Polygon as ProjectPolygon};
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
    polygon: ProjectPolygon,
}

impl PolygonProcessor {
    fn new(polygon: ProjectPolygon) -> Self {
        Self { polygon }
    }
}

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
        let mut projected_interiors_geo = Vec::new();
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
            projected_interiors_geo.push(line_string);
        }
        buffer_pool.return_point_buffer(ring_buffer)?;

        let geo_polygon = GeoPolygon::new(exterior, projected_interiors_geo);
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
    lines: Vec<Line>,
}

impl MultiLineStringProcessor {
    fn new(lines: Vec<Line>) -> Self {
        Self { lines }
    }
}

impl GeometryProcessorTrait for MultiLineStringProcessor {
    fn process(
        &self,
        config: &mut TransformerConfig,
        buffer_pool: &mut CoordinateBufferPool,
    ) -> Result<ProcessedGeometry, ProjectionError> {
        let transformer = config.get_transformer()?;
        let mut projected_lines = Vec::new();

        for line in &self.lines {
            let mut projected_coords = buffer_pool.get_point_buffer()?;
            projected_coords.clear();
            projected_coords.reserve(line.coordinates.len());

            let mut batch_buffer = Vec::with_capacity(1000);
            for chunk in line.coordinates.chunks(1000) {
                batch_buffer.clear();
                batch_buffer.reserve(chunk.len());
                for coord in chunk {
                    let point = Point::new(coord.x, coord.y);
                    let projected = transformer.convert(point)?;
                    batch_buffer.push(projected.into());
                }
                projected_coords.extend_from_slice(&batch_buffer);
            }

            let projected_line = LineString::from(
                projected_coords
                    .iter()
                    .map(|c| geo::Coord::from((c.x, c.y)))
                    .collect::<Vec<_>>(),
            );
            buffer_pool.return_point_buffer(projected_coords)?;
            projected_lines.push(projected_line);
        }

        Ok(ProcessedGeometry::MultiLineString(MultiLineString::new(
            projected_lines,
        )))
    }
}

struct MultiPolygonProcessor {
    polygons: Vec<ProjectPolygon>,
}

impl MultiPolygonProcessor {
    fn new(polygons: Vec<ProjectPolygon>) -> Self {
        Self { polygons }
    }
}

impl GeometryProcessorTrait for MultiPolygonProcessor {
    fn process(
        &self,
        config: &mut TransformerConfig,
        buffer_pool: &mut CoordinateBufferPool,
    ) -> Result<ProcessedGeometry, ProjectionError> {
        let transformer = config.get_transformer()?;
        let mut projected_polygons = Vec::new();
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
            let mut projected_interiors_geo = Vec::new();

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
                projected_interiors_geo.push(line_string);
            }

            let geo_polygon = GeoPolygon::new(exterior, projected_interiors_geo);
            projected_polygons.push(geo_polygon);
        }

        buffer_pool.return_point_buffer(ring_buffer)?;
        buffer_pool.return_point_buffer(projected_exterior)?;

        Ok(ProcessedGeometry::MultiPolygon(MultiPolygon::from(
            projected_polygons,
        )))
    }
}

/// Main geometry processor that uses specialized processors
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
                let processor =
                    PolygonProcessor::new(ProjectPolygon::new(Line::new(exterior), interiors));
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
                let mut project_lines = Vec::new();
                for line in lines {
                    for point in line {
                        Self::validate_coordinate(point[0], point[1])?;
                    }
                    let coords = line.iter().map(|p| Coordinate::new(p[0], p[1])).collect();
                    project_lines.push(Line::new(coords));
                }
                let processor = MultiLineStringProcessor::new(project_lines);
                processor.process(self.config, buffer_pool)
            }
            geojson::Value::MultiPolygon(polygons) => {
                let mut project_polygons = Vec::new();
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
                    project_polygons.push(ProjectPolygon::new(Line::new(exterior), interiors));
                }
                let processor = MultiPolygonProcessor::new(project_polygons);
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
