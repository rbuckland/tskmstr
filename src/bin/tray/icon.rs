//! Renders the tray icon: a circle holding an italic "t" beside the open-task
//! count. On macOS this is a monochrome template image (black + alpha, which
//! the menu bar tints to match its appearance); elsewhere it is a coloured
//! square. Also renders the application icon (the coloured mark without a
//! count) used for the .app / .msi.
//!
//! Geometry is described in a 64-unit-tall design space and scaled to the
//! requested pixel height, so the same code produces crisp 16px and 1024px
//! output.

use ab_glyph::{Font, FontRef, OutlineCurve};
use tiny_skia::{
    BlendMode, FillRule, LineJoin, Paint, Path, PathBuilder, Pixmap, Stroke, Transform,
};

const OUTLINE: (u8, u8, u8) = (0x23, 0x33, 0x40);
const PILL: (u8, u8, u8) = (0xa9, 0xcc, 0xe8);
const TEAL: (u8, u8, u8) = (0x1c, 0x5f, 0x7e);
const WHITE: (u8, u8, u8) = (0xff, 0xff, 0xff);
const BLACK: (u8, u8, u8) = (0, 0, 0);

/// Design-space height of every layout.
const UNIT: f32 = 64.0;

/// Which shape to draw.
#[derive(Clone, Copy, Debug)]
pub enum Layout {
    /// Monochrome template image for the macOS menu bar: a solid disc with
    /// the "t" knocked out, then the count. Its width hugs the count.
    MenuBar,
    /// Square with the count; used on Windows/Linux where tray icons are square.
    Square,
    /// Square application icon: just the "t" mark, no count.
    AppIcon,
}

// MenuBar geometry, in design units.
const MB_R: f32 = 29.0;
const MB_CX: f32 = MB_R + 3.0;
const MB_GAP: f32 = 6.0;
const MB_COUNT_PX: f32 = 50.0;
/// Stroke added around the count glyphs to thicken the light font.
const MB_EMBOLDEN: f32 = 3.5;

impl Layout {
    /// Design-space width of the icon for `count`.
    fn width(self, font: &FontRef, count: usize) -> f32 {
        match self {
            Layout::MenuBar => {
                let text_w = text_path(font, &count_text(count), MB_COUNT_PX, 0.0)
                    .map_or(0.0, |p| p.bounds().width());
                MB_CX + MB_R + MB_GAP + text_w + MB_EMBOLDEN + 3.0
            }
            Layout::Square | Layout::AppIcon => UNIT,
        }
    }

    /// Whether the icon is a template image (black + alpha) that the OS tints.
    pub fn is_template(self) -> bool {
        matches!(self, Layout::MenuBar)
    }
}

pub fn platform_layout() -> Layout {
    if cfg!(target_os = "macos") {
        Layout::MenuBar
    } else {
        Layout::Square
    }
}

/// Render the tray icon for `count` open tasks at the standard tray size.
pub fn render(count: usize, layout: Layout) -> tray_icon::Icon {
    let (rgba, w, h) = render_rgba(count, layout, 64);
    tray_icon::Icon::from_rgba(rgba, w, h).expect("icon buffer is valid RGBA")
}

fn count_text(count: usize) -> String {
    if count > 999 {
        "999+".to_string()
    } else {
        count.to_string()
    }
}

/// Straight (non-premultiplied) RGBA pixels plus dimensions.
pub fn render_rgba(count: usize, layout: Layout, height: u32) -> (Vec<u8>, u32, u32) {
    let pm = render_pixmap(count, layout, height);
    let (w, h) = (pm.width(), pm.height());
    (pm.take_demultiplied(), w, h)
}

