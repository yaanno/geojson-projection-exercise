pub mod helpers;

use crate::helpers::process_feature_collection;
use helpers::ProjectionError;
use serde_json::json;

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
    let point_example = process_feature_collection(json_value)?;
    println!("--- Point example: {:?}", point_example);

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
    let line_string_example = process_feature_collection(json_value)?;
    println!("--- Line string example: {:?}", line_string_example);

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
    let polygon_example = process_feature_collection(json_value)?;
    println!("--- Polygon example: {:?}", polygon_example);

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
    let feature_example = process_feature_collection(json_value)?;
    println!("--- Feature example: {:?}", feature_example);

    // convert geometry to projected
    let json_value = json!({
            "type": "LineString",
            "coordinates": [
                [13.377, 52.518],   // Approximately near the Reichstag (West)
                [13.379, 52.517],   // Moving slightly east along the river
                [13.381, 52.516]    // Further east
              ]
    });
    let geometry_example = process_feature_collection(json_value)?;
    println!("--- Geometry example: {:?}", geometry_example);

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
    let feature_collection_example = process_feature_collection(json_value)?;
    println!(
        "--- Feature collection example: {:?}",
        feature_collection_example
    );
    Ok(())
}
