use eframe::egui;

use egui::{Button, Context, Ui, Visuals};

use windows::{w, Win32::UI::WindowsAndMessaging};

use WindowsAndMessaging::MessageBoxW;
use WindowsAndMessaging::IDYES;
use WindowsAndMessaging::{MB_ICONASTERISK, MB_YESNO};

#[inline(always)]
pub fn confirm_exit() -> bool {
    unsafe {
        IDYES
            == MessageBoxW(
                None,
                w!("你真的要退出吗?"),
                w!("提示"),
                MB_YESNO | MB_ICONASTERISK,
            )
    }
}

/// From egui
/// Render 'close', 'maximize' and 'minimize' buttons on the title bar
#[inline(always)]
fn close_maximize_minimize(ui: &mut Ui, frame: &mut eframe::Frame, should_confirm_exit: bool) {
    use egui::RichText;

    let button_height = 20.0;

    let close_response = ui
        .add(Button::new(RichText::new("🗙").size(button_height)))
        .on_hover_text("关闭窗口");
    if close_response.clicked() {
        let mut should_close = true;
        if should_confirm_exit {
            should_close = confirm_exit();
        };
        if should_close {
            frame.close();
        }
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
            .add(Button::new(RichText::new("🗖").size(button_height)))
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

fn global_dark_light_mode_switch_localizable(
    ui: &mut Ui,
    on_hover_text_to_light: &str,
    on_hover_text_to_dark: &str,
) {
    if ui.visuals().dark_mode {
        if ui
            .add(Button::new("☀").frame(false))
            .on_hover_text(on_hover_text_to_light)
            .clicked()
        {
            *ui.visuals_mut() = Visuals::light();
        }
    } else if ui
        .add(Button::new("🌙").frame(false))
        .on_hover_text(on_hover_text_to_dark)
        .clicked()
    {
        *ui.visuals_mut() = Visuals::dark();
    };
}

/// From egui
/// Render title bar
#[inline(always)]
fn title_bar_ui(
    ui: &mut Ui,
    frame: &mut eframe::Frame,
    title_bar_rect: &eframe::epaint::Rect,
    title: &str,
    should_confirm_exit: bool,
) {
    use egui::*;

    let title_bar_response = ui.interact(*title_bar_rect, Id::new("title_bar"), Sense::click());

    ui.add_space(8.0);

    global_dark_light_mode_switch_localizable(ui, "切换到白昼模式", "切换到夜间模式");

    let painter = ui.painter();

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
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.visuals_mut().button_frame = false;
            ui.add_space(8.0);
            close_maximize_minimize(ui, frame, should_confirm_exit);
        });
    });
}

pub fn window_frame(
    ctx: &Context,
    frame: &mut eframe::Frame,
    window_title: &str,
    should_confirm_exit: bool,
    add_contents: impl FnOnce(&mut eframe::Frame, &mut Ui),
) {
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
            title_bar_ui(
                ui,
                frame,
                &title_bar_rect,
                window_title,
                should_confirm_exit,
            );

            // Add the contents:
            let content_rect = {
                let mut rect = app_rect;
                rect.min.y = title_bar_rect.max.y;
                rect
            }
            .shrink(4.0);

            let mut content_ui = ui.child_ui(content_rect, *ui.layout());
            add_contents(frame, &mut content_ui);
        });
}
