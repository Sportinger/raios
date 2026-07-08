[CmdletBinding()]
param(
    [ValidateSet("release")]
    [string]$Profile = "release",
    [ValidateSet("svc-demo-echo", "svc-demo-bufecho", "svc-demo-certwindow")]
    [string]$Guest = "svc-demo-echo"
)

$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$guestConfig = @{
    "svc-demo-echo" = @{
        Manifest = "wasm-guests\svc-demo-echo\Cargo.toml"
        Built = "svc_demo_echo.wasm"
        Artifact = "svc.demo.echo.wasm"
    }
    "svc-demo-bufecho" = @{
        Manifest = "wasm-guests\svc-demo-bufecho\Cargo.toml"
        Built = "svc_demo_bufecho.wasm"
        Artifact = "svc.demo.bufecho.wasm"
    }
    "svc-demo-certwindow" = @{
        Manifest = "wasm-guests\svc-demo-certwindow\Cargo.toml"
        Built = "svc_demo_certwindow.wasm"
        Artifact = "svc.demo.certwindow.wasm"
    }
}[$Guest]

$manifest = Join-Path $repoRoot $guestConfig.Manifest
$targetDir = Join-Path $repoRoot "target\m4-3-debug\wasm-guest-target"
$artifactDir = Join-Path $repoRoot "seed-kernel\artifacts"
$artifactPath = Join-Path $artifactDir $guestConfig.Artifact
$nightlyRustc = Join-Path $env:USERPROFILE ".rustup\toolchains\nightly-2024-10-15-x86_64-pc-windows-msvc\bin\rustc.exe"
$nightlyRustLld = Join-Path $env:USERPROFILE ".rustup\toolchains\nightly-2024-10-15-x86_64-pc-windows-msvc\lib\rustlib\x86_64-pc-windows-msvc\bin\rust-lld.exe"
$builtArtifact = Join-Path $targetDir "wasm32-unknown-unknown\release\$($guestConfig.Built)"

New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
New-Item -ItemType Directory -Force -Path $artifactDir | Out-Null

if ($Guest -eq "svc-demo-bufecho") {
    $wasmReleaseDir = Split-Path -Parent $builtArtifact
    New-Item -ItemType Directory -Force -Path $wasmReleaseDir | Out-Null
    $objectPath = Join-Path $wasmReleaseDir "svc_demo_bufecho.o"
    $sourcePath = Join-Path $repoRoot "wasm-guests\svc-demo-bufecho\src\lib.rs"
    & $nightlyRustc --crate-name svc_demo_bufecho --crate-type cdylib --edition=2021 --target wasm32-unknown-unknown -Cpanic=abort -Copt-level=3 --emit=obj -o $objectPath $sourcePath
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    $targetLibDir = & $nightlyRustc --print target-libdir --target wasm32-unknown-unknown
    $core = (Get-ChildItem -LiteralPath $targetLibDir -Filter "libcore-*.rlib" | Select-Object -First 1).FullName
    $workspaceCore = (Get-ChildItem -LiteralPath $targetLibDir -Filter "librustc_std_workspace_core-*.rlib" | Select-Object -First 1).FullName
    $selfContained = Join-Path $targetLibDir "self-contained"
    & $nightlyRustLld -flavor wasm --export raios_service_main --export=__heap_base --export=__data_end -z stack-size=1048576 --stack-first --allow-undefined --no-demangle --no-entry $objectPath $workspaceCore $core -L $selfContained -o $builtArtifact --gc-sections --no-entry -O3 --strip-debug
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
} else {
    $oldRustc = $env:RUSTC
    $oldRustflags = $env:RUSTFLAGS
    $env:RUSTC = $nightlyRustc
    $env:RUSTFLAGS = "-Cpanic=abort -Clinker=$nightlyRustLld"

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
}

if (-not (Test-Path -LiteralPath $builtArtifact)) {
    throw "missing wasm build output: $builtArtifact"
}

Copy-Item -LiteralPath $builtArtifact -Destination $artifactPath -Force
$hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $artifactPath).Hash.ToLowerInvariant()
Write-Output "wasm_guest_artifact=$artifactPath"
Write-Output "wasm_guest_sha256=$hash"
