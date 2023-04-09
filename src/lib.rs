// #![allow(dead_code)]
// #![allow(unused)]

mod audio_player;
mod misc;
mod widgets;

use audio_player::AudioPlayer;

use eframe::egui;
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
pub const WINDOW_TITLE: &str = "断点音频播放器";

pub struct PlayerApp {
    window_title: String,
    player: AudioPlayer,
}

impl PlayerApp {
    pub fn new(cc: &eframe::CreationContext<'_>, file_path: String) -> Self {
        misc::setup_font(&cc.egui_ctx);

        PlayerApp {
            player: AudioPlayer::from_path(&file_path),
            window_title: WINDOW_TITLE.to_string(),
        }
    }

    fn play_control_button_ui(&mut self, ui: &mut egui::Ui, _bar_rect: &egui::Rect) {
        ui.horizontal(|ui| {
            if ui
                .add(widgets::rounding_button(emoji_icons::PREV_BRK_PT, 34.0))
                .clicked()
            {}

            let play_control_icon = if self.player.is_paused() {
                emoji_icons::RESUME
            } else {
                emoji_icons::PAUSE
            };

            if ui
                .add(widgets::rounding_button(play_control_icon, 38.0))
                .clicked()
            {
                self.player.switch();
            }

            if ui
                .add(widgets::rounding_button(emoji_icons::NEXT_BRK_PT, 34.0))
                .clicked()
            {}
        });
    }

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
        ui.menu_button("文件", |ui| {
            if ui.button("打开").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.player
                        .play_single_file(path.display().to_string().as_str());
                    
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
