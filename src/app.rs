use crate::audio_player::SingletonPlayer;
use crate::breakpoint::Breakpoint;
use crate::constants;
use crate::constants::toasts::DUR;
use crate::gui;
use crate::misc::{self, icon_button, secs_to_string};
use crate::open_status_guard::Guardian;

use std::collections::{BinaryHeap, VecDeque};
use std::fs::{copy, File};

use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::Duration;

use eframe::egui;
use eframe::epaint::vec2;
use egui::{Button, Context, RichText, Rounding, Slider, Ui};

use egui_notify::Anchor;

type ErrResult = Result<(), anyhow::Error>;

pub enum FileCategory {
    Mp3,
    Bax,
    Nil,
}

impl FileCategory {
    pub fn is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }
}

#[derive(Clone)]
enum Action {
    Remove(Breakpoint),
    Add(Breakpoint),
    ClearAll(BinaryHeap<Breakpoint>),
}

pub struct App {
    action_queue: VecDeque<Action>,
    next_breakpoint: Option<Breakpoint>,
    prev_breakpoint: Option<Breakpoint>,
    queue_size: u16,
    current_queue_idx: usize,
    file_path: Option<PathBuf>,
    file_category: FileCategory,
    toasts: egui_notify::Toasts,
    breakpoints: BinaryHeap<Breakpoint>,
    player: SingletonPlayer,
    is_muted: bool,
    should_confirm_exit: bool,
    hint_to_be_added: Option<String>,
    timepoint_to_be_added: Option<Duration>,
    bp_to_be_added: Option<Breakpoint>,
    volume: u8,
    progress_buffer: u64,
    open_stat_guardian: Guardian,
    cur_speed_enum_idx: usize,
    temp_dir: PathBuf,
    changed: bool,
    adjusted: bool,
}

// construction
impl App {
    #[inline(always)]
    pub fn new(file_to_open: Option<PathBuf>, temp_dir: PathBuf) -> Self {
        let mut player = SingletonPlayer::new();
        let mut toasts = egui_notify::Toasts::default()
            .with_anchor(Anchor::TopRight)
            .with_margin(vec2(3., 32.));
        let mut file_path = None;

        player.pause();

        let (breakpoints, file_category) = || -> (BinaryHeap<Breakpoint>, FileCategory) {
            file_to_open
                .and_then(|v| {
                    file_path = Some(v.clone());

                    misc::open(&v, &temp_dir, &mut player)
                        .map_err(|caption| {
                            toasts.error(caption.to_string()).set_duration(Some(DUR));
                        })
                        .ok()
                })
                .unwrap_or((BinaryHeap::new(), FileCategory::Nil))
        }();

        Self {
            file_path,
            file_category,
            toasts,
            player,
            breakpoints,
            volume: 100,
            changed: false,
            adjusted: false,
            is_muted: false,
            current_queue_idx: 0,
            bp_to_be_added: None,
            timepoint_to_be_added: None,
            hint_to_be_added: None,
            next_breakpoint: None,
            prev_breakpoint: None,
            cur_speed_enum_idx: 2,
            should_confirm_exit: false,
            queue_size: u8::MAX as u16,
            action_queue: VecDeque::new(),
            progress_buffer: 0,
            open_stat_guardian: Default::default(),
            temp_dir,
        }
    }
}

// file io
impl App {
    fn open(&mut self, path: &Path) -> misc::OpenResult {
        self.file_path = Some(path.to_owned());
        let ret = misc::open(path, &self.temp_dir, &mut self.player);
        copy(path, self.temp_dir.join("audio.mp3"))?;
        ret
    }

    fn save_as_mp3(&self, path: &Path) -> ErrResult {
        copy(self.temp_dir.join("audio.mp3"), path)?;
        Ok(())
    }

    fn save_as_bax(&self, path: &Path) -> ErrResult {
        use std::io::prelude::*;
        use zip::write::FileOptions;

        let options = FileOptions::default()
            .unix_permissions(0o755)
            .compression_method(zip::CompressionMethod::Stored);

        let mut data = serde_json::Map::new();
        data.insert(
            "breakpoints".to_owned(),
            serde_json::to_value(&self.breakpoints)?,
        );
        let mut zip = zip::ZipWriter::new(std::fs::File::create(path)?);

        zip.start_file("config.json", options)?;
        zip.write_all(serde_json::to_string(&serde_json::Value::Object(data))?.as_bytes())?;
        zip.start_file("audio.mp3", options)?;
        zip.write_all(
            std::io::read_to_string(BufReader::new(File::open(&self.temp_dir)?))?.as_bytes(),
        )?;
        zip.finish()?;

        Ok(())
    }
}

