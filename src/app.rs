use crate::audio_player::SingletonPlayer;
use crate::breakpoint::Breakpoint;
use crate::constants;
use crate::gui;
use crate::subpanel::*;

use std::collections::BinaryHeap;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;

use eframe::egui;
use eframe::epaint::vec2;
use egui::{Button, Context, RichText, Slider, Ui};

use egui_notify::Anchor;
use zip::ZipArchive;

pub struct App {
    toasts: egui_notify::Toasts,
    window_title: String,
    breakpoints: BinaryHeap<Breakpoint>,
    this_path: PathBuf,
    player: SingletonPlayer,
    is_muted: bool,
    should_confirm_exit: bool,
    volume: u8,
    progress_buffer: u64,
    cur_speed_enum_idx: usize,
}

impl App {
    fn unzip(source_path: &PathBuf, extract_path: &PathBuf) -> Result<PathBuf, anyhow::Error> {
        let mut archive = ZipArchive::new(std::io::BufReader::new(File::open(source_path)?))?;
        let mut ret: Result<PathBuf, anyhow::Error> = Err(anyhow::anyhow!(
            "The archived file do not meet specifications."
        ));

        let iter = archive
            .file_names()
            .map(|v| v.to_string())
            .collect::<Vec<_>>();

        for name in &iter {
            // 会报错吗？先这么写吧。。。
            ret = || -> Result<PathBuf, anyhow::Error> {
                let mut f = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(extract_path.join(name))?;

                f.write_all(archive.by_name(&name)?.extra_data())?;

                let name = PathBuf::from(name);

                if name
                    .extension()
                    .unwrap()
                    .to_str()
                    .is_some_and(|v| v.eq_ignore_ascii_case("mp3"))
                {
                    Ok(extract_path.join(name))
                } else {
                    Err(anyhow::anyhow!(
                        "The archived file do not meet specifications."
                    ))
                }
            }();
        }

        ret
    }
}

// constructor
impl App {
    #[inline(always)]
    pub fn new(app_path: PathBuf, file_to_open: Option<PathBuf>) -> Self {
        let mut player = SingletonPlayer::new();
        let mut toasts = egui_notify::Toasts::default().with_anchor(Anchor::TopRight);
        let mut breakpoints = BinaryHeap::new();

        let result = || -> Result<(), anyhow::Error> {
            if file_to_open.as_ref().is_some_and(|buf| buf.exists()) {
                let p = unsafe { file_to_open.unwrap_unchecked() };
                let err = || -> anyhow::Error {
                    anyhow::anyhow!("Attempting to open an unsupported file type.")
                };

                let extension = (&p)
                    .extension()
                    .map(|s| s.to_str())
                    .ok_or(err())?
                    .ok_or(err())?;

                let p = match extension {
                    "bax" => Ok(Self::unzip(&p, &app_path)?),
                    "mp3" => Ok(p.clone()),
                    _ => Err(err()),
                };

                let file = File::open(p?)?;

                let reader = BufReader::new(file);
                player.replace_file(reader)?;
                // Ok(mp3_duration::from_read(&mut reader).map(|dur| dur.as_secs())?)
            }
            Ok(())
        }();

        if let Some(caption) = result.err() {
            toasts.error(caption.to_string());
        }

        Self {
            toasts,
            player,
            this_path: app_path,
            breakpoints,
            window_title: constants::literal::DEFAULT_WINDOW_TITLE.to_string(),
            is_muted: false,
            should_confirm_exit: false,
            progress_buffer: 0,
            cur_speed_enum_idx: 2,
            volume: 100,
        }
    }
}

// texts
impl App {
    const SPEED_OPTIONS: [&str; 5] = ["0.5×", "0.75×", "1.0×", "1.25×", "1.5×"];

    #[inline(always)]
    fn speed_text(&self) -> &'static str {
        Self::SPEED_OPTIONS[self.cur_speed_enum_idx]
    }
}

