#!/usr/bin/env bash
# ==============================================================================
# Noro Launcher — Agent & Subproject Build and Deploy Script
# Builds all agents, server wrapper, client core, staff, player mods & uploads
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

LOCAL_DATA_DIR="${ROOT_DIR}/data/agents"
REMOTE_IP=""
REMOTE_USER="ubuntu"
REMOTE_KEY="${HOME}/.ssh/main"
REMOTE_DIR="/home/ubuntu/noro-data/agents"
BUILD_ONLY=false
DEPLOY_LOCAL=true

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Builds all Noro agent subprojects (agents, wrapper, client core, staff, player)
and deploys them locally or to a remote master server.

Options:
  -l, --local           Deploy to local data/agents directory
  -ip, --ip IP          Target remote master server IP or hostname
  -u, --user USER       SSH username for remote server (default: ubuntu)
  -k, --key PATH        SSH key path for remote server (default: ~/.ssh/main)
  -r, --remote-dir DIR  Remote data directory (default: /home/ubuntu/noro-data/agents)
  -b, --build-only      Only compile and collect artifacts without deploying
  -h, --help            Show this help message

Examples:
  $(basename "$0")                              # Build & deploy locally
  $(basename "$0") --ip 82.70.40.105            # Build & deploy to remote server
  $(basename "$0") --ip 82.70.40.105 --local    # Build & deploy both locally and remotely
EOF
    exit 0
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help) usage ;;
        -l|--local) DEPLOY_LOCAL=true; shift ;;
        -ip|--ip) REMOTE_IP="$2"; shift 2 ;;
        -u|--user) REMOTE_USER="$2"; shift 2 ;;
        -k|--key) REMOTE_KEY="$2"; shift 2 ;;
        -r|--remote-dir) REMOTE_DIR="$2"; shift 2 ;;
        -b|--build-only) BUILD_ONLY=true; shift ;;
        *)
            if [[ -z "$REMOTE_IP" && "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
                REMOTE_IP="$1"
                shift
            else
                echo "Error: Unknown argument $1" >&2
                usage
            fi
            ;;
    esac
done

echo "==> Setting up Java environment..."
if [[ -z "${JAVA_HOME:-}" ]]; then
    if [[ "$(uname)" == "Darwin" ]] && command -v /usr/libexec/java_home &>/dev/null; then
        export JAVA_HOME="$(/usr/libexec/java_home -v 21 2>/dev/null || /usr/libexec/java_home)"
    fi
fi
if [[ -n "${JAVA_HOME:-}" ]]; then
    echo "    Using JAVA_HOME: ${JAVA_HOME}"
fi

echo "==> Building all agent subprojects & collecting artifacts..."
BUILT_DIR="${ROOT_DIR}/agent/build/agents"
cd "${ROOT_DIR}/agent"
./gradlew collectAgents

if [[ ! -d "$BUILT_DIR" ]]; then
    echo "Error: Artifacts directory $BUILT_DIR not found!" >&2
    exit 1
fi

echo "==> Building vendor mods (StyledChat, TAB)..."
if [[ -d "${ROOT_DIR}/vendor/StyledChat" ]]; then
    (
        cd "${ROOT_DIR}/vendor/TextPlaceholderAPI" && ./gradlew publishToMavenLocal -q
        cd "${ROOT_DIR}/vendor/PredicateAPI" && ./gradlew publishToMavenLocal -q
        cd "${ROOT_DIR}/vendor/PlayerDataAPI" && ./gradlew publishToMavenLocal -q
        cd "${ROOT_DIR}/vendor/StyledChat" && ./gradlew build -q
        find "${ROOT_DIR}/vendor/StyledChat/build/libs" -maxdepth 1 -name "styled-chat-*.jar" ! -name "*sources*" -exec cp -f {} "${BUILT_DIR}/chat-1.21.1-neoforge.jar" \;
    ) || echo "Warning: StyledChat build skipped or failed"
fi

if [[ -d "${ROOT_DIR}/vendor/TAB" ]]; then
    (
        cd "${ROOT_DIR}/vendor/TAB"
        JAVA_HOME=$(/usr/libexec/java_home -v 21 2>/dev/null || echo "$JAVA_HOME")
        export JAVA_HOME
        ./gradlew :api:build :shared:build -q 2>/dev/null || true
        ./gradlew :neoforge:jar :bukkit:jar -q 2>/dev/null || true
        find "${ROOT_DIR}/vendor/TAB/neoforge/build/libs" -maxdepth 1 -name "TAB-neoforge-*.jar" ! -name "*sources*" ! -name "*dev*" -exec cp -f {} "${BUILT_DIR}/tab-1.21.1-neoforge.jar" \;
        find "${ROOT_DIR}/vendor/TAB/bukkit/build/libs" -maxdepth 1 -name "TAB-bukkit-*.jar" ! -name "*sources*" -exec cp -f {} "${BUILT_DIR}/tab-paper.jar" \;
    ) || echo "Warning: TAB build skipped or failed"
fi

COUNT=$(find "$BUILT_DIR" -name "*.jar" | wc -l | tr -d ' ')
echo "==> Successfully collected ${COUNT} JAR files in ${BUILT_DIR}"

if [[ "$BUILD_ONLY" == true ]]; then
    echo "==> Build-only mode complete."
    exit 0
fi

if [[ -n "$REMOTE_IP" ]]; then
    echo "==> Deploying artifacts to remote server ${REMOTE_USER}@${REMOTE_IP}:${REMOTE_DIR}..."
    ssh -i "$REMOTE_KEY" -o StrictHostKeyChecking=no "${REMOTE_USER}@${REMOTE_IP}" "mkdir -p '${REMOTE_DIR}'"
    rsync -avz -e "ssh -i ${REMOTE_KEY} -o StrictHostKeyChecking=no" "${BUILT_DIR}/" "${REMOTE_USER}@${REMOTE_IP}:${REMOTE_DIR}/"
    echo "==> Remote deployment finished successfully!"
fi

if [[ "$DEPLOY_LOCAL" == true || -z "$REMOTE_IP" ]]; then
    echo "==> Deploying artifacts locally to ${LOCAL_DATA_DIR}..."
    mkdir -p "$LOCAL_DATA_DIR"
    cp -f "${BUILT_DIR}"/*.jar "$LOCAL_DATA_DIR/"
    echo "==> Local deployment finished successfully!"
fi

echo "==> All done! Master will serve these artifacts via /api/admin/agents (admin/wrapper)."
