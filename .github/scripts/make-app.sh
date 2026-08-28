#!/usr/bin/env bash
# Builds the .app around the bootstrapper. Kept out of the workflow so the plist
# heredoc doesn't pick up YAML indentation — XML won't take spaces before <?xml.
set -euo pipefail

BIN="$1"
APP="$2"
VERSION="${3:-1.0}"

mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/noro-launcher"
chmod +x "$APP/Contents/MacOS/noro-launcher"
cp assets/icon.icns "$APP/Contents/Resources/AppIcon.icns"

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>noro-launcher</string>
    <key>CFBundleIdentifier</key>
    <string>com.nexbitstd.norolauncher</string>
    <key>CFBundleName</key>
    <string>NoroLauncher</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
</dict>
</plist>
PLIST

if command -v plutil >/dev/null 2>&1; then
  plutil -lint "$APP/Contents/Info.plist"
fi

# Re-sign the whole bundle. The linker ad-hoc signs the bare binary, and on
# arm64 macOS won't run an unsigned one at all — but once that binary is wrapped
# in a bundle with a plist and an icon, the signature no longer matches what is
# there: it claims the bundle has no resources. macOS then treats it as damaged
# and offers to move it to the trash.
#
# MACOS_SIGN_IDENTITY takes a real Developer ID. The ad-hoc default still leaves
# the Gatekeeper warning about an unidentified developer, but the app launches.
IDENTITY="${MACOS_SIGN_IDENTITY:--}"

if [ "$IDENTITY" = "-" ]; then
  codesign --force --deep --sign - --timestamp=none "$APP"
else
  # Notarisation only takes hardened-runtime builds with a secure timestamp;
  # without --options runtime Apple rejects the submission.
  codesign --force --deep --sign "$IDENTITY" --options runtime --timestamp "$APP"
fi

codesign --verify --deep --strict "$APP"

