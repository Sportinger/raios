Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot
try {
    $paths = @(
        @(git ls-files -- '*.rs')
        @(git ls-files --others --exclude-standard -- '*.rs')
    ) | Where-Object {
        $_ -and $_ -notmatch '^(\.git|\.cargo-home|target|vendor)/'
    } | Sort-Object -Unique

    if ($LASTEXITCODE -ne 0) {
        throw 'git ls-files failed'
    }

    $exemptions = @{
        'seed-kernel/src/agent_protocol_memory.rs' = @(11494, 612809)
        # post-P1-C measurement: the concat! literal splits add fragment lines.
        'seed-kernel/src/agent_protocol_module_load_gate_render.rs' = @(6878, 323844)
        # P1-A loader split is PARKED (see the P1 design doc outcome note);
        # this entry disappears when the split lands. Bytes are the CRLF
        # on-disk measurement.
        'seed-kernel/src/agent_protocol_module_loader_runtime.rs' = @(10156, 498005)
        'seed-kernel/src/agent_protocol_recovery.rs' = @(6167, 296022)
        'seed-kernel/src/event_log.rs' = @(7141, 282628)
        'seed-kernel/src/event_log_types.rs' = @(3918, 216113)
        'seed-kernel/src/hello_service/emitters.rs' = @(5086, 265421)
    }

    $rows = @()
    $seenExemptions = @{}
    $failed = $false

    foreach ($path in $paths) {
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            continue
        }

        $lineCount = 0
        foreach ($line in [IO.File]::ReadLines((Resolve-Path -LiteralPath $path))) {
            $lineCount++
        }
        $byteCount = (Get-Item -LiteralPath $path).Length

        $thresholds = @()
        if ($lineCount -ge 3000) { $thresholds += 'lines-warning' }
        if ($byteCount -ge 122880) { $thresholds += 'bytes-warning' }
        if ($lineCount -ge 5000) { $thresholds += 'lines-hard' }
        if ($byteCount -ge 204800) { $thresholds += 'bytes-hard' }

        $isHard = $lineCount -ge 5000 -or $byteCount -ge 204800
        $severity = 'WARN'
        $detail = ''

        if ($exemptions.ContainsKey($path)) {
            $seenExemptions[$path] = $true
            $baseline = $exemptions[$path]
            if (-not $isHard) {
                $severity = 'ERROR'
                $detail = ' stale-exemption'
                $failed = $true
            } elseif ($lineCount -gt $baseline[0] -or $byteCount -gt $baseline[1]) {
                $severity = 'ERROR'
                $detail = " exemption-growth baseline-lines=$($baseline[0]) baseline-bytes=$($baseline[1])"
                $failed = $true
            } else {
                $severity = 'EXEMPT'
                $detail = " baseline-lines=$($baseline[0]) baseline-bytes=$($baseline[1])"
            }
        } elseif ($isHard) {
            $severity = 'ERROR'
            $detail = ' hard-cap'
            $failed = $true
        }

        if ($thresholds.Count -gt 0 -or $exemptions.ContainsKey($path)) {
            $rows += [PSCustomObject]@{
                Path = $path
                Text = "$severity $path lines=$lineCount bytes=$byteCount thresholds=$($thresholds -join ',')$detail"
            }
        }
    }

    foreach ($path in $exemptions.Keys) {
        if (-not $seenExemptions.ContainsKey($path)) {
            $rows += [PSCustomObject]@{
                Path = $path
                Text = "ERROR $path exemption-target-missing"
            }
            $failed = $true
        }
    }

    $rows | Sort-Object Path | ForEach-Object { Write-Output $_.Text }
    if ($failed) {
        exit 1
    }
    exit 0
} finally {
    Pop-Location
}
