//! Convert the tray icon's rect (which `tray-icon` reports in *physical*
//! pixels of whatever screen the icon is on) into global logical points, the
//! coordinate space egui's `ViewportCommand::OuterPosition` works in.
//!
//! On macOS, screens can have different scale factors, and the panel window
//! is often on a different screen than the menu bar item, so dividing by the
//! panel's own scale factor is wrong. We look the icon up in the display list
//! via AppKit's NSScreen instead. Elsewhere the panel's scale factor is used.

use eframe::egui::Rect;

/// The icon in logical points, plus the bounds of the display it sits on
/// (also points) when known, for keeping the panel on that display.
#[derive(Debug, Clone, Copy)]
pub struct Anchor {
    pub rect: Rect,
    pub display: Option<Rect>,
}

/// `None` when the rect cannot be trusted: zero-sized, or (macOS) not on any
/// display, which is what tray-icon reports before the status item has been
/// laid out.
pub fn anchor_from_tray_rect(r: &tray_icon::Rect, panel_pixels_per_point: f32) -> Option<Anchor> {
    if r.size.width == 0 || r.size.height == 0 {
        return None;
    }

    #[cfg(target_os = "macos")]
    match macos::anchor(r) {
        macos::Lookup::Found(a) => return Some(a),
        macos::Lookup::OffScreen => return None,
        macos::Lookup::NoDisplayList => {} // fall back to the panel's scale
    }

    use eframe::egui::{pos2, vec2};
    let s = panel_pixels_per_point.max(0.5);
    Some(Anchor {
        rect: Rect::from_min_size(
            pos2(r.position.x as f32 / s, r.position.y as f32 / s),
            vec2(r.size.width as f32 / s, r.size.height as f32 / s),
        ),
        display: None,
    })
}

#[cfg(target_os = "macos")]
mod macos {
    use super::Anchor;
    use eframe::egui::{pos2, vec2, Rect};
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSScreen;

    struct Display {
        /// Global points, top-left origin (the frame winit and tray-icon use)
        bounds: Rect,
        scale: f32,
    }

    /// All screens, converted from AppKit's bottom-left global coordinates to
    /// the top-left frame used by winit / tray-icon. `NSScreen.screens[0]` is
    /// the display holding the menu bar, whose origin is (0, 0).
    fn displays() -> Vec<Display> {
        let Some(mtm) = MainThreadMarker::new() else {
            return Vec::new();
        };
        let screens = NSScreen::screens(mtm);
        let Some(primary) = screens.firstObject() else {
            return Vec::new();
        };
        let primary_height = primary.frame().size.height;

        screens
            .iter()
            .map(|s| {
                let f = s.frame();
                let top = primary_height - (f.origin.y + f.size.height);
                Display {
                    bounds: Rect::from_min_size(
                        pos2(f.origin.x as f32, top as f32),
                        vec2(f.size.width as f32, f.size.height as f32),
                    ),
                    scale: (s.backingScaleFactor() as f32).max(0.5),
                }
            })
            .collect()
    }

    pub enum Lookup {
        Found(Anchor),
        /// Displays are known but the rect is on none of them (not laid out yet)
        OffScreen,
        /// No screen list available; caller should fall back
        NoDisplayList,
    }

    /// Find the display whose scale factor maps the physical rect onto it.
    pub fn anchor(r: &tray_icon::Rect) -> Lookup {
        let displays = displays();
        if displays.is_empty() {
            log::debug!("NSScreen.screens is empty");
            return Lookup::NoDisplayList;
        }
        for d in displays {
            let rect = Rect::from_min_size(
                pos2(r.position.x as f32 / d.scale, r.position.y as f32 / d.scale),
                vec2(
                    r.size.width as f32 / d.scale,
                    r.size.height as f32 / d.scale,
                ),
            );
            log::debug!(
                "display {:?} scale {} -> icon {:?} inside={}",
                d.bounds,
                d.scale,
                rect,
                d.bounds.contains(rect.center())
            );
            if d.bounds.contains(rect.center()) {
                return Lookup::Found(Anchor {
                    rect,
                    display: Some(d.bounds),
                });
            }
        }
        Lookup::OffScreen
    }
}
