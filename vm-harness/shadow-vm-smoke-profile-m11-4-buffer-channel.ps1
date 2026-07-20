function Test-Sha256Ref {
    param([string]$Value)
    return $Value -match '^sha256:[0-9a-f]{64}$'
}

function Get-LastMarkerJsonAfterOffset {
    param(
        [string]$Prefix,
        [int]$Offset
    )

    $content = Get-SerialLogContent -Path $SerialLog
    if ($null -eq $content) {
        throw "No serial log content found in $SerialLog"
    }
    $slice = if ($Offset -lt $content.Length) { $content.Substring($Offset) } else { "" }
    $markerIndex = $slice.LastIndexOf($Prefix, [System.StringComparison]::Ordinal)
    if ($markerIndex -lt 0) {
        throw "Expected marker '$Prefix' after serial offset $Offset"
    }

    $jsonStart = $markerIndex + $Prefix.Length
    $lineEnd = $slice.IndexOf("`n", $jsonStart, [System.StringComparison]::Ordinal)
    if ($lineEnd -lt 0) {
        $lineEnd = $slice.Length
    }
    $json = $slice.Substring($jsonStart, $lineEnd - $jsonStart).Trim()
    return $json | ConvertFrom-Json
}

$echoOffset = Get-SerialLogOffset
Send-AgentCommand -Command "module.load_ephemeral svc.demo.echo" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "m11-buffer:echo-load"
Send-AgentCommand -Command "service.start svc.demo.echo" -ExpectedMarker "RAIOS_AGENT_END service.start" -Name "m11-buffer:echo-start"
$echoStart = Get-LastAgentResponseJson -Method "service.start"
$echoStartResult = $echoStart.body.result
$echoRun = $echoStartResult.run_evidence
$echoGrant = Get-LastMarkerJsonAfterOffset -Prefix "WASM_IMPORT_GRANT " -Offset $echoOffset
$echoWindow = (Get-SerialLogContent -Path $SerialLog).Substring([int]$echoOffset)
$echoOk = (
    $echoStart.body.method -eq "service.start" -and
    $echoStartResult.service_id -eq "svc.demo.echo" -and
    $echoStartResult.running -eq $true -and
    $echoRun.validation_ok -eq $true -and
    $echoRun.instantiation_ok -eq $true -and
    $echoRun.run_outcome -eq "success" -and
    $echoWindow.Contains("WASM_GUEST_LOG echo counter=") -and
    $echoGrant.service_id -eq "svc.demo.echo" -and
    $echoGrant.import_grant_performed -eq $true -and
    $echoGrant.module_imports_within_authorized_list -eq $true -and
    [int]$echoGrant.authorized_import_count -eq 2 -and
    [int]$echoGrant.linked_host_import_count -eq 2 -and
    (Test-Sha256Ref -Value $echoGrant.authorized_import_list_sha256)
)
Add-Predicate -Name "m11-buffer:echo-unchanged" -Expected "svc.demo.echo still runs with only env.log/env.counter_get and emits its guest log" -Passed $echoOk -Actual $(if ($echoOk) { "matched" } else { "start=$($echoStartResult | ConvertTo-Json -Compress -Depth 6) grant=$($echoGrant | ConvertTo-Json -Compress -Depth 5)" })
if (-not $echoOk) {
    throw "Expected svc.demo.echo import surface and guest log to stay unchanged"
}

Send-AgentCommand -Command "wasm.bufecho_probe" -ExpectedMarker "RAIOS_AGENT_END wasm.bufecho_probe" -Name "m11-buffer:bufecho-probe"
$bufecho = Get-LastAgentResponseJson -Method "wasm.bufecho_probe"
$bufechoResult = $bufecho.body.result
# The import-grant audit record is well-formed (valid pinned record_id + matching
# evidence), but its DURABLE append is honestly fail-closed to RAM-only here: this
# focused current-boot profile does not provision the shared durable reclog region
# (durable persistence is proven only by the dedicated memory-durable profile). So
# append_denied_ram_only + a concrete non-empty audit_reason is the correct, strictly-
# more-restrictive posture; a durable-append value in THIS profile would be a surprise
# worth reviewing. audit_reason (M11-5b) surfaces the exact underlying denial reason.
$auditOk = (
    $bufechoResult.audit_dedupe -eq "append_denied_ram_only" -and
    -not [string]::IsNullOrWhiteSpace([string]$bufechoResult.audit_reason)
)
$roundtripOk = (
    $bufecho.body.method -eq "wasm.bufecho_probe" -and
    $bufechoResult.schema -eq "raios.wasm_bufecho_probe.v0" -and
    $bufechoResult.service_id -eq "svc.demo.bufecho" -and
    $bufechoResult.run_outcome -eq "success" -and
    [int]$bufechoResult.authorized_import_count -eq 3 -and
    [int]$bufechoResult.linked_host_import_count -eq 3 -and
    $bufechoResult.module_imports_within_authorized_list -eq $true -and
    [int]$bufechoResult.captured_output_len -eq [int]$bufechoResult.input_len -and
    $bufechoResult.captured_output_sha256 -eq $bufechoResult.input_sha256 -and
    (Test-Sha256Ref -Value $bufechoResult.input_sha256) -and
    $auditOk -and
    $bufechoResult.audit_record_id -eq "mem.capability_grant.wasm_import_surface.svc_demo_bufecho.current_boot.v0"
)
Add-Predicate -Name "m11-buffer:granted-roundtrip" -Expected "svc.demo.bufecho round-trips staged host bytes through only the three byte-buffer imports (durable audit honestly RAM-only in this profile)" -Passed $roundtripOk -Actual $(if ($roundtripOk) { "round-trip matched (len+sha256); import-grant audit RAM-only, reason=$($bufechoResult.audit_reason)" } else { ($bufechoResult | ConvertTo-Json -Compress -Depth 6) })
if (-not $roundtripOk) {
    throw "Expected svc.demo.bufecho to round-trip host-staged bytes with matching len+sha256 evidence"
}

$negative = $bufechoResult.negative
$negativeOk = (
    $negative.module_imports_within_authorized_list -eq $false -and
    $negative.run_outcome -eq "module_import_not_authorized" -and
    $negative.missing_import_module -eq "env" -and
    $negative.instantiation_ok -eq $false -and
    [int]$negative.captured_output_len -eq 0
)
Add-Predicate -Name "m11-buffer:refused-unless-granted" -Expected "svc.demo.bufecho refuses when only env.input_len is granted, before instantiation and without captured output" -Passed $negativeOk -Actual $(if ($negativeOk) { "matched" } else { ($negative | ConvertTo-Json -Compress -Depth 6) })
if (-not $negativeOk) {
    throw "Expected svc.demo.bufecho subset denial to fail closed before instantiation"
}
