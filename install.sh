#!/bin/sh
set -e
id=io.github.meowsystem.Soneca
bin=meowsystem-soneca
cd "$(dirname "$0")"

if [ "${1:-}" = "--remover" ]; then
  prefix="${2:-$HOME/.local}"
  rm -f "$prefix/share/applications/$id.desktop"
  rm -f "$prefix/share/icons/hicolor/scalable/apps/$id-symbolic.svg"
  rm -f "$prefix/bin/$bin"
  echo "Removido de $prefix."
  exit 0
fi

prefix="${1:-$HOME/.local}"
cargo build --release --locked
alvo="${CARGO_TARGET_DIR:-target}"
install -Dm0755 "$alvo/release/$bin" "$prefix/bin/$bin"
install -Dm0644 "data/icones/$id-symbolic.svg" "$prefix/share/icons/hicolor/scalable/apps/$id-symbolic.svg"
mkdir -p "$prefix/share/applications"
sed "s|@bindir@|$prefix/bin|" "data/$id.desktop.in" > "$prefix/share/applications/$id.desktop"
echo "Instalado. Acrescente \"Soneca\" em Configurações > Área de trabalho > Painel > Applets."
