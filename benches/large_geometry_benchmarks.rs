use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Value};
use proj_exercise_simple::helpers::process_feature_collection;

// Web Mercator valid bounds (approximately)
const MAX_LATITUDE: f64 = 85.06;
const MIN_LATITUDE: f64 = -85.06;
const MAX_LONGITUDE: f64 = 180.0;
const MIN_LONGITUDE: f64 = -180.0;

fn create_large_line_string(num_points: usize) -> FeatureCollection {
    let mut line_string = Vec::new();
    // Use a smaller longitude range to prevent wrapping
    let lon_start = -90.0;
    let lon_end = 90.0;
    let lon_step = (lon_end - lon_start) / (num_points - 1) as f64;

    let lat_start = -75.0;
    let lat_end = 75.0;
    let lat_step = (lat_end - lat_start) / (num_points - 1) as f64;

    for i in 0..num_points {
        let lon = lon_start + (i as f64 * lon_step);
        let lat = lat_start + (i as f64 * lat_step);
        // Ensure coordinates stay within bounds
        let lon = lon.clamp(MIN_LONGITUDE, MAX_LONGITUDE);
        let lat = lat.clamp(MIN_LATITUDE, MAX_LATITUDE);
        line_string.push(vec![lon, lat]);
    }

    let feature = Feature {
        bbox: None,
        geometry: Some(Geometry::new(Value::LineString(line_string))),
        id: None,
        properties: None,
        foreign_members: None,
    };

    FeatureCollection {
        bbox: None,
        features: vec![feature],
        foreign_members: None,
    }
}

fn create_large_polygon(num_points: usize) -> FeatureCollection {
    let mut exterior = Vec::new();
    for i in 0..num_points {
        let angle = (i as f64 / num_points as f64) * 2.0 * std::f64::consts::PI;
        // Use a smaller range for both longitude and latitude
        let lon = 0.0 + angle.cos() * 0.5; // Scale down longitude range
        let lat = 0.0 + angle.sin() * 0.3; // Scale down latitude range
                                           // Ensure coordinates stay within bounds
        let lon = lon.clamp(MIN_LONGITUDE, MAX_LONGITUDE);
        let lat = lat.clamp(MIN_LATITUDE, MAX_LATITUDE);
        exterior.push(vec![lon, lat]);
    }
    // Close the polygon
    exterior.push(exterior[0].clone());

    let feature = Feature {
        bbox: None,
        geometry: Some(Geometry::new(Value::Polygon(vec![exterior]))),
        id: None,
        properties: None,
        foreign_members: None,
    };

    FeatureCollection {
        bbox: None,
        features: vec![feature],
        foreign_members: None,
    }
}

fn create_large_multi_polygon(num_polygons: usize, points_per_polygon: usize) -> FeatureCollection {
    let mut features = Vec::new();

    for i in 0..num_polygons {
        let mut exterior = Vec::new();
        for j in 0..points_per_polygon {
            let angle = (j as f64 / points_per_polygon as f64) * 2.0 * std::f64::consts::PI;
            // Use a smaller range for both longitude and latitude
            let lon = (i as f64 * 0.5) + angle.cos() * 0.5; // Scale down longitude range
            let lat = (i as f64 * 0.3) + angle.sin() * 0.3; // Scale down latitude range
                                                            // Ensure coordinates stay within bounds
            let lon = lon.clamp(MIN_LONGITUDE, MAX_LONGITUDE);
            let lat = lat.clamp(MIN_LATITUDE, MAX_LATITUDE);
            exterior.push(vec![lon, lat]);
        }
        // Close the polygon
        exterior.push(exterior[0].clone());

        let feature = Feature {
            bbox: None,
            geometry: Some(Geometry::new(Value::Polygon(vec![exterior]))),
            id: None,
            properties: None,
            foreign_members: None,
        };
        features.push(feature);
    }

    FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    }
}

fn benchmark_large_geometries(c: &mut Criterion) {
    let mut group = c.benchmark_group("Large Geometry Processing");

    // Benchmark line strings of increasing size
    for size in [1000, 5000, 10000, 50000].iter() {
        let feature_collection = create_large_line_string(*size);
        let geojson = GeoJson::FeatureCollection(feature_collection);
        let json_value = serde_json::to_value(geojson).unwrap();

        group.bench_function(format!("LineString with {} points", size), |b| {
            b.iter(|| {
                let result = process_feature_collection(black_box(json_value.clone()));
                assert!(result.is_ok());
            })
        });
    }

    // Benchmark polygons of increasing size
    for size in [100, 500, 1000, 5000].iter() {
        let feature_collection = create_large_polygon(*size);
        let geojson = GeoJson::FeatureCollection(feature_collection);
        let json_value = serde_json::to_value(geojson).unwrap();

        group.bench_function(format!("Polygon with {} points", size), |b| {
            b.iter(|| {
                let result = process_feature_collection(black_box(json_value.clone()));
                assert!(result.is_ok());
            })
        });
    }

    // Benchmark multi-polygons with increasing number of polygons
    for num_polygons in [10, 50, 100].iter() {
        let feature_collection = create_large_multi_polygon(*num_polygons, 100);
        let geojson = GeoJson::FeatureCollection(feature_collection);
        let json_value = serde_json::to_value(geojson).unwrap();

        group.bench_function(
            format!("MultiPolygon with {} polygons", num_polygons),
            |b| {
                b.iter(|| {
                    let result = process_feature_collection(black_box(json_value.clone()));
                    assert!(result.is_ok());
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_large_geometries);
criterion_main!(benches);
