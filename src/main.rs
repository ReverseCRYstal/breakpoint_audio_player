#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use breakpoint_audio_player::window_options::get;
use breakpoint_audio_player::*;

fn main() -> Result<(), eframe::Error> {
    let result = eframe::run_native(
        WINDOW_TITLE,
        get(),
        Box::new(|cc| Box::new(PlayerApp::new(cc, ".\\assests\\example_audio.mp3".into()))),
    );

    // if result.is_err() {
    //     println!("{}", result.unwrap_err());
    // }
    result
}
