#!/bin/sh
# Builds song2sheet and installs the command and its app menu entry.
#
#   ./install.sh                     for your user, into ~/.local
#   ./install.sh --system            for all users, into /usr/local (uses sudo)
#   ./install.sh --uninstall         remove it again (add --system for /usr/local)
set -eu

cd "$(dirname "$0")"

prefix="$HOME/.local"
sudo=""
uninstall=false
for arg in "$@"; do
    case "$arg" in
        --system) prefix="/usr/local" ;;
        --uninstall) uninstall=true ;;
        *) echo "unknown option: $arg" >&2; exit 1 ;;
    esac
done
if [ ! -w "$(dirname "$prefix")" ] || { [ -d "$prefix" ] && [ ! -w "$prefix" ]; }; then
    [ "$(id -u)" -eq 0 ] || sudo="sudo"
fi

bin="$prefix/bin/song2sheet"
desktop="$prefix/share/applications/song2sheet.desktop"

if $uninstall; then
    $sudo rm -f "$bin" "$desktop"
    echo "Removed $bin and $desktop"
    exit 0
fi

cargo build --release -p song2sheet
$sudo install -Dm755 "${CARGO_TARGET_DIR:-target}/release/song2sheet" "$bin"

# An absolute Exec path, so the launcher works even if $prefix/bin is not
# on the desktop session's PATH.
entry="$(mktemp)"
trap 'rm -f "$entry"' EXIT
sed "s|^Exec=song2sheet|Exec=$bin|" assets/song2sheet.desktop > "$entry"
$sudo install -Dm644 "$entry" "$desktop"

echo "Installed $bin and $desktop"
case ":$PATH:" in
    *":$prefix/bin:"*) ;;
    *) echo "Note: $prefix/bin is not on your PATH, so add it to run song2sheet from a terminal." ;;
esac
