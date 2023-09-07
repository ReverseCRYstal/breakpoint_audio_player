use eframe::egui;

use std::collections::BinaryHeap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

use crate::breakpoint::Breakpoint;

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

/// # Basically
/// A file with a extension name bax
/// is a zip archive
/// # Structure of the content of bax file
/// audio.mp3 \
/// config.json\
/// extension.json
///
/// `audio.mp3` is the file which is going to be played \
/// `config.json` is the file stored the configuration of bax file,\
/// such as the attachments of breakpoints, the version of audio player, etc.
// /// `extension.json` is the file containing the extended setting for audio player.\
// /// it haven't been implemented yet.
///
/// # Return
/// returns the root path of extracted files
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

    let bad_content_err = |detail: String| -> anyhow::Error {
        anyhow::anyhow!("Bad Content Error: Failed to parse the json file, {detail}")
    };
    if let Value::Object(value) =
        serde_json::from_reader(BufReader::new(File::open(root.join("config.json"))?))?
    {
        for breakpoint in value
            .get("breakpoints")
            .ok_or(bad_content_err("missing object: 'breakpoints'".to_owned()))?
            .as_array()
            .ok_or(bad_content_err(
                "the type of the object 'breakpoints' should be an array".to_owned(),
            ))?
            .iter()
        {
            breakpoints.push(serde_json::from_value(breakpoint.clone())?);
        }
        Ok(breakpoints)
    } else {
        Err(bad_content_err(
            "the whole file was supposed to be treated as a JSON object".to_owned(),
        ))
    }
}
