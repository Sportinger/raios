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

function Invoke-M12CargoTool {
    param(
        [string]$Name,
        [string[]]$CargoArgs
    )

    $stderrPath = Join-Path $RunDir "m12-$Name-stderr.txt"
    Push-Location $RepoRoot
    try {
        $output = & cargo @CargoArgs 2>$stderrPath
        if ($LASTEXITCODE -ne 0) {
            $stderr = if (Test-Path -LiteralPath $stderrPath) { Get-Content -LiteralPath $stderrPath -Raw } else { "" }
            throw "cargo $Name failed with exit code $LASTEXITCODE`nstdout:`n$($output -join [Environment]::NewLine)`nstderr:`n$stderr"
        }
        return $output
    }
    finally {
        Pop-Location
    }
}

function New-M12HostStaticCasExport {
    param([string]$ArtifactPath)

    $casRoot = Join-Path $RunDir "m12-host-static-cas"
    $keyDir = Join-Path $casRoot "keys"
    $registryDir = Join-Path $casRoot "registry"
    $manifestPath = Join-Path $casRoot "svc.demo.echo.manifest.json"
    New-Item -ItemType Directory -Force -Path $casRoot, $keyDir, $registryDir | Out-Null

    Invoke-M12CargoTool -Name "ota-keygen" -CargoArgs @(
        "run", "--quiet", "-p", "ota-tools", "--bin", "ota-keygen", "--",
        "--output", $keyDir,
        "--context", "raios/m12-local-static-cas/v1"
    ) | Out-Null

    Invoke-M12CargoTool -Name "mod-sign" -CargoArgs @(
        "run", "--quiet", "-p", "ota-tools", "--bin", "mod-sign", "--",
        "--key", (Join-Path $keyDir "online-key.json"),
        "--cert", (Join-Path $keyDir "online.cert.json"),
        "--input", $ArtifactPath,
        "--output", $manifestPath,
        "--module-id", "svc.demo.echo",
        "--module-version", "m12-host-cas"
    ) | Out-Null

    Invoke-M12CargoTool -Name "registry-publish" -CargoArgs @(
        "run", "--quiet", "-p", "registry-tools", "--",
        "publish",
        "--registry", $registryDir,
        "--blob", $ArtifactPath,
        "--manifest", $manifestPath,
        "--root-pub", (Join-Path $keyDir "root.pub"),
        "--namespace", "modules"
    ) | Out-Null

    $exportJson = Invoke-M12CargoTool -Name "registry-distribution-export" -CargoArgs @(
        "run", "--quiet", "-p", "registry-tools", "--",
        "distribution-export",
        "--registry", $registryDir,
        "--namespace", "modules",
        "--name", "svc.demo.echo",
        "--tag", "m12-host-cas",
        "--chunk-count", "3",
        "--artifact-identity-descriptor", (Join-Path $RepoRoot "seed-kernel\descriptors\svc.demo.echo.wasm_artifact_identity.desc"),
        "--artifact-identity-public-key", (Join-Path $RepoRoot "seed-kernel\descriptors\svc.demo.echo.wasm_artifact_identity.p256.pub.hex"),
        "--artifact-identity-signature", (Join-Path $RepoRoot "seed-kernel\descriptors\svc.demo.echo.wasm_artifact_identity.p256.sig.der.hex"),
        "--load-descriptor", (Join-Path $RepoRoot "seed-kernel\descriptors\svc.demo.echo.current_boot_load.desc"),
        "--load-descriptor-public-key", (Join-Path $RepoRoot "seed-kernel\descriptors\svc.demo.echo.current_boot_load.p256.pub.hex"),
        "--load-descriptor-signature", (Join-Path $RepoRoot "seed-kernel\descriptors\svc.demo.echo.current_boot_load.p256.sig.der.hex")
    )

    return (($exportJson -join [Environment]::NewLine) | ConvertFrom-Json)
}

