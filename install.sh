#!/usr/bin/env bash
# tskmstr installer for macOS and Linux.
#
#   curl -fsSL https://raw.githubusercontent.com/rbuckland/tskmstr/main/install.sh | bash
#
# Installs the `tskmstr` CLI and the `tskmstr-tray` widget from the latest
# GitHub release, adds `alias t=tskmstr` to your shell, and writes a config
# template if you have none.
#
# Environment overrides:
#   TSKMSTR_VERSION    e.g. v0.6.3            (default: latest release)
#   TSKMSTR_BIN_DIR    where the CLI goes     (default: ~/.local/bin)
#   TSKMSTR_APP_DIR    macOS: .app location   (default: /Applications, else ~/Applications)
#   TSKMSTR_AUTOSTART  1 = also register the tray widget to start at login
set -euo pipefail

REPO="rbuckland/tskmstr"
VERSION="${TSKMSTR_VERSION:-}"
BIN_DIR="${TSKMSTR_BIN_DIR:-$HOME/.local/bin}"
DOWNLOAD_BASE="${TSKMSTR_DOWNLOAD_BASE:-}"   # testing hook: file:///path or mirror

say()  { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33mwarning:\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || die "'$1' is required but not installed"; }

need curl; need tar

OS="$(uname -s)"; ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64)  ARCH=amd64 ;;
  aarch64|arm64) ARCH=arm64 ;;
esac

if [ -z "$VERSION" ]; then
  say "Looking up the latest release"
  VERSION="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
    | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -1)"
  [ -n "$VERSION" ] || die "could not determine the latest release (rate limited? set TSKMSTR_VERSION=vX.Y.Z)"
fi
V="${VERSION#v}"
[ -n "$DOWNLOAD_BASE" ] || DOWNLOAD_BASE="https://github.com/$REPO/releases/download/$VERSION"

TMP="$(mktemp -d)"
cleanup() { [ -n "${MNT:-}" ] && hdiutil detach -quiet "$MNT" 2>/dev/null || true; rm -rf "$TMP"; }
trap cleanup EXIT

fetch() { # fetch <asset name>
  say "Downloading $1"
  curl -fL --progress-bar -o "$TMP/$1" "$DOWNLOAD_BASE/$1" || die "download failed: $DOWNLOAD_BASE/$1"
}

SUDO=""
if [ "$(id -u)" -ne 0 ] && command -v sudo >/dev/null 2>&1; then SUDO="sudo"; fi

TRAY_BIN=""      # path of the installed tray binary ("" if CLI only)
TRAY_START=""    # how the user starts it
mkdir -p "$BIN_DIR"

case "$OS" in
  Darwin)
    need hdiutil
    APP_DIR="${TSKMSTR_APP_DIR:-}"
    if [ -z "$APP_DIR" ]; then
      if [ -w /Applications ]; then APP_DIR=/Applications; else APP_DIR="$HOME/Applications"; fi
    fi
    DMG="tskmstr-$V-macos-universal.dmg"
    fetch "$DMG"
    MNT="$TMP/mnt"
    hdiutil attach -nobrowse -quiet -readonly -mountpoint "$MNT" "$TMP/$DMG" || die "could not mount $DMG"
    say "Installing tskmstr-tray.app to $APP_DIR"
    mkdir -p "$APP_DIR"
    rm -rf "$APP_DIR/tskmstr-tray.app"
    cp -R "$MNT/tskmstr-tray.app" "$APP_DIR/"
    say "Installing tskmstr CLI to $BIN_DIR"
    cp "$MNT/tskmstr" "$BIN_DIR/tskmstr"
    hdiutil detach -quiet "$MNT"; MNT=""
    TRAY_BIN="$APP_DIR/tskmstr-tray.app/Contents/MacOS/tskmstr-tray"
    TRAY_START="open -a tskmstr-tray"
    ;;

  Linux)
    if [ -f /etc/alpine-release ]; then
      [ "$ARCH" = amd64 ] || die "no Alpine package for $ARCH yet; build from source (cargo build --release --features tray)"
      APK="tskmstr_${V}_x86_64.apk"
      fetch "$APK"
      say "Installing with apk (needs root)"
      $SUDO apk add --allow-untrusted "$TMP/$APK" || die "apk add failed"
      TRAY_BIN=/usr/bin/tskmstr-tray
      TRAY_START="tskmstr-tray"
    else
      case "$ARCH" in
        amd64) PKG="tskmstr-$V-linux-amd64" ;;
        arm64) PKG="tskmstr-$V-linux-arm64-cli"; warn "arm64 Linux: CLI only (no tray widget package yet)" ;;
        *) die "unsupported architecture $ARCH" ;;
      esac
      fetch "$PKG.tar.gz"
      tar -xzf "$TMP/$PKG.tar.gz" -C "$TMP"
      say "Installing to $BIN_DIR"
      install -m 0755 "$TMP/$PKG/tskmstr" "$BIN_DIR/tskmstr"
      if [ -f "$TMP/$PKG/tskmstr-tray" ]; then
        install -m 0755 "$TMP/$PKG/tskmstr-tray" "$BIN_DIR/tskmstr-tray"
        TRAY_BIN="$BIN_DIR/tskmstr-tray"
        TRAY_START="tskmstr-tray"
        # desktop entry + icon for the application menu
        DATA="${XDG_DATA_HOME:-$HOME/.local/share}"
        mkdir -p "$DATA/applications" "$DATA/icons/hicolor/256x256/apps"
        sed "s|^Exec=.*|Exec=$BIN_DIR/tskmstr-tray|" "$TMP/$PKG/tskmstr-tray.desktop" > "$DATA/applications/tskmstr-tray.desktop"
        cp "$TMP/$PKG/tskmstr-tray.png" "$DATA/icons/hicolor/256x256/apps/tskmstr-tray.png"
        command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$DATA/applications" 2>/dev/null || true
        say "Note: the tray widget needs GTK 3 + an AppIndicator library at runtime"
        say "      (Debian/Ubuntu: sudo apt install libgtk-3-0 libxdo3 libayatana-appindicator3-1)"
      fi
    fi
    ;;

  MINGW*|MSYS*|CYGWIN*|Windows_NT)
    die "on Windows use:  irm https://raw.githubusercontent.com/rbuckland/tskmstr/main/install.ps1 | iex"
    ;;
  *) die "unsupported OS: $OS" ;;
