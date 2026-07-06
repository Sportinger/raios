if (-not $PersistDiskImage) {
    throw "Persistence profile requires PersistDiskImage"
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

function Invoke-NoPersistDiskLayoutProbe {
    $noPersistPort = $SerialTcpPort + 100
    $noPersistLog = Join-Path $RunDir "serial-no-persist.log"
    $noPersistErr = [System.IO.Path]::ChangeExtension($noPersistLog, ".err.txt")
    $noPersistImage = Join-Path $RunDir "raios-stage0-no-persist.img"
    $noPersistScratch = Join-Path $RunDir "raios-stage0-no-persist-scratch.img"
    $noPersistAuditRollback = Join-Path $RunDir "raios-stage0-no-persist-audit-rollback-target.img"
    $code = Join-Path $RunDir "edk2-code-no-persist.fd"
    $vars = Join-Path $RunDir "ovmf-vars-no-persist.fd"
    $qemu = "C:\Program Files\qemu\qemu-system-x86_64.exe"

    Copy-Item -LiteralPath $ResolvedImage -Destination $noPersistImage -Force
    Copy-Item -LiteralPath $ScratchImage -Destination $noPersistScratch -Force
    Copy-Item -LiteralPath $AuditRollbackTargetImage -Destination $noPersistAuditRollback -Force
    Copy-Item -LiteralPath "C:\Program Files\qemu\share\edk2-x86_64-code.fd" -Destination $code -Force
    Copy-Item -LiteralPath (Join-Path $RepoRoot "release\ovmf_vars.fd") -Destination $vars -Force
    Remove-Item -LiteralPath $noPersistLog, $noPersistErr -Force -ErrorAction SilentlyContinue

    $qemuArgs = @(
        "-machine", "q35",
        "-m", "512M",
        "-drive", "if=pflash,format=raw,readonly=on,file=$code",
        "-drive", "if=pflash,format=raw,file=$vars",
        "-drive", "file=$noPersistImage,format=raw,if=ide",
        "-drive", "file=$noPersistScratch,format=raw,if=none,id=raiosscratch_absent0",
        "-device", "ide-hd,drive=raiosscratch_absent0,bus=ide.1,unit=0",
        "-drive", "file=$noPersistAuditRollback,format=raw,if=none,id=raiosauditrollback_absent0",
        "-device", "ide-hd,drive=raiosauditrollback_absent0,bus=ide.2,unit=0",
        "-cpu", "max",
        "-device", "qemu-xhci,id=xhci_absent",
        "-device", "usb-kbd,bus=xhci_absent.0",
        "-device", "usb-tablet,bus=xhci_absent.0",
        "-chardev", "socket,id=seedserial_absent,host=127.0.0.1,port=$noPersistPort,server=on,wait=off,logfile=$noPersistLog,logappend=off",
        "-serial", "chardev:seedserial_absent",
        "-display", "none",
        "-no-reboot"
    )

    $child = Start-Process -FilePath $qemu -ArgumentList $qemuArgs -PassThru -RedirectStandardError $noPersistErr -WindowStyle Hidden
    try {
        # Child VMs boot a full kernel while the main VM is alive; under CPU
        # contention 45s is too short (observed timing out mid-framebuffer-init).
        # Give child VMs a generous boot/answer timeout (kernel proven correct).
        $childTimeout = [Math]::Max(180, $TimeoutSeconds * 4)
        if (-not (Wait-ForLogText -Path $noPersistLog -Needle "SERIAL CONSOLE READY" -TimeoutSeconds $childTimeout)) {
            throw "No-persist child VM did not reach serial console: $(Get-SerialLogTail -Path $noPersistLog)"
        }
        Send-SerialText -Port $noPersistPort -Text "agent persist.layout`r" -TimeoutSeconds $childTimeout
        if (-not (Wait-ForLogText -Path $noPersistLog -Needle "RAIOS_AGENT_END persist.layout" -TimeoutSeconds $childTimeout)) {
            throw "No-persist child VM did not answer persist.layout: $(Get-SerialLogTail -Path $noPersistLog)"
        }
        return (Get-ProfileAgentResponseJson -Path $noPersistLog -Method "persist.layout").body.result
    }
    finally {
        if ($child -and -not $child.HasExited) {
            Stop-Process -Id $child.Id -Force -ErrorAction SilentlyContinue
            try { $child.WaitForExit(5000) | Out-Null } catch {}
        }
    }
}

function Invoke-ReclogFixtureScanProbe {
    param(
        [string]$FixtureSpec,
        [string]$Label,
        [string]$Method = "durable.record_log_scan"
    )

    if (-not $script:ReclogFixtureProbeIndex) {
        $script:ReclogFixtureProbeIndex = 0
    }
    $script:ReclogFixtureProbeIndex += 1
    $suffix = "$Label-$($script:ReclogFixtureProbeIndex)"
    $fixturePort = $SerialTcpPort + 200 + $script:ReclogFixtureProbeIndex
    $fixtureLog = Join-Path $RunDir "serial-reclog-$suffix.log"
    $fixtureErr = [System.IO.Path]::ChangeExtension($fixtureLog, ".err.txt")
    $fixtureImage = Join-Path $RunDir "raios-stage0-reclog-$suffix.img"
    $fixtureScratch = Join-Path $RunDir "raios-stage0-reclog-$suffix-scratch.img"
    $fixtureAuditRollback = Join-Path $RunDir "raios-stage0-reclog-$suffix-audit-rollback-target.img"
    $fixturePersist = Join-Path $RunDir "raios-persist-reclog-$suffix.img"
    $code = Join-Path $RunDir "edk2-code-reclog-$suffix.fd"
    $vars = Join-Path $RunDir "ovmf-vars-reclog-$suffix.fd"
    $qemu = "C:\Program Files\qemu\qemu-system-x86_64.exe"

    $buildOutput = & python $builder --self-check --seed-reclog-fixture $FixtureSpec $fixturePersist 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "RECLOG fixture build failed ($FixtureSpec): $($buildOutput -join [Environment]::NewLine)"
    }

    Copy-Item -LiteralPath $ResolvedImage -Destination $fixtureImage -Force
    Copy-Item -LiteralPath $ScratchImage -Destination $fixtureScratch -Force
    Copy-Item -LiteralPath $AuditRollbackTargetImage -Destination $fixtureAuditRollback -Force
    Copy-Item -LiteralPath "C:\Program Files\qemu\share\edk2-x86_64-code.fd" -Destination $code -Force
    Copy-Item -LiteralPath (Join-Path $RepoRoot "release\ovmf_vars.fd") -Destination $vars -Force
    Remove-Item -LiteralPath $fixtureLog, $fixtureErr -Force -ErrorAction SilentlyContinue

    $driveId = "raiospersist_reclog_$($script:ReclogFixtureProbeIndex)"
    $qemuArgs = @(
        "-machine", "q35",
        "-m", "512M",
        "-drive", "if=pflash,format=raw,readonly=on,file=$code",
        "-drive", "if=pflash,format=raw,file=$vars",
        "-drive", "file=$fixtureImage,format=raw,if=ide",
        "-drive", "file=$fixtureScratch,format=raw,if=none,id=raiosscratch_reclog_$($script:ReclogFixtureProbeIndex)",
        "-device", "ide-hd,drive=raiosscratch_reclog_$($script:ReclogFixtureProbeIndex),bus=ide.1,unit=0",
        "-drive", "file=$fixtureAuditRollback,format=raw,if=none,id=raiosauditrollback_reclog_$($script:ReclogFixtureProbeIndex)",
        "-device", "ide-hd,drive=raiosauditrollback_reclog_$($script:ReclogFixtureProbeIndex),bus=ide.2,unit=0",
        "-drive", "file=$fixturePersist,format=raw,if=none,id=$driveId",
        "-device", "ide-hd,drive=$driveId,bus=ide.3,unit=0",
        "-cpu", "max",
        "-device", "qemu-xhci,id=xhci_reclog_$($script:ReclogFixtureProbeIndex)",
        "-device", "usb-kbd,bus=xhci_reclog_$($script:ReclogFixtureProbeIndex).0",
        "-device", "usb-tablet,bus=xhci_reclog_$($script:ReclogFixtureProbeIndex).0",
        "-chardev", "socket,id=seedserial_reclog_$($script:ReclogFixtureProbeIndex),host=127.0.0.1,port=$fixturePort,server=on,wait=off,logfile=$fixtureLog,logappend=off",
        "-serial", "chardev:seedserial_reclog_$($script:ReclogFixtureProbeIndex)",
        "-display", "none",
        "-no-reboot"
    )

    $child = Start-Process -FilePath $qemu -ArgumentList $qemuArgs -PassThru -RedirectStandardError $fixtureErr -WindowStyle Hidden
    try {
        $childTimeout = [Math]::Max(180, $TimeoutSeconds * 4)
        if (-not (Wait-ForLogText -Path $fixtureLog -Needle "SERIAL CONSOLE READY" -TimeoutSeconds $childTimeout)) {
            throw "RECLOG fixture child VM did not reach serial console: $(Get-SerialLogTail -Path $fixtureLog)"
        }
        Send-SerialText -Port $fixturePort -Text "agent $Method`r" -TimeoutSeconds $childTimeout
        if (-not (Wait-ForLogText -Path $fixtureLog -Needle "RAIOS_AGENT_END $Method" -TimeoutSeconds $childTimeout)) {
            throw "RECLOG fixture child VM did not answer ${Method}: $(Get-SerialLogTail -Path $fixtureLog)"
        }
        return (Get-ProfileAgentResponseJson -Path $fixtureLog -Method $Method).body.result
    }
    finally {
        if ($child -and -not $child.HasExited) {
            Stop-Process -Id $child.Id -Force -ErrorAction SilentlyContinue
            try { $child.WaitForExit(5000) | Out-Null } catch {}
        }
    }
}

