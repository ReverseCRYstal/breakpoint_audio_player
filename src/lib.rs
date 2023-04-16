// #![allow(dead_code)]
// #![allow(unused)]

mod audio_player;
mod misc;
mod my_sink;
mod widgets;

use std::path::PathBuf;

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
    progress: u64,
}

impl PlayerApp {
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
            progress: u64::default(),
            show_volume_slider: bool::default(),
        }
    }

    #[inline(always)]
    fn play_control_button_ui(&mut self, ui: &mut egui::Ui, _bar_rect: &egui::Rect) {
        ui.add(egui::Slider::new(
            &mut self.progress,
            0..=self.player.get_progress(),
        ));
        ui.horizontal(|ui| {
            if ui
                .add(widgets::rounding_button(emoji_icons::PREV_BRK_PT, 34.0))
                .clicked()
            {}

            let play_control_icon = if dbg!(self.player.is_paused()) {
                emoji_icons::RESUME
            } else {
                emoji_icons::PAUSE
            };

            if ui
                .add(widgets::rounding_button(play_control_icon, 38.0))
                .clicked()
            {
                if !self.player.is_empty() {
                    self.player.switch();
                }
            }

            if ui
                .add(widgets::rounding_button(emoji_icons::NEXT_BRK_PT, 34.0))
                .clicked()
            {}
            let volume_icon = match self.volume {
                0 => emoji_icons::NO_VOLUME,
                100 => emoji_icons::FULL_VOLUME,
                _ => emoji_icons::NORMAL_VOLUME,
            };
            ui.vertical(|ui| {
                if ui
                    .button(RichText::new(volume_icon).size(15.0))
                    .on_hover_text("音量")
                    .clicked()
                {
                    self.show_volume_slider = !self.show_volume_slider;
                }

                if self.show_volume_slider {
                    ui.add(egui::Slider::new(&mut self.volume, 0..=100).vertical());
                }
            });
        });
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
                    .add_filter("音频文件", &[".wav", ".mp3"])
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