esac

# ---------------------------------------------------------------- shell ----
# `alias t=tskmstr` (and BIN_DIR on PATH if needed) in the user's shell rc files.
add_line() { # add_line <file> <line>
  grep -qxF "$2" "$1" 2>/dev/null && return 0
  printf '\n%s\n' "$2" >> "$1"
  say "Added to $1: $2"
}
RC_FILES=()
[ -f "$HOME/.bashrc" ] && RC_FILES+=("$HOME/.bashrc")
[ -f "$HOME/.zshrc" ]  && RC_FILES+=("$HOME/.zshrc")
if [ ${#RC_FILES[@]} -eq 0 ]; then
  case "${SHELL:-}" in *zsh) RC_FILES=("$HOME/.zshrc") ;; *) RC_FILES=("$HOME/.bashrc") ;; esac
fi
for rc in "${RC_FILES[@]}"; do
  touch "$rc"
  case ":$PATH:" in *":$BIN_DIR:"*) ;; *) add_line "$rc" "export PATH=\"$BIN_DIR:\$PATH\"   # tskmstr" ;; esac
  add_line "$rc" "alias t=tskmstr"
done
if [ -d "$HOME/.config/fish" ]; then
  mkdir -p "$HOME/.config/fish/conf.d"
  printf 'alias t=tskmstr\nfish_add_path %s\n' "$BIN_DIR" > "$HOME/.config/fish/conf.d/tskmstr.fish"
  say "Added fish alias in ~/.config/fish/conf.d/tskmstr.fish"
fi

# --------------------------------------------------------------- config ----
CONFIG="$HOME/.config/tskmstr/tskmstr.config.yml"
if [ ! -f "$CONFIG" ]; then
  say "Writing a config template"
  "$BIN_DIR/tskmstr" init >/dev/null 2>&1 || "$BIN_DIR/tskmstr" init || true
fi

if [ "${TSKMSTR_AUTOSTART:-0}" = "1" ] && [ -n "$TRAY_BIN" ]; then
  say "Registering the tray widget to start at login"
  "$TRAY_BIN" autostart enable || warn "autostart registration failed"
fi

echo
say "tskmstr $V installed"
echo
echo "  CLI:      $BIN_DIR/tskmstr   (alias: t  -- open a new shell or: source ${RC_FILES[0]})"
[ -n "$TRAY_BIN" ] && echo "  Tray:     $TRAY_START"
echo "  Config:   $CONFIG"
echo
echo "Next steps:"
echo "  1. Edit the config: add your repositories/projects and store API tokens in the OS keyring"
echo "     (the template explains how)."
echo "  2. t                       # list your tasks"
[ -n "$TRAY_BIN" ] && echo "  3. $TRAY_START            # start the tray widget"
[ -n "$TRAY_BIN" ] && echo "     $TRAY_BIN autostart enable   # ...and at every login"