impl App {
    fn undo(&mut self) {
        self.current_queue_idx -= 1;
        match &self.action_queue[self.current_queue_idx] {
            Action::Remove(bp) => self.breakpoints.push(bp.clone()),
            Action::Add(bp) => {
                self.breakpoints.retain(|v| *v != *bp);
            }
            Action::ClearAll(all) => {
                self.breakpoints.clone_from(all);
            }
        }
    }

    fn redo(&mut self) {
        match &self.action_queue[self.current_queue_idx] {
            Action::Remove(bp) => {
                self.breakpoints.retain(|v: &Breakpoint| *v != *bp);
            }
            Action::Add(bp) => {
                self.breakpoints.push(bp.clone());
            }
            Action::ClearAll(_) => {
                self.breakpoints.clear();
            }
        }
        self.current_queue_idx += 1;
    }
}

impl App {
    #[inline(always)]
    fn title_bar_ui(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        frame: &mut eframe::Frame,
        title_bar_rect: &eframe::epaint::Rect,
        title: &str,
    ) {
        ui.scope(|ui| {
            {
                let visual_mut = ui.visuals_mut();
                let bg_fill = visual_mut.noninteractive().bg_fill;
                visual_mut.widgets.inactive.weak_bg_fill = bg_fill;

                let title_bar_response = ui.interact(
                    *title_bar_rect,
                    egui::Id::new("title_bar"),
                    egui::Sense::click(),
                );

                ui.add_space(7.0);

                ui.horizontal(|ui| {
                    gui::global_dark_light_mode_switch_localizable(
                        ui,
                        "切换到白昼模式",
                        "切换到夜间模式",
                    );

                    ui.menu_button(RichText::new("文件"), |ui| {
                        if let Err(caption) = self.file_menu_ui(ui, frame) {
                            self.toasts
                                .error(caption.to_string())
                                .set_duration(Some(DUR));
                        }
                    });

                    ui.menu_button("编辑", |ui| {
                        if ui
                            .add_enabled(self.current_queue_idx != 0, Button::new("撤销"))
                            .clicked()
                        {
                            self.undo();
                        }

                        if ui
                            .add_enabled(
                                self.current_queue_idx != self.action_queue.len(),
                                Button::new("重做"),
                            )
                            .clicked()
                        {
                            self.redo();
                        }
                    });

                    ui.menu_button("断点", |ui| {
                        self.breakpoint_menu_ui(ui);
                    });

                    ui.menu_button("帮助", |ui| {
                        self.help_menu_ui(ui);
                    });

                    self.toasts.show(ctx);
                });

                let painter = ui.painter();

                // Paint the title:
                painter.text(
                    title_bar_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    title,
                    egui::FontId::proportional(20.0),
                    ui.style().visuals.text_color(),
                );

                // Paint the line under the title:
                painter.line_segment(
                    [
                        title_bar_rect.left_bottom() + vec2(1.0, 0.0),
                        title_bar_rect.right_bottom(),
                    ],
                    ui.visuals().widgets.noninteractive.bg_stroke,
                );

                // Interact with the title bar (drag to move window):
                if title_bar_response.double_clicked() {
                    frame.set_maximized(!frame.info().window_info.maximized);
                } else if title_bar_response.is_pointer_button_down_on() {
                    frame.drag_window();
                }

                ui.allocate_ui_at_rect(*title_bar_rect, |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.visuals_mut().button_frame = false;
                        ui.add_space(8.0);
                        gui::close_maximize_minimize(ui, frame);
                    });
                });
            }
        });
    }
}

