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
# QEMU HID input and require non-secret guest acknowledgements before capture.

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
$genesisSystem = $genesisSnapshot.body.result
$genesisStatus = $genesisSystem.status
$genesisProblems = @($genesisSystem.problems)
$genesisSnapshotOk = (
    $genesisSystem.schema -eq "system.snapshot.v0" -and
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
$genesisProblemResult = $genesisProblemList.body.result
$genesisProblemEntries = @($genesisProblemResult.problems)
$genesisProblemFactsOk = (
    $genesisProblemResult.schema -eq "problem.list.v0" -and
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
$calculatorInventory = @($calculatorInventoryResponse.body.result.services | Where-Object { $_.id -eq "svc.user.shell" })
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
$calculatorAfterF12Personal = @($calculatorAfterF12Inventory.body.result.services | Where-Object { $_.id -eq "svc.user.shell" })
$calculatorAfterF12Genesis = @($calculatorAfterF12Inventory.body.result.services | Where-Object { $_.id -eq "core.ui.genesis" })
$calculatorAfterF12Ok = $calculatorAfterF12Personal.Count -eq 0 -and $calculatorAfterF12Genesis.Count -eq 1 -and $calculatorAfterF12Genesis[0].core_owned -eq $true -and $calculatorAfterF12Genesis[0].replaceable -eq $false
Add-Predicate -Name "genesis-ui:calculator-f12-restores-core-genesis" -Expected "F12 exits the calculator, removes svc.user.shell and restores immutable core Genesis" -Passed $calculatorAfterF12Ok -Actual $(if ($calculatorAfterF12Ok) { "core.ui.genesis only" } else { ($calculatorAfterF12Inventory.body.result.services | ConvertTo-Json -Compress -Depth 5) })
if (-not $calculatorAfterF12Ok) {
    throw "Expected F12 to restore Genesis after the current-boot calculator"
}
Save-QemuScreendump -Name "genesis-after-calculator-f12" | Out-Null

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
$personalShellInventory = @($personalShellInventoryResponse.body.result.services | Where-Object { $_.id -eq "svc.user.shell" })
$personalShellInventoryOk = (
    $personalShellInventory.Count -eq 1 -and
    $personalShellInventory[0].kind -eq "service" -and
    $personalShellInventory[0].scope -eq "current_boot" -and
    $personalShellInventory[0].persistence -eq "none" -and
    $personalShellInventory[0].trust_tier -eq "dev_key_not_owner_sealed" -and
    $personalShellInventory[0].owner_sealed -eq $false -and
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
$afterF12Personal = @($afterF12Inventory.body.result.services | Where-Object { $_.id -eq "svc.user.shell" })
$afterF12Genesis = @($afterF12Inventory.body.result.services | Where-Object { $_.id -eq "core.ui.genesis" })
$afterF12Ok = $afterF12Personal.Count -eq 0 -and $afterF12Genesis.Count -eq 1 -and $afterF12Genesis[0].core_owned -eq $true -and $afterF12Genesis[0].replaceable -eq $false
Add-Predicate -Name "genesis-ui:personal-shell-f12-removes-dynamic-inventory" -Expected "F12 returns to core Genesis and removes the current-boot personal row" -Passed $afterF12Ok -Actual $(if ($afterF12Ok) { "core.ui.genesis only" } else { ($afterF12Inventory.body.result.services | ConvertTo-Json -Compress -Depth 5) })
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
$fallbackPersonal = @($fallbackInventory.body.result.services | Where-Object { $_.id -eq "svc.user.shell" })
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
