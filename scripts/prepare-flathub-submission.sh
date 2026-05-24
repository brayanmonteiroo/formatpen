#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="${FLATHUB_SUBMISSION_DIR:-$HOME/flathub-formatpen}"
MANIFEST_SRC="${ROOT}/flatpak/com.formatpen.FormatPen.flathub.yml"
CARGO_SOURCES="${ROOT}/cargo-sources.json"

if [[ ! -f "${CARGO_SOURCES}" ]]; then
  echo "Gere cargo-sources.json primeiro:" >&2
  echo "  python3 /tmp/flatpak-builder-tools/cargo/flatpak-cargo-generator.py \\" >&2
  echo "    ${ROOT}/Cargo.lock -o ${CARGO_SOURCES}" >&2
  exit 1
fi

mkdir -p "${DEST}"
cp "${MANIFEST_SRC}" "${DEST}/com.formatpen.FormatPen.yml"
cp "${CARGO_SOURCES}" "${DEST}/cargo-sources.json"

echo "==> Pasta Flathub pronta: ${DEST}"
echo "    - com.formatpen.FormatPen.yml"
echo "    - cargo-sources.json"
echo ""
echo "Testar build (sem --user; evita conflito com remoto formatpen-local):"
echo "  cd ${DEST}"
echo "  flatpak-builder --force-clean --install-deps-from=flathub \\"
echo "    _build com.formatpen.FormatPen.yml"
