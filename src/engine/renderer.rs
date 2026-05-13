#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColoredChar {
    pub glyph: char,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColoredChar {
    pub fn new(glyph: char, r: u8, g: u8, b: u8) -> Self {
        Self { glyph, r, g, b }
    }
}
