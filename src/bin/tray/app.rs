//! The drop-down panel (an undecorated, always-on-top egui window) plus the
//! glue between tray events, the background worker and the UI.

use std::collections::HashSet;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

use eframe::egui::text::LayoutJob;
use eframe::egui::{
    self, Align, Button, Color32, CursorIcon, FontId, Label, Layout, OpenUrl, RichText, Sense,
    TextFormat, ViewportCommand,
};
use log::{debug, info, warn};
use tray_icon::menu::MenuEvent;
use tray_icon::{MouseButton, MouseButtonState, TrayIconEvent};

use tskmstr::config::AppConfig;
use tskmstr::output::{group_tasks, DisplayOptions, TaskGroup};
use tskmstr::providers::common::model::Issue;

use crate::display::anchor_from_tray_rect;
use crate::tray::{TrayHost, MENU_QUIT, MENU_REFRESH, MENU_SHOW};
use crate::worker::{self, Request, Response};

/// Size of the drop-down panel in logical points.
pub const WINDOW_SIZE: [f32; 2] = [420.0, 540.0];

/// Ignore a tray click that arrives right after the panel auto-hid on focus
/// loss: that click *caused* the focus loss and the user meant "close".
const REOPEN_SUPPRESS: Duration = Duration::from_millis(400);

struct UiColors {
    issue_id: Color32,
    title: Color32,
    tags: Color32,
}

pub struct TrayApp {
    display: DisplayOptions,
    colors: UiColors,

    groups: Vec<TaskGroup>,
    total: usize,
    pending_close: HashSet<String>,
    status: String,
    error: Option<String>,
    last_refresh: Option<Instant>,

    req_tx: Sender<Request>,
    resp_rx: Receiver<Response>,
    tray_rx: Receiver<TrayIconEvent>,
    menu_rx: Receiver<MenuEvent>,

    tray: Option<TrayHost>,
    tray_failed: bool,
    /// Deadline until which we wait for a valid tray rect before opening at start-up
    open_on_start: Option<Instant>,
    visible: bool,
    has_been_focused: bool,
    hidden_at: Option<Instant>,
    /// Icon rect the panel is anchored to. For a few frames after showing we
    /// recompute the position from it: until the window has actually been
    /// mapped, macOS reports a scale factor of 1.0 and no monitor, so the
    /// first computation is wrong on Retina displays.
    anchor: Option<tray_icon::Rect>,
    settle_frames: u8,
}

/// System fonts that may cover the Enclosed Alphanumeric Supplement (U+1F130..),
/// which store ids such as `🄿` or `🅆` use. egui's bundled fonts lack these.
/// Each is checked for coverage before use: DejaVu Sans, for example, does not.
const SYMBOL_FONT_CANDIDATES: &[&str] = &[
    "/System/Library/Fonts/Apple Symbols.ttf",
    "C:\\Windows\\Fonts\\seguisym.ttf",
    "/usr/share/fonts/truetype/noto/NotoSansSymbols-Regular.ttf",
    "/usr/share/fonts/noto/NotoSansSymbols-Regular.ttf",
    "/usr/share/fonts/google-noto/NotoSansSymbols-Regular.ttf",
    "/usr/share/fonts/truetype/ancient-scripts/Symbola_hint.ttf",
    "/usr/share/fonts/TTF/Symbola.ttf",
];

/// Characters a candidate font must contain to be used as the fallback.
const SYMBOL_FONT_PROBE: &str = "🄰🄿🅆🅉";

/// The first candidate that exists and covers [`SYMBOL_FONT_PROBE`].
fn find_symbol_font() -> Option<(&'static str, Vec<u8>)> {
    use ab_glyph::{Font, FontRef};

    SYMBOL_FONT_CANDIDATES.iter().find_map(|path| {
        let bytes = std::fs::read(path).ok()?;
        let font = FontRef::try_from_slice(&bytes).ok()?;
        SYMBOL_FONT_PROBE
            .chars()
            .all(|c| font.glyph_id(c).0 != 0)
            .then_some((*path, bytes))
    })
}

