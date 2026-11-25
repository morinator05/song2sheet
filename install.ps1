# Builds song2sheet and installs it for your user on Windows: the program goes
# to %LOCALAPPDATA%\Programs\song2sheet, with a shortcut in the Start menu.
#
#   powershell -ExecutionPolicy Bypass -File install.ps1               install
#   powershell -ExecutionPolicy Bypass -File install.ps1 -Uninstall    remove it again
param([switch]$Uninstall)

$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot

$name = 'song2sheet'
$dir = Join-Path $env:LOCALAPPDATA "Programs\$name"
$exe = Join-Path $dir "$name.exe"
$shortcut = Join-Path ([Environment]::GetFolderPath('Programs')) "$name.lnk"

if ($Uninstall) {
    # Fails (instead of claiming success) if song2sheet is still running.
    $dir, $shortcut | Where-Object { Test-Path -LiteralPath $_ } | Remove-Item -Recurse -Force
    "Removed $dir and $shortcut"
    exit
}

Get-Command cargo | Out-Null  # stops here with a clear error if Rust is missing
# cargo reports its progress on stderr, which must not count as an error.
$ErrorActionPreference = 'Continue'
cargo build --release -p $name
$ErrorActionPreference = 'Stop'
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$target = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { 'target' }
New-Item -ItemType Directory -Force $dir | Out-Null
Copy-Item (Join-Path $target "release\$name.exe") $exe

$link = (New-Object -ComObject WScript.Shell).CreateShortcut($shortcut)
$link.TargetPath = $exe
$link.Save()

"Installed $exe and $shortcut"
