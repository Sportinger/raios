if (-not $PersistDiskImage) {
    throw "memory-durable profile requires PersistDiskImage"
}

# M9B-1b: the agent-observation command carries a base64 blob (~213-byte line). Sent
# as one fast burst it overflows the guest's 16550 UART RX FIFO (the guest drains
# MAX_BYTES_PER_POLL=64 per poll with UI redraw between polls), dropping the tail so the
# line never terminates and the kernel never dispatches. Pace EVERY send in this profile
# (small chunks + a short delay) so the guest drains between chunks -- exactly how a real
# agent sender, and the existing submit_candidate_chunk path, pace long serial writes.
# Harmless for the short M9A commands (a few extra ms each).
# 2026-07-08: chunk 32 / 10ms began truncating the ~213-byte observation line under
# heavier guest redraw timing (three consecutive host runs). Halved the chunk and
# more-than-doubled the delay so a full guest poll+redraw cycle always drains the RX
# FIFO between chunks; costs the long line ~0.35s, negligible for the short commands.
$script:SerialWriteChunkSize = 16
$script:SerialWriteDelayMilliseconds = 25

# M9C-2b: child-VM serial waits must be OCCURRENCE-aware, not whole-file substring.
# The export-denial family is the first child probe to send a REPEATED method
# (provider.context_export x3, durable.record_log_scan x2). A whole-file
# "RAIOS_AGENT_END <method>" match (Wait-ForLogText) returns instantly on a STALE
# marker left by the previous same-method command, so Get-ProfileAgentResponseJson
# (LastIndexOf) could parse a prior block before the new response lands -- a timing
# race that false-fails under host/disk stress. Count the method's END markers before
# each send and wait until the count strictly increases. No TCP drain here (the child
# logfile is written directly by QEMU; draining the main-VM stream would steal bytes a
# later main-VM assertion needs).
function Get-ChildEndMarkerCount {
    param([string]$Path, [string]$Method)
    $content = Get-SerialLogContent -Path $Path
    if ($null -eq $content) { return 0 }
    return ([regex]::Matches($content, [regex]::Escape("RAIOS_AGENT_END $Method"))).Count
}

function Wait-ForChildEndMarker {
    param([string]$Path, [string]$Method, [int]$PriorCount, [int]$TimeoutSeconds)
    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    do {
        if ((Get-ChildEndMarkerCount -Path $Path -Method $Method) -gt $PriorCount) {
            return $true
        }
        Start-Sleep -Milliseconds 200
    } while ([DateTime]::UtcNow -lt $deadline)
    return $false
}

function Get-ProfileAgentResponseJson {
    param(
        [string]$Path,
        [string]$Method
    )

    $content = Get-Content -LiteralPath $Path -Raw -ErrorAction Stop
    $begin = "RAIOS_AGENT_BEGIN $Method"
    $end = "RAIOS_AGENT_END $Method"
    $beginIndex = $content.LastIndexOf($begin, [System.StringComparison]::Ordinal)
    if ($beginIndex -lt 0) {
        throw "No agent response for method '$Method' found in $Path"
    }
    $jsonStart = $content.IndexOf("{", $beginIndex, [System.StringComparison]::Ordinal)
    $endIndex = $content.IndexOf($end, $jsonStart, [System.StringComparison]::Ordinal)
    if ($jsonStart -lt 0 -or $endIndex -lt 0) {
        throw "Incomplete agent response for method '$Method' found in $Path"
    }
    $json = $content.Substring($jsonStart, $endIndex - $jsonStart).Trim()
    return $json | ConvertFrom-Json
}

function Invoke-MemoryRecordAppendFixtureProbe {
    param(
        [string]$FixtureSpec,
        [string]$Label,
        [string]$BootCtlSpec = "valid-a",
        # M9A-3b: which Read0 append method the child VM is asked to run. Defaults to
        # the M9A-2b single-record method so the existing call site is unaffected;
        # the memory-durable-supersede family passes
        # "memory.decision_problem_log_append" to drive the SAME probe pattern
        # (build -> boot -> agent <method> -> agent durable.record_log_scan) against
        # the M9A-3b decision/problem/supersede trio driver instead.
        [string]$AppendMethod = "memory.record_log_append",
        # M9B-1b: an optional trailing argument sent after $AppendMethod on the SAME
        # line (e.g. "agent memory.observation_log_append <base64>") -- the
        # agent-authored observation method takes a base64 blob arg, unlike the
        # arg-less system-authored append methods above (default empty preserves the
        # existing call sites byte-for-byte).
        [string]$AppendArg = "",
        [object[]]$ExtraAgentCommands = @()
    )

    if (-not $script:MemoryRecordFixtureProbeIndex) {
        $script:MemoryRecordFixtureProbeIndex = 0
    }
    $script:MemoryRecordFixtureProbeIndex += 1
    $suffix = "$Label-$($script:MemoryRecordFixtureProbeIndex)"
    $fixturePort = $SerialTcpPort + 400 + $script:MemoryRecordFixtureProbeIndex
    $fixtureLog = Join-Path $RunDir "serial-memrecord-$suffix.log"
    $fixtureErr = [System.IO.Path]::ChangeExtension($fixtureLog, ".err.txt")
    $fixtureImage = Join-Path $RunDir "raios-stage0-memrecord-$suffix.img"
    $fixtureScratch = Join-Path $RunDir "raios-stage0-memrecord-$suffix-scratch.img"
    $fixtureAuditRollback = Join-Path $RunDir "raios-stage0-memrecord-$suffix-audit-rollback-target.img"
    $fixturePersist = Join-Path $RunDir "raios-persist-memrecord-$suffix.img"
    $code = Join-Path $RunDir "edk2-code-memrecord-$suffix.fd"
    $vars = Join-Path $RunDir "ovmf-vars-memrecord-$suffix.fd"
    $qemu = "C:\Program Files\qemu\qemu-system-x86_64.exe"

    $builderArgs = @("--self-check", "--seed-reclog-fixture", $FixtureSpec, "--seed-bootctl", $BootCtlSpec, $fixturePersist)
    $buildOutput = & python $builder @builderArgs 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Memory-record fixture build failed ($FixtureSpec / bootctl=$BootCtlSpec): $($buildOutput -join [Environment]::NewLine)"
    }

    Copy-Item -LiteralPath $ResolvedImage -Destination $fixtureImage -Force
    Copy-Item -LiteralPath $ScratchImage -Destination $fixtureScratch -Force
    Copy-Item -LiteralPath $AuditRollbackTargetImage -Destination $fixtureAuditRollback -Force
    Copy-Item -LiteralPath "C:\Program Files\qemu\share\edk2-x86_64-code.fd" -Destination $code -Force
    Copy-Item -LiteralPath (Join-Path $RepoRoot "release\ovmf_vars.fd") -Destination $vars -Force
    Remove-Item -LiteralPath $fixtureLog, $fixtureErr -Force -ErrorAction SilentlyContinue

    $driveId = "raiospersist_memrecord_$($script:MemoryRecordFixtureProbeIndex)"
    $qemuArgs = @(
        "-machine", "q35",
        "-m", "512M",
        "-drive", "if=pflash,format=raw,readonly=on,file=$code",
        "-drive", "if=pflash,format=raw,file=$vars",
        "-drive", "file=$fixtureImage,format=raw,if=ide",
        "-drive", "file=$fixtureScratch,format=raw,if=none,id=raiosscratch_memrecord_$($script:MemoryRecordFixtureProbeIndex)",
        "-device", "ide-hd,drive=raiosscratch_memrecord_$($script:MemoryRecordFixtureProbeIndex),bus=ide.1,unit=0",
        "-drive", "file=$fixtureAuditRollback,format=raw,if=none,id=raiosauditrollback_memrecord_$($script:MemoryRecordFixtureProbeIndex)",
        "-device", "ide-hd,drive=raiosauditrollback_memrecord_$($script:MemoryRecordFixtureProbeIndex),bus=ide.2,unit=0",
        "-drive", "file=$fixturePersist,format=raw,if=none,id=$driveId",
        "-device", "ide-hd,drive=$driveId,bus=ide.3,unit=0",
        "-cpu", "max",
        "-device", "qemu-xhci,id=xhci_memrecord_$($script:MemoryRecordFixtureProbeIndex)",
        "-device", "usb-kbd,bus=xhci_memrecord_$($script:MemoryRecordFixtureProbeIndex).0",
        "-device", "usb-tablet,bus=xhci_memrecord_$($script:MemoryRecordFixtureProbeIndex).0",
        "-chardev", "socket,id=seedserial_memrecord_$($script:MemoryRecordFixtureProbeIndex),host=127.0.0.1,port=$fixturePort,server=on,wait=off,logfile=$fixtureLog,logappend=off",
        "-serial", "chardev:seedserial_memrecord_$($script:MemoryRecordFixtureProbeIndex)",
        "-display", "none",
        "-no-reboot"
    )

    $child = Start-Process -FilePath $qemu -ArgumentList $qemuArgs -PassThru -RedirectStandardError $fixtureErr -WindowStyle Hidden
    try {
        $childTimeout = [Math]::Max(180, $TimeoutSeconds * 4)
        if (-not (Wait-ForLogText -Path $fixtureLog -Needle "SERIAL CONSOLE READY" -TimeoutSeconds $childTimeout)) {
            throw "Memory-record fixture child VM did not reach serial console: $(Get-SerialLogTail -Path $fixtureLog)"
        }
        $appendCommandLine = if ($AppendArg.Length -gt 0) { "agent $AppendMethod $AppendArg" } else { "agent $AppendMethod" }
        $appendPriorCount = Get-ChildEndMarkerCount -Path $fixtureLog -Method $AppendMethod
        Send-SerialText -Port $fixturePort -Text "$appendCommandLine`r" -TimeoutSeconds $childTimeout
        if (-not (Wait-ForChildEndMarker -Path $fixtureLog -Method $AppendMethod -PriorCount $appendPriorCount -TimeoutSeconds $childTimeout)) {
            throw "Memory-record fixture child VM did not answer ${AppendMethod}: $(Get-SerialLogTail -Path $fixtureLog)"
        }
        $appendResult = (Get-ProfileAgentResponseJson -Path $fixtureLog -Method $AppendMethod).body.result

        # INSPECT: an independent follow-up durable.record_log_scan (same booted VM,
        # same disk state) must see the SAME tail_seq/count the append reported --
        # proving the frame is durably visible, not just self-reported by the writer.
        $scanPriorCount = Get-ChildEndMarkerCount -Path $fixtureLog -Method "durable.record_log_scan"
        Send-SerialText -Port $fixturePort -Text "agent durable.record_log_scan`r" -TimeoutSeconds $childTimeout
        if (-not (Wait-ForChildEndMarker -Path $fixtureLog -Method "durable.record_log_scan" -PriorCount $scanPriorCount -TimeoutSeconds $childTimeout)) {
            throw "Memory-record fixture child VM did not answer durable.record_log_scan: $(Get-SerialLogTail -Path $fixtureLog)"
        }
        $scanResult = (Get-ProfileAgentResponseJson -Path $fixtureLog -Method "durable.record_log_scan").body.result
        $extraResponses = @{}
        foreach ($extra in $ExtraAgentCommands) {
            $extraPriorCount = Get-ChildEndMarkerCount -Path $fixtureLog -Method $extra.Method
            Send-SerialText -Port $fixturePort -Text "agent $($extra.Command)`r" -TimeoutSeconds $childTimeout
            if (-not (Wait-ForChildEndMarker -Path $fixtureLog -Method $extra.Method -PriorCount $extraPriorCount -TimeoutSeconds $childTimeout)) {
                throw "Memory-record fixture child VM did not answer $($extra.Method): $(Get-SerialLogTail -Path $fixtureLog)"
            }
            $key = if ($extra.PSObject.Properties.Name -contains "Key" -and $extra.Key) { $extra.Key } else { $extra.Method }
            $extraResponses[$key] = Get-ProfileAgentResponseJson -Path $fixtureLog -Method $extra.Method
        }

        return [pscustomobject]@{
            append = $appendResult
            scan = $scanResult
            extra = $extraResponses
        }
    }
    finally {
        if ($child -and -not $child.HasExited) {
            Stop-Process -Id $child.Id -Force -ErrorAction SilentlyContinue
            try { $child.WaitForExit(5000) | Out-Null } catch {}
        }
        # Reclaim this probe's large images once the child has answered (keep the
        # serial log for debugging) -- an unbounded RunDir fills the host disk.
        Remove-Item -LiteralPath $fixtureImage, $fixtureScratch, $fixtureAuditRollback, $fixturePersist, $code, $vars -Force -ErrorAction SilentlyContinue
    }
}

