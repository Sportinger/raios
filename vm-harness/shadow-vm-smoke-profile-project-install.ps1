if (-not $StructuredStoreDiskImage -or -not $PersistDiskImage -or -not $QmpTcpPort -or -not $MonitorTcpPort) {
    throw 'project-install requires C1 source store, SEED_DATA persist disk, QMP pointer and monitor shutdown'
}

function Add-InstallPredicate {
    param([string]$Name, [string]$Expected, [bool]$Passed, [object]$Actual)
    $actualText = if ($Actual -is [string]) { $Actual } else { $Actual | ConvertTo-Json -Compress -Depth 12 }
    Add-Predicate -Name "project-install:$Name" -Expected $Expected -Passed $Passed -Actual $actualText
    if (-not $Passed) { throw "Project-install predicate failed: $Name" }
}

function Send-InstallCommand {
    param([string]$Command, [string]$Method, [string]$Name)
    Send-AgentCommand -Command $Command -ExpectedMarker "RAIOS_AGENT_END $Method" -Name "project-install:$Name"
    return (Get-LastAgentResponseJson -Method $Method).body.result
}

function Assert-InstallReadPosture {
    param([object]$Result, [string]$Name)
    Add-InstallPredicate "$Name`:posture" 'serial response never substitutes for the second Genesis pointer approval' (
        $Result.classification -eq 'local_only' -and
        $Result.persistence_posture -eq 'write_only_after_genesis_pointer' -and
        $Result.serial_approval_authorized -eq $false -and
        $Result.physical_pointer_required -eq $true -and
        $Result.writes_persistent_state -eq $false
    ) $Result
}

function Invoke-SignedPhysicalProjectAction {
    param(
        [ValidateSet('install', 'uninstall')][string]$Kind,
        [string]$BindingSha256 = '',
        [string]$ExpectedCandidateSha256 = '',
        [string]$Name
    )
    $prepareCommand = if ($Kind -eq 'install') {
        "project.install_prepare $BindingSha256"
    } else {
        'project.uninstall_prepare svc.workspace.current_boot'
    }
    $prepareMethod = "project.$Kind`_prepare"
    $signatureMethod = "project.$Kind`_signature"
    $approveMethod = "project.$Kind`_approve"
    $prepare = Send-InstallCommand $prepareCommand $prepareMethod "$Name`:prepare"
    Add-InstallPredicate "$Name`:prepare" 'exact action preview requires the external owner signature before any write' (
        $prepare.status -eq 'accepted' -and
        $prepare.action_kind -eq $Kind -and
        $prepare.phase -eq 'pending_owner_signature' -and
        $prepare.signature_verified -eq $false -and
        $prepare.action_signature_message_sha256 -match '^sha256:[0-9a-f]{64}$' -and
        $prepare.physical_approval_sha256 -match '^sha256:[0-9a-f]{64}$' -and
        $prepare.trust_tier -eq 'dev_key_not_owner_sealed'
    ) $prepare
    Assert-InstallReadPosture $prepare "$Name`:prepare"
    if ($ExpectedCandidateSha256) {
        Add-InstallPredicate "$Name`:candidate_binding" 'install preview binds the exact healthy W5 candidate' (
            $prepare.candidate_sha256 -eq "sha256:$ExpectedCandidateSha256"
        ) $prepare
    }

    $messageHash = ([string]$prepare.action_signature_message_sha256).Substring(7)
    $signatureHex = New-ReliableDevPromotionSignatureHex -AttestationReferenceHash $messageHash
    $signed = Send-InstallCommand "project.$Kind`_signature $signatureHex" $signatureMethod "$Name`:signature"
    Add-InstallPredicate "$Name`:signature" 'host dev-key signature binds the exact action while physical authority remains pending' (
        $signed.status -eq 'accepted' -and $signed.action_kind -eq $Kind -and
        $signed.phase -eq 'pending_physical_pointer_approval' -and
        $signed.signature_verified -eq $true -and
        $signed.action_signature_message_sha256 -eq $prepare.action_signature_message_sha256 -and
        $signed.physical_approval_sha256 -eq $prepare.physical_approval_sha256
    ) $signed
    Assert-InstallReadPosture $signed "$Name`:signature"

    $serial = Send-InstallCommand "project.$Kind`_approve" $approveMethod "$Name`:serial_denied"
    Add-InstallPredicate "$Name`:serial_denied" 'serial approval is denied and leaves the signed physical preview armed' (
        $serial.status -eq 'denied' -and $serial.phase -eq 'pending_physical_pointer_approval' -and
        $serial.signature_verified -eq $true -and $serial.serial_approval_authorized -eq $false
    ) $serial
    Assert-InstallReadPosture $serial "$Name`:serial_denied"

    $captureName = $Name -replace '_', '-'
    Save-QemuScreendump -Name "$captureName-signed-physical-preview" | Out-Null
    Send-QemuAbsolutePointerClick -X 27017 -Y 6559
    Assert-LogContains -Name "project-install:$Name`:physical_commit" `
        -Needle "PROJECT_INSTALL_COMMIT kind=$Kind physical_approval=genesis_pointer result=accepted" `
        -TimeoutSeconds $TimeoutSeconds
    $status = Send-InstallCommand 'project.install_status' 'project.install_status' "$Name`:status"
    Add-InstallPredicate "$Name`:persisted" 'one Genesis pointer crosses the signed preview into the narrow durable action' (
        $status.status -eq 'accepted' -and $status.phase -eq 'idle' -and
        $status.last_commit_sha256 -match '^sha256:[0-9a-f]{64}$'
    ) $status
    Assert-InstallReadPosture $status "$Name`:status"
    return [pscustomobject]@{ prepare = $prepare; signed = $signed; status = $status }
}

