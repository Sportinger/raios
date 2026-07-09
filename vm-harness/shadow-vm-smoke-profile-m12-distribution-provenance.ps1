$expectedShaBare = "f81f9442de3729f58f9d5c43b186a4223e3f0ed0bdde20e94722da8d5733abd2"
$expectedSha = "sha256:$expectedShaBare"
$expectedBufechoShaBare = "1983797d9ecc6f3f85deedc0c82a8651062f01dc80710ee699e834a51c52e544"
$expectedBufechoSha = "sha256:$expectedBufechoShaBare"
$provSigHex = "304402201fd9aa3e26579ab9852a1ea61a7fe23f79c39badd13e2c74dbdf9d957a25449b02204f783191894cfb609d35c5babc9fb3208e77d9c712d2645a633120db6dcdd89b"

function Assert-M12Predicate {
    param(
        [string]$Name,
        [string]$Expected,
        [bool]$Passed,
        [string]$Actual,
        [string]$FailureMessage
    )
    Add-Predicate -Name $Name -Expected $Expected -Passed $Passed -Actual $Actual
    if (-not $Passed) {
        throw $FailureMessage
    }
}

function Test-M12Denials {
    param([object]$Record)
    return (
        $Record.load_authorized -eq $false -and
        $Record.install_authorized -eq $false -and
        $Record.owner_sealed -eq $false -and
        $Record.requires_m6_reverify_for_load -eq $true -and
        $Record.authorizes_load -eq $false -and
        $Record.authorizes_execution -eq $false -and
        $Record.writes_persistent_state -eq $false
    )
}

function Test-M12RegistryDenials {
    param([object]$Record)
    return (
        $Record.authorizes_acquisition -eq $false -and
        $Record.authorizes_install -eq $false -and
        $Record.authorizes_load -eq $false -and
        $Record.authorizes_execute -eq $false -and
        $Record.authorizes_persist -eq $false -and
        $Record.writes_persistent_state -eq $false -and
        $Record.load_attempted -eq $false -and
        $Record.execution_attempted -eq $false -and
        $Record.durable_write_attempted -eq $false -and
        $Record.owner_sealed -eq $false
    )
}

function Get-M12SelftestCase {
    param([object[]]$Cases, [string]$Name)
    return @($Cases | Where-Object { $_.case -eq $Name })[0]
}

function Get-M12BytesSha256Hex {
    param([byte[]]$Bytes)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $hash = $sha.ComputeHash($Bytes)
        return (($hash | ForEach-Object { $_.ToString("x2") }) -join "")
    }
    finally {
        $sha.Dispose()
    }
}

function Get-M12ByteSlice {
    param(
        [byte[]]$Bytes,
        [int]$Offset,
        [int]$Count
    )
    $chunk = New-Object byte[] $Count
    [Array]::Copy($Bytes, $Offset, $chunk, 0, $Count)
    return $chunk
}

function Get-M12DistributionChunks {
    param([byte[]]$Bytes)
    $firstLen = [Math]::Floor($Bytes.Length / 3)
    $secondLen = [Math]::Floor(($Bytes.Length * 2) / 3) - $firstLen
    $thirdLen = $Bytes.Length - $firstLen - $secondLen
    return @(
        [pscustomobject]@{ index = 0; bytes = (Get-M12ByteSlice -Bytes $Bytes -Offset 0 -Count $firstLen) },
        [pscustomobject]@{ index = 1; bytes = (Get-M12ByteSlice -Bytes $Bytes -Offset $firstLen -Count $secondLen) },
        [pscustomobject]@{ index = 2; bytes = (Get-M12ByteSlice -Bytes $Bytes -Offset ($firstLen + $secondLen) -Count $thirdLen) }
    )
}

function Send-M12DistributionBytes {
    param(
        [string]$Path,
        [string]$ExpectedShaBare,
        [string]$SignatureHex
    )

    if (-not (Test-Path -LiteralPath $Path)) {
        throw "Distribution artifact does not exist: $Path"
    }

    $bytes = [System.IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Path).Path)
    $chunks = @(Get-M12DistributionChunks -Bytes $bytes)
    Send-AgentCommand -Command "module.submit_distribution_begin sha256:$ExpectedShaBare $($bytes.Length) $($chunks.Count) sig:$SignatureHex" -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_begin" -Name "m12-distribution:T1_transport_begin"
    foreach ($chunk in @($chunks[2], $chunks[0], $chunks[1])) {
        $chunkHash = Get-M12BytesSha256Hex -Bytes $chunk.bytes
        $chunkBase64 = [Convert]::ToBase64String($chunk.bytes)
        Send-AgentCommand -Command "module.submit_distribution_chunk $($chunk.index) sha256:$chunkHash $chunkBase64" -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_chunk" -Name "m12-distribution:T1_transport_chunk_$($chunk.index)"
    }
    Send-AgentCommand -Command "module.submit_distribution_finalize" -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_finalize" -Name "m12-distribution:T1_transport_finalize"

    return [pscustomobject]@{
        byte_len = $bytes.Length
        chunk_count = $chunks.Count
        finalize_response = (Get-LastAgentResponseJson -Method "module.submit_distribution_finalize")
    }
}

