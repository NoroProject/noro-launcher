#!/usr/bin/env bash
# Packs the .app into a disk image. A .app is a directory and can't go into a
# release as one file; a .dmg also gets the app out of Downloads, since the
# player drags it across to the /Applications symlink inside the image.
set -euo pipefail

APP="$1"
OUT="$2"
VOLNAME="${3:-NoroLauncher}"

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"

# UDZO is a compressed read-only disk image: smaller size and strictly protected.
hdiutil create \
  -volname "$VOLNAME" \
  -srcfolder "$STAGE" \
  -ov -format UDZO \
  -quiet \
  "$OUT"

hdiutil verify -quiet "$OUT"

# Sign the image itself, not just the bundle inside it, or the notarisation
# ticket ends up stapled to a container nothing vouches for. Same Developer ID
# Application certificate — Developer ID Installer is only for .pkg.
IDENTITY="${MACOS_SIGN_IDENTITY:--}"
if [ "$IDENTITY" != "-" ]; then
  codesign --force --sign "$IDENTITY" --timestamp "$OUT"
  codesign --verify --strict "$OUT"
fi
