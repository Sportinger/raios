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
    [switch]$EmbedOpenAiSpkiRotationPinFromEnv,
    [string]$OpenAiSpkiRotationPinEnvVar = "OPENAI_SPKI_SHA256_NEXT",
    [switch]$EmbedNet8W7SpkiPinFromEnv,
    [string]$Net8W7SpkiPinEnvVar = "NET_8_W7_SPKI_SHA256",
    [switch]$AllowUnverifiedOpenAiTls,
    [switch]$UseTempEsp,
    [ValidateSet("A", "B")]
    [string]$CorePolicySlot = "A",
    [UInt64]$CorePolicyGeneration = 1,
    [string]$CorePolicyPrivateKey = (Join-Path $env:LOCALAPPDATA "raiOS\keys\core-policy-owner.p256.secret")
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$DefaultImage = Join-Path $RepoRoot "release\raios-stage0.img"
$BaseEspDir = Join-Path $RepoRoot "release\esp"
$TempEspDir = $null
$EspDir = $BaseEspDir
$KernelProfileDir = if ($Profile -eq "release") { "release" } else { "debug" }
$CargoTargetRoot = if ([string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR)) {
    Join-Path $RepoRoot "target"
}
else {
    [IO.Path]::GetFullPath($env:CARGO_TARGET_DIR)
}
$Kernel = Join-Path $CargoTargetRoot "x86_64-seed\$KernelProfileDir\seed-kernel"
$LimineConfig = Join-Path $RepoRoot "seed-kernel\limine\limine.conf"
$BootConfig = Join-Path $EspDir "EFI\BOOT\limine.conf"
$ImageTool = Join-Path $RepoRoot "scripts\make-fat32-image.py"

try {
    if ($EmbedOpenAiApiKeyFromEnv -or $EmbedNet8W7SpkiPinFromEnv) {
        if (-not $UseTempEsp) {
            throw "Refusing to embed per-run authority into the tracked release\esp staging tree. Re-run with -UseTempEsp."
        }
        $imageFullPath = [IO.Path]::GetFullPath($Image)
        $defaultImageFullPath = [IO.Path]::GetFullPath($DefaultImage)
        if ($imageFullPath -eq $defaultImageFullPath) {
            throw "Refusing to write a per-run authority image to release\raios-stage0.img. Use a temporary ignored image path."
        }
    }

    if ($UseTempEsp) {
        $TempEspDir = Join-Path $env:TEMP ("raios-stage0-esp-{0}-{1}" -f $PID, [Guid]::NewGuid().ToString("N"))
        $tempRoot = [IO.Path]::GetFullPath($env:TEMP).TrimEnd([char]'\', [char]'/') + [IO.Path]::DirectorySeparatorChar
        $tempEspFullPath = [IO.Path]::GetFullPath($TempEspDir)
        if (-not $tempEspFullPath.StartsWith($tempRoot, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing temporary ESP outside the host temp root: $tempEspFullPath"
        }
        if (Test-Path -LiteralPath $TempEspDir) {
            throw "Refusing pre-existing temporary ESP path: $TempEspDir"
        }
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
    if ($EmbedOpenAiSpkiRotationPinFromEnv) {
        $buildArgs += @("-EmbedOpenAiSpkiRotationPinFromEnv", "-OpenAiSpkiRotationPinEnvVar", $OpenAiSpkiRotationPinEnvVar)
    }
    if ($EmbedNet8W7SpkiPinFromEnv) {
        $buildArgs += @("-EmbedNet8W7SpkiPinFromEnv", "-Net8W7SpkiPinEnvVar", $Net8W7SpkiPinEnvVar)
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

    $PolicyDir = Join-Path $EspDir "raios"
    $PolicyPath = Join-Path $PolicyDir "core-policy.bin"
    Remove-Item -LiteralPath $PolicyPath -Force -ErrorAction SilentlyContinue
    if (Test-Path -LiteralPath $CorePolicyPrivateKey -PathType Leaf) {
        if ($CorePolicyGeneration -eq 0) {
            throw "CorePolicyGeneration must be greater than zero."
        }
        New-Item -ItemType Directory -Force -Path $PolicyDir | Out-Null
        # Host tools must never inherit a caller cwd inside seed-kernel/: the
        # kernel's rust-toolchain.toml would hand bare cargo the pinned 2024
        # nightly, which cannot parse edition2024 registry manifests
        # (base64ct >= 1.8). Pin the cwd so rustup resolves the default
        # toolchain deterministically.
        Push-Location $RepoRoot
        try {
            & cargo run --locked --quiet -p core-policy-sign -- sign `
                $CorePolicyPrivateKey $Kernel $CorePolicySlot $CorePolicyGeneration $PolicyPath
            if ($LASTEXITCODE -ne 0) {
                throw "Core Policy signing failed with exit code $LASTEXITCODE"
            }
            & cargo run --locked --quiet -p core-policy-sign -- verify `
                $Kernel $CorePolicySlot $CorePolicyGeneration $PolicyPath
            if ($LASTEXITCODE -ne 0) {
                throw "Core Policy verification failed with exit code $LASTEXITCODE"
            }
        }
        finally {
            Pop-Location
        }
    }
    else {
        Write-Warning "Core Policy owner key is absent; packaged image remains fail-closed for Vault authority."
        if ((Test-Path -LiteralPath $PolicyDir) -and
            -not (Get-ChildItem -LiteralPath $PolicyDir -Force | Select-Object -First 1)) {
            Remove-Item -LiteralPath $PolicyDir -Force
        }
    }

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
