pub mod coordinates;
pub mod helpers;

use crate::coordinates::{Coordinate, Line, Polygon};
use crate::helpers::process_feature_collection;
use helpers::ProjectionError;
use serde_json::json;

fn coordinate_examples() {
    println!("\n--- Coordinate Examples ---");

    // 1. Creating coordinates in different ways
    let our_coord = Coordinate::new(13.377, 52.518);
    let geo_coord = geo::Coord {
        x: 13.377,
        y: 52.518,
    };
    println!("Our Coordinate: {:?}", our_coord);
    println!("geo::Coord: {:?}", geo_coord);

    // 2. Converting between types
    let geo_point = geo::Point::new(13.377, 52.518);
    let our_coord_from_point = Coordinate::from(geo_point);
    let geo_point_from_our = geo::Point::from(our_coord);
    println!("Our Coordinate from geo::Point: {:?}", our_coord_from_point);
    println!("geo::Point from our Coordinate: {:?}", geo_point_from_our);

    // 3. Working with collections
    let coords = vec![
        Coordinate::new(13.377, 52.518),
        Coordinate::new(13.379, 52.517),
        Coordinate::new(13.381, 52.516),
    ];

    // Convert to geo::Point collection
    let geo_points = Coordinate::to_points(&coords);
    println!("Converted to geo::Points: {:?}", geo_points);

    // Convert to GeoJSON format
    let geojson_coords = Coordinate::to_vecs(&coords);
    println!("Converted to GeoJSON format: {:?}", geojson_coords);

    // 4. Creating geometries
    let line = Line::new(coords.clone());
    let geo_line_string = line.to_geo();
    println!(
        "Our Line converted to geo::LineString: {:?}",
        geo_line_string
    );

    // 5. Working with geo::Coord in geo types
    let geo_line = geo::Line::new(
        geo::Coord {
            x: 13.377,
            y: 52.518,
        },
        geo::Coord {
            x: 13.379,
            y: 52.517,
        },
    );
    println!("geo::Line using geo::Coord: {:?}", geo_line);

    // 6. Converting geo::Coord to our Coordinate
    let our_coord_from_geo = Coordinate::new(geo_line.start.x, geo_line.start.y);
    println!("Our Coordinate from geo::Coord: {:?}", our_coord_from_geo);
}

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

    // Add coordinate examples
    coordinate_examples();

    Ok(())
}
