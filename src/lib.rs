// #![allow(dead_code)]
// #![allow(unused)]

mod audio_player;
mod misc;
mod widgets;

pub mod window_options {
    use eframe::{egui, egui::vec2};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SM_CXFULLSCREEN, SM_CYFULLSCREEN,
    };

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

    pub fn get() -> eframe::NativeOptions {
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
}

use audio_player::AudioPlayer;

use eframe::egui;
use egui::{vec2, CentralPanel};

use windows::{w, Win32::UI::WindowsAndMessaging};
use WindowsAndMessaging::{MessageBoxW, IDYES, MB_ICONASTERISK, MB_YESNO};

// Here are some useful emojis that will be used in the future
// ⏴⏵⏶⏷⏩⏪⏭⏮⏸⏹⏺■▶★☆☐☑↺↻⟲⟳⬅➡⬆⬇⬈⬉⬊⬋⬌⬍⮨⮩⮪⮫⊗✔⛶
// ∞⎗⎘⎙⏏📾🔀🔁🔃☀☁
// ☜☝☞☟⛃  ♡ 📅📆 📈📉📊
pub mod emoji_icons {
    /// Next breakpoint
    pub const NEXT_BRK_PT: &str = "⏩";
    /// Previous breakpoint
    pub const PREV_BRK_PT: &str = "⏪";
    pub const PAUSE: &str = "⏸";
    pub const RESUME: &str = "⏵";
}

/// Reserved
pub const WINDOW_TITLE: &str = "断点音频播放器";

pub struct PlayerApp {
    window_title: String,
    audio_path: String,
    player: AudioPlayer,
    // show_play_control_ui: bool,
}

impl PlayerApp {
    pub fn new(cc: &eframe::CreationContext<'_>, file_path: String) -> Self {
        misc::setup_font(&cc.egui_ctx);

        PlayerApp {
            player: AudioPlayer::default(),
            audio_path: file_path,
            window_title: WINDOW_TITLE.to_string(),
            // show_play_control_ui: false,
        }
    }

    /// From egui
    fn function_bar_ui(&mut self, ui: &mut egui::Ui, function_bar_rect: &eframe::epaint::Rect) {
        let painter = ui.painter();

        painter.line_segment(
            [
                function_bar_rect.min + vec2(1.0, 0.0),
                function_bar_rect.right_top() + vec2(0.0, 0.0),
            ],
            ui.visuals().widgets.noninteractive.bg_stroke,
        );

        if ui
            .add(widgets::rounding_button(emoji_icons::PREV_BRK_PT, 34.0))
            .clicked()
        {}

        let play_control_icon = if self.player.is_paused() {
            emoji_icons::PAUSE
        } else {
            emoji_icons::RESUME
        };

        if ui
            .add(widgets::rounding_button(play_control_icon, 38.0))
            .clicked()
        {
            self.player.resume();
        }
        if ui
            .add(widgets::rounding_button(emoji_icons::NEXT_BRK_PT, 34.0))
            .clicked()
        {}
    }

    /// From egui
    ///
    /// Feature requires:
    /// Restore the window while double-clicked the function bar
    /// Restore/Maxiumize the window while double-clicked the title bar
    /// Basic window's function
    /// Audio playback operations
    /// Resize the window while drag the window frame  
    fn custom_window_frame(
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

            self.menu_bar_ui(ui, frame);

            let function_bar_height = 64.0;
            let function_bar_rect = {
                let mut rect = app_rect;
                rect.max.y = rect.max.y - function_bar_height;
                rect.min.y = rect.max.y;
                rect
            };

            self.function_bar_ui(ui, &function_bar_rect);
        });
    }

    fn menu_button_file_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.menu_button("文件", |ui| {
            if ui.button("打开").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.audio_path = path.display().to_string();
                }
            }
            if ui.button("退出").clicked() {
                unsafe {
                    if IDYES
                        == MessageBoxW(
                            None,
                            w!("你真的要退出吗?"),
                            w!("提示"),
                            MB_YESNO | MB_ICONASTERISK,
                        )
                    {
                        frame.close()
                    }
                }
            }
        });
    }

    fn menu_bar_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.horizontal(|ui| {});
    }
}

impl eframe::App for PlayerApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.custom_window_frame(ctx, frame, |ui| {});
    }
}

#[test]
fn foo() {}
