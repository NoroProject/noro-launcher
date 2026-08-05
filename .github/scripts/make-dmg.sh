#!/usr/bin/env bash
# Упаковывает .app в образ для раздачи игрокам.
#
# .app — это директория, одним файлом в релиз она не кладётся. Из архивов
# выбран .dmg: игрок открывает образ и перетаскивает приложение в Applications,
# поэтому оно не остаётся навсегда лежать в Загрузках. Симлинк на /Applications
# внутри образа и делает этот жест очевидным.
set -euo pipefail

APP="$1"
OUT="$2"
VOLNAME="${3:-NoroLauncher}"

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"

# UDZO — сжатый образ только для чтения: меньше весит и не даёт случайно
# изменить содержимое после подписи.
hdiutil create \
  -volname "$VOLNAME" \
  -srcfolder "$STAGE" \
  -ov -format UDZO \
  -quiet \
  "$OUT"

hdiutil verify -quiet "$OUT"
