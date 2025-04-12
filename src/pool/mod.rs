use std::collections::VecDeque;
use std::sync::Mutex;

use crate::coordinates::{Coordinate, Line};

pub struct CoordinateBufferPool {
    pub point_buffers: Mutex<VecDeque<Vec<Coordinate>>>,
    pub line_buffers: Mutex<VecDeque<Vec<Line>>>,
    pub polygon_buffers: Mutex<VecDeque<Vec<Line>>>,
    initial_capacity: usize,
}

impl CoordinateBufferPool {
    /// Create a new buffer pool with a given initial capacity
    ///
    /// # Arguments
    ///
    /// * `initial_capacity` - The initial capacity of the buffer pool
    ///
    /// # Returns
    ///
    /// * `CoordinateBufferPool` - A new buffer pool
    pub fn new(initial_capacity: usize) -> Self {
        Self {
            point_buffers: Mutex::new(VecDeque::new()),
            line_buffers: Mutex::new(VecDeque::new()),
            polygon_buffers: Mutex::new(VecDeque::new()),
            initial_capacity,
        }
    }

    /// Get a buffer for a point
    ///
    /// # Returns
    ///
    /// * `Vec<Coordinate>` - A buffer for a point
    pub fn get_point_buffer(&self) -> Vec<Coordinate> {
        let mut buffers = self.point_buffers.lock().unwrap();
        if let Some(mut buffer) = buffers.pop_front() {
            buffer.clear();
            buffer
        } else {
            Vec::with_capacity(self.initial_capacity)
        }
    }

    /// Return a buffer for a point
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer to return
    pub fn return_point_buffer(&self, mut buffer: Vec<Coordinate>) {
        buffer.clear();
        let mut buffers = self.point_buffers.lock().unwrap();
        buffers.push_back(buffer);
    }

    /// Get a buffer for a line
    ///
    /// # Returns
    ///
    /// * `Vec<Line>` - A buffer for a line
    pub fn get_line_buffer(&self) -> Vec<Line> {
        let mut buffers = self.line_buffers.lock().unwrap();
        if let Some(mut buffer) = buffers.pop_front() {
            buffer.clear();
            buffer
        } else {
            Vec::with_capacity(self.initial_capacity)
        }
    }

    /// Return a buffer for a line
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer to return
    pub fn return_line_buffer(&self, mut buffer: Vec<Line>) {
        buffer.clear();
        let mut buffers = self.line_buffers.lock().unwrap();
        buffers.push_back(buffer);
    }

    /// Get a buffer for a polygon
    ///
    /// # Returns
    ///
    /// * `Vec<Line>` - A buffer for a polygon
    pub fn get_polygon_buffer(&self) -> Vec<Line> {
        let mut buffers = self.polygon_buffers.lock().unwrap();
        if let Some(mut buffer) = buffers.pop_front() {
            buffer.clear();
            buffer
        } else {
            Vec::with_capacity(self.initial_capacity)
        }
    }

    /// Return a buffer for a polygon
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer to return
    pub fn return_polygon_buffer(&self, mut buffer: Vec<Line>) {
        buffer.clear();
        let mut buffers = self.polygon_buffers.lock().unwrap();
        buffers.push_back(buffer);
    }
}
