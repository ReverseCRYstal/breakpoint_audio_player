#![warn(clippy::unnecessary_unwrap, clippy::assign_op_pattern)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio_player;
mod breakpoint;
mod constants;
mod gui;
mod misc;
mod open_status_guard;
mod tests;
mod timer;

use eframe::egui;
use std::{env::current_dir, path::PathBuf};

fn main() -> Result<(), anyhow::Error> {
    let argv: Vec<String> = std::env::args().collect();

    let file_path = if argv.len() >= 2 {
        Some(PathBuf::from(argv[1].to_string()))
    } else {
        None
    };

    let temp = current_dir().unwrap().join("temp");

    if !temp.exists() {
        std::fs::create_dir(temp)?;
    }

    eframe::run_native(
        constants::literal::APP_NAME,
        window_option(),
        Box::new(|cc| {
            misc::setup_font(&cc.egui_ctx);
            Box::new(app::App::new(file_path))
        }),
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))
}

#[inline(always)]
fn window_option() -> eframe::NativeOptions {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SM_CXFULLSCREEN, SM_CYFULLSCREEN,
    };

    let window_rect = unsafe {
        egui::Rect {
            min: egui::Pos2::ZERO,
            max: egui::Pos2 {
                x: GetSystemMetrics(SM_CXFULLSCREEN) as f32,
                y: GetSystemMetrics(SM_CYFULLSCREEN) as f32,
            },
        }
        .shrink(100.0)
    };

    eframe::NativeOptions {
        default_theme: eframe::Theme::Light,
        decorated: false,
        transparent: true,
        resizable: true,
        initial_window_size: Some(window_rect.max - window_rect.min),
        initial_window_pos: Some(window_rect.min),
        ..Default::default()
    }
}
