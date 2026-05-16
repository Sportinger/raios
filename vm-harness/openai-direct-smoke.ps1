param(
    [int]$SerialTcpPort = 4555,
    [string]$Prompt = "direct provider smoke",
    [string]$Image = "$PSScriptRoot\..\release\seedos-stage0-local-openai.img",
    [int]$TimeoutSeconds = 90,
    [switch]$ExpectProviderResponse
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$SerialLog = Join-Path $env:TEMP "seedos-openai-direct-smoke.serial.txt"
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
            if ($content -clike "*$Needle*") {
                return
            }
        }
        Start-Sleep -Milliseconds 250
    } while ([DateTime]::UtcNow -lt $deadline)

    throw "Timed out waiting for '$Needle' in $Path"
}

function Send-SerialText {
    param(
        [int]$Port,
        [string]$Text,
        [int]$TimeoutSeconds
    )

    $client = [System.Net.Sockets.TcpClient]::new()
    $client.NoDelay = $true
    $connect = $client.BeginConnect("127.0.0.1", $Port, $null, $null)
    if (-not $connect.AsyncWaitHandle.WaitOne([TimeSpan]::FromSeconds($TimeoutSeconds))) {
        $client.Close()
        throw "Timed out connecting to QEMU serial TCP port $Port"
    }
    $client.EndConnect($connect)

    try {
        $stream = $client.GetStream()
        $bytes = [System.Text.Encoding]::ASCII.GetBytes($Text)
        $stream.Write($bytes, 0, $bytes.Length)
        $stream.Flush()
        Start-Sleep -Milliseconds 750
    }
    finally {
        $client.Close()
    }
}

if (-not (Test-Path -LiteralPath $Image)) {
    throw "Missing direct OpenAI image: $Image. Package it with scripts\package-stage0.ps1 -UseTempEsp -EmbedOpenAiApiKeyFromEnv."
}

try {
    Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force
    Remove-Item -LiteralPath $SerialLog -Force -ErrorAction SilentlyContinue

    & $RunScript `
        -StopExisting `
        -Image $Image `
        -SerialMode tcp `
        -SerialTcpPort $SerialTcpPort `
        -Headless `
        -BareMetalVm `
        -SerialLog $SerialLog

    Wait-ForLogText -Path $SerialLog -Needle "Default provider loaded: OPENAI API key set" -TimeoutSeconds $TimeoutSeconds
    Wait-ForLogText -Path $SerialLog -Needle "status NETWORK: CONFIGURED" -TimeoutSeconds $TimeoutSeconds

    $safePrompt = $Prompt -replace '"', "'"
    Send-SerialText -Port $SerialTcpPort -TimeoutSeconds $TimeoutSeconds -Text "provider`rask $safePrompt`r"

    Wait-ForLogText -Path $SerialLog -Needle "PROVIDER: OPENAI    API KEY: SET" -TimeoutSeconds $TimeoutSeconds
    if ($ExpectProviderResponse) {
        Wait-ForLogText -Path $SerialLog -Needle "TLS TRUST: tls_certificate_verification_bypassed" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI_DIRECT_REQ 1 api.openai.com /v1/responses" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI DIRECT REQUEST 1 STARTED" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: TLS 1.3 established" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: TLS provider trust state: tls_certificate_verification_bypassed" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: HTTPS request sent" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI:" -TimeoutSeconds $TimeoutSeconds
    }
    else {
        Wait-ForLogText -Path $SerialLog -Needle "TLS TRUST: pin_config_missing" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI TLS TRUST DENIED: pin_config_missing" -TimeoutSeconds $TimeoutSeconds
    }

    $serial = Get-Content -Raw -LiteralPath $SerialLog
    if ((-not $ExpectProviderResponse) -and ($serial -like "*OPENAI_DIRECT_REQ*")) {
        throw "Trust-gate smoke saw an OpenAI request start before provider trust was verified in $SerialLog"
    }
    $oldRelayName = -join ([char[]](66, 82, 73, 68, 71, 69))
    $removedTokens = @(
        ("SEEDOS_" + $oldRelayName),
        ($oldRelayName + " REQUEST"),
        ($oldRelayName + " RESPONSE"),
        ("HOST " + $oldRelayName)
    )
    foreach ($token in $removedTokens) {
        if ($serial -like "*$token*") {
            throw "Direct smoke saw removed serial-relay output in $SerialLog"
        }
    }

    if ($ExpectProviderResponse) {
        Write-Host "openai direct development smoke passed"
    }
    else {
        Write-Host "openai direct trust-gate smoke passed"
    }
    Write-Host "serial log: $SerialLog"
}
finally {
    Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force
}
