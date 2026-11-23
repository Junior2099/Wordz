#!/usr/bin/env bash
# Build WordCraft for Windows x64 on Linux (MinGW) and package it as an MSI with wixl:
#
#   $DIST/wordcraft-<version>-windows-x64.msi
#   $DIST/wordcraft-<version>-windows-x64-portable.zip
#
# Usage: packaging/windows/msi-linux.sh [--skip-build]
#
# Needs: sudo apt install mingw-w64 wixl zip; rustup target add x86_64-pc-windows-gnu.
# Set CRAFT_I18N_DIR to a craft-i18n checkout for the translated interface.
# A simplified installer (wordcraft-wixl.wxs); the official one is package.ps1 on Windows.
set -euo pipefail
# shellcheck source=../env.sh
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"
TARGET=x86_64-pc-windows-gnu
[[ "${1:-}" == "--skip-build" ]] || (
  cd "$ROOT"
  export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS="-C target-feature=+crt-static"
  cargo build --release --locked -p wordcraft -p wordcraft-cli --target "$TARGET"
)
BIN="$CARGO_TARGET_DIR/$TARGET/release"
MSI_VERSION="${VERSION%%-*}"
BASENAME="wordcraft-$VERSION-windows-x64"

wixl -a x64 -D "Version=$MSI_VERSION" -D "BinDir=$BIN" -D "IconPath=$ROOT/assets/app-icon/wordcraft.ico" \
  -o "$DIST/$BASENAME.msi" "$ROOT/packaging/windows/wordcraft-wixl.wxs"

rm -f "$DIST/$BASENAME-portable.zip"
zip -j -q "$DIST/$BASENAME-portable.zip" "$BIN/wordcraft.exe" "$BIN/wordcraft-cli.exe"
echo "==> $DIST/$BASENAME.msi"
echo "==> $DIST/$BASENAME-portable.zip"
