#![allow(dead_code)]
#![allow(unused)]

mod audio_player;
mod widgets;

use audio_player::AudioPlayer;

use eframe::egui;

use egui::vec2;
use egui::Vec2;

// Here are some useful emojis that will be used in the future
// ⏴⏵⏶⏷⏩⏪⏭⏮⏸⏹⏺■▶★☆☐☑↺↻⟲⟳⬅➡⬆⬇⬈⬉⬊⬋⬌⬍⮨⮩⮪⮫⊗✔⛶
// ∞⎗⎘⎙⏏📾🔀🔁🔃☀☁
// ☜☝☞☟⛃  ♡ 📅📆 📈📉📊
pub mod icon_emojis {
    const PAUSE: char = '⏸';
    const RESUME: char = '⏵';
    // Next breakpoint
    const NEXT_BRK_PT: char = '⏩';
    // Previous breakpoint
    const PREV_BRK_PT: char = '⏪';
}

pub const WINDOW_TITLE: &str = "player";

pub struct PlayerApp {
    window_title: String,
    audio_path: String,
    player: AudioPlayer,
    show_play_control_ui: bool,
}

impl PlayerApp {
    pub fn new(cc: &eframe::CreationContext<'_>,file_path: String) -> Self {
        let ctx = &cc.egui_ctx;
        
        PlayerApp {
            player: AudioPlayer::default(),
            audio_path: file_path,
            window_title: WINDOW_TITLE.to_string(),
            show_play_control_ui: false,
        }
    }

    fn window_frame(&mut self) {
        self.title_bar_ui();
        self.menu_bar_ui();

        if self.show_play_control_ui {
            self.play_control_ui();
        }
    }

    fn menu_bar_ui(&mut self) {}
    fn title_bar_ui(&mut self) {}
    fn play_control_ui(&mut self) {}
    fn file_dialog(&mut self) {}
}

impl eframe::App for PlayerApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        custom_window_frame(ctx, frame, &self.window_title, |ui| {});
    }
}

fn close_maximize_minimize(ui: &mut egui::Ui, frame: &mut eframe::Frame) {
    use egui::{Button, RichText};

    let button_height = 12.0;

    let close_response = ui
        .add(Button::new(RichText::new("❌").size(button_height)))
        .on_hover_text("Close the window");
    if close_response.clicked() {
        frame.close();
    }

    if frame.info().window_info.maximized {
        let maximized_response = ui
            .add(Button::new(RichText::new("🗗").size(button_height)))
            .on_hover_text("Restore window");
        if maximized_response.clicked() {
            frame.set_maximized(false);
        }
    } else {
        let maximized_response = ui
            .add(Button::new(RichText::new("🗗").size(button_height)))
            .on_hover_text("Maximize window");
        if maximized_response.clicked() {
            frame.set_maximized(true);
        }
    }

    let minimized_response = ui
        .add(Button::new(RichText::new("🗕").size(button_height)))
        .on_hover_text("Minimize the window");
    if minimized_response.clicked() {
        frame.set_minimized(true);
    }
}

fn title_bar_ui(
    ui: &mut egui::Ui,
    frame: &mut eframe::Frame,
    title_bar_rect: &eframe::epaint::Rect,
    title: &str,
) {
    use egui::*;

    let painter = ui.painter();

    let title_bar_response = ui.interact(*title_bar_rect, Id::new("title_bar"), Sense::click());

    // Paint the title:
    painter.text(
        title_bar_rect.center(),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(20.0),
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
            close_maximize_minimize(ui, frame);
        });
    });
}

fn function_bar_ui(
    ui: &mut egui::Ui,
    frame: &mut eframe::Frame,
    function_bar_rect: &eframe::epaint::Rect,
) {
    use egui::*;

    let painter = ui.painter();

    painter.line_segment(
        [
            function_bar_rect.min + vec2(1.0, 0.0),
            function_bar_rect.right_top() + vec2(0.0, 0.0),
        ],
        ui.visuals().widgets.noninteractive.bg_stroke,
    );
}

fn custom_window_frame(
    ctx: &egui::Context,
    frame: &mut eframe::Frame,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    use egui::*;

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
        title_bar_ui(ui, frame, &title_bar_rect, title);

        let function_bar_height = 64.0;
        let function_bar_rect = {
            let mut rect = app_rect;
            rect.max.y = rect.max.y - function_bar_height;
            rect.min.y = rect.max.y;
            rect
        };
        dbg!(function_bar_rect);
        function_bar_ui(ui, frame, &function_bar_rect);

        // Add the contents:
        let content_rect = {
            let mut rect = app_rect;
            rect.min.y = title_bar_rect.max.y;
            rect
        }
        .shrink(4.0);
        let mut content_ui = ui.child_ui(content_rect, *ui.layout());
        add_contents(&mut content_ui);
    });
}
