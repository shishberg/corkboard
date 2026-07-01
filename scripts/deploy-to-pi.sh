#!/usr/bin/env bash
# Builds the editor/dashboard locally, then rsyncs the repo to the device
# (deploy-to-orange-pi.md step 10). Doesn't touch device/data — that holds
# the device's live config.json (secret feed URL) and document.json, which
# must never be overwritten by a stale local copy.
set -euo pipefail

HOST="${CORKBOARD_HOST:-david@calcifer.local}"
REMOTE_DIR="${CORKBOARD_REMOTE_DIR:-~/corkboard}"

cd "$(dirname "$0")/.."

npm run build

rsync -av \
  --exclude target \
  --exclude node_modules \
  --exclude .git \
  --exclude device/data \
  ./ "$HOST:$REMOTE_DIR/"

echo "Building on-device and restarting the service..."
# Non-interactive SSH sessions don't source ~/.profile, so cargo isn't on PATH
# unless we source its env script explicitly.
ssh "$HOST" ". \"\$HOME/.cargo/env\" && cd $REMOTE_DIR/device && cargo build --release && sudo systemctl restart corkboard"

echo "Deployed and restarted on $HOST."