Send-AgentCommand -Command "agent module.registry_selection_diagnostic $expectedSha" -ExpectedMarker "RAIOS_AGENT_END module.registry_selection_diagnostic" -Name "m12-distribution:P1_registry_selection_valid"
$registry = Get-LastAgentResponseJson -Method "module.registry_selection_diagnostic"
$registryResult = $registry.body.result
$selection = $registryResult.selection
$staged = $registryResult.staged_candidate
$retainedProv = $registryResult.retained_provenance
$registryOk = (
    $registry.t -eq "response" -and
    $registryResult.status -eq "selected" -and
    $registryResult.reason -eq "registry_entry_selected_for_inert_candidate_intake" -and
    [int]$registryResult.registry_entry_count -eq 2 -and
    [int]$registryResult.registry_capacity -ge 2 -and
    $registryResult.entry_id -eq "builtin.svc.demo.echo" -and
    $selection.schema -eq "raios.distribution_registry_selection.v0" -and
    $selection.entry_id -eq "builtin.svc.demo.echo" -and
    $selection.selected_for_candidate_intake -eq $true -and
    $selection.selection_hash_matched -eq $true -and
    $selection.provenance_signature_verified -eq $true -and
    $registryResult.provenance_is_origin_evidence_only -eq $true -and
    $selection.artifact_sha256 -eq $expectedSha -and
    [int]$staged.byte_len -eq 4205 -and
    $staged.artifact_sha256 -eq $expectedSha -and
    $staged.wasm_valid -eq $true -and
    $staged.retained_in_ram -eq $true -and
    $staged.rejected -eq $false -and
    $retainedProv.provenance_verified -eq $true -and
    $retainedProv.artifact_sha256 -eq $expectedSha -and
    $retainedProv.status -eq "distribution_candidate_provenance_verified_load_still_denied" -and
    $registryResult.recomputed_sha256_matches_selection -eq $true -and
    $registryResult.staged_only_after_valid_selection -eq $true -and
    (Test-M12RegistryDenials -Record $registryResult) -and
    (Test-M12RegistryDenials -Record $selection) -and
    (Test-M12Denials -Record $retainedProv)
)
Assert-M12Predicate `
    -Name "m12-distribution:P1_registry_selection_stages_inert_echo_candidate" `
    -Expected "built-in registry entry selected by content hash enters existing candidate intake as inert current_boot retained wasm" `
    -Passed $registryOk `
    -Actual $(if ($registryOk) { "matched" } else { ($registryResult | ConvertTo-Json -Compress -Depth 10) }) `
    -FailureMessage "Expected registry selection to retain an inert valid wasm candidate"

Send-AgentCommand -Command "agent module.registry_selection_diagnostic $expectedBufechoSha" -ExpectedMarker "RAIOS_AGENT_END module.registry_selection_diagnostic" -Name "m12-distribution:P1b_registry_selection_bufecho"
$bufechoRegistry = Get-LastAgentResponseJson -Method "module.registry_selection_diagnostic"
$bufechoRegistryResult = $bufechoRegistry.body.result
$bufechoSelection = $bufechoRegistryResult.selection
$bufechoStaged = $bufechoRegistryResult.staged_candidate
$bufechoRetainedProv = $bufechoRegistryResult.retained_provenance
$bufechoRegistryOk = (
    $bufechoRegistry.t -eq "response" -and
    $bufechoRegistryResult.status -eq "selected" -and
    $bufechoRegistryResult.reason -eq "registry_entry_selected_for_inert_candidate_intake" -and
    [int]$bufechoRegistryResult.registry_entry_count -eq 2 -and
    $bufechoRegistryResult.entry_id -eq "builtin.svc.demo.bufecho" -and
    $bufechoSelection.entry_id -eq "builtin.svc.demo.bufecho" -and
    $bufechoSelection.artifact_sha256 -eq $expectedBufechoSha -and
    [int]$bufechoStaged.byte_len -eq 605 -and
    $bufechoStaged.artifact_sha256 -eq $expectedBufechoSha -and
    $bufechoStaged.wasm_valid -eq $true -and
    $bufechoStaged.retained_in_ram -eq $true -and
    $bufechoStaged.rejected -eq $false -and
    $bufechoRetainedProv.provenance_verified -eq $true -and
    $bufechoRetainedProv.artifact_sha256 -eq $expectedBufechoSha -and
    $bufechoRegistryResult.recomputed_sha256_matches_selection -eq $true -and
    $bufechoRegistryResult.staged_only_after_valid_selection -eq $true -and
    (Test-M12RegistryDenials -Record $bufechoRegistryResult) -and
    (Test-M12RegistryDenials -Record $bufechoSelection) -and
    (Test-M12Denials -Record $bufechoRetainedProv)
)
Assert-M12Predicate `
    -Name "m12-distribution:P1b_multi_entry_registry_selects_bufecho" `
    -Expected "bounded built-in registry selects a second signed local artifact by content hash and stages it inert" `
    -Passed $bufechoRegistryOk `
    -Actual $(if ($bufechoRegistryOk) { "matched" } else { ($bufechoRegistryResult | ConvertTo-Json -Compress -Depth 10) }) `
    -FailureMessage "Expected multi-entry registry selection to retain inert bufecho candidate"

