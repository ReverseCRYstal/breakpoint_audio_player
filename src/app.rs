use crate::audio_player::SingletonPlayer;
use crate::breakpoint::Breakpoint;
use crate::constants;
use crate::gui;
use crate::misc;
use crate::open_status_guard::Guardian;

use std::collections::BinaryHeap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use eframe::egui;
use eframe::epaint::vec2;
use egui::{Button, Context, RichText, Slider, Ui};

use egui_notify::Anchor;

type VoidResult = Result<(), anyhow::Error>;

pub struct App {
    toasts: egui_notify::Toasts,
    breakpoints: BinaryHeap<Breakpoint>,
    this_path: PathBuf,
    player: SingletonPlayer,
    is_muted: bool,
    should_confirm_exit: bool,
    volume: u8,
    progress_buffer: u64,
    open_stat_guardian: Guardian,
    cur_speed_enum_idx: usize,
}

// construction
impl App {
    #[inline(always)]
    pub fn new(app_path: PathBuf, file_to_open: Option<PathBuf>) -> Self {
        let mut player = SingletonPlayer::new();
        let mut toasts = egui_notify::Toasts::default().with_anchor(Anchor::TopRight);
        let mut breakpoints = BinaryHeap::new();

        let result = || -> VoidResult {
            if file_to_open.as_ref().is_some_and(|buf| buf.exists()) {
                let p = unsafe { file_to_open.unwrap_unchecked() };
                let unsupported_err = || -> anyhow::Error {
                    anyhow::anyhow!("Attempting to open an unsupported file type.")
                };

                let extension = p
                    .extension()
                    .map(|s| s.to_str())
                    .ok_or(unsupported_err())?
                    .ok_or(unsupported_err())?;

                let p = match extension {
                    constants::literal::EXTENSION_NAME => {
                        let root = misc::unzip(&p, &app_path)?;
                        breakpoints = misc::handle_config(&root)?;
                        Ok(root.join("audio.mp3"))
                    }
                    "mp3" => Ok(p.clone()),
                    _ => Err(unsupported_err()),
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
            is_muted: false,
            should_confirm_exit: true,
            progress_buffer: 0,
            open_stat_guardian: Default::default(),
            cur_speed_enum_idx: 2,
            volume: 100,
        }
    }
}

// window layout
impl App {
    /// From egui
    /// Render title bar
    #[inline(always)]
    fn title_bar_ui(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        frame: &mut eframe::Frame,
        title_bar_rect: &eframe::epaint::Rect,
        title: &str,
    ) {
        let visual_mut = ui.visuals_mut();
        let bg_fill = visual_mut.noninteractive().bg_fill;
        visual_mut.widgets.inactive.weak_bg_fill = bg_fill;
        // let stroke = &mut visual_mut.widgets.active.bg_stroke;
        // *stroke = egui::Stroke::new(stroke.width, bg_fill);

        let title_bar_response = ui.interact(
            *title_bar_rect,
            egui::Id::new("title_bar"),
            egui::Sense::click(),
        );

        ui.add_space(7.0);

        ui.horizontal(|ui| {
            gui::global_dark_light_mode_switch_localizable(ui, "切换到白昼模式", "切换到夜间模式");

            ui.menu_button(RichText::new("文件"), |ui| {
                if let Err(caption) = self.file_menu_ui(ctx, ui, frame) {
                    self.toasts.error(caption.to_string());
                }
            });
            ui.menu_button("外观", |ui| {
                if let Err(caption) = self.appearance_menu_ui(ctx, ui) {
                    self.toasts.error(caption.to_string());
                }
            });
            ui.menu_button("帮助", |ui| {
                self.help_menu_ui(ui);
            });
        });

        let painter = ui.painter();

        // Paint the title:
        painter.text(
            title_bar_rect.center(),
            egui::Align2::CENTER_CENTER,
            title,
            egui::FontId::proportional(20.0),
            ui.style().visuals.text_color(),
        );

        // Paint the line under the title:
        painter.line_segment(
            [
                title_bar_rect.left_bottom() + vec2(1.0, 0.0),
                title_bar_rect.right_bottom(),
            ],
            ui.visuals().widgets.noninteractive.bg_stroke,
        );

        // Interact with the title bar (drag to move window):
        if title_bar_response.double_clicked() {
            frame.set_maximized(!frame.info().window_info.maximized);
        } else if title_bar_response.is_pointer_button_down_on() {
            frame.drag_window();
        }

        ui.allocate_ui_at_rect(*title_bar_rect, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.visuals_mut().button_frame = false;
                ui.add_space(8.0);
                gui::close_maximize_minimize(ui, frame);
            });
        });
    }
}

// literals
impl App {
    const SPEED_OPTIONS: [&str; 5] = ["0.5×", "0.75×", "1.0×", "1.25×", "1.5×"];
}

