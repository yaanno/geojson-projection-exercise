use std::collections::VecDeque;
use std::sync::Mutex;
use thiserror::Error;

use crate::coordinates::{Coordinate, Line};

#[derive(Error, Debug)]
pub enum BufferPoolError {
    #[error("Mutex was poisoned: {0}")]
    MutexPoisoned(String),
    #[error("Buffer pool is full")]
    PoolFull,
    #[error("Projection error: {0}")]
    ProjError(#[from] proj::ProjCreateError),
}

pub struct CoordinateBufferPool {
    pub point_buffers: Mutex<VecDeque<Vec<Coordinate>>>,
    pub line_buffers: Mutex<VecDeque<Vec<Line>>>,
    pub polygon_buffers: Mutex<VecDeque<Vec<Line>>>,
    pub initial_capacity: usize,
    pub max_size: usize,
}

impl CoordinateBufferPool {
    /// Create a new buffer pool with a given initial capacity
    ///
    /// # Arguments
    ///
    /// * `initial_capacity` - The initial capacity of the buffer pool
    /// * `max_size` - The maximum number of buffers allowed in the pool
    ///
    /// # Returns
    ///
    /// * `CoordinateBufferPool` - A new buffer pool
    pub fn new(initial_capacity: usize, max_size: usize) -> Self {
        Self {
            point_buffers: Mutex::new(VecDeque::new()),
            line_buffers: Mutex::new(VecDeque::new()),
            polygon_buffers: Mutex::new(VecDeque::new()),
            initial_capacity,
            max_size,
        }
    }

    /// Get a buffer for a point
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Coordinate>, BufferPoolError>` - A buffer for a point
    pub fn get_point_buffer(&self) -> Result<Vec<Coordinate>, BufferPoolError> {
        let mut buffers = self
            .point_buffers
            .lock()
            .map_err(|e| BufferPoolError::MutexPoisoned(e.to_string()))?;

        if let Some(mut buffer) = buffers.pop_front() {
            buffer.clear();
            Ok(buffer)
        } else {
            Ok(Vec::with_capacity(self.initial_capacity))
        }
    }

    /// Return a buffer for a point
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer to return
    pub fn return_point_buffer(&self, mut buffer: Vec<Coordinate>) -> Result<(), BufferPoolError> {
        let mut buffers = self
            .point_buffers
            .lock()
            .map_err(|e| BufferPoolError::MutexPoisoned(e.to_string()))?;

        if buffers.len() >= self.max_size {
            return Err(BufferPoolError::PoolFull);
        }

        buffer.clear();
        buffers.push_back(buffer);
        Ok(())
    }

    /// Get a buffer for a line
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Line>, BufferPoolError>` - A buffer for a line
    pub fn get_line_buffer(&self) -> Result<Vec<Line>, BufferPoolError> {
        let mut buffers = self
            .line_buffers
            .lock()
            .map_err(|e| BufferPoolError::MutexPoisoned(e.to_string()))?;

        if let Some(mut buffer) = buffers.pop_front() {
            buffer.clear();
            Ok(buffer)
        } else {
            Ok(Vec::with_capacity(self.initial_capacity))
        }
    }

    /// Return a buffer for a line
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer to return
    pub fn return_line_buffer(&self, mut buffer: Vec<Line>) -> Result<(), BufferPoolError> {
        let mut buffers = self
            .line_buffers
            .lock()
            .map_err(|e| BufferPoolError::MutexPoisoned(e.to_string()))?;

        if buffers.len() >= self.max_size {
            return Err(BufferPoolError::PoolFull);
        }

        buffer.clear();
        buffers.push_back(buffer);
        Ok(())
    }

    /// Get a buffer for a polygon
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Line>, BufferPoolError>` - A buffer for a polygon
    pub fn get_polygon_buffer(&self) -> Result<Vec<Line>, BufferPoolError> {
        let mut buffers = self
            .polygon_buffers
            .lock()
            .map_err(|e| BufferPoolError::MutexPoisoned(e.to_string()))?;

        if let Some(mut buffer) = buffers.pop_front() {
            buffer.clear();
            Ok(buffer)
        } else {
            Ok(Vec::with_capacity(self.initial_capacity))
        }
    }

    /// Return a buffer for a polygon
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer to return
    pub fn return_polygon_buffer(&self, mut buffer: Vec<Line>) -> Result<(), BufferPoolError> {
        let mut buffers = self
            .polygon_buffers
            .lock()
            .map_err(|e| BufferPoolError::MutexPoisoned(e.to_string()))?;

        if buffers.len() >= self.max_size {
            return Err(BufferPoolError::PoolFull);
        }

        buffer.clear();
        buffers.push_back(buffer);
        Ok(())
    }

    /// Clear all buffers in the pool
    pub fn clear(&self) -> Result<(), BufferPoolError> {
        let mut point_buffers = self
            .point_buffers
            .lock()
            .map_err(|e| BufferPoolError::MutexPoisoned(e.to_string()))?;
        let mut line_buffers = self
            .line_buffers
            .lock()
            .map_err(|e| BufferPoolError::MutexPoisoned(e.to_string()))?;
        let mut polygon_buffers = self
            .polygon_buffers
            .lock()
            .map_err(|e| BufferPoolError::MutexPoisoned(e.to_string()))?;

        point_buffers.clear();
        line_buffers.clear();
        polygon_buffers.clear();

        Ok(())
    }

    /// Get statistics about the buffer pool
    pub fn stats(&self) -> Result<BufferPoolStats, BufferPoolError> {
        let point_buffers = self
            .point_buffers
            .lock()
            .map_err(|e| BufferPoolError::MutexPoisoned(e.to_string()))?;
        let line_buffers = self
            .line_buffers
            .lock()
            .map_err(|e| BufferPoolError::MutexPoisoned(e.to_string()))?;
        let polygon_buffers = self
            .polygon_buffers
            .lock()
            .map_err(|e| BufferPoolError::MutexPoisoned(e.to_string()))?;

        Ok(BufferPoolStats {
            point_buffers: point_buffers.len(),
            line_buffers: line_buffers.len(),
            polygon_buffers: polygon_buffers.len(),
            max_size: self.max_size,
        })
    }
}

#[derive(Debug)]
pub struct BufferPoolStats {
    pub point_buffers: usize,
    pub line_buffers: usize,
    pub polygon_buffers: usize,
    pub max_size: usize,
}
