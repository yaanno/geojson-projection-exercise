use geo::Point;
use proj_exercise_simple::Coordinate;
use std::collections::VecDeque;

#[derive(Debug)]
struct CoordinateBufferPool {
    point_buffers: VecDeque<Vec<Point<f64>>>,
    line_buffers: VecDeque<Vec<Coordinate>>,
}

impl CoordinateBufferPool {
    fn new() -> Self {
        Self {
            point_buffers: VecDeque::new(),
            line_buffers: VecDeque::new(),
        }
    }

    fn get_point_buffer(&mut self, capacity: usize) -> Vec<Point<f64>> {
        if let Some(mut buffer) = self.point_buffers.pop_front() {
            buffer.clear();
            buffer.reserve(capacity);
            buffer
        } else {
            Vec::with_capacity(capacity)
        }
    }

    fn return_point_buffer(&mut self, buffer: Vec<Point<f64>>) {
        self.point_buffers.push_back(buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_pool_creation() {
        let pool = CoordinateBufferPool::new();
        assert!(pool.point_buffers.is_empty());
        assert!(pool.line_buffers.is_empty());
    }

    #[test]
    fn test_get_and_return_buffer() {
        let mut pool = CoordinateBufferPool::new();

        // Get a new buffer
        let mut buffer = pool.get_point_buffer(10);
        assert_eq!(buffer.capacity(), 10);
        assert!(buffer.is_empty());

        // Add some points
        buffer.push(Point::new(1.0, 2.0));
        buffer.push(Point::new(3.0, 4.0));

        // Return the buffer
        pool.return_point_buffer(buffer);

        // Get the buffer again
        let buffer = pool.get_point_buffer(10);
        assert_eq!(buffer.capacity(), 10);
        assert!(buffer.is_empty()); // Should be cleared
    }

    #[test]
    fn test_buffer_reuse() {
        let mut pool = CoordinateBufferPool::new();

        // Get and return multiple buffers
        let buffer1 = pool.get_point_buffer(5);
        pool.return_point_buffer(buffer1);

        let buffer2 = pool.get_point_buffer(5);
        pool.return_point_buffer(buffer2);

        // Should reuse the same buffer
        let buffer3 = pool.get_point_buffer(5);
        assert_eq!(buffer3.capacity(), 5);
    }

    #[test]
    fn test_buffer_capacity_growth() {
        let mut pool = CoordinateBufferPool::new();

        // Get a small buffer
        let buffer1 = pool.get_point_buffer(5);
        pool.return_point_buffer(buffer1);

        // Request a larger buffer
        let buffer2 = pool.get_point_buffer(20);
        assert_eq!(buffer2.capacity(), 20);
    }
}
