pub mod coordinates;
pub mod helpers;

use crate::coordinates::{Coordinate, Line, Polygon};
use crate::helpers::process_feature_collection;
use helpers::ProjectionError;
use serde_json::json;

fn main() -> Result<(), ProjectionError> {
    // convert point to projected
    let point = Coordinate::new(13.377, 52.518);
    let json_value = json!({
        "type": "Feature",
        "properties": null,
        "geometry": {
            "type": "Point",
            "coordinates": [point.x, point.y]
        }
    });
    let point_example = process_feature_collection(json_value)?;
    println!("--- Point example: {:?}", point_example);

    // convert line string to projected
    let line = Line::new(vec![
        Coordinate::new(13.377, 52.518), // Approximately near the Reichstag (West)
        Coordinate::new(13.379, 52.517), // Moving slightly east along the river
        Coordinate::new(13.381, 52.516), // Further east
    ]);
    let json_value = json!({
        "type": "Feature",
        "properties": null,
        "geometry": {
            "type": "LineString",
            "coordinates": line.coordinates.iter().map(|c| vec![c.x, c.y]).collect::<Vec<_>>()
        }
    });
    let line_string_example = process_feature_collection(json_value)?;
    println!("--- Line string example: {:?}", line_string_example);

    // convert polygon to projected
    let exterior = Line::new(vec![
        Coordinate::new(13.350, 52.515), // Southwest corner (approx.)
        Coordinate::new(13.355, 52.515), // Southeast corner (approx.)
        Coordinate::new(13.355, 52.510), // Northeast corner (approx.)
        Coordinate::new(13.350, 52.510), // Northwest corner (approx.)
        Coordinate::new(13.350, 52.515), // Closing the polygon
    ]);
    let polygon = Polygon::new(exterior, vec![]);
    let json_value = json!({
        "type": "Feature",
        "properties": {},
        "geometry": {
            "type": "Polygon",
            "coordinates": [
                polygon.exterior.coordinates.iter().map(|c| vec![c.x, c.y]).collect::<Vec<_>>()
            ]
        }
    });
    let polygon_example = process_feature_collection(json_value)?;
    println!("--- Polygon example: {:?}", polygon_example);

    // convert feature collection to projected
    let json_value = json!({
      "type": "FeatureCollection",
      "features": [
        {
          "type": "Feature",
          "properties": null,
          "geometry": {
            "type": "Point",
            "coordinates": [point.x, point.y]
          }
        },
        {
          "type": "Feature",
          "properties": null,
          "geometry": {
            "type": "LineString",
            "coordinates": line.coordinates.iter().map(|c| vec![c.x, c.y]).collect::<Vec<_>>()
          }
        },
        {
          "type": "Feature",
          "properties": {},
          "geometry": {
            "type": "Polygon",
            "coordinates": [
              polygon.exterior.coordinates.iter().map(|c| vec![c.x, c.y]).collect::<Vec<_>>()
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
