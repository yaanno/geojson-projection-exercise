use geo::{LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon};
/// Simplifies a geometry using the Douglas-Peucker algorithm
pub trait Simplify {
    fn simplify(&self, epsilon: f64) -> Self;
}

pub struct GeoJsonLineString(pub Vec<Vec<f64>>);

impl Simplify for GeoJsonLineString {
    fn simplify(&self, epsilon: f64) -> Self {
        // Convert GeoJSON coordinates to geo::LineString
        let coords: Vec<geo::Coord<f64>> = self
            .0
            .iter()
            .map(|p| geo::coord! { x: p[0], y: p[1] })
            .collect();

        let line_string = LineString::new(coords);

        // Apply simplification
        let simplified = line_string.simplify(epsilon);

        // Convert back to GeoJSON format
        let simplified_coords = simplified.coords().map(|c| vec![c.x, c.y]).collect();

        GeoJsonLineString(simplified_coords)
    }
}

// Convenience conversion traits
impl From<Vec<Vec<f64>>> for GeoJsonLineString {
    fn from(coords: Vec<Vec<f64>>) -> Self {
        GeoJsonLineString(coords)
    }
}

impl From<GeoJsonLineString> for Vec<Vec<f64>> {
    fn from(line_string: GeoJsonLineString) -> Self {
        line_string.0
    }
}

impl Simplify for Point {
    fn simplify(&self, _epsilon: f64) -> Self {
        *self
    }
}

impl Simplify for LineString {
    fn simplify(&self, epsilon: f64) -> Self {
        let mut simplified = Vec::new();
        douglas_peucker(&self.0, epsilon, &mut simplified);
        LineString::from(simplified)
    }
}

impl Simplify for Polygon {
    fn simplify(&self, epsilon: f64) -> Self {
        let mut simplified_exterior = self.exterior().0.clone();
        if simplified_exterior.len() > 2 {
            // Remove the last duplicate point if it exists
            if simplified_exterior.first() == simplified_exterior.last() {
                simplified_exterior.pop();
            }
            let mut result = Vec::new();
            douglas_peucker(&simplified_exterior, epsilon, &mut result);
            // Ensure the polygon is closed
            if result.len() > 1 && result.first() != result.last() {
                result.push(*result.first().unwrap());
            }
            if result.len() < 3 && self.exterior().0.len() >= 3 {
                // If simplification results in fewer than 3 points, return the original
                return self.clone();
            }
            simplified_exterior = result;
        }
        let mut simplified_interiors = Vec::new();
        for interior in self.interiors() {
            let mut simplified_interior = interior.0.clone();
            if simplified_interior.len() > 2 {
                if simplified_interior.first() == simplified_interior.last() {
                    simplified_interior.pop();
                }
                let mut result = Vec::new();
                douglas_peucker(&simplified_interior, epsilon, &mut result);
                if result.len() > 1 && result.first() != result.last() {
                    result.push(*result.first().unwrap());
                }
                if result.len() >= 3 {
                    simplified_interiors.push(LineString::from(result));
                }
            }
        }
        Polygon::new(LineString::from(simplified_exterior), simplified_interiors)
    }
}

impl Simplify for MultiPoint {
    fn simplify(&self, _epsilon: f64) -> Self {
        self.clone()
    }
}

impl Simplify for MultiLineString {
    fn simplify(&self, epsilon: f64) -> Self {
        MultiLineString::new(self.0.iter().map(|line| line.simplify(epsilon)).collect())
    }
}

impl Simplify for MultiPolygon {
    fn simplify(&self, epsilon: f64) -> Self {
        MultiPolygon::new(
            self.0
                .iter()
                .map(|polygon| polygon.simplify(epsilon))
                .collect(),
        )
    }
}

