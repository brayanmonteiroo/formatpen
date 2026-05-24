#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="${ROOT}/flatpak/com.formatpen.FormatPen.yml"
BUILD_DIR="${ROOT}/flatpak/.flatpak-builder"
REPO_DIR="${ROOT}/flatpak/repo"
RUNTIME_VERSION="${FLATPAK_RUNTIME_VERSION:-50}"
RUST_EXT_BRANCH="${FLATPAK_RUST_EXT_BRANCH:-25.08}"

echo "==> FormatPen Flatpak (runtime GNOME ${RUNTIME_VERSION})"

if ! command -v flatpak-builder >/dev/null; then
  echo "Instale: sudo dnf install flatpak flatpak-builder" >&2
  exit 1
fi

# Flathub no escopo do sistema (flatpak-builder sem --user usa este remoto).
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo

echo "==> Instalando runtime/SDK no sistema (pode pedir confirmação)..."
flatpak install -y flathub \
  "org.gnome.Platform//${RUNTIME_VERSION}" \
  "org.gnome.Sdk//${RUNTIME_VERSION}" \
  "org.freedesktop.Sdk.Extension.rust-stable//${RUST_EXT_BRANCH}" || true

echo "==> Compilando..."
# Sem --user: dependências vêm do Flathub do sistema (onde você já instalou o SDK 50).
flatpak-builder \
  --force-clean \
  --install-deps-from=flathub \
  --repo="${REPO_DIR}" \
  "${BUILD_DIR}" \
  "${MANIFEST}"

echo "==> Instalando no usuário..."
REPO_URI="file://${REPO_DIR}"
if flatpak --user remote-list | grep -qxF formatpen-local; then
  flatpak --user remote-modify formatpen-local url="${REPO_URI}"
  flatpak --user remote-modify formatpen-local --no-gpg-verify
else
  flatpak --user remote-add --no-gpg-verify formatpen-local "${REPO_URI}"
fi
flatpak --user install -y --reinstall formatpen-local com.formatpen.FormatPen

echo ""
echo "Pronto. Execute:"
echo "  flatpak run com.formatpen.FormatPen"
