# Builds song2sheet and installs it for your user on Windows: the program goes
# to %LOCALAPPDATA%\Programs\song2sheet, with a shortcut in the Start menu.
#
#   powershell -ExecutionPolicy Bypass -File install.ps1               install
#   powershell -ExecutionPolicy Bypass -File install.ps1 -Uninstall    remove it again
param([switch]$Uninstall)

Set-Location $PSScriptRoot

$name = 'song2sheet'
$dir = Join-Path $env:LOCALAPPDATA "Programs\$name"
$exe = Join-Path $dir "$name.exe"
$shortcut = Join-Path ([Environment]::GetFolderPath('Programs')) "$name.lnk"

if ($Uninstall) {
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $dir, $shortcut
    "Removed $dir and $shortcut"
    exit
}

cargo build --release -p $name
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$target = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { 'target' }
New-Item -ItemType Directory -Force -ErrorAction Stop $dir | Out-Null
Copy-Item -ErrorAction Stop (Join-Path $target "release\$name.exe") $exe

$link = (New-Object -ComObject WScript.Shell).CreateShortcut($shortcut)
$link.TargetPath = $exe
$link.Save()

"Installed $exe and $shortcut"
