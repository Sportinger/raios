param(
    [Parameter(Mandatory = $true)]
    [string]$PersistImage,
    [Parameter(Mandatory = $true)]
    [ValidateSet("A", "B")]
    [string]$Slot,
    [string]$PayloadDir = "",
    [switch]$Apply
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$PythonTool = Join-Path $PSScriptRoot "make-gpt-persist-image.py"

function Resolve-ExistingPath([string]$Path, [string]$Label) {
    if (-not $Path) {
        throw "$Label path is required"
    }
    return (Resolve-Path -LiteralPath $Path).Path
}

function Assert-NotReleasePath([string]$Path) {
    $releaseRoot = (Resolve-Path -LiteralPath (Join-Path $RepoRoot "release")).Path
    $releaseRoot = $releaseRoot.TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
    $resolved = (Resolve-Path -LiteralPath $Path).Path
    if ($resolved.Equals($releaseRoot, [System.StringComparison]::OrdinalIgnoreCase) -or
        $resolved.StartsWith($releaseRoot + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase) -or
        $resolved.StartsWith($releaseRoot + [System.IO.Path]::AltDirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "refusing to modify release/ image: $resolved"
    }
}

function Invoke-PythonReleaseGuard([string]$ImagePath) {
    $oldErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        $output = & python $PythonTool --assert-not-release $ImagePath 2>&1
        $exitCode = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $oldErrorActionPreference
    }
    if ($exitCode -ne 0) {
        throw ($output -join "`n")
    }
}

function Inspect-PersistImage([string]$ImagePath) {
    $oldErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        $output = & python $PythonTool --inspect-json $ImagePath 2>&1
        $exitCode = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $oldErrorActionPreference
    }
    if ($exitCode -ne 0) {
        throw ($output -join "`n")
    }
    return ($output -join "`n") | ConvertFrom-Json
}

function Assert-GptPersistImage($Inspect) {
    if (-not ($Inspect.gpt_header_valid -and $Inspect.gpt_seed_data_found -and $Inspect.data_superblock_valid)) {
        throw "refusing non-GPT or unprovisioned persist image: gpt_header_valid=$($Inspect.gpt_header_valid) gpt_seed_data_found=$($Inspect.gpt_seed_data_found) data_superblock_valid=$($Inspect.data_superblock_valid)"
    }
}

function Get-StorageRecord($Bootctl, [string]$StorageSlot) {
    if ($StorageSlot -eq "A") {
        return $Bootctl.storage_slot_a
    }
    if ($StorageSlot -eq "B") {
        return $Bootctl.storage_slot_b
    }
    return $null
}

