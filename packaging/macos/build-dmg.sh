#!/usr/bin/env bash
# usage: build-dmg.sh <staging dir> <output.dmg>
set -euo pipefail
STAGING="$1"; OUT="$2"
rm -f "$OUT"
hdiutil create -volname "tskmstr" -srcfolder "$STAGING" -ov -format UDZO "$OUT" >/dev/null
echo "wrote $OUT"
