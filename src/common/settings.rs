use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorMode {
    Grayscale,
    Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtStyle {
    Classic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsciiSettings {
    pub brightness: f32,
    pub contrast: f32,
    pub color_mode: ColorMode,
    pub art_style: ArtStyle,
    pub font_size: f32,
    pub char_spacing: f32,
}

impl Default for AsciiSettings {
    fn default() -> Self {
        Self {
            brightness: 0.0,
            contrast: 1.0,
            color_mode: ColorMode::Grayscale,
            art_style: ArtStyle::Classic,
            font_size: 10.0,
            char_spacing: 1.0,
        }
    }
}
