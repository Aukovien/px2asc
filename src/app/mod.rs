// src/app/mod.rs

use eframe::egui;
use crate::common::settings::AsciiSettings;

pub struct Px2AscApp {
    settings: AsciiSettings,
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
        }
    }
}

impl eframe::App for Px2AscApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Px2Asc - ASCII Art Converter");
            ui.label("Ready to implement features...");
        });
    }
}

pub fn run() -> eframe::Result<()> {
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
