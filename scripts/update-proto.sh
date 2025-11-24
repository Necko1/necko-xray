#!/bin/bash
set -e

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"

git clone --depth=1 https://github.com/XTLS/Xray-core.git "$TMP_DIR"

mkdir -p "$ROOT_DIR/proto/app" \
         "$ROOT_DIR/proto/common" \
         "$ROOT_DIR/proto/proxy" \
         "$ROOT_DIR/proto/transport" \
         "$ROOT_DIR/proto/core"

echo "Syncing .proto files..."

RSYNC_FILTER=(--include='*/' --include='*.proto' --exclude='*')

rsync -av "${RSYNC_FILTER[@]}" \
  "$TMP_DIR/app/"       "$ROOT_DIR/proto/app/"

rsync -av "${RSYNC_FILTER[@]}" \
  "$TMP_DIR/common/"    "$ROOT_DIR/proto/common/"

rsync -av "${RSYNC_FILTER[@]}" \
  "$TMP_DIR/proxy/"     "$ROOT_DIR/proto/proxy/"

rsync -av "${RSYNC_FILTER[@]}" \
  "$TMP_DIR/transport/" "$ROOT_DIR/proto/transport/"

rsync -av "$TMP_DIR/core/config.proto" "$ROOT_DIR/proto/core/"

rm -rf "$TMP_DIR"

echo "Done. Proto files updated in proto/."
