#!/usr/bin/env bash
# Build a "thin" AppImage: ships the FormatPen binary plus desktop metadata only.
# GTK4, Libadwaita, UDisks2 and mkfs tools come from the host system.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

APP_ID="com.formatpen.FormatPen"
BINARY="formatpen"
APPDIR="$ROOT/AppDir"
VERSION="$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')"
OUTPUT="$ROOT/FormatPen-${VERSION}-x86_64.AppImage"
APPIMAGETOOL="${APPIMAGETOOL:-$ROOT/appimagetool}"

DESKTOP="$ROOT/data/${APP_ID}.desktop"
ICON="$ROOT/data/icons/hicolor/scalable/apps/${APP_ID}.svg"

if [[ ! -f "$DESKTOP" ]]; then
  echo "Erro: desktop file não encontrado: $DESKTOP" >&2
  exit 1
fi

if [[ ! -f "$ICON" ]]; then
  echo "Erro: ícone não encontrado: $ICON" >&2
  exit 1
fi

echo "==> Compilando release..."
cargo build --release

TARGET_DIR="$(cargo metadata --format-version=1 --no-deps 2>/dev/null \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BINARY_PATH="${TARGET_DIR}/release/${BINARY}"
if [[ ! -x "$BINARY_PATH" ]]; then
  echo "Erro: binário não encontrado: $BINARY_PATH" >&2
  exit 1
fi

echo "==> Montando AppDir..."
rm -rf "$APPDIR"
mkdir -p \
  "$APPDIR/usr/bin" \
  "$APPDIR/usr/share/applications" \
  "$APPDIR/usr/share/icons/hicolor/scalable/apps"

cp "$BINARY_PATH" "$APPDIR/usr/bin/"
cp "$DESKTOP" "$APPDIR/usr/share/applications/"
cp "$ICON" "$APPDIR/usr/share/icons/hicolor/scalable/apps/"

ln -sf "usr/share/applications/${APP_ID}.desktop" "$APPDIR/${APP_ID}.desktop"
ln -sf "usr/share/icons/hicolor/scalable/apps/${APP_ID}.svg" "$APPDIR/${APP_ID}.svg"
ln -sf "usr/share/icons/hicolor/scalable/apps/${APP_ID}.svg" "$APPDIR/.DirIcon"

cat > "$APPDIR/AppRun" << 'EOF'
#!/bin/bash
# FormatPen — GTK4 + libadwaita; bibliotecas gráficas vêm do sistema host.
HERE="$(dirname "$(readlink -f "$0")")"
export XDG_DATA_DIRS="${HERE}/usr/share:/usr/share:/usr/local/share:${XDG_DATA_DIRS:-/usr/share}"
exec "${HERE}/usr/bin/formatpen" "$@"
EOF
chmod +x "$APPDIR/AppRun"

if [[ ! -x "$APPIMAGETOOL" ]]; then
  echo "==> Baixando appimagetool..."
  curl -sSL \
    "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage" \
    -o "$APPIMAGETOOL"
  chmod +x "$APPIMAGETOOL"
fi

echo "==> Gerando AppImage..."
rm -f "$OUTPUT"
ARCH=x86_64 "$APPIMAGETOOL" --appimage-extract-and-run "$APPDIR" "$OUTPUT"

if [[ ! -f "$OUTPUT" ]]; then
  echo "Erro: AppImage não foi gerado." >&2
  exit 1
fi

chmod +x "$OUTPUT"
echo "==> Pronto: $OUTPUT"
echo "    Executar: $OUTPUT"
echo "    Sem FUSE2: $OUTPUT --appimage-extract-and-run"
