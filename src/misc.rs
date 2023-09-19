use eframe::egui;

use egui::{Button, RichText};

use std::collections::BinaryHeap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

use crate::app::FileCategory;
use crate::audio_player::SingletonPlayer;
use crate::breakpoint::Breakpoint;

pub type OpenResult = Result<(BinaryHeap<Breakpoint>, FileCategory), anyhow::Error>;

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

// #[inline(always)]
// pub fn form_icon(text: impl Into<String>) -> RichText {
// }

#[inline(always)]
pub fn form_button(text: impl Into<String>) -> Button {
    Button::new(
        RichText::new(text)
            .family(egui::FontFamily::Name("icon_font".into()))
            .size(17.5),
    )
    .rounding(25.)
    .min_size([50., 50.].into())
}

pub fn unzip(source_path: &Path, extract_path: &Path) -> Result<PathBuf, anyhow::Error> {
    let mut archive = zip::ZipArchive::new(BufReader::new(File::open(source_path)?))?;
    let mut ret: Result<PathBuf, anyhow::Error> = Err(anyhow::anyhow!(
        "The archived file do not meet specifications."
    ));

    let iter = archive
        .file_names()
        .map(|v| v.to_string())
        .collect::<Vec<_>>();

    for name in &iter {
        ret = || -> Result<PathBuf, anyhow::Error> {
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(extract_path.join(name))?;

            f.write_all(archive.by_name(name)?.extra_data())?;

            let name = PathBuf::from(name);

            if name
                .extension()
                .unwrap()
                .to_str()
                .is_some_and(|v| v.eq_ignore_ascii_case("mp3"))
            {
                Ok(extract_path.join(name))
            } else {
                Err(anyhow::anyhow!(
                    "The archived file do not meet specifications."
                ))
            }
        }();
    }

    ret
}

pub fn handle_config(root: &Path) -> Result<BinaryHeap<Breakpoint>, anyhow::Error> {
    use serde_json::Value;
    let mut breakpoints = BinaryHeap::new();

    let bad_content_err =
        || -> anyhow::Error { anyhow::anyhow!("文件损坏，无法解析JSON文本") };
    if let Value::Object(value) =
        serde_json::from_reader(BufReader::new(File::open(root.join("config.json"))?))?
    {
        for breakpoint in value
            .get("breakpoints")
            .ok_or(bad_content_err())?
            .as_array()
            .ok_or(bad_content_err())?
            .iter()
        {
            breakpoints.push(serde_json::from_value(breakpoint.clone())?);
        }
        Ok(breakpoints)
    } else {
        Err(bad_content_err())
    }
}

pub fn open(
    p: &Path,
    player: &mut SingletonPlayer,
) -> Result<(BinaryHeap<Breakpoint>, FileCategory), anyhow::Error> {
    let mut breakpoints = BinaryHeap::new();
    let mut file_category = FileCategory::Nil;

    let unsupported_err = || -> anyhow::Error {
        anyhow::anyhow!("本软件不支持打开拥有该扩展名的文件")
    };

    let extension = p
        .extension()
        .map(|s| s.to_str())
        .ok_or(unsupported_err())?
        .ok_or(unsupported_err())?;

    let p = match extension {
        crate::constants::literal::EXTENSION_NAME => {
            let root = unzip(p, &std::env::current_dir()?)?;
            breakpoints = handle_config(&root)?;
            file_category = FileCategory::Bax;
            Ok(root.join("audio.mp3"))
        }
        "mp3" => {
            file_category = FileCategory::Mp3;

            Ok(p.to_owned())
        }
        _ => Err(unsupported_err()),
    }?;

    player.replace_file(BufReader::new(File::open(p)?))?;

    Ok((breakpoints, file_category))
}

pub fn secs_to_string(duration: u64) -> String {
    let seconds = duration % 60;
    let mut minutes = duration / 60;
    let total_hours = minutes / 60;
    minutes %= 60;
    let hours = if total_hours != 0 {
        format!("{total_hours}:")
    } else {
        String::new()
    };
    format!("{hours}{minutes:0>2}:{seconds:0>2}")
}

pub fn string_to_secs(string: &str) -> Option<u64> {
    let mut ret = 0;
    for (gap, literal_duration) in string.rsplit(':').enumerate() {
        if let Ok(n) = literal_duration.parse::<u64>() {
            ret += 60_u64.pow(gap as _) * n;
        } else {
            return None;
        }
    }
    Some(ret)
}