function Invoke-BootControlFixtureProbe {
    param(
        [string]$FixtureSpec,
        [string]$Label
    )

    if (-not $script:BootControlFixtureProbeIndex) {
        $script:BootControlFixtureProbeIndex = 0
    }
    $script:BootControlFixtureProbeIndex += 1
    $suffix = "$Label-$($script:BootControlFixtureProbeIndex)"
    $fixturePort = $SerialTcpPort + 300 + $script:BootControlFixtureProbeIndex
    $fixtureLog = Join-Path $RunDir "serial-bootctl-$suffix.log"
    $fixtureErr = [System.IO.Path]::ChangeExtension($fixtureLog, ".err.txt")
    $fixtureImage = Join-Path $RunDir "raios-stage0-bootctl-$suffix.img"
    $fixtureScratch = Join-Path $RunDir "raios-stage0-bootctl-$suffix-scratch.img"
    $fixtureAuditRollback = Join-Path $RunDir "raios-stage0-bootctl-$suffix-audit-rollback-target.img"
    $fixturePersist = Join-Path $RunDir "raios-persist-bootctl-$suffix.img"
    $code = Join-Path $RunDir "edk2-code-bootctl-$suffix.fd"
    $vars = Join-Path $RunDir "ovmf-vars-bootctl-$suffix.fd"
    $qemu = "C:\Program Files\qemu\qemu-system-x86_64.exe"

    $buildOutput = & python $builder --self-check --seed-bootctl $FixtureSpec $fixturePersist 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "BOOTCTL fixture build failed ($FixtureSpec): $($buildOutput -join [Environment]::NewLine)"
    }

    Copy-Item -LiteralPath $ResolvedImage -Destination $fixtureImage -Force
    Copy-Item -LiteralPath $ScratchImage -Destination $fixtureScratch -Force
    Copy-Item -LiteralPath $AuditRollbackTargetImage -Destination $fixtureAuditRollback -Force
    Copy-Item -LiteralPath "C:\Program Files\qemu\share\edk2-x86_64-code.fd" -Destination $code -Force
    Copy-Item -LiteralPath (Join-Path $RepoRoot "release\ovmf_vars.fd") -Destination $vars -Force
    Remove-Item -LiteralPath $fixtureLog, $fixtureErr -Force -ErrorAction SilentlyContinue

    $driveId = "raiospersist_bootctl_$($script:BootControlFixtureProbeIndex)"
    $qemuArgs = @(
        "-machine", "q35",
        "-m", "512M",
        "-drive", "if=pflash,format=raw,readonly=on,file=$code",
        "-drive", "if=pflash,format=raw,file=$vars",
        "-drive", "file=$fixtureImage,format=raw,if=ide",
        "-drive", "file=$fixtureScratch,format=raw,if=none,id=raiosscratch_bootctl_$($script:BootControlFixtureProbeIndex)",
        "-device", "ide-hd,drive=raiosscratch_bootctl_$($script:BootControlFixtureProbeIndex),bus=ide.1,unit=0",
        "-drive", "file=$fixtureAuditRollback,format=raw,if=none,id=raiosauditrollback_bootctl_$($script:BootControlFixtureProbeIndex)",
        "-device", "ide-hd,drive=raiosauditrollback_bootctl_$($script:BootControlFixtureProbeIndex),bus=ide.2,unit=0",
        "-drive", "file=$fixturePersist,format=raw,if=none,id=$driveId",
        "-device", "ide-hd,drive=$driveId,bus=ide.3,unit=0",
        "-cpu", "max",
        "-device", "qemu-xhci,id=xhci_bootctl_$($script:BootControlFixtureProbeIndex)",
        "-device", "usb-kbd,bus=xhci_bootctl_$($script:BootControlFixtureProbeIndex).0",
        "-device", "usb-tablet,bus=xhci_bootctl_$($script:BootControlFixtureProbeIndex).0",
        "-chardev", "socket,id=seedserial_bootctl_$($script:BootControlFixtureProbeIndex),host=127.0.0.1,port=$fixturePort,server=on,wait=off,logfile=$fixtureLog,logappend=off",
        "-serial", "chardev:seedserial_bootctl_$($script:BootControlFixtureProbeIndex)",
        "-display", "none",
        "-no-reboot"
    )

    $child = Start-Process -FilePath $qemu -ArgumentList $qemuArgs -PassThru -RedirectStandardError $fixtureErr -WindowStyle Hidden
    try {
        $childTimeout = [Math]::Max(180, $TimeoutSeconds * 4)
        if (-not (Wait-ForLogText -Path $fixtureLog -Needle "SERIAL CONSOLE READY" -TimeoutSeconds $childTimeout)) {
            throw "BOOTCTL fixture child VM did not reach serial console: $(Get-SerialLogTail -Path $fixtureLog)"
        }
        Send-SerialText -Port $fixturePort -Text "agent boot.control_read`r" -TimeoutSeconds $childTimeout
        if (-not (Wait-ForLogText -Path $fixtureLog -Needle "RAIOS_AGENT_END boot.control_read" -TimeoutSeconds $childTimeout)) {
            throw "BOOTCTL fixture child VM did not answer boot.control_read: $(Get-SerialLogTail -Path $fixtureLog)"
        }
        return (Get-ProfileAgentResponseJson -Path $fixtureLog -Method "boot.control_read").body.result
    }
    finally {
        if ($child -and -not $child.HasExited) {
            Stop-Process -Id $child.Id -Force -ErrorAction SilentlyContinue
            try { $child.WaitForExit(5000) | Out-Null } catch {}
        }
    }
}

