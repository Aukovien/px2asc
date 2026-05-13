#[derive(Debug, Clone)]
pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub rgb_data: Vec<u8>,
    pub luma_data: Vec<f32>,
}
