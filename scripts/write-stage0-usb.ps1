param(
    [Parameter(Mandatory = $true)]
    [int]$DiskNumber,
    [string]$ConfirmErase = "",
    [ValidateSet("debug", "release")]
    [string]$Profile = "release",
    [ValidateSet("GPT", "MBR")]
    [string]$PartitionStyle = "GPT",
    [int]$BootPartitionSizeMB = 512,
    [switch]$EmbedOpenAiApiKeyFromEnv,
    [string]$OpenAiApiKeyEnvVar = "OPENAI_API_KEY",
    [switch]$EmbedOpenAiCertPinFromEnv,
    [string]$OpenAiCertPinEnvVar = "OPENAI_CERT_SHA256",
    [switch]$EmbedOpenAiSpkiPinFromEnv,
    [string]$OpenAiSpkiPinEnvVar = "OPENAI_SPKI_SHA256",
    [switch]$EmbedOpenAiSpkiRotationPinFromEnv,
    [string]$OpenAiSpkiRotationPinEnvVar = "OPENAI_SPKI_SHA256_NEXT",
    [switch]$AllowUnverifiedOpenAiTls,
    [switch]$UsePersistLayout,
    [switch]$SkipBuild,
    [string]$SourceEspDir = "",
    [switch]$RequireMarvellFirmware,
    [string]$MarvellFirmwarePath = "",
    [ValidateSet("A", "B")]
    [string]$CorePolicySlot = "A",
    [UInt64]$CorePolicyGeneration = 1,
    [switch]$ValidateSourceOnly
)

$ErrorActionPreference = "Stop"

$RequiresFreshKernelBuild = $EmbedOpenAiApiKeyFromEnv -or $EmbedOpenAiCertPinFromEnv -or $EmbedOpenAiSpkiPinFromEnv -or $EmbedOpenAiSpkiRotationPinFromEnv -or $AllowUnverifiedOpenAiTls
if ($RequiresFreshKernelBuild -and $SkipBuild) {
    throw "Refusing -SkipBuild with provider trust/key build flags because they must be compiled into a fresh local kernel before writing the USB stick."
}

