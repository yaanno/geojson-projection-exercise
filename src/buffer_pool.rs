use std::collections::VecDeque;
use std::sync::Mutex;

pub struct CoordinateBufferPool {
    point_buffers: Mutex<VecDeque<Vec<f64>>>,
    line_buffers: Mutex<VecDeque<Vec<Vec<f64>>>>,
    polygon_buffers: Mutex<VecDeque<Vec<Vec<f64>>>>,
    initial_capacity: usize,
}

impl CoordinateBufferPool {
    pub fn new(initial_capacity: usize) -> Self {
        Self {
            point_buffers: Mutex::new(VecDeque::new()),
            line_buffers: Mutex::new(VecDeque::new()),
            polygon_buffers: Mutex::new(VecDeque::new()),
            initial_capacity,
        }
    }

    pub fn get_point_buffer(&self) -> Vec<f64> {
        let mut buffers = self.point_buffers.lock().unwrap();
        buffers
            .pop_front()
            .unwrap_or_else(|| vec![0.0; self.initial_capacity])
    }

    pub fn return_point_buffer(&self, mut buffer: Vec<f64>) {
        buffer.clear();
        let mut buffers = self.point_buffers.lock().unwrap();
        buffers.push_back(buffer);
    }

    pub fn get_line_buffer(&self) -> Vec<Vec<f64>> {
        let mut buffers = self.line_buffers.lock().unwrap();
        buffers
            .pop_front()
            .unwrap_or_else(|| vec![vec![0.0; 2]; self.initial_capacity])
    }

    pub fn return_line_buffer(&self, mut buffer: Vec<Vec<f64>>) {
        buffer.clear();
        let mut buffers = self.line_buffers.lock().unwrap();
        buffers.push_back(buffer);
    }

    pub fn get_polygon_buffer(&self) -> Vec<Vec<f64>> {
        let mut buffers = self.polygon_buffers.lock().unwrap();
        buffers
            .pop_front()
            .unwrap_or_else(|| vec![vec![0.0; 2]; self.initial_capacity])
    }

    pub fn return_polygon_buffer(&self, mut buffer: Vec<Vec<f64>>) {
        buffer.clear();
        let mut buffers = self.polygon_buffers.lock().unwrap();
        buffers.push_back(buffer);
    }
}
