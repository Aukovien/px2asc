use crate::settings::{ArtStyle, AsciiSettings, ColorMode, DitherMethod};

// Output type

/// A single unit of output: a character and the RGB colour to paint it.
#[derive(Debug, Clone, Copy)]
pub struct ColoredChar {
    pub glyph: char,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

// Frame buffer

/// Working state for a single frame.
pub struct FrameBuffer {
    pub width: u32,
    pub height: u32,
    /// Floating-point luminance buffer.  All pipeline math happens here.
    pub luma: Vec<f32>,
    /// Original linearised RGB, stored for FullColor recovery.
    pub rgb: Vec<u8>,
}

impl FrameBuffer {
    // Construction

    /// Ingest raw RGBA bytes and build the working buffers.
    pub fn from_rgba(rgba: &[u8], width: u32, height: u32) -> Self {
        let n = (width * height) as usize;
        let mut luma = Vec::with_capacity(n);
        let mut rgb = Vec::with_capacity(n * 3);

        for chunk in rgba.chunks_exact(4) {
            let (r, g, b) = (chunk[0] as f32, chunk[1] as f32, chunk[2] as f32);
            rgb.push(chunk[0]);
            rgb.push(chunk[1]);
            rgb.push(chunk[2]);
            // ITU-R BT.601 perceptual luminance
            luma.push(0.299 * r + 0.587 * g + 0.114 * b);
        }

        Self {
            width,
            height,
            luma,
            rgb,
        }
    }

    // Pipeline steps

    /// Apply contrast and brightness to the luma buffer.
    ///
    /// contrast: multiplier centred on 128 (1.0 = no change)
    /// brightness: additive offset in luma units
    pub fn apply_levels(&mut self, contrast: f32, brightness: f32) {
        if (contrast - 1.0).abs() < f32::EPSILON && brightness.abs() < f32::EPSILON {
            return; // fast path: nothing to do
        }
        for v in self.luma.iter_mut() {
            *v = ((*v - 128.0) * contrast + 128.0 + brightness).clamp(0.0, 255.0);
        }
    }

    /// Apply luminance inversion in-place.
    pub fn apply_invert(&mut self) {
        for v in self.luma.iter_mut() {
            *v = 255.0 - *v;
        }
    }

    /// Error-diffusion dithering.  Quantises luma to the nearest
    /// available "ASCII shade" and spreads the rounding error to neighbours.
    pub fn apply_dithering(&mut self, method: DitherMethod, strength: f32, charset_len: usize) {
        if method == DitherMethod::None || strength <= 0.0 || charset_len < 2 {
            return;
        }

        let w = self.width as usize;
        let h = self.height as usize;
        let step = 255.0 / (charset_len - 1) as f32;

        match method {
            DitherMethod::None => {}

            DitherMethod::FloydSteinberg => {
                for y in 0..h {
                    for x in 0..w {
                        let idx = y * w + x;
                        let old = self.luma[idx];
                        let new = (old / step).round() * step;
                        let error = (old - new) * strength;

                        self.luma[idx] = new.clamp(0.0, 255.0);

                        // Distribute error to right neighbour
                        if x + 1 < w {
                            self.luma[idx + 1] += error * (7.0 / 16.0);
                        }
                        // Bottom-left
                        if x > 0 && y + 1 < h {
                            self.luma[idx + w - 1] += error * (3.0 / 16.0);
                        }
                        // Below
                        if y + 1 < h {
                            self.luma[idx + w] += error * (5.0 / 16.0);
                        }
                        // Bottom-right
                        if x + 1 < w && y + 1 < h {
                            self.luma[idx + w + 1] += error * (1.0 / 16.0);
                        }
                    }
                }
            }

            DitherMethod::Bayer => {
                // 4x4 Bayer ordered dithering matrix (normalized to 0-15)
                const BAYER_MATRIX: [[f32; 4]; 4] = [
                    [0.0, 8.0, 2.0, 10.0],
                    [12.0, 4.0, 14.0, 6.0],
                    [3.0, 11.0, 1.0, 9.0],
                    [15.0, 7.0, 13.0, 5.0],
                ];

                for y in 0..h {
                    for x in 0..w {
                        let idx = y * w + x;
                        let old = self.luma[idx];

                        // Get threshold from matrix position
                        let threshold = (BAYER_MATRIX[y % 4][x % 4] / 16.0 - 0.5) * strength * step;
                        let new = ((old + threshold) / step).round() * step;

                        self.luma[idx] = new.clamp(0.0, 255.0);
                    }
                }
            }

            DitherMethod::Atkinson => {
                // Atkinson dithering: distributes 6/8 of error (loses 2/8)
                // Pattern:
                //       X  1/8 1/8
                //   1/8 1/8 1/8
                //       1/8
                for y in 0..h {
                    for x in 0..w {
                        let idx = y * w + x;
                        let old = self.luma[idx];
                        let new = (old / step).round() * step;
                        let error = (old - new) * strength;

                        self.luma[idx] = new.clamp(0.0, 255.0);

                        // Right
                        if x + 1 < w {
                            self.luma[idx + 1] += error * (1.0 / 8.0);
                        }
                        // Right +2
                        if x + 2 < w {
                            self.luma[idx + 2] += error * (1.0 / 8.0);
                        }
                        // Below-left
                        if x > 0 && y + 1 < h {
                            self.luma[idx + w - 1] += error * (1.0 / 8.0);
                        }
                        // Below
                        if y + 1 < h {
                            self.luma[idx + w] += error * (1.0 / 8.0);
                        }
                        // Below-right
                        if x + 1 < w && y + 1 < h {
                            self.luma[idx + w + 1] += error * (1.0 / 8.0);
                        }
                        // Below +2 rows
                        if y + 2 < h {
                            self.luma[idx + w + w] += error * (1.0 / 8.0);
                        }
                    }
                }
            }
        }
    }