/// Append a symbol font as the lowest-priority fallback for both font
/// families, so issue ids render instead of showing a box.
fn add_symbol_fallback_font(ctx: &egui::Context) {
    use egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};
    use egui::{FontData, FontFamily};

    let Some((path, bytes)) = find_symbol_font() else {
        warn!("no symbol fallback font found (install Noto Sans Symbols); enclosed-letter store ids may not render");
        return;
    };
    debug!("using symbol fallback font {path}");
    let families = [FontFamily::Proportional, FontFamily::Monospace]
        .into_iter()
        .map(|family| InsertFontFamily {
            family,
            priority: FontPriority::Lowest,
        })
        .collect();
    ctx.add_font(FontInsert::new(
        "symbol-fallback",
        FontData::from_owned(bytes),
        families,
    ));
}

impl TrayApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        config: AppConfig,
        refresh_interval: Duration,
        open_on_start: bool,
    ) -> Self {
        let ctx = cc.egui_ctx.clone();
        add_symbol_fallback_font(&ctx);

        let (tray_tx, tray_rx) = channel::<TrayIconEvent>();
        let c = ctx.clone();
        TrayIconEvent::set_event_handler(Some(move |e| {
            let _ = tray_tx.send(e);
            c.request_repaint();
        }));

        let (menu_tx, menu_rx) = channel::<MenuEvent>();
        let c = ctx.clone();
        MenuEvent::set_event_handler(Some(move |e| {
            let _ = menu_tx.send(e);
            c.request_repaint();
        }));

        let display = DisplayOptions::from_config(&config);
        let colors = UiColors {
            issue_id: color_from_name(&config.colors.issue_id, &ctx),
            title: color_from_name(&config.colors.title, &ctx),
            tags: color_from_name(&config.colors.tags, &ctx),
        };

        let (req_tx, resp_rx) = worker::spawn(config, ctx.clone(), refresh_interval);

        // make sure `logic()` runs at least once so the tray icon gets created
        ctx.request_repaint();

        Self {
            groups: group_tasks(&[], &display),
            display,
            colors,
            total: 0,
            pending_close: HashSet::new(),
            status: "Loading…".to_string(),
            error: None,
            last_refresh: None,
            req_tx,
            resp_rx,
            tray_rx,
            menu_rx,
            tray: None,
            tray_failed: false,
            open_on_start: open_on_start.then(|| Instant::now() + Duration::from_secs(3)),
            visible: false,
            has_been_focused: false,
            hidden_at: None,
            anchor: None,
            settle_frames: 0,
        }
    }

    fn ensure_tray(&mut self) {
        if self.tray.is_some() || self.tray_failed {
            return;
        }
        match TrayHost::new(self.total) {
            Ok(t) => {
                info!("tray icon created");
                self.tray = Some(t);
            }
            Err(e) => {
                warn!("could not create tray icon: {e}");
                self.tray_failed = true;
                self.error = Some(format!("Tray icon unavailable: {e}"));
            }
        }
    }

    fn handle_worker_responses(&mut self) {
        while let Ok(resp) = self.resp_rx.try_recv() {
            match resp {
                Response::Tasks(issues) => {
                    self.total = issues.len();
                    self.groups = group_tasks(&issues, &self.display);
                    self.last_refresh = Some(Instant::now());
                    self.error = None;
                    self.status = format!(
                        "{} open task{}",
                        self.total,
                        if self.total == 1 { "" } else { "s" }
                    );
                    if let Some(t) = &self.tray {
                        t.set_count(self.total);
                    }
                }
                Response::Closed { id, error } => {
                    self.pending_close.remove(&id);
                    match error {
                        None => info!("closed {id}"),
                        Some(e) => self.error = Some(format!("Could not close {id}: {e}")),
                    }
                }
                Response::Error(e) => {
                    self.error = Some(e);
                    self.status = "Refresh failed".to_string();
                }
            }
        }
    }

    fn handle_tray_events(&mut self, ctx: &egui::Context) {
        while let Ok(ev) = self.menu_rx.try_recv() {
            match ev.id().0.as_str() {
                MENU_SHOW => self.show(ctx, None),
                MENU_REFRESH => self.request_refresh(),
                MENU_QUIT => ctx.send_viewport_cmd(ViewportCommand::Close),
                _ => {}
            }
        }

        while let Ok(ev) = self.tray_rx.try_recv() {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = ev
            {
                if self.visible {
                    self.hide(ctx);
                } else {
                    let just_hidden = self
                        .hidden_at
                        .map(|t| t.elapsed() < REOPEN_SUPPRESS)
                        .unwrap_or(false);
                    if !just_hidden {
                        self.show(ctx, Some(rect));
                    }
                }
            }
        }
    }

    fn request_refresh(&mut self) {
        self.status = "Refreshing…".to_string();
        let _ = self.req_tx.send(Request::Refresh);
    }

    fn show(&mut self, ctx: &egui::Context, anchor: Option<tray_icon::Rect>) {
        // A click gives us the icon's rect; for the menu item / --open ask the tray.
        let anchor = anchor.filter(|r| self.rect_is_usable(ctx, r)).or_else(|| {
            self.tray
                .as_ref()
                .and_then(|t| t.rect())
                .filter(|r| self.rect_is_usable(ctx, r))
        });
        let pos = self.panel_position(ctx, anchor);
        debug!("show panel: anchor={anchor:?} -> {pos:?}");

        ctx.send_viewport_cmd(ViewportCommand::OuterPosition(pos));
        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
        self.anchor = anchor;
        self.settle_frames = 8;
        self.visible = true;
        self.has_been_focused = false;
        ctx.request_repaint();
    }

    /// Compute the panel's top-left (global logical points) so it hangs just
    /// below the tray icon (or just above it for a bottom taskbar), kept on the
    /// icon's display.
    fn panel_position(&self, ctx: &egui::Context, anchor: Option<tray_icon::Rect>) -> egui::Pos2 {
        let ppp = Self::panel_ppp(ctx);
        let panel_monitor = ctx
            .input(|i| i.viewport().monitor_size)
            .map(|m| egui::Rect::from_min_size(egui::Pos2::ZERO, m));
        let size = egui::vec2(WINDOW_SIZE[0], WINDOW_SIZE[1]);
        let margin = 6.0;

        let Some(anchor) = anchor.and_then(|r| anchor_from_tray_rect(&r, ppp)) else {
            // no icon position known: top-right of the panel's monitor
            return match panel_monitor {
                Some(m) => egui::pos2(m.right() - size.x - 2.0 * margin, 40.0),
                None => egui::pos2(100.0, 100.0),
            };
        };

        let display = anchor.display.or(panel_monitor);
        // menu bar / tray at the top -> open below; taskbar at the bottom -> open above
        let at_top = display
            .map(|d| anchor.rect.top() < d.center().y)
            .unwrap_or(true);
        let mut y = if at_top {
            anchor.rect.bottom() + margin
        } else {
            anchor.rect.top() - size.y - margin
        };
        let mut x = anchor.rect.center().x - size.x / 2.0;
        if let Some(d) = display {
            x = x.min(d.right() - size.x - margin).max(d.left() + margin);
            y = y.min(d.bottom() - size.y).max(d.top());
        }
        egui::pos2(x, y)
    }

    fn panel_ppp(ctx: &egui::Context) -> f32 {
        ctx.input(|i| i.viewport().native_pixels_per_point)
            .unwrap_or_else(|| ctx.pixels_per_point())
    }

    /// tray-icon reports a zero-sized or off-screen rect while the status item
    /// is still being laid out.
    fn rect_is_usable(&self, ctx: &egui::Context, r: &tray_icon::Rect) -> bool {
        match anchor_from_tray_rect(r, Self::panel_ppp(ctx)) {
            None => false,
            Some(a) if a.display.is_some() => true,
            // no display info: at least require it to be on the panel's monitor
            Some(a) => match ctx.input(|i| i.viewport().monitor_size) {
                Some(m) => egui::Rect::from_min_size(egui::Pos2::ZERO, m).contains(a.rect.center()),
                None => true,
            },
        }
    }

    /// For a few frames after showing, recompute the position (the scale
    /// factor and monitor are only reliable once the window is mapped) and
    /// re-issue the move if the window is not where we want it.
    fn settle_position(&mut self, ctx: &egui::Context) {
        if !self.visible || self.settle_frames == 0 {
            return;
        }
        self.settle_frames -= 1;
        let target = self.panel_position(ctx, self.anchor);
        let actual = ctx.input(|i| i.viewport().outer_rect).map(|r| r.min);
        let ppp = ctx.input(|i| i.viewport().native_pixels_per_point);
        debug!("settle panel: want {target:?}, have {actual:?} (ppp {ppp:?})");
        match actual {
            Some(a) if a.distance(target) < 1.5 => self.settle_frames = 0,
            _ => {
                ctx.send_viewport_cmd(ViewportCommand::OuterPosition(target));
                ctx.request_repaint();
            }
        }
    }

    fn hide(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        self.visible = false;
        self.hidden_at = Some(Instant::now());
    }

    fn auto_hide_on_focus_loss(&mut self, ctx: &egui::Context) {
        if !self.visible {
            return;
        }
        match ctx.input(|i| i.viewport().focused) {
            Some(true) => self.has_been_focused = true,
            Some(false) if self.has_been_focused => self.hide(ctx),
            _ => {}
        }
    }

    fn issue_row(
        &self,
        ui: &mut egui::Ui,
        issue: &Issue,
        to_close: &mut Vec<String>,
        to_open: &mut Vec<String>,
    ) {
        let pending = self.pending_close.contains(&issue.id);
        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let label = if pending { "[…]" } else { "[x]" };
                let done = ui
                    .add_enabled(
                        !pending,
                        Button::new(RichText::new(label).monospace()).frame(false),
                    )
                    .on_hover_text("Done: close this task");
                if done.clicked() {
                    to_close.push(issue.id.clone());
                }

                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    let id_color = issue
                        .color
                        .as_deref()
                        .and_then(parse_color_name)
                        .unwrap_or(self.colors.issue_id);
                    ui.label(RichText::new(&issue.id).color(id_color).monospace());

                    let mut job = LayoutJob::default();
                    job.append(
                        &issue.title,
                        0.0,
                        TextFormat {
                            font_id: FontId::proportional(13.5),
                            color: self.colors.title,
                            ..Default::default()
                        },
                    );
                    if !issue.tags.is_empty() {
                        let tags = issue
                            .tags
                            .iter()
                            .map(|t| t.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");
                        job.append(
                            &format!("({tags})"),
                            6.0,
                            TextFormat {
                                font_id: FontId::proportional(11.5),
                                color: self.colors.tags,
                                ..Default::default()
                            },
                        );
                    }
                    let title = ui
                        .add(Label::new(job).truncate().sense(Sense::click()))
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .on_hover_text(&issue.html_url);
                    if title.clicked() {
                        to_open.push(issue.html_url.clone());
                    }
                });
            });
        });
    }
}