// playback ui
impl App {
    #[inline(always)]
    fn progress_silder(&mut self, ui: &mut Ui) {
        let mut place_holder = 0;
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
            let resp =
                if let Some(total_duration) = self.player.total_duration().map(|v| v.as_secs()) {
                    ui.add_enabled(
                        true,
                        Slider::new(&mut self.progress_buffer, 0..=total_duration)
                            .trailing_fill(true)
                            .custom_formatter(|v, _| {
                                format!("{:} / {:}", tostring(v as u64), tostring(total_duration))
                            }),
                    )
                } else {
                    ui.add_enabled(
                        false,
                        Slider::new(&mut place_holder, 0..=1)
                            .show_value(false)
                            .text("00:00"),
                    )
                };

            if self.progress_buffer < play_progress {
                self.progress_buffer = play_progress;
            }

            if resp.drag_released() {
                self.player.set_progress(self.progress_buffer);
            }
        });
    }

    #[inline(always)]
    fn volume_control(&mut self, ui: &mut Ui) {
        let volume_icon = if self.is_muted {
            constants::icon::MUTED_VOLUME
        } else {
            match self.volume {
                0 => constants::icon::NO_VOLUME,
                75.. => constants::icon::FULL_VOLUME,
                _ => constants::icon::NORMAL_VOLUME,
            }
        };

        // TODO custom volume control ui
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
    fn speed_control(&mut self, ui: &mut Ui) {
        ui.menu_button(self.speed_text(), |ui| {
            for (index, text) in Self::SPEED_OPTIONS.iter().enumerate() {
                if ui.button(*text).clicked() {
                    self.cur_speed_enum_idx = index;
                    self.player.set_speed(0.5 + index as f32 * 0.25);
                    ui.close_menu();
                    break;
                }
            }
        });
    }

    #[inline(always)]
    fn play_control_buttons(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.group(|ui| {
                let size = 35.0;
                let rounding = 3.0;
                let (play_control_icon, icon_size) =
                    if self.player.is_paused() || self.player.is_empty() {
                        (constants::icon::RESUME, size)
                    } else {
                        (constants::icon::PAUSE, size)
                    };

                if ui
                    .add(
                        egui::Button::new(
                            RichText::new(constants::icon::PREV_BRK_PT).size(icon_size),
                        )
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
                    if self.player.is_paused() {
                        self.player.resume();
                    } else {
                        self.player.pause();
                    }
                }

                if ui
                    .add(
                        Button::new(RichText::new(constants::icon::NEXT_BRK_PT).size(icon_size))
                            .rounding(rounding),
                    )
                    .clicked()
                {
                    unimplemented!();
                }
            });
        });
    }
}

// menubar
impl App {
    #[inline(always)]
    fn file_menu_ui(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        frame: &mut eframe::Frame,
    ) -> Result<(), anyhow::Error> {
        if ui.button("打开").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("打开")
                .add_filter("文件", &["bax", "mp3"])
                .pick_file()
            {
                self.player
                    .replace_file(BufReader::new(File::open(Self::unzip(
                        &path,
                        &self.this_path.join("temp"),
                    )?)?))?;
            }
        }

        if ui.button("保存").clicked() {
            unimplemented!()
        }

        if ui.button("另存为").clicked() {
            unimplemented!()
        }

        self.toasts.show(ctx);

        if ui.button("退出").clicked() {
            if self.should_confirm_exit {
                if gui::confirm_exit() {
                    frame.close();
                }
            } else {
                frame.close();
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn menu_bar_ui(&mut self, ctx: &Context, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.horizontal(|ui| {
            ui.menu_button(RichText::new("文件").underline(), |ui| {
                self.file_menu_ui(ctx, ui, frame)
            });
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

        let title = self.window_title.to_string() + " " + constants::literal::VERSION_HASH;

        gui::window_frame(ctx, frame, title.as_str(), true, |frame, ui| {
            // let available_height = ui.available_height();
            self.menu_bar_ui(ctx, ui, frame);

            let painter = ui.painter();
            let rest_rect = ui.available_rect_before_wrap();
            let visuals = ui.visuals();

            painter.line_segment(
                [
                    rest_rect.left_top() + vec2(-3., 0.),
                    rest_rect.right_top() + vec2(4., 0.),
                ],
                visuals.noninteractive().bg_stroke,
            );

            //            ui.add_space(2.);

            // let rect = {
            //     let h = ui.max_rect().height();
            //     let rect = ui.available_rect_before_wrap();
            //     egui::Rect::from_min_size(rect.min, egui::vec2(rect.min.x, h))
            // };
            // let mut a = 10;
            // ui.put(rect, egui::Slider::new(&mut a, 0..=100));
            let rect = {
                let rect = ui.available_rect_before_wrap();
                egui::Rect::from_two_pos(
                    rect.left_bottom() + vec2(0.0, -100.0),
                    rect.right_bottom(),
                )
            };
            ui.allocate_ui_at_rect(rect, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    self.speed_control(ui);
                    self.play_control_buttons(ui);
                    self.volume_control(ui);
                });
            });
        });
    }
}