$builder = Join-Path $RepoRoot "scripts\make-gpt-persist-image.py"

# --- memory-durable-append: a real durable write against a real persist disk ------

$probe = Invoke-MemoryRecordAppendFixtureProbe -FixtureSpec "valid:2" -Label "append"
$append = $probe.append
$scan = $probe.scan

function Add-MemoryDurableAppendFieldPredicate {
    param(
        [string]$Suffix,
        [string]$Expected,
        [bool]$Passed,
        $Actual
    )
    Add-Predicate `
        -Name "memory-durable-append:$Suffix" `
        -Expected $Expected `
        -Passed $Passed `
        -Actual $(if ($Passed) { "matched" } else { [string]$Actual })
}

$appendFieldChecks = @(
    @{ suffix = "durable-append-status"; expected = 'durable_append == "appended"'; actual = $append.durable_append; passed = ($append.durable_append -eq "appended") },
    @{ suffix = "performed-true"; expected = "performed == true"; actual = $append.performed; passed = [bool]$append.performed },
    @{ suffix = "authority"; expected = 'authority == "scoped_memory_record_append_authorized"'; actual = $append.authority; passed = ($append.authority -eq "scoped_memory_record_append_authorized") },
    @{ suffix = "record-schema"; expected = 'record_schema == "raios.memory_record.v0"'; actual = $append.record_schema; passed = ($append.record_schema -eq "raios.memory_record.v0") },
    @{ suffix = "region-marker"; expected = 'region_marker == "RAIOS_DATA_RECLOG"'; actual = $append.region_marker; passed = ($append.region_marker -eq "RAIOS_DATA_RECLOG") },
    @{ suffix = "target-id"; expected = 'target_id == "append.memory_record.seed_data"'; actual = $append.target_id; passed = ($append.target_id -eq "append.memory_record.seed_data") },
    @{ suffix = "kind"; expected = 'kind == "capability_denial"'; actual = $append.kind; passed = ($append.kind -eq "capability_denial") },
    @{ suffix = "classification"; expected = 'classification == "local_only"'; actual = $append.classification; passed = ($append.classification -eq "local_only") },
    @{ suffix = "record-authority"; expected = 'record_authority == "core_ledger"'; actual = $append.record_authority; passed = ($append.record_authority -eq "core_ledger") },
    @{ suffix = "trust-tier"; expected = 'trust_tier == "dev_key_not_owner_sealed"'; actual = $append.trust_tier; passed = ($append.trust_tier -eq "dev_key_not_owner_sealed") }
)
foreach ($check in $appendFieldChecks) {
    Add-MemoryDurableAppendFieldPredicate -Suffix $check.suffix -Expected $check.expected -Passed $check.passed -Actual $check.actual
}
$appendAuthorized = -not (@($appendFieldChecks | Where-Object { -not $_.passed }).Count -gt 0)

$appendReadbackHash = (
    $append.readback_sha256 -eq $append.frame_sha256 -and
    [bool]$append.reparse_valid
)
Add-Predicate `
    -Name "memory-durable-append:readback-hash" `
    -Expected "memory.record_log_append readback_sha256 matches frame_sha256 and the persisted frame reparses" `
    -Passed $appendReadbackHash `
    -Actual $(if ($appendReadbackHash) { "matched" } else { ($append | ConvertTo-Json -Compress -Depth 10) })

$appendChainAdvance = (
    [int64]$append.tail_seq_after -eq ([int64]$append.tail_seq_before + 1) -and
    [int64]$append.count_after -eq ([int64]$append.count_before + 1)
)
Add-Predicate `
    -Name "memory-durable-append:chain-advance" `
    -Expected "memory.record_log_append advances tail_seq and count by exactly one over the seeded valid:2 fixture" `
    -Passed $appendChainAdvance `
    -Actual $(if ($appendChainAdvance) { "matched" } else { ($append | ConvertTo-Json -Compress -Depth 10) })

$appendHonestPosture = (
    -not [bool]$append.owner_sealed -and
    -not [bool]$append.persistence_claimed
)
Add-Predicate `
    -Name "memory-durable-append:honest-posture" `
    -Expected "memory.record_log_append never claims owner_sealed or cross-boot persistence_claimed" `
    -Passed $appendHonestPosture `
    -Actual $(if ($appendHonestPosture) { "matched" } else { ($append | ConvertTo-Json -Compress -Depth 10) })

$appendPayloadShaRendered = ($append.payload_sha256 -match '^sha256:[0-9a-f]{64}$')
Add-Predicate `
    -Name "memory-durable-append:payload-sha256-rendered" `
    -Expected "payload_sha256 renders as sha256:<64hex> via the shared V::Sha256 path" `
    -Passed $appendPayloadShaRendered `
    -Actual $(if ($appendPayloadShaRendered) { "matched" } else { $append.payload_sha256 })

$appendFrameShaRendered = ($append.frame_sha256 -match '^sha256:[0-9a-f]{64}$')
Add-Predicate `
    -Name "memory-durable-append:frame-sha256-rendered" `
    -Expected "frame_sha256 renders as sha256:<64hex> via the shared V::Sha256 path" `
    -Passed $appendFrameShaRendered `
    -Actual $(if ($appendFrameShaRendered) { "matched" } else { $append.frame_sha256 })

$appendRecordIdentity = (
    $append.record_id -eq "mem.capability_denial.module_load_ephemeral_durable.current_boot.v0"
)
Add-Predicate `
    -Name "memory-durable-append:record-id" `
    -Expected "the ONE fixed system-authored record id is used" `
    -Passed $appendRecordIdentity `
    -Actual $(if ($appendRecordIdentity) { "matched" } else { $append.record_id })

# The pinned golden is record_sha256() of the fixed system record, computed in
# raios-core from the identical MemoryRecordInput. Because the reclog payload is
# write_json(record.to_record_value()) and record_sha256 hashes the same rendering,
# an EQUAL payload_sha256 proves the exact raios.memory_record.v0 bytes are on disk
# -- upgrading "a well-formed frame landed" to "THIS memory record landed", which the
# hash-only durable.record_log_scan cannot otherwise show.
$appendPayloadShaGolden = (
    $append.payload_sha256 -eq "sha256:1e0d230ecc56b4a970dd09c6ab6bbc10748aa369934dc9fa412a1f0e1a77ba8f"
)
Add-Predicate `
    -Name "memory-durable-append:payload-sha256-golden" `
    -Expected "payload_sha256 equals the pinned golden record_sha256 of the fixed record (the exact raios.memory_record.v0 landed, not merely a well-formed frame)" `
    -Passed $appendPayloadShaGolden `
    -Actual $(if ($appendPayloadShaGolden) { "matched" } else { $append.payload_sha256 })

# INSPECT: an independent durable.record_log_scan (same VM, same disk) agrees with
# the append's own self-reported tail/count -- durability is independently visible,
# not just self-reported by the writer.
$inspectAgrees = (
    $scan.schema -eq "raios.durable_record_log_scan.v0" -and
    [int64]$scan.tail_seq -eq [int64]$append.tail_seq_after -and
    [int64]$scan.count -eq [int64]$append.count_after -and
    $scan.status -eq "valid"
)
Add-Predicate `
    -Name "memory-durable-append:inspect-scan-agrees" `
    -Expected "a follow-up durable.record_log_scan in the same VM shows the new frame at tail (tail_seq/count match the append's own report)" `
    -Passed $inspectAgrees `
    -Actual $(if ($inspectAgrees) { "matched" } else { ($scan | ConvertTo-Json -Compress -Depth 10) })

if (-not ($appendAuthorized -and $appendReadbackHash -and $appendChainAdvance -and $appendHonestPosture -and $appendPayloadShaRendered -and $appendFrameShaRendered -and $appendRecordIdentity -and $appendPayloadShaGolden -and $inspectAgrees)) {
    throw "memory-durable-append family failed"
}

# --- memory-durable-supersede: THREE truthful system-authored records (decision A,
#     problem P, decision B superseding A) through the SAME gauntlet -- the
#     write-side proof of supersede-not-overwrite (M9A-3b). A fresh child VM, same
#     valid:2 reclog fixture, driven with memory.decision_problem_log_append. -------

$probeSupersede = Invoke-MemoryRecordAppendFixtureProbe -FixtureSpec "valid:2" -Label "decision-problem" -AppendMethod "memory.decision_problem_log_append"
$decisionProblem = $probeSupersede.append
$decisionProblemScan = $probeSupersede.scan
$dpRecords = @($decisionProblem.records)

$dpRecordCountOk = ($dpRecords.Count -eq 3)
Add-Predicate `
    -Name "memory-durable-supersede:record-count" `
    -Expected "memory.decision_problem_log_append records array has exactly 3 entries (A, P, B)" `
    -Passed $dpRecordCountOk `
    -Actual $(if ($dpRecordCountOk) { "matched" } else { $dpRecords.Count })
if (-not $dpRecordCountOk) {
    throw "memory-durable-supersede family failed: expected 3 records, got $($dpRecords.Count)"
}
$recordA = $dpRecords[0]
$recordP = $dpRecords[1]
$recordB = $dpRecords[2]

# Pinned goldens: record_sha256() of A/P/B computed in raios-core from the identical
# frozen MemoryRecordInput -- proves the EXACT raios.memory_record.v0 bytes for each
# of the three records landed, not merely well-formed frames.
$supersedeGoldenShaByLabel = @{
    A = "sha256:9b39ac7309d7b63c95062c78d8c02bb717f25a589c437a54b627598c24198cb0"
    P = "sha256:3b010268c4e45a30bb79fee27f4cf07cead1a4542709fdc1ca9c195a7ee9a249"
    B = "sha256:5f27b06d961fe866f7c025a5a4b7ee3ffb76ee6e51db111f437c73647bb7a262"
}