$builder = Join-Path $RepoRoot "scripts\make-gpt-persist-image.py"
$inspectJson = & python $builder --inspect-json $PersistDiskImage
if ($LASTEXITCODE -ne 0) {
    throw "Persist disk inspection failed with exit code $LASTEXITCODE"
}
$inspection = ($inspectJson -join [Environment]::NewLine) | ConvertFrom-Json

$gptHeaderValid = [bool]$inspection.gpt_header_valid
$gptCrcChecked = [bool]$inspection.gpt_crc_checked
$seedDataFound = [bool]$inspection.gpt_seed_data_found
$superblockValid = [bool]$inspection.data_superblock_valid

Add-Predicate `
    -Name "persistence:host_persist_image_layout_valid" `
    -Expected "host-built GPT persist image has valid GPT and SEED_DATA superblock" `
    -Passed ($gptHeaderValid -and $gptCrcChecked -and $seedDataFound -and $superblockValid) `
    -Actual $(if ($gptHeaderValid -and $gptCrcChecked -and $seedDataFound -and $superblockValid) { "matched" } else { ($inspection | ConvertTo-Json -Compress -Depth 6) })

if (-not ($gptHeaderValid -and $gptCrcChecked -and $seedDataFound -and $superblockValid)) {
    throw "Persistence host-side disk validation failed"
}

