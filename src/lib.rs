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

mod audio_player;
mod misc;
mod timer;
mod widgets;

use std::{path::PathBuf, time::Duration};

use audio_player::SingletonPlayer;

use eframe::egui::{self, RichText};
use egui::{vec2, CentralPanel};

// ⏴⏵⏶⏷⏩⏪⏭⏮⏸⏹⏺■▶★☆☐☑↺↻⟲⟳⬅➡⬆⬇⬈⬉⬊⬋⬌⬍⮨⮩⮪⮫⊗✔⛶
// 🔀🔁🔃
// ☜☝☞☟⛃  ♡
pub mod emoji_icons {
    /// Next breakpoint
    pub const NEXT_BRK_PT: &str = "⏩";
    /// Previous breakpoint
    pub const PREV_BRK_PT: &str = "⏪";
    pub const PAUSE: &str = "⏸";
    pub const RESUME: &str = "⏵";
    pub const NO_VOLUME: &str = "🔈";
    pub const NORMAL_VOLUME: &str = "🔉";
    pub const FULL_VOLUME: &str = "🔊";
}

/// Reserved
pub const DEFAULT_WINDOW_TITLE: &str = "断点音频播放器";

pub struct PlayerApp {
    window_title: String,
    player: SingletonPlayer,
    show_volume_slider: bool,
    volume: u8,
    progress_buffer: u64,
    speed_enum_index: usize,
}

impl PlayerApp {
    const SPEED_ENUMERATION: [&str; 5] = ["0.5×", "0.75×", "1.0×", "1.25×", "1.5×"];

