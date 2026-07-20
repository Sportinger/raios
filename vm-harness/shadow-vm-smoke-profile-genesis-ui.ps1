# Genesis core UI focused profile (A2/G2).
#
# Dot-source this after shadow-vm-smoke-profile-common.ps1.  The dispatcher
# already owns boot, framebuffer, USB, serial TCP and QEMU lifecycle.  This
# profile uses the normal serial command path plus physical QEMU HID input;
# framebuffer captures remain supporting evidence rather than the sole oracle.
#
# What this proves today: the typed current-boot facts that Genesis renders and
# its read-only recovery source are live, coherent and remain outside Wasm and
# provider authority. Trusted setup and Recovery are driven through physical
# QEMU HID input and require non-secret guest acknowledgements before capture. It also
# proves the bounded text-editor program through the same serial, approval and HID paths.

function Send-GenesisUiKey {
    param([string]$KeyName)

    Send-QemuMonitorCommand -Command "sendkey $KeyName 60" -ReplyWaitMilliseconds 250 | Out-Null
}

function Send-GenesisUiProgramBytes {
    param(
        [byte[]]$Bytes,
        [string]$NamePrefix
    )

    $base64 = [Convert]::ToBase64String($Bytes)
    $chunkChars = 3000
    $chunkIndex = 0
    $pendingBytes = 0
    for ($offset = 0; $offset -lt $base64.Length; $offset += $chunkChars) {
        $count = [Math]::Min($chunkChars, $base64.Length - $offset)
        $chunk = $base64.Substring($offset, $count)
        $decodedBytes = [Convert]::FromBase64String($chunk).Length
        $chunkIndex += 1
        $pendingBytes += $decodedBytes
        Send-AgentCommand -Command "program.submit_chunk $chunk" -ExpectedMarker "RAIOS_AGENT_END program.submit_chunk" -Name "$NamePrefix`:chunk_$chunkIndex"
        $result = (Get-LastAgentResponseJson -Method "program.submit_chunk").body.result
        $chunkOk = (
            $result.accepted -eq $true -and
            $result.rejected -eq $false -and
            $result.reason -eq "accepted_program_chunk" -and
            [int]$result.decoded_byte_len -eq $decodedBytes -and
            [int]$result.pending_byte_len -eq $pendingBytes -and
            [int]$result.pending_chunk_count -eq $chunkIndex -and
            $result.discarded_pending_delivery -eq $false -and
            $result.signing_attempted -eq $false -and
            $result.load_attempted -eq $false -and
            $result.execution_attempted -eq $false -and
            $result.writes_persistent_state -eq $false
        )
        Add-Predicate -Name "$NamePrefix`:chunk_$chunkIndex-inert" -Expected "canonical RUIP chunk is retained pending without signing, loading, execution or persistence" -Passed $chunkOk -Actual $(if ($chunkOk) { "pending_bytes=$pendingBytes" } else { ($result | ConvertTo-Json -Compress -Depth 5) })
        if (-not $chunkOk) {
            throw "Expected RUIP chunk $chunkIndex to remain pending and inert"
        }
    }
    return $chunkIndex
}

# Frozen output of raios_core::ui_program::calculator_program().canonical_bytes().
$genesisCalculatorBase64 = @'
UlVJUAEAIAD8FAAABBISMwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEACgAAAAAAQAEgAAAAAABDYWxjdWxhdG9yAAACAAAAAAAoAEABOAAAAAAAAwABAAAAaABIADgACAAAADcAAAADAAEAUABoAEgAOAAJAAAAOAAAAAMAAQCgAGgASAA4AAoAAAA5AAAAAwABAPAAaABIADgADgAAAC8AAAADAAEAAACoAEgAOAAFAAAANAAAAAMAAQBQAKgASAA4AAYAAAA1AAAAAwABAKAAqABIADgABwAAADYAAAADAAEA8ACoAEgAOAANAAAAKgAAAAMAAQAAAOgASAA4AAIAAAAxAAAAAwABAFAA6ABIADgAAwAAADIAAAADAAEAoADoAEgAOAAEAAAAMwAAAAMAAQDwAOgASAA4AAwAAAAtAAAAAwABAAAAKAFIADgAEAAAAEMAAAADAAEAUAAoAUgAOAABAAAAMAAAAAMAAQCgACgBSAA4AA8AAAA9AAAAAwABAPAAKAFIADgACwAAACsAAAAwAAAAAQAAADEAAAACAAAAMgAAAAMAAAAzAAAABAAAADQAAAAFAAAANQAAAAYAAAA2AAAABwAAADcAAAAIAAAAOAAAAAkAAAA5AAAACgAAACsAAAALAAAALQAAAAwAAAAqAAAADQAAAC8AAAAOAAAAPQAAAA8AAAANAAAADwAAAGMAAAAQAAAAQwAAABAAAAABAAECAAAAAAMBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEDAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAEAAQEAAAAAAwEAAAEAAAAAAAAAAAAAAAAAAAAAAAAABwAAAAAAAAAKAAAAAAAAAAAAAAAAAAAAAgABAgAAAAADAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAAAAAAAAAEAAAAAAAAAAAAAAAAAAAABAwAAAAAAAAEAAAAAAAAAAAAAAAAAAAACAAEBAAAAAAMBAAABAAAAAAAAAAAAAAAAAAAAAAAAAAcAAAAAAAAACgAAAAAAAAABAAAAAAAAAAMAAQIAAAAAAwEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAACAAAAAAAAAAAAAAAAAAAAAQMAAAAAAAABAAAAAAAAAAAAAAAAAAAAAwABAQAAAAADAQAAAQAAAAAAAAAAAAAAAAAAAAAAAAAHAAAAAAAAAAoAAAAAAAAAAgAAAAAAAAAEAAECAAAAAAMBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAAAAAAAAAAwAAAAAAAAAAAAAAAAAAAAEDAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAQAAQEAAAAAAwEAAAEAAAAAAAAAAAAAAAAAAAAAAAAABwAAAAAAAAAKAAAAAAAAAAMAAAAAAAAABQABAgAAAAADAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAABAwAAAAAAAAEAAAAAAAAAAAAAAAAAAAAFAAEBAAAAAAMBAAABAAAAAAAAAAAAAAAAAAAAAAAAAAcAAAAAAAAACgAAAAAAAAAEAAAAAAAAAAYAAQIAAAAAAwEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAAFAAAAAAAAAAAAAAAAAAAAAQMAAAAAAAABAAAAAAAAAAAAAAAAAAAABgABAQAAAAADAQAAAQAAAAAAAAAAAAAAAAAAAAAAAAAHAAAAAAAAAAoAAAAAAAAABQAAAAAAAAAHAAECAAAAAAMBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAAAAAAAAABgAAAAAAAAAAAAAAAAAAAAEDAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAcAAQEAAAAAAwEAAAEAAAAAAAAAAAAAAAAAAAAAAAAABwAAAAAAAAAKAAAAAAAAAAYAAAAAAAAACAABAgAAAAADAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAAAAAAAAAcAAAAAAAAAAAAAAAAAAAABAwAAAAAAAAEAAAAAAAAAAAAAAAAAAAAIAAEBAAAAAAMBAAABAAAAAAAAAAAAAAAAAAAAAAAAAAcAAAAAAAAACgAAAAAAAAAHAAAAAAAAAAkAAQIAAAAAAwEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAAIAAAAAAAAAAAAAAAAAAAAAQMAAAAAAAABAAAAAAAAAAAAAAAAAAAACQABAQAAAAADAQAAAQAAAAAAAAAAAAAAAAAAAAAAAAAHAAAAAAAAAAoAAAAAAAAACAAAAAAAAAAKAAECAAAAAAMBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAAAAAAAAACQAAAAAAAAAAAAAAAAAAAAEDAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAoAAQEAAAAAAwEAAAEAAAAAAAAAAAAAAAAAAAAAAAAABwAAAAAAAAAKAAAAAAAAAAkAAAAAAAAACwABAQAAAAADAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAgAAAAAAAAEAAAAAAAAAAAAAAAAAAAALAAIDAAAAAAMBAAABAAAAAAAAAAIBAAAAAAAAAAAAAAIBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAECAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAEDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAsAAgQAAAAAAwEAAAEAAAAAAAAAAgEAAAEAAAAAAAAAAwEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQIAAAAAAAABAAAAAAAAAAAAAAAAAAAAAQMAAAAAAAAAAAAAAAAAAAAAAAAAAAAACwACBAAAAAADAQAAAQAAAAAAAAACAQAAAgAAAAAAAAAEAQEAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAABAgAAAAAAAAEAAAAAAAAAAAAAAAAAAAABAwAAAAAAAAAAAAAAAAAAAAAAAAAAAAALAAIEAAAAAAMBAAABAAAAAAAAAAIBAAADAAAAAAAAAAUBAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAIAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAECAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAEDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAsAAgQAAAAAAwEAAAEAAAAAAAAAAgEAAAQAAAAAAAAABgEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQIAAAAAAAABAAAAAAAAAAAAAAAAAAAAAQMAAAAAAAAAAAAAAAAAAAAAAAAAAAAADAABAQAAAAADAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAgAAAAAAAAIAAAAAAAAAAAAAAAAAAAAMAAIDAAAAAAMBAAABAAAAAAAAAAIBAAAAAAAAAAAAAAIBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAECAAAAAAAAAgAAAAAAAAAAAAAAAAAAAAEDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAwAAgQAAAAAAwEAAAEAAAAAAAAAAgEAAAEAAAAAAAAAAwEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQIAAAAAAAACAAAAAAAAAAAAAAAAAAAAAQMAAAAAAAAAAAAAAAAAAAAAAAAAAAAADAACBAAAAAADAQAAAQAAAAAAAAACAQAAAgAAAAAAAAAEAQEAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAABAgAAAAAAAAIAAAAAAAAAAAAAAAAAAAABAwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMAAIEAAAAAAMBAAABAAAAAAAAAAIBAAADAAAAAAAAAAUBAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAIAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAECAAAAAAAAAgAAAAAAAAAAAAAAAAAAAAEDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAwAAgQAAAAAAwEAAAEAAAAAAAAAAgEAAAQAAAAAAAAABgEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQIAAAAAAAACAAAAAAAAAAAAAAAAAAAAAQMAAAAAAAAAAAAAAAAAAAAAAAAAAAAADQABAQAAAAADAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAgAAAAAAAAMAAAAAAAAAAAAAAAAAAAANAAIDAAAAAAMBAAABAAAAAAAAAAIBAAAAAAAAAAAAAAIBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAECAAAAAAAAAwAAAAAAAAAAAAAAAAAAAAEDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA0AAgQAAAAAAwEAAAEAAAAAAAAAAgEAAAEAAAAAAAAAAwEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQIAAAAAAAADAAAAAAAAAAAAAAAAAAAAAQMAAAAAAAAAAAAAAAAAAAAAAAAAAAAADQACBAAAAAADAQAAAQAAAAAAAAACAQAAAgAAAAAAAAAEAQEAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAABAgAAAAAAAAMAAAAAAAAAAAAAAAAAAAABAwAAAAAAAAAAAAAAAAAAAAAAAAAAAAANAAIEAAAAAAMBAAABAAAAAAAAAAIBAAADAAAAAAAAAAUBAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAIAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAECAAAAAAAAAwAAAAAAAAAAAAAAAAAAAAEDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA0AAgQAAAAAAwEAAAEAAAAAAAAAAgEAAAQAAAAAAAAABgEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQIAAAAAAAADAAAAAAAAAAAAAAAAAAAAAQMAAAAAAAAAAAAAAAAAAAAAAAAAAAAADgABAQAAAAADAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAgAAAAAAAAQAAAAAAAAAAAAAAAAAAAAOAAIDAAAAAAMBAAABAAAAAAAAAAIBAAAAAAAAAAAAAAIBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAECAAAAAAAABAAAAAAAAAAAAAAAAAAAAAEDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4AAgQAAAAAAwEAAAEAAAAAAAAAAgEAAAEAAAAAAAAAAwEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQIAAAAAAAAEAAAAAAAAAAAAAAAAAAAAAQMAAAAAAAAAAAAAAAAAAAAAAAAAAAAADgACBAAAAAADAQAAAQAAAAAAAAACAQAAAgAAAAAAAAAEAQEAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAABAgAAAAAAAAQAAAAAAAAAAAAAAAAAAAABAwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAOAAIEAAAAAAMBAAABAAAAAAAAAAIBAAADAAAAAAAAAAUBAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAIAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAECAAAAAAAABAAAAAAAAAAAAAAAAAAAAAEDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA4AAgQAAAAAAwEAAAEAAAAAAAAAAgEAAAQAAAAAAAAABgEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQIAAAAAAAAEAAAAAAAAAAAAAAAAAAAAAQMAAAAAAAAAAAAAAAAAAAAAAAAAAAAADwABAQAAAAADAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAPAAIBAAAAAAMBAAABAAAAAAAAAAIBAAAAAAAAAAAAAAEDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA8AAgQAAAAAAwEAAAEAAAAAAAAAAgEAAAEAAAAAAAAAAwEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQMAAAAAAAAAAAAAAAAAAAAAAAAAAAAADwACBAAAAAADAQAAAQAAAAAAAAACAQAAAgAAAAAAAAAEAQEAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAABAgAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAPAAIEAAAAAAMBAAABAAAAAAAAAAIBAAADAAAAAAAAAAUBAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAIAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAECAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA8AAgQAAAAAAwEAAAEAAAAAAAAAAgEAAAQAAAAAAAAABgEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQMAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAAABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAgAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAwAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
'@ -replace '\s', ''
$genesisCalculatorBytes = [Convert]::FromBase64String($genesisCalculatorBase64)
$genesisCalculatorSha256 = "7ca0aa21d69baae072675c20f7b44d0e2d9f99ac4e72d6aa64e7a25586dfcd6e"
$genesisCalculatorHash = "sha256:$genesisCalculatorSha256"

