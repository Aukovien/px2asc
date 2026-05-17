#[derive(Debug, Clone)]
pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub rgb_data: Vec<u8>,
    pub luma_data: Vec<f32>,
}

impl FrameBuffer {
    pub fn from_rgba(width: usize, height: usize, rgba_data: Vec<u8>) -> Self {
        assert_eq!(
            rgba_data.len(),
            width * height * 4,
            "RGBA data length mismatch: expected {} bytes, got {}",
            width * height * 4,
            rgba_data.len()
        );

        let pixel_count = width * height;
        let mut luma_data = vec![0.0f32; pixel_count];

        for (i, chunk) in rgba_data.chunks_exact(4).enumerate() {
            let r = chunk[0] as f32;
            let g = chunk[1] as f32;
            let b = chunk[2] as f32;

            luma_data[i] = 0.299 * r + 0.587 * g + 0.114 * b;
        }

        Self {
            width,
            height,
            rgb_data: rgba_data,
            luma_data,
        }
    }
}
