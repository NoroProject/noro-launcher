#!/usr/bin/env bash
# Собирает .app вокруг bootstrapper'а.
#
# Вынесено из workflow: раньше plist лежал в heredoc внутри YAML, и отступы YAML
# заезжали внутрь XML — перед <?xml оказывались пробелы, которых стандарт не
# допускает.
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

# Переподписать бандл целиком.
#
# Линковщик подписывает голый бинарник ad-hoc, и на arm64 без подписи macOS
# приложение вообще не запускает. Но как только бинарник оборачивается в бандл с
# Info.plist и иконкой, та подпись перестаёт соответствовать содержимому: она
# утверждает, что ресурсов нет. Система считает такой бандл повреждённым и
# предлагает выбросить его в корзину.
#
# MACOS_SIGN_IDENTITY позволяет подставить настоящий Developer ID; по умолчанию
# подпись ad-hoc — она не убирает предупреждение Gatekeeper о неизвестном
# разработчике, но делает приложение запускаемым.
IDENTITY="${MACOS_SIGN_IDENTITY:--}"

if [ "$IDENTITY" = "-" ]; then
  codesign --force --deep --sign - --timestamp=none "$APP"
else
  # Нотаризация принимает только сборки с hardened runtime и защищённой
  # меткой времени; без --options runtime Apple отклонит пакет.
  codesign --force --deep --sign "$IDENTITY" --options runtime --timestamp "$APP"
fi

codesign --verify --deep --strict "$APP"

