pub mod helpers;

use crate::helpers::process_feature;
use geo::{LineString, Point, Polygon};
use geojson::Feature;
use helpers::process_feature_collection;
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
    let feature_collection_example = process_featurecollection_example()?;
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
    let projected = process_feature(feature)?;
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
    let projected = process_feature(feature)?;
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
    let projected = process_feature(feature)?;
    Ok(projected)
}

fn process_featurecollection_example() -> Result<Vec<ProcessedGeometry>, ProjectionError> {
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

    match geojson_data {
        geojson::GeoJson::FeatureCollection(fc) => {
            println!(
                "Successfully parsed FeatureCollection with {} features",
                fc.features.len()
            );
            let processed_features = process_feature_collection(fc)?;
            Ok(processed_features)
        }
        _ => Err(ProjectionError::InvalidGeometryType),
    }
}
