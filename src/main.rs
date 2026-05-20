#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod openrouter;
mod secure_store;
mod ui;
mod storage;

use app::App;
use eframe::egui;

struct OpenChatApp {
    inner: App,
}

impl eframe::App for OpenChatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::render(&mut self.inner, ctx);
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([720.0, 480.0])
            .with_title("Lumen Chat"),
        ..Default::default()
    };

    eframe::run_native(
        "Lumen Chat",
        native_options,
        Box::new(|_cc| {
            let app = App::new().expect("failed to initialize app");
            Ok(Box::new(OpenChatApp { inner: app }))
        }),
    )
}