impl eframe::App for TrayApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ensure_tray();
        if let Some(deadline) = self.open_on_start {
            // the status item is laid out asynchronously; wait briefly for a real rect
            let raw = self.tray.as_ref().and_then(|t| t.rect());
            debug!("open-on-start: tray rect {raw:?}");
            let rect = raw.filter(|r| self.rect_is_usable(ctx, r));
            if rect.is_some() || Instant::now() > deadline || self.tray_failed {
                self.open_on_start = None;
                self.show(ctx, rect);
            } else {
                ctx.request_repaint_after(Duration::from_millis(50));
            }
        }
        self.handle_worker_responses();
        self.handle_tray_events(ctx);
        self.settle_position(ctx);
        self.auto_hide_on_focus_loss(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut to_close: Vec<String> = Vec::new();
        let mut to_open: Vec<String> = Vec::new();
        let mut refresh = false;
        let mut hide = false;

        egui::Frame::central_panel(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("tskmstr").strong().size(17.0));
                ui.label(RichText::new(&self.status).weak());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add(Button::new("✕").frame(false))
                        .on_hover_text("Hide")
                        .clicked()
                    {
                        hide = true;
                    }
                    if ui
                        .add(Button::new("⟳").frame(false))
                        .on_hover_text("Refresh now")
                        .clicked()
                    {
                        refresh = true;
                    }
                    if let Some(t) = self.last_refresh {
                        let secs = t.elapsed().as_secs();
                        let ago = if secs < 60 {
                            "just now".to_string()
                        } else {
                            format!("{}m ago", secs / 60)
                        };
                        ui.label(RichText::new(ago).weak().small());
                    }
                });
            });
            if let Some(err) = &self.error {
                ui.colored_label(Color32::from_rgb(220, 80, 80), err);
            }
            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for group in &self.groups {
                        ui.add_space(6.0);
                        if group.separator_before {
                            ui.separator();
                        }
                        if group.show_heading {
                            ui.label(
                                RichText::new(&group.heading)
                                    .color(self.colors.tags)
                                    .strong(),
                            );
                            ui.separator();
                        }
                        if group.issues.is_empty() && group.show_heading {
                            ui.label(RichText::new("   none").weak().italics());
                        }
                        for issue in &group.issues {
                            self.issue_row(ui, issue, &mut to_close, &mut to_open);
                        }
                    }
                    ui.add_space(8.0);
                });
        });

        let ctx = ui.ctx().clone();
        for url in to_open {
            ctx.open_url(OpenUrl::new_tab(url));
        }
        for id in to_close {
            self.pending_close.insert(id.clone());
            let _ = self.req_tx.send(Request::Close(id));
        }
        if refresh {
            self.request_refresh();
        }
        if hide {
            self.hide(&ctx);
        }
    }
}