pub fn render_pixmap(count: usize, layout: Layout, height: u32) -> Pixmap {
    let font = FontRef::try_from_slice(epaint_default_fonts::UBUNTU_LIGHT).expect("embedded font");
    let height = height.max(8);
    let k = height as f32 / UNIT;
    let w = layout.width(&font, count);
    let h = UNIT;
    let mut pm = Pixmap::new((w * k).ceil() as u32, height).unwrap();
    let ts = Transform::from_scale(k, k);

    match layout {
        Layout::MenuBar => {
            let circle = PathBuilder::from_circle(MB_CX, h / 2.0, MB_R).unwrap();
            fill(&mut pm, &circle, BLACK, ts);
            if let Some(t) = fitted_text_path(
                &font,
                "t",
                46.0 * MB_R / 30.0,
                -0.28,
                (MB_CX, h / 2.0 + 1.0),
                MB_R * 1.3,
            ) {
                // knock the "t" out of the disc, slightly widened so it
                // survives downscaling to menu-bar size
                let mut clear = paint(BLACK);
                clear.blend_mode = BlendMode::Clear;
                pm.fill_path(&t, &clear, FillRule::Winding, ts, None);
                let widen = Stroke {
                    width: 1.5,
                    ..Stroke::default()
                };
                pm.stroke_path(&t, &clear, &widen, ts, None);
            }

            if let Some(text) = text_path(&font, &count_text(count), MB_COUNT_PX, 0.0) {
                let b = text.bounds();
                let left = MB_CX + MB_R + MB_GAP + MB_EMBOLDEN / 2.0;
                let dx = left - b.left();
                let dy = h / 2.0 - (b.top() + b.height() / 2.0);
                if let Some(text) = text.transform(Transform::from_translate(dx, dy)) {
                    fill(&mut pm, &text, BLACK, ts);
                    let bold = Stroke {
                        width: MB_EMBOLDEN,
                        line_join: LineJoin::Round,
                        ..Stroke::default()
                    };
                    pm.stroke_path(&text, &paint(BLACK), &bold, ts, None);
                }
            }
        }
        Layout::Square => {
            let border = 4.0;
            let body = rounded_rect(border / 2.0, border / 2.0, w - border, h - border, 14.0);
            fill(&mut pm, &body, PILL, ts);
            stroke(&mut pm, &body, OUTLINE, border, ts);

            let text = count_text(count);
            draw_text(
                &mut pm,
                &font,
                &text,
                44.0,
                0.0,
                (w / 2.0 + 4.0, h / 2.0 + 6.0),
                OUTLINE,
                w - 18.0,
                ts,
            );

            let (bx, by, br) = (15.0, 15.0, 13.0);
            let circle = PathBuilder::from_circle(bx, by, br).unwrap();
            fill(&mut pm, &circle, TEAL, ts);
            stroke(&mut pm, &circle, OUTLINE, 3.0, ts);
            draw_text(
                &mut pm,
                &font,
                "t",
                22.0,
                -0.28,
                (bx, by + 0.5),
                WHITE,
                br * 1.4,
                ts,
            );
        }
        Layout::AppIcon => {
            // inset a little, as macOS app icons conventionally do
            let (inset, border) = (5.0, 3.0);
            let body = rounded_rect(inset, inset, w - 2.0 * inset, h - 2.0 * inset, 13.0);
            fill(&mut pm, &body, PILL, ts);
            stroke(&mut pm, &body, OUTLINE, border, ts);

            let r = 20.0;
            let circle = PathBuilder::from_circle(w / 2.0, h / 2.0, r).unwrap();
            fill(&mut pm, &circle, TEAL, ts);
            stroke(&mut pm, &circle, OUTLINE, border, ts);
            draw_text(
                &mut pm,
                &font,
                "t",
                34.0,
                -0.28,
                (w / 2.0, h / 2.0 + 0.5),
                WHITE,
                r * 1.3,
                ts,
            );
        }
    }

    pm
}

fn paint(rgb: (u8, u8, u8)) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color_rgba8(rgb.0, rgb.1, rgb.2, 255);
    p.anti_alias = true;
    p
}

fn fill(pm: &mut Pixmap, path: &Path, rgb: (u8, u8, u8), ts: Transform) {
    pm.fill_path(path, &paint(rgb), FillRule::Winding, ts, None);
}

fn stroke(pm: &mut Pixmap, path: &Path, rgb: (u8, u8, u8), width: f32, ts: Transform) {
    let stroke = Stroke {
        width,
        ..Stroke::default()
    };
    pm.stroke_path(path, &paint(rgb), &stroke, ts, None);
}

/// Rounded rectangle; with `r == h / 2` this is a pill.
fn rounded_rect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Path {
    let k = 0.552_284_8 * r;
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
    pb.line_to(x, y + r);
    pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    pb.close();
    pb.finish().expect("valid rounded rect")
}