// playback ui
impl App {
    #[inline(always)]
    fn progress_slider(&mut self, ui: &mut Ui) {
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
        ui.menu_button("播放倍速", |ui| {
            for (index, text) in Self::SPEED_OPTIONS.iter().enumerate() {
                let text = if self.cur_speed_enum_idx == index {
                    String::from("⏵")
                } else {
                    String::new()
                } + *text;

                if ui.button(text).clicked() {
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
                    egui::Button::new(RichText::new(constants::icon::PREV_BRK_PT).size(icon_size))
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
    ) -> VoidResult {
        if ui.button("打开").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("打开")
                .add_filter("文件", &["bax", "mp3"])
                .pick_file()
            {
                self.player
                    .replace_file(BufReader::new(File::open(misc::unzip(
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
            frame.close()
        }
        Ok(())
    }

    #[inline(always)]
    fn appearance_menu_ui(&mut self, ctx: &Context, ui: &mut Ui) -> VoidResult {
        if ui.button("打开外观文件").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON文本文件", &["json"])
                .set_title("打开主题文件")
                .pick_file()
            {
                let viuals = serde_json::from_str::<egui::Visuals>(
                    std::io::read_to_string(BufReader::new(File::open(path)?))?.as_str(),
                )?;
                ctx.set_visuals(viuals);
            }
            ui.close_menu();
        }
        Ok(())
    }

    #[inline(always)]
    fn help_menu_ui(&mut self, ui: &mut Ui) {
        if ui.button("教程").clicked() {
            self.open_stat_guardian.set_window_status("教程", true);
            ui.close_menu();
        }
        if ui.button("关于").clicked() {
            self.open_stat_guardian.set_window_status("关于", true);
            ui.close_menu();
        }
    }
}

// Display Panel
impl App {
    #[inline(always)]
    fn bottom_panel(&mut self, panel_frame: egui::Frame, ctx: &Context) {
        let panel_frame = {
            let mut panel_frame = panel_frame;
            panel_frame.rounding.ne = 0.;
            panel_frame.rounding.nw = 0.;
            panel_frame
        };

        egui::TopBottomPanel::bottom("bottom_panel")
            .frame(panel_frame)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    self.play_control_buttons(ui);
                    self.volume_control(ui);
                });
                let painter = ui.painter();
                //painter.debug_rect(ui.max_rect(), egui::Color32::BLUE, "max");
            });
    }
}

// Windows
impl App {
    fn render_windows(&mut self, ctx: &Context, center_pos: egui::Pos2) {
        self.open_stat_guardian
            .create_window("关于", false)
            .collapsible(false)
            .default_pos(center_pos)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.heading(
                            egui::special_emojis::OS_WINDOWS.to_string()
                                + constants::literal::APP_NAME,
                        );
                    });
                    ui.label("提交:".to_owned() + constants::literal::COMMIT_HASH);
                    ui.label("构建工具链:".to_owned() + constants::literal::BUILD_TOOLCHAIN);
                    ui.label("rust版本:".to_owned() + constants::literal::RUST_EDITION);
                    ui.label("构建时间:".to_owned() + constants::literal::BUILD_TIME);
                });
            });

        self.open_stat_guardian
            .create_window("教程", false)
            .collapsible(false)
            .default_pos(center_pos)
            .show(ctx, |_ui| {});
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn on_close_event(&mut self) -> bool {
        if self.should_confirm_exit {
            gui::confirm_exit()
        } else {
            true
        }
    }

    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        ctx.set_debug_on_hover(true);

        if !(self.player.is_empty() || self.player.is_paused()) {
            ctx.request_repaint();
        }

        let panel_frame = egui::Frame {
            fill: ctx.style().visuals.window_fill(),
            rounding: 10.0.into(),
            stroke: ctx.style().visuals.widgets.noninteractive.fg_stroke,
            outer_margin: 0.5.into(), // so the stroke is within the bounds
            ..Default::default()
        };

        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| {
                let app_rect = ui.max_rect();
                let title_bar_height = 32.0;
                let title_bar_rect = {
                    let mut rect = app_rect;
                    rect.max.y = rect.min.y + title_bar_height;
                    rect
                };

                self.title_bar_ui(
                    ctx,
                    ui,
                    frame,
                    &title_bar_rect,
                    (constants::literal::APP_NAME.to_string()
                        + " "
                        + constants::literal::TEST_VERSION
                        + " "
                        + constants::literal::APP_VERSION)
                        .as_str(),
                );

                // Add the contents:
                let content_rect = {
                    let mut rect = app_rect;
                    rect.min.y = title_bar_rect.max.y;
                    rect
                };
                //.shrink(4.0);

                let ui = &mut ui.child_ui(content_rect, *ui.layout());

                self.render_windows(ctx, content_rect.center());

                self.toasts.show(ctx);

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

                let ui = &mut ui.child_ui(ui.available_rect_before_wrap(), *ui.layout());

                self.bottom_panel(panel_frame, ctx);
            })
            .response
            .context_menu(|ui| {
                self.speed_control(ui);
            });
    }
}