/// Map a `colored`-crate colour name from the config to something readable on
/// both light and dark egui themes.
/// Map a `colored`-style colour name (as used in the config) to a panel colour.
/// Returns `None` for names we have no mapping for.
fn parse_color_name(name: &str) -> Option<Color32> {
    match name.trim().to_ascii_lowercase().replace('_', " ").as_str() {
        "red" | "bright red" => Some(Color32::from_rgb(222, 84, 84)),
        "green" | "bright green" => Some(Color32::from_rgb(76, 170, 96)),
        "yellow" | "bright yellow" => Some(Color32::from_rgb(204, 160, 40)),
        "blue" | "bright blue" => Some(Color32::from_rgb(86, 142, 232)),
        "magenta" | "purple" | "bright magenta" => Some(Color32::from_rgb(196, 96, 200)),
        "cyan" | "bright cyan" => Some(Color32::from_rgb(58, 168, 190)),
        _ => None,
    }
}

/// Like [`parse_color_name`] but falls back to the theme's text colour.
fn color_from_name(name: &str, ctx: &egui::Context) -> Color32 {
    parse_color_name(name).unwrap_or_else(|| ctx.global_style().visuals.text_color())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Store ids like `🅆/6` must render in the panel, not as a missing-glyph box.
    #[test]
    fn enclosed_letter_ids_have_glyphs() {
        if find_symbol_font().is_none() {
            eprintln!("no symbol font installed; skipping");
            return;
        }
        let ctx = egui::Context::default();
        add_symbol_fallback_font(&ctx);
        ctx.run_ui(Default::default(), |_| {})
            .textures_delta
            .clear();
        for font_id in [FontId::monospace(13.0), FontId::proportional(13.5)] {
            assert!(ctx.fonts_mut(|f| f.has_glyphs(&font_id, SYMBOL_FONT_PROBE)));
        }
    }
}
