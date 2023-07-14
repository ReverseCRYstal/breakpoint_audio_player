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

use std::path::PathBuf;

use crate::audio_player::SingletonPlayer;
use crate::constants;
use crate::gui;

use eframe::egui;
use egui::{Button, Context, RichText, Ui};

pub enum Mode {
    Edit,
    Play,
}

pub struct PlayerApp {
    window_title: String,
    player: SingletonPlayer,
    show_volume_slider: bool,
    should_confirm_exit: bool,
    volume: u8,
    mode: Mode,
    progress_buffer: u64,
    cur_speed_enum_idx: usize,
}

impl PlayerApp {
    const SPEED_ENUMERATION: [&str; 5] = ["0.5×", "0.75×", "1.0×", "1.25×", "1.5×"];

    fn speed_text(&self) -> &'static str {
        Self::SPEED_ENUMERATION[self.cur_speed_enum_idx]
    }

    pub fn new(file_path: PathBuf, launch_mode: Mode) -> Self {
        // TODO: Display name of played file on the title bar
        // let appended_string = file_path.to_str().unwrap().to_string();

        // if !appended_string.is_empty() {
        //     appended_string = String::from(" - ") + appended_string;
        // }

        Self {
            player: SingletonPlayer::try_new(&file_path).unwrap(),
            window_title: constants::DEFAULT_WINDOW_TITLE.to_string(),
            volume: 100,
            mode: launch_mode,
            progress_buffer: 0,
            show_volume_slider: false,
            should_confirm_exit: false,
            cur_speed_enum_idx: 2,
        }
    }

    #[inline(always)]
    fn volume_control(&mut self, ui: &mut Ui) {
        let volume_icon = match self.volume {
            0 => constants::NO_VOLUME,
            75.. => constants::FULL_VOLUME,
            _ => constants::NORMAL_VOLUME,
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
    }

    #[inline(always)]
    fn play_control_buttons(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.group(|ui| {
                let size = 35.0;
                let rounding = 3.0;
                let (play_control_icon, play_control_size) = if self.player.is_paused() {
                    (constants::RESUME, size + 5.0)
                } else {
                    (constants::PAUSE, size)
                };

                if ui
                    .add(
                        egui::Button::new(RichText::new(constants::PREV_BRK_PT).size(size / 2.0))
                            .rounding(rounding),
                    )
                    .clicked()
                {
                    unimplemented!();
                }

                if ui
                    .add(
                        egui::Button::new(
                            RichText::new(play_control_icon).size((play_control_size + 5.0) / 1.5),
                        )
                        .rounding(rounding),
                    )
                    .clicked()
                {
                    if !self.player.is_empty() {
                        self.player.switch_playback_status();
                    }
                }

                if ui
                    .add(
                        Button::new(RichText::new(constants::NEXT_BRK_PT).size(size))
                            .rounding(rounding),
                    )
                    .clicked()
                {
                    unimplemented!();
                }
            });
        });
    }

    #[inline(always)]
    fn volume_speed_controller(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.menu_button(self.speed_text(), |ui| {
                    for (index, text) in Self::SPEED_ENUMERATION.iter().enumerate() {
                        if ui.button(*text).clicked() {
                            self.cur_speed_enum_idx = index;
                            self.player.set_speed(0.5 + index as f32 * 0.25);
                            ui.close_menu();
                            break;
                        }
                    }
                });

                self.volume_control(ui);
            });
        });

        // playback progress
        if !self.player.is_empty() {
            let progress_in_secs = self.player.get_progress();

            if progress_in_secs > self.progress_buffer {
                self.progress_buffer = progress_in_secs;
            }

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

    // fn appearence_ui(&mut self){}

    #[inline(always)]
    fn menu_bar_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        use windows::{w, Win32::UI::WindowsAndMessaging};

        use windows::core::PCWSTR;
        use WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
        ui.horizontal(|ui| {
            ui.menu_button("文件", |ui| {
                if ui.button("打开").clicked() {
                    let _future = async {
                        let handle = rfd::AsyncFileDialog::new()
                            .set_title("打开")
                            .add_filter("文件", &["bpa", "mp3"])
                            .pick_file()
                            .await;

                        if let Some(path) = handle {
                            let path = path.path();
                            let error_msg = self.player.play_once(&path);
                            if error_msg.is_err() {
                                unsafe {
                                    MessageBoxW(
                                        None,
                                        PCWSTR::from_raw(
                                            error_msg.unwrap_err().to_string().as_ptr()
                                                as *const u16,
                                        ),
                                        w!("错误"),
                                        MB_OK | MB_ICONERROR,
                                    );
                                }
                            } else {
                                self.progress_buffer = Default::default();
                            }

                            ui.close_menu();
                        }
                    };
                }
                if ui.button("退出").clicked() {
                    if self.should_confirm_exit {
                        if gui::confirm_exit() {
                            frame.close();
                        }
                    } else {
                        frame.close();
                    }
                }
            });
        });
    }

    fn show_bottom_bar(&mut self, ctx: &Context) {
        match self.mode {
            Mode::Edit => {}
            Mode::Play => {}
        }

        egui::TopBottomPanel::bottom("btm")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    self.volume_speed_controller(ui);
                    self.play_control_buttons(ui);
                });
            });
    }
}

impl eframe::App for PlayerApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        if self.player.is_empty() || self.player.is_paused() {
            ctx.request_repaint();
        }
        let title = self.window_title.to_string();

        gui::window_frame(ctx, frame, title.as_str(), true, |frame, ui| {
            // let available_height = ui.available_height();
            self.menu_bar_ui(ui, frame);
            match self.mode {
                Mode::Edit => {}
                Mode::Play => {}
            }

            self.show_bottom_bar(ctx);
        });
    }
}
