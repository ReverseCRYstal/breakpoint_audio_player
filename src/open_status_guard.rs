use std::collections::HashMap;

use eframe::egui;

#[derive(Default)]
pub struct Guardian {
    states: HashMap<String, bool>,
}

impl Guardian {
    pub fn create_window(&mut self, title: impl ToString, default_status: bool) -> egui::Window {
        let title = title.to_string();
        let entry = self.states.entry(title.clone());
        egui::Window::new(title).open(entry.or_insert(default_status))
    }

    pub fn set_window_status(&mut self, title: impl ToString, status: bool) {
        self.states.insert(title.to_string(), status);
    }
}