function Stop-ProjectInstallBoot {
    if (-not $QemuPid) { throw 'project-install cannot stop a boot without a QEMU pid' }
    Close-SerialTcpConnection
    $pidToStop = $QemuPid
    Send-QemuMonitorCommand -Command 'quit' | Out-Null
    $process = Get-Process -Id $pidToStop -ErrorAction SilentlyContinue
    if ($process) { $process.WaitForExit(5000) | Out-Null }
    if (Get-Process -Id $pidToStop -ErrorAction SilentlyContinue) {
        throw 'project-install QEMU did not complete clean monitor shutdown'
    }
    $script:QemuPid = $null
    $script:QemuProcess = $null
}

function Start-ProjectInstallBoot {
    param([string]$Name)
    $log = Join-Path $RunDir "serial-project-install-$Name.log"
    $params = $runParams.Clone()
    $params.StopExisting = $false
    $params.SerialLog = $log
    $output = & $RunScript @params
    $script:SerialLogCachePath = $null
    $script:SerialLogCacheLength = [int64]-1
    $script:SerialLogCacheWriteTicks = [int64]-1
    $script:SerialLogCacheContent = $null
    $script:SerialTcpClient = $null
    $script:SerialTcpDrainStream = $null
    $script:SerialTcpStream = $null
    $script:QemuProcess = $null
    $script:QemuProcessBeforeTeardown = $null
    $script:QemuProcessAfterTeardown = $null
    $script:QemuTeardownAction = 'running'
    $script:SerialLog = $log
    $script:QemuPid = $null
    foreach ($line in $output) {
        if ($line -match '^qemu pid:\s*(\d+)') { $script:QemuPid = [int]$Matches[1] }
    }
    if (-not $QemuPid) { throw "project-install $Name boot did not return a QEMU pid" }
    try { $script:QemuProcess = Get-Process -Id $QemuPid -ErrorAction Stop } catch { $script:QemuProcess = $null }
    Assert-LogContains -Name "project-install:$Name`:serial_ready" -Needle 'SERIAL CONSOLE READY' -TimeoutSeconds $TimeoutSeconds
    return $log
}

function Get-ChildProjectRevisionSha256 {
    param([string]$ProjectId, [string]$TreeSha256, [string]$ParentRevisionSha256)
    $stream = [IO.MemoryStream]::new()
    $writer = [IO.BinaryWriter]::new($stream, [Text.Encoding]::UTF8, $true)
    try {
        $writer.Write([Text.Encoding]::ASCII.GetBytes('RAIOSREV1'))
        $writer.Write([byte[]](Convert-HexToBytes $ProjectId))
        $writer.Write([byte]1)
        $writer.Write([byte[]](Convert-HexToBytes $ParentRevisionSha256))
        $writer.Write([byte[]](Convert-HexToBytes $TreeSha256))
        Write-CanonicalString $writer 'agent_overlay_commit'
        Write-CanonicalString $writer 'none_deterministic'
        $writer.Flush()
        return Get-ByteSha256Hex $stream.ToArray()
    }
    finally { $writer.Dispose(); $stream.Dispose() }
}

