# Genesis core UI focused profile (A2/G2).
#
# Dot-source this after shadow-vm-smoke-profile-common.ps1.  The dispatcher
# already owns boot, framebuffer, USB, serial TCP and QEMU lifecycle.  This
# profile deliberately uses only the normal serial command path: Genesis has
# no serial "pixel rendered" marker and the common runner has no pointer-input
# API, so a framebuffer screenshot must not become the test oracle.
#
# What this proves today: the typed current-boot facts that Genesis renders and
# its read-only recovery source are live, coherent and remain outside Wasm and
# provider authority.  Secure-entry and recovery-button interaction need their
# own non-secret runtime acknowledgement before they can be asserted here.

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
