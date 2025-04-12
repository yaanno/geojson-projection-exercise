use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Value};
use proj_exercise_simple::helpers::process_feature_collection;

#[test]
fn test_complex_feature_collection() {
    // Create a complex feature collection with all geometry types
    let features = vec![
        // Point
        Feature {
            bbox: None,
            geometry: Some(Geometry::new(Value::Point(vec![1.0, 2.0]))),
            id: None,
            properties: None,
            foreign_members: None,
        },
        // LineString
        Feature {
            bbox: None,
            geometry: Some(Geometry::new(Value::LineString(vec![
                vec![0.0, 0.0],
                vec![1.0, 1.0],
                vec![2.0, 2.0],
            ]))),
            id: None,
            properties: None,
            foreign_members: None,
        },
        // Polygon
        Feature {
            bbox: None,
            geometry: Some(Geometry::new(Value::Polygon(vec![vec![
                vec![0.0, 0.0],
                vec![0.0, 1.0],
                vec![1.0, 1.0],
                vec![1.0, 0.0],
                vec![0.0, 0.0],
            ]]))),
            id: None,
            properties: None,
            foreign_members: None,
        },
        // MultiPoint
        Feature {
            bbox: None,
            geometry: Some(Geometry::new(Value::MultiPoint(vec![
                vec![0.0, 0.0],
                vec![1.0, 1.0],
                vec![2.0, 2.0],
            ]))),
            id: None,
            properties: None,
            foreign_members: None,
        },
        // MultiLineString
        Feature {
            bbox: None,
            geometry: Some(Geometry::new(Value::MultiLineString(vec![
                vec![vec![0.0, 0.0], vec![1.0, 1.0]],
                vec![vec![2.0, 2.0], vec![3.0, 3.0]],
            ]))),
            id: None,
            properties: None,
            foreign_members: None,
        },
        // MultiPolygon
        Feature {
            bbox: None,
            geometry: Some(Geometry::new(Value::MultiPolygon(vec![
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
            ]))),
            id: None,
            properties: None,
            foreign_members: None,
        },
        // GeometryCollection
        Feature {
            bbox: None,
            geometry: Some(Geometry::new(Value::GeometryCollection(vec![
                Geometry::new(Value::Point(vec![4.0, 4.0])),
                Geometry::new(Value::LineString(vec![vec![4.0, 4.0], vec![5.0, 5.0]])),
            ]))),
            id: None,
            properties: None,
            foreign_members: None,
        },
    ];

    let feature_collection = FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    };

    let geojson = GeoJson::FeatureCollection(feature_collection);
    let json_value = serde_json::to_value(geojson).unwrap();

    // Process the feature collection
    let result = process_feature_collection(json_value);
    assert!(result.is_ok());

    // Verify the results
    if let Ok(GeoJson::FeatureCollection(processed_collection)) = result {
        assert_eq!(processed_collection.features.len(), 7);

        // Verify each geometry type
        for feature in processed_collection.features {
            if let Some(geometry) = feature.geometry {
                match geometry.value {
                    Value::Point(_) => (),
                    Value::LineString(_) => (),
                    Value::Polygon(_) => (),
                    Value::MultiPoint(_) => (),
                    Value::MultiLineString(_) => (),
                    Value::MultiPolygon(_) => (),
                    Value::GeometryCollection(_) => (),
                }
            }
        }
    }
}

#[test]
fn test_complex_feature_collection_with_large_geometries() {
    // Create a feature with a large number of points using valid coordinate ranges
    let mut large_line_string = Vec::new();
    for i in 0..1000 {
        // Use valid longitude (-180 to 180) and latitude (-90 to 90) ranges
        let lon = -180.0 + (i as f64 * 0.36); // 360 degrees / 1000 points
        let lat = -90.0 + (i as f64 * 0.18); // 180 degrees / 1000 points
        large_line_string.push(vec![lon, lat]);
    }

    let feature = Feature {
        bbox: None,
        geometry: Some(Geometry::new(Value::LineString(large_line_string))),
        id: None,
        properties: None,
        foreign_members: None,
    };

    let feature_collection = FeatureCollection {
        bbox: None,
        features: vec![feature],
        foreign_members: None,
    };

    let geojson = GeoJson::FeatureCollection(feature_collection);
    let json_value = serde_json::to_value(geojson).unwrap();

    // Process with a larger buffer pool
    let result = process_feature_collection(json_value);
    match result {
        Ok(GeoJson::FeatureCollection(processed_collection)) => {
            assert_eq!(processed_collection.features.len(), 1);
            if let Some(geometry) = &processed_collection.features[0].geometry {
                match &geometry.value {
                    Value::LineString(coords) => {
                        assert_eq!(coords.len(), 1000);
                        // Verify that coordinates are valid numbers
                        for coord in coords {
                            assert_eq!(coord.len(), 2);
                            assert!(!coord[0].is_nan());
                            assert!(!coord[1].is_nan());
                        }
                    }
                    _ => panic!("Expected LineString geometry"),
                }
            } else {
                panic!("Expected geometry in feature");
            }
        }
        Err(e) => panic!("Failed to process feature collection: {:?}", e),
        _ => panic!("Expected FeatureCollection"),
    }
}

#[test]
fn test_complex_feature_collection_with_very_large_geometries() {
    // Create a feature with a very large number of points using valid coordinate ranges
    let mut very_large_line_string = Vec::new();
    for i in 0..10000 {
        // Use valid longitude (-180 to 180) and latitude (-90 to 90) ranges
        let lon = -180.0 + (i as f64 * 0.036); // 360 degrees / 10000 points
        let lat = -90.0 + (i as f64 * 0.018); // 180 degrees / 10000 points
        very_large_line_string.push(vec![lon, lat]);
    }

    let feature = Feature {
        bbox: None,
        geometry: Some(Geometry::new(Value::LineString(very_large_line_string))),
        id: None,
        properties: None,
        foreign_members: None,
    };

    let feature_collection = FeatureCollection {
        bbox: None,
        features: vec![feature],
        foreign_members: None,
    };

    let geojson = GeoJson::FeatureCollection(feature_collection);
    let json_value = serde_json::to_value(geojson).unwrap();

    // Process with a larger buffer pool
    let result = process_feature_collection(json_value);
    match result {
        Ok(GeoJson::FeatureCollection(processed_collection)) => {
            assert_eq!(processed_collection.features.len(), 1);
            if let Some(geometry) = &processed_collection.features[0].geometry {
                match &geometry.value {
                    Value::LineString(coords) => {
                        assert_eq!(coords.len(), 10000);
                        // Verify that coordinates are valid numbers
                        for coord in coords {
                            assert_eq!(coord.len(), 2);
                            assert!(!coord[0].is_nan());
                            assert!(!coord[1].is_nan());
                        }
                    }
                    _ => panic!("Expected LineString geometry"),
                }
            } else {
                panic!("Expected geometry in feature");
            }
        }
        Err(e) => panic!("Failed to process feature collection: {:?}", e),
        _ => panic!("Expected FeatureCollection"),
    }
}