Send-AgentCommand -Command "snapshot" -ExpectedMarker "RAIOS_AGENT_END system.snapshot" -Name "genesis-ui:context-snapshot"
$genesisSnapshot = Get-LastAgentResponseJson -Method "system.snapshot"
# P4 evidence vocabulary: system.snapshot answers in the raios.evidence_response.v1
# envelope; the snapshot facts live under .facts and the decision must be a pure
# observation that grants nothing (no grants/effects keys at all).
$genesisSystem = $genesisSnapshot.facts
$genesisStatus = $genesisSystem.status
$genesisProblems = @($genesisSystem.problems)
$genesisSnapshotOk = (
    $genesisSnapshot.schema -eq "raios.evidence_response.v1" -and
    $genesisSnapshot.family -eq "system.snapshot" -and
    $genesisSnapshot.source_method -eq "system.snapshot" -and
    $genesisSnapshot.scope -eq "current_boot" -and
    $genesisSnapshot.classification -eq "local_only" -and
    $genesisSnapshot.decision.outcome -eq "observed" -and
    $null -eq $genesisSnapshot.decision.grants -and
    $null -eq $genesisSnapshot.decision.effects -and
    $genesisSystem.os.name -eq "raiOS" -and
    $genesisSystem.os.stage -eq "stage-0" -and
    $null -ne $genesisStatus -and
    $null -ne $genesisStatus.framebuffer -and
    $null -ne $genesisStatus.input -and
    $null -ne $genesisStatus.network -and
    $null -ne $genesisSystem.provider -and
    $null -ne $genesisSystem.problems
)
Add-Predicate -Name "genesis-ui:context-snapshot-live" -Expected "Genesis Context sources are a live system.snapshot with framebuffer, input, network, provider and problems" -Passed $genesisSnapshotOk -Actual $(if ($genesisSnapshotOk) { "live problems=$($genesisProblems.Count)" } else { ($genesisSystem | ConvertTo-Json -Compress -Depth 6) })
if (-not $genesisSnapshotOk) {
    throw "Expected a live system.snapshot for Genesis Context"
}

Send-AgentCommand -Command "problems" -ExpectedMarker "RAIOS_AGENT_END problem.list" -Name "genesis-ui:problem-facts"
$genesisProblemList = Get-LastAgentResponseJson -Method "problem.list"
# P4 evidence vocabulary: problem.list answers in the raios.evidence_response.v1
# envelope with the typed entries under .facts.problems and a pure observation
# decision that grants nothing.
$genesisProblemResult = $genesisProblemList.facts
$genesisProblemEntries = @($genesisProblemResult.problems)
$genesisProblemFactsOk = (
    $genesisProblemList.schema -eq "raios.evidence_response.v1" -and
    $genesisProblemList.family -eq "problem.list" -and
    $genesisProblemList.scope -eq "current_boot" -and
    $genesisProblemList.classification -eq "local_only" -and
    $genesisProblemList.decision.outcome -eq "observed" -and
    $null -eq $genesisProblemList.decision.grants -and
    ($genesisProblemEntries | Where-Object {
        [string]::IsNullOrWhiteSpace([string]$_.id) -or
        [string]::IsNullOrWhiteSpace([string]$_.severity) -or
        [string]::IsNullOrWhiteSpace([string]$_.summary)
    }).Count -eq 0
)
Add-Predicate -Name "genesis-ui:problem-facts-typed" -Expected "Genesis problem source is a typed problem.list without blank ids, severities or summaries" -Passed $genesisProblemFactsOk -Actual $(if ($genesisProblemFactsOk) { "typed problems=$($genesisProblemEntries.Count)" } else { ($genesisProblemResult | ConvertTo-Json -Compress -Depth 6) })
if (-not $genesisProblemFactsOk) {
    throw "Expected typed non-blank Genesis problem facts"
}

Send-AgentCommand -Command "agent recovery.snapshot" -ExpectedMarker "RAIOS_AGENT_END recovery.snapshot" -Name "genesis-ui:recovery-snapshot"
$genesisRecoveryResponse = Get-LastAgentResponseJson -Method "recovery.snapshot"
$genesisRecovery = $genesisRecoveryResponse.body.result
$genesisRecoveryOk = (
    $genesisRecovery.schema -eq "raios.recovery_snapshot.v0" -and
    $genesisRecovery.scope -eq "current_boot" -and
    $genesisRecovery.classification -eq "local_only" -and
    $genesisRecovery.lifeline_available -eq $true -and
    $genesisRecovery.mutates_state -eq $false -and
    $genesisRecovery.routes_through_wasm -eq $false -and
    $genesisRecovery.routes_through_provider -eq $false -and
    $genesisRecovery.redacted -eq $true -and
    $null -ne $genesisRecovery.durable_last_good -and
    $null -ne $genesisRecovery.rollback_preview
)
Add-Predicate -Name "genesis-ui:recovery-source-read-only" -Expected "Genesis Recovery context reads the live redacted local-only recovery snapshot without Wasm, provider or mutation authority" -Passed $genesisRecoveryOk -Actual $(if ($genesisRecoveryOk) { "posture=$($genesisRecovery.boot_posture)" } else { ($genesisRecovery | ConvertTo-Json -Compress -Depth 6) })
if (-not $genesisRecoveryOk) {
    throw "Expected a read-only local recovery snapshot for Genesis"
}

Send-AgentCommand -Command "agent recovery.lifeline_table" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_table" -Name "genesis-ui:recovery-lifeline"
$genesisLifelineResponse = Get-LastAgentResponseJson -Method "recovery.lifeline_table"
$genesisLifeline = $genesisLifelineResponse.body.result
$genesisLifelineMethods = @($genesisLifeline.methods)
$genesisLifelineOk = (
    $genesisLifeline.schema -eq "raios.recovery_lifeline_table.v0" -and
    $genesisLifeline.scope -eq "current_boot" -and
    $genesisLifeline.classification -eq "local_only" -and
    $genesisLifeline.mutates_state -eq $false -and
    $genesisLifeline.routes_through_wasm -eq $false -and
    $genesisLifeline.routes_through_provider -eq $false -and
    $genesisLifelineMethods.Count -ge 1 -and
    (@($genesisLifelineMethods | Where-Object { $_.name -eq "recovery.snapshot" -and $_.implemented -eq $true })).Count -eq 1
)
Add-Predicate -Name "genesis-ui:recovery-lifeline-available" -Expected "the recovery source behind Genesis is the existing local-only lifeline, not a UI-local evaluator" -Passed $genesisLifelineOk -Actual $(if ($genesisLifelineOk) { "methods=$($genesisLifelineMethods.Count)" } else { ($genesisLifeline | ConvertTo-Json -Compress -Depth 6) })
if (-not $genesisLifelineOk) {
    throw "Expected Genesis recovery to bind the existing recovery lifeline"
}

for ($index = 0; $index -lt 3; $index++) {
    Send-GenesisUiKey -KeyName "tab"
}
Send-GenesisUiKey -KeyName "ret"
Assert-LogContains -Name "genesis-ui:trusted-provider-setup-open" -Needle "1 PROVIDER: OPENAI DIRECT" -TimeoutSeconds $TimeoutSeconds
Send-GenesisUiKey -KeyName "tab"
Send-GenesisUiKey -KeyName "tab"
Start-Sleep -Milliseconds 300
Save-QemuScreendump -Name "trusted-provider-setup" | Out-Null
Send-GenesisUiKey -KeyName "esc"
Assert-LogContains -Name "genesis-ui:trusted-provider-setup-close" -Needle "SETUP CLOSED" -TimeoutSeconds $TimeoutSeconds

Send-GenesisUiKey -KeyName "f12"
Assert-LogContains -Name "genesis-ui:recovery-view-open" -Needle "GENESIS_RECOVERY_VIEW_OPENED current_boot=true" -TimeoutSeconds $TimeoutSeconds
Start-Sleep -Milliseconds 300
Save-QemuScreendump -Name "recovery-open" | Out-Null
Send-GenesisUiKey -KeyName "f12"
Assert-LogContains -Name "genesis-ui:recovery-view-close" -Needle "GENESIS_RECOVERY_VIEW_CLOSED current_boot=true" -TimeoutSeconds $TimeoutSeconds

Send-AgentCommand -Command "program.workspace" -ExpectedMarker "RAIOS_AGENT_END program.workspace" -Name "genesis-ui:program-workspace-empty"
$emptyProgramWorkspace = (Get-LastAgentResponseJson -Method "program.workspace").body.result
$emptyProgramWorkspaceOk = (
    $emptyProgramWorkspace.status -eq "empty" -and
    $emptyProgramWorkspace.scope -eq "current_boot" -and
    $emptyProgramWorkspace.classification -eq "local_only" -and
    $emptyProgramWorkspace.retention -eq "current_boot_ram_only" -and
    $emptyProgramWorkspace.present -eq $false -and
    [int]$emptyProgramWorkspace.revision -eq 0 -and
    [int]$emptyProgramWorkspace.byte_len -eq 0 -and
    $null -eq $emptyProgramWorkspace.program_sha256 -and
    [int]$emptyProgramWorkspace.pending_byte_len -eq 0 -and
    [int]$emptyProgramWorkspace.pending_chunk_count -eq 0 -and
    $emptyProgramWorkspace.signing_attempted -eq $false -and
    $emptyProgramWorkspace.load_attempted -eq $false -and
    $emptyProgramWorkspace.execution_attempted -eq $false -and
    $emptyProgramWorkspace.authorizes_load -eq $false -and
    $emptyProgramWorkspace.authorizes_execution -eq $false -and
    $emptyProgramWorkspace.writes_persistent_state -eq $false
)
Add-Predicate -Name "genesis-ui:program-workspace-starts-empty-inert" -Expected "program workspace starts empty, current-boot RAM-only and grants no authority" -Passed $emptyProgramWorkspaceOk -Actual $(if ($emptyProgramWorkspaceOk) { "empty current_boot" } else { ($emptyProgramWorkspace | ConvertTo-Json -Compress -Depth 5) })
if (-not $emptyProgramWorkspaceOk) {
    throw "Expected a fresh inert current-boot program workspace"
}