function Send-ProjectInstallEditFile {
    param([string]$Path, [string]$Classification, [string]$MediaType, [byte[]]$Bytes, [string]$Name)
    $path64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($Path))
    $sha = Get-ByteSha256Hex $Bytes
    $begin = Send-BuildCommand "project.edit_file_begin $path64 $Classification $MediaType $($Bytes.Length) sha256:$sha" 'project.edit_file_begin' "$Name`:begin"
    Add-InstallPredicate "$Name`:begin" 'v2 source replacement starts in a RAM overlay' ($begin.status -eq 'accepted') $begin
    $base64 = [Convert]::ToBase64String($Bytes)
    for ($offset = 0; $offset -lt $base64.Length; $offset += 256) {
        $chunk = $base64.Substring($offset, [Math]::Min(256, $base64.Length - $offset))
        $part = Send-BuildCommand "project.edit_chunk $chunk" 'project.edit_chunk' "$Name`:chunk_$offset"
        Add-InstallPredicate "$Name`:chunk_$offset" 'v2 bytes remain RAM-only before edit commit' ($part.status -eq 'accepted') $part
    }
    $final = Send-BuildCommand 'project.edit_file_finalize' 'project.edit_file_finalize' "$Name`:finalize"
    Add-InstallPredicate "$Name`:finalize" 'v2 replacement hash finalized' ($final.status -eq 'accepted') $final
}

function Assert-SourceRevision {
    param([string]$ProjectId, [string]$RevisionSha256, [string]$Name)
    $inspect = Send-BuildCommand "project.inspect $ProjectId" 'project.inspect' $Name
    Add-InstallPredicate $Name 'install lifecycle never mutates the latest immutable source workspace revision' (
        $inspect.status -eq 'present' -and $inspect.revision_sha256 -eq "sha256:$RevisionSha256"
    ) $inspect
    return ($inspect | ConvertTo-Json -Compress -Depth 12)
}

# Boot 1 reuses the complete W1-W5 source -> reproducible build -> physical run
# path and returns here before W5's intentionally destructive F12 cleanup.
$bootLogs = New-Object System.Collections.Generic.List[string]
$bootLogs.Add($SerialLog)
. (Join-Path $PSScriptRoot 'shadow-vm-smoke-profile-project-build.ps1')
$sourceV1Json = Assert-SourceRevision $projectId $projectRevisionSha256 'source_v1_before_install'
$v1Install = Invoke-SignedPhysicalProjectAction -Kind install -BindingSha256 $challengeSha256 `
    -ExpectedCandidateSha256 $candidateSha256 -Name 'boot1_v1_install'
$v1InstallCommit = [string]$v1Install.status.last_commit_sha256
Add-InstallPredicate 'boot1:v1_blob_location' 'v1 install exposes its exact content-addressed candidate frame location' (
    $v1Install.status.candidate_blob_offset -ne $null -and $v1Install.status.candidate_blob_frame_len -ne $null
) $v1Install.status

Stop-ProjectInstallBoot
$bootLogs.Add((Start-ProjectInstallBoot 'boot2'))
Assert-LogContains -Name 'project-install:boot2_v1_activation_success' `
    -Needle 'PROJECT_APP_AUTOLOAD result=accepted phase=running reason=project_app_activation_success_persisted' `
    -TimeoutSeconds $TimeoutSeconds
$sourceV1Boot2Json = Assert-SourceRevision $projectId $projectRevisionSha256 'source_v1_boot2'
Add-InstallPredicate 'boot2:v1_source_identical' 'v1 source facts remain byte-identical after install and autoload' (
    $sourceV1Boot2Json -ceq $sourceV1Json
) $sourceV1Boot2Json

# Free only the current-boot runtime slot. The durable v1 install remains the
# last-good target while a genuinely different, still healthy v2 is built.
$dropV1 = Send-WorkspaceCommand 'service.drop svc.workspace.current_boot' 'service.drop' 'install_v2_drop_v1_runtime'
Add-InstallPredicate 'boot2:drop_runtime_only' 'dropping the W5 runtime does not write an uninstall tombstone' (
    $dropV1.status -eq 'accepted' -and $dropV1.phase -eq 'missing'
) $dropV1

$v2Files = @($projectFiles | ForEach-Object {
    [pscustomobject]@{
        path = $_.path; classification = $_.classification; media_type = $_.media_type
        bytes = [byte[]]$_.bytes.Clone()
    }
})
$v2Files[2].bytes = [Text.Encoding]::UTF8.GetBytes("#![no_std]`nuse core::panic::PanicInfo;`n#[no_mangle]`npub extern `"C`" fn raios_service_main() -> i32 { raios_local_helper::answer() }`n#[no_mangle]`npub extern `"C`" fn raios_workspace_v2_marker() -> i32 { 2 }`n#[panic_handler]`nfn panic(_: &PanicInfo) -> ! { core::arch::wasm32::unreachable() }`n")
Add-HashesAndChunks $v2Files
$v2TreeSha256 = Get-ProjectTreeSha256 $v2Files
$v2RevisionSha256 = Get-ChildProjectRevisionSha256 $projectId $v2TreeSha256 $projectRevisionSha256
$editBegin = Send-BuildCommand "project.edit_begin $projectId" 'project.edit_begin' 'v2:edit_begin'
Add-InstallPredicate 'v2:edit_parent' 'v2 overlay binds exact v1 parent' (
    $editBegin.status -eq 'accepted' -and $editBegin.base_revision_sha256 -eq "sha256:$projectRevisionSha256"
) $editBegin
Send-ProjectInstallEditFile 'src/lib.rs' 'local_only' 'text/rust' $v2Files[2].bytes 'v2:lib'
$editCommit = Send-BuildCommand 'project.edit_commit' 'project.edit_commit' 'v2:edit_commit'
Add-InstallPredicate 'v2:real_child_revision' 'durable v2 source is a distinct immutable child of v1' (
    $editCommit.status -eq 'accepted' -and $editCommit.revision_sha256 -eq "sha256:$v2RevisionSha256" -and
    $v2RevisionSha256 -ne $projectRevisionSha256
) $editCommit