/// Implementation of the Douglas-Peucker algorithm
fn douglas_peucker(points: &[geo::Coord<f64>], epsilon: f64, result: &mut Vec<geo::Coord<f64>>) {
    if points.len() <= 2 {
        result.extend_from_slice(points);
        return;
    }

    // If epsilon is negative or zero, keep all points
    if epsilon <= 0.0 {
        result.extend_from_slice(points);
        return;
    }

    // Find the point with the maximum distance
    let mut max_dist = 0.0;
    let mut max_idx = 0;
    let start = points[0];
    let end = points[points.len() - 1];

    for (i, point) in points.iter().enumerate().skip(1).take(points.len() - 2) {
        let dist = perpendicular_distance(point, &start, &end);
        if dist > max_dist {
            max_dist = dist;
            max_idx = i;
        }
    }

    // If max distance is greater than epsilon, recursively simplify
    if max_dist > epsilon {
        let mut first_part = Vec::new();
        douglas_peucker(&points[..=max_idx], epsilon, &mut first_part);
        first_part.pop(); // Remove duplicate point
        result.extend_from_slice(&first_part);

        let mut second_part = Vec::new();
        douglas_peucker(&points[max_idx..], epsilon, &mut second_part);
        result.extend_from_slice(&second_part);
    } else {
        // For colinear points, keep only start and end points
        result.push(start);
        result.push(end);
    }
}