Send-AgentCommand -Command "agent module.registry_selection_diagnostic sha256:0000000000000000000000000000000000000000000000000000000000000000" -ExpectedMarker "RAIOS_AGENT_END module.registry_selection_diagnostic" -Name "m12-distribution:P1_registry_selection_wrong_hash"
$wrongSelection = Get-LastAgentResponseJson -Method "module.registry_selection_diagnostic"
$wrongSelectionResult = $wrongSelection.body.result
$wrongSelectionOk = (
    $wrongSelectionResult.status -eq "denied" -and
    $wrongSelectionResult.reason -eq "registry_entry_not_found" -and
    $null -eq $wrongSelectionResult.selection -and
    $null -eq $wrongSelectionResult.staged_candidate -and
    $wrongSelectionResult.staged_only_after_valid_selection -eq $true -and
    (Test-M12RegistryDenials -Record $wrongSelectionResult)
)
Assert-M12Predicate `
    -Name "m12-distribution:P1_registry_selection_wrong_hash_no_stage" `
    -Expected "wrong content hash is denied and does not stage a candidate" `
    -Passed $wrongSelectionOk `
    -Actual $(if ($wrongSelectionOk) { "matched" } else { ($wrongSelectionResult | ConvertTo-Json -Compress -Depth 10) }) `
    -FailureMessage "Expected wrong registry selector to fail closed without staging"

Send-AgentCommand -Command "agent module.registry_selection_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.registry_selection_diagnostic_selftest" -Name "m12-distribution:P1_registry_selection_selftest"
$registrySelftest = Get-LastAgentResponseJson -Method "module.registry_selection_diagnostic_selftest"
$registrySelftestResult = $registrySelftest.body.result
$registryCases = @($registrySelftestResult.cases)
$registryValidCase = Get-M12SelftestCase -Cases $registryCases -Name "valid_echo_registry_selection_stages_inert_candidate"
$registryBufechoCase = Get-M12SelftestCase -Cases $registryCases -Name "valid_bufecho_registry_selection_stages_inert_candidate"
$registryChunkedCase = Get-M12SelftestCase -Cases $registryCases -Name "chunked_bufecho_delivery_stages_inert_candidate"
$registryWrongCase = Get-M12SelftestCase -Cases $registryCases -Name "wrong_hash_denied_without_staging"
$registryInvalidCase = Get-M12SelftestCase -Cases $registryCases -Name "invalid_selector_denied_without_staging"
$registrySelftestOk = (
    $registrySelftestResult.passed -eq $true -and
    [int]$registrySelftestResult.case_count -eq 5 -and
    $registryValidCase.passed -eq $true -and
    $registryValidCase.staged -eq $true -and
    $registryValidCase.retained_provenance_verified -eq $true -and
    $registryBufechoCase.passed -eq $true -and
    $registryBufechoCase.staged -eq $true -and
    $registryBufechoCase.retained_provenance_verified -eq $true -and
    $registryChunkedCase.passed -eq $true -and
    $registryChunkedCase.staged -eq $true -and
    $registryChunkedCase.retained_provenance_verified -eq $true -and
    $registryWrongCase.passed -eq $true -and
    $registryWrongCase.staged -eq $false -and
    $registryWrongCase.reason -eq "registry_entry_not_found" -and
    $registryInvalidCase.passed -eq $true -and
    $registryInvalidCase.reason -eq "invalid_sha256_selector" -and
    $registrySelftestResult.owner_sealed -eq $false -and
    $registrySelftestResult.durable_write -eq $false -and
    $registrySelftestResult.load_authorized -eq $false -and
    $registrySelftestResult.execute_authorized -eq $false -and
    $registrySelftestResult.persist_authorized -eq $false
)
Assert-M12Predicate `
    -Name "m12-distribution:P1_registry_selection_selftest_cases" `
    -Expected "registry-selection selftest proves valid staging plus wrong/invalid selector denials" `
    -Passed $registrySelftestOk `
    -Actual $(if ($registrySelftestOk) { "matched" } else { ($registrySelftestResult | ConvertTo-Json -Compress -Depth 10) }) `
    -FailureMessage "Expected registry selection selftest to pass"