$sha = [System.Security.Cryptography.SHA256]::Create()
try {
    $calculatorActualSha256 = ([BitConverter]::ToString($sha.ComputeHash($genesisCalculatorBytes)) -replace "-", "").ToLowerInvariant()
}
finally {
    $sha.Dispose()
}
$calculatorFixtureOk = $genesisCalculatorBytes.Length -eq 5372 -and $calculatorActualSha256 -eq $genesisCalculatorSha256
Add-Predicate -Name "genesis-ui:calculator-ruip-exact-fixture" -Expected "raios-core calculator canonical RUIP is exactly 5372 bytes with the pinned SHA-256" -Passed $calculatorFixtureOk -Actual "bytes=$($genesisCalculatorBytes.Length) sha256=$calculatorActualSha256"
if (-not $calculatorFixtureOk) {
    throw "Canonical calculator RUIP fixture does not match its pinned identity"
}

$calculatorChunkCount = Send-GenesisUiProgramBytes -Bytes $genesisCalculatorBytes -NamePrefix "genesis-ui:calculator-delivery"
Send-AgentCommand -Command "program.submit_finalize" -ExpectedMarker "RAIOS_AGENT_END program.submit_finalize" -Name "genesis-ui:calculator-finalize"
$calculatorFinalize = (Get-LastAgentResponseJson -Method "program.submit_finalize").body.result
$calculatorFinalizeOk = (
    $calculatorFinalize.method -eq "program.submit_finalize" -and
    $calculatorFinalize.status -eq "ready" -and
    $calculatorFinalize.scope -eq "current_boot" -and
    $calculatorFinalize.classification -eq "local_only" -and
    $calculatorFinalize.retention -eq "current_boot_ram_only" -and
    $calculatorFinalize.accepted -eq $true -and
    $calculatorFinalize.rejected -eq $false -and
    $calculatorFinalize.reason -eq "retained_current_boot_inert_ui_program" -and
    [int]$calculatorFinalize.attempted_byte_len -eq 5372 -and
    $calculatorFinalize.present -eq $true -and
    [int]$calculatorFinalize.revision -eq 1 -and
    [int]$calculatorFinalize.byte_len -eq 5372 -and
    $calculatorFinalize.program_sha256 -eq $genesisCalculatorHash -and
    $calculatorFinalize.source -eq "serial" -and
    $null -eq $calculatorFinalize.provider_request_id -and
    [int]$calculatorFinalize.serial_chunk_count -eq $calculatorChunkCount -and
    [int]$calculatorFinalize.pending_byte_len -eq 0 -and
    [int]$calculatorFinalize.pending_chunk_count -eq 0 -and
    $calculatorFinalize.signing_attempted -eq $false -and
    $calculatorFinalize.load_attempted -eq $false -and
    $calculatorFinalize.execution_attempted -eq $false -and
    $calculatorFinalize.authorizes_load -eq $false -and
    $calculatorFinalize.authorizes_execution -eq $false -and
    $calculatorFinalize.writes_persistent_state -eq $false
)
Add-Predicate -Name "genesis-ui:calculator-retained-exact-current-boot-inert" -Expected "canonical calculator RUIP is retained under its exact hash but cannot sign, load, execute or persist" -Passed $calculatorFinalizeOk -Actual $(if ($calculatorFinalizeOk) { "$genesisCalculatorHash revision=1 chunks=$calculatorChunkCount" } else { ($calculatorFinalize | ConvertTo-Json -Compress -Depth 5) })
if (-not $calculatorFinalizeOk) {
    throw "Expected exact canonical calculator RUIP to be retained inertly"
}

Send-AgentCommand -Command "program.workspace" -ExpectedMarker "RAIOS_AGENT_END program.workspace" -Name "genesis-ui:calculator-workspace"
$calculatorWorkspace = (Get-LastAgentResponseJson -Method "program.workspace").body.result
$calculatorWorkspaceOk = (
    $calculatorWorkspace.status -eq "ready" -and
    $calculatorWorkspace.scope -eq "current_boot" -and
    $calculatorWorkspace.retention -eq "current_boot_ram_only" -and
    $calculatorWorkspace.present -eq $true -and
    [int]$calculatorWorkspace.revision -eq 1 -and
    [int]$calculatorWorkspace.byte_len -eq 5372 -and
    $calculatorWorkspace.program_sha256 -eq $genesisCalculatorHash -and
    $calculatorWorkspace.source -eq "serial" -and
    [int]$calculatorWorkspace.serial_chunk_count -eq $calculatorChunkCount -and
    $calculatorWorkspace.signing_attempted -eq $false -and
    $calculatorWorkspace.load_attempted -eq $false -and
    $calculatorWorkspace.execution_attempted -eq $false -and
    $calculatorWorkspace.authorizes_load -eq $false -and
    $calculatorWorkspace.authorizes_execution -eq $false -and
    $calculatorWorkspace.writes_persistent_state -eq $false
)
Add-Predicate -Name "genesis-ui:calculator-workspace-exact-hash-inert" -Expected "program.workspace exposes the exact current-boot calculator hash while retaining no execution authority" -Passed $calculatorWorkspaceOk -Actual $(if ($calculatorWorkspaceOk) { "$genesisCalculatorHash current_boot inert" } else { ($calculatorWorkspace | ConvertTo-Json -Compress -Depth 5) })
if (-not $calculatorWorkspaceOk) {
    throw "Expected exact inert calculator identity in program.workspace"
}

[byte[]]$malformedCalculator = $genesisCalculatorBytes.Clone()
$malformedCalculator[16] = 1
$malformedChunkCount = Send-GenesisUiProgramBytes -Bytes $malformedCalculator -NamePrefix "genesis-ui:malformed-program-delivery"
Send-AgentCommand -Command "program.submit_finalize" -ExpectedMarker "RAIOS_AGENT_END program.submit_finalize" -Name "genesis-ui:malformed-program-finalize"
$malformedFinalize = (Get-LastAgentResponseJson -Method "program.submit_finalize").body.result
$malformedFinalizeOk = (
    $malformedFinalize.accepted -eq $false -and
    $malformedFinalize.rejected -eq $true -and
    $malformedFinalize.reason -eq "program_malformed" -and
    [int]$malformedFinalize.attempted_byte_len -eq 5372 -and
    $malformedFinalize.status -eq "ready" -and
    $malformedFinalize.present -eq $true -and
    [int]$malformedFinalize.revision -eq 1 -and
    [int]$malformedFinalize.byte_len -eq 5372 -and
    $malformedFinalize.program_sha256 -eq $genesisCalculatorHash -and
    [int]$malformedFinalize.pending_byte_len -eq 0 -and
    [int]$malformedFinalize.pending_chunk_count -eq 0 -and
    $malformedFinalize.signing_attempted -eq $false -and
    $malformedFinalize.load_attempted -eq $false -and
    $malformedFinalize.execution_attempted -eq $false -and
    $malformedFinalize.authorizes_load -eq $false -and
    $malformedFinalize.authorizes_execution -eq $false -and
    $malformedFinalize.writes_persistent_state -eq $false
)
Add-Predicate -Name "genesis-ui:malformed-program-rejected-with-retained-calculator-inert" -Expected "nonzero reserved RUIP byte rejects atomically and leaves the exact prior calculator inert" -Passed $malformedFinalizeOk -Actual $(if ($malformedFinalizeOk) { "rejected chunks=$malformedChunkCount retained=$genesisCalculatorHash revision=1" } else { ($malformedFinalize | ConvertTo-Json -Compress -Depth 5) })
if (-not $malformedFinalizeOk) {
    throw "Expected malformed RUIP to reject without replacing or activating the calculator"
}

Send-AgentCommand -Command "program.workspace" -ExpectedMarker "RAIOS_AGENT_END program.workspace" -Name "genesis-ui:workspace-after-malformed-program"
$workspaceAfterMalformed = (Get-LastAgentResponseJson -Method "program.workspace").body.result
$workspaceAfterMalformedOk = (
    $workspaceAfterMalformed.status -eq "ready" -and
    $workspaceAfterMalformed.present -eq $true -and
    [int]$workspaceAfterMalformed.revision -eq 1 -and
    [int]$workspaceAfterMalformed.byte_len -eq 5372 -and
    $workspaceAfterMalformed.program_sha256 -eq $genesisCalculatorHash -and
    [int]$workspaceAfterMalformed.pending_byte_len -eq 0 -and
    [int]$workspaceAfterMalformed.pending_chunk_count -eq 0 -and
    $workspaceAfterMalformed.load_attempted -eq $false -and
    $workspaceAfterMalformed.execution_attempted -eq $false -and
    $workspaceAfterMalformed.authorizes_load -eq $false -and
    $workspaceAfterMalformed.authorizes_execution -eq $false -and
    $workspaceAfterMalformed.writes_persistent_state -eq $false
)
Add-Predicate -Name "genesis-ui:workspace-unchanged-after-malformed-program" -Expected "malformed delivery cannot change the retained hash, revision, pending state or authority" -Passed $workspaceAfterMalformedOk -Actual $(if ($workspaceAfterMalformedOk) { "$genesisCalculatorHash revision=1 unchanged" } else { ($workspaceAfterMalformed | ConvertTo-Json -Compress -Depth 5) })
if (-not $workspaceAfterMalformedOk) {
    throw "Expected malformed RUIP delivery to leave program.workspace unchanged"
}

Save-QemuScreendump -Name "calculator-awaiting-physical-approval" | Out-Null
# QMP sends the absolute USB-tablet event HMP mouse_move cannot represent.
# These are the approval-button center in QEMU's documented 0..32767 range.
Send-QemuAbsolutePointerClick -X 27017 -Y 6559
$programActivationMarker = "PROGRAM_CURRENT_BOOT_ACTIVATION physical_approval=pointer program_sha256=$genesisCalculatorHash engine=svc.user.shell capability_surface=ui_only wasm=true result=accepted"
Assert-LogContains -Name "genesis-ui:calculator-physical-approval-wasm-accepted" -Needle $programActivationMarker -TimeoutSeconds $TimeoutSeconds
Start-Sleep -Milliseconds 300
Save-QemuScreendump -Name "calculator-active-after-physical-approval" | Out-Null

Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "genesis-ui:calculator-current-boot-inventory"
$calculatorInventoryResponse = Get-LastAgentResponseJson -Method "service.inventory"
$calculatorInventory = @($calculatorInventoryResponse.facts.services | Where-Object { $_.id -eq "svc.user.shell" })
$calculatorInventoryOk = (
    $calculatorInventory.Count -eq 1 -and
    $calculatorInventory[0].kind -eq "service" -and
    $calculatorInventory[0].scope -eq "current_boot" -and
    $calculatorInventory[0].persistence -eq "none" -and
    $calculatorInventory[0].capability_envelope -eq "wasmi_linker_import_surface" -and
    $calculatorInventory[0].host_import_count -eq 6 -and
    $calculatorInventory[0].running -eq $true
)
Add-Predicate -Name "genesis-ui:calculator-runs-only-as-ui-current-boot-service" -Expected "physical approval starts svc.user.shell only as a current-boot Wasmi UI service" -Passed $calculatorInventoryOk -Actual $(if ($calculatorInventoryOk) { "svc.user.shell current_boot ui_only running" } else { ($calculatorInventory | ConvertTo-Json -Compress -Depth 5) })
if (-not $calculatorInventoryOk) {
    throw "Expected calculator to run only through the current-boot UI Wasm service"
}

