mod engine;
mod settings;

use engine::{process, FrameBuffer, RenderOutput};
use image::GenericImageView;
use settings::{ArtStyle, AsciiSettings, ColorMode, DitherMethod};

fn main() {
    //  Configuration
    let settings = AsciiSettings {
        style: ArtStyle::Classic,
        dither_method: DitherMethod::FloydSteinberg,
        dither_strength: 1.0,
        contrast: 1.2,
        brightness: 5.0,
        invert: false,
        color_mode: ColorMode::Grayscale,
        charset: " .:-=+*#%@".to_string(),
        braille_threshold: 128.0,
    };

    //  Load image
    // need a "test_image.jpg" in the project root to run this demo
    let img = image::open("test_image.jpg")
        .expect("Could not open test_image.jpg — place a JPEG in the project root to test");
    let (width, height) = img.dimensions();
    let raw = img.to_rgba8().into_raw();

    //  Run the pipeline
    let mut buf = FrameBuffer::from_rgba(&raw, width, height);
    let output = process(&mut buf, &settings);

    //  Print to terminal
    match output {
        RenderOutput::Classic(chars) => {
            for y in 0..height {
                for x in 0..width {
                    let px = &chars[(y * width + x) as usize];
                    // TODO: wrap with ANSI escape codes for ColorMode::FullColor etc.
                    // for now we just print the glyph
                    print!("{}", px.glyph);
                }
                println!();
            }
        }
        RenderOutput::Braille(text) => {
            print!("{}", text);
        }
    }
}
