use eframe::egui;
use image::DynamicImage;
use crate::common::settings::AsciiSettings;
use crate::engine::pipeline::FrameBuffer;

pub struct Px2AscApp {
    settings: AsciiSettings,
    raw_image: Option<DynamicImage>,
    frame_buffer: Option<FrameBuffer>,
    image_dimensions: (u32, u32),
}

impl Default for Px2AscApp {
    fn default() -> Self {
        Self::new()
    }
}

impl Px2AscApp {
    pub fn new() -> Self {
        Self {
            settings: AsciiSettings::default(),
            raw_image: None,
            frame_buffer: None,
            image_dimensions: (0, 0),
        }
    }

    fn load_image(&mut self) {
        let file_dialog = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "gif", "bmp", "webp"])
            .set_title("Open Image");

        if let Some(path) = file_dialog.pick_file() {
            match image::open(&path) {
                Ok(img) => {
                    self.image_dimensions = (img.width(), img.height());
                    self.raw_image = Some(img);
                    self.rebuild_grid();

                    log::info!("Loaded image: {}", path.display());
                }
                Err(e) => {
                    log::error!("Failed to load image: {}", e);
                }
            }
        }
    }

    fn rebuild_grid(&mut self) {
        let Some(ref img) = self.raw_image else {
            return;
        };

        let cols = (img.width() as f32 / (self.settings.font_size * self.settings.char_spacing))
            .max(1.0) as u32;
        let rows = (img.height() as f32 / self.settings.font_size)
            .max(1.0) as u32;

        log::debug!("Rebuilding grid: {}×{} → {}×{} chars",
            img.width(), img.height(), cols, rows);

        let resized = img.resize_exact(
            cols,
            rows,
            image::imageops::FilterType::Lanczos3,
        );

        let rgba_image = resized.to_rgba8();
        let rgba_data = rgba_image.into_raw();

        self.frame_buffer = Some(FrameBuffer::from_rgba(
            cols as usize,
            rows as usize,
            rgba_data,
        ));
    }
}

impl eframe::App for Px2AscApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Px2Asc - ASCII Art Converter");

            if ui.button("Load Image").clicked() {
                self.load_image();
            }

            if let Some(ref buffer) = self.frame_buffer {
                ui.separator();
                ui.label(format!(
                    "Image: {}×{} → Grid: {}×{} chars",
                    self.image_dimensions.0,
                    self.image_dimensions.1,
                    buffer.width,
                    buffer.height
                ));
            }
        });
    }
}

pub fn run() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Px2Asc",
        options,
        Box::new(|_cc| Ok(Box::new(Px2AscApp::new()))),
    )
}
