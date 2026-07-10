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

$genesisBeforePersonalProof = Save-QemuScreendump -Name "genesis-before-personal-proof"
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
