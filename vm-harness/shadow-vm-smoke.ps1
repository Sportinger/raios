param(
    [int]$SerialTcpPort = 4565,
    [string]$Image = "",
    [string]$ArtifactPath = "",
    [string]$ManifestPath = "",
    [string]$ReportDir = "$PSScriptRoot\..\release\vm-reports",
    [int]$TimeoutSeconds = 45,
    [switch]$Network,
    [switch]$KeepImage,
    [int]$SerialWriteChunkSize = 256,
    [int]$SerialWriteDelayMilliseconds = 0,
    [ValidateSet("full", "quick", "recovery")]
    [string]$Profile = "full"
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$RunScript = Join-Path $RepoRoot "scripts\run-stage0-qemu.ps1"
$PackageScript = Join-Path $RepoRoot "scripts\package-stage0.ps1"
$ValidateManifestScript = Join-Path $PSScriptRoot "validate-module-manifest.ps1"
$RunId = "shadow-{0:yyyyMMdd-HHmmss}-{1}" -f (Get-Date), $PID
$RunDir = Join-Path $env:TEMP "raios-$RunId"
$SerialLog = Join-Path $RunDir "serial.log"
$ReportPath = Join-Path $ReportDir "$RunId.json"
$ReportHashPath = "$ReportPath.sha256"
$QemuPid = $null
$TempImage = $false
$Result = "failed"
$Failures = New-Object System.Collections.Generic.List[string]
$Predicates = New-Object System.Collections.Generic.List[object]
$ExecutedCommands = New-Object System.Collections.Generic.List[object]
$StartedAt = [DateTime]::UtcNow
$script:SerialTcpDrainStream = $null
$QemuArgList = @()
$HardwareProfile = $null
$ResolvedImage = $null
$ResolvedArtifact = $null
$ResolvedManifest = $null
$ManifestValidation = $null
$script:SerialLogCachePath = $null
$script:SerialLogCacheLength = [int64]-1
$script:SerialLogCacheWriteTicks = [int64]-1
$script:SerialLogCacheContent = $null

. (Join-Path $PSScriptRoot "shadow-vm-smoke-support.ps1")

New-Item -ItemType Directory -Force -Path $RunDir | Out-Null

$ResolvedArtifact = Resolve-OptionalPath -Path $ArtifactPath
$ResolvedManifest = Resolve-OptionalPath -Path $ManifestPath

try {
    if ($ResolvedArtifact -and -not $ResolvedManifest) {
        throw "ArtifactPath requires ManifestPath; artifacts must not enter the evidence flow without a manifest"
    }
    if ($ResolvedManifest) {
        $validationParams = @{
            ManifestPath = $ResolvedManifest
        }
        if ($ResolvedArtifact) {
            $validationParams.ArtifactPath = $ResolvedArtifact
        }
        $validationJson = & $ValidateManifestScript @validationParams
        $ManifestValidation = ($validationJson -join [Environment]::NewLine) | ConvertFrom-Json
        if (-not $ManifestValidation.valid) {
            throw "Manifest validation failed"
        }
    }

    if ($Image) {
        $ResolvedImage = (Resolve-Path -LiteralPath $Image).Path
    }
    else {
        $ResolvedImage = Join-Path $RunDir "raios-stage0-shadow.img"
        $TempImage = $true
        & $PackageScript -Profile release -Image $ResolvedImage -UseTempEsp
        if ($LASTEXITCODE -ne 0) {
            throw "Image packaging failed with exit code $LASTEXITCODE"
        }
    }

    $Nic = if ($Network) { "e1000" } else { "none" }
    $HardwareProfile = New-HardwareProfile -Nic $Nic
    $QemuArgList = @(
        "-StopExisting",
        "-Image", $ResolvedImage,
        "-SerialMode", "tcp",
        "-SerialTcpPort", "$SerialTcpPort",
        "-SerialLog", $SerialLog,
        "-Headless",
        "-UsbXhciInput",
        "-Cpu", "max",
        "-Nic", $Nic
    )
    $runParams = @{
        StopExisting = $true
        Image = $ResolvedImage
        SerialMode = "tcp"
        SerialTcpPort = $SerialTcpPort
        SerialLog = $SerialLog
        Headless = $true
        UsbXhciInput = $true
        Cpu = "max"
        Nic = $Nic
    }

    $runOutput = & $RunScript @runParams
    foreach ($line in $runOutput) {
        if ($line -match '^qemu pid:\s*(\d+)') {
            $QemuPid = [int]$Matches[1]
        }
    }
    if (-not $QemuPid) {
        throw "Could not parse QEMU pid from runner output"
    }

    Assert-LogContains -Name "boot:serial_console_ready" -Needle "SERIAL CONSOLE READY" -TimeoutSeconds $TimeoutSeconds
    Assert-LogContains -Name "boot:framebuffer_ready" -Needle "status FRAMEBUFFER: READY" -TimeoutSeconds $TimeoutSeconds
    Assert-LogContains -Name "boot:usb_xhci_ready" -Needle "status USB-XHCI: READY" -TimeoutSeconds $TimeoutSeconds

    :SmokeProfileValidation while ($true) {
        . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-common.ps1")

        if ($Profile -eq "quick") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-quick.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "full") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-full-provider-memory.ps1")
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-full-module-evidence.ps1")
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-full-module-audit-rollback.ps1")
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-full-module-selftests.ps1")
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-full-module-load-gate.ps1")
        }

        . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-recovery-artifact-evidence.ps1")
        . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-recovery-lifeline-foundation.ps1")
        . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-recovery-command-frontdoor.ps1")
        . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-recovery-command-authority.ps1")
        . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-recovery-command-effects.ps1")
        . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-recovery-execution-binding.ps1")

        if ($Profile -eq "recovery") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-recovery-audit.ps1")
            break SmokeProfileValidation
        }

        . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-full-audit.ps1")
        break SmokeProfileValidation
    }

    $Result = "passed"
}
catch {
    $Failures.Add($_.Exception.Message) | Out-Null
    throw
}
finally {
    if ($QemuPid) {
        Stop-Process -Id $QemuPid -Force -ErrorAction SilentlyContinue
    }

    Write-Report `
        -FinalResult $Result `
        -ResolvedImage $ResolvedImage `
        -ResolvedArtifact $ResolvedArtifact `
        -ResolvedManifest $ResolvedManifest `
        -QemuArgList $QemuArgList `
        -HardwareProfile $HardwareProfile `
        -StartedAt $StartedAt

    if ($TempImage -and -not $KeepImage) {
        Remove-Item -LiteralPath $ResolvedImage -Force -ErrorAction SilentlyContinue
    }

    Write-Host "shadow vm result: $Result"
    Write-Host "report: $ReportPath"
    Write-Host "report sha256: $ReportHashPath"
    Write-Host "serial log: $SerialLog"
}