// playback ui
impl App {
    #[inline(always)]
    fn progress_slider(&mut self, ui: &mut Ui) {
        let mut place_holder = 0;

        let play_progress = self.player.get_progress();
        ui.scope(|ui| {
            let spacing = ui.clip_rect().width() - 288.;
            let spacing_mut = &mut ui.spacing_mut().slider_width;
            *spacing_mut = spacing;
            let resp =
                if let Some(total_duration) = self.player.total_duration().map(|v| v.as_secs()) {
                    *spacing_mut -= (total_duration.to_string().len() * 2 + 3) as f32 * 8.;

                    ui.add_enabled(
                        true,
                        Slider::new(&mut self.progress_buffer, 0..=total_duration)
                            .trailing_fill(true)
                            .custom_formatter(|v, _| {
                                format!(
                                    "{:} / {:}",
                                    secs_to_string(v as u64),
                                    secs_to_string(total_duration)
                                )
                            })
                            .custom_parser(|v| misc::string_to_secs(v).map(|v| v as f64))
                            .show_value(true),
                    )
                } else {
                    ui.add_enabled(
                        false,
                        Slider::new(&mut place_holder, 0..=1)
                            .show_value(false)
                            .text("00:00"),
                    )
                };

            if resp.drag_released() {
                self.player
                    .set_progress(Duration::from_secs(self.progress_buffer));
            } else {
                self.progress_buffer = play_progress.as_secs();
            }
        });
    }

    #[inline(always)]
    fn speed_control(&mut self, ui: &mut Ui) {
        ui.menu_button("播放倍速", |ui| {
            for (index, text) in ["0.5×", "0.75×", "正常", "1.25×", "1.5×"]
                .iter()
                .enumerate()
            {
                let text = if self.cur_speed_enum_idx == index {
                    String::from("✔")
                } else {
                    String::new()
                } + *text;

                if ui.button(text).clicked() {
                    self.cur_speed_enum_idx = index;
                    self.player.set_speed(0.5 + index as f32 * 0.25);
                    ui.close_menu();
                    break;
                }
            }
        });
    }

    #[inline(always)]
    fn breakpoint_switch_buttons(&mut self, ui: &mut Ui) {
        use constants::icon::{NEXT_BRK_PT, PREV_BRK_PT};

        if ui
            .add_enabled(self.prev_breakpoint.is_some(), icon_button(PREV_BRK_PT))
            .clicked()
        {
            self.player
                .set_progress(self.prev_breakpoint.as_ref().unwrap().timepoint());
        }
        if ui
            .add_enabled(self.next_breakpoint.is_some(), icon_button(NEXT_BRK_PT))
            .clicked()
        {
            self.player
                .set_progress(self.next_breakpoint.as_ref().unwrap().timepoint());
        }
    }

    #[inline(always)]
    fn play_control_buttons(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let paused = self.player.is_paused();

            let play_control_icon = if paused {
                constants::icon::RESUME
            } else {
                constants::icon::PAUSE
            };

            if ui
                .add_enabled(
                    !self.player.is_empty() || !self.player.get_progress().is_zero(),
                    icon_button(constants::icon::RESET),
                )
                .clicked()
            {
                self.player.reset();
            }

            if ui
                .add_enabled(!self.player.is_empty(), icon_button(play_control_icon))
                .on_hover_text(if paused { "恢复" } else { "暂停" })
                .clicked()
            {
                if paused {
                    self.player.resume();
                } else {
                    self.player.pause();
                }
            }
        });
    }
}

