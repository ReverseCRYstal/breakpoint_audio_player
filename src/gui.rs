use eframe::egui;

use egui::{Button, Ui, Visuals};

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
pub fn close_maximize_minimize(ui: &mut Ui, frame: &mut eframe::Frame) {
    use egui::RichText;

    let button_height = 20.0;

    let close_response = ui
        .add(Button::new(
            RichText::new("🗙")
                .family(FontFamily::Name("icon_font".into()))
                .size(button_height),
        ))
        .on_hover_text("关闭窗口");
    if close_response.clicked() {
        frame.close();
    }
    use egui::FontFamily;

    if frame.info().window_info.maximized {
        let maximized_response = ui
            .add(Button::new(
                RichText::new("🗗")
                    .family(FontFamily::Name("icon_font".into()))
                    .size(button_height),
            ))
            .on_hover_text("恢复窗口");
        if maximized_response.clicked() {
            frame.set_maximized(false);
        }
    } else {
        let maximized_response = ui
            .add(Button::new(
                RichText::new("🗖")
                    .family(FontFamily::Name("icon_font".into()))
                    .size(button_height),
            ))
            .on_hover_text("最大化窗口");
        if maximized_response.clicked() {
            frame.set_maximized(true);
        }
    }

    let minimized_response = ui
        .add(Button::new(RichText::new("▁").size(button_height)))
        .on_hover_text("最小化窗口");
    if minimized_response.clicked() {
        frame.set_minimized(true);
    }
}

pub fn global_dark_light_mode_switch_localizable(
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
            ui.ctx().set_visuals(Visuals::light());
        }
    } else if ui
        .add(Button::new("🌙").frame(false))
        .on_hover_text(on_hover_text_to_dark)
        .clicked()
    {
        ui.ctx().set_visuals(Visuals::dark())
    };
}
