#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
RAW_BUILD_VERSION="${EVO_ONE_BUILD_VERSION:-$(git -C "${ROOT_DIR}" rev-parse --short HEAD 2>/dev/null || date +%s)}"
BUILD_VERSION="$(printf '%s' "${RAW_BUILD_VERSION}" | tr -c 'A-Za-z0-9._-' '-')"

"${ROOT_DIR}/scripts/build-wasm.sh"

rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}/pkg"

sed \
  "s#\"./pkg/evo_1.js\"#\"./pkg/evo_1.js?v=${BUILD_VERSION}\"#" \
  "${ROOT_DIR}/index.html" > "${DIST_DIR}/index.html"
cp "${ROOT_DIR}/LICENSE" "${DIST_DIR}/LICENSE"
cp -R "${ROOT_DIR}/pkg/." "${DIST_DIR}/pkg/"
cp "${ROOT_DIR}/public/_headers" "${DIST_DIR}/_headers"

echo "Built Cloudflare Pages bundle in ${DIST_DIR} with asset version ${BUILD_VERSION}"