// menubar
impl App {
    #[inline(always)]
    fn file_menu_ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) -> ErrResult {
        if ui.button("打开").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("打开")
                .add_filter("文件", &["bax", "mp3"])
                .pick_file()
            {
                (self.breakpoints, self.file_category) = self.open(&path)?;
                self.changed = false;
            }
            ui.close_menu();
        }

        if ui.add_enabled(self.changed, Button::new("保存")).clicked()
            && !self.breakpoints.is_empty()
        {
            if !self.breakpoints.is_empty() {
                self.file_category = FileCategory::Bax;
            }

            match self.file_category {
                FileCategory::Mp3 => {
                    self.save_as_mp3(self.file_path.as_ref().unwrap())?;
                }
                FileCategory::Bax => {
                    self.save_as_bax(self.file_path.as_ref().unwrap())?;
                }
                _ => {
                    unreachable!()
                }
            }
            self.changed = false;
            ui.close_menu();
        }

        if ui
            .add_enabled(self.file_path.is_some(), Button::new("另存为"))
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new()
                .set_directory(self.file_path.as_ref().unwrap())
                .set_title("另存为")
                .add_filter("音频文件", &["bax", "mp3"])
                .save_file()
            {
                let extension = path.extension().unwrap().to_ascii_lowercase();
                let extension = extension.to_str().unwrap();
                match extension {
                    "mp3" => {
                        self.save_as_mp3(&path)?;
                    }
                    "bax" => {
                        self.save_as_bax(&path)?;
                    }
                    _ => {
                        return Err(anyhow::anyhow!("不支持另存为该格式"));
                    }
                }
                self.changed = false;
            }
            ui.close_menu();
        }

        ui.separator();
        if ui
            .add_enabled(!self.file_category.is_nil(), Button::new("退出文件"))
            .clicked()
        {
            self.file_path = None;
            self.action_queue.clear();
            self.next_breakpoint = None;
            self.prev_breakpoint = None;
            self.player.clear();
            self.breakpoints.clear();
            self.current_queue_idx = 0;
            self.changed = false;
            ui.close_menu();
        }
        ui.separator();

        if ui.button("退出").clicked() {
            frame.close()
        }
        Ok(())
    }

    #[inline(always)]
    fn help_menu_ui(&mut self, ui: &mut Ui) {
        if ui.button("教程").clicked() {
            self.open_stat_guardian.set_window_status("教程", true);
            ui.close_menu();
        }
        if ui.button("常见问题").clicked() {
            self.open_stat_guardian.set_window_status("常见问题", true);
            ui.close_menu();
        }
        if ui.button("关于").clicked() {
            self.open_stat_guardian.set_window_status("关于", true);
            ui.close_menu();
        }
    }

    #[inline(always)]
    fn breakpoint_menu_ui(&mut self, ui: &mut Ui) {
        if ui
            .add_enabled(
                self.file_path.is_some(),
                Button::new("在当前播放位置添加断点"),
            )
            .clicked()
        {
            self.timepoint_to_be_added = Some(self.player.get_progress());
            self.open_stat_guardian
                .set_window_status("添加断点（给定时间）", true);
            ui.close_menu();
        }

        // 608
        if ui
            .add_enabled(!self.breakpoints.is_empty(), Button::new("删除最近的断点"))
            .clicked()
        {
            let bp = match self
                .next_breakpoint
                .as_ref()
                .zip(self.prev_breakpoint.as_ref())
            {
                None => self
                    .next_breakpoint
                    .as_ref()
                    .or(self.prev_breakpoint.as_ref())
                    .unwrap(),
                Some((next, prev)) => {
                    let progress = self.player.get_progress().as_millis();
                    if next.timepoint().as_millis() - progress
                        > progress - prev.timepoint().as_millis()
                    {
                        prev
                    } else {
                        next
                    }
                }
            };
            self.breakpoints.retain(|v| bp == v);
            self.action_queue
                .push_front(Action::Remove(dbg!(bp.clone())));

            self.adjusted = true;
        }
        if ui
            .add_enabled(!self.breakpoints.is_empty(), Button::new("删除所有断点"))
            .clicked()
        {
            self.breakpoints.clear();
            self.action_queue
                .push_front(Action::ClearAll(self.breakpoints.clone()));
            self.adjusted = true;
        }

        ui.separator();

        if ui
            .add_enabled(
                self.prev_breakpoint.is_some(),
                Button::new("跳转至上一断点"),
            )
            .clicked()
        {
            self.player
                .set_progress(self.prev_breakpoint.as_ref().unwrap().timepoint());
        }

        if ui
            .add_enabled(
                self.prev_breakpoint.is_some(),
                Button::new("跳转至下一断点"),
            )
            .clicked()
        {
            self.player
                .set_progress(self.prev_breakpoint.as_ref().unwrap().timepoint());
        }
    }
}

