// MIT License
//
// Copyright (c) 2023 CrYStaL
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

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
            Box::new(app::PlayerApp::new(path, mode))
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
