use geo::Point;
use std::collections::VecDeque;

pub struct TestBufferPool {
    point_buffers: VecDeque<Vec<Point<f64>>>,
}

impl TestBufferPool {
    pub fn new() -> Self {
        Self {
            point_buffers: VecDeque::new(),
        }
    }

    pub fn get_buffer_count(&self) -> usize {
        self.point_buffers.len()
    }

    pub fn create_test_points(count: usize) -> Vec<Point<f64>> {
        (0..count)
            .map(|i| Point::new(i as f64, (i * 2) as f64))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_points() {
        let points = TestBufferPool::create_test_points(3);
        assert_eq!(points.len(), 3);
        assert_eq!(points[0], Point::new(0.0, 0.0));
        assert_eq!(points[1], Point::new(1.0, 2.0));
        assert_eq!(points[2], Point::new(2.0, 4.0));
    }
}
