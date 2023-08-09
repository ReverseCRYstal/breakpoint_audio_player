use crate::audio_player::SingletonPlayer;
use crate::constants;
use crate::gui;
use crate::inner_panel::*;

use std::path::PathBuf;

use eframe::egui;
use egui::{Button, Context, RichText, Ui};

pub enum Mode {
    Edit,
    Play,
}

pub struct App {
    window_title: String,
    player: SingletonPlayer,
    is_muted: bool,
    should_confirm_exit: bool,
    volume: u8,
    mode: Mode,
    progress_buffer: u64,
    total_duration: u64,
    cur_speed_enum_idx: usize,
}

impl App {
    const SPEED_ENUMERATION: [&str; 5] = ["0.5×", "0.75×", "1.0×", "1.25×", "1.5×"];

    #[inline(always)]
    fn speed_text(&self) -> &'static str {
        Self::SPEED_ENUMERATION[self.cur_speed_enum_idx]
    }

    #[inline(always)]
    pub fn new(file_path: PathBuf, launch_mode: Mode) -> Self {
        // TODO: Display name of played file on the title bar
        // let appended_string = file_path.to_str().unwrap().to_string();

        // if !appended_string.is_empty() {
        //     appended_string = String::from(" - ") + appended_string;
        // }

        let (player, total_duration) = if file_path.exists() {
            (
                SingletonPlayer::try_new(&file_path).unwrap(),
                mp3_duration::from_path(file_path).unwrap().as_secs(),
            )
        } else {
            (SingletonPlayer::default(), 0)
        };

        Self {
            player,
            total_duration,
            window_title: constants::DEFAULT_WINDOW_TITLE.to_string(),
            mode: launch_mode,
            is_muted: false,
            should_confirm_exit: false,
            progress_buffer: 0,
            cur_speed_enum_idx: 2,
            volume: 100,
        }
    }

    #[inline(always)]
    fn volume_control(&mut self, ui: &mut Ui) {
        let volume_icon = if self.is_muted {
            constants::MUTED_VOLUME
        } else {
            match self.volume {
                0 => constants::NO_VOLUME,
                75.. => constants::FULL_VOLUME,
                _ => constants::NORMAL_VOLUME,
            }
        };

        ui.menu_button(volume_icon, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add_sized([30.0, 16.0], Button::new(volume_icon))
                    .clicked()
                {
                    self.is_muted = !self.is_muted;
                }
                ui.add(
                    egui::Slider::new(&mut self.volume, 0..=100)
                        .trailing_fill(true)
                        .show_value(false)
                        .custom_formatter(|v, _| (v as u64).to_string()),
                );
            });
        })
        .response
        .on_hover_text("调整音量");

        self.player.set_volume(self.volume);
    }

    #[inline(always)]
    fn play_control_buttons(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.group(|ui| {
                let size = 35.0;
                let rounding = 3.0;
                let (play_control_icon, icon_size) =
                    if self.player.is_paused() || self.player.is_empty() {
                        (constants::RESUME, size)
                    } else {
                        (constants::PAUSE, size)
                    };

                if ui
                    .add(
                        egui::Button::new(RichText::new(constants::PREV_BRK_PT).size(icon_size))
                            .rounding(rounding),
                    )
                    .clicked()
                {
                    unimplemented!();
                }

                if ui
                    .add_sized(
                        [50.0, 50.0],
                        egui::Button::new(RichText::new(play_control_icon).size(icon_size))
                            .rounding(rounding),
                    )
                    .clicked()
                    && !self.player.is_empty()
                {
                    self.player.switch_playback_status();
                }

                if ui
                    .add(
                        Button::new(RichText::new(constants::NEXT_BRK_PT).size(icon_size))
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
        });

        ui.group(|ui| {
            self.volume_control(ui);
        });
    }

    #[inline(always)]
    fn menu_bar_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        use windows::{w, Win32::UI::WindowsAndMessaging};

        use windows::core::PCWSTR;
        use WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

        let file_btn_ui = |ui: &mut Ui| {
            if ui.button("打开").clicked() {
                let handle = rfd::FileDialog::new()
                    .set_title("打开")
                    .add_filter("文件", &["bpa", "mp3"])
                    .pick_file();

                if let Some(path) = handle {
                    let path = &path;
                    let error_msg = self.player.load(path);
                    self.total_duration = mp3_duration::from_path(path).unwrap().as_secs();

                    if let Err(err_msg) = error_msg {
                        unsafe {
                            MessageBoxW(
                                None,
                                PCWSTR::from_raw(err_msg.as_ptr() as *const u16),
                                w!("错误"),
                                MB_OK | MB_ICONERROR,
                            );
                        }
                    } else {
                        self.progress_buffer = Default::default();
                    }

                    ui.close_menu();
                }
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
        };

        ui.horizontal(|ui| {
            ui.menu_button("文件", file_btn_ui);
        });
    }

    #[inline(always)]
    fn progress_silder(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            let tostring = |duration: u64| -> String {
                let seconds = duration % 60;
                let mut minutes = duration / 60;
                let total_hours = minutes / 60;
                minutes %= 60;
                let hours = if total_hours != 0 {
                    format!("{total_hours}:")
                } else {
                    String::new()
                };
                format!("{hours}{minutes:0>2}:{seconds:0>2}")
            };

            let play_progress = self.player.get_progress();

            let resp = ui.add(
                egui::Slider::new(&mut self.progress_buffer, 0..=self.total_duration)
                    .trailing_fill(true)
                    .custom_formatter(|v, _| {
                        format!(
                            "{:} / {:}",
                            tostring(v as u64),
                            tostring(self.total_duration)
                        )
                    }),
            );

            if self.progress_buffer < play_progress {
                self.progress_buffer = play_progress;
            }

            if resp.drag_released() {
                self.player.set_progress(self.progress_buffer);
            }
        });
    }

    #[inline(always)]
    fn show_side_bar(&mut self, ui: &mut Ui) {
        self.volume_speed_controller(ui);
    }

    #[inline(always)]
    fn show_bottom_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.play_control_buttons(ui);
            self.progress_silder(ui);
        });
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        if !(self.player.is_empty() || self.player.is_paused()) {
            ctx.request_repaint();
        }

        let title = self.window_title.to_string();

        gui::window_frame(ctx, frame, title.as_str(), true, |frame, ui| {
            // let available_height = ui.available_height();
            self.menu_bar_ui(ui, frame);
            ui.separator();

            let mut spawner = PanelSpawner::new(ui);

            match self.mode {
                Mode::Edit => {}
                Mode::Play => {}
            }

            if !self.player.is_empty() {
                spawner
                    .allocate(100.0, Dir4::Bottom, Some(3.0))
                    .separator()
                    .show(ctx, |ui| {
                        self.show_bottom_bar(ui);
                    });
            }

            spawner
                .allocate(100.0, Dir4::Left, Some(3.0))
                .separator()
                .show(ctx, |ui| {
                    self.show_side_bar(ui);
                });

            spawner.show(|_ui| {});
        });
    }
}
