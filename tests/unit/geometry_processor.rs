use geo::Point;
use geojson::{Geometry, Value};
use proj_exercise_simple::geometry_processor::GeometryProcessor;
use proj_exercise_simple::transformer::TransformerConfig;
#[cfg(test)]
mod tests {

    use proj_exercise_simple::{
        error::ProjectionError, helpers::ProcessedGeometry, pool::CoordinateBufferPool,
    };

    use super::*;

    #[test]
    fn test_point_processing() {
        let mut config = TransformerConfig::default();
        let point = Geometry {
            value: Value::Point(vec![1.0, 2.0]),
            bbox: None,
            foreign_members: None,
        };
        let mut processor = GeometryProcessor::new(&point, &mut config);
        let mut buffer_pool = CoordinateBufferPool::new(10, 100);

        let result = processor.process(&mut buffer_pool).unwrap();
        match result {
            ProcessedGeometry::Point(p) => {
                // Expected Web Mercator coordinates for (1,2)
                assert!((p.x() - 111319.49079327357).abs() < 1e-6);
                assert!((p.y() - 222684.20850554455).abs() < 1e-6);
            }
            _ => panic!("Expected Point geometry"),
        }
    }

    #[test]
    fn test_line_string_processing() {
        let mut config = TransformerConfig::default();

        let line_string = Geometry {
            value: Value::LineString(vec![vec![0.0, 0.0], vec![1.0, 1.0], vec![2.0, 2.0]]),
            bbox: None,
            foreign_members: None,
        };
        let mut processor = GeometryProcessor::new(&line_string, &mut config);
        let mut buffer_pool = CoordinateBufferPool::new(10, 100);
        let result = processor.process(&mut buffer_pool).unwrap();
        match result {
            ProcessedGeometry::LineString(ls) => {
                assert_eq!(ls.points().count(), 3);
                let points: Vec<Point<f64>> = ls.points().collect();
                // Expected Web Mercator coordinates
                assert!((points[0].x() - 0.0).abs() < 1e-6);
                assert!((points[0].y() - 0.0).abs() < 1e-6);
                assert!((points[1].x() - 111319.49079327357).abs() < 1e-6);
                assert!((points[1].y() - 111325.14286638486).abs() < 1e-6);
                assert!((points[2].x() - 222638.98158654715).abs() < 1e-6);
                assert!((points[2].y() - 222684.20850554455).abs() < 1e-6);
            }
            _ => panic!("Expected LineString geometry"),
        }
    }

    #[test]
    fn test_polygon_processing() {
        let mut config = TransformerConfig::default();

        let polygon = Geometry {
            value: Value::Polygon(vec![vec![
                vec![0.0, 0.0],
                vec![0.0, 1.0],
                vec![1.0, 1.0],
                vec![1.0, 0.0],
                vec![0.0, 0.0],
            ]]),
            bbox: None,
            foreign_members: None,
        };
        let mut processor = GeometryProcessor::new(&polygon, &mut config);
        let mut buffer_pool = CoordinateBufferPool::new(10, 100);

        let result = processor.process(&mut buffer_pool).unwrap();
        match result {
            ProcessedGeometry::Polygon(p) => {
                assert_eq!(p.exterior().points().count(), 5);
                let points: Vec<Point<f64>> = p.exterior().points().collect();
                // Expected Web Mercator coordinates
                assert!((points[0].x() - 0.0).abs() < 1e-6);
                assert!((points[0].y() - 0.0).abs() < 1e-6);
                assert!((points[1].x() - 0.0).abs() < 1e-6);
                assert!((points[1].y() - 111325.14286638486).abs() < 1e-6);
                assert!((points[2].x() - 111319.49079327357).abs() < 1e-6);
                assert!((points[2].y() - 111325.14286638486).abs() < 1e-6);
                assert!((points[3].x() - 111319.49079327357).abs() < 1e-6);
                assert!((points[3].y() - 0.0).abs() < 1e-6);
                assert!((points[4].x() - 0.0).abs() < 1e-6);
                assert!((points[4].y() - 0.0).abs() < 1e-6);
            }
            _ => panic!("Expected Polygon geometry"),
        }
    }