$echoArtifactPath = Join-Path $RepoRoot "seed-kernel\artifacts\svc.demo.echo.wasm"
$echoBytes = [System.IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $echoArtifactPath).Path)
$echoChunks = @(Get-M12DistributionChunks -Bytes $echoBytes)
$badChunkBase64 = [Convert]::ToBase64String($echoChunks[0].bytes)
Send-AgentCommand -Command "module.submit_distribution_begin $expectedSha $($echoBytes.Length) $($echoChunks.Count) sig:$provSigHex" -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_begin" -Name "m12-distribution:T0_bad_transport_begin"
Send-AgentCommand -Command "module.submit_distribution_chunk 0 sha256:0000000000000000000000000000000000000000000000000000000000000000 $badChunkBase64" -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_chunk" -Name "m12-distribution:T0_bad_chunk_hash"
$badTransportChunk = Get-LastAgentResponseJson -Method "module.submit_distribution_chunk"
$badTransportChunkResult = $badTransportChunk.body.result
$badTransportChunkOk = (
    $badTransportChunkResult.rejected -eq $true -and
    $badTransportChunkResult.accepted -eq $false -and
    $badTransportChunkResult.discarded_pending_delivery -eq $true -and
    $badTransportChunkResult.reason -eq "chunk_sha256_mismatch" -and
    [int]$badTransportChunkResult.pending_chunk_count -eq 0
)
Assert-M12Predicate `
    -Name "m12-distribution:T0_bad_chunk_hash_discards_pending" `
    -Expected "distribution chunk with mismatched claimed sha256 is rejected and clears pending transport" `
    -Passed $badTransportChunkOk `
    -Actual $(if ($badTransportChunkOk) { "matched" } else { ($badTransportChunkResult | ConvertTo-Json -Compress -Depth 6) }) `
    -FailureMessage "Expected bad distribution chunk hash to discard pending delivery"
Send-AgentCommand -Command "module.submit_distribution_finalize" -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_finalize" -Name "m12-distribution:T0_finalize_after_bad_chunk"
$badTransportFinalize = Get-LastAgentResponseJson -Method "module.submit_distribution_finalize"
$badTransportFinalizeResult = $badTransportFinalize.body.result
$badTransportFinalizeOk = (
    $badTransportFinalizeResult.status -eq "denied" -and
    $badTransportFinalizeResult.reason -eq "distribution_delivery_not_started" -and
    $null -eq $badTransportFinalizeResult.selection -and
    $null -eq $badTransportFinalizeResult.staged_candidate -and
    $badTransportFinalizeResult.authorizes_load -eq $false -and
    $badTransportFinalizeResult.writes_persistent_state -eq $false
)
Assert-M12Predicate `
    -Name "m12-distribution:T0_finalize_after_bad_chunk_no_stage" `
    -Expected "finalize after rejected distribution chunk stages no candidate" `
    -Passed $badTransportFinalizeOk `
    -Actual $(if ($badTransportFinalizeOk) { "matched" } else { ($badTransportFinalizeResult | ConvertTo-Json -Compress -Depth 8) }) `
    -FailureMessage "Expected finalize after bad distribution chunk to fail closed"

$distributionDelivery = Send-M12DistributionBytes -Path $echoArtifactPath -ExpectedShaBare $expectedShaBare -SignatureHex $provSigHex
$distributionFinalize = $distributionDelivery.finalize_response
$distributionResult = $distributionFinalize.body.result
$distributionSelection = $distributionResult.selection
$distributionStaged = $distributionResult.staged_candidate
$distributionRetained = $distributionResult.retained_provenance
$distributionOk = (
    $distributionFinalize.t -eq "response" -and
    $distributionResult.status -eq "selected" -and
    $distributionResult.reason -eq "registry_entry_selected_for_inert_candidate_intake" -and
    $distributionResult.delivery_channel -eq "serial_console_distribution_chunks_v0" -and
    [int]$distributionResult.total_length -eq 4205 -and
    [int]$distributionResult.declared_chunk_count -eq 3 -and
    [int]$distributionResult.accepted_chunk_count -eq 3 -and
    [int]$distributionResult.delivered_byte_len -eq 4205 -and
    $distributionSelection.entry_id -eq "serial.local.distribution" -and
    $distributionSelection.artifact_sha256 -eq $expectedSha -and
    $distributionSelection.provenance_signature_verified -eq $true -and
    $distributionSelection.selected_for_candidate_intake -eq $true -and
    $distributionStaged.artifact_sha256 -eq $expectedSha -and
    $distributionStaged.wasm_valid -eq $true -and
    $distributionStaged.retained_in_ram -eq $true -and
    $distributionStaged.rejected -eq $false -and
    $distributionRetained.provenance_verified -eq $true -and
    $distributionRetained.artifact_sha256 -eq $expectedSha -and
    $distributionResult.staged_only_after_valid_selection -eq $true -and
    (Test-M12RegistryDenials -Record $distributionResult) -and
    (Test-M12RegistryDenials -Record $distributionSelection) -and
    (Test-M12Denials -Record $distributionRetained)
)
Assert-M12Predicate `
    -Name "m12-distribution:T1_serial_distribution_transport_stages_inert_candidate" `
    -Expected "real serial distribution transport reassembles signed echo artifact and stages it inert after chunk and whole hash verification" `
    -Passed $distributionOk `
    -Actual $(if ($distributionOk) { "matched" } else { ($distributionResult | ConvertTo-Json -Compress -Depth 10) }) `
    -FailureMessage "Expected serial distribution delivery to stage a valid inert candidate"

Send-AgentCommand -Command "module.submit_distribution_catalog_entry $expectedSha $($echoBytes.Length) $($echoChunks.Count) sig:$provSigHex" -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_catalog_entry" -Name "m12-distribution:T2_catalog_entry"
$catalogEntry = Get-LastAgentResponseJson -Method "module.submit_distribution_catalog_entry"
$catalogEntryResult = $catalogEntry.body.result
$catalogEntryOk = (
    $catalogEntry.t -eq "response" -and
    $catalogEntryResult.source_id -eq "local.serial.catalog" -and
    $catalogEntryResult.entry_id -eq "local.catalog.distribution" -and
    $catalogEntryResult.content_sha256 -eq $expectedSha -and
    [int]$catalogEntryResult.total_length -eq 4205 -and
    [int]$catalogEntryResult.chunk_count -eq 3 -and
    $catalogEntryResult.retained_in_catalog -eq $true -and
    $catalogEntryResult.accepted -eq $true -and
    $catalogEntryResult.rejected -eq $false -and
    $catalogEntryResult.reason -eq "accepted_local_distribution_catalog_entry" -and
    (Test-M12RegistryDenials -Record $catalogEntryResult)
)
Assert-M12Predicate `
    -Name "m12-distribution:T2_local_catalog_entry_retained" `
    -Expected "non-builtin local catalog retains signed artifact metadata in current_boot without granting authority" `
    -Passed $catalogEntryOk `
    -Actual $(if ($catalogEntryOk) { "matched" } else { ($catalogEntryResult | ConvertTo-Json -Compress -Depth 8) }) `
    -FailureMessage "Expected local catalog entry to be retained without authority"