    // Render

    /// Convert the processed luma buffer into the output grid.
    /// Routes to the appropriate renderer based on `settings.style`.
    pub fn render(&self, settings: &AsciiSettings) -> RenderOutput {
        match settings.style {
            ArtStyle::Classic => {
                let pixels = self.render_classic(settings);
                RenderOutput::Classic(pixels)
            }
            ArtStyle::Braille => {
                let s = self.render_braille(settings.braille_threshold);
                RenderOutput::Braille(s)
            }
        }
    }

    fn render_classic(&self, settings: &AsciiSettings) -> Vec<ColoredChar> {
        let w = self.width as usize;
        let h = self.height as usize;
        let chars: Vec<char> = settings.charset.chars().collect();
        assert!(!chars.is_empty(), "charset must not be empty");
        let max_idx = (chars.len() - 1) as f32;
        let mut out = Vec::with_capacity(w * h);

        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                let luma = self.luma[idx];

                let char_idx = ((luma / 255.0).clamp(0.0, 1.0) * max_idx).round() as usize;
                let glyph = chars[char_idx];

                // Color is computed from the post-pipeline luma.
                // For FullColor we use the ORIGINAL rgb to recover hue, then
                // scale brightness by the dithered luma ratio.
                let (r, g, b) = self.color_for(idx, luma, settings.color_mode);
                out.push(ColoredChar { glyph, r, g, b });
            }
        }
        out
    }

    fn render_braille(&self, threshold: f32) -> String {
        let w = self.width as usize;
        let h = self.height as usize;
        let out_w = w / 2;
        let out_h = h / 4;
        let mut output = String::with_capacity(out_w * out_h + out_h);

        // Braille Unicode dot-weight table (standard Unicode encoding):
        // Column 0 (left):  dots 1,2,3,7 -> weights 1, 2, 4, 64
        // Column 1 (right): dots 4,5,6,8 -> weights 8,16,32,128
        const DOT_MAP: [(usize, usize, u32); 8] = [
            (0, 0, 1),
            (0, 1, 2),
            (0, 2, 4),
            (0, 3, 64),
            (1, 0, 8),
            (1, 1, 16),
            (1, 2, 32),
            (1, 3, 128),
        ];

        for by in 0..out_h {
            for bx in 0..out_w {
                let mut bits: u32 = 0;
                for &(dx, dy, weight) in &DOT_MAP {
                    let px = bx * 2 + dx;
                    let py = by * 4 + dy;
                    if px < w && py < h && self.luma[py * w + px] > threshold {
                        bits += weight;
                    }
                }
                output.push(char::from_u32(0x2800 + bits).expect("Braille codepoint always valid"));
            }
            output.push('\n');
        }
        output
    }

    /// Determine the output RGB for one pixel given the post-pipeline luma.
    #[inline(always)]
    fn color_for(&self, idx: usize, luma: f32, mode: ColorMode) -> (u8, u8, u8) {
        let v = luma as u8;
        match mode {
            ColorMode::Grayscale => (v, v, v),

            ColorMode::FullColor => {
                // Recover original hue, scale brightness by dither ratio.
                // We use the original luma (from the stored rgb) as the
                // denominator so that dithering only affects brightness,
                // not hue.
                let ro = self.rgb[idx * 3] as f32;
                let go = self.rgb[idx * 3 + 1] as f32;
                let bo = self.rgb[idx * 3 + 2] as f32;
                let orig_luma = 0.299 * ro + 0.587 * go + 0.114 * bo;

                if orig_luma > 0.0 {
                    let ratio = luma / orig_luma;
                    (
                        (ro * ratio).clamp(0.0, 255.0) as u8,
                        (go * ratio).clamp(0.0, 255.0) as u8,
                        (bo * ratio).clamp(0.0, 255.0) as u8,
                    )
                } else {
                    (0, 0, 0)
                }
            }

            ColorMode::MatrixGreen => ((luma * 0.2) as u8, v, (luma * 0.3) as u8),

            ColorMode::AmberMonitor => (v, (luma * 0.72) as u8, (luma * 0.16) as u8),
        }
    }
}

