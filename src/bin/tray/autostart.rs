//! `tskmstr-tray autostart enable|disable|status`: register the widget to
//! start when the user logs in, using each platform's native mechanism.
//!
//! - macOS: a LaunchAgent plist in `~/Library/LaunchAgents`
//! - Windows: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
//! - Linux / BSD: an XDG autostart `.desktop` entry in `~/.config/autostart`

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

pub const LABEL: &str = "com.thebuckland.tskmstr-tray";

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Action {
    /// Start tskmstr-tray automatically at login
    Enable,
    /// Stop starting tskmstr-tray at login
    Disable,
    /// Report whether autostart is configured
    Status,
}

pub fn run(action: Action, extra_args: &[String]) -> Result<()> {
    let exe = std::env::current_exe().context("cannot determine the path of tskmstr-tray")?;
    let exe = exe.canonicalize().unwrap_or(exe);
    match action {
        Action::Enable => {
            enable(&exe, extra_args)?;
            println!("tskmstr-tray will start at login ({})", location_hint());
        }
        Action::Disable => {
            disable()?;
            println!("tskmstr-tray will no longer start at login");
        }
        Action::Status => {
            if is_enabled() {
                println!("enabled ({})", location_hint());
            } else {
                println!("disabled");
            }
        }
    }
    Ok(())
}

fn home() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("HOME is not set"))
}

// ---------------------------------------------------------------- macOS ----
#[cfg(target_os = "macos")]
fn plist_path() -> Result<PathBuf> {
    Ok(home()?
        .join("Library/LaunchAgents")
        .join(format!("{LABEL}.plist")))
}

#[cfg(target_os = "macos")]
fn location_hint() -> String {
    plist_path()
        .map(|p| p.display().to_string())
        .unwrap_or_default()
}

#[cfg(target_os = "macos")]
fn enable(exe: &std::path::Path, extra_args: &[String]) -> Result<()> {
    let path = plist_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut args = vec![exe.display().to_string()];
    args.extend(extra_args.iter().cloned());
    let args_xml: String = args
        .iter()
        .map(|a| format!("        <string>{}</string>\n", xml_escape(a)))
        .collect();
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LABEL}</string>
    <key>ProgramArguments</key>
    <array>
{args_xml}    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
    <key>ProcessType</key>
    <string>Interactive</string>
    <key>StandardOutPath</key>
    <string>/tmp/tskmstr-tray.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/tskmstr-tray.log</string>
</dict>
</plist>
"#
    );
    std::fs::write(&path, plist).with_context(|| format!("writing {}", path.display()))?;
    // (Re)load it so it also takes effect for the current session; ignore
    // failures here since the file alone is enough for the next login.
    let _ = launchctl(&["bootout", &format!("gui/{}/{LABEL}", uid())]);
    let _ = launchctl(&[
        "bootstrap",
        &format!("gui/{}", uid()),
        &path.display().to_string(),
    ]);
    Ok(())
}

#[cfg(target_os = "macos")]
fn disable() -> Result<()> {
    let path = plist_path()?;
    let _ = launchctl(&["bootout", &format!("gui/{}/{LABEL}", uid())]);
    if path.exists() {
        std::fs::remove_file(&path).with_context(|| format!("removing {}", path.display()))?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn is_enabled() -> bool {
    plist_path().map(|p| p.exists()).unwrap_or(false)
}

#[cfg(target_os = "macos")]
fn uid() -> u32 {
    // `id -u` avoids pulling in libc just for getuid()
    std::process::Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(501)
}

#[cfg(target_os = "macos")]
fn launchctl(args: &[&str]) -> Result<()> {
    let status = std::process::Command::new("launchctl")
        .args(args)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("launchctl {:?} failed: {status}", args))
    }
}

// -------------------------------------------------------------- Windows ----
#[cfg(target_os = "windows")]
const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
#[cfg(target_os = "windows")]
const RUN_VALUE: &str = "tskmstr-tray";

#[cfg(target_os = "windows")]
fn location_hint() -> String {
    format!(r"{RUN_KEY}\{RUN_VALUE}")
}

#[cfg(target_os = "windows")]
fn enable(exe: &std::path::Path, extra_args: &[String]) -> Result<()> {
    let mut cmd = format!("\"{}\"", exe.display());
    for a in extra_args {
        cmd.push(' ');
        cmd.push_str(&format!("\"{a}\""));
    }
    reg(&[
        "add", RUN_KEY, "/v", RUN_VALUE, "/t", "REG_SZ", "/d", &cmd, "/f",
    ])
}

#[cfg(target_os = "windows")]
fn disable() -> Result<()> {
    if is_enabled() {
        reg(&["delete", RUN_KEY, "/v", RUN_VALUE, "/f"])?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn is_enabled() -> bool {
    std::process::Command::new("reg")
        .args(["query", RUN_KEY, "/v", RUN_VALUE])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn reg(args: &[&str]) -> Result<()> {
    let out = std::process::Command::new("reg").args(args).output()?;
    if out.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "reg {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

// ---------------------------------------------------------- Linux / BSD ----
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn desktop_path() -> Result<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or(home()?.join(".config"));
    Ok(base.join("autostart").join("tskmstr-tray.desktop"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn location_hint() -> String {
    desktop_path()
        .map(|p| p.display().to_string())
        .unwrap_or_default()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn enable(exe: &std::path::Path, extra_args: &[String]) -> Result<()> {
    let path = desktop_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut exec = format!("\"{}\"", exe.display());
    for a in extra_args {
        exec.push(' ');
        exec.push_str(&format!("\"{a}\""));
    }
    let entry = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=tskmstr tray\n\
         Comment=Open tasks from GitHub, GitLab and Jira in your tray\n\
         Exec={exec}\n\
         Icon=tskmstr-tray\n\
         Terminal=false\n\
         X-GNOME-Autostart-enabled=true\n"
    );
    std::fs::write(&path, entry).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn disable() -> Result<()> {
    let path = desktop_path()?;
    if path.exists() {
        std::fs::remove_file(&path).with_context(|| format!("removing {}", path.display()))?;
    }
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn is_enabled() -> bool {
    desktop_path().map(|p| p.exists()).unwrap_or(false)
}

#[allow(dead_code)]
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
