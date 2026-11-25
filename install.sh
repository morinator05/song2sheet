#!/bin/sh
# Builds song2sheet and installs the command and its app menu entry.
#
#   ./install.sh                     for your user, into ~/.local
#   ./install.sh --system            for all users, into /usr/local (uses sudo)
#   ./install.sh --uninstall         remove it again (add --system for /usr/local)
set -eu

cd "$(dirname "$0")"

# Building as root would leave root-owned files in target/.
if [ -n "${SUDO_USER:-}" ]; then
    echo "Run this without sudo: --system uses sudo itself, only to copy the files." >&2
    exit 1
fi

prefix="$HOME/.local"
sudo=""
uninstall=false
for arg in "$@"; do
    case "$arg" in
        --system) prefix="/usr/local"; sudo="sudo" ;;
        --uninstall) uninstall=true ;;
        *) echo "unknown option: $arg" >&2; exit 1 ;;
    esac
done

bin="$prefix/bin/song2sheet"
desktop="$prefix/share/applications/song2sheet.desktop"

if $uninstall; then
    $sudo rm -f "$bin" "$desktop"
    echo "Removed $bin and $desktop"
    exit 0
fi

target="${CARGO_TARGET_DIR:-target}"
cargo build --release -p song2sheet

# An absolute, quoted Exec path, so the launcher works even if $prefix/bin is
# not on the desktop session's PATH.
entry="$target/song2sheet.desktop"
sed '/^Exec=/d' assets/song2sheet.desktop > "$entry"
printf 'Exec="%s" %%f\n' "$bin" >> "$entry"

$sudo install -Dm755 "$target/release/song2sheet" "$bin"
$sudo install -Dm644 "$entry" "$desktop"

echo "Installed $bin and $desktop"
case ":$PATH:" in
    *":$prefix/bin:"*) ;;
    *) echo "Note: $prefix/bin is not on your PATH, so add it to run song2sheet from a terminal." ;;
esac
