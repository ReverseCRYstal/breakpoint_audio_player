use std::collections::HashMap;

use eframe::egui;

#[derive(Default)]
pub struct Guardian {
    states: HashMap<String, bool>,
}

impl Guardian {
    pub fn create_window(&mut self, title: impl ToString, default_status: bool) -> egui::Window {
        self.create_window_by_hash(title.to_string(), title, default_status)
    }

    pub fn create_window_by_hash(
        &mut self,
        title: impl ToString,
        hash: impl ToString,
        default_status: bool,
    ) -> egui::Window {
        let entry = self.states.entry(hash.to_string());
        egui::Window::new(title.to_string()).open(entry.or_insert(default_status))
    }

    pub fn set_window_status(&mut self, title: impl ToString, status: bool) {
        self.states.insert(title.to_string(), status);
    }
}
