use geo::{LineString, Point};

pub struct GeometryFixtures;

impl GeometryFixtures {
    pub fn simple_point() -> Point<f64> {
        Point::new(1.0, 2.0)
    }

    pub fn simple_line_string() -> LineString<f64> {
        LineString::from(vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(2.0, 2.0),
        ])
    }

    pub fn large_line_string(size: usize) -> LineString<f64> {
        let points: Vec<Point<f64>> = (0..size)
            .map(|i| Point::new(i as f64, (i * 2) as f64))
            .collect();
        LineString::from(points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_point() {
        let point = GeometryFixtures::simple_point();
        assert_eq!(point.x(), 1.0);
        assert_eq!(point.y(), 2.0);
    }

    #[test]
    fn test_simple_line_string() {
        let line_string = GeometryFixtures::simple_line_string();
        assert_eq!(line_string.points().count(), 3);
    }

    #[test]
    fn test_large_line_string() {
        let line_string = GeometryFixtures::large_line_string(100);
        assert_eq!(line_string.points().count(), 100);
    }
}
