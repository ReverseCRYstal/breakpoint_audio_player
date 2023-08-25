#![allow(unused)]
#![allow(dead_code)]

use std::rc::Rc;

use eframe::egui;
use eframe::epaint;
use egui::{Rect, Stroke, Ui, Widget};

#[derive(Debug)]
enum Dir2 {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir4 {
    Left,
    Right,
    Bottom,
    Top,
}

impl ToString for Dir4 {
    fn to_string(&self) -> String {
        match self {
            Self::Bottom => String::from("bottom"),
            Self::Left => String::from("left"),
            Self::Right => String::from("right"),
            Self::Top => String::from("top"),
        }
    }
}

impl From<Dir4> for Dir2 {
    fn from(val: Dir4) -> Self {
        match val {
            Dir4::Bottom => Dir2::Vertical,
            Dir4::Top => Dir2::Vertical,
            Dir4::Right => Dir2::Horizontal,
            Dir4::Left => Dir2::Horizontal,
        }
    }
}

pub enum BorderShape {
    LineSegment(Stroke),
    Rect(epaint::RectShape),
}

pub struct Subpanel {
    attachment: Dir4,
    rect: Rc<Rect>,
    border: BorderShape,
    size: Option<f32>,
    contents: Option<Box<dyn FnOnce(&mut Ui)>>,
}

impl Subpanel {
    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }

    pub fn border(mut self, border: Stroke) -> Self {
        self
    }

    pub fn add_contents(mut self, inner_contents: impl FnOnce(&mut Ui)) {}
}

impl Widget for Subpanel {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        todo!("")
    }
}