foreach ($key in @("1", "2", "shift-equal", "3", "0", "equal")) {
    $inputOffset = Get-SerialLogOffset
    Send-GenesisUiKey -KeyName $key
    $updated = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "PERSONAL SHELL FRAME UPDATED sanitized_input" -Offset $inputOffset -TimeoutSeconds $TimeoutSeconds
    Add-Predicate -Name "genesis-ui:calculator-input-$($key.Replace('shift-', 'shift_'))" -Expected "physical calculator key '$key' updates the Wasm-rendered program frame" -Passed $updated -Actual $(if ($updated) { "frame updated" } else { Get-SerialLogTail -Path $SerialLog })
    if (-not $updated) {
        throw "Expected calculator input '$key' to update the personal frame"
    }
}
Start-Sleep -Milliseconds 300
Save-QemuScreendump -Name "calculator-result-42" | Out-Null

Send-GenesisUiKey -KeyName "f12"
Assert-LogContains -Name "genesis-ui:calculator-f12-exit" -Needle "PERSONAL SHELL EXIT F12 genesis" -TimeoutSeconds $TimeoutSeconds
Start-Sleep -Milliseconds 300
Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "genesis-ui:calculator-f12-inventory"
$calculatorAfterF12Inventory = Get-LastAgentResponseJson -Method "service.inventory"
$calculatorAfterF12Personal = @($calculatorAfterF12Inventory.facts.services | Where-Object { $_.id -eq "svc.user.shell" })
$calculatorAfterF12Genesis = @($calculatorAfterF12Inventory.facts.services | Where-Object { $_.id -eq "core.ui.genesis" })
$calculatorAfterF12Ok = $calculatorAfterF12Personal.Count -eq 0 -and $calculatorAfterF12Genesis.Count -eq 1 -and $calculatorAfterF12Genesis[0].core_owned -eq $true -and $calculatorAfterF12Genesis[0].replaceable -eq $false
Add-Predicate -Name "genesis-ui:calculator-f12-restores-core-genesis" -Expected "F12 exits the calculator, removes svc.user.shell and restores immutable core Genesis" -Passed $calculatorAfterF12Ok -Actual $(if ($calculatorAfterF12Ok) { "core.ui.genesis only" } else { ($calculatorAfterF12Inventory.facts.services | ConvertTo-Json -Compress -Depth 5) })
if (-not $calculatorAfterF12Ok) {
    throw "Expected F12 to restore Genesis after the current-boot calculator"
}
Save-QemuScreendump -Name "genesis-after-calculator-f12" | Out-Null

$genesisEditorBase64 = @'
UlVJUAEAIACwAAAAAQMAAQEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIAAABABQACAAIAPAAGAAAAAAAcmFpT1MgRURJVCAgRjEyPUVYSVQEAAAACAAoAGACgAEAAAAAAwAFAAgAtAFgACQAAQAAAENMRUFSAAAAAQAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
'@ -replace '\s', ''
$genesisEditorBytes = [Convert]::FromBase64String($genesisEditorBase64)
$genesisEditorSha256 = "34f726d13818d174e23ef0614ca183a2967b9449c8cf4447151aef13d277d815"
$genesisEditorHash = "sha256:$genesisEditorSha256"
$sha = [System.Security.Cryptography.SHA256]::Create()
try {
    $editorActualSha256 = ([BitConverter]::ToString($sha.ComputeHash($genesisEditorBytes)) -replace "-", "").ToLowerInvariant()
}
finally {
    $sha.Dispose()
}
$editorFixtureOk = $genesisEditorBytes.Length -eq 176 -and $editorActualSha256 -eq $genesisEditorSha256
Add-Predicate -Name "genesis-ui:editor-ruip-exact-fixture" -Expected "raios-core editor canonical RUIP is exactly 176 bytes with the pinned SHA-256" -Passed $editorFixtureOk -Actual "bytes=$($genesisEditorBytes.Length) sha256=$editorActualSha256"
if (-not $editorFixtureOk) {
    throw "Canonical editor RUIP fixture does not match its pinned identity"
}

$editorChunkCount = Send-GenesisUiProgramBytes -Bytes $genesisEditorBytes -NamePrefix "genesis-ui:editor-delivery"
Send-AgentCommand -Command "program.submit_finalize" -ExpectedMarker "RAIOS_AGENT_END program.submit_finalize" -Name "genesis-ui:editor-finalize"
$editorFinalize = (Get-LastAgentResponseJson -Method "program.submit_finalize").body.result
$editorFinalizeOk = (
    $editorFinalize.method -eq "program.submit_finalize" -and
    $editorFinalize.status -eq "ready" -and
    $editorFinalize.scope -eq "current_boot" -and
    $editorFinalize.classification -eq "local_only" -and
    $editorFinalize.retention -eq "current_boot_ram_only" -and
    $editorFinalize.accepted -eq $true -and
    $editorFinalize.rejected -eq $false -and
    $editorFinalize.reason -eq "retained_current_boot_inert_ui_program" -and
    [int]$editorFinalize.attempted_byte_len -eq 176 -and
    $editorFinalize.present -eq $true -and
    [int]$editorFinalize.revision -gt [int]$calculatorFinalize.revision -and
    [int]$editorFinalize.byte_len -eq 176 -and
    $editorFinalize.program_sha256 -eq $genesisEditorHash -and
    $editorFinalize.program_sha256 -ne $calculatorFinalize.program_sha256 -and
    $editorFinalize.source -eq "serial" -and
    $null -eq $editorFinalize.provider_request_id -and
    [int]$editorFinalize.serial_chunk_count -eq $editorChunkCount -and
    [int]$editorFinalize.pending_byte_len -eq 0 -and
    [int]$editorFinalize.pending_chunk_count -eq 0 -and
    $editorFinalize.signing_attempted -eq $false -and
    $editorFinalize.load_attempted -eq $false -and
    $editorFinalize.execution_attempted -eq $false -and
    $editorFinalize.authorizes_load -eq $false -and
    $editorFinalize.authorizes_execution -eq $false -and
    $editorFinalize.writes_persistent_state -eq $false
)
Add-Predicate -Name "genesis-ui:editor-retained-exact-current-boot-inert" -Expected "canonical editor RUIP replaces the calculator under its exact hash but cannot sign, load, execute or persist" -Passed $editorFinalizeOk -Actual $(if ($editorFinalizeOk) { "$genesisEditorHash revision=$($editorFinalize.revision) chunks=$editorChunkCount" } else { ($editorFinalize | ConvertTo-Json -Compress -Depth 5) })
if (-not $editorFinalizeOk) {
    throw "Expected exact canonical editor RUIP to replace the calculator inertly"
}

[byte[]]$malformedEditor = $genesisEditorBytes.Clone()
$malformedEditor[17] = 1
$malformedEditorChunkCount = Send-GenesisUiProgramBytes -Bytes $malformedEditor -NamePrefix "genesis-ui:editor-malformed-delivery"
Send-AgentCommand -Command "program.submit_finalize" -ExpectedMarker "RAIOS_AGENT_END program.submit_finalize" -Name "genesis-ui:editor-malformed-finalize"
$editorMalformedFinalize = (Get-LastAgentResponseJson -Method "program.submit_finalize").body.result
$editorMalformedFinalizeOk = (
    $editorMalformedFinalize.accepted -eq $false -and
    $editorMalformedFinalize.rejected -eq $true -and
    $editorMalformedFinalize.reason -eq "program_malformed" -and
    [int]$editorMalformedFinalize.attempted_byte_len -eq 176 -and
    $editorMalformedFinalize.status -eq "ready" -and
    $editorMalformedFinalize.present -eq $true -and
    [int]$editorMalformedFinalize.revision -eq [int]$editorFinalize.revision -and
    [int]$editorMalformedFinalize.byte_len -eq 176 -and
    $editorMalformedFinalize.program_sha256 -eq $genesisEditorHash -and
    [int]$editorMalformedFinalize.pending_byte_len -eq 0 -and
    [int]$editorMalformedFinalize.pending_chunk_count -eq 0 -and
    $editorMalformedFinalize.signing_attempted -eq $false -and
    $editorMalformedFinalize.load_attempted -eq $false -and
    $editorMalformedFinalize.execution_attempted -eq $false -and
    $editorMalformedFinalize.authorizes_load -eq $false -and
    $editorMalformedFinalize.authorizes_execution -eq $false -and
    $editorMalformedFinalize.writes_persistent_state -eq $false
)
Add-Predicate -Name "genesis-ui:editor-malformed-rejected-with-retained-editor-inert" -Expected "nonzero reserved RUIP byte rejects atomically and leaves the exact editor inert" -Passed $editorMalformedFinalizeOk -Actual $(if ($editorMalformedFinalizeOk) { "rejected chunks=$malformedEditorChunkCount retained=$genesisEditorHash revision=$($editorFinalize.revision)" } else { ($editorMalformedFinalize | ConvertTo-Json -Compress -Depth 5) })
if (-not $editorMalformedFinalizeOk) {
    throw "Expected malformed RUIP to reject without replacing or activating the editor"
}

Send-AgentCommand -Command "program.workspace" -ExpectedMarker "RAIOS_AGENT_END program.workspace" -Name "genesis-ui:editor-workspace-after-malformed"
$editorWorkspaceAfterMalformed = (Get-LastAgentResponseJson -Method "program.workspace").body.result
$editorWorkspaceAfterMalformedOk = (
    $editorWorkspaceAfterMalformed.status -eq "ready" -and
    $editorWorkspaceAfterMalformed.present -eq $true -and
    [int]$editorWorkspaceAfterMalformed.revision -eq [int]$editorFinalize.revision -and
    [int]$editorWorkspaceAfterMalformed.byte_len -eq 176 -and
    $editorWorkspaceAfterMalformed.program_sha256 -eq $genesisEditorHash -and
    [int]$editorWorkspaceAfterMalformed.pending_byte_len -eq 0 -and
    [int]$editorWorkspaceAfterMalformed.pending_chunk_count -eq 0 -and
    $editorWorkspaceAfterMalformed.load_attempted -eq $false -and
    $editorWorkspaceAfterMalformed.execution_attempted -eq $false -and
    $editorWorkspaceAfterMalformed.authorizes_load -eq $false -and
    $editorWorkspaceAfterMalformed.authorizes_execution -eq $false -and
    $editorWorkspaceAfterMalformed.writes_persistent_state -eq $false
)
Add-Predicate -Name "genesis-ui:editor-workspace-unchanged-after-malformed" -Expected "malformed delivery cannot change the retained editor hash, revision, pending state or authority" -Passed $editorWorkspaceAfterMalformedOk -Actual $(if ($editorWorkspaceAfterMalformedOk) { "$genesisEditorHash revision=$($editorFinalize.revision) unchanged" } else { ($editorWorkspaceAfterMalformed | ConvertTo-Json -Compress -Depth 5) })
if (-not $editorWorkspaceAfterMalformedOk) {
    throw "Expected malformed RUIP delivery to leave the editor workspace unchanged"
}