Send-AgentCommand -Command "module.submit_distribution_begin_from_catalog sha256:0000000000000000000000000000000000000000000000000000000000000000" -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_begin_from_catalog" -Name "m12-distribution:T2_wrong_catalog_selector"
$wrongCatalogBegin = Get-LastAgentResponseJson -Method "module.submit_distribution_begin_from_catalog"
$wrongCatalogBeginResult = $wrongCatalogBegin.body.result
$wrongCatalogBeginOk = (
    $wrongCatalogBeginResult.source_id -eq "local.serial.catalog" -and
    $wrongCatalogBeginResult.entry_id -eq "local.catalog.distribution" -and
    $wrongCatalogBeginResult.accepted -eq $false -and
    $wrongCatalogBeginResult.rejected -eq $true -and
    $wrongCatalogBeginResult.reason -eq "local_distribution_catalog_entry_not_found" -and
    $null -eq $wrongCatalogBeginResult.content_sha256 -and
    (Test-M12RegistryDenials -Record $wrongCatalogBeginResult)
)
Assert-M12Predicate `
    -Name "m12-distribution:T2_wrong_catalog_selector_no_begin" `
    -Expected "wrong local catalog selector does not start delivery or stage a candidate" `
    -Passed $wrongCatalogBeginOk `
    -Actual $(if ($wrongCatalogBeginOk) { "matched" } else { ($wrongCatalogBeginResult | ConvertTo-Json -Compress -Depth 8) }) `
    -FailureMessage "Expected wrong local catalog selector to fail closed"
Send-AgentCommand -Command "module.submit_distribution_finalize" -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_finalize" -Name "m12-distribution:T2_finalize_after_wrong_catalog_selector"
$wrongCatalogFinalize = Get-LastAgentResponseJson -Method "module.submit_distribution_finalize"
$wrongCatalogFinalizeResult = $wrongCatalogFinalize.body.result
$wrongCatalogFinalizeOk = (
    $wrongCatalogFinalizeResult.status -eq "denied" -and
    $wrongCatalogFinalizeResult.reason -eq "distribution_delivery_not_started" -and
    $null -eq $wrongCatalogFinalizeResult.selection -and
    $null -eq $wrongCatalogFinalizeResult.staged_candidate -and
    $wrongCatalogFinalizeResult.authorizes_load -eq $false -and
    $wrongCatalogFinalizeResult.writes_persistent_state -eq $false
)
Assert-M12Predicate `
    -Name "m12-distribution:T2_wrong_catalog_finalize_no_stage" `
    -Expected "finalize after wrong local catalog selector stages no candidate" `
    -Passed $wrongCatalogFinalizeOk `
    -Actual $(if ($wrongCatalogFinalizeOk) { "matched" } else { ($wrongCatalogFinalizeResult | ConvertTo-Json -Compress -Depth 8) }) `
    -FailureMessage "Expected finalize after wrong local catalog selector to fail closed"