    fn speed_text(&self) -> &'static str {
        Self::SPEED_ENUMERATION[self.speed_enum_index]
    }

    pub fn new(cc: &eframe::CreationContext<'_>, file_path: PathBuf) -> Self {
        misc::setup_font(&cc.egui_ctx);

        // TODO: Display name of played file on the title bar
        // let appended_string = file_path.to_str().unwrap().to_string();

        // if !appended_string.is_empty() {
        //     appended_string = String::from(" - ") + appended_string;
        // }

        Self {
            player: SingletonPlayer::try_new(&file_path).unwrap(),
            window_title: DEFAULT_WINDOW_TITLE.to_string(),
            volume: 100,
            progress_buffer: u64::default(),
            show_volume_slider: bool::default(),
            speed_enum_index: 2,
        }
    }

    #[inline(always)]
    fn play_control_button_ui(&mut self, ui: &mut egui::Ui, _bar_rect: &egui::Rect) {
        ui.horizontal(|ui| {
            ui.menu_button(self.speed_text(), |ui| {
                let mut index = 0;
                for text in Self::SPEED_ENUMERATION {
                    if ui.button(text).clicked() {
                        self.speed_enum_index = index;
                        self.player.set_speed(0.5 + index as f32 * 0.25);
                        ui.close_menu();
                        break;
                    }
                    index += 1;
                }
            });

            let button_radius = 35.0;

            if ui
                .add(widgets::rounding_button(
                    RichText::new(emoji_icons::PREV_BRK_PT).size(button_radius / 2.0),
                    button_radius,
                ))
                .clicked()
            {}

            let play_control_icon = if self.player.is_paused() {
                emoji_icons::RESUME
            } else {
                emoji_icons::PAUSE
            };

            if ui
                .add(widgets::rounding_button(
                    RichText::new(play_control_icon).size((button_radius + 5.0) / 1.5),
                    button_radius + 5.0,
                ))
                .clicked()
            {
                if !self.player.is_empty() {
                    self.player.switch_playback_status();
                }
            }

            if ui
                .add(widgets::rounding_button(
                    RichText::new(emoji_icons::NEXT_BRK_PT).size(button_radius / 2.0),
                    button_radius,
                ))
                .clicked()
            {}

            let volume_icon = match self.volume {
                0 => emoji_icons::NO_VOLUME,
                75.. => emoji_icons::FULL_VOLUME,
                _ => emoji_icons::NORMAL_VOLUME,
            };

            ui.vertical(|ui| {
                if ui
                    .add(egui::Button::new(RichText::new(volume_icon).size(15.0)).frame(false))
                    .on_hover_text("音量")
                    .clicked()
                {
                    self.show_volume_slider = !self.show_volume_slider;
                }

                let volume = self.volume;

                if self.show_volume_slider {
                    ui.add(
                        egui::Slider::new(&mut self.volume, 0..=100)
                            .show_value(false)
                            .text(volume.to_string())
                            .vertical(),
                    );
                }

                self.player.set_volume(volume as f32);
            });
        });

        if self.player.get_total_duration() != Duration::default() {
            let progress_in_secs = self.player.get_progress();

            if ui
                .add(
                    egui::Slider::new(
                        &mut self.progress_buffer,
                        0..=self.player.get_total_duration().as_secs(),
                    )
                    .show_value(false)
                    .text(format!(
                        "{}:{}",
                        progress_in_secs / 60,
                        progress_in_secs % 60
                    )),
                )
                .drag_released()
            {
                self.player.set_progress(self.progress_buffer);
            }
        }
    }

    #[inline(always)]
    fn function_bar_ui(&mut self, ui: &mut egui::Ui, function_bar_rect: &egui::Rect) {
        let painter = ui.painter();

        painter.line_segment(
            [
                function_bar_rect.min + vec2(1.0, 0.0),
                function_bar_rect.right_top() + vec2(0.0, 0.0),
            ],
            ui.visuals().widgets.noninteractive.bg_stroke,
        );

        self.play_control_button_ui(ui, function_bar_rect);
    }

    fn content(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
        add_contents: impl FnOnce(&mut egui::Ui),
    ) {
        let panel_frame = egui::Frame {
            fill: ctx.style().visuals.window_fill(),
            rounding: 10.0.into(),
            stroke: ctx.style().visuals.widgets.noninteractive.fg_stroke,
            outer_margin: 0.5.into(), // so the stroke is within the bounds
            ..Default::default()
        };

        CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            let app_rect = ui.max_rect();

            let title_bar_height = 32.0;
            let title_bar_rect = {
                let mut rect = app_rect;
                rect.max.y = rect.min.y + title_bar_height;
                rect
            };

            misc::title_bar_ui(ui, frame, &title_bar_rect, self.window_title.as_str());

            self.menu_bar_ui(ui, frame);

            let function_bar_height = 64.0;
            let function_bar_rect = {
                let mut rect = app_rect;
                rect.max.y = rect.max.y - function_bar_height;
                rect.min.y = rect.max.y;
                rect
            };

            self.function_bar_ui(ui, &function_bar_rect);

            let app_rect = ui.min_rect();

            // Add the contents:
            let content_rect = {
                let mut rect = app_rect;
                rect.min.y = title_bar_rect.max.y;
                rect.max.y = rect.max.y;
                rect
            }
            .shrink(4.0);
            let mut content_ui = ui.child_ui(content_rect, *ui.layout());
            add_contents(&mut content_ui);
        });
    }

    fn menu_button_file_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        use windows::{w, Win32::UI::WindowsAndMessaging};

        use windows::core::PCWSTR;
        use WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

        ui.menu_button("文件", |ui| {
            if ui.button("打开").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("打开音频文件")
                    .add_filter("音频文件", &["wav", "mp3"])
                    .pick_file()
                {
                    let error_msg = self.player.play_once(&path);
                    if error_msg.is_err() {
                        unsafe {
                            MessageBoxW(
                                None,
                                PCWSTR::from_raw(
                                    error_msg.unwrap_err_unchecked().as_ptr() as *const u16
                                ),
                                w!("错误"),
                                MB_OK | MB_ICONERROR,
                            );
                        }
                    }

                    ui.close_menu();
                }
            }
            if ui.button("退出").clicked() {
                misc::confirm_exit(frame);
            }
        });
    }

    // fn appearence_ui(&mut self){}

    fn menu_bar_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.horizontal(|ui| {
            self.menu_button_file_ui(ui, frame);
        });
    }
}

impl eframe::App for PlayerApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // #[allow(unused_variables)]
        self.content(ctx, frame, |_ui| {
            // ui.add(egui::Label::new(emoji_icons::FULL_VOLUME));
            // ui.label(emoji_icons::NORMAL_VOLUME);
            // ui.label(emoji_icons::NO_VOLUME);
        });
    }
}

#[test]
fn foo() {}