    #[test]
    fn test_invalid_geometry_handling() {
        let mut config = TransformerConfig::default();

        let invalid_geometry = Geometry {
            value: Value::Point(vec![f64::NAN, f64::NAN]),
            bbox: None,
            foreign_members: None,
        };
        let mut processor = GeometryProcessor::new(&invalid_geometry, &mut config);
        let mut buffer_pool = CoordinateBufferPool::new(10, 100);

        let result = processor.process(&mut buffer_pool);
        assert!(result.is_err());
        match result.unwrap_err() {
            ProjectionError::InvalidCoordinates(_) => (),
            _ => panic!("Expected InvalidCoordinates error"),
        }
    }

    #[test]
    fn test_coordinate_transformation() {
        let mut config =
            TransformerConfig::new("EPSG:4326".to_string(), "EPSG:3857".to_string()).unwrap();

        let point = Geometry {
            value: Value::Point(vec![0.0, 0.0]),
            bbox: None,
            foreign_members: None,
        };
        let mut processor = GeometryProcessor::new(&point, &mut config);
        let mut buffer_pool = CoordinateBufferPool::new(10, 100);

        let result = processor.process(&mut buffer_pool).unwrap();
        match result {
            ProcessedGeometry::Point(p) => {
                // Expected Web Mercator coordinates for (0,0)
                assert!((p.x() - 0.0).abs() < 1e-6);
                assert!((p.y() - 0.0).abs() < 1e-6);
            }
            _ => panic!("Expected Point geometry"),
        }
    }

    #[test]
    fn test_geometry_collection_processing() {
        let mut config = TransformerConfig::default();

        let points = Geometry {
            value: Value::MultiPoint(vec![vec![0.0, 0.0], vec![1.0, 1.0]]),
            bbox: None,
            foreign_members: None,
        };
        let mut processor = GeometryProcessor::new(&points, &mut config);
        let mut buffer_pool = CoordinateBufferPool::new(10, 100);

        let result = processor.process(&mut buffer_pool).unwrap();
        match result {
            ProcessedGeometry::MultiPoint(mp) => {
                assert_eq!(mp.0.len(), 2);
                // Expected Web Mercator coordinates
                assert!((mp.0[0].x() - 0.0).abs() < 1e-6);
                assert!((mp.0[0].y() - 0.0).abs() < 1e-6);
                assert!((mp.0[1].x() - 111319.49079327357).abs() < 1e-6);
                assert!((mp.0[1].y() - 111325.14286638486).abs() < 1e-6);
            }
            _ => panic!("Expected MultiPoint geometry"),
        }
    }

    #[test]
    fn test_multi_line_string_processing() {
        let mut config = TransformerConfig::default();

        let multi_line_string = Geometry {
            value: Value::MultiLineString(vec![
                vec![vec![0.0, 0.0], vec![1.0, 1.0]],
                vec![vec![2.0, 2.0], vec![3.0, 3.0]],
            ]),
            bbox: None,
            foreign_members: None,
        };
        let mut processor = GeometryProcessor::new(&multi_line_string, &mut config);
        let mut buffer_pool = CoordinateBufferPool::new(10, 100);

        let result = processor.process(&mut buffer_pool).unwrap();
        match result {
            ProcessedGeometry::MultiLineString(mls) => {
                let lines: Vec<_> = mls.into_iter().collect();
                assert_eq!(lines.len(), 2);

                let line1_points: Vec<Point<f64>> = lines[0].points().collect();
                let line2_points: Vec<Point<f64>> = lines[1].points().collect();

                // First line
                assert!((line1_points[0].x() - 0.0).abs() < 1e-6);
                assert!((line1_points[0].y() - 0.0).abs() < 1e-6);
                assert!((line1_points[1].x() - 111319.49079327357).abs() < 1e-6);
                assert!((line1_points[1].y() - 111325.14286638486).abs() < 1e-6);

                // Second line
                assert!((line2_points[0].x() - 222638.98158654715).abs() < 1e-6);
                assert!((line2_points[0].y() - 222684.20850554455).abs() < 1e-6);
                assert!((line2_points[1].x() - 333958.4723798207).abs() < 1e-6);
                assert!((line2_points[1].y() - 334111.1714019596).abs() < 1e-6);
            }
            _ => panic!("Expected MultiLineString geometry"),
        }
    }

