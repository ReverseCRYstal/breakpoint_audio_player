#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use breakpoint_audio_player::*;
use eframe::{egui, egui::vec2};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXFULLSCREEN, SM_CYFULLSCREEN};

fn get_screen_rect_no_task_bar() -> egui::Rect {
    unsafe {
        egui::Rect {
            min: egui::Pos2::ZERO,
            max: egui::Pos2 {
                x: GetSystemMetrics(SM_CXFULLSCREEN) as f32,
                y: GetSystemMetrics(SM_CYFULLSCREEN) as f32,
            },
        }
    }
}

fn get_window_options() -> eframe::NativeOptions {
    // Proportional configuration for window's option
    let golden_factor: f32 = (5.0_f32.sqrt() - 1.0) / 2.0;
    let screen_rect_no_task_bar = get_screen_rect_no_task_bar();

    let initial_window_size = {
        let mut size = (screen_rect_no_task_bar.max - screen_rect_no_task_bar.min) / 2.0;
        size.y = size.y / golden_factor;
        size
    };

    let intial_window_pos = {
        let mut pos = screen_rect_no_task_bar.left_bottom();
        pos += vec2(screen_rect_no_task_bar.max.x / 10.0, -initial_window_size.y);
        pos
    };

    eframe::NativeOptions {
        default_theme: eframe::Theme::Light,
        decorated: false,
        transparent: true,
        resizable: true,
        // min_window_size: Some(egui::vec2(320.0, 320.0 * golden_factor)),
        initial_window_size: Some(initial_window_size),
        initial_window_pos: Some(intial_window_pos),
        ..Default::default()
    }
}

fn main() -> Result<(), eframe::Error> {
    let result = eframe::run_native(
        WINDOW_TITLE,
        get_window_options(),
        Box::new(|cc| Box::new(PlayerApp::new(cc, ".\\assests\\example_audio.mp3".into()))),
    );

    // if result.is_err() {
    //     println!("{}", result.unwrap_err());
    // }
    result
}