Send-AgentCommand -Command "agent persist.layout" -ExpectedMarker "RAIOS_AGENT_END persist.layout" -Name "persistence:persist_layout_query"
$layoutResponse = Get-LastAgentResponseJson -Method "persist.layout"
$layout = $layoutResponse.body.result
$gpt = $layout.gpt_layout
$data = $layout.data_layout
$kernelGptHeaderValid = [bool]$gpt.gpt_header_valid
$kernelGptCrcChecked = [bool]$gpt.gpt_crc_checked
$kernelSeedDataFound = [bool]$gpt.seed_data_found
$kernelSuperblockValid = [bool]$data.superblock_valid -and [bool]$data.superblock_copy_valid

Add-Predicate `
    -Name "gpt-header-valid" `
    -Expected 'kernel persist.layout gpt_layout.gpt_header_valid true for signature "EFI PART"' `
    -Passed $kernelGptHeaderValid `
    -Actual $(if ($kernelGptHeaderValid) { "matched" } else { ($gpt | ConvertTo-Json -Compress -Depth 6) })

Add-Predicate `
    -Name "gpt-crc-checked" `
    -Expected "kernel persist.layout gpt_layout.gpt_crc_checked true after header and entry-array CRC32 checks" `
    -Passed $kernelGptCrcChecked `
    -Actual $(if ($kernelGptCrcChecked) { "matched" } else { ($gpt | ConvertTo-Json -Compress -Depth 6) })

