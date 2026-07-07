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

# The resource pack type-checks against @scratch/scratch-blocks (type-only imports,
# erased from the built output), so its tsc step needs that package's emitted
# declarations. A fresh `npm ci` installs but does not build workspace packages, so
# build scratch-blocks first to produce its dist/types — matching the editor
# monorepo's own build order.
Write-Host "Building @scratch/scratch-blocks (resource pack's type declarations)..."
& npm --prefix "$EditorDir\packages\scratch-blocks" run build
if ($LASTEXITCODE -ne 0) { throw "scratch-blocks build failed" }

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

# The daemon needs arduino-cli.yaml and a data/ bundle (--config-dir contract in
# the link's daemon.rs). Stage the yaml plus a seed with the arduino:avr core
# pre-installed so compiles work offline; the app copies this seed into a
# writable per-user dir on first run. Mirrors thingblock-link/scripts/
# bundle-data.sh. The CLI runs with cwd set to the seed dir so the yaml's
# relative directories.* resolve into it — the same mechanism daemon.rs uses.
Write-Host "Staging arduino config seed..."
$SeedDir = Join-Path $ResDir "arduino"
New-Item -ItemType Directory -Force -Path $SeedDir | Out-Null
Copy-Item "$LinkDir\arduino-cli.yaml" (Join-Path $SeedDir "arduino-cli.yaml") -Force
if (-not (Test-Path (Join-Path $SeedDir "data\packages\arduino"))) {
    $Cli = Join-Path $BinResDir "arduino-cli.exe"
    Push-Location $SeedDir
    try {
        & $Cli --config-file arduino-cli.yaml core update-index
        if ($LASTEXITCODE -ne 0) { throw "arduino-cli core update-index failed" }
        & $Cli --config-file arduino-cli.yaml core install arduino:avr
        if ($LASTEXITCODE -ne 0) { throw "arduino-cli core install arduino:avr failed" }
    } finally {
        Pop-Location
    }
    # Prune the download cache and temp files; inventory.yaml carries a
    # per-machine installation id/secret — never ship one.
    Remove-Item -Recurse -Force (Join-Path $SeedDir "data\staging") -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force (Join-Path $SeedDir "data\tmp") -ErrorAction SilentlyContinue
    Remove-Item -Force (Join-Path $SeedDir "data\inventory.yaml") -ErrorAction SilentlyContinue
} else {
    Write-Host "arduino:avr already seeded; skipping install."
}

Write-Host "Sidecar staged."
