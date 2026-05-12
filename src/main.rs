// src/main.rs

mod app;
mod common;
mod engine;

fn main() -> eframe::Result<()> {
    env_logger::init();
    app::run()
}
