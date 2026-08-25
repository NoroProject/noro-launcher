#!/usr/bin/env bash
# Retries "launch instance from boot volume" until OCI frees up capacity.
# A boot volume is AD-local, so each AD needs its own copy: TARGETS maps AD=volume.
# Config: scripts/oci-launch.env (copy from oci-launch.env.example).
set -uo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="${ENV_FILE:-$DIR/oci-launch.env}"
[ -f "$ENV_FILE" ] && . "$ENV_FILE"

: "${OCI_PROFILE:=noro-api}"
: "${OCI_AUTH:=api_key}" # api_key never expires; security_token dies in ~24h
: "${SHAPE:=VM.Standard.A1.Flex}"
: "${SHAPE_CONFIGS:=1:1 2:12}" # "ocpus:memory" pairs, smallest first
: "${ASSIGN_PUBLIC_IP:=true}"
: "${SLEEP_OK:=120}"       # pause between calls; faster than this earns 429s
: "${SLEEP_THROTTLE:=900}" # pause after HTTP 429
: "${DISPLAY_NAME:=noro-prod}"
: "${LOG_FILE:=$DIR/oci-retry-launch.log}"

for v in COMPARTMENT_ID SUBNET_ID TARGETS; do
  [ -n "${!v:-}" ] || { echo "ERROR: $v is not set (see $ENV_FILE)"; exit 2; }
done

log() { printf '%s %s\n' "$(date '+%F %T')" "$*" | tee -a "$LOG_FILE"; }
oci_() {
  if [ "$OCI_AUTH" = security_token ]; then
    oci --profile "$OCI_PROFILE" --auth security_token "$@"
  else
    oci --profile "$OCI_PROFILE" "$@"
  fi
}

ensure_session() {
  oci_ iam region list >/dev/null 2>&1 && return 0
  # An API key cannot expire, so a failure here is a blip or network trouble.
  if [ "$OCI_AUTH" != security_token ]; then
    for _ in 1 2 3; do
      sleep 10
      oci_ iam region list >/dev/null 2>&1 && return 0
    done
    log "WARN: API unreachable, will retry"
    sleep "$SLEEP_THROTTLE"
    return 0
  fi
  log "session stale, refreshing"
  if oci session refresh --profile "$OCI_PROFILE" </dev/null >>"$LOG_FILE" 2>&1 &&
     oci_ iam region list >/dev/null 2>&1; then
    log "session refreshed"; return 0
  fi
  log "FATAL: session dead. Run: oci session authenticate --profile-name $OCI_PROFILE"
  return 1
}

launch() { # $1=AD $2=boot volume OCID $3=ocpus $4=memory -> output + exit code
  local args=(compute instance launch
    --availability-domain "$1"
    --compartment-id "$COMPARTMENT_ID"
    --shape "$SHAPE"
    --source-details "{\"sourceType\":\"bootVolume\",\"bootVolumeId\":\"$2\"}"
    --subnet-id "$SUBNET_ID"
    --assign-public-ip "$ASSIGN_PUBLIC_IP"
    --display-name "$DISPLAY_NAME")
  case "$SHAPE" in
    *.Flex) args+=(--shape-config "{\"ocpus\":$3,\"memoryInGBs\":$4}") ;;
  esac
  # --no-retry: the CLI would retry 5xx internally for minutes, pinning one AD.
  oci_ --no-retry "${args[@]}" 2>&1
}

notify() { # $1=text; silent no-op unless both Telegram vars are set
  [ -n "${TELEGRAM_TOKEN:-}" ] && [ -n "${TELEGRAM_CHAT_ID:-}" ] || return 0
  curl -sS --max-time 20 -o /dev/null \
    "https://api.telegram.org/bot$TELEGRAM_TOKEN/sendMessage" \
    -d chat_id="$TELEGRAM_CHAT_ID" --data-urlencode text="$1" 2>/dev/null
}

on_success() { # $1=launch output
  local id ip
  id=$(printf '%s' "$1" | grep -o 'ocid1\.instance\.[a-z0-9.-]*' | head -1)
  log "SUCCESS: instance $id"
  printf '%s\n' "$id" > "$DIR/.last-instance-ocid"
  oci_ compute instance get --instance-id "$id" --wait-for-state RUNNING >/dev/null 2>&1
  ip=$(oci_ compute instance list-vnics --instance-id "$id" \
    --query 'data[0]."public-ip"' --raw-output 2>/dev/null)
  log "instance RUNNING, public IP: ${ip:-<none>}"
  notify "OCI instance is up.
IP: ${ip:-<none>}
ssh ubuntu@${ip:-?}
attempts: $attempt, waited $((($(date +%s) - start) / 60)) min
$id"
}

ensure_session || exit 3
log "targets: $TARGETS"
log "shape=$SHAPE configs='$SHAPE_CONFIGS' name=$DISPLAY_NAME"

attempt=0
start=$(date +%s)
last_refresh=$start

# One flat rotation of AD x config. Bursting several launches and then sleeping
# earns 429s from OCI; spacing every single call evenly does not.
combos=()
for cfg in $SHAPE_CONFIGS; do
  for target in $TARGETS; do
    combos+=("${target%%=*}|${target#*=}|${cfg%%:*}|${cfg#*:}")
  done
done
log "rotation of ${#combos[@]} combinations, one call every ${SLEEP_OK}s"

while :; do
    IFS='|' read -r ad vol cpu mem <<< "${combos[$((attempt % ${#combos[@]}))]}"
    attempt=$((attempt + 1))

    out=$(launch "$ad" "$vol" "$cpu" "$mem")
    rc=$?

    if [ $rc -eq 0 ]; then
      on_success "$out"
      exit 0
    fi

    case "$out" in
      *"Out of host capacity"*|*"out of capacity"*|*"Out of capacity"*)
        printf '%s #%d %s %sc/%sG: no capacity\n' \
          "$(date '+%T')" "$attempt" "$ad" "$cpu" "$mem" >> "$LOG_FILE"
        printf '\r%s attempt %d, %s %sc/%sG: no capacity (%dm elapsed)  ' \
          "$(date '+%T')" "$attempt" "${ad##*:}" "$cpu" "$mem" \
          "$((($(date +%s) - start) / 60))"
        ;;
      # Match the status field, not a bare number: request ids contain digits
      # like 429 and would otherwise trigger a bogus five-minute sleep.
      *TooManyRequests*|*'"status": 429'*)
        log "#$attempt $ad: throttled, sleeping ${SLEEP_THROTTLE}s"
        log "  $(printf '%s' "$out" | grep -E '"(code|status)"' | tr -d '\n')"
        sleep "$SLEEP_THROTTLE"
        ;;
      *NotAuthenticated*|*'"status": 401'*)
        ensure_session || exit 3
        ;;
      *"LimitExceeded"*|*"QuotaExceeded"*)
        log "FATAL #$attempt $ad: quota/limit exceeded"; log "$out"
        notify "OCI hunt stopped: quota/limit exceeded on $ad"; exit 5
        ;;
      *)
        log "FATAL #$attempt $ad: unexpected error"; log "$out"
        notify "OCI hunt stopped: unexpected error on $ad. Check the log."; exit 6
        ;;
    esac

  sleep "$SLEEP_OK"

  now=$(date +%s)
  if [ $((now - last_refresh)) -ge 1500 ]; then
    ensure_session || exit 3
    last_refresh=$now
  fi
done
