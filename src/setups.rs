// ! setups that are usually called before create the window

use eframe::egui;

pub fn setup_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
    "my_font".to_owned(),
    egui::FontData::from_static(include_bytes!(
        "C:\\Users\\Admin\\AppData\\Roaming\\Aseprite\\extensions\\aseprite-theme-pixel\\Zfull-GB.ttf"
    )));

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
