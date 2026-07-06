[CmdletBinding()]
param(
    [ValidateSet("release")]
    [string]$Profile = "release"
)

$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$manifest = Join-Path $repoRoot "wasm-guests\svc-demo-echo\Cargo.toml"
$targetDir = Join-Path $repoRoot "target\m4-3-debug\wasm-guest-target"
$artifactDir = Join-Path $repoRoot "seed-kernel\artifacts"
$artifactPath = Join-Path $artifactDir "svc.demo.echo.wasm"
$nightlyRustc = Join-Path $env:USERPROFILE ".rustup\toolchains\nightly-2024-10-15-x86_64-pc-windows-msvc\bin\rustc.exe"

New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
New-Item -ItemType Directory -Force -Path $artifactDir | Out-Null

$oldRustc = $env:RUSTC
$oldRustflags = $env:RUSTFLAGS
$env:RUSTC = $nightlyRustc
$env:RUSTFLAGS = "-Cpanic=abort"

$cargoArgs = @(
    "build",
    "--locked",
    "--manifest-path", $manifest,
    "--target", "wasm32-unknown-unknown",
    "--release",
    "--target-dir", $targetDir
)

try {
    & cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
} finally {
    $env:RUSTC = $oldRustc
    $env:RUSTFLAGS = $oldRustflags
}

$builtArtifact = Join-Path $targetDir "wasm32-unknown-unknown\release\svc_demo_echo.wasm"
if (-not (Test-Path -LiteralPath $builtArtifact)) {
    throw "missing wasm build output: $builtArtifact"
}

Copy-Item -LiteralPath $builtArtifact -Destination $artifactPath -Force
$hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $artifactPath).Hash.ToLowerInvariant()
Write-Output "wasm_guest_artifact=$artifactPath"
Write-Output "wasm_guest_sha256=$hash"