    #[test]
    fn test_multi_polygon_processing() {
        let mut config = TransformerConfig::default();

        let multi_polygon = Geometry {
            value: Value::MultiPolygon(vec![
                vec![vec![
                    vec![0.0, 0.0],
                    vec![0.0, 1.0],
                    vec![1.0, 1.0],
                    vec![1.0, 0.0],
                    vec![0.0, 0.0],
                ]],
                vec![vec![
                    vec![2.0, 2.0],
                    vec![2.0, 3.0],
                    vec![3.0, 3.0],
                    vec![3.0, 2.0],
                    vec![2.0, 2.0],
                ]],
            ]),
            bbox: None,
            foreign_members: None,
        };
        let mut processor = GeometryProcessor::new(&multi_polygon, &mut config);
        let mut buffer_pool = CoordinateBufferPool::new(10, 100);

        let result = processor.process(&mut buffer_pool).unwrap();
        match result {
            ProcessedGeometry::MultiPolygon(mp) => {
                let polygons: Vec<_> = mp.into_iter().collect();
                assert_eq!(polygons.len(), 2);

                let poly1_exterior_points: Vec<Point<f64>> =
                    polygons[0].exterior().points().collect();
                let poly2_exterior_points: Vec<Point<f64>> =
                    polygons[1].exterior().points().collect();

                // First polygon
                assert!((poly1_exterior_points[0].x() - 0.0).abs() < 1e-6);
                assert!((poly1_exterior_points[0].y() - 0.0).abs() < 1e-6);
                assert!((poly1_exterior_points[1].x() - 0.0).abs() < 1e-6);
                assert!((poly1_exterior_points[1].y() - 111325.14286638486).abs() < 1e-6);

                // Second polygon
                assert!((poly2_exterior_points[0].x() - 222638.98158654715).abs() < 1e-6);
                assert!((poly2_exterior_points[0].y() - 222684.20850554455).abs() < 1e-6);
                assert!((poly2_exterior_points[1].x() - 222638.98158654715).abs() < 1e-6);
                assert!((poly2_exterior_points[1].y() - 334111.1714019596).abs() < 1e-6);
                // Updated value
            }
            _ => panic!("Expected MultiPolygon geometry"),
        }
    }

    #[test]
    fn test_multi_polygon_with_interiors_processing() {
        let mut config = TransformerConfig::default();

        let multi_polygon = Geometry {
            value: Value::MultiPolygon(vec![
                // First Polygon (with an interior)
                vec![
                    // Exterior Ring
                    vec![
                        vec![0.0, 0.0],
                        vec![0.0, 2.0],
                        vec![2.0, 2.0],
                        vec![2.0, 0.0],
                        vec![0.0, 0.0],
                    ],
                    // Interior Ring
                    vec![
                        vec![0.5, 0.5],
                        vec![0.5, 1.5],
                        vec![1.5, 1.5],
                        vec![1.5, 0.5],
                        vec![0.5, 0.5],
                    ],
                ],
                // Second Polygon (without interior)
                vec![vec![
                    vec![3.0, 3.0],
                    vec![3.0, 4.0],
                    vec![4.0, 4.0],
                    vec![4.0, 3.0],
                    vec![3.0, 3.0],
                ]],
            ]),
            bbox: None,
            foreign_members: None,
        };
        let mut processor = GeometryProcessor::new(&multi_polygon, &mut config);
        let mut buffer_pool = CoordinateBufferPool::new(10, 100);

        let result = processor.process(&mut buffer_pool).unwrap();
        match result {
            ProcessedGeometry::MultiPolygon(mp) => {
                assert_eq!(mp.0.len(), 2); // Expecting two polygons

                // Check the first polygon (with interior)
                let poly1 = &mp.0[0];
                assert_eq!(poly1.exterior().points().count(), 5);
                assert_eq!(poly1.interiors().len(), 1);
                assert_eq!(poly1.interiors()[0].points().count(), 5);

                // Check the second polygon (without interior)
                let poly2 = &mp.0[1];
                assert_eq!(poly2.exterior().points().count(), 5);
                assert_eq!(poly2.interiors().len(), 0);

                // More detailed coordinate checks can be added here if needed
            }
            _ => panic!("Expected MultiPolygon with interiors geometry"),
        }
    }
}
