# Stage the built editor plus the thingblock-resource pack into frontend/, the
# directory tauri.conf.json points frontendDist at. Runs after build:editor (it
# consumes that build's output). Windows mirror of stage-frontend.sh.
#
# The pack ships inside the frontend so the editor reads it from its own origin.
# On Windows the webview is Chromium, which gates cross-address-space requests
# into loopback: reaching the link's own /resources route over HTTP is blocked
# there. The pack is staged a second time under src-tauri/resources/ for the link
# itself (compile lib resolution, a filesystem read) — different consumers, same
# build output.
$ErrorActionPreference = "Stop"

$DesktopDir = (Resolve-Path "$PSScriptRoot\..").Path
$EditorDir  = (Resolve-Path "$DesktopDir\..\thingblock-editor").Path

$FrontendDir  = Join-Path $DesktopDir "frontend"
$EditorBuild  = Join-Path $EditorDir "packages\scratch-gui\build"
$ResourcePack = Join-Path $EditorDir "packages\thingblock-resource\dist\thingblock-resource"

if (-not (Test-Path $EditorBuild)) {
    throw "stage-frontend: no editor build at $EditorBuild; run build:editor first"
}
if (-not (Test-Path $ResourcePack)) {
    throw "stage-frontend: no resource pack at $ResourcePack; run stage:sidecar first"
}

Write-Host "Staging editor build..."
if (Test-Path $FrontendDir) { Remove-Item -Recurse -Force $FrontendDir }
Copy-Item $EditorBuild $FrontendDir -Recurse -Force

Write-Host "Staging thingblock-resource pack into the frontend..."
Copy-Item $ResourcePack (Join-Path $FrontendDir "thingblock-resource") -Recurse -Force

# The editor reads its pack base from this global (see the editor's
# link-controller.js). Injected into the staged copy rather than built into the
# editor itself, so the editor artifact stays host-agnostic and reusable against a
# cloud backend. The inline script runs before the deferred bundle.
Write-Host "Injecting the resource base..."
$Index = Join-Path $FrontendDir "index.html"
$html = Get-Content $Index -Raw
if ($html -notmatch '</head>') {
    throw "stage-frontend: no </head> in $Index; cannot inject the resource base"
}
$inject = '<script>globalThis.__THINGBLOCK_RESOURCE_BASE__="/thingblock-resource";</script></head>'
Set-Content -Path $Index -Value ($html -replace '</head>', $inject) -NoNewline

Write-Host "Frontend staged."