Send-AgentCommand -Command "module.submit_distribution_begin_from_catalog $expectedSha" -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_begin_from_catalog" -Name "m12-distribution:T2_catalog_begin"
$catalogBegin = Get-LastAgentResponseJson -Method "module.submit_distribution_begin_from_catalog"
$catalogBeginResult = $catalogBegin.body.result
$catalogBeginOk = (
    $catalogBeginResult.source_id -eq "local.serial.catalog" -and
    $catalogBeginResult.entry_id -eq "local.catalog.distribution" -and
    $catalogBeginResult.content_sha256 -eq $expectedSha -and
    [int]$catalogBeginResult.total_length -eq 4205 -and
    [int]$catalogBeginResult.chunk_count -eq 3 -and
    $catalogBeginResult.accepted -eq $true -and
    $catalogBeginResult.rejected -eq $false -and
    $catalogBeginResult.reason -eq "accepted_catalog_distribution_delivery_target" -and
    (Test-M12RegistryDenials -Record $catalogBeginResult)
)
Assert-M12Predicate `
    -Name "m12-distribution:T2_catalog_begin_uses_retained_metadata" `
    -Expected "matching local catalog selector starts the same bounded chunk transport from retained metadata" `
    -Passed $catalogBeginOk `
    -Actual $(if ($catalogBeginOk) { "matched" } else { ($catalogBeginResult | ConvertTo-Json -Compress -Depth 8) }) `
    -FailureMessage "Expected local catalog begin to use retained metadata"
foreach ($chunk in @($echoChunks[2], $echoChunks[0], $echoChunks[1])) {
    $chunkHash = Get-M12BytesSha256Hex -Bytes $chunk.bytes
    $chunkBase64 = [Convert]::ToBase64String($chunk.bytes)
    Send-AgentCommand -Command "module.submit_distribution_chunk $($chunk.index) sha256:$chunkHash $chunkBase64" -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_chunk" -Name "m12-distribution:T2_catalog_chunk_$($chunk.index)"
}
Send-AgentCommand -Command "module.submit_distribution_finalize" -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_finalize" -Name "m12-distribution:T2_catalog_finalize"
$catalogFinalize = Get-LastAgentResponseJson -Method "module.submit_distribution_finalize"
$catalogResult = $catalogFinalize.body.result
$catalogSelection = $catalogResult.selection
$catalogStaged = $catalogResult.staged_candidate
$catalogRetained = $catalogResult.retained_provenance
$catalogOk = (
    $catalogFinalize.t -eq "response" -and
    $catalogResult.status -eq "selected" -and
    $catalogResult.reason -eq "registry_entry_selected_for_inert_candidate_intake" -and
    $catalogResult.source_id -eq "local.serial.catalog" -and
    $catalogResult.entry_id -eq "local.catalog.distribution" -and
    [int]$catalogResult.total_length -eq 4205 -and
    [int]$catalogResult.declared_chunk_count -eq 3 -and
    [int]$catalogResult.accepted_chunk_count -eq 3 -and
    [int]$catalogResult.delivered_byte_len -eq 4205 -and
    $catalogSelection.entry_id -eq "local.catalog.distribution" -and
    $catalogSelection.artifact_sha256 -eq $expectedSha -and
    $catalogSelection.provenance_signature_verified -eq $true -and
    $catalogSelection.selected_for_candidate_intake -eq $true -and
    $catalogStaged.artifact_sha256 -eq $expectedSha -and
    $catalogStaged.wasm_valid -eq $true -and
    $catalogStaged.retained_in_ram -eq $true -and
    $catalogStaged.rejected -eq $false -and
    $catalogRetained.provenance_verified -eq $true -and
    $catalogRetained.artifact_sha256 -eq $expectedSha -and
    $catalogResult.staged_only_after_valid_selection -eq $true -and
    (Test-M12RegistryDenials -Record $catalogResult) -and
    (Test-M12RegistryDenials -Record $catalogSelection) -and
    (Test-M12Denials -Record $catalogRetained)
)
Assert-M12Predicate `
    -Name "m12-distribution:T2_local_catalog_delivery_stages_inert_candidate" `
    -Expected "local catalog source feeds the existing signed chunk delivery path and stages only an inert candidate" `
    -Passed $catalogOk `
    -Actual $(if ($catalogOk) { "matched" } else { ($catalogResult | ConvertTo-Json -Compress -Depth 10) }) `
    -FailureMessage "Expected local catalog delivery to stage a valid inert candidate"