impl App {
    #[inline(always)]
    fn bottom_panel(&mut self, panel_frame: egui::Frame, ctx: &Context) {
        let panel_frame = {
            let mut panel_frame = panel_frame;
            panel_frame.rounding.ne = 0.;
            panel_frame.rounding.nw = 0.;
            panel_frame
        };

        egui::TopBottomPanel::bottom("bottom_panel")
            .show_separator_line(false)
            .frame(panel_frame)
            .show(ctx, |ui| {
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.add_space(5.0);
                    self.play_control_buttons(ui);
                    self.progress_slider(ui);
                    self.breakpoint_switch_buttons(ui);
                });
                ui.add_space(5.0);
            });
    }

    fn scroll_area(&mut self, ui: &mut Ui, rest_rect: egui::Rect) {
        egui::ScrollArea::horizontal()
            .max_width(f32::INFINITY)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if let Some(bp) = self.bp_to_be_added.take() {
                    self.breakpoints.push(bp.clone());
                    self.action_queue.push_front(Action::Add(bp));
                }

                if self.breakpoints.is_empty() {
                    ui.allocate_ui_at_rect(
                        egui::Rect::from_center_size(rest_rect.center(), rest_rect.size() / 2.),
                        |ui| {
                            ui.vertical_centered(|ui| {
                                ui.heading("这个文件还没有添加任何的断点！");
                            });
                        },
                    );
                } else {
                    ui.add_space(ui.max_rect().height() / 2.);
                    ui.horizontal(|ui| {
                        let mut do_assign = true;
                        let progress = self.player.get_progress();

                        self.next_breakpoint = None;
                        self.prev_breakpoint = None;

                        for breakpoint in self.breakpoints.iter().rev() {
                            if progress < breakpoint.timepoint() {
                                self.next_breakpoint = Some(breakpoint.clone());
                            } else if do_assign {
                                self.prev_breakpoint = Some(breakpoint.clone());
                                do_assign = false;
                            }
                            let resp = ui.add(Button::new(secs_to_string(
                                breakpoint.timepoint().as_secs(),
                            )));
                            if resp.clicked() {
                                self.player.set_progress(breakpoint.timepoint());
                            }
                            resp.on_hover_text(breakpoint.hint());
                        }
                    });
                    ui.add_space(ui.max_rect().height() / 2. - 20.);
                }
            });
    }
}