Save-QemuScreendump -Name "editor-awaiting-physical-approval" | Out-Null
Send-QemuAbsolutePointerClick -X 27017 -Y 6559
$editorActivationMarker = "PROGRAM_CURRENT_BOOT_ACTIVATION physical_approval=pointer program_sha256=$genesisEditorHash engine=svc.user.shell capability_surface=ui_only wasm=true result=accepted"
Assert-LogContains -Name "genesis-ui:editor-physical-approval-wasm-accepted" -Needle $editorActivationMarker -TimeoutSeconds $TimeoutSeconds
Start-Sleep -Milliseconds 300
Save-QemuScreendump -Name "editor-active-after-physical-approval" | Out-Null

Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "genesis-ui:editor-current-boot-inventory"
$editorInventoryResponse = Get-LastAgentResponseJson -Method "service.inventory"
$editorInventory = @($editorInventoryResponse.facts.services | Where-Object { $_.id -eq "svc.user.shell" })
$editorInventoryOk = (
    $editorInventory.Count -eq 1 -and
    $editorInventory[0].kind -eq "service" -and
    $editorInventory[0].scope -eq "current_boot" -and
    $editorInventory[0].persistence -eq "none" -and
    $editorInventory[0].capability_envelope -eq "wasmi_linker_import_surface" -and
    $editorInventory[0].host_import_count -eq 6 -and
    $editorInventory[0].running -eq $true
)
Add-Predicate -Name "genesis-ui:editor-runs-only-as-ui-current-boot-service" -Expected "physical approval starts svc.user.shell only as a current-boot Wasmi UI service" -Passed $editorInventoryOk -Actual $(if ($editorInventoryOk) { "svc.user.shell current_boot ui_only running" } else { ($editorInventory | ConvertTo-Json -Compress -Depth 5) })
if (-not $editorInventoryOk) {
    throw "Expected editor to run only through the current-boot UI Wasm service"
}

foreach ($key in @("h", "i")) {
    $inputOffset = Get-SerialLogOffset
    Send-GenesisUiKey -KeyName $key
    $updated = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "PERSONAL SHELL FRAME UPDATED sanitized_input" -Offset $inputOffset -TimeoutSeconds $TimeoutSeconds
    Add-Predicate -Name "genesis-ui:editor-input-$key" -Expected "physical editor key '$key' updates the Wasm-rendered program frame" -Passed $updated -Actual $(if ($updated) { "frame updated" } else { Get-SerialLogTail -Path $SerialLog })
    if (-not $updated) {
        throw "Expected editor input '$key' to update the personal frame"
    }
}
Start-Sleep -Milliseconds 300
Save-QemuScreendump -Name "editor-content-hi" | Out-Null

foreach ($key in @("ret", "r")) {
    $inputOffset = Get-SerialLogOffset
    Send-GenesisUiKey -KeyName $key
    $updated = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "PERSONAL SHELL FRAME UPDATED sanitized_input" -Offset $inputOffset -TimeoutSeconds $TimeoutSeconds
    Add-Predicate -Name "genesis-ui:editor-input-$key" -Expected "physical editor key '$key' updates the Wasm-rendered program frame" -Passed $updated -Actual $(if ($updated) { "frame updated" } else { Get-SerialLogTail -Path $SerialLog })
    if (-not $updated) {
        throw "Expected editor input '$key' to update the personal frame"
    }
}
Start-Sleep -Milliseconds 300
Save-QemuScreendump -Name "editor-content-second-line" | Out-Null

$inputOffset = Get-SerialLogOffset
Send-GenesisUiKey -KeyName "backspace"
$updated = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "PERSONAL SHELL FRAME UPDATED sanitized_input" -Offset $inputOffset -TimeoutSeconds $TimeoutSeconds
Add-Predicate -Name "genesis-ui:editor-input-backspace" -Expected "physical editor key 'backspace' updates the Wasm-rendered program frame" -Passed $updated -Actual $(if ($updated) { "frame updated" } else { Get-SerialLogTail -Path $SerialLog })
if (-not $updated) {
    throw "Expected editor input 'backspace' to update the personal frame"
}
Start-Sleep -Milliseconds 300
Save-QemuScreendump -Name "editor-after-backspace" | Out-Null

$clearOffset = Get-SerialLogOffset
# The CLEAR button rect is fixed program data (8,436)-(104,472) in PROGRAM
# coordinates. The kernel localizes pointer input for personal programs as
# program = physical/LOGICAL_SCALE - personal_surface origin, with
# personal_surface at logical (0, SECURE_STRIP_HEIGHT) (raios-core
# genesis_layout.rs: LOGICAL_SCALE=2, SECURE_STRIP_HEIGHT=38). QMP absolute
# events are normalized 0..32767 over the LIVE framebuffer size from the
# snapshot, so invert the whole chain instead of assuming a resolution.
$editorFramebufferDetail = [string]$genesisSystem.details.framebuffer.detail
if ($editorFramebufferDetail -notmatch '^(\d+)x(\d+)\b') {
    throw "Cannot derive framebuffer size for the editor CLEAR click from snapshot detail '$editorFramebufferDetail'"
}
$editorFbWidth = [int]$Matches[1]
$editorFbHeight = [int]$Matches[2]
$editorClearPhysicalX = 2 * 56
$editorClearPhysicalY = 2 * (454 + 38)
$editorClearX = [int][Math]::Round($editorClearPhysicalX * 32767 / $editorFbWidth)
$editorClearY = [int][Math]::Round($editorClearPhysicalY * 32767 / $editorFbHeight)
Send-QemuAbsolutePointerClick -X $editorClearX -Y $editorClearY
$clearUpdated = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "PERSONAL SHELL FRAME UPDATED sanitized_input" -Offset $clearOffset -TimeoutSeconds $TimeoutSeconds
Add-Predicate -Name "genesis-ui:editor-clear-click-updates-frame" -Expected "physical CLEAR click is a sanitized pointer input that updates the Wasm-rendered program frame" -Passed $clearUpdated -Actual $(if ($clearUpdated) { "frame updated" } else { Get-SerialLogTail -Path $SerialLog })
if (-not $clearUpdated) {
    throw "Expected editor CLEAR click to update the personal frame"
}
Start-Sleep -Milliseconds 300
Save-QemuScreendump -Name "editor-after-clear" | Out-Null

Send-GenesisUiKey -KeyName "f12"
Assert-LogContains -Name "genesis-ui:editor-f12-exit" -Needle "PERSONAL SHELL EXIT F12 genesis" -TimeoutSeconds $TimeoutSeconds
Start-Sleep -Milliseconds 300
Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "genesis-ui:editor-f12-inventory"
$editorAfterF12Inventory = Get-LastAgentResponseJson -Method "service.inventory"
$editorAfterF12Personal = @($editorAfterF12Inventory.facts.services | Where-Object { $_.id -eq "svc.user.shell" })
$editorAfterF12Genesis = @($editorAfterF12Inventory.facts.services | Where-Object { $_.id -eq "core.ui.genesis" })
$editorAfterF12Ok = $editorAfterF12Personal.Count -eq 0 -and $editorAfterF12Genesis.Count -eq 1 -and $editorAfterF12Genesis[0].core_owned -eq $true -and $editorAfterF12Genesis[0].replaceable -eq $false
Add-Predicate -Name "genesis-ui:editor-f12-restores-core-genesis" -Expected "F12 exits the editor, removes svc.user.shell and restores immutable core Genesis" -Passed $editorAfterF12Ok -Actual $(if ($editorAfterF12Ok) { "core.ui.genesis only" } else { ($editorAfterF12Inventory.facts.services | ConvertTo-Json -Compress -Depth 5) })
if (-not $editorAfterF12Ok) {
    throw "Expected F12 to restore Genesis after the current-boot editor"
}
Save-QemuScreendump -Name "genesis-after-editor-f12" | Out-Null

$genesisBeforePersonalProof = Save-QemuScreendump -Name "genesis-context-diagnostics"
Send-AgentCommand -Command "ui.personal_shell_proof" -ExpectedMarker "RAIOS_AGENT_END ui.personal_shell_proof" -Name "genesis-ui:personal-shell-proof"
$personalShellResponse = Get-LastAgentResponseJson -Method "ui.personal_shell_proof"
$personalShell = $personalShellResponse.body.result
$personalShellImports = @($personalShell.authorized_imports | ForEach-Object { "$($_.module).$($_.name)" })
$expectedPersonalShellImports = @(
    "ui.viewport",
    "ui.context_len",
    "ui.context_read",
    "ui.input_len",
    "ui.input_read",
    "ui.frame_submit"
)
$personalShellProofOk = (
    $personalShell.schema -eq "raios.personal_shell_proof.v0" -and
    $personalShell.scope -eq "current_boot" -and
    $personalShell.classification -eq "local_only" -and
    $personalShell.test_infrastructure -eq $true -and
    $personalShell.non_default -eq $true -and
    $personalShell.activation_mode -eq "normal" -and
    $personalShell.activation_requested -eq $true -and
    $personalShell.activation_request_reason -eq "queued_for_core_owned_shell_host" -and
    $personalShell.service_id -eq "svc.user.shell" -and
    $personalShell.trust_tier -eq "dev_key_not_owner_sealed" -and
    $personalShell.owner_sealed -eq $false -and
    $personalShell.artifact_validation_ok -eq $true -and
    $personalShell.authorized_import_count -eq 6 -and
    $personalShell.linked_host_import_count -eq 6 -and
    (($personalShellImports -join "|") -eq ($expectedPersonalShellImports -join "|")) -and
    $personalShell.fuel_budget -eq 250000 -and
    $personalShell.normal.accepted -eq $true -and
    $personalShell.normal.instantiation_error_kind -eq "none" -and
    $personalShell.normal.run_outcome -eq "success" -and
    $personalShell.sanitized_input.accepted -eq $true -and
    $personalShell.sanitized_input.instantiation_error_kind -eq "none" -and
    $personalShell.sanitized_input.run_outcome -eq "success" -and
    $personalShell.frame_changed_after_sanitized_input -eq $true -and
    $personalShell.malformed_frame_rejected_atomically -eq $true -and
    $personalShell.malformed_frame.instantiation_error_kind -eq "none" -and
    $personalShell.malformed_frame.run_outcome -eq "frame_rejected" -and
    $personalShell.clipped_overdraw_proved -eq $true -and
    $personalShell.clipped_overdraw.instantiation_error_kind -eq "none" -and
    $personalShell.guest_trap_rejected -eq $true -and
    $personalShell.guest_trap.instantiation_error_kind -eq "none" -and
    $personalShell.fuel_exhaustion_rejected -eq $true -and
    $personalShell.fuel_exhaustion.instantiation_error_kind -eq "none" -and
    $personalShell.missing_frame_submit_linker_denial -eq "linker_implementation_subset" -and
    $personalShell.broader_import_denial -eq "personal_shell_import_superset" -and
    $personalShell.generic_loader_used -eq $false -and
    $personalShell.accepts_external_artifact_bytes -eq $false -and
    $personalShell.authorizes_external_artifact_intake -eq $false -and
    $personalShell.authorizes_arbitrary_shell_artifacts -eq $false -and
    $personalShell.authorizes_persistent_install -eq $false -and
    $personalShell.writes_persistent_state -eq $false -and
    $personalShell.authorizes_provider_access -eq $false -and
    $personalShell.authorizes_provider_export -eq $false -and
    $personalShell.authorizes_secret_access -eq $false -and
    $personalShell.authorizes_secret_plaintext -eq $false -and
    $personalShell.authorizes_network_access -eq $false -and
    $personalShell.authorizes_recovery_access -eq $false -and
    $personalShell.authorizes_capability_decision -eq $false -and
    $personalShell.authorizes_raw_framebuffer_access -eq $false -and
    $personalShell.authorizes_broader_mutation -eq $false -and
    $personalShell.persistent_service_install -eq $false -and
    $personalShell.service_inventory_change -eq "pending_current_boot_activation" -and
    $personalShell.evidence_complete -eq $true
)
Add-Predicate -Name "genesis-ui:personal-shell-proof" -Expected "the signed current-boot personal shell renders through only six UI imports, rejects malformed/broader paths, and grants no broader authority" -Passed $personalShellProofOk -Actual $(if ($personalShellProofOk) { "artifact=$($personalShell.artifact_sha256) imports=$($personalShell.authorized_import_count)" } else { ($personalShell | ConvertTo-Json -Compress -Depth 6) })
if (-not $personalShellProofOk) {
    throw "Expected the signed bounded personal-shell proof path to hold every I2 boundary"
}

