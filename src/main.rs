pub mod helpers;

use crate::helpers::process_geometry;
use geo::{CoordsIter, LineString, Point, Polygon};
use geojson::Feature;
use serde_json::json;
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

fn main() -> Result<(), ProjectionError> {
    // convert point to projected
    let point_example = process_point_example()?;
    println!("Point example: {:?}", point_example);

    // // convert line string to projected
    let line_string_example = process_line_string_example()?;
    println!("Line string example: {:?}", line_string_example);

    // convert polygon to projected
    let polygon_example = process_polygon_example()?;
    println!("Polygon example: {:?}", polygon_example);

    // convert feature collection to projected
    let feature_collection_example = process_feature_collection_example()?;
    println!(
        "Feature collection example: {:?}",
        feature_collection_example
    );

    Ok(())
}

fn process_line_string_example() -> Result<ProcessedGeometry, ProjectionError> {
    let json_value = json!({
        "type": "Feature",
        "properties": null,
        "geometry": {
            "type": "LineString",
            "coordinates": [
                [13.377, 52.518],   // Approximately near the Reichstag (West)
                [13.379, 52.517],   // Moving slightly east along the river
                [13.381, 52.516]    // Further east
              ]
        }
    });
    let feature = Feature::from_json_value(json_value)?;
    let projected = process_geometry(feature)?;
    Ok(projected)
}

fn process_point_example() -> Result<ProcessedGeometry, ProjectionError> {
    // prepare feature as json
    let json_value = json!({
        "type": "Feature",
        "properties": null,
        "geometry": {
            "type": "Point",
            "coordinates": [13.377, 52.518]
        }
    });
    // parse json into feature
    let feature = Feature::from_json_value(json_value)?;
    // extract geometry and project
    let projected = process_geometry(feature)?;
    Ok(projected)
}

fn process_polygon_example() -> Result<ProcessedGeometry, ProjectionError> {
    let json_value = json!({
        "type": "Feature",
        "properties": {},
        "geometry": {
            "type": "Polygon",
            "coordinates": [ // Added an outer array here
                [
                    [13.350, 52.515],   // Southwest corner (approx.)
                    [13.355, 52.515],   // Southeast corner (approx.)
                    [13.355, 52.510],   // Northeast corner (approx.)
                    [13.350, 52.510],   // Northwest corner (approx.)
                    [13.350, 52.515]    // Closing the polygon
                ]
            ]
        }
    });
    let feature = Feature::from_json_value(json_value)?;
    let projected = process_geometry(feature)?;
    Ok(projected)
}

fn process_feature_collection_example() -> Result<(), ProjectionError> {
    let json_value = json!({
      "type": "FeatureCollection",
      "features": [
        {
          "type": "Feature",
          "properties": null,
          "geometry": {
            "type": "Point",
            "coordinates": [13.377, 52.518]
          }
        },
        {
          "type": "Feature",
          "properties": null,
          "geometry": {
            "type": "LineString",
            "coordinates": [
              [13.377, 52.518],
              [13.379, 52.517],
              [13.381, 52.516]
            ]
          }
        },
        {
          "type": "Feature",
          "properties": {},
          "geometry": {
            "type": "Polygon",
            "coordinates": [
              [
                [13.350, 52.515],
                [13.355, 52.515],
                [13.355, 52.510],
                [13.350, 52.510],
                [13.350, 52.515]
              ]
            ]
          }
        }
      ]
    });
    let geojson_data = geojson::GeoJson::from_json_value(json_value)?;
    let mut transformed_features: Vec<Feature> = Vec::new();

    match geojson_data {
        geojson::GeoJson::FeatureCollection(fc) => {
            println!(
                "Successfully parsed FeatureCollection with {} features",
                fc.features.len()
            );
            for original_feature in fc.features {
                let processed_geometry = process_geometry(original_feature.clone())?;
                let transformed_feature = match processed_geometry {
                    ProcessedGeometry::Point(point) => Feature {
                        bbox: None,
                        geometry: Some(geojson::Geometry::new(geojson::Value::Point(vec![
                            point.x(),
                            point.y(),
                        ]))),
                        id: original_feature.id,
                        properties: original_feature.properties,
                        foreign_members: original_feature.foreign_members,
                    },
                    ProcessedGeometry::LineString(line_string) => Feature {
                        bbox: None,
                        geometry: Some(geojson::Geometry::new(geojson::Value::LineString(
                            line_string
                                .into_iter()
                                .map(|coord| vec![coord.x, coord.y])
                                .collect(),
                        ))),
                        id: original_feature.id,
                        properties: original_feature.properties,
                        foreign_members: original_feature.foreign_members,
                    },
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

                        Feature {
                            bbox: None,
                            geometry: Some(geojson::Geometry::new(geojson::Value::Polygon(rings))),
                            id: original_feature.id,
                            properties: original_feature.properties,
                            foreign_members: original_feature.foreign_members,
                        }
                    }
                };
                transformed_features.push(transformed_feature);
            }
            // Create the new FeatureCollection
            let new_feature_collection = geojson::FeatureCollection {
                bbox: None,
                features: transformed_features,
                foreign_members: None,
            };

            println!(
                "Created new FeatureCollection with {} transformed features",
                new_feature_collection.features.len()
            );
            // You can now serialize this new_feature_collection to JSON if needed
            let json_output = serde_json::to_string_pretty(&new_feature_collection).unwrap();
            println!("{}", json_output);
        }
        _ => todo!(),
    }
    Ok(())
}
