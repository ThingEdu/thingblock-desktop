# Build the sibling thingblock-link repo and stage its binary, the editor-produced
# thingblock-resource pack, and the host arduino-cli.exe into src-tauri/ so Tauri
# can bundle them. Windows mirror of stage-sidecar.sh; needs no Git Bash.
$ErrorActionPreference = "Stop"

$DesktopDir = (Resolve-Path "$PSScriptRoot\..").Path
$LinkDir    = (Resolve-Path "$DesktopDir\..\thingblock-link").Path
$EditorDir  = (Resolve-Path "$DesktopDir\..\thingblock-editor").Path
$TargetTriple = ((& rustc -vV) | Select-String '^host: ').ToString() -replace '^host: ', ''

$BinDir = Join-Path $DesktopDir "src-tauri\binaries"
$ResDir = Join-Path $DesktopDir "src-tauri\resources"

Write-Host "Building thingblock-link (release)..."
& cargo build --release --manifest-path "$LinkDir\Cargo.toml"
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

Write-Host "Staging sidecar binary for $TargetTriple..."
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
Copy-Item "$LinkDir\target\release\thingblock-link.exe" `
          (Join-Path $BinDir "thingblock-link-$TargetTriple.exe") -Force

# The pack is produced by the editor workspace @thingblock/thingblock-resource;
# build it and stage its output so the bundled pack always tracks that single
# source of truth (the link repo's copy is a gitignored dev convenience).
Write-Host "Building thingblock-resource pack..."
& npm --prefix "$EditorDir\packages\thingblock-resource" run build
if ($LASTEXITCODE -ne 0) { throw "thingblock-resource build failed" }

Write-Host "Staging thingblock-resource pack..."
$ResourcePack = Join-Path $ResDir "thingblock-resource"
if (Test-Path $ResourcePack) { Remove-Item -Recurse -Force $ResourcePack }
New-Item -ItemType Directory -Force -Path $ResDir | Out-Null
Copy-Item "$EditorDir\packages\thingblock-resource\dist\thingblock-resource" `
          $ResourcePack -Recurse -Force

# The link runs arduino-cli at a path we pass it (--arduino-cli); bundle the host
# binary under bin/ (a directory resource, uniform across platforms) so the
# tauri.conf resource map needs no per-platform override.
Write-Host "Staging host arduino-cli..."
$BinResDir = Join-Path $ResDir "bin"
New-Item -ItemType Directory -Force -Path $BinResDir | Out-Null
Copy-Item "$LinkDir\arduino-cli-binaries\arduino-cli_win_64bit\arduino-cli.exe" `
          (Join-Path $BinResDir "arduino-cli.exe") -Force

Write-Host "Sidecar staged."
