#!/usr/bin/env bash
# Wrapper to invoke scripts/build-and-deploy-agents.sh
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "${SCRIPT_DIR}/../scripts/build-and-deploy-agents.sh" "$@"
