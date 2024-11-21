mod app;
mod models;
mod pdf_handler;

use app::ChecklistApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Legal Case Checklist"),
        default_theme: eframe::Theme::Light,
        ..Default::default()
    };
    
    eframe::run_native(
        "Legal Case Checklist",
        options,
        Box::new(|_cc| Box::new(ChecklistApp::new())),
    )
}