Assert-LogContains -Name "genesis-ui:personal-shell-active" -Needle "PERSONAL SHELL ACTIVE current_boot proof" -TimeoutSeconds $TimeoutSeconds
Start-Sleep -Milliseconds 300
Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "genesis-ui:personal-shell-inventory"
$personalShellInventoryResponse = Get-LastAgentResponseJson -Method "service.inventory"
$personalShellInventory = @($personalShellInventoryResponse.facts.services | Where-Object { $_.id -eq "svc.user.shell" })
$personalShellInventoryOk = (
    $personalShellInventory.Count -eq 1 -and
    $personalShellInventory[0].kind -eq "service" -and
    $personalShellInventory[0].scope -eq "current_boot" -and
    $personalShellInventory[0].persistence -eq "none" -and
    # P4-9b moved trust-tier facts out of the inventory rows (a catalog read must
    # not carry authority claims); the dev-key tier is pinned by the
    # ui.personal_shell_proof response above. The row proves the runtime posture:
    $personalShellInventory[0].core_owned -eq $false -and
    $personalShellInventory[0].replaceable -eq $true -and
    $personalShellInventory[0].capability_envelope -eq "wasmi_linker_import_surface" -and
    $personalShellInventory[0].last_lifecycle_reason -eq "running" -and
    $personalShellInventory[0].host_import_count -eq 6 -and
    $personalShellInventory[0].running -eq $true
)
Add-Predicate -Name "genesis-ui:personal-shell-dynamic-inventory" -Expected "svc.user.shell appears only as a running current-boot signed proof service" -Passed $personalShellInventoryOk -Actual $(if ($personalShellInventoryOk) { "current_boot running" } else { ($personalShellInventory | ConvertTo-Json -Compress -Depth 6) })
if (-not $personalShellInventoryOk) {
    throw "Expected the running personal proof to be the only dynamic svc.user.shell inventory row"
}

$personalProofActive = Save-QemuScreendump -Name "personal-proof-active"
Send-QemuMonitorCommand -Command "sendkey a" | Out-Null
Assert-LogContains -Name "genesis-ui:personal-shell-sanitized-input" -Needle "PERSONAL SHELL FRAME UPDATED sanitized_input" -TimeoutSeconds $TimeoutSeconds
Start-Sleep -Milliseconds 300
$personalProofAfterInput = Save-QemuScreendump -Name "personal-proof-after-input"
$personalSecureStripOk = (
    $genesisBeforePersonalProof.secure_strip_sha256 -eq $personalProofActive.secure_strip_sha256 -and
    $personalProofActive.secure_strip_sha256 -eq $personalProofAfterInput.secure_strip_sha256
)
Add-Predicate -Name "genesis-ui:personal-shell-cannot-overdraw-secure-strip" -Expected "Genesis secure strip pixels stay byte-identical before, during and after personal-shell input" -Passed $personalSecureStripOk -Actual $(if ($personalSecureStripOk) { $personalProofActive.secure_strip_sha256 } else { "before=$($genesisBeforePersonalProof.secure_strip_sha256) active=$($personalProofActive.secure_strip_sha256) input=$($personalProofAfterInput.secure_strip_sha256)" })
if (-not $personalSecureStripOk) {
    throw "Expected the personal frame to leave Genesis secure-strip pixels unchanged"
}

Send-QemuMonitorCommand -Command "sendkey f12" | Out-Null
Assert-LogContains -Name "genesis-ui:personal-shell-f12-exit" -Needle "PERSONAL SHELL EXIT F12 genesis" -TimeoutSeconds $TimeoutSeconds
Start-Sleep -Milliseconds 300
Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "genesis-ui:personal-shell-f12-inventory"
$afterF12Inventory = Get-LastAgentResponseJson -Method "service.inventory"
$afterF12Personal = @($afterF12Inventory.facts.services | Where-Object { $_.id -eq "svc.user.shell" })
$afterF12Genesis = @($afterF12Inventory.facts.services | Where-Object { $_.id -eq "core.ui.genesis" })
$afterF12Ok = $afterF12Personal.Count -eq 0 -and $afterF12Genesis.Count -eq 1 -and $afterF12Genesis[0].core_owned -eq $true -and $afterF12Genesis[0].replaceable -eq $false
Add-Predicate -Name "genesis-ui:personal-shell-f12-removes-dynamic-inventory" -Expected "F12 returns to core Genesis and removes the current-boot personal row" -Passed $afterF12Ok -Actual $(if ($afterF12Ok) { "core.ui.genesis only" } else { ($afterF12Inventory.facts.services | ConvertTo-Json -Compress -Depth 5) })
if (-not $afterF12Ok) {
    throw "Expected F12 to restore Genesis without a personal service inventory row"
}
Save-QemuScreendump -Name "genesis-after-f12" | Out-Null

Send-AgentCommand -Command "ui.personal_shell_proof trap" -ExpectedMarker "RAIOS_AGENT_END ui.personal_shell_proof" -Name "genesis-ui:personal-shell-trap-request"
$personalTrapResponse = Get-LastAgentResponseJson -Method "ui.personal_shell_proof"
$personalTrap = $personalTrapResponse.body.result
$personalTrapRequestOk = $personalTrap.activation_mode -eq "trap" -and $personalTrap.activation_requested -eq $true -and $personalTrap.activation_request_reason -eq "queued_for_core_owned_shell_host"
Add-Predicate -Name "genesis-ui:personal-shell-trap-request-bounded" -Expected "only the named built-in trap proof mode is queued; no raw packet input is accepted" -Passed $personalTrapRequestOk -Actual $(if ($personalTrapRequestOk) { "trap queued" } else { ($personalTrap | ConvertTo-Json -Compress -Depth 5) })
if (-not $personalTrapRequestOk) {
    throw "Expected the typed built-in trap mode to queue exactly once"
}
Assert-LogContains -Name "genesis-ui:personal-shell-trap-fallback" -Needle "PERSONAL SHELL FALLBACK trap" -TimeoutSeconds $TimeoutSeconds
Start-Sleep -Milliseconds 300
Save-QemuScreendump -Name "genesis-after-personal-trap" | Out-Null

Send-AgentCommand -Command "ui.personal_shell_proof fuel" -ExpectedMarker "RAIOS_AGENT_END ui.personal_shell_proof" -Name "genesis-ui:personal-shell-fuel-request"
$personalFuelResponse = Get-LastAgentResponseJson -Method "ui.personal_shell_proof"
$personalFuel = $personalFuelResponse.body.result
$personalFuelRequestOk = $personalFuel.activation_mode -eq "fuel" -and $personalFuel.activation_requested -eq $true -and $personalFuel.activation_request_reason -eq "queued_for_core_owned_shell_host"
Add-Predicate -Name "genesis-ui:personal-shell-fuel-request-bounded" -Expected "only the named built-in fuel proof mode is queued; no raw packet input is accepted" -Passed $personalFuelRequestOk -Actual $(if ($personalFuelRequestOk) { "fuel queued" } else { ($personalFuel | ConvertTo-Json -Compress -Depth 5) })
if (-not $personalFuelRequestOk) {
    throw "Expected the typed built-in fuel mode to queue exactly once"
}
Assert-LogContains -Name "genesis-ui:personal-shell-fuel-fallback" -Needle "PERSONAL SHELL FALLBACK fuel_exhausted" -TimeoutSeconds $TimeoutSeconds
Start-Sleep -Milliseconds 300
Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "genesis-ui:personal-shell-fallback-inventory"
$fallbackInventory = Get-LastAgentResponseJson -Method "service.inventory"
$fallbackPersonal = @($fallbackInventory.facts.services | Where-Object { $_.id -eq "svc.user.shell" })
$fallbackInventoryOk = $fallbackPersonal.Count -eq 0
Add-Predicate -Name "genesis-ui:personal-shell-fallback-removes-dynamic-inventory" -Expected "trap and fuel fallback leave no personal current-boot inventory row" -Passed $fallbackInventoryOk -Actual $(if ($fallbackInventoryOk) { "absent" } else { ($fallbackPersonal | ConvertTo-Json -Compress -Depth 5) })
if (-not $fallbackInventoryOk) {
    throw "Expected trap/fuel fallback to remove the personal inventory row"
}
Send-AgentCommand -Command "agent recovery.snapshot" -ExpectedMarker "RAIOS_AGENT_END recovery.snapshot" -Name "genesis-ui:recovery-after-personal-fallback"
$recoveryAfterFallback = Get-LastAgentResponseJson -Method "recovery.snapshot"
$recoveryAfterFallbackOk = $recoveryAfterFallback.body.result.schema -eq "raios.recovery_snapshot.v0" -and $recoveryAfterFallback.body.result.lifeline_available -eq $true -and $recoveryAfterFallback.body.result.mutates_state -eq $false
Add-Predicate -Name "genesis-ui:recovery-callable-after-personal-fallback" -Expected "Genesis recovery remains the core-owned read-only lifeline after personal trap/fuel fallback" -Passed $recoveryAfterFallbackOk -Actual $(if ($recoveryAfterFallbackOk) { "lifeline available" } else { ($recoveryAfterFallback.body.result | ConvertTo-Json -Compress -Depth 5) })
if (-not $recoveryAfterFallbackOk) {
    throw "Expected Recovery to remain callable after the personal-shell fallback"
}
Save-QemuScreendump -Name "genesis-after-personal-fallback" | Out-Null

# Snapshot the predicate list BEFORE the B1.3 tail so the compatibility check
# can prove every pre-existing predicate already passed. ToArray() copies the
# generic List safely (PS 5.1 @() on the live List raised ArgumentException).
$preB13Predicates = $Predicates.ToArray()
$readyLog = Get-SerialLogContent -Path $SerialLog
$editorReadyLines = @($readyLog -split '\r?\n' | Where-Object {
    $_.StartsWith("PROGRAM_INSTALL_READY result=accepted physical_approval=genesis_pointer program_sha256=$genesisEditorHash ", [System.StringComparison]::Ordinal)
} | ForEach-Object { $_.TrimEnd() })
$editorReadyLine = if ($editorReadyLines.Count -eq 1) { [string]$editorReadyLines[0] } else { "" }
$editorReadyPattern = '^PROGRAM_INSTALL_READY result=accepted physical_approval=genesis_pointer program_sha256=(sha256:[0-9a-f]{64}) activation_approval_sha256=(sha256:[0-9a-f]{64}) engine=(svc\.user\.shell) persistence_authority=(false) reason=(program_current_boot_approved)$'
$editorReadyMatch = [regex]::Match($editorReadyLine, $editorReadyPattern)
$editorActivationApprovalSha256 = if ($editorReadyMatch.Success) { [string]$editorReadyMatch.Groups[2].Value } else { "" }
$editorActivationOffset = $readyLog.IndexOf($editorActivationMarker, [System.StringComparison]::Ordinal)
$editorReadyOffset = $readyLog.IndexOf($editorReadyLine, [System.StringComparison]::Ordinal)

