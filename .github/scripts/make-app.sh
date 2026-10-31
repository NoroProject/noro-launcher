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

# The game runs as this app's child, so macOS asks for the microphone on the
# app's behalf. With no usage string it refuses without asking, and voice chat
# mods hear nothing.
MIC_EN="Minecraft uses the microphone for voice chat in game."
mkdir -p "$APP/Contents/Resources/en.lproj" "$APP/Contents/Resources/ru.lproj"
printf '"NSMicrophoneUsageDescription" = "%s";\n' "$MIC_EN" > "$APP/Contents/Resources/en.lproj/InfoPlist.strings"
printf '"NSMicrophoneUsageDescription" = "%s";\n' "Minecraft использует микрофон для голосового чата в игре." > "$APP/Contents/Resources/ru.lproj/InfoPlist.strings"

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
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleLocalizations</key>
    <array>
        <string>en</string>
        <string>ru</string>
    </array>
    <key>NSMicrophoneUsageDescription</key>
    <string>${MIC_EN}</string>
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
  # Hardened runtime even ad-hoc: the master re-signs this bundle with the
  # Developer ID and keeps the flags it finds, and notarisation rejects a
  # binary without the runtime.
  codesign --force --deep --sign - --options runtime --timestamp=none \
    --entitlements .github/scripts/noro-launcher.entitlements "$APP"
else
  # Notarisation only takes hardened-runtime builds with a secure timestamp;
  # without --options runtime Apple rejects the submission.
  # The hardened runtime also blocks the microphone unless the entitlement asks for it.
  codesign --force --deep --sign "$IDENTITY" --options runtime --timestamp \
    --entitlements .github/scripts/noro-launcher.entitlements "$APP"
fi

codesign --verify --deep --strict "$APP"

