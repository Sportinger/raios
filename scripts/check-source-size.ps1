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
        # Count bytes with CRLF normalized to LF: checkout smudge (core.autocrlf)
        # must not move a file relative to the byte caps or its exemption
        # baseline (2026-07-13: agent_protocol_memory.rs gained exactly +1
        # byte/line after a revert re-checked it out with CRLF).
        $rawBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $path))
        $crlfCount = 0
        for ($i = 1; $i -lt $rawBytes.Length; $i++) {
            if ($rawBytes[$i] -eq 0x0A -and $rawBytes[$i - 1] -eq 0x0D) { $crlfCount++ }
        }
        $byteCount = $rawBytes.Length - $crlfCount

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
