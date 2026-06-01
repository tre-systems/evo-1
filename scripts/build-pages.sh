#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"

"${ROOT_DIR}/scripts/build-wasm.sh"

rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}/pkg"

cp "${ROOT_DIR}/index.html" "${DIST_DIR}/index.html"
cp "${ROOT_DIR}/LICENSE" "${DIST_DIR}/LICENSE"
cp -R "${ROOT_DIR}/pkg/." "${DIST_DIR}/pkg/"
cp "${ROOT_DIR}/public/_headers" "${DIST_DIR}/_headers"

echo "Built Cloudflare Pages bundle in ${DIST_DIR}"
