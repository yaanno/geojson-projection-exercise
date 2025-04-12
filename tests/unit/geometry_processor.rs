use geo::Point;
use geojson::{Geometry, Value};
use proj_exercise_simple::helpers::GeometryProcessor;
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
        let mut processor = GeometryProcessor::new(point, &mut config);
        let mut buffer_pool = CoordinateBufferPool::new(10, 100);

        let result = processor.convert(&mut buffer_pool).unwrap();
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
        let mut processor = GeometryProcessor::new(line_string, &mut config);
        let mut buffer_pool = CoordinateBufferPool::new(10, 100);
        let result = processor.convert(&mut buffer_pool).unwrap();
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
        let mut processor = GeometryProcessor::new(polygon, &mut config);
        let mut buffer_pool = CoordinateBufferPool::new(10, 100);

        let result = processor.convert(&mut buffer_pool).unwrap();
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
        let mut processor = GeometryProcessor::new(invalid_geometry, &mut config);
        let mut buffer_pool = CoordinateBufferPool::new(10, 100);

        let result = processor.convert(&mut buffer_pool);
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
        let mut processor = GeometryProcessor::new(point, &mut config);
        let mut buffer_pool = CoordinateBufferPool::new(10, 100);

        let result = processor.convert(&mut buffer_pool).unwrap();
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
        let mut processor = GeometryProcessor::new(points, &mut config);
        let mut buffer_pool = CoordinateBufferPool::new(10, 100);

        let result = processor.convert(&mut buffer_pool).unwrap();
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
}