function Get-M12ExportCommand {
    param(
        [object]$Export,
        [string]$Prefix
    )
    $matches = @($Export.commands | Where-Object { $_.StartsWith($Prefix) })
    if ($matches.Count -ne 1) {
        throw "Expected exactly one exported command matching '$Prefix', found $($matches.Count)"
    }
    return $matches[0]
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
$hostCasExport = New-M12HostStaticCasExport -ArtifactPath $echoArtifactPath
$hostCasCommands = @($hostCasExport.commands)
$hostCasIdentity = $hostCasExport.receiver_identity
$hostCasReceiverIdentityCommand = Get-M12ExportCommand -Export $hostCasExport -Prefix "module.submit_distribution_receiver_identity "
$hostCasReceiverIdentityEvidenceCommands = @($hostCasCommands | Where-Object { $_.StartsWith("module.submit_distribution_receiver_identity_evidence ") })
$hostCasReceiverIdentityFinalizeCommand = Get-M12ExportCommand -Export $hostCasExport -Prefix "module.submit_distribution_receiver_identity_finalize "
$hostCasExportOk = (
    $hostCasExport.source_kind -eq "local_static_cas_registry" -and
    $hostCasExport.source_id -eq "host.local_static_cas" -and
    $hostCasExport.namespace -eq "modules" -and
    $hostCasExport.name -eq "svc_demo_echo" -and
    $hostCasExport.tag -eq "m12-host-cas" -and
    $hostCasExport.artifact_sha256 -eq $expectedShaBare -and
    [int]$hostCasExport.payload_len -eq 4205 -and
    [int]$hostCasExport.chunk_count -eq 3 -and
    @($hostCasExport.chunks).Count -eq 3 -and
    $hostCasExport.provenance_signature_hex -eq $provSigHex -and
    $hostCasCommands.Count -eq 14 -and
    $hostCasCommands[0] -eq "module.submit_distribution_catalog_entry $expectedSha 4205 3 sig:$provSigHex" -and
    $hostCasCommands[1].StartsWith("module.submit_distribution_receiver_identity $expectedSha ") -and
    $hostCasReceiverIdentityEvidenceCommands.Count -eq 6 -and
    $hostCasCommands[8] -eq "module.submit_distribution_receiver_identity_finalize $expectedSha" -and
    $hostCasCommands[9] -eq "module.submit_distribution_begin_from_catalog $expectedSha" -and
    $hostCasCommands[-1] -eq "module.submit_distribution_finalize" -and
    $hostCasExport.registry_payload_hash_blake3.Length -eq 64 -and
    $null -ne $hostCasIdentity -and
    $hostCasIdentity.classification -eq "local_only" -and
    $hostCasIdentity.service_id -eq "svc.demo.echo" -and
    $hostCasIdentity.artifact_id -eq "wasm:svc.demo.echo" -and
    $hostCasIdentity.artifact_identity_id -eq "builtin_artifact_identity.svc.demo.echo.wasm.v0" -and
    $hostCasIdentity.load_descriptor_id -eq "load_descriptor.current_boot.svc.demo.echo.v0" -and
    $hostCasIdentity.artifact_sha256 -eq $expectedShaBare -and
    $hostCasIdentity.artifact_identity_signature_verified -eq $true -and
    $hostCasIdentity.load_descriptor_signature_verified -eq $true -and
    $hostCasIdentity.artifact_hash_bound_by_identity -eq $true -and
    $hostCasIdentity.artifact_hash_bound_by_load_descriptor -eq $true -and
    $hostCasIdentity.load_descriptor_binds_artifact_identity -eq $true -and
    $hostCasIdentity.load_descriptor_authorizes_current_boot_wasm_execution -eq $true -and
    $hostCasIdentity.export_authorizes_load -eq $false -and
    $hostCasIdentity.export_authorizes_install -eq $false -and
    $hostCasIdentity.export_authorizes_execute -eq $false -and
    $hostCasIdentity.export_writes_persistent_state -eq $false -and
    $hostCasIdentity.requires_m6_m7_reverify_for_load -eq $true
)
Assert-M12Predicate `
    -Name "m12-distribution:T2_host_static_cas_export_commands" `
    -Expected "host-side local static/CAS registry export emits bounded serial distribution commands without granting authority" `
    -Passed $hostCasExportOk `
    -Actual $(if ($hostCasExportOk) { "matched" } else { ($hostCasExport | ConvertTo-Json -Compress -Depth 10) }) `
    -FailureMessage "Expected local static/CAS registry export to bind echo artifact to serial delivery commands"
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

Send-AgentCommand -Command $hostCasCommands[0] -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_catalog_entry" -Name "m12-distribution:T2_host_cas_catalog_entry"
$catalogEntry = Get-LastAgentResponseJson -Method "module.submit_distribution_catalog_entry"
$catalogEntryResult = $catalogEntry.body.result
$catalogEntryOk = (
    $catalogEntry.t -eq "response" -and
    $catalogEntryResult.source_id -eq "local.serial.catalog" -and
    $catalogEntryResult.entry_id -eq "local.catalog.distribution" -and
    $catalogEntryResult.content_sha256 -eq $expectedSha -and
    [int]$catalogEntryResult.total_length -eq 4205 -and
    [int]$catalogEntryResult.chunk_count -eq 3 -and
    $catalogEntryResult.receiver_identity_retained -eq $false -and
    $catalogEntryResult.retained_in_catalog -eq $true -and
    $catalogEntryResult.accepted -eq $true -and
    $catalogEntryResult.rejected -eq $false -and
    $catalogEntryResult.reason -eq "accepted_local_distribution_catalog_entry" -and
    (Test-M12RegistryDenials -Record $catalogEntryResult)
)
Assert-M12Predicate `
    -Name "m12-distribution:T2_local_catalog_entry_retained" `
    -Expected "host static/CAS export feeds a non-builtin local catalog entry retained in current_boot without granting authority" `
    -Passed $catalogEntryOk `
    -Actual $(if ($catalogEntryOk) { "matched" } else { ($catalogEntryResult | ConvertTo-Json -Compress -Depth 8) }) `
    -FailureMessage "Expected local catalog entry to be retained without authority"

Send-AgentCommand -Command $hostCasReceiverIdentityCommand -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_receiver_identity" -Name "m12-distribution:T2_host_cas_receiver_identity"
$catalogReceiverIdentity = Get-LastAgentResponseJson -Method "module.submit_distribution_receiver_identity"
$catalogReceiverIdentityResult = $catalogReceiverIdentity.body.result
$catalogReceiverIdentityRecord = $catalogReceiverIdentityResult.receiver_identity
$artifactIdentityDescriptorSha = "sha256:$($hostCasIdentity.artifact_identity_descriptor_sha256)"
$artifactIdentityPublicKeySha = "sha256:$($hostCasIdentity.artifact_identity_public_key_sha256)"
$artifactIdentitySignatureSha = "sha256:$($hostCasIdentity.artifact_identity_signature_sha256)"
$loadDescriptorSha = "sha256:$($hostCasIdentity.load_descriptor_sha256)"
$loadDescriptorPublicKeySha = "sha256:$($hostCasIdentity.load_descriptor_public_key_sha256)"
$loadDescriptorSignatureSha = "sha256:$($hostCasIdentity.load_descriptor_signature_sha256)"
$catalogReceiverIdentityOk = (
    $catalogReceiverIdentity.t -eq "response" -and
    $catalogReceiverIdentityResult.source_id -eq "local.serial.catalog" -and
    $catalogReceiverIdentityResult.entry_id -eq "local.catalog.distribution" -and
    $catalogReceiverIdentityResult.content_sha256 -eq $expectedSha -and
    $catalogReceiverIdentityResult.retained_in_catalog -eq $true -and
    $catalogReceiverIdentityResult.accepted -eq $true -and
    $catalogReceiverIdentityResult.rejected -eq $false -and
    $catalogReceiverIdentityResult.reason -eq "accepted_local_distribution_receiver_identity" -and
    $catalogReceiverIdentityResult.metadata_is_non_authorizing -eq $true -and
    $catalogReceiverIdentityResult.guest_signature_verification_performed -eq $false -and
    $catalogReceiverIdentityResult.requires_m6_m7_reverify_for_load -eq $true -and
    $catalogReceiverIdentityRecord.classification -eq "local_only" -and
    $catalogReceiverIdentityRecord.artifact_identity_descriptor_sha256 -eq $artifactIdentityDescriptorSha -and
    $catalogReceiverIdentityRecord.artifact_identity_public_key_sha256 -eq $artifactIdentityPublicKeySha -and
    $catalogReceiverIdentityRecord.artifact_identity_signature_sha256 -eq $artifactIdentitySignatureSha -and
    $catalogReceiverIdentityRecord.load_descriptor_sha256 -eq $loadDescriptorSha -and
    $catalogReceiverIdentityRecord.load_descriptor_public_key_sha256 -eq $loadDescriptorPublicKeySha -and
    $catalogReceiverIdentityRecord.load_descriptor_signature_sha256 -eq $loadDescriptorSignatureSha -and
    $catalogReceiverIdentityRecord.artifact_identity_signature_verified_by_host_export -eq $true -and
    $catalogReceiverIdentityRecord.load_descriptor_signature_verified_by_host_export -eq $true -and
    $catalogReceiverIdentityRecord.artifact_identity_signature_verified_by_guest -eq $false -and
    $catalogReceiverIdentityRecord.load_descriptor_signature_verified_by_guest -eq $false -and
    $catalogReceiverIdentityRecord.guest_signature_verification_performed -eq $false -and
    $catalogReceiverIdentityRecord.receiver_identity_complete -eq $false -and
    $catalogReceiverIdentityRecord.artifact_hash_bound_by_identity -eq $true -and
    $catalogReceiverIdentityRecord.artifact_hash_bound_by_load_descriptor -eq $true -and
    $catalogReceiverIdentityRecord.load_descriptor_binds_artifact_identity -eq $true -and
    $catalogReceiverIdentityRecord.load_descriptor_authorizes_current_boot_wasm_execution -eq $true -and
    $catalogReceiverIdentityRecord.metadata_is_non_authorizing -eq $true -and
    $catalogReceiverIdentityRecord.export_authorizes_load -eq $false -and
    $catalogReceiverIdentityRecord.export_authorizes_install -eq $false -and
    $catalogReceiverIdentityRecord.export_authorizes_execute -eq $false -and
    $catalogReceiverIdentityRecord.export_writes_persistent_state -eq $false -and
    $catalogReceiverIdentityRecord.requires_m6_m7_reverify_for_load -eq $true -and
    (Test-M12RegistryDenials -Record $catalogReceiverIdentityResult)
)
Assert-M12Predicate `
    -Name "m12-distribution:T2_receiver_identity_retained_in_catalog" `
    -Expected "host-verified receiver identity metadata is retained as current_boot RAM-only catalog evidence without authorizing load" `
    -Passed $catalogReceiverIdentityOk `
    -Actual $(if ($catalogReceiverIdentityOk) { "matched" } else { ($catalogReceiverIdentityResult | ConvertTo-Json -Compress -Depth 10) }) `
    -FailureMessage "Expected receiver identity metadata to be retained without authority"

Send-AgentCommand -Command "module.distribution_receiver_identity_load_preflight $expectedSha" -ExpectedMarker "RAIOS_AGENT_END module.distribution_receiver_identity_load_preflight" -Name "m12-distribution:T2_receiver_identity_preflight_requires_complete_evidence"
$receiverPreEvidencePreflight = Get-LastAgentResponseJson -Method "module.distribution_receiver_identity_load_preflight"
$receiverPreEvidencePreflightResult = $receiverPreEvidencePreflight.body.result
$receiverPreEvidencePreflightOk = (
    $receiverPreEvidencePreflight.t -eq "response" -and
    $receiverPreEvidencePreflightResult.status -eq "denied" -and
    $receiverPreEvidencePreflightResult.reason -eq "distribution_receiver_identity_evidence_incomplete" -and
    $receiverPreEvidencePreflightResult.content_sha256 -eq $expectedSha -and
    $receiverPreEvidencePreflightResult.receiver_identity_retained -eq $true -and
    $receiverPreEvidencePreflightResult.receiver_identity_complete -eq $false -and
    $receiverPreEvidencePreflightResult.guest_signature_verification_performed -eq $false -and
    $receiverPreEvidencePreflightResult.preflight_evaluated -eq $false -and
    $receiverPreEvidencePreflightResult.accepted -eq $false -and
    $receiverPreEvidencePreflightResult.rejected -eq $true -and
    [int]$receiverPreEvidencePreflightResult.missing_gate_count -eq 0 -and
    $receiverPreEvidencePreflightResult.can_load_now -eq $false -and
    $receiverPreEvidencePreflightResult.load_authorized -eq $false -and
    $receiverPreEvidencePreflightResult.install_authorized -eq $false -and
    (Test-M12RegistryDenials -Record $receiverPreEvidencePreflightResult)
)
Assert-M12Predicate `
    -Name "m12-distribution:T2_receiver_identity_load_preflight_requires_complete_evidence" `
    -Expected "receiver identity load preflight refuses to evaluate until guest-complete receiver evidence exists" `
    -Passed $receiverPreEvidencePreflightOk `
    -Actual $(if ($receiverPreEvidencePreflightOk) { "matched" } else { ($receiverPreEvidencePreflightResult | ConvertTo-Json -Compress -Depth 10) }) `
    -FailureMessage "Expected receiver identity load preflight to require complete guest evidence"

$receiverEvidenceAccepted = $true
foreach ($evidenceCommand in $hostCasReceiverIdentityEvidenceCommands) {
    $evidenceKind = ($evidenceCommand -split " ")[2]
    Send-AgentCommand -Command $evidenceCommand -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_receiver_identity_evidence" -Name "m12-distribution:T2_host_cas_receiver_identity_evidence_$evidenceKind"
    $receiverEvidence = Get-LastAgentResponseJson -Method "module.submit_distribution_receiver_identity_evidence"
    $receiverEvidenceResult = $receiverEvidence.body.result
    $receiverEvidenceAccepted = $receiverEvidenceAccepted -and (
        $receiverEvidence.t -eq "response" -and
        $receiverEvidenceResult.source_id -eq "local.serial.catalog" -and
        $receiverEvidenceResult.entry_id -eq "local.catalog.distribution" -and
        $receiverEvidenceResult.content_sha256 -eq $expectedSha -and
        $receiverEvidenceResult.evidence_kind -eq $evidenceKind -and
        [int]$receiverEvidenceResult.decoded_byte_len -gt 0 -and
        [int]$receiverEvidenceResult.retained_part_count -ge 1 -and
        [int]$receiverEvidenceResult.retained_part_count -le 6 -and
        $receiverEvidenceResult.receiver_identity_complete -eq $false -and
        $receiverEvidenceResult.accepted -eq $true -and
        $receiverEvidenceResult.rejected -eq $false -and
        $receiverEvidenceResult.reason -eq "accepted_local_distribution_receiver_identity_evidence" -and
        $receiverEvidenceResult.guest_signature_verification_performed -eq $false -and
        (Test-M12RegistryDenials -Record $receiverEvidenceResult)
    )
}
Assert-M12Predicate `
    -Name "m12-distribution:T2_receiver_identity_raw_evidence_retained" `
    -Expected "six bounded receiver identity evidence payloads are accepted into current_boot RAM without authority" `
    -Passed $receiverEvidenceAccepted `
    -Actual $(if ($receiverEvidenceAccepted) { "matched" } else { "one receiver identity evidence command was rejected" }) `
    -FailureMessage "Expected all receiver identity evidence payloads to be retained"

Send-AgentCommand -Command $hostCasReceiverIdentityFinalizeCommand -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_receiver_identity_finalize" -Name "m12-distribution:T2_host_cas_receiver_identity_finalize"
$receiverEvidenceFinalize = Get-LastAgentResponseJson -Method "module.submit_distribution_receiver_identity_finalize"
$receiverEvidenceFinalizeResult = $receiverEvidenceFinalize.body.result
$receiverEvidenceFinalizeRecord = $receiverEvidenceFinalizeResult.receiver_identity
$receiverEvidenceFinalizeOk = (
    $receiverEvidenceFinalize.t -eq "response" -and
    $receiverEvidenceFinalizeResult.source_id -eq "local.serial.catalog" -and
    $receiverEvidenceFinalizeResult.entry_id -eq "local.catalog.distribution" -and
    $receiverEvidenceFinalizeResult.content_sha256 -eq $expectedSha -and
    $receiverEvidenceFinalizeResult.evidence_kind -eq "finalize" -and
    [int]$receiverEvidenceFinalizeResult.retained_part_count -eq 6 -and
    $receiverEvidenceFinalizeResult.receiver_identity_complete -eq $true -and
    $receiverEvidenceFinalizeResult.accepted -eq $true -and
    $receiverEvidenceFinalizeResult.rejected -eq $false -and
    $receiverEvidenceFinalizeResult.reason -eq "local_distribution_receiver_identity_evidence_verified_by_guest" -and
    $receiverEvidenceFinalizeResult.guest_signature_verification_performed -eq $true -and
    $receiverEvidenceFinalizeRecord.artifact_identity_signature_verified_by_guest -eq $true -and
    $receiverEvidenceFinalizeRecord.load_descriptor_signature_verified_by_guest -eq $true -and
    $receiverEvidenceFinalizeRecord.receiver_identity_complete -eq $true -and
    $receiverEvidenceFinalizeRecord.export_authorizes_load -eq $false -and
    $receiverEvidenceFinalizeRecord.export_authorizes_install -eq $false -and
    $receiverEvidenceFinalizeRecord.export_authorizes_execute -eq $false -and
    $receiverEvidenceFinalizeRecord.export_writes_persistent_state -eq $false -and
    (Test-M12RegistryDenials -Record $receiverEvidenceFinalizeResult)
)
Assert-M12Predicate `
    -Name "m12-distribution:T2_receiver_identity_raw_evidence_guest_verified" `
    -Expected "guest re-verifies receiver identity descriptor/key/signature payloads before marking identity complete, still without authority" `
    -Passed $receiverEvidenceFinalizeOk `
    -Actual $(if ($receiverEvidenceFinalizeOk) { "matched" } else { ($receiverEvidenceFinalizeResult | ConvertTo-Json -Compress -Depth 10) }) `
    -FailureMessage "Expected guest receiver identity evidence verification to complete without authority"

Send-AgentCommand -Command "module.distribution_receiver_identity_load_preflight $expectedSha" -ExpectedMarker "RAIOS_AGENT_END module.distribution_receiver_identity_load_preflight" -Name "m12-distribution:T2_receiver_identity_preflight_requires_catalog_finalize_candidate"
$receiverLoadPreflight = Get-LastAgentResponseJson -Method "module.distribution_receiver_identity_load_preflight"
$receiverLoadPreflightResult = $receiverLoadPreflight.body.result
$receiverLoadPreflightIdentity = $receiverLoadPreflightResult.receiver_identity
$receiverLoadPreflightOk = (
    $receiverLoadPreflight.t -eq "response" -and
    $receiverLoadPreflightResult.status -eq "denied" -and
    $receiverLoadPreflightResult.reason -eq "distribution_receiver_identity_load_preflight_candidate_not_finalized" -and
    $receiverLoadPreflightResult.content_sha256 -eq $expectedSha -and
    $receiverLoadPreflightResult.receiver_identity_retained -eq $true -and
    $receiverLoadPreflightResult.receiver_identity_complete -eq $true -and
    $receiverLoadPreflightResult.guest_signature_verification_performed -eq $true -and
    $null -eq $receiverLoadPreflightResult.catalog_finalize_candidate_sha256 -and
    $receiverLoadPreflightResult.retained_candidate_matches_catalog_finalize -eq $false -and
    $receiverLoadPreflightResult.preflight_evaluated -eq $false -and
    $receiverLoadPreflightResult.accepted -eq $false -and
    $receiverLoadPreflightResult.rejected -eq $true -and
    [int]$receiverLoadPreflightResult.missing_gate_count -eq 0 -and
    $receiverLoadPreflightResult.can_load_now -eq $false -and
    $receiverLoadPreflightResult.load_authorized -eq $false -and
    $receiverLoadPreflightResult.install_authorized -eq $false -and
    $receiverLoadPreflightIdentity.receiver_identity_complete -eq $true -and
    $receiverLoadPreflightIdentity.artifact_identity_signature_verified_by_guest -eq $true -and
    $receiverLoadPreflightIdentity.load_descriptor_signature_verified_by_guest -eq $true -and
    (Test-M12RegistryDenials -Record $receiverLoadPreflightResult)
)
Assert-M12Predicate `
    -Name "m12-distribution:T2_receiver_identity_load_preflight_requires_catalog_finalize_candidate" `
    -Expected "receiver identity load preflight refuses to name missing load gates until a matching inert candidate was produced by catalog finalize" `
    -Passed $receiverLoadPreflightOk `
    -Actual $(if ($receiverLoadPreflightOk) { "matched" } else { ($receiverLoadPreflightResult | ConvertTo-Json -Compress -Depth 10) }) `
    -FailureMessage "Expected receiver identity load preflight to require a catalog-finalized retained candidate"

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

Send-AgentCommand -Command (Get-M12ExportCommand -Export $hostCasExport -Prefix "module.submit_distribution_begin_from_catalog ") -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_begin_from_catalog" -Name "m12-distribution:T2_host_cas_catalog_begin"
$catalogBegin = Get-LastAgentResponseJson -Method "module.submit_distribution_begin_from_catalog"
$catalogBeginResult = $catalogBegin.body.result
$catalogBeginIdentity = $catalogBeginResult.receiver_identity
$catalogBeginOk = (
    $catalogBeginResult.source_id -eq "local.serial.catalog" -and
    $catalogBeginResult.entry_id -eq "local.catalog.distribution" -and
    $catalogBeginResult.content_sha256 -eq $expectedSha -and
    [int]$catalogBeginResult.total_length -eq 4205 -and
    [int]$catalogBeginResult.chunk_count -eq 3 -and
    $catalogBeginResult.receiver_identity_retained -eq $true -and
    $catalogBeginIdentity.artifact_identity_descriptor_sha256 -eq $artifactIdentityDescriptorSha -and
    $catalogBeginIdentity.load_descriptor_sha256 -eq $loadDescriptorSha -and
    $catalogBeginIdentity.artifact_identity_signature_verified_by_guest -eq $true -and
    $catalogBeginIdentity.load_descriptor_signature_verified_by_guest -eq $true -and
    $catalogBeginIdentity.guest_signature_verification_performed -eq $true -and
    $catalogBeginIdentity.receiver_identity_complete -eq $true -and
    $catalogBeginIdentity.export_authorizes_load -eq $false -and
    $catalogBeginIdentity.requires_m6_m7_reverify_for_load -eq $true -and
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
foreach ($chunkCommand in @($hostCasCommands | Where-Object { $_.StartsWith("module.submit_distribution_chunk ") })) {
    $chunkIndex = ($chunkCommand -split " ")[1]
    Send-AgentCommand -Command $chunkCommand -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_chunk" -Name "m12-distribution:T2_host_cas_catalog_chunk_$chunkIndex"
}
Send-AgentCommand -Command (Get-M12ExportCommand -Export $hostCasExport -Prefix "module.submit_distribution_finalize") -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_finalize" -Name "m12-distribution:T2_host_cas_catalog_finalize"
$catalogFinalize = Get-LastAgentResponseJson -Method "module.submit_distribution_finalize"
$catalogResult = $catalogFinalize.body.result
$catalogSelection = $catalogResult.selection
$catalogStaged = $catalogResult.staged_candidate
$catalogRetained = $catalogResult.retained_provenance
$catalogReceiverIdentity = $catalogResult.receiver_identity
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
    $catalogResult.receiver_identity_retained -eq $true -and
    $catalogReceiverIdentity.artifact_identity_descriptor_sha256 -eq $artifactIdentityDescriptorSha -and
    $catalogReceiverIdentity.load_descriptor_sha256 -eq $loadDescriptorSha -and
    $catalogReceiverIdentity.artifact_identity_signature_verified_by_guest -eq $true -and
    $catalogReceiverIdentity.load_descriptor_signature_verified_by_guest -eq $true -and
    $catalogReceiverIdentity.guest_signature_verification_performed -eq $true -and
    $catalogReceiverIdentity.receiver_identity_complete -eq $true -and
    $catalogReceiverIdentity.export_authorizes_load -eq $false -and
    $catalogReceiverIdentity.requires_m6_m7_reverify_for_load -eq $true -and
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

Send-AgentCommand -Command "module.distribution_receiver_identity_load_preflight $expectedSha" -ExpectedMarker "RAIOS_AGENT_END module.distribution_receiver_identity_load_preflight" -Name "m12-distribution:T2_receiver_identity_load_preflight_missing_gates"
$catalogBoundPreflight = Get-LastAgentResponseJson -Method "module.distribution_receiver_identity_load_preflight"
$catalogBoundPreflightResult = $catalogBoundPreflight.body.result
$catalogBoundPreflightIdentity = $catalogBoundPreflightResult.receiver_identity
$catalogBoundPreflightOk = (
    $catalogBoundPreflight.t -eq "response" -and
    $catalogBoundPreflightResult.status -eq "denied" -and
    $catalogBoundPreflightResult.reason -eq "distribution_receiver_identity_load_preflight_missing_required_gates" -and
    $catalogBoundPreflightResult.content_sha256 -eq $expectedSha -and
    $catalogBoundPreflightResult.receiver_identity_retained -eq $true -and
    $catalogBoundPreflightResult.receiver_identity_complete -eq $true -and
    $catalogBoundPreflightResult.guest_signature_verification_performed -eq $true -and
    $catalogBoundPreflightResult.retained_candidate_sha256 -eq $expectedSha -and
    $catalogBoundPreflightResult.retained_candidate_present -eq $true -and
    $catalogBoundPreflightResult.retained_candidate_wasm_valid -eq $true -and
    $catalogBoundPreflightResult.catalog_finalize_candidate_sha256 -eq $expectedSha -and
    $catalogBoundPreflightResult.retained_candidate_matches_catalog_finalize -eq $true -and
    $catalogBoundPreflightResult.preflight_evaluated -eq $true -and
    $catalogBoundPreflightResult.accepted -eq $true -and
    $catalogBoundPreflightResult.rejected -eq $false -and
    [int]$catalogBoundPreflightResult.missing_gate_count -eq 4 -and
    $catalogBoundPreflightResult.m6_reverification_gate_satisfied -eq $false -and
    $catalogBoundPreflightResult.m7_loader_policy_gate_satisfied -eq $false -and
    $catalogBoundPreflightResult.provider_trust_gate_satisfied -eq $false -and
    $catalogBoundPreflightResult.owner_seal_gate_satisfied -eq $false -and
    $catalogBoundPreflightResult.requires_m6_m7_reverify_for_load -eq $true -and
    $catalogBoundPreflightResult.requires_provider_trust_for_load -eq $true -and
    $catalogBoundPreflightResult.requires_owner_seal_for_load -eq $true -and
    $catalogBoundPreflightResult.can_load_now -eq $false -and
    $catalogBoundPreflightResult.load_authorized -eq $false -and
    $catalogBoundPreflightResult.install_authorized -eq $false -and
    $catalogBoundPreflightIdentity.receiver_identity_complete -eq $true -and
    $catalogBoundPreflightIdentity.artifact_identity_signature_verified_by_guest -eq $true -and
    $catalogBoundPreflightIdentity.load_descriptor_signature_verified_by_guest -eq $true -and
    (Test-M12RegistryDenials -Record $catalogBoundPreflightResult)
)
Assert-M12Predicate `
    -Name "m12-distribution:T2_receiver_identity_load_preflight_names_missing_gates" `
    -Expected "receiver identity load preflight names missing gates only after the catalog-finalized retained candidate hash matches the guest-complete receiver identity" `
    -Passed $catalogBoundPreflightOk `
    -Actual $(if ($catalogBoundPreflightOk) { "matched" } else { ($catalogBoundPreflightResult | ConvertTo-Json -Compress -Depth 10) }) `
    -FailureMessage "Expected receiver identity load preflight to bind the catalog-finalized candidate before naming missing gates"

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
$loadPreflightProjection = $load.body.receiver_identity_load_preflight
$loadRuntimeReadiness = $load.body.loader_runtime_readiness
$loadPreflightSourceFacts = @($loadRuntimeReadiness.source_fact_map | Where-Object { $_.fact -eq "receiver_identity_load_preflight" })
$loadPreflightSourceFact = if ($loadPreflightSourceFacts.Count -eq 1) { $loadPreflightSourceFacts[0] } else { $null }
$loadPreflightRuntimeFact = $loadRuntimeReadiness.loader_runtime_facts.receiver_identity_load_preflight
$loadM6M7ReverifyInputCheck = $loadRuntimeReadiness.m6_m7_reverify_input_check
$loadM6ReverifyInputDiagnostic = $loadM6M7ReverifyInputCheck.m6_reverify_input_diagnostic
$loadM7LoaderPolicyInputDiagnostic = $loadM6M7ReverifyInputCheck.m7_loader_policy_input_diagnostic
$loadProviderTrustInputDiagnostic = $loadM6M7ReverifyInputCheck.provider_trust_input_diagnostic
$loadAfter = (Get-SerialLogContent -Path $SerialLog).Substring([int]$loadOffset)
$loadDeniedOk = (
    $load.t -eq "error" -and
    $load.body.code -eq "capability_denied" -and
    $load.body.schema -eq "raios.module_load_gate.v0" -and
    $load.body.gate_state.can_load -eq $false -and
    [int]$loadRuntimeReadiness.source_fact_count -eq 11 -and
    $loadRuntimeReadiness.source_fact_map_complete -eq $true -and
    $loadPreflightSourceFacts.Count -eq 1 -and
    $loadPreflightSourceFact.schema -eq "raios.module_load_gate.v0" -and
    $loadPreflightSourceFact.source_method -eq "module.distribution_receiver_identity_load_preflight" -and
    $loadPreflightSourceFact.source_fact_locator -eq "module.load_ephemeral.receiver_identity_load_preflight" -and
    $loadPreflightSourceFact.present -eq $true -and
    $loadPreflightSourceFact.status -eq "denied" -and
    $loadPreflightSourceFact.reason -eq "distribution_receiver_identity_load_preflight_missing_required_gates" -and
    $loadPreflightSourceFact.receiver_identity_complete -eq $true -and
    $loadPreflightSourceFact.retained_candidate_matches_catalog_finalize -eq $true -and
    $loadPreflightSourceFact.preflight_evaluated -eq $true -and
    $loadPreflightSourceFact.can_load_now -eq $false -and
    $loadPreflightSourceFact.authorizes_load -eq $false -and
    $loadPreflightRuntimeFact.present -eq $true -and
    $loadPreflightRuntimeFact.status -eq "denied" -and
    $loadPreflightRuntimeFact.reason -eq "distribution_receiver_identity_load_preflight_missing_required_gates" -and
    $loadPreflightRuntimeFact.content_sha256 -eq $expectedSha -and
    $loadPreflightRuntimeFact.receiver_identity_complete -eq $true -and
    $loadPreflightRuntimeFact.retained_candidate_matches_catalog_finalize -eq $true -and
    $loadPreflightRuntimeFact.preflight_evaluated -eq $true -and
    $loadPreflightRuntimeFact.can_load_now -eq $false -and
    $loadPreflightRuntimeFact.authorizes_load -eq $false -and
    $loadM6M7ReverifyInputCheck.source_fact -eq "receiver_identity_load_preflight" -and
    $loadM6M7ReverifyInputCheck.source_fact_locator -eq "module.load_ephemeral.receiver_identity_load_preflight" -and
    $loadM6M7ReverifyInputCheck.source_fact_present -eq $true -and
    $loadM6M7ReverifyInputCheck.source_fact_ready_for_reverify -eq $true -and
    $loadM6M7ReverifyInputCheck.source_fact_status -eq "denied" -and
    $loadM6M7ReverifyInputCheck.source_fact_reason -eq "distribution_receiver_identity_load_preflight_missing_required_gates" -and
    $loadM6M7ReverifyInputCheck.status -eq "denied_missing_m6_m7_reverify_inputs" -and
    $loadM6M7ReverifyInputCheck.reason -eq "m6_m7_reverify_inputs_missing" -and
    $loadM6ReverifyInputDiagnostic.id -eq "module.load_ephemeral.m6_reverify_input_diagnostic.current_boot" -and
    $loadM6ReverifyInputDiagnostic.consumes_check -eq "module.load_ephemeral.m6_m7_reverify_input_check.current_boot" -and
    $loadM6ReverifyInputDiagnostic.receiver_preflight_input_ready -eq $true -and
    $loadM6ReverifyInputDiagnostic.receiver_candidate_binding_absent -eq $false -and
    $loadM6ReverifyInputDiagnostic.status -eq "denied_missing_m6_reverify_evidence" -and
    $loadM6ReverifyInputDiagnostic.reason -eq "m6_reverification_evidence_missing" -and
    $loadM6ReverifyInputDiagnostic.m6_reverification_evidence_present -eq $false -and
    $loadM6ReverifyInputDiagnostic.m6_reverification_evidence_reason -eq "m6_reverification_evidence_missing" -and
    $loadM6ReverifyInputDiagnostic.can_enter_m6_reverify -eq $false -and
    $loadM6ReverifyInputDiagnostic.authorizes_load -eq $false -and
    $loadM7LoaderPolicyInputDiagnostic.id -eq "module.load_ephemeral.m7_loader_policy_input_diagnostic.current_boot" -and
    $loadM7LoaderPolicyInputDiagnostic.consumes_diagnostic -eq "module.load_ephemeral.m6_reverify_input_diagnostic.current_boot" -and
    $loadM7LoaderPolicyInputDiagnostic.consumes_check -eq "module.load_ephemeral.m6_m7_reverify_input_check.current_boot" -and
    $loadM7LoaderPolicyInputDiagnostic.m6_reverify_input_ready_for_loader_policy -eq $false -and
    $loadM7LoaderPolicyInputDiagnostic.m6_reverify_input_status -eq "denied_missing_m6_reverify_evidence" -and
    $loadM7LoaderPolicyInputDiagnostic.m6_reverify_input_reason -eq "m6_reverification_evidence_missing" -and
    $loadM7LoaderPolicyInputDiagnostic.m6_reverification_evidence_present -eq $false -and
    $loadM7LoaderPolicyInputDiagnostic.m7_loader_policy_evidence_present -eq $false -and
    $loadM7LoaderPolicyInputDiagnostic.m7_loader_policy_evidence_reason -eq "m7_loader_policy_evidence_missing" -and
    $loadM7LoaderPolicyInputDiagnostic.status -eq "denied_m6_reverify_input_not_ready_for_m7_loader_policy" -and
    $loadM7LoaderPolicyInputDiagnostic.reason -eq "m6_reverification_evidence_missing" -and
    $loadM7LoaderPolicyInputDiagnostic.can_enter_m7_loader_policy -eq $false -and
    $loadM7LoaderPolicyInputDiagnostic.can_load_now -eq $false -and
    $loadM7LoaderPolicyInputDiagnostic.authorizes_load -eq $false -and
    $loadProviderTrustInputDiagnostic.id -eq "module.load_ephemeral.provider_trust_input_diagnostic.current_boot" -and
    $loadProviderTrustInputDiagnostic.consumes_diagnostic -eq "module.load_ephemeral.m7_loader_policy_input_diagnostic.current_boot" -and
    $loadProviderTrustInputDiagnostic.consumes_check -eq "module.load_ephemeral.m6_m7_reverify_input_check.current_boot" -and
    $loadProviderTrustInputDiagnostic.m7_loader_policy_input_ready_for_provider_trust -eq $false -and
    $loadProviderTrustInputDiagnostic.m7_loader_policy_input_status -eq "denied_m6_reverify_input_not_ready_for_m7_loader_policy" -and
    $loadProviderTrustInputDiagnostic.m7_loader_policy_input_reason -eq "m6_reverification_evidence_missing" -and
    $loadProviderTrustInputDiagnostic.m7_loader_policy_evidence_present -eq $false -and
    $loadProviderTrustInputDiagnostic.m7_loader_policy_evidence_reason -eq "m7_loader_policy_evidence_missing" -and
    $loadProviderTrustInputDiagnostic.provider_trust_evidence_present -eq $false -and
    $loadProviderTrustInputDiagnostic.provider_trust_evidence_reason -eq "provider_trust_evidence_missing" -and
    $loadProviderTrustInputDiagnostic.provider_trust_positive -eq $false -and
    $loadProviderTrustInputDiagnostic.status -eq "denied_m7_loader_policy_input_not_ready_for_provider_trust" -and
    $loadProviderTrustInputDiagnostic.reason -eq "m7_loader_policy_evidence_missing" -and
    $loadProviderTrustInputDiagnostic.can_enter_provider_trust -eq $false -and
    $loadProviderTrustInputDiagnostic.can_load_now -eq $false -and
    $loadProviderTrustInputDiagnostic.authorizes_load -eq $false -and
    $loadM6M7ReverifyInputCheck.receiver_identity_complete -eq $true -and
    $loadM6M7ReverifyInputCheck.retained_candidate_matches_catalog_finalize -eq $true -and
    $loadM6M7ReverifyInputCheck.preflight_evaluated -eq $true -and
    $loadM6M7ReverifyInputCheck.m6_reverification_input_present -eq $false -and
    $loadM6M7ReverifyInputCheck.m6_reverification_input_reason -eq "m6_reverification_evidence_missing" -and
    $loadM6M7ReverifyInputCheck.m7_loader_policy_input_present -eq $false -and
    $loadM6M7ReverifyInputCheck.m7_loader_policy_input_reason -eq "m7_loader_policy_evidence_missing" -and
    $loadM6M7ReverifyInputCheck.m6_reverification_gate_satisfied -eq $false -and
    $loadM6M7ReverifyInputCheck.m7_loader_policy_gate_satisfied -eq $false -and
    $loadM6M7ReverifyInputCheck.can_enter_m6_reverify -eq $false -and
    $loadM6M7ReverifyInputCheck.can_enter_m7_loader_policy -eq $false -and
    $loadM6M7ReverifyInputCheck.can_load_now -eq $false -and
    $loadM6M7ReverifyInputCheck.authorizes_load -eq $false -and
    $loadPreflightProjection.present -eq $true -and
    $loadPreflightProjection.status -eq "denied" -and
    $loadPreflightProjection.reason -eq "distribution_receiver_identity_load_preflight_missing_required_gates" -and
    $loadPreflightProjection.content_sha256 -eq $expectedSha -and
    $loadPreflightProjection.receiver_identity_retained -eq $true -and
    $loadPreflightProjection.receiver_identity_complete -eq $true -and
    $loadPreflightProjection.guest_signature_verification_performed -eq $true -and
    $loadPreflightProjection.retained_candidate_sha256 -eq $expectedSha -and
    $loadPreflightProjection.retained_candidate_present -eq $true -and
    $loadPreflightProjection.retained_candidate_wasm_valid -eq $true -and
    $loadPreflightProjection.catalog_finalize_candidate_sha256 -eq $expectedSha -and
    $loadPreflightProjection.retained_candidate_matches_catalog_finalize -eq $true -and
    $loadPreflightProjection.preflight_evaluated -eq $true -and
    $loadPreflightProjection.accepted -eq $true -and
    $loadPreflightProjection.rejected -eq $false -and
    [int]$loadPreflightProjection.missing_gate_count -eq 4 -and
    $loadPreflightProjection.m6_reverification_gate_satisfied -eq $false -and
    $loadPreflightProjection.m7_loader_policy_gate_satisfied -eq $false -and
    $loadPreflightProjection.can_load_now -eq $false -and
    $loadPreflightProjection.load_authorized -eq $false -and
    $loadPreflightProjection.install_authorized -eq $false -and
    (Test-M12RegistryDenials -Record $loadPreflightProjection) -and
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
    $honesty.owner_key_no_overclaim -eq $true -and
    $honesty.external_acquisition.acquisition_active -eq $false -and
    $honesty.owner_key_provisioning.id -eq "owner_key.provisioning.posture.current_boot" -and
    $honesty.owner_key_provisioning.automatic_generation_intended -eq $true -and
    $honesty.owner_key_provisioning.automatic_generation_performed -eq $true -and
    $honesty.owner_key_provisioning.persistent_install_policy -eq "generate_hardware_bound_owner_key_on_persistent_install" -and
    $honesty.owner_key_provisioning.ram_boot_policy -eq "ephemeral_current_boot_key_only" -and
    $honesty.owner_key_provisioning.persistent_install_hardware_binding_required -eq $true -and
    $honesty.owner_key_provisioning.entropy_evidence_present -eq $true -and
    $honesty.owner_key_provisioning.hardware_binding_probe_source -eq "acpi_tpm2_table" -and
    $honesty.owner_key_provisioning.hardware_binding_probe_performed -eq $true -and
    $honesty.owner_key_provisioning.hardware_binding_probe_status -eq "tpm2_acpi_absent" -and
    $honesty.owner_key_provisioning.hardware_binding_probe_reason -eq "TPM2 ACPI table missing" -and
    $honesty.owner_key_provisioning.tpm2_acpi_table_present -eq $false -and
    [int64]$honesty.owner_key_provisioning.tpm2_acpi_table_phys -eq 0 -and
    [int]$honesty.owner_key_provisioning.tpm2_acpi_table_length -eq 0 -and
    [int]$honesty.owner_key_provisioning.tpm2_acpi_table_revision -eq 0 -and
    $honesty.owner_key_provisioning.tpm2_table_details_valid -eq $false -and
    [int]$honesty.owner_key_provisioning.tpm2_platform_class -eq 0 -and
    [int64]$honesty.owner_key_provisioning.tpm2_control_area -eq 0 -and
    [int]$honesty.owner_key_provisioning.tpm2_start_method -eq 0 -and
    $honesty.owner_key_provisioning.tpm2_interface_kind -eq "none" -and
    $honesty.owner_key_provisioning.tpm2_interface_status_probe_performed -eq $true -and
    $honesty.owner_key_provisioning.tpm2_interface_status -eq "tpm2_acpi_absent" -and
    $honesty.owner_key_provisioning.tpm2_interface_status_reason -eq "TPM2 ACPI table missing" -and
    $honesty.owner_key_provisioning.tpm2_status_read_plan_available -eq $false -and
    $honesty.owner_key_provisioning.tpm2_status_register_kind -eq "none" -and
    [int64]$honesty.owner_key_provisioning.tpm2_status_register_phys -eq 0 -and
    [int]$honesty.owner_key_provisioning.tpm2_status_register_width_bytes -eq 0 -and
    $honesty.owner_key_provisioning.tpm2_status_read_plan_reason -eq "TPM2 ACPI table missing" -and
    $honesty.owner_key_provisioning.hardware_binding_evidence_present -eq $false -and
    $honesty.owner_key_provisioning.ram_boot_ephemeral_input_ready -eq $true -and
    $honesty.owner_key_provisioning.persistent_install_input_ready -eq $false -and
    $honesty.owner_key_provisioning.persistent_owner_key_generated -eq $false -and
    $honesty.owner_key_provisioning.ram_boot_ephemeral_key_allowed -eq $true -and
    $honesty.owner_key_provisioning.ram_boot_entropy_required -eq $true -and
    $honesty.owner_key_provisioning.ram_boot_ephemeral_key_generated -eq $true -and
    $honesty.owner_key_provisioning.ram_boot_ephemeral_key_id -eq "owner_key.ram_candidate.current_boot" -and
    $honesty.owner_key_provisioning.ram_boot_ephemeral_key_handle -eq "owner_key.handle.current_boot.ram0" -and
    $honesty.owner_key_provisioning.ram_boot_ephemeral_key_algorithm -eq "ram_32_byte_entropy_seed_sha256_fingerprint" -and
    [int]$honesty.owner_key_provisioning.ram_boot_ephemeral_key_secret_len -eq 32 -and
    $honesty.owner_key_provisioning.ram_boot_ephemeral_key_material_classification -eq "secret" -and
    $honesty.owner_key_provisioning.ram_boot_ephemeral_key_fingerprint -match '^sha256:[0-9a-f]{64}$' -and
    $honesty.owner_key_provisioning.owner_key_material_exported -eq $false -and
    $honesty.owner_key_provisioning.status -eq "ram_ephemeral_candidate_generated_persistent_hardware_binding_missing" -and
    $honesty.owner_key_provisioning.reason -eq "hardware_bound_owner_key_evidence_missing" -and
    $honesty.owner_key_provisioning.authorizes_owner_seal -eq $false -and
    $honesty.owner_key_provisioning.authorizes_persistent_install -eq $false -and
    $honesty.owner_key_provisioning.authorizes_load -eq $false -and
    $honesty.owner_key_provisioning.durable_write -eq $false -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.id -eq "owner_key.evidence_input.current_boot" -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.consumes_entropy_source -eq "core.entropy" -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.entropy_evidence_present -eq $true -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.entropy_status -eq "ready" -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.entropy_source_rdrand_observed -eq $true -and
    [int]$honesty.owner_key_provisioning.owner_key_evidence_input.entropy_pool_fill -ge 0 -and
    [int]$honesty.owner_key_provisioning.owner_key_evidence_input.entropy_pool_fill -le 64 -and
    [int]$honesty.owner_key_provisioning.owner_key_evidence_input.entropy_pool_capacity -eq 64 -and
    [int]$honesty.owner_key_provisioning.owner_key_evidence_input.entropy_total_collected -ge 32 -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.hardware_binding_source -eq "tpm_or_platform_seal" -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.hardware_binding_probe_source -eq "acpi_tpm2_table" -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.hardware_binding_probe_performed -eq $true -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.hardware_binding_probe_status -eq "tpm2_acpi_absent" -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.hardware_binding_probe_reason -eq "TPM2 ACPI table missing" -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.acpi_rsdp_present -eq $true -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.acpi_root_table_valid -eq $true -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.tpm2_acpi_table_present -eq $false -and
    [int64]$honesty.owner_key_provisioning.owner_key_evidence_input.tpm2_acpi_table_phys -eq 0 -and
    [int]$honesty.owner_key_provisioning.owner_key_evidence_input.tpm2_acpi_table_length -eq 0 -and
    [int]$honesty.owner_key_provisioning.owner_key_evidence_input.tpm2_acpi_table_revision -eq 0 -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.tpm2_table_details_valid -eq $false -and
    [int]$honesty.owner_key_provisioning.owner_key_evidence_input.tpm2_platform_class -eq 0 -and
    [int64]$honesty.owner_key_provisioning.owner_key_evidence_input.tpm2_control_area -eq 0 -and
    [int]$honesty.owner_key_provisioning.owner_key_evidence_input.tpm2_start_method -eq 0 -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.tpm2_interface_kind -eq "none" -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.tpm2_interface_status_probe_performed -eq $true -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.tpm2_interface_status -eq "tpm2_acpi_absent" -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.tpm2_interface_status_reason -eq "TPM2 ACPI table missing" -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.tpm2_status_read_plan_available -eq $false -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.tpm2_status_register_kind -eq "none" -and
    [int64]$honesty.owner_key_provisioning.owner_key_evidence_input.tpm2_status_register_phys -eq 0 -and
    [int]$honesty.owner_key_provisioning.owner_key_evidence_input.tpm2_status_register_width_bytes -eq 0 -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.tpm2_status_read_plan_reason -eq "TPM2 ACPI table missing" -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.hardware_binding_evidence_present -eq $false -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.tpm_binding_state -eq "tpm2_acpi_absent" -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.ram_boot_ephemeral_input_ready -eq $true -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.persistent_install_input_ready -eq $false -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.status -eq "ram_ephemeral_input_ready_persistent_hardware_binding_missing" -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.reason -eq "hardware_bound_owner_key_evidence_missing" -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.ram_candidate_generated -eq $true -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.ram_candidate_id -eq "owner_key.ram_candidate.current_boot" -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.ram_candidate_handle -eq "owner_key.handle.current_boot.ram0" -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.ram_candidate_fingerprint -eq $honesty.owner_key_provisioning.ram_boot_ephemeral_key_fingerprint -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.authorizes_key_generation -eq $false -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.authorizes_owner_seal -eq $false -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.authorizes_persistent_install -eq $false -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.authorizes_load -eq $false -and
    $honesty.owner_key_provisioning.owner_key_evidence_input.durable_write -eq $false -and
    $honesty.owner_sealed -eq $false -and
    $honesty.trust_tier -eq "dev_key_not_owner_sealed"
)
Assert-M12Predicate `
    -Name "m12-distribution:N5_owner_key_provisioning_posture" `
    -Expected "system.honesty_report reports external acquisition inert and owner-key provisioning unsealed" `
    -Passed $honestyOk `
    -Actual $(if ($honestyOk) { "matched" } else { ($honesty | ConvertTo-Json -Compress -Depth 10) }) `
    -FailureMessage "Expected system.honesty_report owner-key provisioning posture to remain unsealed and non-authorizing"

Send-AgentCommand -Command "ownerkey" -ExpectedMarker "OWNER AUTH: SEAL NO PERSIST NO LOAD NO DURABLE NO" -Name "m12-distribution:N6_ownerkey_console_status"
Assert-LogContains -Name "m12-distribution:N6_ownerkey_ram_candidate" -Needle "OWNER KEY: RAM YES HANDLE owner_key.handle.current_boot.ram0" -TimeoutSeconds 1
Assert-LogContains -Name "m12-distribution:N6_ownerkey_fingerprint_redacted" -Needle "OWNER KEY FINGERPRINT: sha256:" -TimeoutSeconds 1
Assert-LogContains -Name "m12-distribution:N6_ownerkey_tpm2_absent" -Needle "TPM2 ACPI: PRESENT NO PHYS 0x0000000000000000 LEN 0 REV 0" -TimeoutSeconds 1
Assert-LogContains -Name "m12-distribution:N6_ownerkey_tpm2_interface_absent" -Needle "TPM2 IFACE: KIND none START 0 CONTROL 0x0000000000000000 DETAILS NO" -TimeoutSeconds 1
Assert-LogContains -Name "m12-distribution:N6_ownerkey_tpm2_status_reason" -Needle "TPM2 STATUS: tpm2_acpi_absent REASON TPM2 ACPI table missing" -TimeoutSeconds 1
Assert-LogContains -Name "m12-distribution:N6_ownerkey_tpm2_status_read_absent" -Needle "TPM2 STATUS READ: PLAN NO KIND none PHYS 0x0000000000000000 WIDTH 0 REASON TPM2 ACPI table missing" -TimeoutSeconds 1
