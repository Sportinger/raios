param(
    [int]$SerialTcpPort = 4555,
    [string]$Prompt = "ping from vm harness",
    [int]$TimeoutSeconds = 30
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$SerialLog = Join-Path $env:TEMP "seedos-host-bridge-smoke.serial.txt"
$BridgeOut = Join-Path $env:TEMP "seedos-host-bridge-smoke.bridge.out.txt"
$BridgeScript = Join-Path $RepoRoot "scripts\host-bridge.ps1"
$RunScript = Join-Path $RepoRoot "scripts\run-stage0-qemu.ps1"

function Wait-ForLogText {
    param(
        [string]$Path,
        [string]$Needle,
        [int]$TimeoutSeconds
    )

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    do {
        if (Test-Path -LiteralPath $Path) {
            $content = Get-Content -Raw -LiteralPath $Path -ErrorAction SilentlyContinue
            if ($content -like "*$Needle*") {
                return
            }
        }
        Start-Sleep -Milliseconds 250
    } while ([DateTime]::UtcNow -lt $deadline)

    throw "Timed out waiting for '$Needle' in $Path"
}

try {
    Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force
    Remove-Item -LiteralPath $SerialLog, $BridgeOut -Force -ErrorAction SilentlyContinue

    & $RunScript `
        -StopExisting `
        -SerialMode tcp `
        -SerialTcpPort $SerialTcpPort `
        -Headless `
        -SerialLog $SerialLog

    $safePrompt = $Prompt -replace '"', "'"
    $bridgeOutput = & powershell `
        -NoProfile `
        -ExecutionPolicy Bypass `
        -File $BridgeScript `
        -Port $SerialTcpPort `
        -Once `
        -Ask $safePrompt `
        -RequestTimeoutSeconds $TimeoutSeconds 2>&1
    $bridgeExit = $LASTEXITCODE
    $bridgeOutput | Set-Content -LiteralPath $BridgeOut
    if ($bridgeExit -ne 0) {
        throw "Host bridge failed with exit code ${bridgeExit}: $($bridgeOutput -join "`n")"
    }

    Wait-ForLogText -Path $SerialLog -Needle "SEEDOS_BRIDGE_REQ" -TimeoutSeconds $TimeoutSeconds
    Wait-ForLogText -Path $SerialLog -Needle "BRIDGE RESPONSE 1: HOST BRIDGE OK: $safePrompt" -TimeoutSeconds $TimeoutSeconds

    Write-Host "host bridge smoke passed"
    Write-Host "serial log: $SerialLog"
    Write-Host "bridge output: $BridgeOut"
}
finally {
    Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force
}
