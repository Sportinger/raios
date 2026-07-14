param(
    [ValidateSet("debug", "release")]
    [string]$Profile = "debug",
    [switch]$EmbedOpenAiApiKeyFromEnv,
    [string]$OpenAiApiKeyEnvVar = "OPENAI_API_KEY",
    [switch]$EmbedOpenAiCertPinFromEnv,
    [string]$OpenAiCertPinEnvVar = "OPENAI_CERT_SHA256",
    [switch]$EmbedOpenAiSpkiPinFromEnv,
    [string]$OpenAiSpkiPinEnvVar = "OPENAI_SPKI_SHA256",
    [switch]$EmbedOpenAiSpkiRotationPinFromEnv,
    [string]$OpenAiSpkiRotationPinEnvVar = "OPENAI_SPKI_SHA256_NEXT",
    [switch]$EmbedNet8W7SpkiPinFromEnv,
    [string]$Net8W7SpkiPinEnvVar = "NET_8_W7_SPKI_SHA256",
    [switch]$AllowUnverifiedOpenAiTls
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
$oldDefaultOpenAiApiKey = $env:RAIOS_DEFAULT_OPENAI_API_KEY
$oldOpenAiCertSha256 = $env:RAIOS_OPENAI_CERT_SHA256
$oldOpenAiSpkiSha256 = $env:RAIOS_OPENAI_SPKI_SHA256
$oldOpenAiSpkiSha256Next = $env:RAIOS_OPENAI_SPKI_SHA256_NEXT
$oldNet8W7SpkiSha256 = $env:RAIOS_NET_8_W7_SPKI_SHA256
$oldAllowUnverifiedOpenAiTls = $env:RAIOS_ALLOW_UNVERIFIED_OPENAI_TLS
$kernelRustFlags = @(
    "-C", "link-arg=-T$LinkerScript",
    "-C", "relocation-model=static",
    "-C", "code-model=kernel",
    "-C", "force-frame-pointers=yes",
    "-C", "link-arg=--gc-sections"
) -join " "

try {
    if ($EmbedOpenAiApiKeyFromEnv) {
        $apiKey = [Environment]::GetEnvironmentVariable($OpenAiApiKeyEnvVar, "Process")
        if ([string]::IsNullOrWhiteSpace($apiKey)) {
            throw "Environment variable '$OpenAiApiKeyEnvVar' is not set."
        }
        $env:RAIOS_DEFAULT_OPENAI_API_KEY = $apiKey
    }
    else {
        Remove-Item Env:\RAIOS_DEFAULT_OPENAI_API_KEY -ErrorAction SilentlyContinue
    }

    if ($EmbedOpenAiCertPinFromEnv) {
        $certPin = [Environment]::GetEnvironmentVariable($OpenAiCertPinEnvVar, "Process")
        if ([string]::IsNullOrWhiteSpace($certPin)) {
            throw "Environment variable '$OpenAiCertPinEnvVar' is not set."
        }
        $env:RAIOS_OPENAI_CERT_SHA256 = $certPin
    }
    else {
        Remove-Item Env:\RAIOS_OPENAI_CERT_SHA256 -ErrorAction SilentlyContinue
    }

    if ($EmbedOpenAiSpkiPinFromEnv) {
        $spkiPin = [Environment]::GetEnvironmentVariable($OpenAiSpkiPinEnvVar, "Process")
        if ([string]::IsNullOrWhiteSpace($spkiPin)) {
            throw "Environment variable '$OpenAiSpkiPinEnvVar' is not set."
        }
        $env:RAIOS_OPENAI_SPKI_SHA256 = $spkiPin
    }
    else {
        Remove-Item Env:\RAIOS_OPENAI_SPKI_SHA256 -ErrorAction SilentlyContinue
    }

    if ($EmbedOpenAiSpkiRotationPinFromEnv) {
        $spkiRotationPin = [Environment]::GetEnvironmentVariable($OpenAiSpkiRotationPinEnvVar, "Process")
        if ([string]::IsNullOrWhiteSpace($spkiRotationPin)) {
            throw "Environment variable '$OpenAiSpkiRotationPinEnvVar' is not set."
        }
        $env:RAIOS_OPENAI_SPKI_SHA256_NEXT = $spkiRotationPin
    }
    else {
        Remove-Item Env:\RAIOS_OPENAI_SPKI_SHA256_NEXT -ErrorAction SilentlyContinue
    }

    if ($EmbedNet8W7SpkiPinFromEnv) {
        $net8Pin = [Environment]::GetEnvironmentVariable($Net8W7SpkiPinEnvVar, "Process")
        if ($net8Pin -notmatch '^[0-9a-fA-F]{64}$') {
            throw "Environment variable '$Net8W7SpkiPinEnvVar' must contain exactly 64 hexadecimal characters."
        }
        $env:RAIOS_NET_8_W7_SPKI_SHA256 = $net8Pin.ToLowerInvariant()
    }
    else {
        Remove-Item Env:\RAIOS_NET_8_W7_SPKI_SHA256 -ErrorAction SilentlyContinue
    }

    if ($AllowUnverifiedOpenAiTls) {
        $env:RAIOS_ALLOW_UNVERIFIED_OPENAI_TLS = "1"
    }
    else {
        Remove-Item Env:\RAIOS_ALLOW_UNVERIFIED_OPENAI_TLS -ErrorAction SilentlyContinue
    }

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
    if ($null -eq $oldDefaultOpenAiApiKey) {
        Remove-Item Env:\RAIOS_DEFAULT_OPENAI_API_KEY -ErrorAction SilentlyContinue
    }
    else {
        $env:RAIOS_DEFAULT_OPENAI_API_KEY = $oldDefaultOpenAiApiKey
    }
    if ($null -eq $oldOpenAiCertSha256) {
        Remove-Item Env:\RAIOS_OPENAI_CERT_SHA256 -ErrorAction SilentlyContinue
    }
    else {
        $env:RAIOS_OPENAI_CERT_SHA256 = $oldOpenAiCertSha256
    }
    if ($null -eq $oldOpenAiSpkiSha256) {
        Remove-Item Env:\RAIOS_OPENAI_SPKI_SHA256 -ErrorAction SilentlyContinue
    }
    else {
        $env:RAIOS_OPENAI_SPKI_SHA256 = $oldOpenAiSpkiSha256
    }
    if ($null -eq $oldOpenAiSpkiSha256Next) {
        Remove-Item Env:\RAIOS_OPENAI_SPKI_SHA256_NEXT -ErrorAction SilentlyContinue
    }
    else {
        $env:RAIOS_OPENAI_SPKI_SHA256_NEXT = $oldOpenAiSpkiSha256Next
    }
    if ($null -eq $oldNet8W7SpkiSha256) {
        Remove-Item Env:\RAIOS_NET_8_W7_SPKI_SHA256 -ErrorAction SilentlyContinue
    }
    else {
        $env:RAIOS_NET_8_W7_SPKI_SHA256 = $oldNet8W7SpkiSha256
    }
    if ($null -eq $oldAllowUnverifiedOpenAiTls) {
        Remove-Item Env:\RAIOS_ALLOW_UNVERIFIED_OPENAI_TLS -ErrorAction SilentlyContinue
    }
    else {
        $env:RAIOS_ALLOW_UNVERIFIED_OPENAI_TLS = $oldAllowUnverifiedOpenAiTls
    }
}

$profileDir = if ($Profile -eq "release") { "release" } else { "debug" }
$targetRoot = if ([string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR)) {
    Join-Path $RepoRoot "target"
}
else {
    [IO.Path]::GetFullPath($env:CARGO_TARGET_DIR)
}
Write-Output "built $(Join-Path $targetRoot "x86_64-seed\$profileDir\seed-kernel")"
