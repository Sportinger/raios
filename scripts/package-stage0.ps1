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
    [switch]$RequireMarvellFirmware,
    [string]$MarvellFirmwarePath = "",
    [switch]$AllowUnverifiedOpenAiTls,
    [switch]$UseTempEsp,
    [string]$ExportEspDir = "",
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
$Kernel = [IO.Path]::Combine($CargoTargetRoot, "x86_64-seed", $KernelProfileDir, "seed-kernel")
$LimineConfig = Join-Path $RepoRoot "seed-kernel\limine\limine.conf"
$BootConfig = Join-Path $EspDir "EFI\BOOT\limine.conf"
$ImageTool = Join-Path $RepoRoot "scripts\make-fat32-image.py"
$ExportEspFullPath = $null

function Test-PathOverlap {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Left,
        [Parameter(Mandatory = $true)]
        [string]$Right
    )

    $leftFull = [IO.Path]::GetFullPath($Left).TrimEnd([char]'\', [char]'/')
    $rightFull = [IO.Path]::GetFullPath($Right).TrimEnd([char]'\', [char]'/')
    $separator = [IO.Path]::DirectorySeparatorChar
    return $leftFull.Equals($rightFull, [StringComparison]::OrdinalIgnoreCase) -or
        $leftFull.StartsWith($rightFull + $separator, [StringComparison]::OrdinalIgnoreCase) -or
        $rightFull.StartsWith($leftFull + $separator, [StringComparison]::OrdinalIgnoreCase)
}

function Get-BinaryPatternCount {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ArtifactPath,
        [Parameter(Mandatory = $true)]
        [string]$PatternPath
    )

    $count = & python -c `
        "import pathlib,sys; print(pathlib.Path(sys.argv[1]).read_bytes().count(pathlib.Path(sys.argv[2]).read_bytes()))" `
        $ArtifactPath $PatternPath
    if ($LASTEXITCODE -ne 0) {
        throw "Marvell firmware payload scan failed for $ArtifactPath."
    }
    return [int]$count
}

