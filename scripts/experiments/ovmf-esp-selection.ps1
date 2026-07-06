param(
    [Parameter(Mandatory = $true)]
    [string]$PersistImage,
    [string]$PayloadDirA = "",
    [string]$PayloadDirB = "",
    [switch]$StagePayloads,
    [int]$Runs = 3,
    [int]$TimeoutSeconds = 20,
    [string]$LogPath = "$env:TEMP\raios-ovmf-esp-selection.log"
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$SwitchScript = Join-Path $RepoRoot "scripts\switch-boot-slot.ps1"
$Qemu = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$CodeSrc = "C:\Program Files\qemu\share\edk2-x86_64-code.fd"
$VarsSrc = Join-Path $RepoRoot "release\ovmf_vars.fd"

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
        throw "refusing release/ persist image for OVMF experiment: $resolved"
    }
}

function Append-Log([string]$Message) {
    Add-Content -LiteralPath $LogPath -Value $Message
    Write-Output $Message
}

$imagePath = Resolve-ExistingPath $PersistImage "PersistImage"
Assert-NotReleasePath $imagePath
if ($Runs -lt 1) {
    throw "Runs must be at least 1"
}

Remove-Item -LiteralPath $LogPath -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $LogPath) | Out-Null
Append-Log "OVMF ESP-selection experiment (non-gating)"
Append-Log "persist_image=$imagePath"
Append-Log "measures=direct OVMF boot/serial/stderr output for a GPT persist disk; no VM predicate or slot winner is asserted"

if ($StagePayloads) {
    if (-not $PayloadDirA -or -not $PayloadDirB) {
        throw "-StagePayloads requires both -PayloadDirA and -PayloadDirB"
    }
    $payloadA = Resolve-ExistingPath $PayloadDirA "PayloadDirA"
    $payloadB = Resolve-ExistingPath $PayloadDirB "PayloadDirB"
    Append-Log "staging=using switch-boot-slot.ps1 -Apply for A and B"
    & powershell -NoProfile -ExecutionPolicy Bypass -File $SwitchScript -PersistImage $imagePath -Slot A -PayloadDir $payloadA -Apply | Tee-Object -FilePath $LogPath -Append
    if ($LASTEXITCODE -ne 0) {
        throw "slot A staging failed"
    }
    & powershell -NoProfile -ExecutionPolicy Bypass -File $SwitchScript -PersistImage $imagePath -Slot B -PayloadDir $payloadB -Apply | Tee-Object -FilePath $LogPath -Append
    if ($LASTEXITCODE -ne 0) {
        throw "slot B staging failed"
    }
}
else {
    Append-Log "staging=not performed; this run only records firmware-visible boot/enumeration output"
}

foreach ($run in 1..$Runs) {
    $code = Join-Path $env:TEMP "raios-ovmf-esp-code-$run.fd"
    $vars = Join-Path $env:TEMP "raios-ovmf-esp-vars-$run.fd"
    $serial = Join-Path $env:TEMP "raios-ovmf-esp-selection-$run.serial.txt"
    $err = Join-Path $env:TEMP "raios-ovmf-esp-selection-$run.err.txt"
    Copy-Item -LiteralPath $CodeSrc -Destination $code -Force
    Copy-Item -LiteralPath $VarsSrc -Destination $vars -Force
    Remove-Item -LiteralPath $serial, $err -Force -ErrorAction SilentlyContinue

    $qemuArgs = @(
        "-machine", "q35",
        "-m", "512M",
        "-drive", "if=pflash,format=raw,readonly=on,file=$code",
        "-drive", "if=pflash,format=raw,file=$vars",
        "-drive", "file=$imagePath,format=raw,if=ide,index=0",
        "-display", "none",
        "-serial", "file:$serial",
        "-net", "none",
        "-no-reboot"
    )

    Append-Log "run=$run status=starting serial=$serial stderr=$err"
    $process = Start-Process -FilePath $Qemu -ArgumentList $qemuArgs -PassThru -WindowStyle Hidden -RedirectStandardError $err
    if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
        Stop-Process -Id $process.Id -Force
        $status = "timeout_stopped"
    }
    else {
        $status = "exited_$($process.ExitCode)"
    }

    $serialText = ""
    if (Test-Path -LiteralPath $serial) {
        $serialText = Get-Content -Raw -LiteralPath $serial
    }
    $errText = ""
    if (Test-Path -LiteralPath $err) {
        $errText = Get-Content -Raw -LiteralPath $err
    }
    $interesting = @($serialText, $errText) -join "`n"
    $observed = @($interesting -split "`r?`n" | Where-Object { $_ -match "SLOT [AB]|FS[0-9]+:|BdsDxe|OVMF|EDK|Tiano|UEFI|Boot" } | Select-Object -First 30)
    Append-Log "run=$run status=$status observed_lines=$($observed.Count)"
    foreach ($line in $observed) {
        Append-Log "  $line"
    }
}

Append-Log "conclusion=inconclusive unless the log contains a self-identifying SLOT A/SLOT B banner; deterministic firmware slot boot is not claimed"
