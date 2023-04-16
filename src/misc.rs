//! The functions implemented in this file are generally not called many times

use eframe::egui;

use windows::{w, Win32::UI::WindowsAndMessaging};

use WindowsAndMessaging::MessageBoxW;
use WindowsAndMessaging::IDYES;
use WindowsAndMessaging::{MB_ICONASTERISK, MB_YESNO};

#[inline(always)]
pub fn confirm_exit(frame: &mut eframe::Frame) {
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

/// From egui
/// Render 'close', 'maximize' and 'minimize' buttons on the title bar
#[inline(always)]
fn close_maximize_minimize(ui: &mut egui::Ui, frame: &mut eframe::Frame) {
    use egui::{Button, RichText};

    let button_height = 12.0;

    let close_response = ui
        .add(Button::new(RichText::new("❌").size(button_height)))
        .on_hover_text("关闭窗口");
    if close_response.clicked() {
        confirm_exit(frame);
    }

    if frame.info().window_info.maximized {
        let maximized_response = ui
            .add(Button::new(RichText::new("🗗").size(button_height)))
            .on_hover_text("恢复窗口");
        if maximized_response.clicked() {
            frame.set_maximized(false);
        }
    } else {
        let maximized_response = ui
            .add(Button::new(RichText::new("🗗").size(button_height)))
            .on_hover_text("最大化窗口");
        if maximized_response.clicked() {
            frame.set_maximized(true);
        }
    }

    let minimized_response = ui
        .add(Button::new(RichText::new("🗕").size(button_height)))
        .on_hover_text("最小化窗口");
    if minimized_response.clicked() {
        frame.set_minimized(true);
    }
}

/// From egui
/// Render title bar
#[inline(always)]
pub fn title_bar_ui(
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

#[inline(always)]
pub fn setup_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!(".\\..\\assests\\Zfull-GB.ttf")),
    );

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("my_font".to_owned());

    ctx.set_fonts(fonts);
}
