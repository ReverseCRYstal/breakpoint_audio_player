use eframe::egui;

use std::ffi::OsStr;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

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

/// 处理将要打开的文件
pub fn handle_file(src_path: &PathBuf, extract_path: &PathBuf) -> Result<PathBuf, std::io::Error> {
    if let Some(ext) = src_path.extension() {
        match ext.to_str().unwrap_or_default() {
            "bax" => {
                let mut file =
                    zip::ZipArchive::new(std::io::BufReader::new(std::fs::File::open(src_path)?))?;

                file.extract(extract_path.join("/path/"))?;

                Ok(PathBuf::new())
            }
            "mp3" => Ok(src_path.clone()),
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                "Opening the current file is not supported.",
            )),
        }
    } else {
        Err(Error::new(
            ErrorKind::InvalidInput,
            "Opening the current file is not supported.",
        ))
    }
}