Add-Predicate `
    -Name "gpt-seed-data-found" `
    -Expected "kernel persist.layout gpt_layout.seed_data_found true with SEED_DATA partition" `
    -Passed $kernelSeedDataFound `
    -Actual $(if ($kernelSeedDataFound) { "matched" } else { ($gpt | ConvertTo-Json -Compress -Depth 6) })

Add-Predicate `
    -Name "data-superblock-valid" `
    -Expected "kernel persist.layout data_layout validates RAIOS_DATA_SB_V0 and matching LBA0/LBA1 copy" `
    -Passed $kernelSuperblockValid `
    -Actual $(if ($kernelSuperblockValid) { "matched" } else { ($data | ConvertTo-Json -Compress -Depth 6) })

$readOnly = (-not [bool]$layout.write_attempted) -and (-not [bool]$layout.write_dma_ext_called) -and (-not [bool]$layout.writes_enabled) -and (-not [bool]$layout.persistence_claimed)
Add-Predicate `
    -Name "persistence:kernel_layout_read_only_current_boot" `
    -Expected "persist.layout reports read-only current_boot evidence with no WRITE_DMA_EXT and no persistence claim" `
    -Passed $readOnly `
    -Actual $(if ($readOnly) { "matched" } else { ($layout | ConvertTo-Json -Compress -Depth 6) })

