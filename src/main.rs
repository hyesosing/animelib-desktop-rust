// src/main.rs
mod api;
mod app;
mod cache;
mod ui;

use app::AnimeLibApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    if std::env::args().any(|a| a == "--auth-mode") {
        crate::ui::auth::run_auth_process();
        return Ok(());
    }

    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([900.0, 600.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_title("AnimeLib Desktop"),
        ..Default::default()
    };

    eframe::run_native(
        "AnimeLib Desktop",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(AnimeLibApp::new(cc))
        }),
    )
}
