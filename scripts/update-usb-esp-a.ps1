param(
    [int]$DiskNumber = 2,
    [string]$Profile = "release",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

function Test-Admin {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = [Security.Principal.WindowsPrincipal]::new($identity)
    $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

if (-not (Test-Admin)) {
    throw "Run this script from an elevated PowerShell window."
}

$RepoRoot = Split-Path -Parent $PSScriptRoot
$LogPath = Join-Path $env:TEMP ("raios-usb-esp-a-update-disk{0}-{1}.log" -f $DiskNumber, (Get-Date -Format "yyyyMMdd-HHmmss"))

function Log([string]$Message) {
    Write-Host $Message
    Add-Content -LiteralPath $LogPath -Value $Message
}

if (-not $SkipBuild) {
    & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $RepoRoot "scripts\package-stage0.ps1") -Profile $Profile
    if ($LASTEXITCODE -ne 0) {
        throw "package-stage0 failed with exit code $LASTEXITCODE"
    }
}

$SourceEsp = Join-Path $RepoRoot "release\esp"
if (-not (Test-Path -LiteralPath (Join-Path $SourceEsp "EFI\BOOT\BOOTX64.EFI"))) {
    throw "Missing BOOTX64.EFI in $SourceEsp."
}
if (-not (Test-Path -LiteralPath (Join-Path $SourceEsp "kernel\kernel.elf"))) {
    throw "Missing kernel\kernel.elf in $SourceEsp."
}

$disk = Get-Disk -Number $DiskNumber
if ($disk.FriendlyName -notlike "*SanDisk*" -or $disk.IsBoot -or $disk.IsSystem) {
    throw "Refusing Disk ${DiskNumber}: $($disk.FriendlyName) boot=$($disk.IsBoot) system=$($disk.IsSystem)"
}

$partition = Get-Partition -DiskNumber $DiskNumber -PartitionNumber 1
if ($partition.GptType -ne "{c12a7328-f81f-11d2-ba4b-00a0c93ec93b}") {
    throw "Disk $DiskNumber partition 1 is not an EFI System Partition."
}

$letter = @("R", "Y", "X", "W", "V") | Where-Object {
    -not (Get-Volume -DriveLetter $_ -ErrorAction SilentlyContinue)
} | Select-Object -First 1
if (-not $letter) {
    throw "No free drive letter available."
}

$access = "${letter}:"
Log "mounting Disk $DiskNumber partition 1 at $access"
Add-PartitionAccessPath -DiskNumber $DiskNumber -PartitionNumber 1 -AccessPath $access
try {
    $target = "${access}\"
    foreach ($name in @("EFI", "kernel", "raios", "limine-uefi-cd.bin", "limine.conf")) {
        $path = Join-Path $target $name
        if (Test-Path -LiteralPath $path) {
            Remove-Item -LiteralPath $path -Recurse -Force
        }
    }
    Copy-Item -Path (Join-Path $SourceEsp "*") -Destination $target -Recurse -Force

    $srcHash = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $SourceEsp "kernel\kernel.elf")).Hash
    $dstHash = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $target "kernel\kernel.elf")).Hash
    if ($srcHash -ne $dstHash) {
        throw "kernel hash mismatch: src=$srcHash dst=$dstHash"
    }

    $sourcePolicy = Join-Path $SourceEsp "raios\core-policy.bin"
    $targetPolicy = Join-Path $target "raios\core-policy.bin"
    if (Test-Path -LiteralPath $sourcePolicy) {
        if (-not (Test-Path -LiteralPath $targetPolicy)) {
            throw "Core Policy was not copied to ESP A."
        }
        $sourcePolicyHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $sourcePolicy).Hash
        $targetPolicyHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $targetPolicy).Hash
        if ($sourcePolicyHash -ne $targetPolicyHash) {
            throw "Core Policy hash mismatch: src=$sourcePolicyHash dst=$targetPolicyHash"
        }
        Log "core policy sha256 $targetPolicyHash"
    }
    elseif (Test-Path -LiteralPath $targetPolicy) {
        throw "Stale Core Policy remained on ESP A."
    }

    Log "SEED_ESP_A updated on Disk $DiskNumber ($($disk.FriendlyName))"
    Log "kernel sha256 $dstHash"
    Log "log $LogPath"
}
finally {
    Remove-PartitionAccessPath -DiskNumber $DiskNumber -PartitionNumber 1 -AccessPath $access -ErrorAction SilentlyContinue
}