function Test-MemoryDurableSupersedeRecord {
    param(
        [string]$Label,
        $Record,
        [string]$ExpectedKind,
        [string]$GoldenPayloadSha256
    )
    $checks = @(
        @{ suffix = "durable-append-status"; expected = 'durable_append == "appended"'; passed = ($Record.durable_append -eq "appended"); actual = $Record.durable_append },
        @{ suffix = "performed-true"; expected = "performed == true"; passed = [bool]$Record.performed; actual = $Record.performed },
        @{ suffix = "authority"; expected = 'authority == "scoped_memory_record_append_authorized"'; passed = ($Record.authority -eq "scoped_memory_record_append_authorized"); actual = $Record.authority },
        @{ suffix = "kind"; expected = "kind == `"$ExpectedKind`""; passed = ($Record.kind -eq $ExpectedKind); actual = $Record.kind },
        @{ suffix = "classification"; expected = 'classification == "local_only"'; passed = ($Record.classification -eq "local_only"); actual = $Record.classification },
        @{ suffix = "readback-hash"; expected = "readback_sha256 == frame_sha256 and reparse_valid"; passed = ($Record.readback_sha256 -eq $Record.frame_sha256 -and [bool]$Record.reparse_valid); actual = "readback_sha256=$($Record.readback_sha256) frame_sha256=$($Record.frame_sha256) reparse_valid=$($Record.reparse_valid)" },
        @{ suffix = "honest-posture"; expected = "owner_sealed == false and persistence_claimed == false"; passed = (-not [bool]$Record.owner_sealed -and -not [bool]$Record.persistence_claimed); actual = "owner_sealed=$($Record.owner_sealed) persistence_claimed=$($Record.persistence_claimed)" },
        @{ suffix = "payload-sha256-golden"; expected = "payload_sha256 == pinned golden $GoldenPayloadSha256"; passed = ($Record.payload_sha256 -eq $GoldenPayloadSha256); actual = $Record.payload_sha256 }
    )
    foreach ($check in $checks) {
        Add-Predicate `
            -Name "memory-durable-supersede:${Label}-$($check.suffix)" `
            -Expected $check.expected `
            -Passed $check.passed `
            -Actual $(if ($check.passed) { "matched" } else { [string]$check.actual })
    }
    return -not (@($checks | Where-Object { -not $_.passed }).Count -gt 0)
}

$recordAOk = Test-MemoryDurableSupersedeRecord -Label "A" -Record $recordA -ExpectedKind "decision" -GoldenPayloadSha256 $supersedeGoldenShaByLabel.A
$recordPOk = Test-MemoryDurableSupersedeRecord -Label "P" -Record $recordP -ExpectedKind "problem" -GoldenPayloadSha256 $supersedeGoldenShaByLabel.P
$recordBOk = Test-MemoryDurableSupersedeRecord -Label "B" -Record $recordB -ExpectedKind "decision" -GoldenPayloadSha256 $supersedeGoldenShaByLabel.B

# Chain advance: exactly +1 at each of A, P, B (each one's count_before equals the
# prior one's count_after), and exactly +3 across the whole trio.
$chainStepA = ([int64]$recordA.tail_seq_after -eq ([int64]$recordA.tail_seq_before + 1) -and [int64]$recordA.count_after -eq ([int64]$recordA.count_before + 1))
$chainStepP = ([int64]$recordP.tail_seq_after -eq ([int64]$recordP.tail_seq_before + 1) -and [int64]$recordP.count_after -eq ([int64]$recordP.count_before + 1) -and [int64]$recordP.count_before -eq [int64]$recordA.count_after)
$chainStepB = ([int64]$recordB.tail_seq_after -eq ([int64]$recordB.tail_seq_before + 1) -and [int64]$recordB.count_after -eq ([int64]$recordB.count_before + 1) -and [int64]$recordB.count_before -eq [int64]$recordP.count_after)
$chainOverall = ([int64]$recordB.count_after -eq ([int64]$recordA.count_before + 3))
$chainAdvanceOk = ($chainStepA -and $chainStepP -and $chainStepB -and $chainOverall)
Add-Predicate `
    -Name "memory-durable-supersede:chain-advance" `
    -Expected "count/tail_seq advance by exactly +1 at each of A, P, B, and by exactly +3 across the trio" `
    -Passed $chainAdvanceOk `
    -Actual $(if ($chainAdvanceOk) { "matched" } else { ($dpRecords | ConvertTo-Json -Compress -Depth 10) })

# Supersede proven: B echoes supersedes == [A.id] (A/P echo empty supersedes), backed
# by B's already-verified golden payload_sha256 (whose known bytes include
# supersedes:[A.id]) -- the on-disk proof, not just an in-memory claim.
$supersedeEchoOk = (
    (@($recordB.supersedes)).Count -eq 1 -and
    (@($recordB.supersedes))[0] -eq "mem.decision.module_sharing_confirmed_vision.current_boot.v0" -and
    (@($recordA.supersedes)).Count -eq 0 -and
    (@($recordP.supersedes)).Count -eq 0
)
Add-Predicate `
    -Name "memory-durable-supersede:supersede-echo" `
    -Expected "B echoes supersedes == [A.id] and A/P echo empty supersedes (the write-side supersede-not-overwrite proof)" `
    -Passed $supersedeEchoOk `
    -Actual $(if ($supersedeEchoOk) { "matched" } else { ($dpRecords | ConvertTo-Json -Compress -Depth 10) })

# INSPECT: an independent follow-up durable.record_log_scan (same booted VM, same
# disk state) shows B -- the trio's last record -- at tail.
$supersedeInspectAgrees = (
    $decisionProblemScan.schema -eq "raios.durable_record_log_scan.v0" -and
    [int64]$decisionProblemScan.tail_seq -eq [int64]$recordB.tail_seq_after -and
    [int64]$decisionProblemScan.count -eq [int64]$recordB.count_after -and
    $decisionProblemScan.status -eq "valid"
)
Add-Predicate `
    -Name "memory-durable-supersede:inspect-scan-agrees" `
    -Expected "a follow-up durable.record_log_scan shows B (the trio's last record) at tail" `
    -Passed $supersedeInspectAgrees `
    -Actual $(if ($supersedeInspectAgrees) { "matched" } else { ($decisionProblemScan | ConvertTo-Json -Compress -Depth 10) })

if (-not ($recordAOk -and $recordPOk -and $recordBOk -and $chainAdvanceOk -and $supersedeEchoOk -and $supersedeInspectAgrees)) {
    throw "memory-durable-supersede family failed"
}

# --- memory-agent-observation-authorized: the FIRST agent-authored durable write
#     (M9B-1b) -- a fresh child VM, same valid:2 reclog fixture, driven with
#     memory.observation_log_append <FROZEN_BLOB>. The frozen blob decodes to
#     EXACTLY 4 newline-separated fields (entity/predicate/value/source_record_id);
#     the kernel FORCES every other authority-bearing field (id, kind,
#     classification, authority, boot_id, supersedes, tags). ----------------------

# FROZEN test blob -- decodes to:
#   entity           = vm.smoke.observation
#   predicate        = observed
#   value            = agent authored durable observation via memory.observation_log_append
#   source_record_id = vm.smoke.memory-agent-observation
$agentObservationBlob = "dm0uc21va2Uub2JzZXJ2YXRpb24Kb2JzZXJ2ZWQKYWdlbnQgYXV0aG9yZWQgZHVyYWJsZSBvYnNlcnZhdGlvbiB2aWEgbWVtb3J5Lm9ic2VydmF0aW9uX2xvZ19hcHBlbmQKdm0uc21va2UubWVtb3J5LWFnZW50LW9ic2VydmF0aW9u"

$probeObservation = Invoke-MemoryRecordAppendFixtureProbe -FixtureSpec "valid:2" -Label "agent-observation" -AppendMethod "memory.observation_log_append" -AppendArg $agentObservationBlob
$agentObservation = $probeObservation.append
$agentObservationScan = $probeObservation.scan

$agentObservationFieldChecks = @(
    @{ suffix = "durable-append-status"; expected = 'durable_append == "appended"'; actual = $agentObservation.durable_append; passed = ($agentObservation.durable_append -eq "appended") },
    @{ suffix = "performed-true"; expected = "performed == true"; actual = $agentObservation.performed; passed = [bool]$agentObservation.performed },
    @{ suffix = "authority"; expected = 'authority == "scoped_memory_record_append_authorized"'; actual = $agentObservation.authority; passed = ($agentObservation.authority -eq "scoped_memory_record_append_authorized") },
    @{ suffix = "kind"; expected = 'kind == "observation"'; actual = $agentObservation.kind; passed = ($agentObservation.kind -eq "observation") },
    @{ suffix = "classification"; expected = 'classification == "local_only"'; actual = $agentObservation.classification; passed = ($agentObservation.classification -eq "local_only") },
    @{ suffix = "record-authority"; expected = 'record_authority == "agent"'; actual = $agentObservation.record_authority; passed = ($agentObservation.record_authority -eq "agent") },
    @{ suffix = "record-id"; expected = 'record_id == "mem.observation.agent.current_boot.00000001.v0"'; actual = $agentObservation.record_id; passed = ($agentObservation.record_id -eq "mem.observation.agent.current_boot.00000001.v0") },
    @{ suffix = "readback-hash"; expected = "readback_sha256 == frame_sha256 and reparse_valid"; actual = "readback_sha256=$($agentObservation.readback_sha256) frame_sha256=$($agentObservation.frame_sha256) reparse_valid=$($agentObservation.reparse_valid)"; passed = ($agentObservation.readback_sha256 -eq $agentObservation.frame_sha256 -and [bool]$agentObservation.reparse_valid) },
    @{ suffix = "honest-posture"; expected = "owner_sealed == false and persistence_claimed == false"; actual = "owner_sealed=$($agentObservation.owner_sealed) persistence_claimed=$($agentObservation.persistence_claimed)"; passed = (-not [bool]$agentObservation.owner_sealed -and -not [bool]$agentObservation.persistence_claimed) },
    @{ suffix = "frame-len-bounded"; expected = "frame_len <= 1024 (proves the 2-sector agent quota charge is not an undercharge)"; actual = $agentObservation.frame_len; passed = ([int64]$agentObservation.frame_len -le 1024) },
    @{ suffix = "payload-sha256-golden"; expected = "payload_sha256 == pinned golden sha256:75ea5ab92fc9dafe908bae204e5a357947e47ba7e231aaddc0c19854288e198d (the EXACT agent record landed)"; actual = $agentObservation.payload_sha256; passed = ($agentObservation.payload_sha256 -eq "sha256:75ea5ab92fc9dafe908bae204e5a357947e47ba7e231aaddc0c19854288e198d") }
)
foreach ($check in $agentObservationFieldChecks) {
    Add-Predicate `
        -Name "memory-agent-observation-authorized:$($check.suffix)" `
        -Expected $check.expected `
        -Passed $check.passed `
        -Actual $(if ($check.passed) { "matched" } else { [string]$check.actual })
}
$agentObservationAuthorized = -not (@($agentObservationFieldChecks | Where-Object { -not $_.passed }).Count -gt 0)

$agentObservationChainAdvance = (
    [int64]$agentObservation.tail_seq_after -eq ([int64]$agentObservation.tail_seq_before + 1) -and
    [int64]$agentObservation.count_after -eq ([int64]$agentObservation.count_before + 1)
)
Add-Predicate `
    -Name "memory-agent-observation-authorized:chain-advance" `
    -Expected "memory.observation_log_append advances tail_seq and count by exactly one over the seeded valid:2 fixture" `
    -Passed $agentObservationChainAdvance `
    -Actual $(if ($agentObservationChainAdvance) { "matched" } else { ($agentObservation | ConvertTo-Json -Compress -Depth 10) })

# INSPECT: an independent follow-up durable.record_log_scan (same booted child VM,
# same disk state) agrees with the append's own self-reported tail/count.
$agentObservationInspectAgrees = (
    $agentObservationScan.schema -eq "raios.durable_record_log_scan.v0" -and
    [int64]$agentObservationScan.tail_seq -eq [int64]$agentObservation.tail_seq_after -and
    [int64]$agentObservationScan.count -eq [int64]$agentObservation.count_after -and
    $agentObservationScan.status -eq "valid"
)
Add-Predicate `
    -Name "memory-agent-observation-authorized:inspect-scan-agrees" `
    -Expected "a follow-up durable.record_log_scan in the same VM shows the new agent-authored observation frame at tail" `
    -Passed $agentObservationInspectAgrees `
    -Actual $(if ($agentObservationInspectAgrees) { "matched" } else { ($agentObservationScan | ConvertTo-Json -Compress -Depth 10) })

if (-not ($agentObservationAuthorized -and $agentObservationChainAdvance -and $agentObservationInspectAgrees)) {
    throw "memory-agent-observation-authorized family failed"
}

# --- broker-* families: durable memory is reparsed, resolved, and surfaced through
#     memory.context while provider export stays fail-closed (M9C-1b). -------------

function Get-BrokerDurableRecordById {
    param(
        $Records,
        [string]$Id
    )
    $matches = @($Records | Where-Object { $_.id -eq $Id })
    if ($matches.Count -gt 0) { return $matches[0] }
    return $null
}

function Get-BrokerDurableRecordIndex {
    param(
        $Records,
        [string]$Id
    )
    for ($i = 0; $i -lt @($Records).Count; $i += 1) {
        if ($Records[$i].id -eq $Id) { return $i }
    }
    return -1
}

function Test-BrokerRecordOmitsValue {
    param($Record)
    return ($Record -and -not ($Record.PSObject.Properties.Name -contains "value"))
}

$brokerExtraCommands = @(
    [pscustomobject]@{ Command = "memory.decision_problem_log_append"; Method = "memory.decision_problem_log_append" },
    [pscustomobject]@{ Command = "memory.observation_log_append $agentObservationBlob"; Method = "memory.observation_log_append" },
    [pscustomobject]@{ Command = "memory.context provider_minimal"; Method = "memory.context" },
    [pscustomobject]@{ Command = "provider.context_export provider_minimal"; Method = "provider.context_export" },
    [pscustomobject]@{ Command = "memory.broker_resolve_selftest"; Method = "memory.broker_resolve_selftest" }
)
$brokerProbe = Invoke-MemoryRecordAppendFixtureProbe -FixtureSpec "valid:2" -Label "broker" -AppendMethod "memory.record_log_append" -ExtraAgentCommands $brokerExtraCommands
$brokerContext = $brokerProbe.extra["memory.context"].body.result
$brokerProviderExport = $brokerProbe.extra["provider.context_export"]
$brokerSelftest = $brokerProbe.extra["memory.broker_resolve_selftest"].body.result
$brokerRecords = @($brokerContext.durable_records)
$brokerOmitted = @($brokerContext.omitted)

$brokerCapDenialId = "mem.capability_denial.module_load_ephemeral_durable.current_boot.v0"
$brokerDecisionAId = "mem.decision.module_sharing_confirmed_vision.current_boot.v0"
$brokerProblemId = "mem.problem.memory_mutation_denied.current_boot.v0"
$brokerDecisionBId = "mem.decision.module_sharing_evidence_gated.current_boot.v0"
$brokerObservationId = "mem.observation.agent.current_boot.00000001.v0"

$brokerCapDenial = Get-BrokerDurableRecordById -Records $brokerRecords -Id $brokerCapDenialId
$brokerProblem = Get-BrokerDurableRecordById -Records $brokerRecords -Id $brokerProblemId
$brokerDecisionB = Get-BrokerDurableRecordById -Records $brokerRecords -Id $brokerDecisionBId
$brokerObservationVisible = Get-BrokerDurableRecordById -Records $brokerRecords -Id $brokerObservationId

$brokerIncludedOk = (
    $brokerDecisionB -and
    $brokerDecisionB.kind -eq "decision" -and
    $brokerDecisionB.authority -eq "decision" -and
    $brokerDecisionB.entity -eq "raios.module_sharing" -and
    $brokerDecisionB.predicate -eq "standing_decision" -and
    $brokerDecisionB.scope -eq "durable" -and
    (Test-BrokerRecordOmitsValue -Record $brokerDecisionB) -and
    $brokerObservationVisible -and
    $brokerObservationVisible.kind -eq "observation" -and
    $brokerObservationVisible.authority -eq "agent" -and
    $brokerObservationVisible.entity -eq "vm.smoke.observation" -and
    $brokerObservationVisible.predicate -eq "observed" -and
    $brokerObservationVisible.scope -eq "durable" -and
    (Test-BrokerRecordOmitsValue -Record $brokerObservationVisible)
)
Add-Predicate `
    -Name "broker-durable-included:decision-and-observation" `
    -Expected "memory.context durable_records includes the resolved system decision and agent observation with metadata only (no raw value)" `
    -Passed $brokerIncludedOk `
    -Actual $(if ($brokerIncludedOk) { "matched" } else { ($brokerRecords | ConvertTo-Json -Compress -Depth 12) })

$brokerVisibleIds = @($brokerRecords | ForEach-Object { $_.id })
$brokerSupersededFold = @($brokerOmitted | Where-Object { $_.kind -eq "durable_superseded" })[0]
$brokerSupersededIds = if ($brokerSupersededFold) { @($brokerSupersededFold.ids) } else { @() }
$brokerSupersedeOk = (
    $brokerSupersededIds -contains $brokerDecisionAId -and
    -not ($brokerVisibleIds -contains $brokerDecisionAId)
)
Add-Predicate `
    -Name "broker-durable-supersede:A-hidden" `
    -Expected "the superseded decision A is folded under durable_superseded and is not visible in durable_records" `
    -Passed $brokerSupersedeOk `
    -Actual $(if ($brokerSupersedeOk) { "matched" } else { "records=$($brokerRecords | ConvertTo-Json -Compress -Depth 12) omitted=$($brokerOmitted | ConvertTo-Json -Compress -Depth 12)" })

$idxCap = Get-BrokerDurableRecordIndex -Records $brokerRecords -Id $brokerCapDenialId
$idxProblem = Get-BrokerDurableRecordIndex -Records $brokerRecords -Id $brokerProblemId
$idxDecisionB = Get-BrokerDurableRecordIndex -Records $brokerRecords -Id $brokerDecisionBId
$idxObservation = Get-BrokerDurableRecordIndex -Records $brokerRecords -Id $brokerObservationId
$brokerOrderingOk = (
    $idxCap -ge 0 -and
    $idxProblem -gt $idxCap -and
    $idxDecisionB -gt $idxProblem -and
    $idxObservation -gt $idxDecisionB
)
Add-Predicate `
    -Name "broker-ordering:frame-seq-order" `
    -Expected "visible durable_records remain in reclog frame-seq order after resolution" `
    -Passed $brokerOrderingOk `
    -Actual $(if ($brokerOrderingOk) { "matched" } else { ($brokerVisibleIds -join ",") })

$brokerLocalOnlyFold = @($brokerOmitted | Where-Object { $_.kind -eq "durable_local_only_value" }).Count -eq 1
$brokerClassificationOk = (
    $brokerObservationVisible -and
    $brokerObservationVisible.classification -eq "local_only" -and
    [bool]$brokerObservationVisible.exportable -eq $false -and
    $brokerLocalOnlyFold
)
Add-Predicate `
    -Name "broker-classification:local-only-not-exportable" `
    -Expected "a local_only durable record is surfaced locally with exportable=false and a durable_local_only_value omission fold" `
    -Passed $brokerClassificationOk `
    -Actual $(if ($brokerClassificationOk) { "matched" } else { "record=$($brokerObservationVisible | ConvertTo-Json -Compress -Depth 8) omitted=$($brokerOmitted | ConvertTo-Json -Compress -Depth 12)" })

$brokerProjectionJson = $brokerContext.provider_projection | ConvertTo-Json -Depth 20 -Compress
$brokerProjectionClean = (
    $brokerContext.provider_projection -and
    -not ($brokerProjectionJson -match [regex]::Escape($brokerDecisionBId)) -and
    -not ($brokerProjectionJson -match [regex]::Escape($brokerObservationId)) -and
    -not ($brokerProjectionJson -match '"durable_records"')
)
$brokerExportStillClosed = (
    $brokerContext.provider_export -eq "disabled" -and
    $brokerProviderExport.body.code -eq "capability_denied" -and
    $brokerProviderExport.body.method -eq "provider.context_export" -and
    $brokerProjectionClean
)
Add-Predicate `
    -Name "broker-export-still-closed:provider-projection-clean" `
    -Expected "memory.context provider_export stays disabled; provider.context_export is denied; durable content is absent from provider_projection" `
    -Passed $brokerExportStillClosed `
    -Actual $(if ($brokerExportStillClosed) { "matched" } else { "context=$($brokerContext | ConvertTo-Json -Compress -Depth 20) export=$($brokerProviderExport | ConvertTo-Json -Compress -Depth 10)" })

$brokerSelftestCaseNames = @($brokerSelftest.cases | ForEach-Object { $_.case })
$brokerExpectedSelftestCases = @(
    "R1_audit_supersede_ignored",
    "LOW_1_frame_seq_beats_payload_sequence",
    "audit_id_shadow_keeps_audit_visible",
    "LOW_3_source_record_id_spoof_is_not_identity"
)
$brokerSelftestCasesPresent = $true
foreach ($caseName in $brokerExpectedSelftestCases) {
    if (-not ($brokerSelftestCaseNames -contains $caseName)) {
        $brokerSelftestCasesPresent = $false
    }
}
$brokerResolveSelftestOk = (
    $brokerSelftest.schema -eq "raios.memory_broker_resolve_selftest.v0" -and
    [bool]$brokerSelftest.passed -and
    [int64]$brokerSelftest.case_count -eq 4 -and
    $brokerSelftestCasesPresent
)
Add-Predicate `
    -Name "broker-resolve-selftest:R1-LOW1-audit-shadow-LOW3" `
    -Expected "memory.broker_resolve_selftest passes R1, LOW-1, audit id shadow, and LOW-3 cases" `
    -Passed $brokerResolveSelftestOk `
    -Actual $(if ($brokerResolveSelftestOk) { "matched" } else { ($brokerSelftest | ConvertTo-Json -Compress -Depth 12) })

if (-not ($brokerIncludedOk -and $brokerSupersedeOk -and $brokerOrderingOk -and $brokerClassificationOk -and $brokerExportStillClosed -and $brokerResolveSelftestOk)) {
    throw "memory broker durable context family failed"
}

# --- export-denial-durable: provider.context_export denials append exactly one
#     durable capability_denial record per gate/reason key, then dedupe repeats. ---

$exportDenialExtraCommands = @(
    [pscustomobject]@{ Key = "export1"; Command = "provider.context_export provider_minimal"; Method = "provider.context_export" },
    [pscustomobject]@{ Key = "export2"; Command = "provider.context_export provider_minimal"; Method = "provider.context_export" },
    [pscustomobject]@{ Key = "export3"; Command = "provider.context_export diagnostic"; Method = "provider.context_export" },
    [pscustomobject]@{ Key = "scan2"; Command = "durable.record_log_scan"; Method = "durable.record_log_scan" },
    [pscustomobject]@{ Key = "context"; Command = "memory.context provider_minimal"; Method = "memory.context" }
)
$exportDenialProbe = Invoke-MemoryRecordAppendFixtureProbe -FixtureSpec "valid:2" -Label "export-denial" -ExtraAgentCommands $exportDenialExtraCommands
$export1 = $exportDenialProbe.extra["export1"]
$export2 = $exportDenialProbe.extra["export2"]
$export3 = $exportDenialProbe.extra["export3"]
$exportScan2 = $exportDenialProbe.extra["scan2"].body.result
$exportContext = $exportDenialProbe.extra["context"].body.result
$export1Audit = $export1.body.durable_denial_audit
$export2Audit = $export2.body.durable_denial_audit
$export3Audit = $export3.body.durable_denial_audit
$export1RecordId = "mem.capability_denial.provider_context_export.trust_state_not_positive.current_boot.v0"
$export3RecordId = "mem.capability_denial.provider_context_export.profile_not_provider_minimal.current_boot.v0"

$exportEnvelopeStillDenied = (
    $export1.body.code -eq "capability_denied" -and
    $export1.body.method -eq "provider.context_export"
)
Add-Predicate `
    -Name "export-denial-durable:envelope-still-denied" `
    -Expected "provider.context_export still returns capability_denied" `
    -Passed $exportEnvelopeStillDenied `
    -Actual $(if ($exportEnvelopeStillDenied) { "matched" } else { ($export1 | ConvertTo-Json -Compress -Depth 12) })

$exportGateAndReason = (
    $export1Audit.gate -eq "scoped_provider_export_authorization.current_boot.provider_minimal.v0" -and
    $export1Audit.gate_reason -eq "trust_state_not_positive"
)
Add-Predicate `
    -Name "export-denial-durable:gate-and-reason" `
    -Expected "the durable denial audit cites the scoped provider export gate and trust_state_not_positive" `
    -Passed $exportGateAndReason `
    -Actual $(if ($exportGateAndReason) { "matched" } else { ($export1Audit | ConvertTo-Json -Compress -Depth 12) })

$exportFirstAppended = (
    $export1Audit.durable_append -eq "appended" -and
    [bool]$export1Audit.performed -and
    $export1Audit.dedupe -eq "first_denial_appended" -and
    $export1Audit.kind -eq "capability_denial" -and
    $export1Audit.classification -eq "local_only" -and
    $export1Audit.record_authority -eq "core_ledger" -and
    $export1Audit.record_id -eq $export1RecordId -and
    $export1Audit.readback_sha256 -eq $export1Audit.frame_sha256 -and
    [bool]$export1Audit.reparse_valid
)
Add-Predicate `
    -Name "export-denial-durable:first-appended" `
    -Expected "first provider export denial appends one durable capability_denial memory record" `
    -Passed $exportFirstAppended `
    -Actual $(if ($exportFirstAppended) { "matched" } else { ($export1Audit | ConvertTo-Json -Compress -Depth 12) })

$exportNonSuperseding = (@($export1Audit.supersedes).Count -eq 0)
Add-Predicate `
    -Name "export-denial-durable:non-superseding" `
    -Expected "provider export denial audit record has an empty supersedes array" `
    -Passed $exportNonSuperseding `
    -Actual $(if ($exportNonSuperseding) { "matched" } else { ($export1Audit.supersedes | ConvertTo-Json -Compress -Depth 4) })

$exportChainAdvance = (
    [int64]$export1Audit.tail_seq_after -eq ([int64]$export1Audit.tail_seq_before + 1) -and
    [int64]$export1Audit.count_after -eq ([int64]$export1Audit.count_before + 1)
)
Add-Predicate `
    -Name "export-denial-durable:chain-advance" `
    -Expected "first denial append advances tail_seq and count by exactly one" `
    -Passed $exportChainAdvance `
    -Actual $(if ($exportChainAdvance) { "matched" } else { ($export1Audit | ConvertTo-Json -Compress -Depth 12) })

$exportDuplicateDeduped = (
    $export2Audit.dedupe -eq "duplicate_ram_only" -and
    $export2Audit.durable_append -eq "not_attempted_deduplicated" -and
    -not [bool]$export2Audit.performed -and
    $export2Audit.record_id -eq $export1Audit.record_id -and
    $export2Audit.payload_sha256 -eq $export1Audit.payload_sha256
)
Add-Predicate `
    -Name "export-denial-durable:duplicate-deduped" `
    -Expected "second identical denial cites the first audit and does not append" `
    -Passed $exportDuplicateDeduped `
    -Actual $(if ($exportDuplicateDeduped) { "matched" } else { "export1=$($export1Audit | ConvertTo-Json -Compress -Depth 12) export2=$($export2Audit | ConvertTo-Json -Compress -Depth 12)" })

$exportDistinctReasonAppended = (
    $export3Audit.gate_reason -eq "profile_not_provider_minimal" -and
    $export3Audit.durable_append -eq "appended" -and
    $export3Audit.record_id -eq $export3RecordId -and
    $export3Audit.record_id -ne $export1Audit.record_id
)
Add-Predicate `
    -Name "export-denial-durable:distinct-reason-appended" `
    -Expected "a distinct export gate reason appends a separate durable denial audit" `
    -Passed $exportDistinctReasonAppended `
    -Actual $(if ($exportDistinctReasonAppended) { "matched" } else { ($export3Audit | ConvertTo-Json -Compress -Depth 12) })

$exportDedupeCount = (
    [int64]$exportScan2.count -eq ([int64]$exportDenialProbe.scan.count + 2) -and
    [int64]$exportScan2.tail_seq -eq ([int64]$exportDenialProbe.scan.tail_seq + 2)
)
Add-Predicate `
    -Name "export-denial-durable:dedupe-count" `
    -Expected "three export attempts with two distinct denial reasons append exactly two frames" `
    -Passed $exportDedupeCount `
    -Actual $(if ($exportDedupeCount) { "matched" } else { "baseline=$($exportDenialProbe.scan | ConvertTo-Json -Compress -Depth 8) scan2=$($exportScan2 | ConvertTo-Json -Compress -Depth 8)" })

$exportContextRecords = @($exportContext.durable_records)
$exportContextRecord = Get-BrokerDurableRecordById -Records $exportContextRecords -Id $export1RecordId
$exportStillDisabled = (
    $exportContext.provider_export -eq "disabled" -and
    $exportContextRecord -and
    $exportContextRecord.kind -eq "capability_denial" -and
    $exportContextRecord.classification -eq "local_only" -and
    [bool]$exportContextRecord.exportable -eq $false -and
    (Test-BrokerRecordOmitsValue -Record $exportContextRecord)
)
Add-Predicate `
    -Name "export-denial-durable:export-still-disabled" `
    -Expected "memory.context keeps provider_export disabled and surfaces the local-only denial metadata without value" `
    -Passed $exportStillDisabled `
    -Actual $(if ($exportStillDisabled) { "matched" } else { "context=$($exportContext | ConvertTo-Json -Compress -Depth 14)" })

if (-not ($exportEnvelopeStillDenied -and $exportGateAndReason -and $exportFirstAppended -and $exportNonSuperseding -and $exportChainAdvance -and $exportDuplicateDeduped -and $exportDistinctReasonAppended -and $exportDedupeCount -and $exportStillDisabled)) {
    throw "export-denial-durable family failed"
}

# --- export-packet: public-only provider-export packet evidence assembly. The
#     helper's primary memory.record_log_append gives the child one local_only
#     durable record; the extras append exactly one public fixture, scan, then run
#     the packet selftest twice to prove hash determinism and no selftest write. ---

$exportPacketFixtureId = "mem.decision.provider_export_public_fixture.current_boot.v0"
$exportPacketExtraCommands = @(
    [pscustomobject]@{ Key = "fixture"; Command = "memory.provider_export_public_fixture_append"; Method = "memory.provider_export_public_fixture_append" },
    [pscustomobject]@{ Key = "scan_before"; Command = "durable.record_log_scan"; Method = "durable.record_log_scan" },
    [pscustomobject]@{ Key = "packet1"; Command = "provider.context_export_packet_selftest provider_minimal"; Method = "provider.context_export_packet_selftest" },
    [pscustomobject]@{ Key = "packet2"; Command = "provider.context_export_packet_selftest provider_minimal"; Method = "provider.context_export_packet_selftest" },
    [pscustomobject]@{ Key = "scan_after"; Command = "durable.record_log_scan"; Method = "durable.record_log_scan" }
)
$exportPacketProbe = Invoke-MemoryRecordAppendFixtureProbe -FixtureSpec "valid:2" -Label "export-packet" -AppendMethod "memory.record_log_append" -ExtraAgentCommands $exportPacketExtraCommands
$exportPacketFixture = $exportPacketProbe.extra["fixture"].body.result
$exportPacketScanBefore = $exportPacketProbe.extra["scan_before"].body.result
$packet1 = $exportPacketProbe.extra["packet1"].body.result
$packet2 = $exportPacketProbe.extra["packet2"].body.result
$exportPacketScanAfter = $exportPacketProbe.extra["scan_after"].body.result
$packet1Ids = @($packet1.included_public_ids)

$exportPacketPublicIncluded = (
    $exportPacketFixture.durable_append -eq "appended" -and
    [bool]$exportPacketFixture.performed -and
    $exportPacketFixture.record_id -eq $exportPacketFixtureId -and
    $exportPacketFixture.classification -eq "public" -and
    $packet1Ids -contains $exportPacketFixtureId
)
Add-Predicate `
    -Name "export-packet:public-record-included" `
    -Expected "packet1 included_public_ids contains the fixed public provider-export fixture id" `
    -Passed $exportPacketPublicIncluded `
    -Actual $(if ($exportPacketPublicIncluded) { "matched" } else { "fixture=$($exportPacketFixture | ConvertTo-Json -Compress -Depth 10) packet1=$($packet1 | ConvertTo-Json -Compress -Depth 10)" })

$exportPacketLocalOnlyIds = @(
    "mem.capability_denial.module_load_ephemeral_durable.current_boot.v0",
    "mem.decision.module_sharing_confirmed_vision.current_boot.v0",
    "mem.problem.memory_mutation_denied.current_boot.v0",
    "mem.decision.module_sharing_evidence_gated.current_boot.v0",
    "mem.observation.agent.current_boot.00000001.v0"
)
$exportPacketContainsLocalOnly = $false
foreach ($id in $exportPacketLocalOnlyIds) {
    if ($packet1Ids -contains $id) {
        $exportPacketContainsLocalOnly = $true
    }
}
$exportPacketLocalOnlyExcluded = (
    -not $exportPacketContainsLocalOnly -and
    [int64]$packet1.excluded_local_only_count -ge 1
)
Add-Predicate `
    -Name "export-packet:local-only-excluded" `
    -Expected "packet1 includes no known local_only durable ids and reports excluded_local_only_count >= 1" `
    -Passed $exportPacketLocalOnlyExcluded `
    -Actual $(if ($exportPacketLocalOnlyExcluded) { "matched" } else { ($packet1 | ConvertTo-Json -Compress -Depth 10) })

$exportPacketAllPublic = (
    [bool]$packet1.packet_all_records_public -and
    [int64]$packet1.packet_record_count -eq [int64]$packet1Ids.Count
)
Add-Predicate `
    -Name "export-packet:all-records-public-after-filter" `
    -Expected "packet_all_records_public == true and packet_record_count equals count(included_public_ids)" `
    -Passed $exportPacketAllPublic `
    -Actual $(if ($exportPacketAllPublic) { "matched" } else { ($packet1 | ConvertTo-Json -Compress -Depth 10) })

$exportPacketHashDeterministic = (
    $packet1.packet_hash -eq $packet2.packet_hash -and
    $packet1.packet_hash -match '^sha256:[0-9a-f]{64}$'
)
Add-Predicate `
    -Name "export-packet:hash-deterministic" `
    -Expected "packet1.packet_hash equals packet2.packet_hash and renders as sha256:<64hex>" `
    -Passed $exportPacketHashDeterministic `
    -Actual $(if ($exportPacketHashDeterministic) { "matched" } else { "packet1=$($packet1.packet_hash) packet2=$($packet2.packet_hash)" })

$exportPacketNoAuthNoWrite = (
    -not [bool]$packet1.gate_evaluated -and
    -not [bool]$packet1.authorized -and
    -not [bool]$packet1.export_performed -and
    $packet1.provider_write -eq "not_attempted" -and
    [bool]$packet1.test_infrastructure -and
    [int64]$exportPacketScanBefore.count -eq ([int64]$exportPacketProbe.scan.count + 1) -and
    [int64]$exportPacketScanBefore.tail_seq -eq ([int64]$exportPacketProbe.scan.tail_seq + 1) -and
    [int64]$exportPacketScanAfter.count -eq [int64]$exportPacketScanBefore.count -and
    [int64]$exportPacketScanAfter.tail_seq -eq [int64]$exportPacketScanBefore.tail_seq
)
Add-Predicate `
    -Name "export-packet:no-authorization-no-audit-no-write" `
    -Expected "packet selftest does not evaluate gates, authorize, export, write provider output, or append durable records" `
    -Passed $exportPacketNoAuthNoWrite `
    -Actual $(if ($exportPacketNoAuthNoWrite) { "matched" } else { "packet1=$($packet1 | ConvertTo-Json -Compress -Depth 10) helper_scan=$($exportPacketProbe.scan | ConvertTo-Json -Compress -Depth 8) before=$($exportPacketScanBefore | ConvertTo-Json -Compress -Depth 8) after=$($exportPacketScanAfter | ConvertTo-Json -Compress -Depth 8)" })

if (-not ($exportPacketPublicIncluded -and $exportPacketLocalOnlyExcluded -and $exportPacketAllPublic -and $exportPacketHashDeterministic -and $exportPacketNoAuthNoWrite)) {
    throw "export-packet family failed"
}

# --- export-authorized-selftest: fixed synthetic positive vector authorizes the
#     gate, appends exactly one local_only export_audit, and still transmits
#     nothing; a simulated non-public packet stays denied. -------------------------

$exportAuthorizedAuditId = "mem.export_audit.provider_context_export_selftest.current_boot.v0"
$exportAuthorizedExtraCommands = @(
    [pscustomobject]@{ Key = "fixture"; Command = "memory.provider_export_public_fixture_append"; Method = "memory.provider_export_public_fixture_append" },
    [pscustomobject]@{ Key = "scanBefore"; Command = "durable.record_log_scan"; Method = "durable.record_log_scan" },
    [pscustomobject]@{ Key = "authorized"; Command = "provider.context_export_authorized_selftest provider_minimal"; Method = "provider.context_export_authorized_selftest" },
    [pscustomobject]@{ Key = "authorized2"; Command = "provider.context_export_authorized_selftest provider_minimal"; Method = "provider.context_export_authorized_selftest" },
    [pscustomobject]@{ Key = "smuggle"; Command = "provider.context_export_authorized_selftest_smuggle provider_minimal"; Method = "provider.context_export_authorized_selftest_smuggle" },
    [pscustomobject]@{ Key = "scanAfter"; Command = "durable.record_log_scan"; Method = "durable.record_log_scan" },
    [pscustomobject]@{ Key = "realDenied"; Command = "provider.context_export provider_minimal"; Method = "provider.context_export" },
    [pscustomobject]@{ Key = "context"; Command = "memory.context provider_minimal"; Method = "memory.context" }
)
$exportAuthorizedProbe = Invoke-MemoryRecordAppendFixtureProbe -FixtureSpec "valid:2" -Label "export-authorized-selftest" -AppendMethod "memory.record_log_append" -ExtraAgentCommands $exportAuthorizedExtraCommands
$exportAuthorizedFixture = $exportAuthorizedProbe.extra["fixture"].body.result
$exportAuthorizedScanBefore = $exportAuthorizedProbe.scan
$exportAuthorizedFixtureScan = $exportAuthorizedProbe.extra["scanBefore"].body.result
$exportAuthorized = $exportAuthorizedProbe.extra["authorized"].body.result
$exportAuthorizedAudit = $exportAuthorized.durable_export_audit
$exportAuthorized2 = $exportAuthorizedProbe.extra["authorized2"].body.result
$exportAuthorized2Audit = $exportAuthorized2.durable_export_audit
$exportSmuggle = $exportAuthorizedProbe.extra["smuggle"].body.result
$exportAuthorizedScanAfter = $exportAuthorizedProbe.extra["scanAfter"].body.result
$exportRealDenied = $exportAuthorizedProbe.extra["realDenied"]
$exportAuthorizedContext = $exportAuthorizedProbe.extra["context"].body.result

$exportAuthorizedGateOk = (
    [bool]$exportAuthorized.authorized -and
    $exportAuthorized.gate_reason -eq "authorized_provider_export_public_only_audited"
)
Add-Predicate `
    -Name "export-authorized-selftest:gate-authorized" `
    -Expected "authorized selftest reports authorized=true and the scoped provider export success reason" `
    -Passed $exportAuthorizedGateOk `
    -Actual $(if ($exportAuthorizedGateOk) { "matched" } else { ($exportAuthorized | ConvertTo-Json -Compress -Depth 14) })

$exportAuthorizedAuditAppended = (
    $exportAuthorizedAudit -and
    $exportAuthorizedAudit.durable_append -eq "appended" -and
    [bool]$exportAuthorizedAudit.performed -and
    $exportAuthorizedAudit.record_id -eq $exportAuthorizedAuditId -and
    $exportAuthorizedAudit.kind -eq "export_audit" -and
    $exportAuthorizedAudit.classification -eq "local_only" -and
    $exportAuthorizedAudit.record_authority -eq "core_ledger" -and
    $exportAuthorizedAudit.readback_sha256 -eq $exportAuthorizedAudit.frame_sha256 -and
    [bool]$exportAuthorizedAudit.reparse_valid
)
Add-Predicate `
    -Name "export-authorized-selftest:durable-export-audit-appended" `
    -Expected "authorized selftest appends one durable local_only export_audit memory record through the shared gauntlet" `
    -Passed $exportAuthorizedAuditAppended `
    -Actual $(if ($exportAuthorizedAuditAppended) { "matched" } else { ($exportAuthorizedAudit | ConvertTo-Json -Compress -Depth 14) })

$exportAuthorized2Deduped = (
    $exportAuthorized2Audit -and
    $exportAuthorized2Audit.dedupe -eq "duplicate_ram_only" -and
    $exportAuthorized2Audit.durable_append -eq "not_attempted_deduplicated" -and
    -not [bool]$exportAuthorized2Audit.performed -and
    $exportAuthorized2Audit.record_id -eq $exportAuthorizedAudit.record_id -and
    $exportAuthorized2Audit.payload_sha256 -eq $exportAuthorizedAudit.payload_sha256 -and
    $exportAuthorized2Audit.seq -eq $exportAuthorizedAudit.seq
)
Add-Predicate `
    -Name "export-authorized-selftest:dedupe-second-appends-nothing" `
    -Expected "the second authorized selftest cites the first export_audit and appends no duplicate frame" `
    -Passed $exportAuthorized2Deduped `
    -Actual $(if ($exportAuthorized2Deduped) { "matched" } else { "first=$($exportAuthorizedAudit | ConvertTo-Json -Compress -Depth 14) second=$($exportAuthorized2Audit | ConvertTo-Json -Compress -Depth 14)" })

$exportAuthorizedNonSuperseding = (@($exportAuthorizedAudit.supersedes).Count -eq 0)
Add-Predicate `
    -Name "export-authorized-selftest:non-superseding" `
    -Expected "authorized export_audit has an empty supersedes array" `
    -Passed $exportAuthorizedNonSuperseding `
    -Actual $(if ($exportAuthorizedNonSuperseding) { "matched" } else { ($exportAuthorizedAudit.supersedes | ConvertTo-Json -Compress -Depth 4) })

$exportAuthorizedHonestLabels = (
    -not [bool]$exportAuthorized.owner_sealed -and
    -not [bool]$exportAuthorized.persistence_claimed -and
    $exportAuthorized.trust_tier -eq "dev_key_not_owner_sealed" -and
    -not [bool]$exportAuthorized.export_performed -and
    $exportAuthorized.provider_write -eq "selftest_no_transmission" -and
    [bool]$exportAuthorized.test_infrastructure
)
Add-Predicate `
    -Name "export-authorized-selftest:honest-labels" `
    -Expected "authorized selftest is dev-labeled, records no owner seal, no persistence claim, no export, and no provider write" `
    -Passed $exportAuthorizedHonestLabels `
    -Actual $(if ($exportAuthorizedHonestLabels) { "matched" } else { ($exportAuthorized | ConvertTo-Json -Compress -Depth 14) })

$smuggleAuditProperty = $exportSmuggle.PSObject.Properties["durable_export_audit"]
$smuggleAuditPerformed = $smuggleAuditProperty -and [bool]$smuggleAuditProperty.Value.performed
$exportSmuggleDenied = (
    -not [bool]$exportSmuggle.authorized -and
    $exportSmuggle.gate_reason -eq "packet_contains_non_public_record" -and
    -not $smuggleAuditPerformed
)
Add-Predicate `
    -Name "export-authorized-selftest:local-only-smuggle-denied" `
    -Expected "simulated non-public packet denies with packet_contains_non_public_record and appends no export audit" `
    -Passed $exportSmuggleDenied `
    -Actual $(if ($exportSmuggleDenied) { "matched" } else { ($exportSmuggle | ConvertTo-Json -Compress -Depth 14) })

$exportAuthorizedChainAdvance = (
    $exportAuthorizedFixture.durable_append -eq "appended" -and
    [int64]$exportAuthorizedFixtureScan.count -eq ([int64]$exportAuthorizedScanBefore.count + 1) -and
    [int64]$exportAuthorizedFixtureScan.tail_seq -eq ([int64]$exportAuthorizedScanBefore.tail_seq + 1) -and
    [int64]$exportAuthorizedScanAfter.count -eq ([int64]$exportAuthorizedScanBefore.count + 2) -and
    [int64]$exportAuthorizedScanAfter.tail_seq -eq ([int64]$exportAuthorizedScanBefore.tail_seq + 2)
)
Add-Predicate `
    -Name "export-authorized-selftest:audit-chain-advance" `
    -Expected "scanAfter advances by fixture append plus one export_audit; authorized2 and smuggle append nothing" `
    -Passed $exportAuthorizedChainAdvance `
    -Actual $(if ($exportAuthorizedChainAdvance) { "matched" } else { "baseline=$($exportAuthorizedScanBefore | ConvertTo-Json -Compress -Depth 8) fixture_scan=$($exportAuthorizedFixtureScan | ConvertTo-Json -Compress -Depth 8) scanAfter=$($exportAuthorizedScanAfter | ConvertTo-Json -Compress -Depth 8) fixture=$($exportAuthorizedFixture | ConvertTo-Json -Compress -Depth 10)" })

$exportRealStillDenied = (
    $exportRealDenied.body.code -eq "capability_denied" -and
    $exportRealDenied.body.method -eq "provider.context_export" -and
    $exportRealDenied.body.durable_denial_audit
)
Add-Predicate `
    -Name "export-authorized-selftest:real-provider-context-export-still-denied" `
    -Expected "the real provider.context_export path still returns capability_denied with its durable denial audit" `
    -Passed $exportRealStillDenied `
    -Actual $(if ($exportRealStillDenied) { "matched" } else { ($exportRealDenied | ConvertTo-Json -Compress -Depth 14) })

$exportAuthorizedContextRecords = @($exportAuthorizedContext.durable_records)
$exportAuthorizedContextRecord = Get-BrokerDurableRecordById -Records $exportAuthorizedContextRecords -Id $exportAuthorizedAuditId
$exportAuthorizedContextShowsAudit = (
    $exportAuthorizedContextRecord -and
    $exportAuthorizedContextRecord.kind -eq "export_audit" -and
    $exportAuthorizedContextRecord.classification -eq "local_only" -and
    [bool]$exportAuthorizedContextRecord.exportable -eq $false
)
Add-Predicate `
    -Name "export-authorized-selftest:memory-context-shows-export-audit-local-only" `
    -Expected "memory.context durable_records shows the export_audit metadata as local_only and not exportable" `
    -Passed $exportAuthorizedContextShowsAudit `
    -Actual $(if ($exportAuthorizedContextShowsAudit) { "matched" } else { ($exportAuthorizedContext | ConvertTo-Json -Compress -Depth 16) })

$exportAuthorizedProviderExportDisabled = ($exportAuthorizedContext.provider_export -eq "disabled")
Add-Predicate `
    -Name "export-authorized-selftest:provider-export-status-still-disabled" `
    -Expected "memory.context provider_export remains disabled" `
    -Passed $exportAuthorizedProviderExportDisabled `
    -Actual $(if ($exportAuthorizedProviderExportDisabled) { "matched" } else { ($exportAuthorizedContext | ConvertTo-Json -Compress -Depth 12) })

if (-not ($exportAuthorizedGateOk -and $exportAuthorizedAuditAppended -and $exportAuthorized2Deduped -and $exportAuthorizedNonSuperseding -and $exportAuthorizedHonestLabels -and $exportSmuggleDenied -and $exportAuthorizedChainAdvance -and $exportRealStillDenied -and $exportAuthorizedContextShowsAudit -and $exportAuthorizedProviderExportDisabled)) {
    throw "export-authorized-selftest family failed"
}

# --- memory-agent-observation-denied: 5 distinct RAM-only denials against the MAIN
#     VM (M9B-1b) -- each a different malformed `memory.observation_log_append`
#     argument. Bracketed by a single before/after durable.record_log_scan on the
#     main VM proving NOTHING was appended across all 5 attempts. ------------------

function ConvertTo-Base64AsciiArg {
    param([string]$Text)
    $bytes = [System.Text.Encoding]::ASCII.GetBytes($Text)
    return [Convert]::ToBase64String($bytes)
}

# (a) not base64 at all.
$agentObsBadNotBase64 = "!!!!"
# (b) valid base64, but only 3 newline-separated fields (wrong count).
$agentObsBadFieldCount = ConvertTo-Base64AsciiArg "vm.smoke.bad`nobserved`nvalue"
# (c) valid base64, 4 fields, but entity is 65 bytes (over the 64-byte cap).
$agentObsBadTooLong = ConvertTo-Base64AsciiArg ("x" * 65 + "`nobserved`nvalue`nvm.smoke.source")
# (d) valid base64, 4 fields, but entity carries a disallowed charset byte (a quote).
$agentObsBadCharset = ConvertTo-Base64AsciiArg "vm.smoke.bad`"entity`nobserved`nvalue`nvm.smoke.source"
# (e) valid base64, 4 fields, but entity is empty.
$agentObsBadEmptyEntity = ConvertTo-Base64AsciiArg "`nobserved`nvalue`nvm.smoke.source"

Send-AgentCommand -Command "agent durable.record_log_scan" -ExpectedMarker "RAIOS_AGENT_END durable.record_log_scan" -Name "memory-agent-observation-denied:scan_before"
$agentObsScanBefore = (Get-LastAgentResponseJson -Method "durable.record_log_scan").body.result

$agentObservationDeniedCases = @(
    @{ suffix = "not-base64"; arg = $agentObsBadNotBase64; expectedReason = "rejected_malformed_base64_chunk" },
    @{ suffix = "field-count"; arg = $agentObsBadFieldCount; expectedReason = "agent_observation_field_count" },
    @{ suffix = "field-too-long"; arg = $agentObsBadTooLong; expectedReason = "agent_observation_field_too_long" },
    @{ suffix = "field-charset"; arg = $agentObsBadCharset; expectedReason = "agent_observation_field_charset" },
    @{ suffix = "entity-empty"; arg = $agentObsBadEmptyEntity; expectedReason = "agent_observation_entity_empty" }
)

$agentObservationDeniedAllOk = $true
foreach ($case in $agentObservationDeniedCases) {
    Send-AgentCommand -Command "agent memory.observation_log_append $($case.arg)" -ExpectedMarker "RAIOS_AGENT_END memory.observation_log_append" -Name "memory-agent-observation-denied:$($case.suffix)_send"
    $result = (Get-LastAgentResponseJson -Method "memory.observation_log_append").body.result
    $caseOk = (
        $result.durable_append -eq "capability_denied" -and
        [bool]$result.performed -eq $false -and
        $result.reason -eq $case.expectedReason
    )
    if (-not $caseOk) { $agentObservationDeniedAllOk = $false }
    Add-Predicate `
        -Name "memory-agent-observation-denied:$($case.suffix)" `
        -Expected "durable_append == `"capability_denied`" and performed == false and reason == `"$($case.expectedReason)`"" `
        -Passed $caseOk `
        -Actual $(if ($caseOk) { "matched" } else { ($result | ConvertTo-Json -Compress -Depth 8) })
}

Send-AgentCommand -Command "agent durable.record_log_scan" -ExpectedMarker "RAIOS_AGENT_END durable.record_log_scan" -Name "memory-agent-observation-denied:scan_after"
$agentObsScanAfter = (Get-LastAgentResponseJson -Method "durable.record_log_scan").body.result

$agentObservationDeniedNothingAppended = (
    [int64]$agentObsScanAfter.count -eq [int64]$agentObsScanBefore.count -and
    [int64]$agentObsScanAfter.tail_seq -eq [int64]$agentObsScanBefore.tail_seq
)
Add-Predicate `
    -Name "memory-agent-observation-denied:nothing-appended" `
    -Expected "all 5 malformed memory.observation_log_append attempts are RAM-only: reclog count/tail_seq unchanged across the whole set" `
    -Passed $agentObservationDeniedNothingAppended `
    -Actual $(if ($agentObservationDeniedNothingAppended) { "matched" } else { "before=$($agentObsScanBefore | ConvertTo-Json -Compress) after=$($agentObsScanAfter | ConvertTo-Json -Compress)" })

if (-not ($agentObservationDeniedAllOk -and $agentObservationDeniedNothingAppended)) {
    throw "memory-agent-observation-denied family failed"
}

# --- memory-durable-secret-denied / memory-durable-quota: synthetic selftest ------
# One selftest call covers BOTH families: constructor fail-closed cases (secret
# classification / unknown kind) and the scoped evaluator's own defensive pins
# (classification_secret_never_durable / memory_record_kind_out_of_scope /
# memory_write_quota_exhausted). NONE of these cases ever call
# durable_store::append_memory_record, so nothing is appended -- verified below by
# scanning the MAIN VM's reclog before and after.

Send-AgentCommand -Command "agent durable.record_log_scan" -ExpectedMarker "RAIOS_AGENT_END durable.record_log_scan" -Name "memory-durable-selftest:scan_before"
$scanBefore = (Get-LastAgentResponseJson -Method "durable.record_log_scan").body.result

Send-AgentCommand -Command "agent memory.record_log_append_selftest" -ExpectedMarker "RAIOS_AGENT_END memory.record_log_append_selftest" -Name "memory-durable-selftest:run"
$selftest = (Get-LastAgentResponseJson -Method "memory.record_log_append_selftest").body.result
$selftestCases = @($selftest.cases)

Send-AgentCommand -Command "agent durable.record_log_scan" -ExpectedMarker "RAIOS_AGENT_END durable.record_log_scan" -Name "memory-durable-selftest:scan_after"
$scanAfter = (Get-LastAgentResponseJson -Method "durable.record_log_scan").body.result

$selftestNothingAppended = (
    [int64]$scanAfter.count -eq [int64]$scanBefore.count -and
    [int64]$scanAfter.tail_seq -eq [int64]$scanBefore.tail_seq
)

$selftestShapeOk = (
    $selftest.schema -eq "raios.memory_record_append_selftest.v0" -and
    [bool]$selftest.test_infrastructure -and
    -not [bool]$selftest.mutates_global_event_log -and
    -not [bool]$selftest.writes_persistent_state -and
    [bool]$selftest.passed -and
    [int64]$selftest.case_count -eq 11
)
Add-Predicate `
    -Name "memory-durable-secret-denied:selftest-shape" `
    -Expected "memory.record_log_append_selftest is RAM-only test infrastructure and reports all cases passed" `
    -Passed $selftestShapeOk `
    -Actual $(if ($selftestShapeOk) { "matched" } else { ($selftest | ConvertTo-Json -Compress -Depth 10) })

$secretConstructorDenied = @($selftestCases | Where-Object { $_.case -eq "secret_classification_constructor_denied" -and $_.actual_status -eq "denied" -and $_.actual_reason -eq "secret_never_durable_until_sealed_secret_design" }).Count -eq 1
Add-Predicate `
    -Name "memory-durable-secret-denied:constructor-secret" `
    -Expected "MemoryRecord::new rejects secret classification with secret_never_durable_until_sealed_secret_design" `
    -Passed $secretConstructorDenied `
    -Actual $(if ($secretConstructorDenied) { "matched" } else { ($selftestCases | ConvertTo-Json -Compress -Depth 10) })

$unknownKindConstructorDenied = @($selftestCases | Where-Object { $_.case -eq "unknown_kind_constructor_denied" -and $_.actual_status -eq "denied" -and $_.actual_reason -eq "memory_record_kind_out_of_scope" }).Count -eq 1
Add-Predicate `
    -Name "memory-durable-secret-denied:constructor-unknown-kind" `
    -Expected "MemoryRecord::new rejects an unknown kind with memory_record_kind_out_of_scope" `
    -Passed $unknownKindConstructorDenied `
    -Actual $(if ($unknownKindConstructorDenied) { "matched" } else { ($selftestCases | ConvertTo-Json -Compress -Depth 10) })

$scopedSecretDenied = @($selftestCases | Where-Object { $_.case -eq "scoped_classification_secret_denied" -and $_.actual_status -eq "denied" -and $_.actual_reason -eq "classification_secret_never_durable" }).Count -eq 1
Add-Predicate `
    -Name "memory-durable-secret-denied:scoped-classification-secret" `
    -Expected "evaluate_scoped_memory_record_append defensively denies classification=secret with classification_secret_never_durable" `
    -Passed $scopedSecretDenied `
    -Actual $(if ($scopedSecretDenied) { "matched" } else { ($selftestCases | ConvertTo-Json -Compress -Depth 10) })

$scopedKindDenied = @($selftestCases | Where-Object { $_.case -eq "scoped_kind_out_of_scope_denied" -and $_.actual_status -eq "denied" -and $_.actual_reason -eq "memory_record_kind_out_of_scope" }).Count -eq 1
Add-Predicate `
    -Name "memory-durable-secret-denied:scoped-kind-out-of-scope" `
    -Expected "evaluate_scoped_memory_record_append defensively denies an out-of-scope kind with memory_record_kind_out_of_scope" `
    -Passed $scopedKindDenied `
    -Actual $(if ($scopedKindDenied) { "matched" } else { ($selftestCases | ConvertTo-Json -Compress -Depth 10) })

Add-Predicate `
    -Name "memory-durable-secret-denied:nothing-appended" `
    -Expected "the secret/bad-kind selftest denials are RAM-only: reclog count/tail_seq unchanged across the call" `
    -Passed $selftestNothingAppended `
    -Actual $(if ($selftestNothingAppended) { "matched" } else { "before=$($scanBefore | ConvertTo-Json -Compress) after=$($scanAfter | ConvertTo-Json -Compress)" })

$scopedQuotaDenied = @($selftestCases | Where-Object { $_.case -eq "scoped_quota_exhausted_denied" -and $_.actual_status -eq "denied" -and $_.actual_reason -eq "memory_write_quota_exhausted" }).Count -eq 1
Add-Predicate `
    -Name "memory-durable-quota:scoped-quota-exhausted" `
    -Expected "evaluate_scoped_memory_record_append denies quota_ok=false with memory_write_quota_exhausted" `
    -Passed $scopedQuotaDenied `
    -Actual $(if ($scopedQuotaDenied) { "matched" } else { ($selftestCases | ConvertTo-Json -Compress -Depth 10) })

# LIVE quota: the scoped case above only proves the evaluator's synthetic quota_ok=false
# pin. This drives the REAL MEMORY_WRITE_QUOTA static (via
# durable_store::memory_write_quota_probe_exhaustion) to exhaustion and back -- the one
# primitive this slice adds over its append_recovery_load reference -- so the live gate
# is proven to fire AND refund, not merely reasoned about. RAM-only: the selftest's
# before/after reclog scan (memory-durable-*-quota:nothing-appended) shows no durable write.
$liveQuotaCase = @($selftestCases | Where-Object { $_.case -eq "live_quota_exhausted_and_restored" -and [bool]$_.passed }).Count -eq 1
Add-Predicate `
    -Name "memory-durable-quota:live-gate-exhausts-and-restores" `
    -Expected "the REAL per-boot RAM quota reserves a finite number of records then denies, then fully refunds so a transient denial never permanently burns the boot budget" `
    -Passed $liveQuotaCase `
    -Actual $(if ($liveQuotaCase) { "matched" } else { ($selftestCases | ConvertTo-Json -Compress -Depth 10) })

$liveQuotaEvidence = (
    [int64]$selftest.live_quota_reservations_until_exhausted -ge 1 -and
    [bool]$selftest.live_quota_restored
)
Add-Predicate `
    -Name "memory-durable-quota:live-reservations-evidence" `
    -Expected "the live quota admits >=1 reservation before the gate fires and reports restored=true afterwards" `
    -Passed $liveQuotaEvidence `
    -Actual $(if ($liveQuotaEvidence) { "matched" } else { "reservations=$($selftest.live_quota_reservations_until_exhausted) restored=$($selftest.live_quota_restored)" })

$quotaBudgetHonest = (
    [int64]$selftest.quota_budget_records -eq 128 -and
    [int64]$selftest.quota_budget_bytes -eq 32768
)
Add-Predicate `
    -Name "memory-durable-quota:budget-per-boot-bounded" `
    -Expected "the RAM-only per-boot memory write quota is honestly bounded (128 records / 32768 bytes)" `
    -Passed $quotaBudgetHonest `
    -Actual $(if ($quotaBudgetHonest) { "matched" } else { "records=$($selftest.quota_budget_records) bytes=$($selftest.quota_budget_bytes)" })

Add-Predicate `
    -Name "memory-durable-quota:nothing-appended" `
    -Expected "the quota-exhaustion selftest denial is RAM-only: reclog count/tail_seq unchanged across the call" `
    -Passed $selftestNothingAppended `
    -Actual $(if ($selftestNothingAppended) { "matched" } else { "before=$($scanBefore | ConvertTo-Json -Compress) after=$($scanAfter | ConvertTo-Json -Compress)" })

# M9A-3b: 5 new RAM-only fail-closed constructor cases (never call
# durable_store::append_memory_record, so nothing is appended -- the SAME
# before/after main-VM reclog scan above already proves that for the whole call).
$auditKindSupersedeDenied = @($selftestCases | Where-Object { $_.case -eq "audit_kind_supersede_denied" -and $_.actual_status -eq "denied" -and $_.actual_reason -eq "audit_kind_may_not_supersede" }).Count -eq 1
Add-Predicate `
    -Name "memory-durable-supersede:selftest-audit-kind-supersede-denied" `
    -Expected "MemoryRecord::new rejects an audit kind (capability_denial) authored as a superseding record with audit_kind_may_not_supersede" `
    -Passed $auditKindSupersedeDenied `
    -Actual $(if ($auditKindSupersedeDenied) { "matched" } else { ($selftestCases | ConvertTo-Json -Compress -Depth 10) })

$supersedesListTooLongDenied = @($selftestCases | Where-Object { $_.case -eq "supersedes_list_too_long_denied" -and $_.actual_status -eq "denied" -and $_.actual_reason -eq "supersedes_list_too_long" }).Count -eq 1
Add-Predicate `
    -Name "memory-durable-supersede:selftest-supersedes-list-too-long-denied" `
    -Expected "MemoryRecord::new rejects a supersedes list past MAX_SUPERSEDES_PER_RECORD (8) with supersedes_list_too_long" `
    -Passed $supersedesListTooLongDenied `
    -Actual $(if ($supersedesListTooLongDenied) { "matched" } else { ($selftestCases | ConvertTo-Json -Compress -Depth 10) })

$selfSupersedeDenied = @($selftestCases | Where-Object { $_.case -eq "self_supersede_denied" -and $_.actual_status -eq "denied" -and $_.actual_reason -eq "supersede_self_reference" }).Count -eq 1
Add-Predicate `
    -Name "memory-durable-supersede:selftest-self-supersede-denied" `
    -Expected "MemoryRecord::new rejects a record naming its own id in supersedes with supersede_self_reference" `
    -Passed $selfSupersedeDenied `
    -Actual $(if ($selfSupersedeDenied) { "matched" } else { ($selftestCases | ConvertTo-Json -Compress -Depth 10) })

$decisionMissingSourceDenied = @($selftestCases | Where-Object { $_.case -eq "decision_missing_source_denied" -and $_.actual_status -eq "denied" -and $_.actual_reason -eq "decision_missing_source" }).Count -eq 1
Add-Predicate `
    -Name "memory-durable-supersede:selftest-decision-missing-source-denied" `
    -Expected "MemoryRecord::new rejects a decision with an empty source with decision_missing_source" `
    -Passed $decisionMissingSourceDenied `
    -Actual $(if ($decisionMissingSourceDenied) { "matched" } else { ($selftestCases | ConvertTo-Json -Compress -Depth 10) })

$problemMissingStatusDenied = @($selftestCases | Where-Object { $_.case -eq "problem_missing_status_denied" -and $_.actual_status -eq "denied" -and $_.actual_reason -eq "problem_missing_status" }).Count -eq 1
Add-Predicate `
    -Name "memory-durable-supersede:selftest-problem-missing-status-denied" `
    -Expected "MemoryRecord::new rejects a problem with an empty predicate (status) with problem_missing_status" `
    -Passed $problemMissingStatusDenied `
    -Actual $(if ($problemMissingStatusDenied) { "matched" } else { ($selftestCases | ConvertTo-Json -Compress -Depth 10) })

Add-Predicate `
    -Name "memory-durable-supersede:selftest-nothing-appended" `
    -Expected "the 5 new M9A-3b selftest denials are RAM-only: reclog count/tail_seq unchanged across the call" `
    -Passed $selftestNothingAppended `
    -Actual $(if ($selftestNothingAppended) { "matched" } else { "before=$($scanBefore | ConvertTo-Json -Compress) after=$($scanAfter | ConvertTo-Json -Compress)" })

if (-not ($selftestShapeOk -and $secretConstructorDenied -and $unknownKindConstructorDenied -and $scopedSecretDenied -and $scopedKindDenied -and $selftestNothingAppended -and $scopedQuotaDenied -and $quotaBudgetHonest -and $liveQuotaCase -and $liveQuotaEvidence)) {
    throw "memory-durable secret/quota selftest family failed"
}

if (-not ($auditKindSupersedeDenied -and $supersedesListTooLongDenied -and $selfSupersedeDenied -and $decisionMissingSourceDenied -and $problemMissingStatusDenied)) {
    throw "memory-durable-supersede selftest family failed"
}

# --- guard needles: every OTHER memory mutation stays denied, provider export stays
#     fail-closed (main VM path) ------------------------------------------------

# Each guard parses the SPECIFIC method's response (Get-LastAgentResponseJson isolates
# the RAIOS_AGENT_BEGIN/END <method> window) and asserts the denial code WITHIN that
# response -- not a whole-log substring, which an earlier unrelated denial could satisfy.

# A denied mutation renders as { "t":"error", "body": { "method":<m>, "code":"capability_denied", ... } }.
function Assert-MemoryDurableMutationDenied {
    param(
        [string]$Command,
        [string]$Method,
        [string]$Suffix
    )
    Send-AgentCommand -Command $Command -ExpectedMarker "RAIOS_AGENT_END $Method" -Name "memory-durable-guard:$Suffix"
    $resp = Get-LastAgentResponseJson -Method $Method
    $denied = ($resp.body.code -eq "capability_denied" -and $resp.body.method -eq $Method)
    Add-Predicate `
        -Name "memory-durable-guard:${Suffix}_denied" `
        -Expected "$Method response is capability_denied (parsed from that response, not a whole-log match)" `
        -Passed $denied `
        -Actual $(if ($denied) { "matched" } else { ($resp | ConvertTo-Json -Compress -Depth 8) })
    return $denied
}

$obsDenied = Assert-MemoryDurableMutationDenied -Command "agent memory.record_observation" -Method "memory.record_observation" -Suffix "memory_record_observation"
# M9B-1b: the broad memory.record_observation boundary did NOT open just because the
# NARROW memory.observation_log_append method now exists -- re-asserted here, AFTER
# every memory-agent-observation-authorized/denied case above already exercised the
# new method against this same main VM.
$obsStillDeniedAfterAgentObservationMethod = Assert-MemoryDurableMutationDenied -Command "agent memory.record_observation" -Method "memory.record_observation" -Suffix "memory_record_observation_after_agent_observation_method_exists"
# L1: memory.redact must be proven capability_denied (was previously only a method echo).
$redactDenied = Assert-MemoryDurableMutationDenied -Command "agent memory.redact" -Method "memory.redact" -Suffix "memory_redact"
# L2: parse the provider.context_export response specifically, so an earlier mutation
# denial in the accumulated log cannot satisfy this check.
$exportDenied = Assert-MemoryDurableMutationDenied -Command "agent provider.context_export provider_minimal" -Method "provider.context_export" -Suffix "provider_context_export"

Send-AgentCommand -Command "agent memory.context provider_minimal" -ExpectedMarker "RAIOS_AGENT_END memory.context" -Name "memory-durable-guard:memory_context"
$contextJson = (Get-LastAgentResponseJson -Method "memory.context") | ConvertTo-Json -Depth 12 -Compress
$contextExportDisabled = ($contextJson -match '"provider_export":\s*"disabled"')
Add-Predicate `
    -Name "memory-durable-guard:memory_context_provider_export_disabled" `
    -Expected "memory.context reports provider_export disabled (parsed from that response only)" `
    -Passed $contextExportDisabled `
    -Actual $(if ($contextExportDisabled) { "matched" } else { $contextJson })

if (-not ($obsDenied -and $obsStillDeniedAfterAgentObservationMethod -and $redactDenied -and $exportDenied -and $contextExportDisabled)) {
    throw "memory-durable guard family failed"
}