Send-AgentCommand -Command "agent durable.record_log_scan" -ExpectedMarker "RAIOS_AGENT_END durable.record_log_scan" -Name "persistence:durable_record_log_scan_empty_query"
$emptyScanResponse = Get-LastAgentResponseJson -Method "durable.record_log_scan"
$emptyScan = $emptyScanResponse.body.result
$emptyLogValid = (
    $emptyScan.schema -eq "raios.durable_record_log_scan.v0" -and
    $emptyScan.status -eq "valid_empty" -and
    [int64]$emptyScan.count -eq 0 -and
    [int64]$emptyScan.head_seq -eq 0 -and
    [int64]$emptyScan.tail_seq -eq 0 -and
    $emptyScan.terminated_reason -eq "zero_filled" -and
    -not [bool]$emptyScan.torn_tail -and
    [bool]$emptyScan.read_completed -and
    -not [bool]$emptyScan.write_attempted -and
    -not [bool]$emptyScan.write_dma_ext_called -and
    -not [bool]$emptyScan.writes_enabled
)
Add-Predicate `
    -Name "empty-log-valid" `
    -Expected "empty SEED_DATA RECLOG region scans as valid_empty current_boot evidence" `
    -Passed $emptyLogValid `
    -Actual $(if ($emptyLogValid) { "matched" } else { ($emptyScan | ConvertTo-Json -Compress -Depth 8) })

$appendDenied = (
    $emptyScan.append_result -eq "capability_denied" -and
    $emptyScan.durable_append -eq "capability_denied" -and
    -not [bool]$emptyScan.append_authority -and
    -not [bool]$emptyScan.writes_enabled -and
    -not [bool]$emptyScan.write_attempted
)
Add-Predicate `
    -Name "durable-append-capability-denied" `
    -Expected "durable append remains capability_denied while RECLOG scan is read-only" `
    -Passed $appendDenied `
    -Actual $(if ($appendDenied) { "matched" } else { ($emptyScan | ConvertTo-Json -Compress -Depth 8) })

$bootControl = Invoke-BootControlFixtureProbe -FixtureSpec "valid-a" -Label "valid-a"
$bootControlRead = (
    $bootControl.schema -eq "raios.boot_control_read.v0" -and
    $bootControl.posture -eq "Normal" -and
    $bootControl.selected_storage_slot -eq "A" -and
    $bootControl.selected_payload_slot -eq "A" -and
    $bootControl.authoritative_bootctl_slot -eq "A" -and
    -not [bool]$bootControl.pending_consumed -and
    [bool]$bootControl.read_only -and
    -not [bool]$bootControl.write_attempted -and
    -not [bool]$bootControl.write_dma_ext_called -and
    -not [bool]$bootControl.writes_enabled -and
    -not [bool]$bootControl.persistence_claimed
)
Add-Predicate `
    -Name "boot-control-read" `
    -Expected "valid-a BOOTCTL fixture reads as Normal with authoritative storage slot A and read-only evidence" `
    -Passed $bootControlRead `
    -Actual $(if ($bootControlRead) { "matched" } else { ($bootControl | ConvertTo-Json -Compress -Depth 8) })

$invalidBootControl = Invoke-BootControlFixtureProbe -FixtureSpec "both-invalid" -Label "both-invalid"
$safePostureBothSlotsInvalid = (
    $invalidBootControl.schema -eq "raios.boot_control_read.v0" -and
    $invalidBootControl.posture -eq "Safe" -and
    $invalidBootControl.decision_reason -eq "both_slots_invalid" -and
    $null -eq $invalidBootControl.selected_storage_slot -and
    $null -eq $invalidBootControl.selected_payload_slot -and
    $null -eq $invalidBootControl.authoritative_bootctl_slot -and
    -not [bool]$invalidBootControl.pending_consumed -and
    -not [bool]$invalidBootControl.would_mark_good
)
Add-Predicate `
    -Name "safe-posture-both-slots-invalid" `
    -Expected "corrupted BOOTCTL storage slots enter Safe posture without selecting a payload slot" `
    -Passed $safePostureBothSlotsInvalid `
    -Actual $(if ($safePostureBothSlotsInvalid) { "matched" } else { ($invalidBootControl | ConvertTo-Json -Compress -Depth 8) })

