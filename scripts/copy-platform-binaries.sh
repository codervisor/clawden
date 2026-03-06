#!/usr/bin/env bash
set -euo pipefail

# Copy platform binaries from GitHub Actions artifacts to the correct directories.
#
# Usage:
#   ./scripts/copy-platform-binaries.sh <artifacts-dir>
#
# Expected layout:
#   <artifacts-dir>/binaries-darwin-x64/clawden
#   <artifacts-dir>/binaries-darwin-arm64/clawden
#   <artifacts-dir>/binaries-linux-x64/clawden
#   <artifacts-dir>/binaries-windows-x64/clawden.exe

ARTIFACTS_DIR="${1:?Usage: $0 <artifacts-dir>}"
DEST_BASE="npm/clawden/binaries"

PLATFORMS=("darwin-x64" "darwin-arm64" "linux-x64" "windows-x64")

echo "📦 Copying platform binaries from ${ARTIFACTS_DIR}..."
echo ""

for platform in "${PLATFORMS[@]}"; do
  src="${ARTIFACTS_DIR}/binaries-${platform}"
  dest="${DEST_BASE}/${platform}"

  if [[ ! -d "$src" ]]; then
    echo "  ⚠️  Skipping ${platform}: ${src} not found"
    continue
  fi

  mkdir -p "$dest"

  if [[ "$platform" == windows-* ]]; then
    binary_name="clawden.exe"
  else
    binary_name="clawden"
  fi

  if [[ -f "${src}/${binary_name}" ]]; then
    cp "${src}/${binary_name}" "${dest}/${binary_name}"
    chmod +x "${dest}/${binary_name}" 2>/dev/null || true
    echo "  ✓ ${platform}/${binary_name} ($(du -h "${dest}/${binary_name}" | cut -f1))"
  else
    echo "  ✗ ${platform}/${binary_name} not found in artifacts"
  fi
done

echo ""
echo "✅ Binary copy complete"