Send-AgentCommand -Command "agent module.distribution_provenance_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.distribution_provenance_diagnostic_selftest" -Name "m12-distribution:P2_selftest_command"
$selftest = Get-LastAgentResponseJson -Method "module.distribution_provenance_diagnostic_selftest"
$selftestResult = $selftest.body.result
$cases = @($selftestResult.cases)
$selftestOk = (
    $selftest.t -eq "response" -and
    $selftestResult.passed -eq $true -and
    [int]$selftestResult.case_count -eq 4 -and
    $selftestResult.trust_tier -eq "dev_key_not_owner_sealed" -and
    $selftestResult.owner_sealed -eq $false -and
    $selftestResult.read_only -eq $true -and
    $selftestResult.durable_write -eq $false
)
Assert-M12Predicate `
    -Name "m12-distribution:P2_selftest_positive_and_negative" `
    -Expected "in-guest provenance selftest passes 4 cases and grants nothing" `
    -Passed $selftestOk `
    -Actual $(if ($selftestOk) { "matched" } else { ($selftestResult | ConvertTo-Json -Compress -Depth 8) }) `
    -FailureMessage "Expected distribution provenance selftest to pass"

$validCase = Get-M12SelftestCase -Cases $cases -Name "valid_provenance_load_still_denied"
$validCaseOk = (
    $validCase.passed -eq $true -and
    $validCase.provenance_verified -eq $true -and
    $validCase.status -eq "distribution_candidate_provenance_verified_load_still_denied" -and
    (Test-M12Denials -Record $validCase)
)
Assert-M12Predicate `
    -Name "m12-distribution:P2_valid_provenance_load_still_denied" `
    -Expected "valid provenance verifies but load/install/owner/persistence remain denied" `
    -Passed $validCaseOk `
    -Actual $(if ($validCaseOk) { "matched" } else { ($validCase | ConvertTo-Json -Compress -Depth 6) }) `
    -FailureMessage "Expected valid provenance selftest case to keep load denied"