$safePendingBootControl = Invoke-BootControlFixtureProbe -FixtureSpec "pending-safe" -Label "pending-safe"
$pendingNotConsumedInSafe = (
    $safePendingBootControl.schema -eq "raios.boot_control_read.v0" -and
    $safePendingBootControl.posture -eq "Safe" -and
    $safePendingBootControl.safe_mode -eq $true -and
    $safePendingBootControl.pending -eq "B" -and
    -not [bool]$safePendingBootControl.pending_consumed -and
    -not [bool]$safePendingBootControl.would_mark_good
)
Add-Predicate `
    -Name "pending-not-consumed-in-safe" `
    -Expected "safe_mode BOOTCTL fixture keeps pending=B unconsumed and marks nothing good" `
    -Passed $pendingNotConsumedInSafe `
    -Actual $(if ($pendingNotConsumedInSafe) { "matched" } else { ($safePendingBootControl | ConvertTo-Json -Compress -Depth 8) })

$append = Invoke-ReclogFixtureScanProbe -FixtureSpec "valid:2" -Label "append" -Method "durable.record_log_append"
$appendAuthorized = (
    $append.schema -eq "raios.durable_record_log_append.v0" -and
    $append.durable_append -eq "appended" -and
    [bool]$append.performed -and
    $append.authority -eq "scoped_seed_data_append_authorized" -and
    [int64]$append.seq -eq 3 -and
    $append.record_schema -eq "raios.durable_record.v0" -and
    $append.region_marker -eq "RAIOS_DATA_RECLOG"
)
Add-Predicate `
    -Name "durable-append-authorized" `
    -Expected "durable.record_log_append appends one scoped SEED_DATA RECLOG frame after valid:2" `
    -Passed $appendAuthorized `
    -Actual $(if ($appendAuthorized) { "matched" } else { ($append | ConvertTo-Json -Compress -Depth 8) })

$appendReadbackHash = (
    $append.readback_sha256 -eq $append.frame_sha256 -and
    [bool]$append.reparse_valid
)
Add-Predicate `
    -Name "durable-readback-hash" `
    -Expected "append readback_sha256 matches frame_sha256 and persisted frame reparses" `
    -Passed $appendReadbackHash `
    -Actual $(if ($appendReadbackHash) { "matched" } else { ($append | ConvertTo-Json -Compress -Depth 8) })

$appendChainHead = (
    [int64]$append.tail_seq_after -eq ([int64]$append.tail_seq_before + 1) -and
    [int64]$append.count_after -eq ([int64]$append.count_before + 1)
)
Add-Predicate `
    -Name "durable-chain-head" `
    -Expected "append rescan advances tail_seq and count by exactly one" `
    -Passed $appendChainHead `
    -Actual $(if ($appendChainHead) { "matched" } else { ($append | ConvertTo-Json -Compress -Depth 8) })

$fullAppend = Invoke-ReclogFixtureScanProbe -FixtureSpec "full" -Label "full-append" -Method "durable.record_log_append"
$fullDenied = (
    $fullAppend.durable_append -eq "capability_denied" -and
    -not [bool]$fullAppend.performed -and
    $fullAppend.reason -eq "durable_store_full"
)
Add-Predicate `
    -Name "durable-store-full-denied" `
    -Expected "full RECLOG fixture denies append without rotation" `
    -Passed $fullDenied `
    -Actual $(if ($fullDenied) { "matched" } else { ($fullAppend | ConvertTo-Json -Compress -Depth 8) })

Send-AgentCommand -Command "agent module.audit_rollback_write_policy_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_write_policy_selftest" -Name "generic-target-still-denied:write_policy_selftest"
Assert-LogContains -Name "generic-target-still-denied:write_policy_status" -Needle '"actual_status": "denied_write_path_unimplemented"' -TimeoutSeconds 1
Assert-LogContains -Name "generic-target-still-denied:write_policy_reason" -Needle '"actual_reason": "durable_audit_rollback_writer_unimplemented"' -TimeoutSeconds 1

