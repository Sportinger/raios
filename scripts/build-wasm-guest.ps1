[CmdletBinding()]
param(
    [ValidateSet("release")]
    [string]$Profile = "release",
    [ValidateSet("svc-demo-echo", "svc-demo-bufecho", "svc-demo-certwindow", "svc-demo-httphead", "svc-demo-certspki", "svc-demo-dnsparse", "svc-net-acquire-w7", "svc-personal-shell-proof", "svc-build-assembler")]
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
    "svc-demo-httphead" = @{
        Manifest = "wasm-guests\svc-demo-httphead\Cargo.toml"
        Built = "svc_demo_httphead.wasm"
        Artifact = "svc.demo.httphead.wasm"
    }
    "svc-demo-certspki" = @{
        Manifest = "wasm-guests\svc-demo-certspki\Cargo.toml"
        Built = "svc_demo_certspki.wasm"
        Artifact = "svc.demo.certspki.wasm"
    }
    "svc-demo-dnsparse" = @{
        Manifest = "wasm-guests\svc-demo-dnsparse\Cargo.toml"
        Built = "svc_demo_dnsparse.wasm"
        Artifact = "svc.demo.dnsparse.wasm"
    }
    "svc-build-assembler" = @{
        Manifest = "wasm-guests\svc-build-assembler\Cargo.toml"
        Built = "svc_build_assembler.wasm"
        Artifact = "svc.build.assembler.wasm"
    }
    "svc-net-acquire-w7" = @{
        Manifest = "wasm-guests\svc-net-acquire-w7\Cargo.toml"
        Built = "svc_net_acquire_w7.wasm"
        Artifact = "svc.net.acquire.w7.wasm"
    }
    "svc-personal-shell-proof" = @{
        Manifest = "wasm-guests\svc-personal-shell-proof\Cargo.toml"
        Built = "svc_personal_shell_proof.wasm"
        Artifact = "svc.user.shell.wasm"
    }
}[$Guest]

$manifest = Join-Path $repoRoot $guestConfig.Manifest
$targetDir = Join-Path $repoRoot "target\lanes\$Guest\wasm-guest-target"
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

    $cargoExitCode = 0
    $cargoStderr = Join-Path $targetDir "$Guest-cargo-stderr.txt"
    $oldErrorActionPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        $cargoOutput = & cargo @cargoArgs 2>$cargoStderr
        $cargoExitCode = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $oldErrorActionPreference
        $env:RUSTC = $oldRustc
        $env:RUSTFLAGS = $oldRustflags
    }
    if ($cargoExitCode -eq 0) {
        $cargoOutput | Write-Output
    }
    if ($cargoExitCode -ne 0) {
        $wasmReleaseDir = Split-Path -Parent $builtArtifact
        $depsDir = Join-Path $wasmReleaseDir "deps"
        $crateStem = [IO.Path]::GetFileNameWithoutExtension($guestConfig.Built)
        $objectPath = (Get-ChildItem -LiteralPath $depsDir -Filter "$crateStem.*.rcgu.o" -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
        if (-not $objectPath) {
            $cargoOutput | Write-Output
            if (Test-Path -LiteralPath $cargoStderr) {
                Get-Content -LiteralPath $cargoStderr | Write-Output
            }
            exit $cargoExitCode
        }

        $depRlibs = @(Get-ChildItem -LiteralPath $depsDir -Filter "*.rlib" | Select-Object -ExpandProperty FullName)
        $targetLibDir = & $nightlyRustc --print target-libdir --target wasm32-unknown-unknown
        $core = (Get-ChildItem -LiteralPath $targetLibDir -Filter "libcore-*.rlib" | Select-Object -First 1).FullName
        $workspaceCore = (Get-ChildItem -LiteralPath $targetLibDir -Filter "librustc_std_workspace_core-*.rlib" | Select-Object -First 1).FullName
        $compilerBuiltins = (Get-ChildItem -LiteralPath $targetLibDir -Filter "libcompiler_builtins-*.rlib" | Select-Object -First 1).FullName
        $selfContained = Join-Path $targetLibDir "self-contained"
        $lldArgs = @(
            "-flavor", "wasm",
            "--export", "raios_service_main",
            "--export=__heap_base",
            "--export=__data_end",
            "-z", "stack-size=1048576",
            "--stack-first",
            "--allow-undefined",
            "--no-demangle",
            "--no-entry",
            $objectPath
        ) + $depRlibs + @(
            $workspaceCore,
            $core,
            $compilerBuiltins,
            "-L", $selfContained,
            "-o", $builtArtifact,
            "--gc-sections",
            "--no-entry",
            "-O3",
            "--strip-debug"
        )
        $lldStderr = Join-Path $targetDir "$Guest-rust-lld-stderr.txt"
        $oldErrorActionPreference = $ErrorActionPreference
        try {
            $ErrorActionPreference = "Continue"
            $lldOutput = & $nightlyRustLld @lldArgs 2>$lldStderr
            $lldExitCode = $LASTEXITCODE
        } finally {
            $ErrorActionPreference = $oldErrorActionPreference
        }
        if ($lldExitCode -ne 0) {
            $cargoOutput | Write-Output
            if (Test-Path -LiteralPath $cargoStderr) {
                Get-Content -LiteralPath $cargoStderr | Write-Output
            }
            $lldOutput | Write-Output
            if (Test-Path -LiteralPath $lldStderr) {
                Get-Content -LiteralPath $lldStderr | Write-Output
            }
            exit $lldExitCode
        }
    }
}

if (-not (Test-Path -LiteralPath $builtArtifact)) {
    throw "missing wasm build output: $builtArtifact"
}

Copy-Item -LiteralPath $builtArtifact -Destination $artifactPath -Force
$hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $artifactPath).Hash.ToLowerInvariant()
Write-Output "wasm_guest_artifact=$artifactPath"
Write-Output "wasm_guest_sha256=$hash"
