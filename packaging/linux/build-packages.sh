#!/usr/bin/env bash
# Build Linux artefacts from already-compiled binaries.
#
# usage: build-packages.sh <version> <arch> <bindir> <assets dir> <out dir> <format>...
#   arch    : amd64 | arm64  (bindir/assets must be absolute paths)
#   format  : deb | apk | tgz
#
# Requires `nfpm` on PATH for deb/apk (https://nfpm.goreleaser.com/install/).
set -euo pipefail

VERSION="$1"; ARCH="$2"; BINDIR="$3"; ASSETS="$4"; OUT="$5"; shift 5
FORMATS=("$@")
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
mkdir -p "$OUT"

for fmt in "${FORMATS[@]}"; do
  case "$fmt" in
    deb|apk)
      CFG="$(mktemp)"
      sed -e "s|\${VERSION}|$VERSION|g" -e "s|\${ARCH}|$ARCH|g" \
          -e "s|\${BINDIR}|$BINDIR|g" -e "s|\${ASSETS}|$ASSETS|g" \
          "$ROOT/packaging/linux/nfpm.yaml.in" > "$CFG"
      ( cd "$ROOT" && nfpm package --config "$CFG" --packager "$fmt" --target "$OUT/" )
      rm -f "$CFG"
      ;;
    tgz)
      STAGE="$(mktemp -d)"
      PKG="tskmstr-${VERSION}-linux-${ARCH}"
      mkdir -p "$STAGE/$PKG"
      cp "$BINDIR/tskmstr" "$BINDIR/tskmstr-tray" "$STAGE/$PKG/"
      cp "$ROOT/README.md" "$ROOT/LICENSE" "$ROOT/packaging/linux/tskmstr-tray.desktop" "$STAGE/$PKG/"
      cp "$ASSETS/app-icon-256.png" "$STAGE/$PKG/tskmstr-tray.png"
      cat > "$STAGE/$PKG/INSTALL.txt" <<TXT
tskmstr $VERSION

  install -m 0755 tskmstr tskmstr-tray ~/.local/bin/
  ln -sf ~/.local/bin/tskmstr ~/.local/bin/t
  tskmstr init                          # write a config template
  tskmstr-tray autostart enable         # start the tray widget at login

The tray widget needs GTK 3 and an AppIndicator library at runtime
(Debian/Ubuntu: libgtk-3-0 libxdo3 libayatana-appindicator3-1).
TXT
      tar -C "$STAGE" -czf "$OUT/$PKG.tar.gz" "$PKG"
      rm -rf "$STAGE"
      ;;
    *) echo "unknown format $fmt" >&2; exit 2;;
  esac
done
ls -la "$OUT"
