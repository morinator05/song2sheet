#!/bin/sh
# Builds song2sheet and installs it on Linux or macOS (on Windows, use
# install.ps1). On Linux it adds an app menu entry; on macOS an app in
# Applications, with the command linked to the binary inside it.
#
#   ./install.sh                for your user, into ~/.local (and ~/Applications on macOS)
#   ./install.sh --system       for all users, into /usr/local (and /Applications), uses sudo
#   ./install.sh --uninstall    remove it again (add --system for a system install)
set -eu

cd "$(dirname "$0")"

# Building as root would leave root-owned files in target/.
if [ -n "${SUDO_USER:-}" ]; then
    echo "Run this without sudo: --system uses sudo itself, only to copy the files." >&2
    exit 1
fi

name="song2sheet"
prefix="$HOME/.local"
apps="$HOME/Applications"
sudo=""
uninstall=false
for arg in "$@"; do
    case "$arg" in
        --system) prefix="/usr/local"; apps="/Applications"; sudo="sudo" ;;
        --uninstall) uninstall=true ;;
        *) echo "unknown option: $arg" >&2; exit 1 ;;
    esac
done

bin="$prefix/bin/$name"
app="$apps/$name.app"
desktop="$prefix/share/applications/$name.desktop"
macos=false
[ "$(uname -s)" = Darwin ] && macos=true
if $macos; then installed="$bin and $app"; else installed="$bin and $desktop"; fi

if $uninstall; then
    $sudo rm -f "$bin"
    if $macos; then $sudo rm -rf "$app"; else $sudo rm -f "$desktop"; fi
    echo "Removed $installed"
    exit 0
fi

target="${CARGO_TARGET_DIR:-target}"
cargo build --release -p "$name"
built="$target/release/$name"

# put SOURCE DEST MODE: copies a file, creating its folder.
put() {
    $sudo mkdir -p "$(dirname "$2")"
    $sudo install -m "$3" "$1" "$2"
}

if $macos; then
    $sudo rm -rf "$app"
    put "$built" "$app/Contents/MacOS/$name" 755
    put assets/macos/Info.plist "$app/Contents/Info.plist" 644
    # The linker signed only the binary; sign the whole bundle ad hoc.
    $sudo codesign --force --sign - "$app"
    $sudo mkdir -p "$(dirname "$bin")"
    $sudo ln -sf "$app/Contents/MacOS/$name" "$bin"
else
    put "$built" "$bin" 755
    # An absolute, quoted Exec path, so the launcher works even if
    # $prefix/bin is not on the desktop session's PATH.
    entry="$target/$name.desktop"
    sed '/^Exec=/d' "assets/$name.desktop" > "$entry"
    printf 'Exec="%s" %%f\n' "$bin" >> "$entry"
    put "$entry" "$desktop" 644
fi

echo "Installed $installed"
case ":$PATH:" in
    *":$prefix/bin:"*) ;;
    *) echo "Note: $prefix/bin is not on your PATH, so add it to run $name from a terminal." ;;
esac
