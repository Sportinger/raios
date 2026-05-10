param(
    [int]$SerialTcpPort = 4555,
    [string]$Prompt = "hi",
    [string]$Image = "$PSScriptRoot\..\release\seedos-stage0-local-openai.img",
    [int]$TimeoutSeconds = 90
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$SerialLog = Join-Path $env:TEMP "seedos-openai-bridge-smoke.serial.txt"
$BridgeOut = Join-Path $env:TEMP "seedos-openai-bridge-smoke.bridge.out.txt"
$BridgeScript = Join-Path $RepoRoot "scripts\host-bridge.ps1"
$RunScript = Join-Path $RepoRoot "scripts\run-stage0-qemu.ps1"

if (-not $env:OPENAI_API_KEY) {
    throw "OPENAI_API_KEY is not set in this PowerShell process."
}

if (-not (Test-Path -LiteralPath $Image)) {
    $Image = Join-Path $RepoRoot "release\seedos-stage0.img"
}

try {
    Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force
    Remove-Item -LiteralPath $SerialLog, $BridgeOut -Force -ErrorAction SilentlyContinue

    & $RunScript `
        -StopExisting `
        -Image $Image `
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
        -Provider openai `
        -Once `
        -Ask $safePrompt `
        -RequestTimeoutSeconds $TimeoutSeconds `
        -OpenAiTimeoutSeconds $TimeoutSeconds 2>&1
    $bridgeExit = $LASTEXITCODE
    $bridgeOutput | Set-Content -LiteralPath $BridgeOut
    if ($bridgeExit -ne 0) {
        throw "OpenAI bridge failed with exit code ${bridgeExit}: $($bridgeOutput -join "`n")"
    }

    $serial = Get-Content -Raw -LiteralPath $SerialLog
    if ($serial -notlike "*BRIDGE RESPONSE 1:*") {
        throw "Timed out waiting for an OpenAI bridge response in $SerialLog"
    }
    if ($serial -like "*HOST BRIDGE OK*") {
        throw "OpenAI bridge fell back to the echo response."
    }

    Write-Host "openai bridge smoke passed"
    Write-Host "serial log: $SerialLog"
    Write-Host "bridge output: $BridgeOut"
}
finally {
    Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force
}