impl App {
    fn render_windows(&mut self, ctx: &Context, center_pos: egui::Pos2) {
        let collapsible = false;
        let resizable = false;
        let default_status = false;
        self.open_stat_guardian
            .create_window("关于", default_status)
            .collapsible(collapsible)
            .resizable(resizable)
            .default_pos(center_pos)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.heading(
                            egui::special_emojis::OS_WINDOWS.to_string()
                                + constants::literal::APP_NAME,
                        );
                    });
                    ui.label("提交:".to_owned() + env!("COMMIT_HASH"));
                    ui.label("构建工具链:".to_owned() + env!("BUILD_TOOLCHAIN"));
                    ui.label("rust版本:".to_owned() + env!("RUST_EDITION"));
                    ui.label("构建时间:".to_owned() + env!("BUILD_TIME"));
                });
            });

        self.open_stat_guardian
            .create_window("教程", default_status)
            .collapsible(collapsible)
            .resizable(resizable)
            .default_pos(center_pos)
            .show(ctx, |ui| {
                ui.label("将光标悬停在控件上以查看功能");
                ui.label("右键（长按）打开菜单，以调节播放速度和音量");
                ui.label("如果使用体验不好，请查看 帮助 > 常见问题");
            });

        self.open_stat_guardian
            .create_window("常见问题", default_status)
            .collapsible(collapsible)
            .resizable(resizable)
            .default_pos(center_pos)
            .show(ctx, |ui|{
                ui.heading("Q:为什么调整播放进度的时候会卡顿？");
                ui.horizontal(|ui|{
                 ui.label("A:这是因为rodio库没有实现调整播放进度的功能，只能通过调用Source::skip_duration()来实现");
                 ui.hyperlink("https://github.com/RustAudio/rodio/issues/443");
            });
        });

        let should_add = self
            .open_stat_guardian
            .create_window("添加断点（给定时间）", default_status)
            .collapsible(collapsible)
            .resizable(resizable)
            .show(ctx, |ui| {
                if self.hint_to_be_added.is_none() {
                    self.hint_to_be_added = Some(String::new())
                }

                egui::TextEdit::multiline(self.hint_to_be_added.as_mut().unwrap())
                    .hint_text("点此添加断点提示信息……")
                    .char_limit(64)
                    .show(ui);

                ui.vertical_centered(|ui| ui.button("添加").clicked()).inner
            })
            .is_some_and(|v| v.inner.is_some_and(|v| v));

        if should_add {
            self.open_stat_guardian
                .set_window_status("添加断点（给定时间）", false);
            self.bp_to_be_added = Some(Breakpoint::new(
                self.timepoint_to_be_added.unwrap(),
                self.hint_to_be_added.take().unwrap(),
            ));
            self.adjusted = true;
            self.action_queue
                .push_front(Action::Add(self.bp_to_be_added.as_ref().unwrap().clone()));
        }
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn on_close_event(&mut self) -> bool {
        if self.should_confirm_exit {
            gui::confirm_exit()
        } else {
            true
        }
    }

    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        if !(self.player.is_empty() || self.player.is_paused()) {
            ctx.request_repaint();
        }

        let mut rounding = {
            let mut r: Rounding = 10.0.into();
            r.ne = 0.;
            r.nw = 0.;
            r
        };
        let panel_frame = egui::Frame {
            rounding,
            fill: ctx.style().visuals.window_fill(),
            stroke: ctx.style().visuals.widgets.noninteractive.fg_stroke,
            outer_margin: 0.5.into(), // so the stroke is within the bounds
            ..Default::default()
        };

        self.bottom_panel(panel_frame, ctx);

        rounding = {
            let mut r: Rounding = 10.0.into();
            r.se = 0.;
            r.sw = 0.;
            r
        };

        egui::CentralPanel::default()
            .frame(panel_frame.rounding(rounding))
            .show(ctx, |ui| {
                let app_rect = ui.max_rect();
                let title_bar_height = 32.0;
                let title_bar_rect = {
                    let mut rect = app_rect;
                    rect.max.y = rect.min.y + title_bar_height;
                    rect
                };

                self.title_bar_ui(
                    ctx,
                    ui,
                    frame,
                    &title_bar_rect,
                    (constants::literal::APP_NAME.to_string()
                        + " "
                        + constants::literal::TEST_VERSION
                        + " "
                        + env!("APP_VERSION"))
                    .as_str(),
                );

                if self.adjusted {
                    self.current_queue_idx += 1;
                    if self.current_queue_idx < self.action_queue.len() {
                        self.action_queue.clone_from(
                            &self
                                .action_queue
                                .range(0..self.current_queue_idx)
                                .cloned()
                                .collect(),
                        );
                        self.current_queue_idx = self.action_queue.len();
                    } else if self.current_queue_idx > self.queue_size as usize {
                        self.current_queue_idx -= 1;
                        self.action_queue.pop_back();
                    }
                    self.adjusted = false;
                    self.changed = true;
                }
                self.render_windows(
                    ctx,
                    {
                        let mut rect = app_rect;
                        rect.min.y = title_bar_rect.max.y;
                        rect
                    }
                    .center(),
                );

                let rest_rect = ui.available_rect_before_wrap().shrink(4.0);
                let mut ui = ui.child_ui(rest_rect, *ui.layout());

                if self.file_category.is_nil() {
                    ui.vertical_centered(|ui| {
                        ui.heading("你未打开任何文件，依次点击 `文件 -> 打开文件` ，或者点击");

                        if ui.link(RichText::new("这里").heading()).clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("打开")
                                .add_filter("文件", &["bax", "mp3"])
                                .pick_file()
                            {
                                match self.open(&path) {
                                    Ok(v) => {
                                        (self.breakpoints, self.file_category) = v;
                                    }
                                    Err(e) => {
                                        self.toasts
                                            .error(e.to_string())
                                            .set_closable(true)
                                            .set_duration(Some(DUR));
                                    }
                                }
                                self.changed = false;
                            }
                        }
                        ui.heading("来打开文件。");
                    });
                } else {
                    self.scroll_area(&mut ui, rest_rect);
                }
                self.toasts.show(ctx);
            })
            .response
            .context_menu(|ui| {
                self.speed_control(ui);
                if ui
                    .add(
                        egui::Slider::new(&mut self.volume, 0..=100)
                            .trailing_fill(true)
                            .show_value(true)
                            .custom_formatter(|v, _| (v as u64).to_string() + "%"),
                    )
                    .changed()
                {
                    self.player.set_volume(self.volume);
                }
                if self.is_muted {
                    if ui.button("取消静音").clicked() {
                        self.is_muted = false;
                    }
                } else if ui.button("静音").clicked() {
                    self.is_muted = true;
                }
            });
    }
}
