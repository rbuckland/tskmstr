//! `tskmstr-tray`: a system tray / menu bar widget for tskmstr.
//!
//! Shows a "(t) nn" pill icon with the number of open tasks. Clicking the icon
//! drops down a panel listing every task, grouped exactly like `tskmstr list`.
//! Clicking a task title opens it in the browser; the `[x]` on the right closes
//! (completes) the task.

// No console window when launched on Windows (release builds).
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod assets;
mod autostart;
mod display;
mod icon;
mod tray;
mod worker;

use std::time::Duration;

use anyhow::anyhow;
use clap::{Parser, Subcommand};

use tskmstr::config::{load_config, resolve_config_path};

#[derive(Debug, Parser)]
#[command(name = "tskmstr-tray")]
#[command(about = "tskmstr: system tray widget showing your open tasks", long_about = None)]
struct Cli {
    #[arg(short, long)]
    debug: bool,

    /// Config file default is "~/.config/tskmstr/tskmstr.config.yml" (all platforms)
    #[arg(short, long)]
    config: Option<String>,

    /// How often (seconds) to re-fetch tasks from all providers
    #[arg(long, default_value_t = 300)]
    refresh_secs: u64,

    /// Show the task panel immediately on start-up
    #[arg(long)]
    open: bool,

    #[command(subcommand)]
    cmd: Option<TrayCommand>,
}

#[derive(Debug, Subcommand)]
enum TrayCommand {
    /// Start tskmstr-tray automatically when you log in (or stop doing so)
    Autostart {
        #[arg(value_enum)]
        action: autostart::Action,
    },

    /// Write the application icon (.png set, .icns, .ico) for packaging
    #[command(hide = true)]
    RenderIcon {
        #[arg(long)]
        out_dir: std::path::PathBuf,
    },
}

fn main() -> Result<(), anyhow::Error> {
    let args = Cli::parse();

    if let Some(TrayCommand::Autostart { action }) = args.cmd {
        // pass the same --config / --refresh-secs through to the autostarted instance
        let mut extra = Vec::new();
        if let Some(c) = &args.config {
            extra.push("--config".to_string());
            extra.push(c.clone());
        }
        if args.refresh_secs != 300 {
            extra.push("--refresh-secs".to_string());
            extra.push(args.refresh_secs.to_string());
        }
        return autostart::run(action, &extra);
    }
    if let Some(TrayCommand::RenderIcon { out_dir }) = &args.cmd {
        return assets::run(out_dir);
    }

    let config_path = resolve_config_path(&args.config);
    let config = load_config(&config_path)?;

    let level = if args.debug || config.debug.is_some() {
        log::Level::Debug
    } else {
        log::Level::Info
    };
    simple_logger::init_with_level(level).expect("Failed to initialize logger");

    let viewport = eframe::egui::ViewportBuilder::default()
        .with_title("tskmstr")
        .with_inner_size(app::WINDOW_SIZE)
        .with_min_inner_size(app::WINDOW_SIZE)
        .with_decorations(false)
        .with_resizable(false)
        .with_always_on_top()
        .with_taskbar(false)
        .with_active(false)
        .with_visible(false);

    #[allow(unused_mut)]
    let mut options = eframe::NativeOptions {
        viewport,
        centered: false,
        persist_window: false,
        ..Default::default()
    };

    // macOS: run as an "accessory" app so there is no Dock icon / app menu,
    // only the menu bar item.
    #[cfg(target_os = "macos")]
    {
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
        options.event_loop_builder = Some(Box::new(|builder| {
            builder.with_activation_policy(ActivationPolicy::Accessory);
        }));
    }

    let refresh_interval = Duration::from_secs(args.refresh_secs.max(15));
    let open_on_start = args.open;

    eframe::run_native(
        "tskmstr",
        options,
        Box::new(move |cc| {
            Ok(Box::new(app::TrayApp::new(
                cc,
                config,
                refresh_interval,
                open_on_start,
            )))
        }),
    )
    .map_err(|e| anyhow!("{e}"))
}