Send-AgentCommand -Command "agent module.audit_rollback_write_boundary_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_write_boundary_selftest" -Name "generic-target-still-denied:write_boundary_selftest"
Assert-LogContains -Name "generic-target-still-denied:write_boundary_status" -Needle '"actual_status": "denied_write_path_unimplemented"' -TimeoutSeconds 1
Assert-LogContains -Name "generic-target-still-denied:write_boundary_reason" -Needle '"actual_reason": "durable_audit_rollback_writer_unimplemented"' -TimeoutSeconds 1

$chainScan = Invoke-ReclogFixtureScanProbe -FixtureSpec "valid:3" -Label "chain"
$chainHeadOk = (
    $chainScan.status -eq "valid" -and
    [int64]$chainScan.head_seq -eq 1 -and
    $null -ne $chainScan.head_frame_sha256 -and
    [bool]$chainScan.full_region_valid -and
    -not [bool]$chainScan.torn_tail
)
Add-Predicate `
    -Name "durable-scan-chain-head" `
    -Expected "kernel scan reports the head of a valid seeded RECLOG hash chain" `
    -Passed $chainHeadOk `
    -Actual $(if ($chainHeadOk) { "matched" } else { ($chainScan | ConvertTo-Json -Compress -Depth 8) })

$chainCountOk = (
    [int64]$chainScan.count -eq 3 -and
    [int64]$chainScan.tail_seq -eq 3 -and
    $null -ne $chainScan.tail_frame_sha256 -and
    $chainScan.terminated_reason -eq "zero_filled"
)
Add-Predicate `
    -Name "durable-chain-count" `
    -Expected "kernel scan reports count=3 and tail_seq=3 for a valid seeded RECLOG chain" `
    -Passed $chainCountOk `
    -Actual $(if ($chainCountOk) { "matched" } else { ($chainScan | ConvertTo-Json -Compress -Depth 8) })

$tornScan = Invoke-ReclogFixtureScanProbe -FixtureSpec "valid:2,torn" -Label "torn"
$tornOk = (
    $tornScan.status -eq "torn_tail" -and
    [int64]$tornScan.count -eq 2 -and
    [int64]$tornScan.head_seq -eq 1 -and
    [int64]$tornScan.tail_seq -eq 2 -and
    [bool]$tornScan.torn_tail -and
    -not [bool]$tornScan.full_region_valid -and
    $tornScan.terminated_reason -eq "bad_magic"
)
Add-Predicate `
    -Name "torn-tail-detected" `
    -Expected "kernel scan stops at first invalid RECLOG frame and reports torn-tail evidence" `
    -Passed $tornOk `
    -Actual $(if ($tornOk) { "matched" } else { ($tornScan | ConvertTo-Json -Compress -Depth 8) })

$absentActual = ""
$absentPassed = $false
try {
    $absentLayout = Invoke-NoPersistDiskLayoutProbe
    $absentPassed = (
        $absentLayout.status -eq "absent" -and
        $absentLayout.gpt_layout.status -eq "absent" -and
        $absentLayout.data_layout.status -eq "absent" -and
        -not [bool]$absentLayout.write_attempted -and
        -not [bool]$absentLayout.write_dma_ext_called -and
        -not [bool]$absentLayout.writes_enabled -and
        -not [bool]$absentLayout.persistence_claimed
    )
    $absentActual = if ($absentPassed) { "matched" } else { ($absentLayout | ConvertTo-Json -Compress -Depth 6) }
}
catch {
    $absentActual = $_.Exception.Message
}
Add-Predicate `
    -Name "gpt-absent-fail-closed" `
    -Expected "no persist disk: persist.layout reports absent and keeps persistence/write authority false" `
    -Passed $absentPassed `
    -Actual $absentActual

if (-not ($kernelGptHeaderValid -and $kernelGptCrcChecked -and $kernelSeedDataFound -and $kernelSuperblockValid -and $readOnly -and $emptyLogValid -and $appendDenied -and $bootControlRead -and $safePostureBothSlotsInvalid -and $pendingNotConsumedInSafe -and $appendAuthorized -and $appendReadbackHash -and $appendChainHead -and $fullDenied -and $chainHeadOk -and $chainCountOk -and $tornOk -and $absentPassed)) {
    throw "Persistence kernel layout validation failed"
}