$v2DependencyBundleSha256 = Get-DependencyBundleSha256 `
    -ProjectId $projectId -RevisionSha256 $v2RevisionSha256 -CargoLockSha256 $cargoLockSha256 `
    -Name 'raios-local-helper' -Version '0.1.0' -Origin 'owner-local://serial/raios-local-helper/0.1.0' `
    -LicenseExpression 'MIT' -LicensePath 'LICENSE' -LicenseSha256 $dependencyFiles[1].sha256 `
    -TreeSha256 $dependencyTreeSha256
Import-Dependency $projectId $v2RevisionSha256 $cargoLockSha256 $dependencyFiles $dependencyTreeSha256 `
    $v2DependencyBundleSha256 $dependencyFiles[1].sha256 'v2_dependency'
$v2MaterializedTreeSha256 = Get-MaterializedTreeSha256 $v2Files $dependencyFiles
$v2InputManifestSha256 = Get-BuildSnapshotSha256 $projectId $v2RevisionSha256 $v2TreeSha256 `
    $cargoLockSha256 $v2Files $v2DependencyBundleSha256 $dependencyTreeSha256 $dependencyFiles

$v2Input = Join-Path $RunDir 'project-install-v2-input'
$v2Vendor = Join-Path $v2Input 'vendor\raios-local-helper'
foreach ($file in $v2Files) {
    Read-BuildBytes source $projectId $v2RevisionSha256 '' $file (Join-Path $v2Input $file.path) "v2_source_$($file.path)"
}
foreach ($file in $dependencyFiles) {
    Read-BuildBytes dependency $projectId $v2RevisionSha256 $v2DependencyBundleSha256 $file (Join-Path $v2Vendor $file.path) "v2_dependency_$($file.path)"
}
$v2Output = Join-Path $RunDir 'project-install-v2-output'
& powershell -NoProfile -ExecutionPolicy Bypass -File $builderScript `
    -InputDirectory $v2Input -OutputDirectory $v2Output `
    -ProjectId $projectId -ProjectRevisionSha256 $v2RevisionSha256 `
    -ExpectedSourceTreeSha256 $v2TreeSha256 -ExpectedCargoLockSha256 $cargoLockSha256 `
    -ExpectedMaterializedTreeSha256 $v2MaterializedTreeSha256 `
    -ExpectedPackageName $rootPackageName -ExpectedPackageVersion $rootPackageVersion `
    -VendorDirectory $v2Vendor -ExpectedDependencyName 'raios-local-helper' `
    -ExpectedDependencyVersion '0.1.0' -ExpectedDependencyBundleSha256 $v2DependencyBundleSha256 `
    -TimeoutSeconds $TimeoutSeconds