//
// Render output
//

/// Tagged union so the caller knows what it received.
pub enum RenderOutput {
    Classic(Vec<ColoredChar>),
    Braille(String),
}

//
// Convenience: run the full pipeline in one call
//

/// Apply all pipeline stages and render.
/// Order: levels -> invert -> dither -> render.
pub fn process(buf: &mut FrameBuffer, settings: &AsciiSettings) -> RenderOutput {
    buf.apply_levels(settings.contrast, settings.brightness);
    if settings.invert {
        buf.apply_invert();
    }
    buf.apply_dithering(
        settings.dither_method,
        settings.dither_strength,
        settings.charset.chars().count(),
    );
    buf.render(settings)
}

//
// Tests
//

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::AsciiSettings;

    // Helper: build a minimal FrameBuffer from a &[(r,g,b,a)] slice
    fn buf_from_pixels(pixels: &[(u8, u8, u8, u8)], width: u32, height: u32) -> FrameBuffer {
        let raw: Vec<u8> = pixels
            .iter()
            .flat_map(|(r, g, b, a)| [*r, *g, *b, *a])
            .collect();
        FrameBuffer::from_rgba(&raw, width, height)
    }

    // ── Luminance ─────────────

    #[test]
    fn test_luminance_pure_white() {
        let buf = buf_from_pixels(&[(255, 255, 255, 255)], 1, 1);
        assert_eq!(buf.luma[0], 255.0);
    }

    #[test]
    fn test_luminance_pure_black() {
        let buf = buf_from_pixels(&[(0, 0, 0, 255)], 1, 1);
        assert_eq!(buf.luma[0], 0.0);
    }

    #[test]
    fn test_luminance_pure_red() {
        let buf = buf_from_pixels(&[(255, 0, 0, 255)], 1, 1);
        let expected = 0.299 * 255.0_f32;
        assert!(
            (buf.luma[0] - expected).abs() < 0.01,
            "red luma = {}, expected ≈ {}",
            buf.luma[0],
            expected
        );
    }

    // ── Levels ────────────────

    #[test]
    fn test_brightness_clamps_to_255() {
        let mut buf = buf_from_pixels(&[(200, 200, 200, 255)], 1, 1);
        buf.apply_levels(1.0, 200.0); // would overflow without clamp
        assert_eq!(buf.luma[0], 255.0);
    }

    #[test]
    fn test_contrast_doubles_deviation() {
        let mut buf = buf_from_pixels(&[(178, 178, 178, 255)], 1, 1);
        // luma ≈ 178; contrast = 2.0 means (178 - 128) * 2 + 128 = 228
        buf.apply_levels(2.0, 0.0);
        let expected = ((178.0 - 128.0) * 2.0 + 128.0) as u8;
        assert!(
            (buf.luma[0] - expected as f32).abs() < 1.0,
            "luma = {}, expected ≈ {}",
            buf.luma[0],
            expected
        );
    }

    #[test]
    fn test_levels_noop_at_defaults() {
        let mut buf = buf_from_pixels(&[(100, 150, 200, 255)], 1, 1);
        let before = buf.luma[0];
        buf.apply_levels(1.0, 0.0);
        assert_eq!(buf.luma[0], before);
    }

    // ── Invert ────────────────

    #[test]
    fn test_invert_white_to_black() {
        let mut buf = buf_from_pixels(&[(255, 255, 255, 255)], 1, 1);
        buf.apply_invert();
        assert_eq!(buf.luma[0], 0.0);
    }

    #[test]
    fn test_invert_double_is_identity() {
        let mut buf = buf_from_pixels(&[(80, 120, 200, 255)], 1, 1);
        let before = buf.luma[0];
        buf.apply_invert();
        buf.apply_invert();
        assert!((buf.luma[0] - before).abs() < 0.001);
    }

    // ── Dithering ─────────────

    #[test]
    fn test_floyd_steinberg_differentiates_uniform_field() {
        let pixels: Vec<(u8, u8, u8, u8)> = vec![(128, 128, 128, 255); 9];
        let mut buf = FrameBuffer::from_rgba(
            &pixels
                .iter()
                .flat_map(|(r, g, b, a)| [*r, *g, *b, *a])
                .collect::<Vec<_>>(),
            3,
            3,
        );
        buf.apply_dithering(DitherMethod::FloydSteinberg, 1.0, 2);
        // After FS on a uniform grey, not all pixels can stay identical —
        // the first pixel is quantised and its error propagates.
        let first = buf.luma[0];
        let second = buf.luma[1];
        assert_ne!(
            first, second,
            "FS dithering should break up a uniform mid-grey field"
        );
    }

    #[test]
    fn test_none_dither_leaves_buffer_unchanged() {
        let pixels: Vec<(u8, u8, u8, u8)> = vec![(100, 100, 100, 255); 4];
        let raw: Vec<u8> = pixels
            .iter()
            .flat_map(|(r, g, b, a)| [*r, *g, *b, *a])
            .collect();
        let mut buf = FrameBuffer::from_rgba(&raw, 2, 2);
        let before = buf.luma.clone();
        buf.apply_dithering(DitherMethod::None, 1.0, 10);
        assert_eq!(buf.luma, before);
    }

    // Character mapping

    #[test]
    fn test_classic_maps_black_to_first_char() {
        let buf = buf_from_pixels(&[(0, 0, 0, 255)], 1, 1);
        let settings = AsciiSettings {
            charset: " .:-=+*#%@".to_string(),
            ..Default::default()
        };
        if let RenderOutput::Classic(chars) = buf.render(&settings) {
            assert_eq!(chars[0].glyph, ' ');
        } else {
            panic!("Expected Classic output");
        }
    }

    #[test]
    fn test_classic_maps_white_to_last_char() {
        let buf = buf_from_pixels(&[(255, 255, 255, 255)], 1, 1);
        let settings = AsciiSettings {
            charset: " .:-=+*#%@".to_string(),
            ..Default::default()
        };
        if let RenderOutput::Classic(chars) = buf.render(&settings) {
            assert_eq!(chars[0].glyph, '@');
        } else {
            panic!("Expected Classic output");
        }
    }

    #[test]
    fn test_classic_maps_midgrey_to_midchar() {
        // luma ≈ 127.5 with a 10-char set -> index = round(0.5 * 9) = 5 -> '+'
        let buf = buf_from_pixels(&[(128, 128, 128, 255)], 1, 1);
        let settings = AsciiSettings {
            charset: " .:-=+*#%@".to_string(),
            ..Default::default()
        };
        if let RenderOutput::Classic(chars) = buf.render(&settings) {
            assert_eq!(chars[0].glyph, '+');
        } else {
            panic!("Expected Classic output");
        }
    }

    // ── Braille mapping ───────

    /// Pixel layout (2×4, threshold = 128):
    ///   (0,0)=255  (1,0)=0
    ///   (0,1)=0    (1,1)=0
    ///   (0,2)=0    (1,2)=0
    ///   (0,3)=0    (1,3)=255
    ///
    /// Bits fired: dot1 (weight 1) + dot8 (weight 128) = 129
    /// Unicode: U+2881 = ⢁
    #[test]
    fn test_braille_dot_encoding() {
        let luma_data = vec![255.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 255.0];
        let buf = FrameBuffer {
            width: 2,
            height: 4,
            luma: luma_data,
            rgb: vec![],
        };
        let result = buf.render_braille(128.0);
        assert_eq!(result.trim(), "\u{2881}", "expected ⢁ (U+2881)");
    }

    #[test]
    fn test_braille_all_on_is_full_block() {
        let luma_data = vec![255.0; 8]; // all dots active
        let buf = FrameBuffer {
            width: 2,
            height: 4,
            luma: luma_data,
            rgb: vec![],
        };
        let result = buf.render_braille(0.0);
        // All 8 weights: 1+2+4+8+16+32+64+128 = 255 -> U+28FF = ⣿
        assert_eq!(result.trim(), "\u{28FF}", "expected ⣿ (U+28FF)");
    }

    // Full pipeline

    #[test]
    fn test_process_applies_brightness_before_render() {
        // Very dark pixel + large brightness bump -> should map to a light char
        let mut buf = buf_from_pixels(&[(10, 10, 10, 255)], 1, 1);
        let settings = AsciiSettings {
            brightness: 200.0,
            charset: " .:-=+*#%@".to_string(),
            dither_method: DitherMethod::None,
            ..Default::default()
        };
        if let RenderOutput::Classic(chars) = process(&mut buf, &settings) {
            // After +200 brightness, luma ≈ 212 -> near the end of the ramp
            assert!(
                chars[0].glyph == '#' || chars[0].glyph == '%' || chars[0].glyph == '@',
                "Expected near-white char, got '{}'",
                chars[0].glyph
            );
        } else {
            panic!("Expected Classic output");
        }
    }
}