try {
    $exportRequested = $PSBoundParameters.ContainsKey("ExportEspDir")
    if ($exportRequested -and [string]::IsNullOrWhiteSpace($ExportEspDir)) {
        throw "-ExportEspDir must name a non-empty caller-supplied path."
    }
    if ($exportRequested) {
        if (-not $UseTempEsp) {
            throw "-ExportEspDir requires -UseTempEsp."
        }
        if (-not $RequireMarvellFirmware) {
            throw "-ExportEspDir requires -RequireMarvellFirmware."
        }
        if ([string]::IsNullOrWhiteSpace($MarvellFirmwarePath)) {
            throw "-ExportEspDir requires an explicit -MarvellFirmwarePath."
        }
        if (-not (Test-Path -LiteralPath $CorePolicyPrivateKey -PathType Leaf)) {
            throw "Refusing ESP export without the Core Policy owner key; an exported payload must contain a verified kernel-bound policy."
        }

        $ExportEspFullPath = [IO.Path]::GetFullPath($ExportEspDir)
        $tempRootFullPath = [IO.Path]::GetFullPath($env:TEMP).TrimEnd([char]'\', [char]'/')
        $tempRootPrefix = $tempRootFullPath + [IO.Path]::DirectorySeparatorChar
        if (-not $ExportEspFullPath.StartsWith($tempRootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing ESP export outside the host temp root: $ExportEspFullPath"
        }
        if (Test-PathOverlap -Left $ExportEspFullPath -Right $RepoRoot) {
            throw "Refusing ESP export path that overlaps the repository: $ExportEspFullPath"
        }
        if (Test-PathOverlap -Left $ExportEspFullPath -Right ([IO.Path]::GetFullPath($Image))) {
            throw "Refusing ESP export path that overlaps the image path: $ExportEspFullPath"
        }
        if (Test-Path -LiteralPath $ExportEspFullPath) {
            throw "Refusing pre-existing ESP export path: $ExportEspFullPath"
        }
        $exportParent = Split-Path -Parent $ExportEspFullPath
        if (-not (Test-Path -LiteralPath $exportParent -PathType Container)) {
            throw "ESP export parent directory does not exist: $exportParent"
        }
        $unsafeParent = Get-Item -LiteralPath $exportParent -Force
        while ($null -ne $unsafeParent -and
            -not $unsafeParent.FullName.TrimEnd([char]'\', [char]'/').Equals($tempRootFullPath, [StringComparison]::OrdinalIgnoreCase)) {
            if (($unsafeParent.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "Refusing ESP export through reparse-point directory: $($unsafeParent.FullName)"
            }
            $unsafeParent = $unsafeParent.Parent
        }
        if ($null -eq $unsafeParent) {
            throw "Refusing ESP export whose resolved parent is outside the host temp root: $exportParent"
        }
    }

    if ($EmbedOpenAiApiKeyFromEnv -or $EmbedNet8W7SpkiPinFromEnv -or $RequireMarvellFirmware) {
        if (-not $UseTempEsp) {
            throw "Refusing to embed per-run local-only material into the tracked release\esp staging tree. Re-run with -UseTempEsp."
        }
        $imageFullPath = [IO.Path]::GetFullPath($Image)
        $defaultImageFullPath = [IO.Path]::GetFullPath($DefaultImage)
        if ($imageFullPath -eq $defaultImageFullPath) {
            throw "Refusing to write a per-run local-only image to release\raios-stage0.img. Use a temporary ignored image path."
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
    if ($RequireMarvellFirmware) {
        $buildArgs += "-RequireMarvellFirmware"
        if (-not [string]::IsNullOrWhiteSpace($MarvellFirmwarePath)) {
            $buildArgs += @("-MarvellFirmwarePath", $MarvellFirmwarePath)
        }
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

    if ($ExportEspFullPath) {
        $firmwareFullPath = [IO.Path]::GetFullPath($MarvellFirmwarePath)
        if (-not (Test-Path -LiteralPath $firmwareFullPath -PathType Leaf)) {
            throw "Pinned Marvell firmware is missing before ESP export: $firmwareFullPath"
        }
        $firmwareItem = Get-Item -LiteralPath $firmwareFullPath
        if ($firmwareItem.Length -ne 723540) {
            throw "Pinned Marvell firmware length mismatch before ESP export."
        }
        $firmwareSha256 = (Get-FileHash -LiteralPath $firmwareFullPath -Algorithm SHA256).Hash
        if ($firmwareSha256 -ne "CF4F51F41BD7EF4D7FE65FB76B8A2A0897BC70A0742BC4AEA13D93B03FFFD03A") {
            throw "Pinned Marvell firmware SHA-256 mismatch before ESP export."
        }
        $exportKernelPath = Join-Path $EspDir "kernel\kernel.elf"
        $exportCompatKernelPath = Join-Path $EspDir "kernel\seed-kernel.elf"
        if ((Get-FileHash -LiteralPath $exportKernelPath -Algorithm SHA256).Hash -ne
            (Get-FileHash -LiteralPath $exportCompatKernelPath -Algorithm SHA256).Hash) {
            throw "Exported ESP kernel aliases do not contain identical bytes."
        }
        if ((Get-BinaryPatternCount -ArtifactPath $exportKernelPath -PatternPath $firmwareFullPath) -ne 1) {
            throw "Exported kernel does not contain the pinned pcie8897_uapsta.bin exactly once."
        }
        if (-not (Test-Path -LiteralPath $PolicyPath -PathType Leaf)) {
            throw "Exported ESP payload is missing its verified kernel-bound Core Policy."
        }
        Push-Location $RepoRoot
        try {
            & cargo run --locked --quiet -p core-policy-sign -- verify `
                $exportKernelPath $CorePolicySlot $CorePolicyGeneration $PolicyPath
            if ($LASTEXITCODE -ne 0) {
                throw "Exported ESP Core Policy is not bound to the exported kernel."
            }
        }
        finally {
            Pop-Location
        }
    }

    python $ImageTool --root $EspDir --output $Image --size 67108864
    if ($LASTEXITCODE -ne 0) {
        throw "stage0 FAT32 image build failed with exit code $LASTEXITCODE"
    }

    if ($ExportEspFullPath) {
        [IO.Directory]::Move($TempEspDir, $ExportEspFullPath)
        $TempEspDir = $null
        Write-Output "exported ESP payload: $ExportEspFullPath"
    }
}
finally {
    if ($TempEspDir) {
        Remove-Item -LiteralPath $TempEspDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