if ($LASTEXITCODE -ne 0) { throw "project-install v2 workstation build failed: $LASTEXITCODE" }
$v2BuildResult = Get-Content -LiteralPath (Join-Path $v2Output 'build-result.json') -Raw | ConvertFrom-Json
$v2CandidatePath = Join-Path $v2Output 'candidate.wasm'
$v2CandidateSha256 = (Get-FileHash -LiteralPath $v2CandidatePath -Algorithm SHA256).Hash.ToLowerInvariant()
$v2CandidateByteLen = (Get-Item -LiteralPath $v2CandidatePath).Length
Add-InstallPredicate 'v2:genuine_distinct_candidate' 'real source child produces a different healthy Wasm artifact' (
    $v2BuildResult.status -eq 'accepted' -and $v2BuildResult.reproducible -eq $true -and
    $v2CandidateSha256 -ne $candidateSha256
) $v2BuildResult
$v2Delivery = Send-CandidateBytes -Path $v2CandidatePath
Add-InstallPredicate 'v2:candidate_retained' 'exact v2 Wasm is validated and retained for the real W5 path' (
    $v2Delivery.finalize_response.body.result.artifact_sha256 -eq "sha256:$v2CandidateSha256" -and
    $v2Delivery.finalize_response.body.result.wasm_valid -eq $true
) $v2Delivery.finalize_response.body.result

$v2Begin = Start-BuildSession $projectId $v2RevisionSha256 $pinnedToolchainSha256 'v2'
Add-InstallPredicate 'v2:build_session' 'guest build session binds the exact v2 source/dependency snapshot' (
    $v2Begin.status -eq 'accepted' -and $v2Begin.input_manifest_sha256 -eq "sha256:$v2InputManifestSha256"
) $v2Begin
$v2RunOneFact = $v2BuildResult.builds[0]
$v2RunTwoFact = $v2BuildResult.builds[1]
$v2RunOne = [pscustomobject]@{ exit_code=$v2RunOneFact.run.exit_code; stdout_sha256=$v2RunOneFact.run.stdout_sha256; stderr_sha256=$v2RunOneFact.run.stderr_sha256; output_byte_len=$v2RunOneFact.wasm_byte_len; output_sha256=$v2RunOneFact.wasm_sha256 }
$v2RunTwo = [pscustomobject]@{ exit_code=$v2RunTwoFact.run.exit_code; stdout_sha256=$v2RunTwoFact.run.stdout_sha256; stderr_sha256=$v2RunTwoFact.run.stderr_sha256; output_byte_len=$v2RunTwoFact.wasm_byte_len; output_sha256=$v2RunTwoFact.wasm_sha256 }
Send-BuildRun 1 $v2RunOne.exit_code $v2RunOne.stdout_sha256 $v2RunOne.stderr_sha256 $v2RunOne.output_byte_len $v2RunOne.output_sha256 'v2' | Out-Null
Send-BuildRun 2 $v2RunTwo.exit_code $v2RunTwo.stdout_sha256 $v2RunTwo.stderr_sha256 $v2RunTwo.output_byte_len $v2RunTwo.output_sha256 'v2' | Out-Null
$v2ReceiptSha256 = Get-BuildReceiptSha256 `
    -InputManifestSha256 $v2InputManifestSha256 -ToolchainSha256 $pinnedToolchainSha256 `
    -FlagsSha256 $flagsContractSha256 -EnvironmentSha256 $environmentContractSha256 `
    -RunOne $v2RunOne -RunTwo $v2RunTwo -CandidateSha256 $v2CandidateSha256 -CandidateByteLen $v2CandidateByteLen `
    -ProjectId $projectId -RevisionSha256 $v2RevisionSha256 -SourceTreeSha256 $v2TreeSha256 `
    -CargoLockSha256 $cargoLockSha256 -SourceFiles $v2Files `
    -BundleSha256 $v2DependencyBundleSha256 -DependencyTreeSha256 $dependencyTreeSha256 -DependencyFiles $dependencyFiles
