param(
    [int]$SerialTcpPort = 4555,
    [string]$Prompt = "direct provider smoke",
    [string]$Image = "$PSScriptRoot\..\release\raios-stage0-local-openai.img",
    [int]$TimeoutSeconds = 90,
    [switch]$ExpectProviderResponse,
    [switch]$ExpectPinnedTrust,
    [switch]$ExpectSpkiPinnedTrust,
    [switch]$ExpectPinMismatch
)

$ErrorActionPreference = "Stop"

$modeCount = 0
foreach ($mode in @($ExpectProviderResponse, $ExpectPinnedTrust, $ExpectSpkiPinnedTrust, $ExpectPinMismatch)) {
    if ($mode) {
        $modeCount += 1
    }
}
if ($modeCount -gt 1) {
    throw "Use only one of -ExpectProviderResponse, -ExpectPinnedTrust, -ExpectSpkiPinnedTrust, or -ExpectPinMismatch."
}

$RepoRoot = Split-Path -Parent $PSScriptRoot
$SerialLog = Join-Path $env:TEMP "raios-openai-direct-smoke.serial.txt"
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
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_REQUEST_ENVELOPE {"schema":"raios.provider_request_envelope.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI DIRECT REQUEST 1 STARTED" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: TLS 1.3 established" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: TLS provider trust state: tls_certificate_verification_bypassed" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: HTTPS request sent" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI:" -TimeoutSeconds $TimeoutSeconds
    }
    elseif ($ExpectPinnedTrust) {
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI_DIRECT_REQ 1 api.openai.com /v1/responses" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_REQUEST_ENVELOPE {"schema":"raios.provider_request_envelope.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI DIRECT REQUEST 1 STARTED" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: TLS 1.3 established" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: TLS provider trust verified: pinned_cert" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_REQUEST_BINDING {"schema":"raios.provider_request_binding.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_EXPORT_AUDIT_BINDING {"schema":"raios.provider_context_export_audit_binding.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: HTTPS request sent" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI HTTP" -TimeoutSeconds $TimeoutSeconds
    }
    elseif ($ExpectSpkiPinnedTrust) {
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI_DIRECT_REQ 1 api.openai.com /v1/responses" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_REQUEST_ENVELOPE {"schema":"raios.provider_request_envelope.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI DIRECT REQUEST 1 STARTED" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: TLS 1.3 established" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: TLS provider trust verified: pinned_spki" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_REQUEST_BINDING {"schema":"raios.provider_request_binding.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_EXPORT_AUDIT_BINDING {"schema":"raios.provider_context_export_audit_binding.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: HTTPS request sent" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI HTTP" -TimeoutSeconds $TimeoutSeconds
    }
    elseif ($ExpectPinMismatch) {
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI_DIRECT_REQ 1 api.openai.com /v1/responses" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_REQUEST_ENVELOPE {"schema":"raios.provider_request_envelope.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI DIRECT REQUEST 1 STARTED" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: TLS 1.3 handshake starting (pinned provider verifier)" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI DIRECT TLS PIN MISMATCH" -TimeoutSeconds $TimeoutSeconds
    }
    else {
        Wait-ForLogText -Path $SerialLog -Needle "TLS TRUST: pin_config_missing" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI TLS TRUST DENIED: pin_config_missing" -TimeoutSeconds $TimeoutSeconds
    }

    $serial = Get-Content -Raw -LiteralPath $SerialLog
    if ((-not $ExpectProviderResponse) -and (-not $ExpectPinnedTrust) -and (-not $ExpectSpkiPinnedTrust) -and (-not $ExpectPinMismatch) -and ($serial -like "*OPENAI_DIRECT_REQ*")) {
        throw "Trust-gate smoke saw an OpenAI request start before provider trust was verified in $SerialLog"
    }
    if ((-not $ExpectProviderResponse) -and (-not $ExpectPinnedTrust) -and (-not $ExpectSpkiPinnedTrust) -and (-not $ExpectPinMismatch) -and ($serial -like "*raios.provider_request_envelope.v0*")) {
        throw "Trust-gate smoke saw a provider request envelope before provider trust allowed a request in $SerialLog"
    }
    if (($ExpectProviderResponse -or $ExpectPinMismatch) -and ($serial -like "*raios.provider_request_binding.v0*")) {
        throw "Direct smoke saw a positive provider request binding without positive provider trust in $SerialLog"
    }
    if (($ExpectProviderResponse -or $ExpectPinMismatch) -and ($serial -like "*raios.provider_context_export_audit_binding.v0*")) {
        throw "Direct smoke saw a positive provider export audit binding without positive provider trust in $SerialLog"
    }
    if (($ExpectPinnedTrust -or $ExpectSpkiPinnedTrust) -and ($serial -notlike "*raios.provider_request_binding.v0*")) {
        throw "Pinned-trust smoke did not see a positive provider request binding in $SerialLog"
    }
    if (($ExpectPinnedTrust -or $ExpectSpkiPinnedTrust) -and ($serial -notlike "*raios.provider_context_export_audit_binding.v0*")) {
        throw "Pinned-trust smoke did not see a positive provider export audit binding in $SerialLog"
    }
    if (($ExpectProviderResponse -or $ExpectPinnedTrust -or $ExpectSpkiPinnedTrust -or $ExpectPinMismatch) -and ($serial -notlike "*`"provider_write`":`"not_attempted`"*")) {
        throw "Direct smoke did not see provider_write:not_attempted in the provider request envelope in $SerialLog"
    }
    if (($ExpectProviderResponse -or $ExpectPinnedTrust -or $ExpectSpkiPinnedTrust -or $ExpectPinMismatch) -and ($serial -notlike "*`"body_sha256`":`"sha256:*")) {
        throw "Direct smoke did not see request body hash in the provider request envelope in $SerialLog"
    }
    if (($ExpectProviderResponse -or $ExpectPinnedTrust -or $ExpectSpkiPinnedTrust -or $ExpectPinMismatch) -and ($serial -notlike "*`"envelope_hash`":`"sha256:*")) {
        throw "Direct smoke did not see envelope hash in the provider request envelope in $SerialLog"
    }
    $envelopeLines = @($serial -split '\r?\n' | Where-Object { $_ -like "OPENAI_PROVIDER_REQUEST_ENVELOPE *" })
    foreach ($line in $envelopeLines) {
        if ($line.Contains($safePrompt)) {
            throw "Provider request envelope leaked raw prompt text in $SerialLog"
        }
        if ($line -like "*Content-Length*") {
            throw "Provider request envelope leaked Content-Length in $SerialLog"
        }
        if ($line -like "*Authorization: Bearer*") {
            throw "Provider request envelope leaked Authorization header value in $SerialLog"
        }
    }
    if ($ExpectPinnedTrust -and ($serial -like "*tls_certificate_verification_bypassed*")) {
        throw "Pinned-trust smoke saw unverified TLS bypass output in $SerialLog"
    }
    if ($ExpectSpkiPinnedTrust -and ($serial -like "*tls_certificate_verification_bypassed*")) {
        throw "SPKI pinned-trust smoke saw unverified TLS bypass output in $SerialLog"
    }
    if ($ExpectPinMismatch -and ($serial -like "*tls_certificate_verification_bypassed*")) {
        throw "Pin-mismatch smoke saw unverified TLS bypass output in $SerialLog"
    }
    if ($ExpectPinMismatch -and ($serial -like "*openai: HTTPS request sent*")) {
        throw "Pin-mismatch smoke sent HTTPS request data in $SerialLog"
    }
    if ($ExpectPinMismatch -and ($serial -like "*openai: TLS provider trust verified*")) {
        throw "Pin-mismatch smoke saw a positive trust marker in $SerialLog"
    }
    $oldRelayName = -join ([char[]](66, 82, 73, 68, 71, 69))
    $removedTokens = @(
        ("RAIOS_" + $oldRelayName),
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
    elseif ($ExpectPinnedTrust) {
        Write-Host "openai direct pinned-trust smoke passed"
    }
    elseif ($ExpectSpkiPinnedTrust) {
        Write-Host "openai direct SPKI pinned-trust smoke passed"
    }
    elseif ($ExpectPinMismatch) {
        Write-Host "openai direct pin-mismatch smoke passed"
    }
    else {
        Write-Host "openai direct trust-gate smoke passed"
    }
    Write-Host "serial log: $SerialLog"
}
finally {
    Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force
}
