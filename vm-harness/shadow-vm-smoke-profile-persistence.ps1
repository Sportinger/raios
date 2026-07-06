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
        if (-not (Wait-ForLogText -Path $noPersistLog -Needle "SERIAL CONSOLE READY" -TimeoutSeconds $TimeoutSeconds)) {
            throw "No-persist child VM did not reach serial console: $(Get-SerialLogTail -Path $noPersistLog)"
        }
        Send-SerialText -Port $noPersistPort -Text "agent persist.layout`r" -TimeoutSeconds $TimeoutSeconds
        if (-not (Wait-ForLogText -Path $noPersistLog -Needle "RAIOS_AGENT_END persist.layout" -TimeoutSeconds $TimeoutSeconds)) {
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

if (-not ($kernelGptHeaderValid -and $kernelGptCrcChecked -and $kernelSeedDataFound -and $kernelSuperblockValid -and $readOnly -and $absentPassed)) {
    throw "Persistence kernel layout validation failed"
}
