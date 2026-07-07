if (-not $PersistDiskImage) {
    throw "memory-durable profile requires PersistDiskImage"
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
        [string]$AppendMethod = "memory.record_log_append"
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
        Send-SerialText -Port $fixturePort -Text "agent $AppendMethod`r" -TimeoutSeconds $childTimeout
        if (-not (Wait-ForLogText -Path $fixtureLog -Needle "RAIOS_AGENT_END $AppendMethod" -TimeoutSeconds $childTimeout)) {
            throw "Memory-record fixture child VM did not answer ${AppendMethod}: $(Get-SerialLogTail -Path $fixtureLog)"
        }
        $appendResult = (Get-ProfileAgentResponseJson -Path $fixtureLog -Method $AppendMethod).body.result

        # INSPECT: an independent follow-up durable.record_log_scan (same booted VM,
        # same disk state) must see the SAME tail_seq/count the append reported --
        # proving the frame is durably visible, not just self-reported by the writer.
        Send-SerialText -Port $fixturePort -Text "agent durable.record_log_scan`r" -TimeoutSeconds $childTimeout
        if (-not (Wait-ForLogText -Path $fixtureLog -Needle "RAIOS_AGENT_END durable.record_log_scan" -TimeoutSeconds $childTimeout)) {
            throw "Memory-record fixture child VM did not answer durable.record_log_scan: $(Get-SerialLogTail -Path $fixtureLog)"
        }
        $scanResult = (Get-ProfileAgentResponseJson -Path $fixtureLog -Method "durable.record_log_scan").body.result

        return [pscustomobject]@{
            append = $appendResult
            scan = $scanResult
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

if (-not ($obsDenied -and $redactDenied -and $exportDenied -and $contextExportDisabled)) {
    throw "memory-durable guard family failed"
}