$v2Commit = Send-BuildCommand 'project.build_commit' 'project.build_commit' 'v2:build_commit'
Add-InstallPredicate 'v2:receipt' 'guest seals exact real v2 reproducible receipt' (
    $v2Commit.status -eq 'accepted' -and $v2Commit.receipt_sha256 -eq "sha256:$v2ReceiptSha256"
) $v2Commit

$v2Prepared = Send-WorkspaceCommand "project.run_prepare $projectId sha256:$v2RevisionSha256 sha256:$v2ReceiptSha256 sha256:$v2CandidateSha256" 'project.run_prepare' 'v2_prepare'
Add-InstallPredicate 'v2:w5_prepare' 'real W5 binding accepts exact v2 receipt and candidate' (
    $v2Prepared.status -eq 'accepted' -and $v2Prepared.phase -eq 'pending_physical_approval'
) $v2Prepared
Send-QemuAbsolutePointerClick -X 27017 -Y 6559
Assert-LogContains -Name 'project-install:v2_w5_healthy' `
    -Needle "WORKSPACE_CURRENT_BOOT_ACTIVATION physical_approval=pointer result=accepted candidate_sha256=sha256:$v2CandidateSha256" `
    -TimeoutSeconds $TimeoutSeconds
$v2Running = Send-WorkspaceCommand 'project.run_status' 'project.run_status' 'v2_running'
Add-InstallPredicate 'v2:w5_result_42' 'genuinely distinct v2 completes the real runtime call with result 42' (
    $v2Running.phase -eq 'running' -and $v2Running.run_outcome -eq 'success' -and
    [int]$v2Running.return_value_i32 -eq 42
) $v2Running
$v2Install = Invoke-SignedPhysicalProjectAction -Kind install `
    -BindingSha256 ([string]$v2Prepared.approval_challenge_sha256) `
    -ExpectedCandidateSha256 $v2CandidateSha256 -Name 'boot2_v2_install'
$v2BlobOffset = [int64]$v2Install.status.candidate_blob_offset
$v2BlobFrameLen = [int64]$v2Install.status.candidate_blob_frame_len
$sourceV2Json = Assert-SourceRevision $projectId $v2RevisionSha256 'source_v2_before_tamper'

# QEMU monitor quit flushes both test disks. The host then flips exactly one
# payload byte in the v2 frame selected by the guest-returned relative offset.
Stop-ProjectInstallBoot
$persistInfo = (& python (Join-Path $RepoRoot 'scripts\make-gpt-persist-image.py') --inspect-json $PersistDiskImage 2>&1 | Out-String) | ConvertFrom-Json
$seedData = @($persistInfo.partitions | Where-Object { $_.name -eq 'SEED_DATA' })
if ($seedData.Count -ne 1) { throw 'project-install could not resolve exact SEED_DATA partition' }
$artstorStartLba = [int64]$persistInfo.constants.ARTSTOR.start_lba
$sectorSize = [int64]$persistInfo.sector_size
$frameOffset = ([int64]$seedData[0].first_lba + $artstorStartLba) * $sectorSize + $v2BlobOffset
$persistLength = (Get-Item -LiteralPath $PersistDiskImage).Length
Add-InstallPredicate 'tamper:bounded_exact_v2_frame' 'guest-selected v2 candidate payload byte is inside its exact persisted frame and disposable image' (
    $artstorStartLba -gt 0 -and $sectorSize -gt 0 -and $v2BlobFrameLen -gt 0 -and
    $frameOffset -ge 0 -and ($frameOffset + $v2BlobFrameLen) -le $persistLength
) "absolute_frame_offset=$frameOffset relative_frame_offset=$v2BlobOffset frame_len=$v2BlobFrameLen"
$v2CandidateBytes = [IO.File]::ReadAllBytes($v2CandidatePath)
$frameBytes = [byte[]]::new($v2BlobFrameLen)
$stream = [IO.File]::Open($PersistDiskImage, [IO.FileMode]::Open, [IO.FileAccess]::ReadWrite, [IO.FileShare]::Read)
try {
    $stream.Position = $frameOffset
    $read = $stream.Read($frameBytes, 0, $frameBytes.Length)
    if ($read -ne $frameBytes.Length) { throw 'project-install could not read exact v2 candidate frame' }
    $payloadOffsets = New-Object System.Collections.Generic.List[int]
    for ($candidateOffset = 0; $candidateOffset -le $frameBytes.Length - $v2CandidateBytes.Length; $candidateOffset++) {
        $match = $true
        for ($candidateIndex = 0; $candidateIndex -lt $v2CandidateBytes.Length; $candidateIndex++) {
            if ($frameBytes[$candidateOffset + $candidateIndex] -ne $v2CandidateBytes[$candidateIndex]) {
                $match = $false
                break
            }
        }
        if ($match) { $payloadOffsets.Add($candidateOffset) }
    }
    Add-InstallPredicate 'tamper:exact_payload_located' 'exact host-built v2 bytes occur once inside the guest-returned ARTSTOR frame' (
        $payloadOffsets.Count -eq 1 -and $payloadOffsets[0] -gt 0
    ) $payloadOffsets
    $tamperOffset = $frameOffset + $payloadOffsets[0]
    $stream.Position = $tamperOffset
    $beforeByte = $stream.ReadByte()
    if ($beforeByte -lt 0) { throw 'project-install tamper could not read selected byte' }
    $afterByte = $beforeByte -bxor 1
    $stream.Position = $tamperOffset
    $stream.WriteByte([byte]$afterByte)
    $stream.Flush($true)
    $stream.Position = $tamperOffset
    $readbackByte = $stream.ReadByte()
}
finally { $stream.Dispose() }
Add-InstallPredicate 'tamper:exactly_one_byte_written' 'one v2 ARTSTOR candidate payload byte changed and read back' (
    $readbackByte -eq $afterByte -and $afterByte -ne $beforeByte
) "offset=$tamperOffset before=$beforeByte after=$afterByte"

