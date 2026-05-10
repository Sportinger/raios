param(
    [ValidateSet("debug", "release")]
    [string]$Profile = "release",
    [string]$Image = "$PSScriptRoot\..\release\seedos-stage0.img",
    [switch]$EmbedOpenAiApiKeyFromEnv,
    [string]$OpenAiApiKeyEnvVar = "OPENAI_API_KEY",
    [switch]$UseTempEsp
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$BaseEspDir = Join-Path $RepoRoot "release\esp"
$TempEspDir = $null
$EspDir = $BaseEspDir
$KernelProfileDir = if ($Profile -eq "release") { "release" } else { "debug" }
$Kernel = Join-Path $RepoRoot "target\x86_64-seed\$KernelProfileDir\seed-kernel"
$LimineConfig = Join-Path $RepoRoot "seed-kernel\limine\limine.conf"
$BootConfig = Join-Path $EspDir "EFI\BOOT\limine.conf"
$ImageTool = Join-Path $RepoRoot "scripts\make-fat32-image.py"

try {
    if ($UseTempEsp) {
        $TempEspDir = Join-Path $env:TEMP "seedos-stage0-esp-$PID"
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
