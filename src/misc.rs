use eframe::egui;

#[inline]
pub fn setup_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "icon_font".to_owned(),
        egui::FontData::from_static(include_bytes!(".\\..\\assets\\Symbola.ttf")),
    );

    fonts
        .families
        .entry(egui::FontFamily::Name("icon_font".into()))
        .or_default()
        .insert(0, "icon_font".to_owned());

    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!(".\\..\\assets\\simhei.ttf")),
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
