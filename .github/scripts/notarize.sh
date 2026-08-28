#!/usr/bin/env bash
# Submits the image for notarisation and staples the result. Without it macOS
# warns about an unidentified developer even for a properly signed app. The
# ticket is stapled to the file, so later checks work offline.
set -euo pipefail

DMG="$1"

if [ -z "${NOTARY_KEY_ID:-}" ]; then
  echo "notarisation skipped: secrets are not set"
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