$editorInstall = @(Invoke-SignedUiProgramInstall -ProgramSha256 $genesisEditorHash -ActivationApprovalSha256 $editorActivationApprovalSha256 -NamePrefix "genesis-ui:editor")[-1]
$editorPreview = $editorInstall.Preview
$editorSignedPreview = $editorInstall.SignedPreview
$editorInstallDenial = $editorInstall.Denial
$editorArtifactRecord = $editorInstall.InstalledArtifactRecord

Send-AgentCommand -Command "agent module.loader_runtime" -ExpectedMarker "RAIOS_AGENT_END module.loader_runtime" -Name "genesis-ui:b13-loader-runtime"
$b13LoaderResponse = Get-LastAgentResponseJson -Method "module.loader_runtime"
$b13LoaderEvidence = @($b13LoaderResponse.evidence)
$b13LoaderApproval = @($b13LoaderResponse.evidence | Where-Object id -eq "local_approval_reference")[0]
Send-AgentCommand -Command "agent module.service_slot_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic" -Name "genesis-ui:b13-service-slot"
$b13SlotResponse = Get-LastAgentResponseJson -Method "module.service_slot_diagnostic"
Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "genesis-ui:b13-inventory"
$b13InventoryResponse = Get-LastAgentResponseJson -Method "service.inventory"
$b13InventoryRows = @($b13InventoryResponse.facts.services)
Send-AgentCommand -Command "program.workspace" -ExpectedMarker "RAIOS_AGENT_END program.workspace" -Name "genesis-ui:b13-program-workspace"
$b13ProgramWorkspaceResponse = Get-LastAgentResponseJson -Method "program.workspace"

$editorReadyExpected = "PROGRAM_INSTALL_READY result=accepted physical_approval=genesis_pointer program_sha256=$genesisEditorHash activation_approval_sha256=$editorActivationApprovalSha256 engine=svc.user.shell persistence_authority=false reason=program_current_boot_approved"
$editorReadyOk = $editorReadyMatch.Success -and
    $editorReadyLine -eq $editorReadyExpected -and
    $editorReadyMatch.Groups[1].Value -eq $genesisEditorHash -and
    $editorReadyMatch.Groups[3].Value -eq "svc.user.shell" -and
    $editorReadyMatch.Groups[4].Value -eq "false" -and
    $editorActivationOffset -ge 0 -and $editorReadyOffset -gt $editorActivationOffset -and
    [int64]$editorInstall.PreArtifactScan.artifact_persist_record_count -eq 0 -and
    [int64]$editorInstall.PreArtifactScan.ui_program_persist_record_count -eq 0 -and
    [int64]$editorInstall.PreArtifactScan.garbage_blob_count -eq 0 -and
    -not $editorInstall.PreInstallLog.Contains("PROGRAM_INSTALL_COMMIT")
    # Note: physically running the editor legitimately records ONE durable
    # import-grant memory record, so pre-install RECLOG count is 1, not 0; the
    # binding invariant is the absence of any install artifact, pinned above.
$editorReadyDump = [ordered]@{ activation_marker = $editorActivationMarker; activation_offset = $editorActivationOffset; ready_marker = $editorReadyLine; ready_offset = $editorReadyOffset; pre_reclog = $editorInstall.PreReclogResponse; pre_artstor = $editorInstall.PreArtifactResponse }
Add-Predicate -Name "genesis-ui:editor-install-ready-exact-physical-binding" -Expected "the exact merged PROGRAM_INSTALL_READY line binds the physically run 176-byte editor to its activation and svc.user.shell without persistence authority or prior RECLOG/ARTSTOR mutation" -Passed $editorReadyOk -Actual $(if ($editorReadyOk) { $editorReadyLine } else { $editorReadyDump | ConvertTo-Json -Compress -Depth 16 })

$unsignedPreviewExpected = "PROJECT_INSTALL_PREVIEW kind=install result=accepted signature_verified=false action_signature_message_sha256=$($editorPreview.action_signature_message_sha256) physical_approval_sha256=$($editorPreview.physical_approval_sha256) generation=$($editorPreview.generation) sequence=$($editorPreview.log_sequence) approval=owner_signature_required"
$editorPrepareOk = $editorPreview.status -eq "accepted" -and
    $editorPreview.reason -eq "project_install_signature_required" -and
    $editorPreview.accepted -eq $true -and $editorPreview.rejected -eq $false -and
    $editorPreview.service_id -eq "svc.user.shell" -and
    $editorPreview.phase -eq "pending_owner_signature" -and
    $editorPreview.action_kind -eq "install" -and $editorPreview.signature_verified -eq $false -and
    $editorPreview.install_source -eq "ui_program" -and
    $editorPreview.receipt_kind -eq "ruip_canonical" -and $editorPreview.w4_project_receipt_present -eq $false -and
    $editorPreview.candidate_sha256 -eq $genesisEditorHash -and
    $editorPreview.receipt_sha256 -eq $genesisEditorHash -and
    $editorPreview.activation_approval_sha256 -eq $editorActivationApprovalSha256 -and
    $editorPreview.install_envelope_sha256 -match '^sha256:[0-9a-f]{64}$' -and
    $editorPreview.promotion_transaction_sha256 -eq $null -and
    $editorPreview.artifact_persist_frame_sha256 -eq $null -and
    $editorPreview.writes_persistent_state -eq $false -and
    $editorInstall.UnsignedPreviewMarker -eq $unsignedPreviewExpected
Add-Predicate -Name "genesis-ui:editor-w6-prepare-binds-approved-ruip" -Expected "ui_program/ruip_canonical W6 prepare binds svc.user.shell, the exact editor and activation, no W4 receipt, the unchanged unsigned PROJECT_INSTALL_PREVIEW shape, and no write" -Passed $editorPrepareOk -Actual $(if ($editorPrepareOk) { "editor RUIP activation bound; owner signature required" } else { $editorInstall.PrepareResponse | ConvertTo-Json -Compress -Depth 16 })

$signedPreviewExpected = "PROJECT_INSTALL_PREVIEW kind=install result=accepted signature_verified=true action_signature_message_sha256=$($editorSignedPreview.action_signature_message_sha256) physical_approval_sha256=$($editorSignedPreview.physical_approval_sha256) generation=$($editorSignedPreview.generation) sequence=$($editorSignedPreview.log_sequence) approval=genesis_pointer_required"
$editorSignatureOk = $editorSignedPreview.status -eq "accepted" -and
    $editorSignedPreview.reason -eq "project_install_pending_physical_pointer_approval" -and
    $editorSignedPreview.phase -eq "pending_physical_pointer_approval" -and
    $editorSignedPreview.signature_verified -eq $true -and
    $editorSignedPreview.action_signature_message_sha256 -eq $editorPreview.action_signature_message_sha256 -and
    $editorSignedPreview.action_signature_message_sha256 -match '^sha256:[0-9a-f]{64}$' -and
    $editorSignedPreview.action_signature_message_sha256 -ne $editorActivationApprovalSha256 -and
    $editorSignedPreview.physical_approval_sha256 -match '^sha256:[0-9a-f]{64}$' -and
    $editorSignedPreview.install_envelope_sha256 -eq $editorPreview.install_envelope_sha256 -and
    $editorSignedPreview.writes_persistent_state -eq $false -and
    $editorInstall.SignedPreviewMarker -eq $signedPreviewExpected
$editorSignatureDump = [ordered]@{ activation = $editorActivationApprovalSha256; unsigned = $editorInstall.PrepareResponse; signed = $editorInstall.SignatureResponse; preview_markers = @($editorInstall.UnsignedPreviewMarker, $editorInstall.SignedPreviewMarker) }
Add-Predicate -Name "genesis-ui:editor-w6-signature-separate-authority" -Expected "the Rust signer arms the unchanged signed PROJECT_INSTALL_PREVIEW; its W6 action digest differs from the RUIP activation and still requires Genesis" -Passed $editorSignatureOk -Actual $(if ($editorSignatureOk) { "activation=$editorActivationApprovalSha256 w6=$($editorSignedPreview.action_signature_message_sha256)" } else { $editorSignatureDump | ConvertTo-Json -Compress -Depth 16 })

$editorSerialDenialOk = $editorInstallDenial.status -eq "denied" -and
    $editorInstallDenial.reason -eq "project_install_physical_pointer_approval_required" -and
    $editorInstallDenial.accepted -eq $false -and $editorInstallDenial.rejected -eq $true -and
    $editorInstallDenial.phase -eq "pending_physical_pointer_approval" -and
    $editorInstallDenial.signature_verified -eq $true -and
    $editorInstallDenial.action_signature_message_sha256 -eq $editorSignedPreview.action_signature_message_sha256 -and
    $editorInstallDenial.physical_approval_sha256 -eq $editorSignedPreview.physical_approval_sha256 -and
    $editorInstallDenial.install_envelope_sha256 -eq $editorSignedPreview.install_envelope_sha256 -and
    [int64]$editorInstall.DeniedReclogScan.count -eq [int64]$editorInstall.PreReclogScan.count -and
    $editorInstall.DeniedReclogScan.head_frame_sha256 -eq $editorInstall.PreReclogScan.head_frame_sha256 -and
    $editorInstall.DeniedReclogScan.tail_frame_sha256 -eq $editorInstall.PreReclogScan.tail_frame_sha256 -and
    [int64]$editorInstall.DeniedArtifactScan.artifact_persist_record_count -eq [int64]$editorInstall.PreArtifactScan.artifact_persist_record_count -and
    [int64]$editorInstall.DeniedArtifactScan.garbage_blob_count -eq [int64]$editorInstall.PreArtifactScan.garbage_blob_count -and
    -not $editorInstall.BeforeClickLog.Contains("PROGRAM_INSTALL_COMMIT")
$editorSerialDenialDump = [ordered]@{ denial = $editorInstall.DenialResponse; before_reclog = $editorInstall.PreReclogResponse; denied_reclog = $editorInstall.DeniedReclogResponse; before_artstor = $editorInstall.PreArtifactResponse; denied_artstor = $editorInstall.DeniedArtifactResponse; serial = $editorInstall.BeforeClickLog }
Add-Predicate -Name "genesis-ui:editor-serial-install-approval-denied-zero-effect" -Expected "the typed response denies serial approval with physical-pointer-required, retains the signed preview, changes no RECLOG/ARTSTOR fact, and emits no denied marker" -Passed $editorSerialDenialOk -Actual $(if ($editorSerialDenialOk) { "denied response pinned; RECLOG/ARTSTOR unchanged" } else { $editorSerialDenialDump | ConvertTo-Json -Compress -Depth 16 })

