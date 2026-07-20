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
Send-AgentCommand -Command "module.load_ephemeral svc.demo.echo" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "m11-import-grant:echo-load"
Send-AgentCommand -Command "service.start svc.demo.echo" -ExpectedMarker "RAIOS_AGENT_END service.start" -Name "m11-import-grant:echo-start"
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
Add-Predicate -Name "m11-import-grant:echo-runs-authorized" -Expected "svc.demo.echo runs with authorized env.log/env.counter_get import grant evidence" -Passed $echoOk -Actual $(if ($echoOk) { "matched" } else { "start=$($echoStartResult | ConvertTo-Json -Compress -Depth 6) grant=$($echoGrant | ConvertTo-Json -Compress -Depth 5)" })
if (-not $echoOk) {
    throw "Expected svc.demo.echo to run with M11 import-grant evidence"
}

Send-AgentCommand -Command "agent module.granted_candidate_selftest" -ExpectedMarker "RAIOS_AGENT_END module.granted_candidate_selftest" -Name "m11-import-grant:granted-candidate-selftest"
$selftest = Get-LastAgentResponseJson -Method "module.granted_candidate_selftest"

$grantedRun = $selftest.body.result.authorized_import_probe
$grantedOk = (
    $grantedRun.present -eq $true -and
    $grantedRun.validation_ok -eq $true -and
    $grantedRun.instantiation_ok -eq $true -and
    $grantedRun.run_outcome -eq "success" -and
    $grantedRun.import_grant_performed -eq $true -and
    $grantedRun.module_imports_within_authorized_list -eq $true -and
    [int]$grantedRun.authorized_import_count -eq 2 -and
    [int]$grantedRun.linked_host_import_count -eq 2 -and
    $grantedRun.log_line_emitted -eq $true -and
    (Test-Sha256Ref -Value $grantedRun.authorized_import_list_sha256)
)
Add-Predicate -Name "m11-import-grant:granted-artifact-runs-authorized" -Expected "granted-candidate artifact selftest probe runs with import-grant evidence present" -Passed $grantedOk -Actual $(if ($grantedOk) { "matched" } else { ($grantedRun | ConvertTo-Json -Compress -Depth 6) })
if (-not $grantedOk) {
    throw "Expected granted artifact selftest probe to carry M11 import-grant evidence"
}

$unauthorized = $selftest.body.result.unauthorized_import_probe
$unauthorizedOk = (
    $unauthorized.present -eq $true -and
    $unauthorized.import_grant_performed -eq $true -and
    [int]$unauthorized.authorized_import_count -eq 1 -and
    [int]$unauthorized.linked_host_import_count -eq 1 -and
    $unauthorized.module_imports_within_authorized_list -eq $false -and
    $unauthorized.run_outcome -eq "module_import_not_authorized" -and
    $unauthorized.missing_import_module -eq "env" -and
    $unauthorized.missing_import_name -eq "counter_get" -and
    $unauthorized.instantiation_ok -eq $false -and
    $unauthorized.log_line_emitted -eq $false -and
    (Test-Sha256Ref -Value $unauthorized.authorized_import_list_sha256)
)
Add-Predicate -Name "m11-import-grant:unauthorized-import-refused" -Expected "echo with only env.log authorized refuses env.counter_get before instantiation and emits no guest log" -Passed $unauthorizedOk -Actual $(if ($unauthorizedOk) { "matched" } else { ($unauthorized | ConvertTo-Json -Compress -Depth 6) })
if (-not $unauthorizedOk) {
    throw "Expected unauthorized echo import to fail closed before instantiation"
}

$forbidden = $selftest.body.result.forbidden_import_link_failure
$forbiddenOk = (
    $forbidden.negative_probe -eq "forbidden_import_link_failure" -and
    $forbidden.negative_validation_ok -eq $true -and
    $forbidden.negative_instantiation_ok -eq $false -and
    $forbidden.negative_link_error_kind -eq "missing_definition" -and
    $forbidden.negative_missing_import_module -eq "env" -and
    $forbidden.negative_missing_import_name -eq "forbidden_write" -and
    $forbidden.capability_boundary_held -eq $true
)
Add-Predicate -Name "m11-import-grant:forbidden-import-link-failure-preserved" -Expected "env.forbidden_write still fails at wasmi missing_definition link boundary" -Passed $forbiddenOk -Actual $(if ($forbiddenOk) { "matched" } else { ($forbidden | ConvertTo-Json -Compress -Depth 5) })
if (-not $forbiddenOk) {
    throw "Expected forbidden import physical link failure to be preserved"
}
