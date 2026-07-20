param(
    [int]$SerialTcpPort = 4555,
    [string]$Prompt = "direct provider smoke",
    [string]$Image = "$PSScriptRoot\..\release\raios-stage0-local-openai.img",
    [int]$TimeoutSeconds = 90,
    [switch]$ExpectProviderResponse,
    [switch]$ExpectPinnedTrust,
    [switch]$ExpectSpkiPinnedTrust,
    [switch]$ExpectPinMismatch,
    [switch]$ExpectProjectWorkspaceAnswer,
    [switch]$ExpectScopedFeedbackExport
)

$ErrorActionPreference = "Stop"

$modeCount = 0
foreach ($mode in @($ExpectProviderResponse, $ExpectPinnedTrust, $ExpectSpkiPinnedTrust, $ExpectProjectWorkspaceAnswer, $ExpectScopedFeedbackExport, $ExpectPinMismatch)) {
    if ($mode) {
        $modeCount += 1
    }
}
if ($modeCount -gt 1) {
    throw "Use only one of -ExpectProviderResponse, -ExpectPinnedTrust, -ExpectSpkiPinnedTrust, -ExpectProjectWorkspaceAnswer, -ExpectScopedFeedbackExport, or -ExpectPinMismatch."
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

function Invoke-AgentCommandJson {
    param(
        [int]$Port,
        [string]$SerialLog,
        [string]$Command,
        [string]$Method,
        [int]$TimeoutSeconds
    )

    $end = "RAIOS_AGENT_END $Method"
    $before = if (Test-Path -LiteralPath $SerialLog) { Get-Content -Raw -LiteralPath $SerialLog } else { "" }
    $beforeCount = [regex]::Matches($before, [regex]::Escape($end)).Count
    Send-SerialText -Port $Port -TimeoutSeconds $TimeoutSeconds -Text "agent $Command`r"

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    do {
        if (Test-Path -LiteralPath $SerialLog) {
            $serial = Get-Content -Raw -LiteralPath $SerialLog -ErrorAction SilentlyContinue
            if ([regex]::Matches($serial, [regex]::Escape($end)).Count -gt $beforeCount) {
                return Get-AgentResponseJson -Serial $serial -Method $Method
            }
        }
        Start-Sleep -Milliseconds 250
    } while ([DateTime]::UtcNow -lt $deadline)

    throw "Timed out waiting for a new agent response for method '$Method' in $SerialLog"
}

function ConvertFrom-ProjectOutcomeLine {
    param(
        [string]$Line,
        [string]$ExpectedRequestId,
        [string]$RetainedRevisionSha256 = "none",
        [int]$RetainedFileCount = 0
    )

    $trimmed = $Line.TrimEnd()
    if ($trimmed -match '^PROJECT SOURCE READY request=([0-9]+) project=([0-9a-fA-F]+) revision=(sha256:[0-9a-fA-F]{64}) files=([0-9]+) inert$') {
        $outcome = [ordered]@{
            outcome = "CONFORMING"
            accepted = $true
            request_id = $Matches[1]
            project_id = $Matches[2]
            revision_sha256 = $Matches[3]
            file_count = [int]$Matches[4]
            reason = $null
            line = $trimmed
        }
        if ($outcome.request_id -ne $ExpectedRequestId -or $outcome.file_count -lt 1) {
            throw "Conforming project outcome carried inconsistent request or file count: $trimmed"
        }
        return [pscustomobject]@{
            Outcome = $outcome
            Summary = "CONFORMING-committed-inert revision=$($outcome.revision_sha256) files=$($outcome.file_count)"
        }
    }
    if ($trimmed -match '^PROJECT SOURCE REJECTED request=([0-9]+) project=([0-9a-fA-F]+|none) revision=(sha256:[0-9a-fA-F]{64}|none) files=([0-9]+) reason=(.+)$') {
        $outcome = [ordered]@{
            outcome = "NONCONFORMING"
            accepted = $false
            request_id = $Matches[1]
            project_id = $Matches[2]
            revision_sha256 = $Matches[3]
            file_count = [int]$Matches[4]
            reason = $Matches[5]
            line = $trimmed
        }
        if ($outcome.request_id -ne $ExpectedRequestId -or
            $outcome.revision_sha256 -cne $RetainedRevisionSha256 -or
            $outcome.file_count -ne $RetainedFileCount) {
            if ($RetainedRevisionSha256 -eq "none" -and $RetainedFileCount -eq 0) {
                throw "Rejected project outcome carried a revision or files: $trimmed"
            }
            throw "Rejected project outcome carried inconsistent retained revision facts: $trimmed"
        }
        return [pscustomobject]@{
            Outcome = $outcome
            Summary = "NONCONFORMING-rejected reason=$($outcome.reason)"
        }
    }
    throw "Malformed project source outcome: $trimmed"
}

function Get-TextSha256 {
    param([string]$Text)

    $sha256 = [System.Security.Cryptography.SHA256]::Create()
    try {
        $hash = $sha256.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($Text))
        return "sha256:" + (($hash | ForEach-Object { $_.ToString("x2") }) -join "")
    }
    finally {
        $sha256.Dispose()
    }
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

$projectDiskMode = $ExpectProjectWorkspaceAnswer -or $ExpectScopedFeedbackExport
$projectFailureLabel = if ($ExpectScopedFeedbackExport) { "B2.2 live harness" } else { "B2.1b live harness" }

if ($projectDiskMode) {
    $projectDiagnostics = [ordered]@{ image = $Image }
}

if (-not (Test-Path -LiteralPath $Image)) {
    if ($projectDiskMode) {
        Write-Host "$projectFailureLabel failure: missing direct OpenAI image"
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

    if ($projectDiskMode) {
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
    if ($projectDiskMode) {
        $runArgs.StructuredStoreDiskPath = $projectStructuredStoreDisk
        $runArgs.PersistDiskPath = $projectPersistDisk
    }
    & $RunScript @runArgs

    Wait-ForLogText -Path $SerialLog -Needle "Default provider loaded: OPENAI API key set" -TimeoutSeconds $TimeoutSeconds
    Wait-ForLogText -Path $SerialLog -Needle "status NETWORK: CONFIGURED" -TimeoutSeconds $TimeoutSeconds

    $safePrompt = $Prompt -replace '"', "'"
    if ($projectDiskMode) {
        $projectDescription = "a minimal Rust hello world crate with a Cargo.toml and a src/main.rs that prints Hello raiOS"
        $safePrompt = $projectDescription -replace '"', "'"
        if ($ExpectScopedFeedbackExport) {
            Write-Host "openai-direct:b2-2-live-ready passed: image present, provider loaded, network configured"
        }
        else {
            Write-Host "openai-direct:b2-live-provider-ready passed: image present, provider loaded, network configured"
        }
        Send-SerialText -Port $SerialTcpPort -TimeoutSeconds $TimeoutSeconds -Text "project.ask $safePrompt`r"
    }
    else {
        Send-SerialText -Port $SerialTcpPort -TimeoutSeconds $TimeoutSeconds -Text "provider`rask $safePrompt`r"
    }

    if ($ExpectScopedFeedbackExport) {
        $liveOutcomeTimeoutSeconds = [Math]::Max($TimeoutSeconds, 300)
        $projectStarted = Wait-ForLogLineMatch `
            -Path $SerialLog `
            -Pattern '^PROJECT SOURCE REQUEST (?<request>[0-9]+) STARTED$' `
            -FailurePattern '^PROJECT SOURCE REQUEST [0-9]+ TRACKING DENIED: .+$' `
            -TimeoutSeconds $TimeoutSeconds
        $projectRequestId = $projectStarted.Match.Groups['request'].Value
        $projectDiagnostics.request_started = $projectStarted.Line

        Wait-ForLogText -Path $SerialLog -Needle "OPENAI_DIRECT_REQ $projectRequestId api.openai.com /v1/responses" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_REQUEST_ENVELOPE {"schema":"raios.provider_request_envelope.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: TLS 1.3 established" -TimeoutSeconds $TimeoutSeconds
        $initialTrust = Wait-ForLogLineMatch `
            -Path $SerialLog `
            -Pattern '^openai: TLS provider trust verified: (?<kind>pinned_cert|pinned_spki)(?:\s|$)' `
            -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_REQUEST_BINDING {"schema":"raios.provider_request_binding.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_EXPORT_AUDIT_BINDING {"schema":"raios.provider_context_export_audit_binding.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle 'OPENAI_PROVIDER_CONTEXT_INJECTION_GATE {"schema":"raios.provider_context_injection_gate.v0"' -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText -Path $SerialLog -Needle "openai: HTTPS request sent" -TimeoutSeconds $TimeoutSeconds

        $initialTransportSerial = Get-Content -Raw -LiteralPath $SerialLog
        if ($initialTransportSerial -like "*tls_certificate_verification_bypassed*") {
            throw "B2.2 live smoke saw unverified TLS bypass output during revision 1"
        }
        Assert-PositiveBindingMarkers -Serial $initialTransportSerial
        $projectDiagnostics.initial_transport = [ordered]@{
            trust = $initialTrust.Line
            envelope = Get-MarkerJson -Serial $initialTransportSerial -Prefix "OPENAI_PROVIDER_REQUEST_ENVELOPE"
            request_binding = Get-MarkerJson -Serial $initialTransportSerial -Prefix "OPENAI_PROVIDER_REQUEST_BINDING"
            export_binding = Get-MarkerJson -Serial $initialTransportSerial -Prefix "OPENAI_PROVIDER_EXPORT_AUDIT_BINDING"
            injection_gate = Get-MarkerJson -Serial $initialTransportSerial -Prefix "OPENAI_PROVIDER_CONTEXT_INJECTION_GATE"
        }

        $revisionOneOutcomeLine = Wait-ForLogLineMatch `
            -Path $SerialLog `
            -Pattern "^PROJECT SOURCE (READY|REJECTED) request=$projectRequestId(?:\s|$)" `
            -TimeoutSeconds $liveOutcomeTimeoutSeconds
        $parsedRevisionOneOutcome = ConvertFrom-ProjectOutcomeLine -Line $revisionOneOutcomeLine.Line -ExpectedRequestId $projectRequestId
        $revisionOneOutcome = $parsedRevisionOneOutcome.Outcome
        $projectDiagnostics.revision_one_outcome = $revisionOneOutcome
        if (-not $revisionOneOutcome.accepted -or $revisionOneOutcome.file_count -ne 2) {
            throw "B2.2 requires revision 1 to be the exact two-file live source revision: $($revisionOneOutcome.line)"
        }
        $revisionOneSha256 = $revisionOneOutcome.revision_sha256
        $projectId = $revisionOneOutcome.project_id
        if ($projectId -notmatch '^[0-9a-f]{32}$') {
            throw "Revision 1 project id was not the merged 16-byte hex shape."
        }

        $revisionOneWorkspaceResponse = Invoke-AgentCommandJson `
            -Port $SerialTcpPort `
            -SerialLog $SerialLog `
            -Command "project.workspace" `
            -Method "project.workspace" `
            -TimeoutSeconds $TimeoutSeconds
        $revisionOneWorkspace = $revisionOneWorkspaceResponse.body.result
        $projectDiagnostics.revision_one_workspace = $revisionOneWorkspaceResponse
        Assert-Equal -Name "revision 1 workspace version" -Actual $revisionOneWorkspaceResponse.v -Expected "raios.agent.v0"
        Assert-Equal -Name "revision 1 workspace method" -Actual $revisionOneWorkspace.method -Expected "project.workspace"
        Assert-Equal -Name "revision 1 workspace classification" -Actual $revisionOneWorkspace.classification -Expected "local_only"
        Assert-Equal -Name "revision 1 workspace latest revision" -Actual $revisionOneWorkspace.latest_revision_present -Expected $true
        Assert-Equal -Name "revision 1 workspace revision action" -Actual $revisionOneWorkspace.revision_action -Expected "agent_answer"
        Assert-Equal -Name "revision 1 workspace answer origin" -Actual $revisionOneWorkspace.answer_origin -Expected "live"
        Assert-Equal -Name "revision 1 workspace provider trust" -Actual $revisionOneWorkspace.provider_trust_positive -Expected $true
        Assert-Equal -Name "revision 1 workspace source authority" -Actual $revisionOneWorkspace.source_authority -Expected "untrusted_agent_candidate"
        Assert-Equal -Name "revision 1 workspace project id" -Actual $revisionOneWorkspace.project_id -Expected $projectId
        Assert-Equal -Name "revision 1 workspace revision" -Actual $revisionOneWorkspace.revision_sha256 -Expected $revisionOneSha256
        Assert-Equal -Name "revision 1 workspace file count" -Actual ([int]$revisionOneWorkspace.file_count) -Expected 2
        $revisionOnePaths = @($revisionOneWorkspace.files | ForEach-Object { $_.path })
        if ($revisionOnePaths.Count -ne 2 -or $revisionOnePaths[0] -cne "Cargo.toml" -or $revisionOnePaths[1] -cne "src/main.rs") {
            throw "Revision 1 did not contain exactly Cargo.toml and src/main.rs."
        }

        $verifyResponse = Invoke-AgentCommandJson `
            -Port $SerialTcpPort `
            -SerialLog $SerialLog `
            -Command "project.verify_revision" `
            -Method "project.verify_revision" `
            -TimeoutSeconds $TimeoutSeconds
        $verify = $verifyResponse.body.result
        $projectDiagnostics.verify_revision = $verifyResponse
        Assert-Equal -Name "revision verifier version" -Actual $verifyResponse.v -Expected "raios.agent.v0"
        Assert-Equal -Name "revision verifier method" -Actual $verify.method -Expected "project.verify_revision"
        Assert-Equal -Name "revision verifier status" -Actual $verify.status -Expected "recorded"
        Assert-Equal -Name "revision verifier accepted" -Actual $verify.accepted -Expected $true
        Assert-Equal -Name "revision verifier check" -Actual $verify.check_id -Expected "project.source_preflight.v1"
        Assert-Equal -Name "revision verifier outcome" -Actual $verify.outcome -Expected "failed"
        Assert-Equal -Name "revision verifier passed" -Actual ($verify.outcome -eq "passed") -Expected $false
        Assert-Equal -Name "revision verifier reason" -Actual $verify.reason -Expected "build_cargo_lock_missing"
        Assert-Equal -Name "revision verifier revision" -Actual $verify.revision_sha256 -Expected $revisionOneSha256
        Assert-Equal -Name "revision verifier tree" -Actual $verify.tree_sha256 -Expected $revisionOneWorkspace.tree_sha256
        Assert-Equal -Name "revision verifier system computed" -Actual $verify.system_computed -Expected $true
        Assert-Equal -Name "revision verifier provider supplied" -Actual $verify.provider_supplied -Expected $false
        if ($null -ne $verify.PSObject.Properties['passed']) {
            throw "Merged project.verify_revision unexpectedly emitted a provider-shaped passed field."
        }
        Write-Host "openai-direct:b2-2-revision1-preflight-failed passed: build_cargo_lock_missing"

        $feedbackResponse = Invoke-AgentCommandJson `
            -Port $SerialTcpPort `
            -SerialLog $SerialLog `
            -Command "project.feedback_packet" `
            -Method "project.feedback_packet" `
            -TimeoutSeconds $TimeoutSeconds
        $feedback = $feedbackResponse.body.result
        $packet = $feedback.feedback_packet
        $projectDiagnostics.feedback_packet = $feedbackResponse
        Assert-Equal -Name "feedback packet version" -Actual $feedbackResponse.v -Expected "raios.agent.v0"
        Assert-Equal -Name "feedback packet method" -Actual $feedback.method -Expected "project.feedback_packet"
        Assert-Equal -Name "feedback packet classification" -Actual $feedback.classification -Expected "local_only"
        Assert-Equal -Name "feedback packet status" -Actual $feedback.status -Expected "ready"
        Assert-Equal -Name "feedback packet accepted" -Actual $feedback.accepted -Expected $true
        Assert-Equal -Name "feedback packet field count" -Actual ([int]$feedback.packet_field_count) -Expected 4
        Assert-Equal -Name "feedback packet property count" -Actual (@($packet.PSObject.Properties).Count) -Expected 4
        Assert-Equal -Name "feedback packet check" -Actual $packet.check_id -Expected "project.source_preflight.v1"
        Assert-Equal -Name "feedback packet revision" -Actual $packet.revision_sha256 -Expected $revisionOneSha256
        Assert-Equal -Name "feedback packet tree" -Actual $packet.tree_sha256 -Expected $verify.tree_sha256
        Assert-Equal -Name "feedback packet reason" -Actual $packet.reason -Expected "build_cargo_lock_missing"
        Assert-Equal -Name "feedback packet system computed" -Actual $feedback.system_computed -Expected $true
        Assert-Equal -Name "feedback packet provider supplied" -Actual $feedback.provider_supplied -Expected $false
        foreach ($field in @("source_bytes_included", "secret_bytes_included", "log_bytes_included", "unclassified_text_included")) {
            Assert-Equal -Name "feedback packet $field" -Actual $feedback.$field -Expected $false
        }
        Write-Host "openai-direct:b2-2-feedback-packet-4-public-fields passed: local_only at rest, exact four-field packet"

        $transportBeforeSubmit = Get-Content -Raw -LiteralPath $SerialLog
        $tlsEstablishedCountBeforeSubmit = [regex]::Matches($transportBeforeSubmit, '(?m)^openai: TLS 1\.3 established\r*$').Count
        $positiveTrustCountBeforeSubmit = [regex]::Matches($transportBeforeSubmit, '(?m)^openai: TLS provider trust verified: (?:pinned_cert|pinned_spki)(?:\s|$)').Count
        $submitResponse = Invoke-AgentCommandJson `
            -Port $SerialTcpPort `
            -SerialLog $SerialLog `
            -Command "project.feedback_submit" `
            -Method "project.feedback_submit" `
            -TimeoutSeconds $TimeoutSeconds
        $submit = $submitResponse.body.result
        $projectDiagnostics.feedback_submit = $submitResponse
        Assert-Equal -Name "feedback submit version" -Actual $submitResponse.v -Expected "raios.agent.v0"
        Assert-Equal -Name "feedback submit method" -Actual $submit.method -Expected "project.feedback_submit"
        Assert-Equal -Name "feedback submit status" -Actual $submit.status -Expected "started"
        Assert-Equal -Name "feedback submit accepted" -Actual $submit.accepted -Expected $true
        Assert-Equal -Name "feedback submit reason" -Actual $submit.reason -Expected "provider_feedback_request_started"
        Assert-Equal -Name "feedback submit export method" -Actual $submit.export_method -Expected "provider.context_export"
        Assert-Equal -Name "feedback submit profile" -Actual $submit.profile -Expected "provider_minimal"
        Assert-Equal -Name "feedback submit field count" -Actual ([int]$submit.packet_field_count) -Expected 4
        Assert-Equal -Name "feedback submit context attachment" -Actual $submit.context_attached_to_provider_body -Expected $false
        Assert-Equal -Name "feedback submit provider write" -Actual $submit.provider_write -Expected "not_attempted"
        $feedbackRequestId = [string]$submit.request_id
        if (-not $feedbackRequestId -or $feedbackRequestId -eq $projectRequestId) {
            throw "Feedback submit did not allocate a new request id."
        }
        Write-Host "openai-direct:b2-2-feedback-submit-started passed: request=$feedbackRequestId"

        Wait-ForLogText -Path $SerialLog -Needle "OPENAI_DIRECT_REQ $feedbackRequestId api.openai.com /v1/responses" -TimeoutSeconds $TimeoutSeconds
        Wait-ForLogText `
            -Path $SerialLog `
            -Needle "OPENAI_SCOPED_PROJECT_FEEDBACK_EXPORT {`"status`":`"authorized`",`"reason`":`"authorized_provider_export_public_only_audited`",`"request_id`":$feedbackRequestId" `
            -TimeoutSeconds $liveOutcomeTimeoutSeconds
        Wait-ForLogText `
            -Path $SerialLog `
            -Needle "OPENAI_SCOPED_PROJECT_FEEDBACK_SENT {`"request_id`":$feedbackRequestId" `
            -TimeoutSeconds $liveOutcomeTimeoutSeconds

        $scopedTransportSerial = Get-Content -Raw -LiteralPath $SerialLog
        $scopedTrust = Wait-ForLogLineMatch `
            -Path $SerialLog `
            -Pattern '^openai: TLS provider trust verified: (?<kind>pinned_cert|pinned_spki)(?:\s|$)' `
            -TimeoutSeconds $TimeoutSeconds
        $scopedEnvelope = Get-MarkerJson -Serial $scopedTransportSerial -Prefix "OPENAI_PROVIDER_REQUEST_ENVELOPE"
        $scopedRequestBinding = Get-MarkerJson -Serial $scopedTransportSerial -Prefix "OPENAI_PROVIDER_REQUEST_BINDING"
        $scopedExportBinding = Get-MarkerJson -Serial $scopedTransportSerial -Prefix "OPENAI_PROVIDER_EXPORT_AUDIT_BINDING"
        $scopedAuthorization = Get-MarkerJson -Serial $scopedTransportSerial -Prefix "OPENAI_SCOPED_PROJECT_FEEDBACK_EXPORT"
        $scopedSent = Get-MarkerJson -Serial $scopedTransportSerial -Prefix "OPENAI_SCOPED_PROJECT_FEEDBACK_SENT"
        $projectDiagnostics.scoped_transport = [ordered]@{
            trust = $scopedTrust.Line
            envelope = $scopedEnvelope
            request_binding = $scopedRequestBinding
            export_binding = $scopedExportBinding
            authorization = $scopedAuthorization
            sent = $scopedSent
        }

        $expectedEnvelopeId = "provider_request_envelope.current_boot.{0:D8}" -f [int]$feedbackRequestId
        Assert-Equal -Name "scoped envelope id" -Actual $scopedEnvelope.id -Expected $expectedEnvelopeId
        Assert-Equal -Name "scoped envelope source method" -Actual $scopedEnvelope.source.method -Expected "project.feedback_export"
        Assert-Equal -Name "scoped envelope provider write" -Actual $scopedEnvelope.provider_write -Expected "not_attempted"
        Assert-Equal -Name "scoped envelope instruction marker" -Actual $scopedEnvelope.request_body.instructions -Expected "fixed_scoped_project_feedback"
        Assert-Equal -Name "scoped envelope input marker" -Actual $scopedEnvelope.request_body.input -Expected "provider_minimal_packet_redacted"
        Assert-Equal -Name "scoped envelope body attachment" -Actual $scopedEnvelope.request_body.context_attached_to_provider_body -Expected $true
        Assert-Equal -Name "scoped envelope max output" -Actual ([int]$scopedEnvelope.request_body.max_output_tokens) -Expected 8192
        Assert-Equal -Name "scoped envelope store" -Actual $scopedEnvelope.request_body.store -Expected $false
        Assert-Equal -Name "scoped envelope context attachment" -Actual $scopedEnvelope.provider_minimal_context.attached -Expected $true
        Assert-Equal -Name "scoped envelope binding status" -Actual $scopedEnvelope.provider_minimal_context.binding_status -Expected "bound"
        Assert-Equal -Name "scoped envelope provider trust" -Actual $scopedEnvelope.trust_snapshot.provider_trust_positive -Expected $true
        Assert-Equal -Name "scoped envelope development bypass" -Actual $scopedEnvelope.trust_snapshot.development_tls_bypass -Expected $false
        Assert-Equal -Name "scoped envelope authorization redaction" -Actual $scopedEnvelope.secret_state.authorization_header -Expected "redacted"

        Assert-Equal -Name "scoped request binding request" -Actual ([string]$scopedRequestBinding.request_id) -Expected $feedbackRequestId
        Assert-Equal -Name "scoped request binding export gate" -Actual $scopedRequestBinding.satisfies_current_boot_export_gate -Expected $true
        Assert-Equal -Name "scoped request binding attachment" -Actual $scopedRequestBinding.context_attached_to_provider_body -Expected $true
        Assert-Equal -Name "scoped request binding bypass" -Actual $scopedRequestBinding.trust_snapshot.development_tls_bypass -Expected $false
        Assert-PositiveTrustDecision -Name "scoped request trust verifier decision" -Decision $scopedRequestBinding.trust_snapshot.provider_trust_verifier_decision
        Assert-Equal -Name "scoped export binding request" -Actual ([string]$scopedExportBinding.request_id) -Expected $feedbackRequestId
        Assert-Equal -Name "scoped export binding status" -Actual $scopedExportBinding.status -Expected "authorized_for_single_provider_request"
        Assert-Equal -Name "scoped export binding positive authorization" -Actual $scopedExportBinding.positive_export_authorization -Expected $true
        Assert-Equal -Name "scoped export binding export gate" -Actual $scopedExportBinding.satisfies_current_boot_export_gate -Expected $true
        Assert-Equal -Name "scoped export binding injection mode" -Actual $scopedExportBinding.automatic_context_injection -Expected "deliberate_scoped_feedback_only"
        Assert-Equal -Name "scoped export binding attachment" -Actual $scopedExportBinding.context_attached_to_provider_body -Expected $true
        Assert-Equal -Name "scoped export binding bypass" -Actual $scopedExportBinding.trust_snapshot.development_tls_bypass -Expected $false
        Assert-Equal -Name "scoped packet canonicalization" -Actual $scopedExportBinding.hashes.packet_canonicalization -Expected "raios.scoped_project_feedback.packet.canonical_json.v1"

        Assert-Equal -Name "scoped authorization status" -Actual $scopedAuthorization.status -Expected "authorized"
        Assert-Equal -Name "scoped gate reason" -Actual $scopedAuthorization.reason -Expected "authorized_provider_export_public_only_audited"
        Assert-Equal -Name "scoped authorization request" -Actual ([string]$scopedAuthorization.request_id) -Expected $feedbackRequestId
        Assert-Equal -Name "scoped packet record count" -Actual ([int]$scopedAuthorization.packet_record_count) -Expected 4
        Assert-Equal -Name "scoped packet public posture" -Actual $scopedAuthorization.packet_all_records_public -Expected $true
        Assert-Equal -Name "scoped budget" -Actual ([int]$scopedAuthorization.budget_tokens) -Expected 1000
        if ([int]$scopedAuthorization.packet_estimated_tokens -lt 1 -or
            [int]$scopedAuthorization.packet_estimated_tokens -gt [int]$scopedAuthorization.budget_tokens) {
            throw "Scoped packet token estimate exceeded its provider_minimal budget."
        }
        Assert-Equal -Name "scoped durable audit reason" -Actual $scopedAuthorization.durable_audit_reason -Expected "authorized_memory_record_append_readback_reparse_verified"
        if ($scopedAuthorization.durable_audit_payload_sha256 -notmatch '^sha256:[0-9a-f]{64}$' -or $null -eq $scopedAuthorization.durable_audit_seq) {
            throw "Scoped export omitted durable raios.memory_record.v0 export_audit append evidence."
        }
        Assert-Equal -Name "scoped authorization attachment" -Actual $scopedAuthorization.context_attached_to_provider_body -Expected $true
        Assert-Equal -Name "scoped authorization prewrite" -Actual $scopedAuthorization.provider_write -Expected "not_attempted"
        Assert-Equal -Name "scoped authorization consumed" -Actual $scopedAuthorization.authorization_consumed -Expected $true
        Assert-Equal -Name "scoped sent request" -Actual ([string]$scopedSent.request_id) -Expected $feedbackRequestId
        Assert-Equal -Name "scoped sent attachment" -Actual $scopedSent.context_attached_to_provider_body -Expected $true
        Assert-Equal -Name "scoped sent provider write" -Actual $scopedSent.provider_write -Expected "performed"
        Assert-Equal -Name "scoped sent authorization" -Actual $scopedSent.authorization_consumed -Expected $true

        Assert-Equal -Name "scoped body hash binding" -Actual $scopedRequestBinding.request_body_hash -Expected $scopedEnvelope.request_body.body_sha256
        Assert-Equal -Name "scoped export body hash binding" -Actual $scopedExportBinding.request_body_hash -Expected $scopedEnvelope.request_body.body_sha256
        Assert-Equal -Name "scoped authorization body hash" -Actual $scopedAuthorization.request_body_hash -Expected $scopedEnvelope.request_body.body_sha256
        Assert-Equal -Name "scoped sent body hash" -Actual $scopedSent.request_body_hash -Expected $scopedEnvelope.request_body.body_sha256
        Assert-Equal -Name "scoped request envelope hash" -Actual $scopedRequestBinding.request_envelope_hash -Expected $scopedEnvelope.evidence.envelope_hash
        Assert-Equal -Name "scoped export envelope hash" -Actual $scopedExportBinding.request_envelope_hash -Expected $scopedEnvelope.evidence.envelope_hash
        Assert-Equal -Name "scoped authorization envelope hash" -Actual $scopedAuthorization.request_envelope_hash -Expected $scopedEnvelope.evidence.envelope_hash
        Assert-Equal -Name "scoped sent envelope hash" -Actual $scopedSent.request_envelope_hash -Expected $scopedEnvelope.evidence.envelope_hash
        Assert-Equal -Name "scoped authorization packet hash binding" -Actual $scopedAuthorization.packet_hash -Expected $scopedRequestBinding.hashes.projected_packet_hash
        Assert-Equal -Name "scoped export packet hash binding" -Actual $scopedExportBinding.hashes.projected_packet_hash -Expected $scopedRequestBinding.hashes.projected_packet_hash

        $packetJson = '{"profile":"provider_minimal","records":[{"field":"check_id","classification":"public","value":"' +
            $packet.check_id + '"},{"field":"revision_sha256","classification":"public","value":"' +
            $packet.revision_sha256 + '"},{"field":"tree_sha256","classification":"public","value":"' +
            $packet.tree_sha256 + '"},{"field":"reason","classification":"public","value":"' +
            $packet.reason + '"}]}'
        $escapedPacketJson = $packetJson.Replace('\', '\\').Replace('"', '\"')
        $expectedBody = '{"model":"gpt-5.4","instructions":"Return one complete corrected RAIOS_SOURCE_FILES_V1 revision using only the attached system feedback.","input":"' +
            $escapedPacketJson + '","max_output_tokens":8192,"store":false}'
        $expectedPacketHash = Get-TextSha256 -Text $packetJson
        $expectedBodyHash = Get-TextSha256 -Text $expectedBody
        Assert-Equal -Name "exact four-field packet hash" -Actual $scopedAuthorization.packet_hash -Expected $expectedPacketHash
        Assert-Equal -Name "exact scoped provider body hash" -Actual $scopedEnvelope.request_body.body_sha256 -Expected $expectedBodyHash
        $projectDiagnostics.scoped_transport.expected_packet_json = $packetJson
        $projectDiagnostics.scoped_transport.expected_packet_hash = $expectedPacketHash
        $projectDiagnostics.scoped_transport.expected_body_hash = $expectedBodyHash

        $scopedMarkerLines = @($scopedTransportSerial -split '\r?\n' | Where-Object {
            $_ -like 'OPENAI_PROVIDER_REQUEST_ENVELOPE *' -or
            $_ -like 'OPENAI_PROVIDER_REQUEST_BINDING *' -or
            $_ -like 'OPENAI_PROVIDER_EXPORT_AUDIT_BINDING *' -or
            $_ -like 'OPENAI_SCOPED_PROJECT_FEEDBACK_EXPORT *' -or
            $_ -like 'OPENAI_SCOPED_PROJECT_FEEDBACK_SENT *'
        } | Select-Object -Last 5)
        Assert-Equal -Name "scoped marker count" -Actual $scopedMarkerLines.Count -Expected 5
        $scopedMarkerText = $scopedMarkerLines -join "`n"
        foreach ($forbidden in @($safePrompt, "src/main.rs", "Cargo", "Content-Length", "Authorization: Bearer")) {
            if ($scopedMarkerText.IndexOf($forbidden, [StringComparison]::OrdinalIgnoreCase) -ge 0) {
                throw "Scoped provider marker leaked forbidden text '$forbidden'."
            }
        }
        if ($scopedTransportSerial -like "*tls_certificate_verification_bypassed*") {
            throw "B2.2 live scoped export saw unverified TLS bypass output."
        }
        $tlsEstablishedCountAfterSubmit = [regex]::Matches($scopedTransportSerial, '(?m)^openai: TLS 1\.3 established\r*$').Count
        $positiveTrustCountAfterSubmit = [regex]::Matches($scopedTransportSerial, '(?m)^openai: TLS provider trust verified: (?:pinned_cert|pinned_spki)(?:\s|$)').Count
        Assert-Equal -Name "scoped TLS-established marker count" -Actual $tlsEstablishedCountAfterSubmit -Expected ($tlsEstablishedCountBeforeSubmit + 1)
        Assert-Equal -Name "scoped positive-trust marker count" -Actual $positiveTrustCountAfterSubmit -Expected ($positiveTrustCountBeforeSubmit + 1)
        $authorizedIndex = $scopedTransportSerial.LastIndexOf("OPENAI_SCOPED_PROJECT_FEEDBACK_EXPORT {`"status`":`"authorized`"", [StringComparison]::Ordinal)
        $httpsWriteIndex = $scopedTransportSerial.LastIndexOf("openai: HTTPS request sent", [StringComparison]::Ordinal)
        $sentIndex = $scopedTransportSerial.LastIndexOf("OPENAI_SCOPED_PROJECT_FEEDBACK_SENT {`"request_id`":$feedbackRequestId", [StringComparison]::Ordinal)
        if ($authorizedIndex -lt 0 -or $httpsWriteIndex -le $authorizedIndex -or $sentIndex -le $httpsWriteIndex) {
            throw "Scoped gate/audit authorization did not precede the r2 HTTPS write marker."
        }
        Write-Host "openai-direct:b2-2-live-scoped-export-authorized passed: gate + durable export_audit + attached body, no bypass"
        Write-Host "openai-direct:b2-2-export-provider-minimal-no-leak passed: exact four-field packet and fixed instruction body hash"

        $revisionTwoOutcomeLine = Wait-ForLogLineMatch `
            -Path $SerialLog `
            -Pattern "^PROJECT SOURCE (READY|REJECTED) request=$feedbackRequestId(?:\s|$)" `
            -TimeoutSeconds $liveOutcomeTimeoutSeconds
        $parsedRevisionTwoOutcome = ConvertFrom-ProjectOutcomeLine `
            -Line $revisionTwoOutcomeLine.Line `
            -ExpectedRequestId $feedbackRequestId `
            -RetainedRevisionSha256 $revisionOneSha256 `
            -RetainedFileCount 2
        $revisionTwoOutcome = $parsedRevisionTwoOutcome.Outcome
        $projectDiagnostics.revision_two_outcome = $revisionTwoOutcome
        Assert-Equal -Name "revision 2 outcome project id" -Actual $revisionTwoOutcome.project_id -Expected $projectId

        $finalWorkspaceResponse = Invoke-AgentCommandJson `
            -Port $SerialTcpPort `
            -SerialLog $SerialLog `
            -Command "project.workspace" `
            -Method "project.workspace" `
            -TimeoutSeconds $TimeoutSeconds
        $finalWorkspace = $finalWorkspaceResponse.body.result
        $projectDiagnostics.final_workspace = $finalWorkspaceResponse
        Assert-Equal -Name "final workspace version" -Actual $finalWorkspaceResponse.v -Expected "raios.agent.v0"
        Assert-Equal -Name "final workspace method" -Actual $finalWorkspace.method -Expected "project.workspace"
        Assert-Equal -Name "final workspace scope" -Actual $finalWorkspace.scope -Expected "current_boot"
        Assert-Equal -Name "final workspace classification" -Actual $finalWorkspace.classification -Expected "local_only"
        Assert-Equal -Name "final workspace project id" -Actual $finalWorkspace.project_id -Expected $projectId
        Assert-Equal -Name "final workspace pending request" -Actual $finalWorkspace.pending_request_id -Expected $null
        Assert-Equal -Name "final workspace latest revision" -Actual $finalWorkspace.latest_revision_present -Expected $true
        Assert-Equal -Name "final workspace revision action" -Actual $finalWorkspace.revision_action -Expected "agent_answer"
        Assert-Equal -Name "final workspace source authority" -Actual $finalWorkspace.source_authority -Expected "untrusted_agent_candidate"
        Assert-Equal -Name "final workspace answer origin" -Actual $finalWorkspace.answer_origin -Expected "live"
        Assert-Equal -Name "final workspace provider trust" -Actual $finalWorkspace.provider_trust_positive -Expected $true
        Assert-Equal -Name "final workspace test infrastructure" -Actual $finalWorkspace.test_infrastructure -Expected $false
        Assert-Equal -Name "final workspace retained feedback revision" -Actual $finalWorkspace.feedback_packet.revision_sha256 -Expected $revisionOneSha256
        Assert-Equal -Name "final workspace retained feedback tree" -Actual $finalWorkspace.feedback_packet.tree_sha256 -Expected $verify.tree_sha256
        Assert-Equal -Name "final workspace retained feedback reason" -Actual $finalWorkspace.feedback_packet.reason -Expected "build_cargo_lock_missing"
        $finalLineage = @($finalWorkspace.revision_lineage)
        if ($revisionTwoOutcome.accepted) {
            Assert-Equal -Name "child workspace latest request" -Actual ([string]$finalWorkspace.latest_request_id) -Expected $feedbackRequestId
            Assert-Equal -Name "child workspace phase" -Actual $finalWorkspace.phase -Expected "source_ready"
            Assert-Equal -Name "child workspace latest revision" -Actual $finalWorkspace.latest_revision_present -Expected $true
            Assert-Equal -Name "child workspace revision" -Actual $finalWorkspace.revision_sha256 -Expected $revisionTwoOutcome.revision_sha256
            Assert-Equal -Name "child workspace file count" -Actual ([int]$finalWorkspace.file_count) -Expected $revisionTwoOutcome.file_count
            Assert-Equal -Name "child workspace parent" -Actual $finalWorkspace.parent_revision_sha256 -Expected $revisionOneSha256
            Assert-Equal -Name "child workspace parent readback" -Actual $finalWorkspace.parent_revision_readback_verified -Expected $true
            Assert-Equal -Name "child lineage count" -Actual $finalLineage.Count -Expected 2
            Assert-Equal -Name "child lineage revision 1" -Actual $finalLineage[0].revision_sha256 -Expected $revisionOneSha256
            Assert-Equal -Name "child lineage revision 1 tree" -Actual $finalLineage[0].tree_sha256 -Expected $verify.tree_sha256
            Assert-Equal -Name "child lineage revision 1 parent" -Actual $finalLineage[0].parent_revision_sha256 -Expected $null
            Assert-Equal -Name "child lineage revision 2" -Actual $finalLineage[1].revision_sha256 -Expected $revisionTwoOutcome.revision_sha256
            Assert-Equal -Name "child lineage revision 2 parent" -Actual $finalLineage[1].parent_revision_sha256 -Expected $revisionOneSha256
            $revisedChildOutcomeSummary = "live child parent=$revisionOneSha256"
            $mayProbeReplayWithoutExport = $true
        }
        else {
            $grammarReasons = @(
                "project_no_files",
                "project_file_quota_exceeded",
                "project_path_invalid",
                "project_source_path_type_unsupported",
                "project_source_utf8_invalid",
                "project_path_collision",
                "project_total_quota_exceeded"
            )
            if ($revisionTwoOutcome.reason -notlike "agent_answer_*" -and $grammarReasons -notcontains $revisionTwoOutcome.reason) {
                throw "Revised provider answer failed outside the accepted grammar boundary: $($revisionTwoOutcome.reason)"
            }
            Assert-Equal -Name "rejected workspace phase" -Actual $finalWorkspace.phase -Expected "rejected"
            Assert-Equal -Name "rejected workspace latest request" -Actual ([string]$finalWorkspace.latest_request_id) -Expected $projectRequestId
            Assert-Equal -Name "rejected workspace revision" -Actual $finalWorkspace.revision_sha256 -Expected $revisionOneSha256
            Assert-Equal -Name "rejected workspace tree" -Actual $finalWorkspace.tree_sha256 -Expected $verify.tree_sha256
            Assert-Equal -Name "rejected workspace file count" -Actual ([int]$finalWorkspace.file_count) -Expected 2
            Assert-Equal -Name "rejected lineage count" -Actual $finalLineage.Count -Expected 1
            Assert-Equal -Name "rejected lineage revision 1" -Actual $finalLineage[0].revision_sha256 -Expected $revisionOneSha256
            Assert-Equal -Name "rejected lineage revision 1 tree" -Actual $finalLineage[0].tree_sha256 -Expected $verify.tree_sha256
            Assert-Equal -Name "rejected lineage parent" -Actual $finalLineage[0].parent_revision_sha256 -Expected $null
            $finalPaths = @($finalWorkspace.files | ForEach-Object { $_.path })
            if (($finalPaths -join "`n") -cne ($revisionOnePaths -join "`n")) {
                throw "Grammar rejection did not retain revision 1's readable file list."
            }
            $revisedChildOutcomeSummary = "honest grammar rejection=$($revisionTwoOutcome.reason), revision 1 intact"
            $mayProbeReplayWithoutExport = $false
        }
        Write-Host "openai-direct:b2-2-revised-child-outcome passed: $revisedChildOutcomeSummary"
        if ($mayProbeReplayWithoutExport) {
            # Success path: the committed child moved the latest revision, so a fresh
            # project.feedback_submit is denied for packet mismatch and sends no body.
            $directRequestCountBeforeReplay = [regex]::Matches((Get-Content -Raw -LiteralPath $SerialLog), '(?m)^OPENAI_DIRECT_REQ [0-9]+ api\.openai\.com /v1/responses\r*$').Count
            $replayResponse = Invoke-AgentCommandJson `
                -Port $SerialTcpPort `
                -SerialLog $SerialLog `
                -Command "project.feedback_submit" `
                -Method "project.feedback_submit" `
                -TimeoutSeconds $TimeoutSeconds
            $replay = $replayResponse.body.result
            $projectDiagnostics.feedback_submit_replay = $replayResponse
            Assert-Equal -Name "feedback replay version" -Actual $replayResponse.v -Expected "raios.agent.v0"
            Assert-Equal -Name "feedback replay status" -Actual $replay.status -Expected "denied"
            Assert-Equal -Name "feedback replay accepted" -Actual $replay.accepted -Expected $false
            Assert-Equal -Name "feedback replay reason" -Actual $replay.reason -Expected "agent_revision_feedback_packet_mismatch"
            Assert-Equal -Name "feedback replay request id" -Actual $replay.request_id -Expected $null
            Assert-Equal -Name "feedback replay context attachment" -Actual $replay.context_attached_to_provider_body -Expected $false
            Assert-Equal -Name "feedback replay provider write" -Actual $replay.provider_write -Expected "not_attempted"
            $directRequestCountAfterReplay = [regex]::Matches((Get-Content -Raw -LiteralPath $SerialLog), '(?m)^OPENAI_DIRECT_REQ [0-9]+ api\.openai\.com /v1/responses\r*$').Count
            Assert-Equal -Name "feedback replay provider request count" -Actual $directRequestCountAfterReplay -Expected $directRequestCountBeforeReplay
            Write-Host "openai-direct:b2-2-single-use-consumed passed: agent_revision_feedback_packet_mismatch, no second provider body"
        }
        else {
            # Rejection path: revision 1 and its retained feedback packet remain for owner
            # transparency. Single use is per authorization, not per packet: the one
            # authorization produced exactly one durably audited scoped export, and a
            # deliberate fresh project.feedback_submit would be a NEW gated+audited export
            # by design, so the harness does not send one (that is not a single-use
            # violation; each export is independently authorized and logged).
            $scopedSentCount = [regex]::Matches($scopedTransportSerial, [regex]::Escape("OPENAI_SCOPED_PROJECT_FEEDBACK_SENT {`"request_id`":$feedbackRequestId")).Count
            Assert-Equal -Name "scoped export sent exactly once for authorization" -Actual $scopedSentCount -Expected 1
            if ($scopedAuthorization.durable_audit_payload_sha256 -notmatch '^sha256:[0-9a-f]{64}$' -or $null -eq $scopedAuthorization.durable_audit_seq) {
                throw "Scoped export authorization lacked a unique durable audit binding."
            }
            $directRequestTotal = [regex]::Matches((Get-Content -Raw -LiteralPath $SerialLog), '(?m)^OPENAI_DIRECT_REQ [0-9]+ api\.openai\.com /v1/responses\r*$').Count
            Assert-Equal -Name "exactly two provider requests total (initial + one export)" -Actual $directRequestTotal -Expected 2
            Write-Host "openai-direct:b2-2-single-use-consumed passed: one durably-audited export per authorization; retained packet kept for transparency, no unintended second export"
        }
    }
    elseif ($ExpectProjectWorkspaceAnswer) {
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
        $parsedProjectOutcome = ConvertFrom-ProjectOutcomeLine -Line $projectOutcomeLine.Line -ExpectedRequestId $projectRequestId
        $projectOutcome = $parsedProjectOutcome.Outcome
        $projectOutcomeSummary = $parsedProjectOutcome.Summary
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
    if ($ExpectProjectWorkspaceAnswer -or $ExpectScopedFeedbackExport) {
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
    if ((-not $ExpectProviderResponse) -and (-not $ExpectPinnedTrust) -and (-not $ExpectSpkiPinnedTrust) -and (-not $ExpectProjectWorkspaceAnswer) -and (-not $ExpectScopedFeedbackExport) -and (-not $ExpectPinMismatch) -and ($serial -like "*OPENAI_DIRECT_REQ*")) {
        throw "Trust-gate smoke saw an OpenAI request start before provider trust was verified in $SerialLog"
    }
    if ((-not $ExpectProviderResponse) -and (-not $ExpectPinnedTrust) -and (-not $ExpectSpkiPinnedTrust) -and (-not $ExpectProjectWorkspaceAnswer) -and (-not $ExpectScopedFeedbackExport) -and (-not $ExpectPinMismatch) -and ($serial -like "*raios.provider_request_envelope.v0*")) {
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
    if ((-not $ExpectScopedFeedbackExport) -and (($serial -like '*"context_attached_to_provider_body":true*') -or ($serial -like '*"context_attached_to_provider_body": true*'))) {
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
    if ($ExpectScopedFeedbackExport -and ($serial -like "*tls_certificate_verification_bypassed*")) {
        throw "Live scoped-feedback smoke saw unverified TLS bypass output in $SerialLog"
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

    if ($ExpectScopedFeedbackExport) {
        Write-Host "openai direct B2.2 scoped-feedback live smoke passed"
    }
    elseif ($ExpectProjectWorkspaceAnswer) {
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
    if ($projectDiskMode) {
        Write-Host "$projectFailureLabel failure: $($_.Exception.Message)"
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
