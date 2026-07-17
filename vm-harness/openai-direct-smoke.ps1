param(
    [int]$SerialTcpPort = 4555,
    [string]$Prompt = "direct provider smoke",
    [string]$Image = "$PSScriptRoot\..\release\raios-stage0-local-openai.img",
    [int]$TimeoutSeconds = 90,
    [switch]$ExpectProviderResponse,
    [switch]$ExpectPinnedTrust,
    [switch]$ExpectSpkiPinnedTrust,
    [switch]$ExpectPinMismatch,
    [switch]$ExpectProjectWorkspaceAnswer
)

$ErrorActionPreference = "Stop"

$modeCount = 0
foreach ($mode in @($ExpectProviderResponse, $ExpectPinnedTrust, $ExpectSpkiPinnedTrust, $ExpectProjectWorkspaceAnswer, $ExpectPinMismatch)) {
    if ($mode) {
        $modeCount += 1
    }
}
if ($modeCount -gt 1) {
    throw "Use only one of -ExpectProviderResponse, -ExpectPinnedTrust, -ExpectSpkiPinnedTrust, -ExpectProjectWorkspaceAnswer, or -ExpectPinMismatch."
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

function Wait-ForLogLineMatch {
    param(
        [string]$Path,
        [string]$Pattern,
        [int]$TimeoutSeconds,
        [string]$FailurePattern = ""
    )

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    do {
        if (Test-Path -LiteralPath $Path) {
            $content = Get-Content -Raw -LiteralPath $Path -ErrorAction SilentlyContinue
            $matchedLine = $null
            foreach ($rawLine in @($content -split '\n')) {
                $line = $rawLine.TrimEnd()
                if ($FailurePattern -and ($line -match $FailurePattern)) {
                    throw "Guest reported harness failure: $line"
                }
                $match = [regex]::Match($line, $Pattern)
                if ($match.Success) {
                    $matchedLine = [pscustomobject]@{ Line = $line; Match = $match }
                }
            }
            if ($null -ne $matchedLine) {
                return $matchedLine
            }
        }
        Start-Sleep -Milliseconds 250
    } while ([DateTime]::UtcNow -lt $deadline)

    throw "Timed out waiting for line matching '$Pattern' in $Path"
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

function Get-MarkerJson {
    param(
        [string]$Serial,
        [string]$Prefix
    )

    $line = @($Serial -split '\r?\n' | Where-Object { $_ -like "$Prefix *" } | Select-Object -Last 1)
    if (-not $line) {
        throw "Missing marker $Prefix in serial log"
    }
    return ($line.Substring($Prefix.Length + 1) | ConvertFrom-Json)
}

function Get-AgentResponseJson {
    param(
        [string]$Serial,
        [string]$Method
    )

    $begin = "RAIOS_AGENT_BEGIN $Method"
    $end = "RAIOS_AGENT_END $Method"
    $beginIndex = $Serial.LastIndexOf($begin, [System.StringComparison]::Ordinal)
    if ($beginIndex -lt 0) {
        throw "No agent response for method '$Method' found in serial log"
    }
    $jsonStart = $Serial.IndexOf("{", $beginIndex, [System.StringComparison]::Ordinal)
    if ($jsonStart -lt 0) {
        throw "No JSON body for method '$Method' found in serial log"
    }
    $endIndex = $Serial.IndexOf($end, $jsonStart, [System.StringComparison]::Ordinal)
    if ($endIndex -lt 0) {
        throw "Incomplete agent response for method '$Method' found in serial log"
    }
    return $Serial.Substring($jsonStart, $endIndex - $jsonStart).Trim() | ConvertFrom-Json
}

function Get-RedactedSerialLogTail {
    param(
        [string]$Path,
        [int]$LineCount = 200
    )

    if (-not (Test-Path -LiteralPath $Path)) {
        return "<serial log missing>"
    }
    $tail = (Get-Content -LiteralPath $Path -Tail $LineCount -ErrorAction SilentlyContinue) -join [Environment]::NewLine
    return [regex]::Replace($tail, '(?im)(Authorization:\s*Bearer\s+)[^\r\n]*', '${1}<redacted>')
}

function Assert-Equal {
    param(
        [string]$Name,
        $Actual,
        $Expected
    )

    if ($Actual -ne $Expected) {
        throw "$Name mismatch. Expected '$Expected' but saw '$Actual'."
    }
}

function Assert-PositiveTrustDecision {
    param(
        [string]$Name,
        $Decision
    )

    Assert-Equal -Name "$Name schema" -Actual $Decision.schema -Expected "raios.provider_trust_verifier_decision.v0"
    Assert-Equal -Name "$Name verifier id" -Actual $Decision.verifier_id -Expected "openai.pinned_tls13_p256_sha256.v0"
    Assert-Equal -Name "$Name stage" -Actual $Decision.stage -Expected "certificate_verify"
    Assert-Equal -Name "$Name outcome" -Actual $Decision.outcome -Expected "verified"
    $validReasons = @(
        "leaf_cert_pin_and_certificate_verify_valid",
        "spki_pin_and_certificate_verify_valid",
        "spki_rotation_pin_and_certificate_verify_valid"
    )
    if ($validReasons -notcontains $Decision.reason) {
        throw "$Name reason mismatch. Expected a verified pin reason but saw '$($Decision.reason)'."
    }
}

function Assert-TextOrder {
    param(
        [string]$Name,
        [string]$Serial,
        [string]$Earlier,
        [string]$Later
    )

    $earlierIndex = $Serial.IndexOf($Earlier, [StringComparison]::Ordinal)
    $laterIndex = $Serial.IndexOf($Later, [StringComparison]::Ordinal)
    if ($earlierIndex -lt 0) {
        throw "$Name missing earlier marker '$Earlier'."
    }
    if ($laterIndex -lt 0) {
        throw "$Name missing later marker '$Later'."
    }
    if ($earlierIndex -gt $laterIndex) {
        throw "$Name order mismatch. '$Earlier' appeared after '$Later'."
    }
}

function Assert-PositiveBindingMarkers {
    param(
        [string]$Serial
    )

    $envelope = Get-MarkerJson -Serial $Serial -Prefix "OPENAI_PROVIDER_REQUEST_ENVELOPE"
    $requestBinding = Get-MarkerJson -Serial $Serial -Prefix "OPENAI_PROVIDER_REQUEST_BINDING"
    $exportBinding = Get-MarkerJson -Serial $Serial -Prefix "OPENAI_PROVIDER_EXPORT_AUDIT_BINDING"
    $injectionGate = Get-MarkerJson -Serial $Serial -Prefix "OPENAI_PROVIDER_CONTEXT_INJECTION_GATE"

    Assert-Equal -Name "request body hash" -Actual $requestBinding.request_body_hash -Expected $envelope.request_body.body_sha256
    Assert-Equal -Name "request envelope hash" -Actual $requestBinding.request_envelope_hash -Expected $envelope.evidence.envelope_hash
    Assert-Equal -Name "export request body hash" -Actual $exportBinding.request_body_hash -Expected $requestBinding.request_body_hash
    Assert-Equal -Name "export request envelope hash" -Actual $exportBinding.request_envelope_hash -Expected $requestBinding.request_envelope_hash
    Assert-Equal -Name "export request binding hash" -Actual $exportBinding.request_binding_hash -Expected $requestBinding.request_binding_hash
    Assert-Equal -Name "export request binding event id" -Actual $exportBinding.request_binding_event_id -Expected $requestBinding.event_id
    Assert-Equal -Name "provider packet hash" -Actual $exportBinding.hashes.projected_packet_hash -Expected $requestBinding.hashes.projected_packet_hash
    Assert-Equal -Name "exported field list hash" -Actual $exportBinding.hashes.exported_field_list_hash -Expected $requestBinding.hashes.exported_field_list_hash
    Assert-Equal -Name "omitted field list hash" -Actual $exportBinding.hashes.omitted_field_list_hash -Expected $requestBinding.hashes.omitted_field_list_hash
    Assert-Equal -Name "redaction policy hash" -Actual $exportBinding.hashes.redaction_policy_hash -Expected $requestBinding.hashes.redaction_policy_hash
    Assert-Equal -Name "field classification hash" -Actual $exportBinding.hashes.field_classification_hash -Expected $requestBinding.hashes.field_classification_hash
    Assert-Equal -Name "token budget hash" -Actual $exportBinding.hashes.token_budget_hash -Expected $requestBinding.hashes.token_budget_hash
    Assert-Equal -Name "trust evidence hash" -Actual $exportBinding.trust_snapshot.provider_trust_evidence_hash -Expected $requestBinding.trust_snapshot.provider_trust_evidence_hash
    Assert-Equal -Name "trust verifier schema" -Actual $requestBinding.trust_snapshot.provider_trust_verifier.schema -Expected "raios.provider_trust_verifier_metadata.v0"
    Assert-Equal -Name "trust verifier id" -Actual $requestBinding.trust_snapshot.provider_trust_verifier.id -Expected "openai.pinned_tls13_p256_sha256.v0"
    Assert-Equal -Name "trust verifier host" -Actual $requestBinding.trust_snapshot.provider_trust_verifier.host -Expected "api.openai.com"
    Assert-Equal -Name "trust verifier hostname policy" -Actual $requestBinding.trust_snapshot.provider_trust_verifier.hostname_policy -Expected "exact_api.openai.com_required"
    Assert-Equal -Name "trust verifier chain policy" -Actual $requestBinding.trust_snapshot.provider_trust_verifier.chain_policy -Expected "pin_only_no_webpki_chain_validation"
    Assert-Equal -Name "trust verifier time policy" -Actual $requestBinding.trust_snapshot.provider_trust_verifier.time_policy -Expected "not_validated_stage0"
    Assert-Equal -Name "trust verifier pin policy" -Actual $requestBinding.trust_snapshot.provider_trust_verifier.pin_policy -Expected "configured_leaf_or_spki_sha256_required_optional_spki_rotation"
    Assert-Equal -Name "export trust verifier id" -Actual $exportBinding.trust_snapshot.provider_trust_verifier.id -Expected $requestBinding.trust_snapshot.provider_trust_verifier.id
    Assert-Equal -Name "request trust pin rotation policy" -Actual $requestBinding.trust_snapshot.provider_trust_pin_rotation_policy -Expected "single_active_pin"
    Assert-Equal -Name "export trust pin rotation policy" -Actual $exportBinding.trust_snapshot.provider_trust_pin_rotation_policy -Expected $requestBinding.trust_snapshot.provider_trust_pin_rotation_policy
    Assert-PositiveTrustDecision -Name "request trust verifier decision" -Decision $requestBinding.trust_snapshot.provider_trust_verifier_decision
    Assert-Equal -Name "export trust verifier decision schema" -Actual $exportBinding.trust_snapshot.provider_trust_verifier_decision.schema -Expected $requestBinding.trust_snapshot.provider_trust_verifier_decision.schema
    Assert-Equal -Name "export trust verifier decision stage" -Actual $exportBinding.trust_snapshot.provider_trust_verifier_decision.stage -Expected $requestBinding.trust_snapshot.provider_trust_verifier_decision.stage
    Assert-Equal -Name "export trust verifier decision outcome" -Actual $exportBinding.trust_snapshot.provider_trust_verifier_decision.outcome -Expected $requestBinding.trust_snapshot.provider_trust_verifier_decision.outcome
    Assert-Equal -Name "export trust verifier decision reason" -Actual $exportBinding.trust_snapshot.provider_trust_verifier_decision.reason -Expected $requestBinding.trust_snapshot.provider_trust_verifier_decision.reason
    Assert-Equal -Name "request binding current boot export gate" -Actual $requestBinding.satisfies_current_boot_export_gate -Expected $false
    Assert-Equal -Name "export binding current boot export gate" -Actual $exportBinding.satisfies_current_boot_export_gate -Expected $false
    Assert-Equal -Name "automatic context injection" -Actual $exportBinding.automatic_context_injection -Expected "disabled"
    Assert-Equal -Name "request binding body attachment" -Actual $requestBinding.context_attached_to_provider_body -Expected $false
    Assert-Equal -Name "export binding body attachment" -Actual $exportBinding.context_attached_to_provider_body -Expected $false
    Assert-Equal -Name "injection gate request body hash" -Actual $injectionGate.request_body_hash -Expected $requestBinding.request_body_hash
    Assert-Equal -Name "injection gate request envelope hash" -Actual $injectionGate.request_envelope_hash -Expected $requestBinding.request_envelope_hash
    Assert-Equal -Name "injection gate packet hash" -Actual $injectionGate.hashes.projected_packet_hash -Expected $requestBinding.hashes.projected_packet_hash
    Assert-Equal -Name "injection gate exported field list hash" -Actual $injectionGate.hashes.exported_field_list_hash -Expected $requestBinding.hashes.exported_field_list_hash
    Assert-Equal -Name "injection gate omitted field list hash" -Actual $injectionGate.hashes.omitted_field_list_hash -Expected $requestBinding.hashes.omitted_field_list_hash
    Assert-Equal -Name "injection gate redaction policy hash" -Actual $injectionGate.hashes.redaction_policy_hash -Expected $requestBinding.hashes.redaction_policy_hash
    Assert-Equal -Name "injection gate field classification hash" -Actual $injectionGate.hashes.field_classification_hash -Expected $requestBinding.hashes.field_classification_hash
    Assert-Equal -Name "injection gate token budget hash" -Actual $injectionGate.hashes.token_budget_hash -Expected $requestBinding.hashes.token_budget_hash
    Assert-Equal -Name "injection gate trust evidence hash" -Actual $injectionGate.provider_trust_evidence_hash -Expected $requestBinding.trust_snapshot.provider_trust_evidence_hash
    Assert-Equal -Name "injection gate trust verifier id" -Actual $injectionGate.provider_trust_verifier.id -Expected $requestBinding.trust_snapshot.provider_trust_verifier.id
    Assert-Equal -Name "injection gate trust verifier decision schema" -Actual $injectionGate.provider_trust_verifier_decision.schema -Expected $requestBinding.trust_snapshot.provider_trust_verifier_decision.schema
    Assert-Equal -Name "injection gate trust verifier decision stage" -Actual $injectionGate.provider_trust_verifier_decision.stage -Expected $requestBinding.trust_snapshot.provider_trust_verifier_decision.stage
    Assert-Equal -Name "injection gate trust verifier decision outcome" -Actual $injectionGate.provider_trust_verifier_decision.outcome -Expected $requestBinding.trust_snapshot.provider_trust_verifier_decision.outcome
    Assert-Equal -Name "injection gate trust verifier decision reason" -Actual $injectionGate.provider_trust_verifier_decision.reason -Expected $requestBinding.trust_snapshot.provider_trust_verifier_decision.reason
    Assert-Equal -Name "injection gate status" -Actual $injectionGate.status -Expected "blocked"
    Assert-Equal -Name "injection gate reason" -Actual $injectionGate.reason -Expected "automatic_context_injection_disabled"
    Assert-Equal -Name "injection gate final schema" -Actual $injectionGate.final_authorization_schema -Expected "raios.provider_context_injection_authorization.v0"
    Assert-Equal -Name "injection gate final authorization" -Actual $injectionGate.final_authorization -Expected "missing"
    Assert-Equal -Name "injection gate provider trust positive" -Actual $injectionGate.provider_trust_positive -Expected $true
    Assert-Equal -Name "injection gate current boot export gate" -Actual $injectionGate.satisfies_current_boot_export_gate -Expected $false
    Assert-Equal -Name "injection gate automatic context injection" -Actual $injectionGate.automatic_context_injection -Expected "disabled"
    Assert-Equal -Name "injection gate body attachment" -Actual $injectionGate.context_attached_to_provider_body -Expected $false
    Assert-Equal -Name "injection gate provider write" -Actual $injectionGate.provider_write -Expected "not_attempted"
    Assert-Equal -Name "injection gate can attach" -Actual $injectionGate.can_attach_context -Expected $false
}

function Invoke-PositiveBindingGateChecks {
    param(
        [int]$Port,
        [string]$SerialLog,
        [int]$TimeoutSeconds
    )

    Send-SerialText -Port $Port -TimeoutSeconds $TimeoutSeconds -Text "agent provider.context_gate provider_minimal`r"
    Wait-ForLogText -Path $SerialLog -Needle "RAIOS_AGENT_END provider.context_gate" -TimeoutSeconds $TimeoutSeconds
    Wait-ForLogText -Path $SerialLog -Needle '"schema": "raios.evidence_response.v1"' -TimeoutSeconds $TimeoutSeconds
    Wait-ForLogText -Path $SerialLog -Needle '"id": "provider_binding_consumption", "kind": "provider_binding_consumption", "status": "valid", "reason": "binding_pair_valid_for_gate_evaluation"' -TimeoutSeconds $TimeoutSeconds
    Wait-ForLogText -Path $SerialLog -Needle '"id": "provider_trust_binding", "kind": "provider_trust_binding", "status": "verified", "reason": "provider_trust_evidence_bound"' -TimeoutSeconds $TimeoutSeconds
    Wait-ForLogText -Path $SerialLog -Needle '"verifier_id": "openai.pinned_tls13_p256_sha256.v0", "chain_policy": "pin_only_no_webpki_chain_validation"' -TimeoutSeconds $TimeoutSeconds
    Wait-ForLogText -Path $SerialLog -Needle '"decision": {"outcome": "denied", "reason": "automatic_context_injection_disabled", "requested_capability": "cap.provider.context_export", "grants": [], "effects": []' -TimeoutSeconds $TimeoutSeconds

    Send-SerialText -Port $Port -TimeoutSeconds $TimeoutSeconds -Text "agent provider.context_export provider_minimal`r"
    Wait-ForLogText -Path $SerialLog -Needle '"provider_binding_consumption": "consumed_for_gate_evaluation"' -TimeoutSeconds $TimeoutSeconds
    Wait-ForLogText -Path $SerialLog -Needle '"schema": "raios.provider_context_binding_consumption.v0"' -TimeoutSeconds $TimeoutSeconds
    Wait-ForLogText -Path $SerialLog -Needle '"provider_binding_consumed_without_body_attachment"' -TimeoutSeconds $TimeoutSeconds
    Wait-ForLogText -Path $SerialLog -Needle '"automatic_context_injection": "disabled"' -TimeoutSeconds $TimeoutSeconds
    Wait-ForLogText -Path $SerialLog -Needle '"context_attached_to_provider_body": false' -TimeoutSeconds $TimeoutSeconds

    Send-SerialText -Port $Port -TimeoutSeconds $TimeoutSeconds -Text "agent provider.context_export provider_minimal`r"
    Wait-ForLogText -Path $SerialLog -Needle '"binding_validation_reason": "binding_already_consumed"' -TimeoutSeconds $TimeoutSeconds
}

if ($ExpectProjectWorkspaceAnswer) {
    $projectDiagnostics = [ordered]@{ image = $Image }
}

if (-not (Test-Path -LiteralPath $Image)) {
    if ($ExpectProjectWorkspaceAnswer) {
        Write-Host "B2.1b live harness failure: missing direct OpenAI image"
        Write-Host ($projectDiagnostics | ConvertTo-Json -Depth 12)
        Write-Host "serial log tail:"
        Write-Host (Get-RedactedSerialLogTail -Path $SerialLog)
    }
    throw "Missing direct OpenAI image: $Image. Package it with scripts\package-stage0.ps1 -UseTempEsp -EmbedOpenAiApiKeyFromEnv."
}

$projectStructuredStoreDisk = $null
$projectPersistDisk = $null

try {
    Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force
    Remove-Item -LiteralPath $SerialLog -Force -ErrorAction SilentlyContinue

    if ($ExpectProjectWorkspaceAnswer) {
        # ProjectWorkspace commits land in the disposable QEMU-only C1 structured
        # store (an AHCI disk at 00:1f.2); Normal boot posture needs a valid-a
        # BOOTCTL SEED_DATA disk. Without both, the guest boots
        # PersistenceUnavailable and rejects the answer with project_qemu_store_missing.
        $projectRunDir = Join-Path $env:TEMP "raios-openai-project-store"
        if (Test-Path -LiteralPath $projectRunDir) {
            Remove-Item -LiteralPath $projectRunDir -Recurse -Force
        }
        New-Item -ItemType Directory -Force -Path $projectRunDir | Out-Null

        $projectStructuredStoreDisk = Join-Path $projectRunDir "raios-structured-store-openai.img"
        $structuredStoreBuilder = Join-Path $RepoRoot "scripts\make-structured-store-image.py"
        $structuredStoreErr = Join-Path $projectRunDir "structured-store-builder.err.txt"
        $structuredStoreJson = & python $structuredStoreBuilder create $projectStructuredStoreDisk --size-mib 16 --json 2> $structuredStoreErr
        if ($LASTEXITCODE -ne 0) {
            throw "Structured-store fixture build failed: $(Get-Content -Raw -LiteralPath $structuredStoreErr -ErrorAction SilentlyContinue)"
        }
        $structuredStoreFixture = ($structuredStoreJson -join [Environment]::NewLine) | ConvertFrom-Json
        if (-not $structuredStoreFixture.valid -or -not $structuredStoreFixture.disposable_qemu_only -or
            $structuredStoreFixture.store_state -ne "empty_unformatted") {
            throw "Structured-store fixture identity or empty-state validation failed"
        }
        $projectStructuredStoreDisk = (Resolve-Path -LiteralPath $projectStructuredStoreDisk).Path

        $projectPersistDisk = Join-Path $projectRunDir "raios-persist-openai.img"
        $projectPersistErr = Join-Path $projectRunDir "persist-builder.err.txt"
        $null = & python (Join-Path $RepoRoot "scripts\make-gpt-persist-image.py") --self-check --seed-bootctl valid-a $projectPersistDisk 2> $projectPersistErr
        if ($LASTEXITCODE -ne 0) {
            throw "BOOTCTL persist fixture build failed: $(Get-Content -Raw -LiteralPath $projectPersistErr -ErrorAction SilentlyContinue)"
        }
        $projectPersistDisk = (Resolve-Path -LiteralPath $projectPersistDisk).Path

        $projectDiagnostics.structured_store_disk = $projectStructuredStoreDisk
        $projectDiagnostics.persist_disk = $projectPersistDisk
    }

    $runArgs = @{
        StopExisting  = $true
        Image         = $Image
        SerialMode    = "tcp"
        SerialTcpPort = $SerialTcpPort
        Headless      = $true
        BareMetalVm   = $true
        SerialLog     = $SerialLog
    }
    if ($ExpectProjectWorkspaceAnswer) {
        $runArgs.StructuredStoreDiskPath = $projectStructuredStoreDisk
        $runArgs.PersistDiskPath = $projectPersistDisk
    }
    & $RunScript @runArgs

    Wait-ForLogText -Path $SerialLog -Needle "Default provider loaded: OPENAI API key set" -TimeoutSeconds $TimeoutSeconds
    Wait-ForLogText -Path $SerialLog -Needle "status NETWORK: CONFIGURED" -TimeoutSeconds $TimeoutSeconds

    $safePrompt = $Prompt -replace '"', "'"
    if ($ExpectProjectWorkspaceAnswer) {
        $projectDescription = "a minimal Rust hello world crate with a Cargo.toml and a src/main.rs that prints Hello raiOS"
        $safePrompt = $projectDescription -replace '"', "'"
        Write-Host "openai-direct:b2-live-provider-ready passed: image present, provider loaded, network configured"
        Send-SerialText -Port $SerialTcpPort -TimeoutSeconds $TimeoutSeconds -Text "project.ask $safePrompt`r"
    }
    else {
        Send-SerialText -Port $SerialTcpPort -TimeoutSeconds $TimeoutSeconds -Text "provider`rask $safePrompt`r"
    }

    if ($ExpectProjectWorkspaceAnswer) {
        $projectStarted = Wait-ForLogLineMatch `
            -Path $SerialLog `
            -Pattern '^PROJECT SOURCE REQUEST (?<request>[0-9]+) STARTED$' `
            -FailurePattern '^PROJECT SOURCE REQUEST [0-9]+ TRACKING DENIED: .+$' `
            -TimeoutSeconds $TimeoutSeconds
        $projectRequestId = $projectStarted.Match.Groups['request'].Value
        $projectDiagnostics.request_started = $projectStarted.Line
        Write-Host "openai-direct:b2-build-request-sent passed: request=$projectRequestId"

        Wait-ForLogText -Path $SerialLog -Needle "OPENAI_DIRECT_REQ 1 api.openai.com /v1/responses" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_REQUEST_ENVELOPE {"schema":"raios.provider_request_envelope.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: TLS 1.3 established" -TimeoutSeconds $TimeoutSeconds
        $projectTrust = Wait-ForLogLineMatch `
            -Path $SerialLog `
            -Pattern '^openai: TLS provider trust verified: (?<kind>pinned_cert|pinned_spki)(?:\s|$)' `
            -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_REQUEST_BINDING {"schema":"raios.provider_request_binding.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_EXPORT_AUDIT_BINDING {"schema":"raios.provider_context_export_audit_binding.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_CONTEXT_INJECTION_GATE {"schema":"raios.provider_context_injection_gate.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: HTTPS request sent" -TimeoutSeconds $TimeoutSeconds

        $projectTransportSerial = Get-Content -Raw -LiteralPath $SerialLog
        if ($projectTransportSerial -like "*tls_certificate_verification_bypassed*") {
            throw "Live project-workspace smoke saw unverified TLS bypass output in $SerialLog"
        }
        $projectEnvelopeLines = @($projectTransportSerial -split '\r?\n' | Where-Object { $_ -like "OPENAI_PROVIDER_REQUEST_ENVELOPE *" })
        if ($projectEnvelopeLines.Count -eq 0) {
            throw "Live project-workspace smoke did not retain a provider request envelope in $SerialLog"
        }
        $projectDiagnostics.transport = [ordered]@{ trust = $projectTrust.Line }
        $projectDiagnostics.transport["envelope"] = Get-MarkerJson -Serial $projectTransportSerial -Prefix "OPENAI_PROVIDER_REQUEST_ENVELOPE"
        $projectDiagnostics.transport["request_binding"] = Get-MarkerJson -Serial $projectTransportSerial -Prefix "OPENAI_PROVIDER_REQUEST_BINDING"
        $projectDiagnostics.transport["export_binding"] = Get-MarkerJson -Serial $projectTransportSerial -Prefix "OPENAI_PROVIDER_EXPORT_AUDIT_BINDING"
        $projectDiagnostics.transport["injection_gate"] = Get-MarkerJson -Serial $projectTransportSerial -Prefix "OPENAI_PROVIDER_CONTEXT_INJECTION_GATE"
        foreach ($line in $projectEnvelopeLines) {
            if ($line.Contains($safePrompt)) {
                throw "Provider request envelope leaked raw project description text in $SerialLog"
            }
            if ($line -like "*Content-Length*") {
                throw "Provider request envelope leaked Content-Length in $SerialLog"
            }
            if ($line -like "*Authorization: Bearer*") {
                throw "Provider request envelope leaked Authorization header value in $SerialLog"
            }
            if ($line -notlike '*"provider_write":"not_attempted"*') {
                throw "Provider request envelope did not carry provider_write:not_attempted in $SerialLog"
            }
            if ($line -notlike '*"body_sha256":"sha256:*') {
                throw "Provider request envelope did not carry a request body hash in $SerialLog"
            }
            if ($line -notlike '*"envelope_hash":"sha256:*') {
                throw "Provider request envelope did not carry an envelope hash in $SerialLog"
            }
        }
        Assert-PositiveBindingMarkers -Serial $projectTransportSerial
        Assert-TextOrder -Name "injection gate before HTTPS write" -Serial $projectTransportSerial -Earlier "OPENAI_PROVIDER_CONTEXT_INJECTION_GATE" -Later "openai: HTTPS request sent"
        Write-Host "openai-direct:b2-live-answer-positive-pinned-provenance passed: $($projectTrust.Match.Groups['kind'].Value)"

        $projectOutcomeTimeoutSeconds = [Math]::Max($TimeoutSeconds, 300)
        $projectOutcomeLine = Wait-ForLogLineMatch `
            -Path $SerialLog `
            -Pattern "^PROJECT SOURCE (READY|REJECTED) request=$projectRequestId(?:\s|$)" `
            -TimeoutSeconds $projectOutcomeTimeoutSeconds
        $trimmedOutcomeLine = $projectOutcomeLine.Line.TrimEnd()
        if ($trimmedOutcomeLine -match '^PROJECT SOURCE READY request=([0-9]+) project=([0-9a-fA-F]+) revision=(sha256:[0-9a-fA-F]{64}) files=([0-9]+) inert$') {
            $projectOutcome = [ordered]@{
                outcome = "CONFORMING"
                accepted = $true
                request_id = $Matches[1]
                project_id = $Matches[2]
                revision_sha256 = $Matches[3]
                file_count = [int]$Matches[4]
                reason = $null
                line = $trimmedOutcomeLine
            }
            if ($projectOutcome.request_id -ne $projectRequestId -or $projectOutcome.file_count -lt 1) {
                throw "Conforming project outcome carried inconsistent request or file count: $trimmedOutcomeLine"
            }
            $projectOutcomeSummary = "CONFORMING-committed-inert revision=$($projectOutcome.revision_sha256) files=$($projectOutcome.file_count)"
        }
        elseif ($trimmedOutcomeLine -match '^PROJECT SOURCE REJECTED request=([0-9]+) project=([0-9a-fA-F]+|none) revision=(sha256:[0-9a-fA-F]{64}|none) files=([0-9]+) reason=(.+)$') {
            $projectOutcome = [ordered]@{
                outcome = "NONCONFORMING"
                accepted = $false
                request_id = $Matches[1]
                project_id = $Matches[2]
                revision_sha256 = $Matches[3]
                file_count = [int]$Matches[4]
                reason = $Matches[5]
                line = $trimmedOutcomeLine
            }
            if ($projectOutcome.request_id -ne $projectRequestId -or $projectOutcome.revision_sha256 -ne "none" -or $projectOutcome.file_count -ne 0) {
                throw "Rejected project outcome carried a revision or files: $trimmedOutcomeLine"
            }
            $projectOutcomeSummary = "NONCONFORMING-rejected reason=$($projectOutcome.reason)"
        }
        else {
            throw "Malformed project source outcome: $trimmedOutcomeLine"
        }
        $projectDiagnostics.outcome = $projectOutcome
        Write-Host "openai-direct:b2-answer-outcome passed: $projectOutcomeSummary"

        # Re-verify the context-export gate only after the provider request has
        # fully resolved, so the fresh agent queries never race an in-flight request.
        Invoke-PositiveBindingGateChecks -Port $SerialTcpPort -SerialLog $SerialLog -TimeoutSeconds $TimeoutSeconds

        Send-SerialText -Port $SerialTcpPort -TimeoutSeconds $TimeoutSeconds -Text "agent project.workspace`r"
        Wait-ForLogText -Path $SerialLog -Needle "RAIOS_AGENT_END project.workspace" -TimeoutSeconds $TimeoutSeconds
        $projectWorkspaceSerial = Get-Content -Raw -LiteralPath $SerialLog
        $projectWorkspaceResponse = Get-AgentResponseJson -Serial $projectWorkspaceSerial -Method "project.workspace"
        $projectWorkspace = $projectWorkspaceResponse.body.result
        $projectDiagnostics.workspace = $projectWorkspaceResponse

        foreach ($field in @("revision_action", "answer_origin", "revision_sha256", "file_count", "files")) {
            if ($null -eq $projectWorkspace.PSObject.Properties[$field]) {
                throw "Project workspace response omitted required field '$field'."
            }
        }
        Assert-Equal -Name "project workspace response version" -Actual $projectWorkspaceResponse.v -Expected "raios.agent.v0"
        Assert-Equal -Name "project workspace method" -Actual $projectWorkspace.method -Expected "project.workspace"
        Assert-Equal -Name "project workspace scope" -Actual $projectWorkspace.scope -Expected "current_boot"
        Assert-Equal -Name "project workspace classification" -Actual $projectWorkspace.classification -Expected "local_only"
        Assert-Equal -Name "project workspace provider trust" -Actual $projectWorkspace.provider_trust_positive -Expected $true
        Assert-Equal -Name "project workspace test infrastructure" -Actual $projectWorkspace.test_infrastructure -Expected $false
        Assert-Equal -Name "project workspace source authority" -Actual $projectWorkspace.source_authority -Expected "untrusted_agent_candidate"
        if ($projectOutcome.accepted) {
            Assert-Equal -Name "project workspace latest revision" -Actual $projectWorkspace.latest_revision_present -Expected $true
            Assert-Equal -Name "project workspace revision action" -Actual $projectWorkspace.revision_action -Expected "agent_answer"
            Assert-Equal -Name "project workspace answer origin" -Actual $projectWorkspace.answer_origin -Expected "live"
            Assert-Equal -Name "project workspace project id" -Actual $projectWorkspace.project_id -Expected $projectOutcome.project_id
            if ($projectWorkspace.revision_sha256 -cne $projectOutcome.revision_sha256) {
                throw "Project workspace revision hash did not exactly match the terminal marker."
            }
            Assert-Equal -Name "project workspace file count" -Actual ([int]$projectWorkspace.file_count) -Expected $projectOutcome.file_count
            if (@($projectWorkspace.files).Count -ne $projectOutcome.file_count) {
                throw "Project workspace files array did not match marker file count."
            }
        }
        else {
            Assert-Equal -Name "rejected project workspace latest revision" -Actual $projectWorkspace.latest_revision_present -Expected $false
            if ($projectWorkspace.revision_action -eq "agent_answer") {
                throw "Rejected project answer committed revision_action=agent_answer."
            }
            Assert-Equal -Name "rejected project workspace answer origin" -Actual $projectWorkspace.answer_origin -Expected $null
            Assert-Equal -Name "rejected project workspace revision hash" -Actual $projectWorkspace.revision_sha256 -Expected $null
            Assert-Equal -Name "rejected project workspace file count" -Actual ([int]$projectWorkspace.file_count) -Expected 0
            if (@($projectWorkspace.files).Count -ne 0) {
                throw "Rejected project workspace retained source files."
            }
        }
        Write-Host "openai-direct:b2-workspace-provenance-exact passed: outcome=$($projectOutcome.outcome)"

        $inertFields = @(
            "builder_attempted",
            "build_authorized",
            "candidate_intake_attempted",
            "load_attempted",
            "load_authorized",
            "execution_attempted",
            "execution_authorized",
            "install_attempted",
            "install_authorized",
            "promotion_attempted",
            "promotion_authorized",
            "wasm_instance_created",
            "w6_preview_created",
            "reclog_executable_record_written",
            "artstor_executable_record_written"
        )
        foreach ($field in $inertFields) {
            Assert-Equal -Name "project workspace inert field $field" -Actual $projectWorkspace.$field -Expected $false
        }
        Assert-Equal -Name "project workspace service inventory mutation" -Actual $projectWorkspace.service_inventory_mutation -Expected "none"
        $executableEffectPatterns = @(
            'RAIOS_AGENT_BEGIN service\.start(?:\s|$)',
            'RAIOS_AGENT_BEGIN (?:module\.submit_candidate|project\.build|project\.install|project\.promote|program\.install|program\.promote)(?:\s|$)',
            '"(?:candidate_intake_attempted|load_attempted|execution_attempted|install_attempted|promotion_attempted|service_start_attempted|service_started|wasm_instance_created|w6_preview_created|reclog_executable_record_written|artstor_executable_record_written)"\s*:\s*true',
            '"service_inventory_mutation"\s*:\s*"(?!none")'
        )
        foreach ($pattern in $executableEffectPatterns) {
            if ([regex]::IsMatch($projectWorkspaceSerial, $pattern)) {
                throw "Live project-workspace smoke saw executable-effect marker matching '$pattern' in $SerialLog"
            }
        }
        $projectOutcomeCount = [regex]::Matches($projectWorkspaceSerial, "(?m)^PROJECT SOURCE (?:READY|REJECTED) request=$projectRequestId(?:\s|$)").Count
        Assert-Equal -Name "project source terminal outcome count" -Actual $projectOutcomeCount -Expected 1
        if ($projectWorkspaceSerial -notlike "*automatic_context_injection_disabled*") {
            throw "Live project-workspace smoke did not retain automatic_context_injection_disabled in $SerialLog"
        }
        if (($projectWorkspaceSerial -like '*"context_attached_to_provider_body":true*') -or ($projectWorkspaceSerial -like '*"context_attached_to_provider_body": true*')) {
            throw "Live project-workspace smoke saw provider context attached to the request body in $SerialLog"
        }
        Write-Host "openai-direct:b2-inert-zero-executable-effect passed: source remains local-only and inert"
    }
    else {
        Wait-ForLogText -Path $SerialLog -Needle "PROVIDER: OPENAI    API KEY: SET" -TimeoutSeconds $TimeoutSeconds
    }
    if ($ExpectProjectWorkspaceAnswer) {
        # The live source-lane assertions completed above.
    }
    elseif ($ExpectProviderResponse) {
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
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_CONTEXT_INJECTION_GATE {"schema":"raios.provider_context_injection_gate.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: HTTPS request sent" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI HTTP" -TimeoutSeconds $TimeoutSeconds
        Invoke-PositiveBindingGateChecks -Port $SerialTcpPort -SerialLog $SerialLog -TimeoutSeconds $TimeoutSeconds
    }
    elseif ($ExpectSpkiPinnedTrust) {
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI_DIRECT_REQ 1 api.openai.com /v1/responses" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_REQUEST_ENVELOPE {"schema":"raios.provider_request_envelope.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI DIRECT REQUEST 1 STARTED" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: TLS 1.3 established" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: TLS provider trust verified: pinned_spki" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_REQUEST_BINDING {"schema":"raios.provider_request_binding.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_EXPORT_AUDIT_BINDING {"schema":"raios.provider_context_export_audit_binding.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_CONTEXT_INJECTION_GATE {"schema":"raios.provider_context_injection_gate.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: HTTPS request sent" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "OPENAI HTTP" -TimeoutSeconds $TimeoutSeconds
        Invoke-PositiveBindingGateChecks -Port $SerialTcpPort -SerialLog $SerialLog -TimeoutSeconds $TimeoutSeconds
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
    if ((-not $ExpectProviderResponse) -and (-not $ExpectPinnedTrust) -and (-not $ExpectSpkiPinnedTrust) -and (-not $ExpectProjectWorkspaceAnswer) -and (-not $ExpectPinMismatch) -and ($serial -like "*OPENAI_DIRECT_REQ*")) {
        throw "Trust-gate smoke saw an OpenAI request start before provider trust was verified in $SerialLog"
    }
    if ((-not $ExpectProviderResponse) -and (-not $ExpectPinnedTrust) -and (-not $ExpectSpkiPinnedTrust) -and (-not $ExpectProjectWorkspaceAnswer) -and (-not $ExpectPinMismatch) -and ($serial -like "*raios.provider_request_envelope.v0*")) {
        throw "Trust-gate smoke saw a provider request envelope before provider trust allowed a request in $SerialLog"
    }
    if (($ExpectProviderResponse -or $ExpectPinMismatch) -and ($serial -like "*raios.provider_request_binding.v0*")) {
        throw "Direct smoke saw a positive provider request binding without positive provider trust in $SerialLog"
    }
    if (($ExpectProviderResponse -or $ExpectPinMismatch) -and ($serial -like "*raios.provider_context_export_audit_binding.v0*")) {
        throw "Direct smoke saw a positive provider export audit binding without positive provider trust in $SerialLog"
    }
    if (($ExpectProviderResponse -or $ExpectPinMismatch) -and ($serial -like "*raios.provider_context_injection_gate.v0*")) {
        throw "Direct smoke saw a provider context injection gate marker without positive provider trust in $SerialLog"
    }
    if (($ExpectPinnedTrust -or $ExpectSpkiPinnedTrust) -and ($serial -notlike "*raios.provider_request_binding.v0*")) {
        throw "Pinned-trust smoke did not see a positive provider request binding in $SerialLog"
    }
    if (($ExpectPinnedTrust -or $ExpectSpkiPinnedTrust) -and ($serial -notlike "*raios.provider_context_export_audit_binding.v0*")) {
        throw "Pinned-trust smoke did not see a positive provider export audit binding in $SerialLog"
    }
    if ($ExpectPinnedTrust -or $ExpectSpkiPinnedTrust) {
        Assert-PositiveBindingMarkers -Serial $serial
        Assert-TextOrder -Name "injection gate before HTTPS write" -Serial $serial -Earlier "OPENAI_PROVIDER_CONTEXT_INJECTION_GATE" -Later "openai: HTTPS request sent"
    }
    if (($serial -like '*"context_attached_to_provider_body":true*') -or ($serial -like '*"context_attached_to_provider_body": true*')) {
        throw "Direct smoke saw provider context body attachment before final injection authorization in $SerialLog"
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
    if ($ExpectProjectWorkspaceAnswer -and ($serial -like "*tls_certificate_verification_bypassed*")) {
        throw "Live project-workspace smoke saw unverified TLS bypass output in $SerialLog"
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

    if ($ExpectProjectWorkspaceAnswer) {
        Write-Host "openai direct project-workspace live smoke passed"
    }
    elseif ($ExpectProviderResponse) {
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
catch {
    if ($ExpectProjectWorkspaceAnswer) {
        Write-Host "B2.1b live harness failure: $($_.Exception.Message)"
        $projectDiagnosticJson = $projectDiagnostics | ConvertTo-Json -Depth 12
        $projectDiagnosticJson = [regex]::Replace($projectDiagnosticJson, '(?i)(Authorization:\s*Bearer\s+)[^"\r\n]*', '${1}<redacted>')
        Write-Host $projectDiagnosticJson
        Write-Host "serial log tail:"
        Write-Host (Get-RedactedSerialLogTail -Path $SerialLog)
    }
    throw
}
finally {
    Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force
}
