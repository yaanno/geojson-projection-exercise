use proj_exercise_simple::pool::CoordinateBufferPool;
#[cfg(test)]
mod tests {

    use geo::Point;

    use super::*;

    #[test]
    fn test_buffer_pool_creation() {
        let pool = CoordinateBufferPool::new(10);
        assert!(pool.point_buffers.lock().unwrap().is_empty());
        assert!(pool.line_buffers.lock().unwrap().is_empty());
    }

    #[test]
    fn test_get_and_return_buffer() {
        let pool = CoordinateBufferPool::new(10);

        // Get a new buffer
        let mut buffer = pool.get_point_buffer();
        assert_eq!(buffer.capacity(), 10);
        assert!(buffer.is_empty());

        // Add some points
        buffer.push(Point::new(1.0, 1.0).into());
        buffer.push(Point::new(1.0, 1.0).into());
        buffer.push(Point::new(1.0, 1.0).into());
        buffer.push(Point::new(1.0, 1.0).into());

        // Return the buffer
        pool.return_point_buffer(buffer);

        // Get the buffer again
        let buffer = pool.get_point_buffer();
        assert_eq!(buffer.capacity(), 10);
        assert!(buffer.is_empty()); // Should be cleared
    }

    #[test]
    fn test_buffer_reuse() {
        let pool = CoordinateBufferPool::new(10);

        // Get and return multiple buffers
        let buffer1 = pool.get_point_buffer();
        pool.return_point_buffer(buffer1);

        let buffer2 = pool.get_point_buffer();
        pool.return_point_buffer(buffer2);

        // Should reuse the same buffer
        let buffer3 = pool.get_point_buffer();
        assert_eq!(buffer3.capacity(), 10);
    }

    #[test]
    fn test_buffer_capacity_growth() {
        let pool = CoordinateBufferPool::new(10);

        // Get a small buffer
        let buffer1 = pool.get_point_buffer();
        pool.return_point_buffer(buffer1);

        // Request a larger buffer
        let buffer2 = pool.get_point_buffer();
        assert_eq!(buffer2.capacity(), 10); // Capacity should remain the same
    }
}