$bootLogs.Add((Start-ProjectInstallBoot 'boot3'))
Assert-LogContains -Name 'project-install:boot3_real_v2_failure' `
    -Needle 'project_install_blob_frame_hash_mismatch' -TimeoutSeconds $TimeoutSeconds
Assert-LogContains -Name 'project-install:boot3_rollback_v1' `
    -Needle 'rollback_persisted=true fallback_running=true' -TimeoutSeconds $TimeoutSeconds
$sourceV2Boot3Json = Assert-SourceRevision $projectId $v2RevisionSha256 'source_v2_boot3_after_rollback'
Add-InstallPredicate 'boot3:source_unchanged_by_rollback' 'runtime rollback changes install state, never immutable source workspace' (
    $sourceV2Boot3Json -ceq $sourceV2Json
) $sourceV2Boot3Json

$uninstall = Invoke-SignedPhysicalProjectAction -Kind uninstall -Name 'boot3_uninstall_v1'
Assert-LogContains -Name 'project-install:uninstall_snapshot_refreshed' `
    -Needle 'reason=project_app_uninstall_tombstone_persisted' -TimeoutSeconds $TimeoutSeconds

Stop-ProjectInstallBoot
$bootLogs.Add((Start-ProjectInstallBoot 'boot4'))
Assert-LogContains -Name 'project-install:boot4_no_autoload' `
    -Needle 'PROJECT_APP_AUTOLOAD result=accepted phase=not_installed reason=project_app_uninstall_tombstone_resolved' `
    -TimeoutSeconds $TimeoutSeconds
Send-AgentCommand -Command 'services' -ExpectedMarker 'RAIOS_AGENT_END service.inventory' -Name 'project-install:boot4_inventory'
$boot4Inventory = (Get-LastAgentResponseJson -Method 'service.inventory').facts
$boot4Workspace = @($boot4Inventory.services | Where-Object { $_.id -eq 'svc.workspace.current_boot' })
Add-InstallPredicate 'boot4:no_workspace_autoload' 'uninstall tombstone suppresses durable workspace inventory and autoload' (
    $boot4Workspace.Count -eq 0
) $boot4Inventory.services
$sourceV2Boot4Json = Assert-SourceRevision $projectId $v2RevisionSha256 'source_v2_boot4_after_uninstall'
Add-InstallPredicate 'boot4:source_workspace_unchanged' 'install, real corruption rollback and uninstall never mutate the source workspace' (
    $sourceV2Boot4Json -ceq $sourceV2Json
) $sourceV2Boot4Json

$combinedLog = Join-Path $RunDir 'serial-project-install-combined.log'
$contents = @($bootLogs | ForEach-Object { Get-Content -LiteralPath $_ -Raw -ErrorAction Stop })
Set-Content -LiteralPath $combinedLog -Value ($contents -join [Environment]::NewLine) -Encoding UTF8
$SerialLog = $combinedLog