/// Calculate the perpendicular distance from a point to a line segment
fn perpendicular_distance(
    point: &geo::Coord<f64>,
    line_start: &geo::Coord<f64>,
    line_end: &geo::Coord<f64>,
) -> f64 {
    let x = point.x;
    let y = point.y;
    let x1 = line_start.x;
    let y1 = line_start.y;
    let x2 = line_end.x;
    let y2 = line_end.y;

    let dx = x2 - x1;
    let dy = y2 - y1;

    // Calculate the perpendicular distance
    let numerator = (dy * x - dx * y + x2 * y1 - y2 * x1).abs();
    let denominator = (dx * dx + dy * dy).sqrt();

    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::{coord, point, polygon};

    #[test]
    fn test_point_simplification() {
        let point = Point::new(1.0, 2.0);
        let simplified = point.simplify(0.1);
        assert_eq!(point, simplified);
    }

    #[test]
    fn test_line_string_simplification() {
        // Create a line string with non-colinear points
        let line = LineString::from(vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 1.0, y: 0.1 },
            coord! { x: 2.0, y: 0.0 },
            coord! { x: 3.0, y: 0.1 },
            coord! { x: 4.0, y: 0.0 },
        ]);

        // With epsilon = 0.1, should remove some points
        let simplified = line.simplify(0.1);
        assert_eq!(
            simplified,
            LineString::from(vec![coord! { x: 0.0, y: 0.0 }, coord! { x: 4.0, y: 0.0 }])
        );

        // With larger epsilon, should remove some points
        let simplified = line.simplify(0.2);
        assert!(simplified.0.len() < line.0.len());
        assert!(simplified.0.len() >= 2); // Should keep more than just start and end points
    }

    #[test]
    fn test_polygon_simplification() {
        // Create a polygon with non-colinear points
        let exterior = LineString::from(vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 1.0, y: 0.1 },
            coord! { x: 1.0, y: 1.0 },
            coord! { x: 0.1, y: 1.0 },
            coord! { x: 0.0, y: 0.0 },
        ]);
        let polygon = Polygon::new(exterior, vec![]);

        // With epsilon = 0.1, should keep all points
        let simplified = polygon.simplify(0.1);
        assert_eq!(polygon, simplified);

        // With larger epsilon, should not remove all points
        let simplified = polygon.simplify(0.2);
        assert_eq!(simplified.exterior().0.len(), polygon.exterior().0.len());
        assert!(simplified.exterior().0.len() >= 3); // Should keep at least 3 points for a polygon
    }

    #[test]
    fn test_multi_point_simplification() {
        let points = vec![
            point! { x: 0.0, y: 0.0 },
            point! { x: 1.0, y: 1.0 },
            point! { x: 2.0, y: 2.0 },
        ];
        let multi_point = MultiPoint::from(points);
        let simplified = multi_point.simplify(0.1);
        assert_eq!(multi_point, simplified);
    }

    #[test]
    fn test_multi_line_string_simplification() {
        let line1 = LineString::from(vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 1.0, y: 0.1 },
            coord! { x: 2.0, y: 0.0 },
        ]);
        let line2 = LineString::from(vec![
            coord! { x: 3.0, y: 0.0 },
            coord! { x: 4.0, y: 0.1 },
            coord! { x: 5.0, y: 0.0 },
        ]);
        let multi_line = MultiLineString::new(vec![line1, line2]);

        // With epsilon = 0.1, should remove some points
        let simplified = multi_line.simplify(0.1);
        assert_eq!(
            simplified,
            MultiLineString::new(vec![
                LineString::from(vec![coord! { x: 0.0, y: 0.0 }, coord! { x: 2.0, y: 0.0 }]),
                LineString::from(vec![coord! { x: 3.0, y: 0.0 }, coord! { x: 5.0, y: 0.0 }])
            ])
        );

        // With larger epsilon, should remove some points
        let simplified = multi_line.simplify(0.2);
        assert!(simplified.0.iter().all(|line| line.0.len() >= 2)); // Should keep at least start and end points
    }

    #[test]
    fn test_multi_polygon_simplification() {
        let poly1 = polygon![
            (x: 0.0, y: 0.0),
            (x: 1.0, y: 0.1),
            (x: 1.0, y: 1.0),
            (x: 0.1, y: 1.0),
            (x: 0.0, y: 0.0),
        ];
        let poly2 = polygon![
            (x: 2.0, y: 2.0),
            (x: 3.0, y: 2.1),
            (x: 3.0, y: 3.0),
            (x: 2.1, y: 3.0),
            (x: 2.0, y: 2.0),
        ];
        let multi_poly = MultiPolygon::from(vec![poly1, poly2]);

        // With epsilon = 0.1, should keep all points
        let simplified = multi_poly.simplify(0.1);
        assert_eq!(multi_poly, simplified);

        // With larger epsilon, should not remove all points
        let simplified = multi_poly.simplify(0.2);
        assert!(simplified.0.iter().all(|poly| poly.exterior().0.len() == 5));
        assert!(simplified.0.iter().all(|poly| poly.exterior().0.len() >= 3)); // Should keep at least 3 points for a polygon
    }

    #[test]
    fn test_simplification_with_zero_epsilon() {
        let line = LineString::from(vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 1.0, y: 0.1 },
            coord! { x: 2.0, y: 0.0 },
        ]);
        let simplified = line.simplify(0.0);
        assert_eq!(line, simplified);
    }

    #[test]
    fn test_simplification_with_negative_epsilon() {
        let line = LineString::from(vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 1.0, y: 0.1 },
            coord! { x: 2.0, y: 0.0 },
        ]);
        let simplified = line.simplify(-1.0);
        assert_eq!(line, simplified);
    }

    #[test]
    fn test_geojson_line_string_simplification() {
        let coords = vec![
            vec![0.0, 0.0],
            vec![1.0, 0.1], // This point should be removed with sufficient epsilon
            vec![2.0, 0.0],
            vec![3.0, 0.1], // This point should be removed with sufficient epsilon
            vec![4.0, 0.0],
        ];

        let line_string = GeoJsonLineString(coords);

        // With small epsilon, should keep most points
        let simplified = line_string.simplify(0.05);
        assert!(simplified.0.len() > 2);

        // With larger epsilon, should remove intermediate points
        let simplified = line_string.simplify(0.2);
        assert_eq!(simplified.0, vec![vec![0.0, 0.0], vec![4.0, 0.0],]);
    }

    #[test]
    fn test_geojson_line_string_zero_epsilon() {
        let coords = vec![vec![0.0, 0.0], vec![1.0, 1.0], vec![2.0, 2.0]];

        let line_string = GeoJsonLineString(coords.clone());
        let simplified = line_string.simplify(0.0);

        assert_eq!(simplified.0, coords);
    }
}