$editorSecondClickOk = $editorInstall.ClickCount -eq 1 -and
    $editorInstall.ProgramSha256 -eq $genesisEditorHash -and
    $editorInstall.ActivationApprovalSha256 -eq $editorActivationApprovalSha256 -and
    $editorInstall.InstallEnvelopeSha256 -eq $editorSignedPreview.install_envelope_sha256 -and
    $editorInstall.InstallActionSha256 -match '^sha256:[0-9a-f]{64}$' -and
    $editorInstall.PromotionTransactionSha256 -match '^sha256:[0-9a-f]{64}$' -and
    $editorInstall.ProgramPersistFrameSha256 -match '^sha256:[0-9a-f]{64}$' -and
    $editorInstall.Generation -eq [int64]$editorSignedPreview.generation -and
    $editorInstall.Sequence -eq [int64]$editorSignedPreview.log_sequence -and
    $editorInstall.Engine -eq "svc.user.shell" -and
    $editorInstall.GuestInstalled -eq $false -and $editorInstall.DurableWrites -eq $true -and
    [int64]$editorInstall.PostReclogScan.count -eq ([int64]$editorInstall.PreReclogScan.count + 3) -and
    # Three linked frames: authorization (pre_tail+1), promote = the marker
    # sequence (pre_tail+2), program-persist tail (pre_tail+3 = Sequence+1).
    $editorInstall.Sequence -eq ([int64]$editorInstall.PreReclogScan.tail_seq + 2) -and
    [int64]$editorInstall.PostReclogScan.tail_seq -eq ($editorInstall.Sequence + 1) -and
    $editorInstall.PostReclogScan.status -eq "valid" -and $editorInstall.PostReclogScan.valid_prefix_chain -eq $true -and
    (([int64]$editorInstall.PreReclogScan.count -eq 0 -and $editorInstall.PostReclogScan.head_frame_sha256 -match '^sha256:[0-9a-f]{64}$') -or
        ([int64]$editorInstall.PreReclogScan.count -gt 0 -and $editorInstall.PostReclogScan.head_frame_sha256 -eq $editorInstall.PreReclogScan.head_frame_sha256)) -and
    $editorInstall.PostReclogScan.tail_frame_sha256 -eq $editorInstall.ProgramPersistFrameSha256 -and
    # The UI-program record increments the dedicated scan field; the granted
    # artifact_persist_record_count stays unchanged (byte-identical for B1.2c).
    [int64]$editorInstall.PostArtifactScan.ui_program_persist_record_count -eq ([int64]$editorInstall.PreArtifactScan.ui_program_persist_record_count + 1) -and
    [int64]$editorInstall.PostArtifactScan.artifact_persist_record_count -eq [int64]$editorInstall.PreArtifactScan.artifact_persist_record_count -and
    $editorInstall.ActivationCountAfterClick -eq $editorInstall.ActivationCountBeforeClick -and
    $editorInstall.ShellActiveCountAfterClick -eq $editorInstall.ShellActiveCountBeforeClick
$editorSecondClickDump = [ordered]@{ marker = $editorInstall.MarkerLine; before_reclog = $editorInstall.PreReclogResponse; after_reclog = $editorInstall.PostReclogResponse; before_artstor = $editorInstall.PreArtifactResponse; after_artstor = $editorInstall.PostArtifactResponse; activation_before = $editorInstall.ActivationCountBeforeClick; activation_after = $editorInstall.ActivationCountAfterClick; shell_before = $editorInstall.ShellActiveCountBeforeClick; shell_after = $editorInstall.ShellActiveCountAfterClick }
Add-Predicate -Name "genesis-ui:editor-second-click-persists-without-rerun" -Expected "one additional Genesis click emits the exact merged PROGRAM_INSTALL_COMMIT chain, appends authorization/promote/program-persist plus one ARTSTOR record, reports guest_installed=false durable_writes=true, and does not reactivate or rerun the shell" -Passed $editorSecondClickOk -Actual $(if ($editorSecondClickOk) { $editorInstall.MarkerLine } else { $editorSecondClickDump | ConvertTo-Json -Compress -Depth 18 })

# The record comes from the dedicated ui_program_persist_records scan field, so
# its ui_program identity is implicit; the view exposes no subject_kind or own
# frame_sha256 (the persist frame is the RECLOG tail, pinned in the second-click
# predicate). It readback-verifies the exact 176-byte canonical editor payload.
$editorArtstorOk = $null -ne $editorArtifactRecord -and
    $editorArtifactRecord.canonical_program_sha256 -eq $genesisEditorHash -and
    [int64]$editorArtifactRecord.canonical_program_byte_len -eq 176 -and
    $editorArtifactRecord.activation_approval_sha256 -eq $editorActivationApprovalSha256 -and
    $editorArtifactRecord.install_envelope_sha256 -eq $editorInstall.InstallEnvelopeSha256 -and
    $editorArtifactRecord.install_authorization_frame_sha256 -match '^sha256:[0-9a-f]{64}$' -and
    $editorArtifactRecord.promotion_transaction_sha256 -eq $editorInstall.PromotionTransactionSha256 -and
    $editorArtifactRecord.artstor_blob_frame_sha256 -match '^sha256:[0-9a-f]{64}$' -and
    $editorArtifactRecord.present -eq $true -and $editorArtifactRecord.blob_hash_verified -eq $true -and
    $editorArtifactRecord.parsed_payload_sha256 -eq $genesisEditorHash
$editorArtstorDump = [ordered]@{ expected_program_sha256 = $genesisEditorHash; expected_byte_len = 176; marker = $editorInstall.MarkerLine; artifact_record = $editorArtifactRecord; artstor_scan = $editorInstall.PostArtifactResponse; reclog_scan = $editorInstall.PostReclogResponse }
Add-Predicate -Name "genesis-ui:editor-artstor-canonical-readback" -Expected "artifact.store_scan exposes the linked ui_program record and readback-verifies the exact 176-byte editor payload, blob frame, authorization, promote, and persist hashes" -Passed $editorArtstorOk -Actual $(if ($editorArtstorOk) { "program=$genesisEditorHash bytes=176 blob=$($editorArtifactRecord.artstor_blob_frame_sha256)" } else { $editorArtstorDump | ConvertTo-Json -Compress -Depth 18 })

$preB13Failures = @($preB13Predicates | Where-Object { -not $_.passed })
$ruipCompatibilityOk = $preB13Failures.Count -eq 0 -and
    $genesisCalculatorBytes.Length -eq 5372 -and $calculatorActualSha256 -eq $genesisCalculatorSha256 -and
    $genesisEditorBytes.Length -eq 176 -and $editorActualSha256 -eq $genesisEditorSha256 -and
    $calculatorFixtureOk -and $calculatorFinalizeOk -and $calculatorWorkspaceOk -and
    $malformedFinalizeOk -and $workspaceAfterMalformedOk -and $calculatorInventoryOk -and $calculatorAfterF12Ok -and
    $editorFixtureOk -and $editorFinalizeOk -and $editorMalformedFinalizeOk -and $editorWorkspaceAfterMalformedOk -and
    $editorInventoryOk -and $clearUpdated -and $editorAfterF12Ok -and
    $personalShellProofOk -and $personalShellInventoryOk -and $personalSecureStripOk -and $afterF12Ok -and
    $personalTrapRequestOk -and $personalFuelRequestOk -and $fallbackInventoryOk -and $recoveryAfterFallbackOk
$ruipCompatibilityDump = [ordered]@{ calculator = [ordered]@{ byte_len = $genesisCalculatorBytes.Length; sha256 = $calculatorActualSha256 }; editor = [ordered]@{ byte_len = $genesisEditorBytes.Length; sha256 = $editorActualSha256 }; prior_predicate_count = $preB13Predicates.Count; prior_failures = $preB13Failures }
Add-Predicate -Name "genesis-ui:ruip-byte-compatibility-pins-unchanged" -Expected "calculator stays 5372 bytes at its pinned hash, editor stays 176 bytes at its pinned hash, and every pre-existing delivery, malformed-atomicity, HID, inventory, F12, trap, fuel, and recovery predicate passed unchanged" -Passed $ruipCompatibilityOk -Actual $(if ($ruipCompatibilityOk) { "calculator=$genesisCalculatorHash/5372 editor=$genesisEditorHash/176 prior_predicates=$($preB13Predicates.Count)" } else { $ruipCompatibilityDump | ConvertTo-Json -Compress -Depth 16 })

$b13LoaderOk = $b13LoaderResponse.schema -eq "raios.evidence_response.v1" -and
    $b13LoaderResponse.family -eq "module.loader_runtime" -and
    $b13LoaderResponse.scope -eq "current_boot" -and
    $b13LoaderResponse.classification -eq "local_only" -and
    $b13LoaderResponse.source_method -eq "module.loader_runtime" -and
    $null -eq $b13LoaderResponse.event_id -and
    $b13LoaderEvidence.Count -eq 54 -and
    $b13LoaderEvidence[0].id -eq "manifest_reference" -and
    $b13LoaderEvidence[53].id -eq "executable_entrypoint_invocation_boundary" -and
    $b13LoaderResponse.PSObject.Properties.Name -notcontains "body" -and
    $b13LoaderResponse.PSObject.Properties.Name -notcontains "live_granted_load_projection" -and
    @($b13LoaderResponse.evidence | Where-Object id -eq "live_granted_load_projection").Count -eq 0 -and
    $b13LoaderApproval.facts.present -eq $false -and $b13LoaderApproval.facts.status_detail -eq "missing" -and
    $b13LoaderResponse.decision.outcome -eq "denied" -and
    # genesis-ui runs no M6 diagnostic chain, so the first missing loader
    # evidence is the manifest reference (nothing retained), not local_approval.
    $b13LoaderResponse.decision.reason -eq "retained_module_manifest_reference_missing" -and
    @($b13LoaderResponse.decision.grants).Count -eq 0 -and @($b13LoaderResponse.decision.effects).Count -eq 0
$b13SlotOk = $b13SlotResponse.facts.runtime.live_granted_service_slot_present -eq $false -and
    @($b13SlotResponse.evidence | Where-Object id -eq "live_granted_service_slot").Count -eq 0 -and
    $b13SlotResponse.decision.outcome -eq "denied"
$b13InventoryOk = $b13InventoryRows.Count -gt 0 -and
    @($b13InventoryRows | Where-Object { $_.PSObject.Properties.Name -contains "run_count" }).Count -eq 0
# raios.agent.v0 carve-outs carry the version under the top-level `v` field
# (not `schema`, which is the v1-envelope key); the payload is under body.result.
$b13CarveoutsOk = $editorInstall.PrepareResponse.v -eq "raios.agent.v0" -and
    $editorInstall.PrepareResponse.body.result -eq $editorPreview -and
    $editorInstall.SignatureResponse.v -eq "raios.agent.v0" -and
    $editorInstall.SignatureResponse.body.result -eq $editorSignedPreview -and
    $editorInstall.DenialResponse.v -eq "raios.agent.v0" -and
    $editorInstall.DenialResponse.body.result -eq $editorInstallDenial -and
    $b13ProgramWorkspaceResponse.v -eq "raios.agent.v0" -and
    $null -ne $b13ProgramWorkspaceResponse.body.result -and
    $personalShellResponse.v -eq "raios.agent.v0" -and
    $null -ne $personalShellResponse.body.result
$b12cShapesOk = $b13LoaderOk -and $b13SlotOk -and $b13InventoryOk -and $b13CarveoutsOk
$b12cShapesDump = [ordered]@{ loader_runtime = $b13LoaderResponse; service_slot = $b13SlotResponse; inventory = $b13InventoryResponse; install_prepare = $editorInstall.PrepareResponse; install_signature = $editorInstall.SignatureResponse; install_denial = $editorInstall.DenialResponse; program_workspace = $b13ProgramWorkspaceResponse; personal_shell_lifecycle = $personalShellResponse }
Add-Predicate -Name "genesis-ui:b12c-response-shapes-unchanged" -Expected "loader_runtime remains the bare 54-evidence v1 denial, slot presence remains facts.runtime with no positive evidence, inventory rows have no run_count, and program/install/personal-shell lifecycle responses remain raios.agent.v0 body.result carve-outs" -Passed $b12cShapesOk -Actual $(if ($b12cShapesOk) { "loader=bare-v1/54 slot=facts.runtime/false inventory=no-run_count lifecycle=body.result" } else { $b12cShapesDump | ConvertTo-Json -Compress -Depth 18 })

if (-not ($editorReadyOk -and $editorPrepareOk -and $editorSignatureOk -and $editorSerialDenialOk -and
    $editorSecondClickOk -and $editorArtstorOk -and $ruipCompatibilityOk -and $b12cShapesOk)) {
    throw "B1.3 genesis-ui install predicates did not all pass"
}