foreach ($caseName in @("absent_signature_denied", "tampered_signature_rejected", "signature_bound_to_wrong_artifact_rejected")) {
    $case = Get-M12SelftestCase -Cases $cases -Name $caseName
    $caseOk = (
        $case.passed -eq $true -and
        $case.provenance_verified -eq $false -and
        (Test-M12Denials -Record $case)
    )
    Assert-M12Predicate `
        -Name "m12-distribution:P2_$caseName" `
        -Expected "$caseName fails closed and grants nothing" `
        -Passed $caseOk `
        -Actual $(if ($caseOk) { "matched" } else { ($case | ConvertTo-Json -Compress -Depth 6) }) `
        -FailureMessage "Expected $caseName to fail closed"
}

Send-AgentCommand -Command "agent module.registry_selection_diagnostic $expectedSha" -ExpectedMarker "RAIOS_AGENT_END module.registry_selection_diagnostic" -Name "m12-distribution:P3_restages_echo_candidate_for_live_provenance"
Send-AgentCommand -Command "agent module.distribution_provenance_diagnostic $provSigHex" -ExpectedMarker "RAIOS_AGENT_END module.distribution_provenance_diagnostic" -Name "m12-distribution:P3_live_diagnostic_valid_signature"
$live = Get-LastAgentResponseJson -Method "module.distribution_provenance_diagnostic"
$liveResult = $live.body.result
$liveOk = (
    $live.t -eq "response" -and
    $liveResult.schema -eq "raios.distribution_candidate.v0" -and
    $liveResult.provenance_verified -eq $true -and
    $liveResult.status -eq "distribution_candidate_provenance_verified_load_still_denied" -and
    $liveResult.honest -eq $true -and
    $liveResult.retained_present -eq $true -and
    $liveResult.artifact_sha256 -eq $expectedSha -and
    $liveResult.trust_tier -eq "dev_key_not_owner_sealed" -and
    $liveResult.live_load_projection_present -eq $false -and
    $liveResult.live_load_projection_can_load_now -eq $false -and
    (Test-M12Denials -Record $liveResult)
)
Assert-M12Predicate `
    -Name "m12-distribution:P3_live_provenance_verified_load_still_denied" `
    -Expected "live diagnostic verifies provenance over retained in-guest hash and grants nothing" `
    -Passed $liveOk `
    -Actual $(if ($liveOk) { "matched" } else { ($liveResult | ConvertTo-Json -Compress -Depth 8) }) `
    -FailureMessage "Expected live provenance diagnostic to verify and keep load denied"
Assert-LogContains -Name "m12-distribution:P3_serial_provenance_verified" -Needle '"provenance_verified": true' -TimeoutSeconds 1
Assert-LogContains -Name "m12-distribution:P3_serial_load_authorized_false" -Needle '"load_authorized": false' -TimeoutSeconds 1
Assert-LogContains -Name "m12-distribution:P3_serial_requires_m6_reverify" -Needle '"requires_m6_reverify_for_load": true' -TimeoutSeconds 1
Assert-LogContains -Name "m12-distribution:P3_serial_dev_key_tier" -Needle '"trust_tier": "dev_key_not_owner_sealed"' -TimeoutSeconds 1

$tamperedSigHex = "31" + $provSigHex.Substring(2)
Send-AgentCommand -Command "agent module.distribution_provenance_diagnostic $tamperedSigHex" -ExpectedMarker "RAIOS_AGENT_END module.distribution_provenance_diagnostic" -Name "m12-distribution:N1_live_tampered_signature"
$tampered = Get-LastAgentResponseJson -Method "module.distribution_provenance_diagnostic"
$tamperedResult = $tampered.body.result
$tamperedOk = (
    $tamperedResult.provenance_verified -eq $false -and
    $tamperedResult.reason -eq "provenance_signature_unverified" -and
    (Test-M12Denials -Record $tamperedResult)
)
Assert-M12Predicate `
    -Name "m12-distribution:N1_tampered_signature_unverified" `
    -Expected "tampered provenance signature is unverified and grants nothing" `
    -Passed $tamperedOk `
    -Actual $(if ($tamperedOk) { "matched" } else { ($tamperedResult | ConvertTo-Json -Compress -Depth 8) }) `
    -FailureMessage "Expected tampered provenance signature to fail closed"
Send-AgentCommand -Command "caps" -ExpectedMarker "RAIOS_AGENT_END system.capabilities" -Name "m12-distribution:N1_liveness_after_tamper"

$wrongArtifactCase = Get-M12SelftestCase -Cases $cases -Name "signature_bound_to_wrong_artifact_rejected"
$wrongArtifactOk = $wrongArtifactCase.passed -eq $true -and $wrongArtifactCase.provenance_verified -eq $false
Assert-M12Predicate `
    -Name "m12-distribution:N2_wrong_artifact_binding_selftest" `
    -Expected "valid fixture over wrong artifact hash is rejected in guest selftest" `
    -Passed $wrongArtifactOk `
    -Actual $(if ($wrongArtifactOk) { "matched" } else { ($wrongArtifactCase | ConvertTo-Json -Compress -Depth 6) }) `
    -FailureMessage "Expected wrong-artifact selftest case to pass"

$loadOffset = Get-SerialLogOffset
Send-AgentCommand -Command "module.load_ephemeral svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "m12-distribution:N3_load_still_denied"
$load = Get-LastAgentResponseJson -Method "module.load_ephemeral"
$loadAfter = (Get-SerialLogContent -Path $SerialLog).Substring([int]$loadOffset)
$loadDeniedOk = (
    $load.t -eq "error" -and
    $load.body.code -eq "capability_denied" -and
    $load.body.schema -eq "raios.module_load_gate.v0" -and
    -not $loadAfter.Contains("WASM_GUEST_LOG") -and
    -not $loadAfter.Contains('"instantiation_ok": true')
)
Assert-M12Predicate `
    -Name "m12-distribution:N3_provenance_does_not_enable_granted_candidate_load" `
    -Expected "valid provenance does not create a grant; module.load_ephemeral remains capability_denied with no instantiation" `
    -Passed $loadDeniedOk `
    -Actual $(if ($loadDeniedOk) { "matched" } else { ($load | ConvertTo-Json -Compress -Depth 8) }) `
    -FailureMessage "Expected provenance-verified candidate load to stay denied"

Send-AgentCommand -Command "module.load_ephemeral" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "m12-distribution:N4_generic_durable_gate"
$generic = Get-LastAgentResponseJson -Method "module.load_ephemeral"
$genericOk = (
    $generic.body.code -eq "capability_denied" -and
    $generic.body.schema -eq "raios.module_load_gate.v0" -and
    $generic.body.gate_state.rollback_plan -eq "missing" -and
    $generic.body.gate_state.durable_audit_record -eq "missing" -and
    $generic.body.gate_state.artifact_loaded -eq $false -and
    $generic.body.gate_state.service_started -eq $false
)
Assert-M12Predicate `
    -Name "m12-distribution:N4_generic_durable_load_gate_preserved" `
    -Expected "generic durable load gate remains denied with audit/rollback missing" `
    -Passed $genericOk `
    -Actual $(if ($genericOk) { "matched" } else { ($generic.body | ConvertTo-Json -Compress -Depth 8) }) `
    -FailureMessage "Expected generic durable module.load_ephemeral gate to remain denied"

Send-AgentCommand -Command "agent system.honesty_report" -ExpectedMarker "RAIOS_AGENT_END system.honesty_report" -Name "m12-distribution:N5_honesty_report"
$honesty = (Get-LastAgentResponseJson -Method "system.honesty_report").body.result
$honestyOk = (
    $honesty.no_dishonest_overclaim -eq $true -and
    $honesty.external_no_overclaim -eq $true -and
    $honesty.external_acquisition.acquisition_active -eq $false -and
    $honesty.owner_sealed -eq $false -and
    $honesty.trust_tier -eq "dev_key_not_owner_sealed"
)
Assert-M12Predicate `
    -Name "m12-distribution:N5_honesty_report_unchanged" `
    -Expected "system.honesty_report still reports no live external acquisition and dev-key-not-owner-sealed" `
    -Passed $honestyOk `
    -Actual $(if ($honestyOk) { "matched" } else { ($honesty | ConvertTo-Json -Compress -Depth 10) }) `
    -FailureMessage "Expected system.honesty_report standing external acquisition posture to remain unchanged"