function Test-Admin {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = [Security.Principal.WindowsPrincipal]::new($identity)
    $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

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

function Assert-ExportedEspPayload {
    param(
        [Parameter(Mandatory = $true)]
        [string]$PayloadDir,
        [Parameter(Mandatory = $true)]
        [string]$FirmwarePath,
        [Parameter(Mandatory = $true)]
        [ValidateSet("A", "B")]
        [string]$PolicySlot,
        [Parameter(Mandatory = $true)]
        [UInt64]$PolicyGeneration,
        [Parameter(Mandatory = $true)]
        [string]$RepoRoot
    )

    if ($PolicyGeneration -eq 0) {
        throw "CorePolicyGeneration must be greater than zero."
    }
    if (-not (Test-Path -LiteralPath $PayloadDir -PathType Container)) {
        throw "Exported ESP source directory is missing or invalid: $PayloadDir"
    }

    $payloadFullPath = (Resolve-Path -LiteralPath $PayloadDir).Path.TrimEnd([char]'\', [char]'/')
    $tempRootFullPath = (Resolve-Path -LiteralPath $env:TEMP).Path.TrimEnd([char]'\', [char]'/')
    $tempRootPrefix = $tempRootFullPath + [IO.Path]::DirectorySeparatorChar
    if (-not $payloadFullPath.StartsWith($tempRootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing exported ESP source outside the host temp root: $payloadFullPath"
    }
    if (Test-PathOverlap -Left $payloadFullPath -Right $RepoRoot) {
        throw "Refusing exported ESP source that overlaps the repository: $payloadFullPath"
    }

    $payloadItem = Get-Item -LiteralPath $payloadFullPath -Force
    while ($null -ne $payloadItem -and
        -not $payloadItem.FullName.TrimEnd([char]'\', [char]'/').Equals($tempRootFullPath, [StringComparison]::OrdinalIgnoreCase)) {
        if (($payloadItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "Refusing exported ESP source through reparse-point path: $($payloadItem.FullName)"
        }
        $payloadItem = $payloadItem.Parent
    }
    if ($null -eq $payloadItem) {
        throw "Refusing exported ESP source whose resolved parent is outside the host temp root: $payloadFullPath"
    }
    $nestedReparsePoint = Get-ChildItem -LiteralPath $payloadFullPath -Recurse -Force |
        Where-Object { ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 } |
        Select-Object -First 1
    if ($null -ne $nestedReparsePoint) {
        throw "Refusing exported ESP source containing a reparse point: $($nestedReparsePoint.FullName)"
    }

    $requiredFiles = @(
        "EFI\BOOT\BOOTX64.EFI",
        "EFI\BOOT\limine.conf",
        "kernel\kernel.elf",
        "kernel\seed-kernel.elf",
        "limine.conf",
        "raios\core-policy.bin"
    )
    foreach ($relativePath in $requiredFiles) {
        $requiredPath = Join-Path $payloadFullPath $relativePath
        if (-not (Test-Path -LiteralPath $requiredPath -PathType Leaf)) {
            throw "Exported ESP payload is missing required file '$relativePath'."
        }
    }

    if (-not (Test-Path -LiteralPath $FirmwarePath -PathType Leaf)) {
        throw "Pinned Marvell firmware is missing for writer validation: $FirmwarePath"
    }
    $firmwareFullPath = (Resolve-Path -LiteralPath $FirmwarePath).Path
    $firmwareItem = Get-Item -LiteralPath $firmwareFullPath
    if ($firmwareItem.Length -ne 723540) {
        throw "Pinned Marvell firmware length mismatch for writer validation."
    }
    $firmwareSha256 = (Get-FileHash -LiteralPath $firmwareFullPath -Algorithm SHA256).Hash
    if ($firmwareSha256 -ne "CF4F51F41BD7EF4D7FE65FB76B8A2A0897BC70A0742BC4AEA13D93B03FFFD03A") {
        throw "Pinned Marvell firmware SHA-256 mismatch for writer validation."
    }

    $kernelPath = Join-Path $payloadFullPath "kernel\kernel.elf"
    $compatKernelPath = Join-Path $payloadFullPath "kernel\seed-kernel.elf"
    $kernelSha256 = (Get-FileHash -LiteralPath $kernelPath -Algorithm SHA256).Hash
    $compatKernelSha256 = (Get-FileHash -LiteralPath $compatKernelPath -Algorithm SHA256).Hash
    if ($kernelSha256 -ne $compatKernelSha256) {
        throw "Exported ESP kernel aliases do not contain identical bytes."
    }
    if ((Get-BinaryPatternCount -ArtifactPath $kernelPath -PatternPath $firmwareFullPath) -ne 1) {
        throw "Exported ESP kernel does not contain the pinned pcie8897_uapsta.bin exactly once."
    }

    $policyPath = Join-Path $payloadFullPath "raios\core-policy.bin"
    Push-Location -LiteralPath $RepoRoot
    try {
        $oldErrorActionPreference = $ErrorActionPreference
        try {
            $ErrorActionPreference = "Continue"
            $verifyOutput = & cargo run --locked --quiet -p core-policy-sign -- verify `
                $kernelPath $PolicySlot $PolicyGeneration $policyPath 2>&1
            $verifyExitCode = $LASTEXITCODE
        }
        finally {
            $ErrorActionPreference = $oldErrorActionPreference
        }
        if ($verifyExitCode -ne 0) {
            $verifyDetail = ($verifyOutput | Out-String).Trim()
            throw "Exported ESP Core Policy is missing a valid binding to the packaged kernel. verifier=$verifyDetail"
        }
    }
    finally {
        Pop-Location
    }

    return $payloadFullPath
}

function Get-FreeDriveLetter {
    foreach ($letter in @("S", "T", "U", "V", "W", "X", "Y", "Z", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R")) {
        if (-not (Get-Volume -DriveLetter $letter -ErrorAction SilentlyContinue)) {
            return $letter
        }
    }
    throw "Could not find a free drive letter for the USB boot partition."
}

function New-GptFat32BootPartition {
    param(
        [int]$DiskNumber,
        [int]$SizeMB,
        [string]$DriveLetter
    )

    $diskpartScript = Join-Path $env:TEMP "raios-usb-gpt-$PID.diskpart"
    @"
select disk $DiskNumber
clean
convert gpt noerr
create partition primary size=$SizeMB
select partition 1
format quick fs=fat32 label=RAIOS
assign letter=$DriveLetter
set id=c12a7328-f81f-11d2-ba4b-00a0c93ec93b override
exit
"@ | Set-Content -LiteralPath $diskpartScript -Encoding ASCII

    try {
        & diskpart /s $diskpartScript
        if ($LASTEXITCODE -ne 0) {
            throw "diskpart failed while preparing GPT/FAT32 USB partition."
        }
    }
    finally {
        Remove-Item -LiteralPath $diskpartScript -Force -ErrorAction SilentlyContinue
    }
}

function Get-DiskIdentitySnapshot {
    param(
        [Parameter(Mandatory = $true)]
        $Disk
    )

    return [pscustomobject]@{
        Number = [int]$Disk.Number
        UniqueId = ([string]$Disk.UniqueId).Trim()
        SerialNumber = ([string]$Disk.SerialNumber).Trim()
        FriendlyName = ([string]$Disk.FriendlyName).Trim()
        Size = [UInt64]$Disk.Size
        BusType = $Disk.BusType.ToString()
    }
}

function Assert-DiskIdentity {
    param(
        [Parameter(Mandatory = $true)]
        [int]$DiskNumber,
        [Parameter(Mandatory = $true)]
        $ExpectedIdentity
    )

    $current = Get-Disk -Number $DiskNumber -ErrorAction Stop
    $actual = Get-DiskIdentitySnapshot -Disk $current
    if ($actual.Number -ne $ExpectedIdentity.Number -or
        $actual.UniqueId -ne $ExpectedIdentity.UniqueId -or
        $actual.SerialNumber -ne $ExpectedIdentity.SerialNumber -or
        $actual.FriendlyName -ne $ExpectedIdentity.FriendlyName -or
        $actual.Size -ne $ExpectedIdentity.Size -or
        $actual.BusType -ne $ExpectedIdentity.BusType -or
        $current.IsBoot -or $current.IsSystem) {
        throw "Disk $DiskNumber identity changed after erase confirmation; refusing all disk mutation."
    }
    return $current
}

function Write-RawImageToDisk {
    param(
        [int]$DiskNumber,
        [string]$ImagePath,
        [Parameter(Mandatory = $true)]
        $ExpectedDiskIdentity
    )

    $resolvedImage = (Resolve-Path -LiteralPath $ImagePath).Path
    $imageInfo = Get-Item -LiteralPath $resolvedImage
    $disk = Assert-DiskIdentity -DiskNumber $DiskNumber -ExpectedIdentity $ExpectedDiskIdentity
    if ($disk.Size -lt $imageInfo.Length) {
        throw "USB disk is too small for persist image: disk=$($disk.Size) image=$($imageInfo.Length)."
    }

    $diskOfflined = $false
    try {
        Set-Disk -Number $DiskNumber -IsOffline $true -ErrorAction Stop
        $diskOfflined = $true
    }
    catch {
        Write-Host "Disk $DiskNumber could not be offlined; clearing partition table before raw write."
        Assert-DiskIdentity -DiskNumber $DiskNumber -ExpectedIdentity $ExpectedDiskIdentity | Out-Null
        Clear-Disk -Number $DiskNumber -RemoveData -RemoveOEM -Confirm:$false -ErrorAction Stop
        Update-Disk -Number $DiskNumber -ErrorAction SilentlyContinue
    }
    try {
        Assert-DiskIdentity -DiskNumber $DiskNumber -ExpectedIdentity $ExpectedDiskIdentity | Out-Null
        $targetPath = "\\.\PhysicalDrive$DiskNumber"
        $input = [System.IO.File]::Open($resolvedImage, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::Read)
        $output = [System.IO.File]::Open($targetPath, [System.IO.FileMode]::Open, [System.IO.FileAccess]::ReadWrite, [System.IO.FileShare]::ReadWrite)
        try {
            $buffer = New-Object byte[] (4MB)
            while (($read = $input.Read($buffer, 0, $buffer.Length)) -gt 0) {
                $output.Write($buffer, 0, $read)
            }
            $output.Flush($true)
        }
        finally {
            $output.Dispose()
            $input.Dispose()
        }
    }
    finally {
        if ($diskOfflined) {
            Set-Disk -Number $DiskNumber -IsOffline $false
        }
        Update-Disk -Number $DiskNumber -ErrorAction SilentlyContinue
    }
}

$RepoRoot = Split-Path -Parent $PSScriptRoot
$BaseEspDir = Join-Path $RepoRoot "release\esp"
$TempEspDir = $null
$TempPersistImage = $null
$SelectedEspDir = $BaseEspDir
$KernelProfileDir = if ($Profile -eq "release") { "release" } else { "debug" }
$Kernel = Join-Path $RepoRoot "target\x86_64-seed\$KernelProfileDir\seed-kernel"
$LimineConfig = Join-Path $RepoRoot "seed-kernel\limine\limine.conf"
$PushedRepoRoot = $false

try {
    $hasExplicitSource = $PSBoundParameters.ContainsKey("SourceEspDir")
    if ($hasExplicitSource -and [string]::IsNullOrWhiteSpace($SourceEspDir)) {
        throw "-SourceEspDir must name a non-empty exported ESP path."
    }
    if ($hasExplicitSource -and -not $SkipBuild) {
        throw "-SourceEspDir is accepted only together with -SkipBuild."
    }
    if ($hasExplicitSource -and -not $UsePersistLayout) {
        throw "-SourceEspDir is accepted only for the persistent A/B layout; add -UsePersistLayout."
    }
    if ($hasExplicitSource -and -not $RequireMarvellFirmware) {
        throw "-SourceEspDir requires -RequireMarvellFirmware."
    }
    if ($hasExplicitSource -and [string]::IsNullOrWhiteSpace($MarvellFirmwarePath)) {
        throw "-SourceEspDir requires an explicit -MarvellFirmwarePath."
    }
    if ((-not $hasExplicitSource) -and ($RequireMarvellFirmware -or
        -not [string]::IsNullOrWhiteSpace($MarvellFirmwarePath))) {
        throw "Writer-side Marvell firmware validation requires -SourceEspDir and -SkipBuild."
    }
    if ($ValidateSourceOnly -and -not $hasExplicitSource) {
        throw "-ValidateSourceOnly requires -SourceEspDir and -SkipBuild."
    }
    if ($ValidateSourceOnly -and -not [string]::IsNullOrEmpty($ConfirmErase)) {
        throw "-ValidateSourceOnly refuses -ConfirmErase because it never authorizes a disk write."
    }
    if ($UsePersistLayout -and $PartitionStyle -ne "GPT") {
        throw "-UsePersistLayout requires -PartitionStyle GPT."
    }
    if ($BootPartitionSizeMB -lt 64 -or $BootPartitionSizeMB -gt 32768) {
        throw "BootPartitionSizeMB must be between 64 and 32768."
    }

    if ($hasExplicitSource) {
        $validatedSource = Assert-ExportedEspPayload `
            -PayloadDir $SourceEspDir `
            -FirmwarePath $MarvellFirmwarePath `
            -PolicySlot $CorePolicySlot `
            -PolicyGeneration $CorePolicyGeneration `
            -RepoRoot $RepoRoot

        $TempEspDir = Join-Path $env:TEMP ("raios-usb-source-snapshot-{0}-{1}" -f $PID, [Guid]::NewGuid().ToString("N"))
        if (Test-Path -LiteralPath $TempEspDir) {
            throw "Refusing pre-existing writer source snapshot path: $TempEspDir"
        }
        Copy-Item -LiteralPath $validatedSource -Destination $TempEspDir -Recurse
        $SelectedEspDir = Assert-ExportedEspPayload `
            -PayloadDir $TempEspDir `
            -FirmwarePath $MarvellFirmwarePath `
            -PolicySlot $CorePolicySlot `
            -PolicyGeneration $CorePolicyGeneration `
            -RepoRoot $RepoRoot

        if ($ValidateSourceOnly) {
            Write-Output "exported ESP source validation: green"
            Write-Output "source_snapshot=$SelectedEspDir"
            return
        }
    }

    if (-not (Test-Admin)) {
        throw "Run this script from an elevated PowerShell window."
    }

    $expectedConfirmation = "ERASE DISK $DiskNumber"
    if ($ConfirmErase -ne $expectedConfirmation) {
        throw "Refusing to erase disk. Re-run with -ConfirmErase '$expectedConfirmation'."
    }

    $disk = Get-Disk -Number $DiskNumber -ErrorAction Stop
    if ($disk.IsBoot -or $disk.IsSystem) {
        throw "Refusing to erase disk $DiskNumber because Windows marks it as boot/system."
    }
    if ($disk.BusType -ne "USB") {
        throw "Refusing to erase disk $DiskNumber because BusType is '$($disk.BusType)', not USB."
    }
    $diskIdentity = Get-DiskIdentitySnapshot -Disk $disk
    Push-Location -LiteralPath $RepoRoot
    $PushedRepoRoot = $true

    if ($EmbedOpenAiApiKeyFromEnv) {
        $apiKey = [Environment]::GetEnvironmentVariable($OpenAiApiKeyEnvVar, "Process")
        if ([string]::IsNullOrWhiteSpace($apiKey)) {
            throw "Environment variable '$OpenAiApiKeyEnvVar' is not set."
        }
    }
    if ($EmbedOpenAiCertPinFromEnv) {
        $certPin = [Environment]::GetEnvironmentVariable($OpenAiCertPinEnvVar, "Process")
        if ([string]::IsNullOrWhiteSpace($certPin)) {
            throw "Environment variable '$OpenAiCertPinEnvVar' is not set."
        }
    }
    if ($EmbedOpenAiSpkiPinFromEnv) {
        $spkiPin = [Environment]::GetEnvironmentVariable($OpenAiSpkiPinEnvVar, "Process")
        if ([string]::IsNullOrWhiteSpace($spkiPin)) {
            throw "Environment variable '$OpenAiSpkiPinEnvVar' is not set."
        }
    }
    if ($EmbedOpenAiSpkiRotationPinFromEnv) {
        $spkiRotationPin = [Environment]::GetEnvironmentVariable($OpenAiSpkiRotationPinEnvVar, "Process")
        if ([string]::IsNullOrWhiteSpace($spkiRotationPin)) {
            throw "Environment variable '$OpenAiSpkiRotationPinEnvVar' is not set."
        }
    }

    if (-not $SkipBuild) {
        if ($RequiresFreshKernelBuild) {
            $TempEspDir = Join-Path $env:TEMP "raios-baremetal-esp-$PID"
            Remove-Item -LiteralPath $TempEspDir -Recurse -Force -ErrorAction SilentlyContinue
            Copy-Item -LiteralPath $BaseEspDir -Destination $TempEspDir -Recurse -Force
            $SelectedEspDir = $TempEspDir

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
            if ($AllowUnverifiedOpenAiTls) {
                $buildArgs += "-AllowUnverifiedOpenAiTls"
            }

            & powershell @buildArgs
            if ($LASTEXITCODE -ne 0) {
                exit $LASTEXITCODE
            }

            New-Item -ItemType Directory -Force -Path (Join-Path $SelectedEspDir "kernel") | Out-Null
            Copy-Item -LiteralPath $Kernel -Destination (Join-Path $SelectedEspDir "kernel\kernel.elf") -Force
            Copy-Item -LiteralPath $Kernel -Destination (Join-Path $SelectedEspDir "kernel\seed-kernel.elf") -Force
            Copy-Item -LiteralPath $LimineConfig -Destination (Join-Path $SelectedEspDir "limine.conf") -Force
            Copy-Item -LiteralPath $LimineConfig -Destination (Join-Path $SelectedEspDir "EFI\BOOT\limine.conf") -Force
        }
        else {
            & powershell `
                -NoProfile `
                -ExecutionPolicy Bypass `
                -File (Join-Path $RepoRoot "scripts\package-stage0.ps1") `
                -Profile $Profile
            if ($LASTEXITCODE -ne 0) {
                exit $LASTEXITCODE
            }
        }
    }

    if (-not (Test-Path -LiteralPath (Join-Path $SelectedEspDir "EFI\BOOT\BOOTX64.EFI"))) {
        throw "Missing BOOTX64.EFI in $SelectedEspDir."
    }
    if (-not (Test-Path -LiteralPath (Join-Path $SelectedEspDir "kernel\kernel.elf"))) {
        throw "Missing kernel\kernel.elf in $SelectedEspDir."
    }

    if ($hasExplicitSource) {
        $SelectedEspDir = Assert-ExportedEspPayload `
            -PayloadDir $SelectedEspDir `
            -FirmwarePath $MarvellFirmwarePath `
            -PolicySlot $CorePolicySlot `
            -PolicyGeneration $CorePolicyGeneration `
            -RepoRoot $RepoRoot
    }

    $disk = Assert-DiskIdentity -DiskNumber $DiskNumber -ExpectedIdentity $diskIdentity
    Write-Host "About to erase USB disk ${DiskNumber}: $($disk.FriendlyName) ($([math]::Round($disk.Size / 1GB, 2)) GB)"
    Set-Disk -Number $DiskNumber -IsOffline $false
    Set-Disk -Number $DiskNumber -IsReadOnly $false

    if ($UsePersistLayout) {
        $TempPersistImage = Join-Path $env:TEMP "raios-usb-persist-$PID.img"
        Remove-Item -LiteralPath $TempPersistImage -Force -ErrorAction SilentlyContinue
        & python (Join-Path $RepoRoot "scripts\make-gpt-persist-image.py") `
            --self-check `
            --seed-bootctl valid-a `
            $TempPersistImage
        if ($LASTEXITCODE -ne 0) {
            throw "persist GPT base image build failed with exit code $LASTEXITCODE"
        }
        & python (Join-Path $RepoRoot "scripts\make-gpt-persist-image.py") `
            --image $TempPersistImage `
            --stage-slot A `
            --payload-dir $SelectedEspDir
        if ($LASTEXITCODE -ne 0) {
            throw "persist GPT slot staging failed with exit code $LASTEXITCODE"
        }
        & python (Join-Path $RepoRoot "scripts\make-gpt-persist-image.py") `
            --image $TempPersistImage `
            --seed-bootctl valid-a
        if ($LASTEXITCODE -ne 0) {
            throw "persist GPT bootctl reset failed with exit code $LASTEXITCODE"
        }
        Write-RawImageToDisk `
            -DiskNumber $DiskNumber `
            -ImagePath $TempPersistImage `
            -ExpectedDiskIdentity $diskIdentity
        $inspectJson = & python (Join-Path $RepoRoot "scripts\make-gpt-persist-image.py") --inspect-json $TempPersistImage
        $inspect = ($inspectJson -join [Environment]::NewLine) | ConvertFrom-Json
        Write-Host "raiOS persistent USB image written to PhysicalDrive$DiskNumber"
        Write-Host "Layout: SEED_ESP_A + SEED_ESP_B + SEED_DATA"
        Write-Host "Image bytes: $($inspect.size_bytes)"
        Write-Host "SEED_DATA superblock valid: $($inspect.data_superblock_valid)"
        return
    }

    if ($PartitionStyle -eq "GPT") {
        $driveLetter = Get-FreeDriveLetter
        New-GptFat32BootPartition `
            -DiskNumber $DiskNumber `
            -SizeMB $BootPartitionSizeMB `
            -DriveLetter $driveLetter
    }
    else {
        Clear-Disk -Number $DiskNumber -RemoveData -RemoveOEM -Confirm:$false
        Initialize-Disk -Number $DiskNumber -PartitionStyle MBR
        $partition = New-Partition `
            -DiskNumber $DiskNumber `
            -Size ($BootPartitionSizeMB * 1MB) `
            -AssignDriveLetter `
            -IsActive

        $volume = Format-Volume `
            -Partition $partition `
            -FileSystem FAT32 `
            -NewFileSystemLabel "RAIOS" `
            -Confirm:$false
        $driveLetter = $volume.DriveLetter
        if ([string]::IsNullOrWhiteSpace($driveLetter)) {
            $driveLetter = (Get-Partition -DiskNumber $DiskNumber | Where-Object DriveLetter | Select-Object -First 1).DriveLetter
        }
    }
    if ([string]::IsNullOrWhiteSpace($driveLetter)) {
        throw "Could not determine the assigned USB drive letter."
    }

    $targetRoot = "${driveLetter}:\"
    Copy-Item -Path (Join-Path $SelectedEspDir "*") -Destination $targetRoot -Recurse -Force

    Write-Host "raiOS USB prepared at $targetRoot"
    Write-Host "UEFI boot path: EFI\BOOT\BOOTX64.EFI"
    Write-Host "Kernel path: kernel\kernel.elf"
}
finally {
    if ($PushedRepoRoot) {
        Pop-Location
    }
    if ($TempEspDir) {
        Remove-Item -LiteralPath $TempEspDir -Recurse -Force -ErrorAction SilentlyContinue
    }
    if ($TempPersistImage) {
        Remove-Item -LiteralPath $TempPersistImage -Force -ErrorAction SilentlyContinue
    }
}
