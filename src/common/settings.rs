// src/common/settings.rs
//
// config & settings types

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColorMode {
    Grayscale,
    Color,
    GrayscaleWithAccent,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ArtStyle {
    Classic,
    Braille,
    BlockArt,
    Hatching,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsciiSettings {
    pub color_mode: ColorMode,
    pub art_style: ArtStyle,
    pub brightness: f32,
    pub contrast: f32,
}

impl Default for AsciiSettings {
    fn default() -> Self {
        Self {
            color_mode: ColorMode::Grayscale,
            art_style: ArtStyle::Classic,
            brightness: 0.0,
            contrast: 1.0,
        }
    }
}
