param(
    [ValidateSet("debug", "release")]
    [string]$Profile = "debug"
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$Toolchain = "nightly-2024-10-15"
$Target = Join-Path $RepoRoot "seed-kernel\x86_64-seed.json"
$LinkerScript = Join-Path $RepoRoot "seed-kernel\linker.ld"

if (-not ((rustup toolchain list) -match [regex]::Escape($Toolchain))) {
    rustup toolchain install $Toolchain --component rust-src --component llvm-tools-preview
}

$oldRustFlags = $env:RUSTFLAGS
$kernelRustFlags = @(
    "-C", "link-arg=-T$LinkerScript",
    "-C", "relocation-model=static",
    "-C", "code-model=kernel",
    "-C", "force-frame-pointers=yes",
    "-C", "link-arg=--gc-sections"
) -join " "

try {
    $env:RUSTFLAGS = "$kernelRustFlags $oldRustFlags".Trim()
    $cargoArgs = @(
        "+$Toolchain",
        "-Zbuild-std=core,compiler_builtins,alloc",
        "build",
        "--locked",
        "--target", $Target,
        "-p", "seed-kernel"
    )
    if ($Profile -eq "release") {
        $cargoArgs += "--release"
    }
    cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}
finally {
    $env:RUSTFLAGS = $oldRustFlags
}

$profileDir = if ($Profile -eq "release") { "release" } else { "debug" }
Write-Output "built target/x86_64-seed/$profileDir/seed-kernel"