function Get-PayloadFiles([string]$RootPath) {
    if (-not $RootPath) {
        return @()
    }
    $root = (Get-Item -LiteralPath $RootPath).FullName.TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
    return @(Get-ChildItem -LiteralPath $RootPath -Recurse -File | Sort-Object FullName | ForEach-Object {
        $_.FullName.Substring($root.Length).TrimStart([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar).Replace("\", "/")
    })
}

function New-SwitchPlan($Inspect, [string]$ImagePath, [string]$TargetSlot, [string]$ResolvedPayloadDir, [string[]]$Files, [string]$Mode) {
    $bootctl = $Inspect.bootctl_read
    $decision = $bootctl.decision
    $authoritative = $decision.authoritative_bootctl_slot
    $reason = $decision.reason
    if ([string]::IsNullOrEmpty($authoritative)) {
        if ($reason -eq "no_valid_boot_control_slot" -or $reason -eq "both_slots_invalid") {
            $authoritativeSeq = 0
            $loser = "A"
        }
        else {
            throw "cannot choose loser BOOTCTL slot without an authoritative record: $reason"
        }
    }
    else {
        $record = Get-StorageRecord $bootctl $authoritative
        $authoritativeSeq = [int64]$record.seq
        if ($authoritative -eq "A") {
            $loser = "B"
        }
        else {
            $loser = "A"
        }
    }

    $slotSectors = [int64]$Inspect.constants.BOOTCTL.storage_slot_sectors
    $bootctlRelativeLba = [int64]$Inspect.constants.BOOTCTL.start_lba
    if ($loser -eq "B") {
        $bootctlRelativeLba += $slotSectors
    }
    $seedData = $Inspect.partitions | Where-Object { $_.name -eq "SEED_DATA" } | Select-Object -First 1
    $esp = $Inspect.partitions | Where-Object { $_.name -eq "SEED_ESP_$TargetSlot" } | Select-Object -First 1
    if (-not $seedData -or -not $esp) {
        throw "required SEED_DATA or SEED_ESP_$TargetSlot partition is missing"
    }

    return [ordered]@{
        mode = $Mode
        image = $ImagePath
        target_payload_slot = $TargetSlot
        stage_payload = [bool]$ResolvedPayloadDir
        payload_dir = $(if ($ResolvedPayloadDir) { $ResolvedPayloadDir } else { $null })
        payload_files = $Files
        esp = [ordered]@{
            slot = $TargetSlot
            start_lba = [int64]$esp.first_lba
        }
        boot_control = [ordered]@{
            current_authoritative_storage_slot = $(if ($authoritative) { $authoritative } else { $null })
            current_authoritative_seq = $authoritativeSeq
            write_storage_slot = $loser
            write_relative_lba = $bootctlRelativeLba
            write_absolute_lba = [int64]$seedData.first_lba + $bootctlRelativeLba
            new_seq = $authoritativeSeq + 1
            pending = $TargetSlot
            target_slot_state = "pending"
            success_marked = $false
        }
    }
}

$imagePath = Resolve-ExistingPath $PersistImage "PersistImage"
Assert-NotReleasePath $imagePath
Invoke-PythonReleaseGuard $imagePath

$resolvedPayloadDir = ""
if ($PayloadDir) {
    $resolvedPayloadDir = Resolve-ExistingPath $PayloadDir "PayloadDir"
    if (-not (Test-Path -LiteralPath $resolvedPayloadDir -PathType Container)) {
        throw "PayloadDir is not a directory: $resolvedPayloadDir"
    }
}

$inspect = Inspect-PersistImage $imagePath
Assert-GptPersistImage $inspect
$files = Get-PayloadFiles $resolvedPayloadDir
$plan = New-SwitchPlan $inspect $imagePath $Slot $resolvedPayloadDir $files $(if ($Apply) { "apply" } else { "dry-run" })

if (-not $Apply) {
    Write-Output "DRY-RUN plan; no writes performed."
    Write-Output ($plan | ConvertTo-Json -Depth 6)
    exit 0
}

Write-Output "APPLY plan:"
Write-Output ($plan | ConvertTo-Json -Depth 6)

$args = @($PythonTool, "--image", $imagePath)
if ($resolvedPayloadDir) {
    $args += @("--stage-slot", $Slot, "--payload-dir", $resolvedPayloadDir, "--set-pending", $Slot)
}
else {
    $args += @("--set-pending", $Slot)
}

$oldErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = "Continue"
try {
    $applyOutput = & python @args 2>&1
    $applyExitCode = $LASTEXITCODE
}
finally {
    $ErrorActionPreference = $oldErrorActionPreference
}
if ($applyExitCode -ne 0) {
    throw ($applyOutput -join "`n")
}
Write-Output "python apply:"
Write-Output ($applyOutput -join "`n")

$post = Inspect-PersistImage $imagePath
$postDecision = $post.bootctl_read.decision
$postRecord = Get-StorageRecord $post.bootctl_read $postDecision.authoritative_bootctl_slot
$postScan = [ordered]@{
    authoritative_storage_slot = $postDecision.authoritative_bootctl_slot
    seq = $postDecision.seq
    pending = $postRecord.pending
    selected_payload_slot = $postDecision.selected_payload_slot
    posture = $postDecision.posture
    reason = $postDecision.reason
}
Write-Output "post-write scan:"
Write-Output ($postScan | ConvertTo-Json -Depth 4)
