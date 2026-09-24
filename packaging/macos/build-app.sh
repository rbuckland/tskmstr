#!/usr/bin/env bash
# Assemble the macOS distribution folder:
#   <staging>/tskmstr-tray.app   (menu bar widget, LSUIElement)
#   <staging>/tskmstr            (CLI binary)
#   <staging>/README.txt
#
# usage: build-app.sh <version> <tskmstr-tray binary> <tskmstr binary> <icns> <staging dir>
set -euo pipefail

VERSION="$1"; TRAY_BIN="$2"; CLI_BIN="$3"; ICNS="$4"; STAGING="$5"
HERE="$(cd "$(dirname "$0")" && pwd)"
APP="$STAGING/tskmstr-tray.app"

rm -rf "$STAGING"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

sed "s/__VERSION__/$VERSION/g" "$HERE/Info.plist" > "$APP/Contents/Info.plist"
cp "$TRAY_BIN" "$APP/Contents/MacOS/tskmstr-tray"
cp "$ICNS" "$APP/Contents/Resources/tskmstr.icns"
chmod +x "$APP/Contents/MacOS/tskmstr-tray"
# The CLI is also shipped inside the bundle so `autostart` and PATH set-ups
# can find both binaries in one place.
cp "$CLI_BIN" "$APP/Contents/MacOS/tskmstr"
chmod +x "$APP/Contents/MacOS/tskmstr"

# ad-hoc signature so Gatekeeper at least sees a consistent bundle
codesign --force --deep --sign - "$APP" 2>/dev/null || true

cp "$CLI_BIN" "$STAGING/tskmstr"
chmod +x "$STAGING/tskmstr"
ln -s /Applications "$STAGING/Applications"

cat > "$STAGING/README.txt" <<TXT
tskmstr $VERSION - task/issue aggregation for GitHub, GitLab and Jira

1. Drag tskmstr-tray.app to Applications.
2. Copy the "tskmstr" CLI somewhere on your PATH, e.g.
       cp tskmstr /usr/local/bin/tskmstr && ln -s /usr/local/bin/tskmstr /usr/local/bin/t
   (it is also inside tskmstr-tray.app/Contents/MacOS/)
3. Create your config:   tskmstr init
4. Start at login:       /Applications/tskmstr-tray.app/Contents/MacOS/tskmstr-tray autostart enable

The binaries are not notarised. If macOS refuses to open them, run:
       xattr -dr com.apple.quarantine /Applications/tskmstr-tray.app
   or right-click the app and choose Open.

https://github.com/rbuckland/tskmstr
TXT
echo "staged $APP"
