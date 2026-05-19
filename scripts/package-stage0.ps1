param(
    [ValidateSet("debug", "release")]
    [string]$Profile = "release",
    [string]$Image = "$PSScriptRoot\..\release\raios-stage0.img",
    [switch]$EmbedOpenAiApiKeyFromEnv,
    [string]$OpenAiApiKeyEnvVar = "OPENAI_API_KEY",
    [switch]$EmbedOpenAiCertPinFromEnv,
    [string]$OpenAiCertPinEnvVar = "OPENAI_CERT_SHA256",
    [switch]$EmbedOpenAiSpkiPinFromEnv,
    [string]$OpenAiSpkiPinEnvVar = "OPENAI_SPKI_SHA256",
    [switch]$AllowUnverifiedOpenAiTls,
    [switch]$UseTempEsp
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$DefaultImage = Join-Path $RepoRoot "release\raios-stage0.img"
$BaseEspDir = Join-Path $RepoRoot "release\esp"
$TempEspDir = $null
$EspDir = $BaseEspDir
$KernelProfileDir = if ($Profile -eq "release") { "release" } else { "debug" }
$Kernel = Join-Path $RepoRoot "target\x86_64-seed\$KernelProfileDir\seed-kernel"
$LimineConfig = Join-Path $RepoRoot "seed-kernel\limine\limine.conf"
$BootConfig = Join-Path $EspDir "EFI\BOOT\limine.conf"
$ImageTool = Join-Path $RepoRoot "scripts\make-fat32-image.py"

try {
    if ($EmbedOpenAiApiKeyFromEnv) {
        if (-not $UseTempEsp) {
            throw "Refusing to embed a provider key into the tracked release\esp staging tree. Re-run with -UseTempEsp."
        }
        $imageFullPath = [IO.Path]::GetFullPath($Image)
        $defaultImageFullPath = [IO.Path]::GetFullPath($DefaultImage)
        if ($imageFullPath -eq $defaultImageFullPath) {
            throw "Refusing to write a provider-key image to release\raios-stage0.img. Use an ignored local image path such as release\raios-stage0-local-openai.img."
        }
    }

    if ($UseTempEsp) {
        $TempEspDir = Join-Path $env:TEMP "raios-stage0-esp-$PID"
        Remove-Item -LiteralPath $TempEspDir -Recurse -Force -ErrorAction SilentlyContinue
        Copy-Item -LiteralPath $BaseEspDir -Destination $TempEspDir -Recurse -Force
        $EspDir = $TempEspDir
        $BootConfig = Join-Path $EspDir "EFI\BOOT\limine.conf"
    }

    $buildArgs = @(
        "-NoProfile",
        "-ExecutionPolicy", "Bypass",
        "-File", (Join-Path $RepoRoot "scripts\build-seed-kernel.ps1"),
        "-Profile", $Profile
    )
    if ($EmbedOpenAiApiKeyFromEnv) {
        $buildArgs += @("-EmbedOpenAiApiKeyFromEnv", "-OpenAiApiKeyEnvVar", $OpenAiApiKeyEnvVar)
    }
    if ($EmbedOpenAiCertPinFromEnv) {
        $buildArgs += @("-EmbedOpenAiCertPinFromEnv", "-OpenAiCertPinEnvVar", $OpenAiCertPinEnvVar)
    }
    if ($EmbedOpenAiSpkiPinFromEnv) {
        $buildArgs += @("-EmbedOpenAiSpkiPinFromEnv", "-OpenAiSpkiPinEnvVar", $OpenAiSpkiPinEnvVar)
    }
    if ($AllowUnverifiedOpenAiTls) {
        $buildArgs += "-AllowUnverifiedOpenAiTls"
    }
    powershell @buildArgs
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    New-Item -ItemType Directory -Force -Path (Join-Path $EspDir "EFI\BOOT") | Out-Null
    New-Item -ItemType Directory -Force -Path (Join-Path $EspDir "kernel") | Out-Null

    Copy-Item -LiteralPath $Kernel -Destination (Join-Path $EspDir "kernel\kernel.elf") -Force
    Copy-Item -LiteralPath $Kernel -Destination (Join-Path $EspDir "kernel\seed-kernel.elf") -Force
    Copy-Item -LiteralPath $LimineConfig -Destination (Join-Path $EspDir "limine.conf") -Force
    Copy-Item -LiteralPath $LimineConfig -Destination $BootConfig -Force

    if (-not (Test-Path (Join-Path $EspDir "EFI\BOOT\BOOTX64.EFI"))) {
        throw "Missing Limine bootloader at $EspDir\EFI\BOOT\BOOTX64.EFI"
    }

    python $ImageTool --root $EspDir --output $Image --size 67108864
}
finally {
    if ($TempEspDir) {
        Remove-Item -LiteralPath $TempEspDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
