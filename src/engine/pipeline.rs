// src/engine/pipeline.rs

pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![0; width * height * 4],
        }
    }
}
