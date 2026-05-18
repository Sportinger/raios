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
    [switch]$AllowUnverifiedOpenAiTls,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

$RequiresFreshKernelBuild = $EmbedOpenAiApiKeyFromEnv -or $EmbedOpenAiCertPinFromEnv -or $AllowUnverifiedOpenAiTls
if ($RequiresFreshKernelBuild -and $SkipBuild) {
    throw "Refusing -SkipBuild with provider trust/key build flags because they must be compiled into a fresh local kernel before writing the USB stick."
}

function Test-Admin {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = [Security.Principal.WindowsPrincipal]::new($identity)
    $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
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
if ($BootPartitionSizeMB -lt 64 -or $BootPartitionSizeMB -gt 32768) {
    throw "BootPartitionSizeMB must be between 64 and 32768."
}

$RepoRoot = Split-Path -Parent $PSScriptRoot
$BaseEspDir = Join-Path $RepoRoot "release\esp"
$TempEspDir = $null
$SourceEspDir = $BaseEspDir
$KernelProfileDir = if ($Profile -eq "release") { "release" } else { "debug" }
$Kernel = Join-Path $RepoRoot "target\x86_64-seed\$KernelProfileDir\seed-kernel"
$LimineConfig = Join-Path $RepoRoot "seed-kernel\limine\limine.conf"
$PushedRepoRoot = $false

try {
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

    if (-not $SkipBuild) {
        if ($RequiresFreshKernelBuild) {
            $TempEspDir = Join-Path $env:TEMP "raios-baremetal-esp-$PID"
            Remove-Item -LiteralPath $TempEspDir -Recurse -Force -ErrorAction SilentlyContinue
            Copy-Item -LiteralPath $BaseEspDir -Destination $TempEspDir -Recurse -Force
            $SourceEspDir = $TempEspDir

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
            if ($AllowUnverifiedOpenAiTls) {
                $buildArgs += "-AllowUnverifiedOpenAiTls"
            }

            & powershell @buildArgs
            if ($LASTEXITCODE -ne 0) {
                exit $LASTEXITCODE
            }

            New-Item -ItemType Directory -Force -Path (Join-Path $SourceEspDir "kernel") | Out-Null
            Copy-Item -LiteralPath $Kernel -Destination (Join-Path $SourceEspDir "kernel\kernel.elf") -Force
            Copy-Item -LiteralPath $Kernel -Destination (Join-Path $SourceEspDir "kernel\seed-kernel.elf") -Force
            Copy-Item -LiteralPath $LimineConfig -Destination (Join-Path $SourceEspDir "limine.conf") -Force
            Copy-Item -LiteralPath $LimineConfig -Destination (Join-Path $SourceEspDir "EFI\BOOT\limine.conf") -Force
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

    if (-not (Test-Path -LiteralPath (Join-Path $SourceEspDir "EFI\BOOT\BOOTX64.EFI"))) {
        throw "Missing BOOTX64.EFI in $SourceEspDir."
    }
    if (-not (Test-Path -LiteralPath (Join-Path $SourceEspDir "kernel\kernel.elf"))) {
        throw "Missing kernel\kernel.elf in $SourceEspDir."
    }

    Write-Host "About to erase USB disk ${DiskNumber}: $($disk.FriendlyName) ($([math]::Round($disk.Size / 1GB, 2)) GB)"
    Set-Disk -Number $DiskNumber -IsOffline $false
    Set-Disk -Number $DiskNumber -IsReadOnly $false

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
    Copy-Item -Path (Join-Path $SourceEspDir "*") -Destination $targetRoot -Recurse -Force

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
}
