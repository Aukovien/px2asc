use serde::{Deserialize, Serialize};

/// How characters are chosen to represent a pixel region.
/// Only add a variant here when its rendering code exists in engine.rs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArtStyle {
    /// Standard: one character per pixel, chosen by luminance.
    Classic,
    /// Unicode Braille: 2×4 pixel cells mapped to U+2800–U+28FF.
    Braille,
    // Halftone, DotCross, Line, Particles, Terminal — add when implemented.
}

/// Error-diffusion / ordered-dither algorithms.
/// Only add a variant when the kernel is coded in engine.rs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DitherMethod {
    None,
    FloydSteinberg,
    Bayer,
    Atkinson,
}

/// How the RGB value of each output character is determined.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ColorMode {
    Grayscale,
    FullColor,
    MatrixGreen,
    AmberMonitor,
    // CustomHex: add when hex-parsing code exists.
}

/// Master configuration passed into the engine for a single render.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsciiSettings {
    pub style: ArtStyle,
    pub dither_method: DitherMethod,
    pub color_mode: ColorMode,

    // Tuning parameters (all applied in the pipeline)
    /// Error-diffusion multiplier. 0.0 = no dithering, 1.0 = full, >1.0 = overdrive.
    pub dither_strength: f32,
    /// Contrast multiplier centred on mid-grey. 1.0 = no change.
    pub contrast: f32,
    /// Additive brightness offset in luma units (-255 … +255).
    pub brightness: f32,
    /// Invert luminance before character selection.
    pub invert: bool,

    // Style-specific options
    /// Character ramp for Classic style, ordered dark→light (e.g. " .:-=+*#%@").
    pub charset: String,
    /// Luminance threshold (0–255) for Braille dot on/off.
    pub braille_threshold: f32,
}

impl Default for AsciiSettings {
    fn default() -> Self {
        Self {
            style: ArtStyle::Classic,
            dither_method: DitherMethod::FloydSteinberg,
            color_mode: ColorMode::Grayscale,
            dither_strength: 0.8,
            contrast: 1.0,
            brightness: 0.0,
            invert: false,
            charset: " .:-=+*#%@".to_string(),
            braille_threshold: 128.0,
        }
    }
}
