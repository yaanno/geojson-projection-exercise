pub mod helpers;

use crate::helpers::process_feature_geometry;
use geo::{CoordsIter, LineString, Point, Polygon};
use geojson::Feature;
use helpers::{TransformerConfig, process_geometry};
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

impl ProcessedGeometry {
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

fn main() -> Result<(), ProjectionError> {
    // convert point to projected

    let json_value = json!({
        "type": "Feature",
        "properties": null,
        "geometry": {
            "type": "Point",
            "coordinates": [13.377, 52.518]
        }
    });
    let point_example = process_feature_collection_example(json_value)?;
    println!("Point example: {:?}", point_example);

    // // convert line string to projected
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
    let line_string_example = process_feature_collection_example(json_value)?;
    println!("Line string example: {:?}", line_string_example);

    // convert polygon to projected
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
    let polygon_example = process_feature_collection_example(json_value)?;
    println!("Polygon example: {:?}", polygon_example);

    // // convert feature to projected
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
    let feature_example = process_feature_collection_example(json_value)?;
    println!("Feature example: {:?}", feature_example);

    // convert geometry to projected
    let json_value = json!({
            "type": "LineString",
            "coordinates": [
                [13.377, 52.518],   // Approximately near the Reichstag (West)
                [13.379, 52.517],   // Moving slightly east along the river
                [13.381, 52.516]    // Further east
              ]
    });
    let geometry_example = process_feature_collection_example(json_value)?;
    println!("Geometry example: {:?}", geometry_example);

    // convert feature collection to projected
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
    let feature_collection_example = process_feature_collection_example(json_value)?;
    println!(
        "Feature collection example: {:?}",
        feature_collection_example
    );
    println!("Success");
    Ok(())
}

fn process_feature_collection_example(
    json_value: serde_json::Value,
) -> Result<geojson::GeoJson, ProjectionError> {
    let geojson_data = geojson::GeoJson::from_json_value(json_value)?;
    let mut transformed_features: Vec<Feature> = Vec::new();
    let config = TransformerConfig::new("EPSG:4326".to_string(), "EPSG:25832".to_string(), false);

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
