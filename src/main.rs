#![warn(clippy::unnecessary_unwrap, clippy::assign_op_pattern)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio_player;
mod constants;
mod gui;
mod misc;
mod timer;

use app::Mode;
use std::path::PathBuf;

fn main() -> Result<(), eframe::Error> {
    let argv: Vec<String> = std::env::args().collect();
    let (path, mode) = if argv.len() >= 2 {
        (PathBuf::from(argv[1].as_str()), Mode::Play)
    } else {
        (PathBuf::new(), Mode::Edit)
    };

    eframe::run_native(
        constants::DEFAULT_WINDOW_TITLE,
        window_option(),
        Box::new(|cc| {
            misc::setup_font(&cc.egui_ctx);
            Box::new(app::App::new(path, mode))
        }),
    )
}

#[inline(always)]
fn window_option() -> eframe::NativeOptions {
    use eframe::egui;

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