/// Build a path for `text` laid out along a baseline at the origin, in
/// design units, for a font size of `px` (per em). `skew` shears the glyphs
/// for a faux italic (negative leans right).
fn text_path(font: &FontRef, text: &str, px: f32, skew: f32) -> Option<Path> {
    let units = font.units_per_em()?;
    let s = px / units;
    let mut pb = PathBuilder::new();
    let mut pen_x = 0.0f32;

    for ch in text.chars() {
        let gid = font.glyph_id(ch);
        if let Some(outline) = font.outline(gid) {
            let tx = |p: ab_glyph::Point| (p.x * s + pen_x, -p.y * s);
            let mut last_end: Option<(f32, f32)> = None;
            for curve in &outline.curves {
                let (start, end) = match curve {
                    OutlineCurve::Line(a, b) => (tx(*a), tx(*b)),
                    OutlineCurve::Quad(a, _, b) => (tx(*a), tx(*b)),
                    OutlineCurve::Cubic(a, _, _, b) => (tx(*a), tx(*b)),
                };
                let continues = last_end
                    .map(|(lx, ly)| (lx - start.0).abs() < 1e-3 && (ly - start.1).abs() < 1e-3)
                    .unwrap_or(false);
                if !continues {
                    if last_end.is_some() {
                        pb.close();
                    }
                    pb.move_to(start.0, start.1);
                }
                match curve {
                    OutlineCurve::Line(_, b) => {
                        let b = tx(*b);
                        pb.line_to(b.0, b.1);
                    }
                    OutlineCurve::Quad(_, c, b) => {
                        let (c, b) = (tx(*c), tx(*b));
                        pb.quad_to(c.0, c.1, b.0, b.1);
                    }
                    OutlineCurve::Cubic(_, c1, c2, b) => {
                        let (c1, c2, b) = (tx(*c1), tx(*c2), tx(*b));
                        pb.cubic_to(c1.0, c1.1, c2.0, c2.1, b.0, b.1);
                    }
                }
                last_end = Some(end);
            }
            if last_end.is_some() {
                pb.close();
            }
        }
        pen_x += font.h_advance_unscaled(gid) * s;
    }

    let path = pb.finish()?;
    if skew != 0.0 {
        path.transform(Transform::from_skew(skew, 0.0))
    } else {
        Some(path)
    }
}

/// Draw `text` centred on `center` (design units), shrinking it if wider
/// than `max_width`.
#[allow(clippy::too_many_arguments)]
fn draw_text(
    pm: &mut Pixmap,
    font: &FontRef,
    text: &str,
    px: f32,
    skew: f32,
    center: (f32, f32),
    rgb: (u8, u8, u8),
    max_width: f32,
    ts: Transform,
) {
    if let Some(p) = fitted_text_path(font, text, px, skew, center, max_width) {
        fill(pm, &p, rgb, ts);
    }
}

/// Path for `text` centred on `center` (design units), shrunk if wider than
/// `max_width`.
fn fitted_text_path(
    font: &FontRef,
    text: &str,
    px: f32,
    skew: f32,
    center: (f32, f32),
    max_width: f32,
) -> Option<Path> {
    let path = text_path(font, text, px, skew)?;
    let b = path.bounds();
    let path = if b.width() > max_width {
        let scale = max_width / b.width();
        path.transform(Transform::from_scale(scale, scale))?
    } else {
        path
    };
    let b = path.bounds();
    let dx = center.0 - (b.left() + b.width() / 2.0);
    let dy = center.1 - (b.top() + b.height() / 2.0);
    path.transform(Transform::from_translate(dx, dy))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_all_layouts() {
        for layout in [Layout::MenuBar, Layout::Square, Layout::AppIcon] {
            for n in [0usize, 7, 43, 128, 5000] {
                for height in [16u32, 64, 256] {
                    let (rgba, w, h) = render_rgba(n, layout, height);
                    assert_eq!(h, height);
                    if layout.is_template() {
                        assert!(w > h && w < 4 * h);
                        assert!(rgba.chunks(4).all(|p| p[..3] == [0, 0, 0]));
                    } else {
                        assert_eq!(w, h);
                    }
                    assert_eq!(rgba.len(), (w * h * 4) as usize);
                    assert!(rgba.chunks(4).any(|p| p[3] == 255));
                }
            }
        }
    }

    /// Writes sample tray icons and the app icon at all the sizes the
    /// packaging assets need:
    /// `TSKMSTR_ICON_DUMP=/tmp/dir cargo test --features tray --bin tskmstr-tray dump -- --ignored`
    #[test]
    #[ignore]
    fn dump_pngs() {
        let dir =
            std::env::var("TSKMSTR_ICON_DUMP").expect("set TSKMSTR_ICON_DUMP to an output dir");
        for (name, layout) in [("menubar", Layout::MenuBar), ("square", Layout::Square)] {
            for n in [0usize, 7, 43, 128] {
                render_pixmap(n, layout, 64)
                    .save_png(format!("{dir}/icon-{name}-{n}.png"))
                    .unwrap();
            }
        }
        for size in [16u32, 32, 48, 64, 128, 256, 512, 1024] {
            render_pixmap(0, Layout::AppIcon, size)
                .save_png(format!("{dir}/app-icon-{size}.png"))
                .unwrap();
        }
    }
}
