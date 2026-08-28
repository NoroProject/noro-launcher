#!/usr/bin/env bash
# Отправляет образ на нотаризацию и пришивает результат.
#
# Без нотаризации macOS показывает предупреждение о неизвестном разработчике
# даже у правильно подписанного приложения. Штамп пришивается к самому файлу,
# поэтому проверка потом проходит и без интернета.
set -euo pipefail

DMG="$1"

if [ -z "${NOTARY_KEY_ID:-}" ]; then
  echo "нотаризация пропущена: секреты не заданы"
  exit 0
fi

KEY_FILE="$(mktemp).p8"
trap 'rm -f "$KEY_FILE"' EXIT
printf '%s' "$NOTARY_KEY_P8" | base64 --decode > "$KEY_FILE"

xcrun notarytool submit "$DMG" \
  --key "$KEY_FILE" \
  --key-id "$NOTARY_KEY_ID" \
  --issuer "$NOTARY_ISSUER_ID" \
  --wait --timeout 30m

xcrun stapler staple "$DMG"
xcrun stapler validate "$DMG"
