use eframe::egui;
use egui::{Context, Pos2, Rect, Separator, Stroke, Ui};

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

pub struct PanelSpawner<'a> {
    ui: &'a mut Ui,
    rect: Rect,
}

impl<'a> PanelSpawner<'a> {
    pub fn new(ui: &'a mut Ui) -> PanelSpawner {
        let rect = ui.available_rect_before_wrap();
        Self { ui, rect }
    }

    pub fn allocate_auto(&mut self, direction: Dir4, border_shrinking: Option<f32>) -> InnerPanel {
        unimplemented!()
    }

    pub fn allocate(
        &mut self,
        width: f32,
        direction: Dir4,
        border_shrinking: Option<f32>,
    ) -> InnerPanel {
        if border_shrinking.is_some_and(|v| v.is_sign_negative()) {
            panic!("The value of border_shrinking.unwrap() shouldn't be less than 0.0");
        }

        let (allocated_rect, extra_rect, border) = {
            let mut allocated = self.rect;
            let mut extra = self.rect;

            let points: [Pos2; 2];
            let shrinking = border_shrinking.unwrap_or_default();

            match direction {
                Dir4::Left => {
                    allocated.max.x = width;
                    points = [allocated.right_top(), allocated.right_bottom()];
                    allocated.max.x -= shrinking;
                    extra.min.x += width;
                }
                Dir4::Bottom => {
                    allocated.min.y = allocated.max.y - width;
                    points = [allocated.left_top(), allocated.right_top()];
                    allocated.min.y += shrinking;
                    extra.max.y -= width;
                }
                Dir4::Right => {
                    allocated.min.x = allocated.max.x - width;
                    points = [allocated.left_top(), allocated.left_bottom()];
                    allocated.min.x += shrinking;
                    extra.max.x -= width;
                }

                Dir4::Top => {
                    allocated.max.y = width;
                    points = [allocated.left_bottom(), allocated.right_bottom()];
                    allocated.max.y -= shrinking;
                    extra.min.y += width;
                }
            };

            (allocated, extra, points)
        };
        self.rect = extra_rect;

        let child_ui = self.ui.child_ui(allocated_rect, *self.ui.layout());

        InnerPanel {
            border,
            direction,
            rect: allocated_rect,
            ui: child_ui,
            separating: Separating::None,
        }
    }

    pub fn shrink(mut self, amnt: f32) -> Self {
        self.rect = self.rect.shrink(amnt);
        self
    }

    pub fn show(self, add_contents: impl FnOnce(&mut Ui)) {
        let mut ui = self.ui.child_ui(self.rect, *self.ui.layout());
        add_contents(&mut ui);
    }
}

pub enum Separating {
    Line(Stroke),
    Separator(Separator),
    None,
}

impl From<Option<Stroke>> for Separating {
    fn from(value: Option<egui::Stroke>) -> Self {
        match value {
            Some(stroke) => Separating::Line(stroke),
            None => Self::None,
        }
    }
}

impl From<Option<Separator>> for Separating {
    fn from(value: Option<Separator>) -> Self {
        match value {
            Some(separator) => Separating::Separator(separator),
            None => Self::None,
        }
    }
}

pub struct InnerPanel {
    rect: Rect,
    direction: Dir4,
    ui: Ui,
    separating: Separating,
    border: [Pos2; 2],
}

impl InnerPanel {
    pub fn separator(mut self) -> Self {
        let matched = match self.direction.into() {
            Dir2::Horizontal => egui::Separator::default().vertical(),
            Dir2::Vertical => egui::Separator::default().horizontal(),
        };

        self.separating = Separating::Separator(matched);

        self
    }

    pub fn separating_line(mut self, stroke: Stroke) -> Self {
        self.separating = Separating::Line(stroke);
        self
    }

    pub fn show(mut self, _ctx: &Context, add_contents: impl FnOnce(&mut Ui)) {
        let rect = self.rect;

        match self.separating {
            Separating::Line(stroke) => {
                self.ui.painter().line_segment(self.border, stroke);
            }
            Separating::Separator(separator) => {
                self.ui.put(self.border.into(), separator);
            }
            _ => {}
        }
        self.ui.allocate_ui_at_rect(rect, add_contents);
    }
}
