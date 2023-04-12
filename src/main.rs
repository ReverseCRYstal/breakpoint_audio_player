#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};

use breakpoint_audio_player::*;

fn main() -> Result<(), eframe::Error> {
    let argv: Vec<String> = std::env::args().collect();
    let path = if argv.len() >= 2 {
        PathBuf::from(argv[1].as_str())
    } else {
        PathBuf::new()
    };

    let result = eframe::run_native(
        WINDOW_TITLE,
        get_window_option(),
        Box::new(|cc| Box::new(PlayerApp::new(cc, path.into()))),
    );

    // if result.is_err() {
    //     println!("{}", result.unwrap_err());
    // }
    result
}

fn get_window_option() -> eframe::NativeOptions {
    use eframe::egui;

    use egui::vec2;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SM_CXFULLSCREEN, SM_CYFULLSCREEN,
    };

    // Proportional configuration for window's option
    let golden_factor: f32 = (5.0_f32.sqrt() - 1.0) / 2.0;

    let screen_rect = unsafe {
        egui::Rect {
            min: egui::Pos2::ZERO,
            max: egui::Pos2 {
                x: GetSystemMetrics(SM_CXFULLSCREEN) as f32,
                y: GetSystemMetrics(SM_CYFULLSCREEN) as f32,
            },
        }
    };

    let initial_window_size = {
        let mut size = (screen_rect.max - screen_rect.min) / 2.0;
        size.y = size.y / golden_factor;
        size
    };

    let initial_window_pos = {
        let mut pos = screen_rect.left_bottom();
        pos += vec2(screen_rect.max.x / 10.0, -initial_window_size.y);
        pos
    };

    eframe::NativeOptions {
        default_theme: eframe::Theme::Light,
        decorated: false,
        transparent: true,
        resizable: true,
        // min_window_size: Some(egui::vec2(320.0, 320.0 * golden_factor)),
        initial_window_size: Some(initial_window_size),
        initial_window_pos: Some(initial_window_pos),
        ..Default::default()
    }
}
