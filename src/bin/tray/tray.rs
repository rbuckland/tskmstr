//! Tray icon host. On macOS and Windows the icon lives on the main (winit)
//! thread. On Linux the AppIndicator backend needs a GTK loop, which we run
//! on a dedicated thread and talk to over a channel.

use anyhow::Result;
use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem};
use tray_icon::TrayIconBuilder;

use crate::icon;

pub const MENU_SHOW: &str = "show";
pub const MENU_REFRESH: &str = "refresh";
pub const MENU_QUIT: &str = "quit";

fn build_menu() -> Result<Menu> {
    let show = MenuItem::with_id(MENU_SHOW, "Show tasks", true, None);
    let refresh = MenuItem::with_id(MENU_REFRESH, "Refresh now", true, None);
    let quit = MenuItem::with_id(MENU_QUIT, "Quit tskmstr", true, None);
    Ok(Menu::with_items(&[
        &show,
        &refresh,
        &PredefinedMenuItem::separator(),
        &quit,
    ])?)
}

fn tooltip(count: usize) -> String {
    format!(
        "tskmstr: {} open task{}",
        count,
        if count == 1 { "" } else { "s" }
    )
}

fn build_tray(count: usize) -> Result<tray_icon::TrayIcon> {
    Ok(TrayIconBuilder::new()
        .with_id("tskmstr")
        .with_icon(icon::render(count, icon::platform_layout()))
        .with_menu(Box::new(build_menu()?))
        .with_menu_on_left_click(false)
        .with_tooltip(tooltip(count))
        .build()?)
}

#[cfg(not(target_os = "linux"))]
pub struct TrayHost {
    tray: tray_icon::TrayIcon,
}

#[cfg(not(target_os = "linux"))]
impl TrayHost {
    /// Must be called on the main thread, after the event loop has started.
    pub fn new(count: usize) -> Result<Self> {
        Ok(Self {
            tray: build_tray(count)?,
        })
    }

    pub fn set_count(&self, count: usize) {
        let _ = self
            .tray
            .set_icon(Some(icon::render(count, icon::platform_layout())));
        let _ = self.tray.set_tooltip(Some(tooltip(count)));
    }

    /// Screen position/size of the tray icon (physical pixels), if known.
    pub fn rect(&self) -> Option<tray_icon::Rect> {
        self.tray.rect()
    }
}

#[cfg(target_os = "linux")]
pub struct TrayHost {
    tx: std::sync::mpsc::Sender<usize>,
}

#[cfg(target_os = "linux")]
impl TrayHost {
    pub fn new(count: usize) -> Result<Self> {
        use std::sync::mpsc::{channel, RecvTimeoutError};
        use std::time::Duration;

        let (tx, rx) = channel::<usize>();
        let (ready_tx, ready_rx) = channel::<Result<()>>();

        std::thread::Builder::new()
            .name("tskmstr-tray-gtk".into())
            .spawn(move || {
                if let Err(e) = gtk::init() {
                    let _ = ready_tx.send(Err(anyhow::anyhow!("gtk init failed: {e}")));
                    return;
                }
                let tray = match build_tray(count) {
                    Ok(t) => t,
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                };
                let _ = ready_tx.send(Ok(()));

                loop {
                    while gtk::events_pending() {
                        gtk::main_iteration_do(false);
                    }
                    match rx.recv_timeout(Duration::from_millis(50)) {
                        Ok(n) => {
                            let _ = tray.set_icon(Some(icon::render(n, icon::platform_layout())));
                            let _ = tray.set_tooltip(Some(tooltip(n)));
                        }
                        Err(RecvTimeoutError::Timeout) => {}
                        Err(RecvTimeoutError::Disconnected) => break,
                    }
                }
            })?;

        ready_rx
            .recv()
            .map_err(|_| anyhow::anyhow!("tray thread exited before becoming ready"))??;
        Ok(Self { tx })
    }

    pub fn set_count(&self, count: usize) {
        let _ = self.tx.send(count);
    }

    /// AppIndicator icons have no queryable screen position.
    pub fn rect(&self) -> Option<tray_icon::Rect> {
        None
    }
}
