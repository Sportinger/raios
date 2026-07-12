if (-not $StructuredStoreDiskImage -or -not $StructuredStoreFixture -or -not $PersistDiskImage) {
    throw 'project-build profile requires separate C1 store and valid-a BOOTCTL fixtures'
}
$fixtureReady = (
    [bool]$StructuredStoreFixture.valid -and
    [bool]$StructuredStoreFixture.disposable_qemu_only -and
    $StructuredStoreFixture.store_state -eq 'empty_unformatted' -and
    (Test-Path -LiteralPath $PersistDiskImage)
)
Add-Predicate -Name 'project-build:host_fixture_ready' -Expected 'dedicated empty qemu-only C1 store and separate valid-a BOOTCTL fixture' -Passed $fixtureReady -Actual $(if ($fixtureReady) { $StructuredStoreDiskImage } else { $StructuredStoreFixture | ConvertTo-Json -Compress -Depth 6 })
if (-not $fixtureReady) { throw 'Project-build fixture is not the dedicated empty QEMU test image' }

$script:SerialWriteChunkSize = 16
$script:SerialWriteDelayMilliseconds = 25

function Add-BuildPredicate {
    param([string]$Name, [string]$Expected, [bool]$Passed, [object]$Actual)
    $actualText = if ($Actual -is [string]) { $Actual } else { $Actual | ConvertTo-Json -Compress -Depth 12 }
    Add-Predicate -Name "project-build:$Name" -Expected $Expected -Passed $Passed -Actual $actualText
    if (-not $Passed) { throw "Project-build predicate failed: $Name" }
}

function Send-BuildCommand {
    param([string]$Command, [string]$Method, [string]$Name)
    Send-AgentCommand -Command $Command -ExpectedMarker "RAIOS_AGENT_END $Method" -Name "project-build:$Name"
    return (Get-LastAgentResponseJson -Method $Method).body.result
}

function Get-ByteSha256Hex {
    param([byte[]]$Bytes)
    $sha = [Security.Cryptography.SHA256]::Create()
    try { return ([BitConverter]::ToString($sha.ComputeHash($Bytes)) -replace '-', '').ToLowerInvariant() }
    finally { $sha.Dispose() }
}

function Convert-HexToBytes {
    param([string]$Hex)
    $bytes = [byte[]]::new($Hex.Length / 2)
    for ($index = 0; $index -lt $bytes.Length; $index++) {
        $bytes[$index] = [Convert]::ToByte($Hex.Substring($index * 2, 2), 16)
    }
    return $bytes
}

function Write-CanonicalString {
    param([IO.BinaryWriter]$Writer, [string]$Value)
    $bytes = [Text.Encoding]::UTF8.GetBytes($Value)
    $Writer.Write([uint16]$bytes.Length)
    $Writer.Write($bytes)
}

function Get-ProjectTreeSha256 {
    param([object[]]$Files)
    $stream = [IO.MemoryStream]::new()
    $writer = [IO.BinaryWriter]::new($stream, [Text.Encoding]::UTF8, $true)
    try {
        $writer.Write([Text.Encoding]::ASCII.GetBytes('RAIOSTR1'))
        $writer.Write([uint16]$Files.Count)
        foreach ($file in $Files) {
            Write-CanonicalString $writer $file.path
            $writer.Write([byte]$(if ($file.classification -eq 'public') { 1 } else { 2 }))
            Write-CanonicalString $writer $file.media_type
            $writer.Write([uint32]$file.bytes.Length)
            $writer.Write([byte[]](Convert-HexToBytes $file.sha256))
        }
        $writer.Flush()
        return Get-ByteSha256Hex $stream.ToArray()
    }
    finally { $writer.Dispose(); $stream.Dispose() }
}

function Get-ProjectRevisionSha256 {
    param([string]$ProjectId, [string]$TreeSha256)
    $stream = [IO.MemoryStream]::new()
    $writer = [IO.BinaryWriter]::new($stream, [Text.Encoding]::UTF8, $true)
    try {
        $writer.Write([Text.Encoding]::ASCII.GetBytes('RAIOSREV1'))
        $writer.Write([byte[]](Convert-HexToBytes $ProjectId))
        $writer.Write([byte]0)
        $writer.Write([byte[]]::new(32))
        $writer.Write([byte[]](Convert-HexToBytes $TreeSha256))
        Write-CanonicalString $writer 'owner_local_import'
        Write-CanonicalString $writer 'none_deterministic'
        $writer.Flush()
        return Get-ByteSha256Hex $stream.ToArray()
    }
    finally { $writer.Dispose(); $stream.Dispose() }
}

function Get-MaterializedTreeSha256 {
    param([object[]]$ProjectFiles, [object[]]$DependencyFiles)
    $files = @(
        @($ProjectFiles | ForEach-Object { [pscustomobject]@{ path = $_.path; bytes = $_.bytes; sha256 = $_.sha256 } }) +
        @($DependencyFiles | ForEach-Object { [pscustomobject]@{ path = "vendor/raios-local-helper/$($_.path)"; bytes = $_.bytes; sha256 = $_.sha256 } }) |
            Sort-Object path
    )
    $stream = [IO.MemoryStream]::new()
    $writer = [IO.BinaryWriter]::new($stream, [Text.Encoding]::UTF8, $true)
    try {
        $writer.Write([Text.Encoding]::ASCII.GetBytes('raios.host_builder_input_tree.v1'))
        $writer.Write([uint32]$files.Count)
        foreach ($file in $files) {
            Write-CanonicalString $writer $file.path
            $writer.Write([uint64]$file.bytes.Length)
            $writer.Write([byte[]](Convert-HexToBytes $file.sha256))
        }
        $writer.Flush()
        return Get-ByteSha256Hex $stream.ToArray()
    }
    finally { $writer.Dispose(); $stream.Dispose() }
}

function Add-HashesAndChunks {
    param([object[]]$Files)
    foreach ($file in $Files) {
        $file | Add-Member -NotePropertyName sha256 -NotePropertyValue (Get-ByteSha256Hex $file.bytes)
        $file | Add-Member -NotePropertyName chunks -NotePropertyValue @([pscustomobject]@{
            bytes = $file.bytes
            sha256 = Get-ByteSha256Hex $file.bytes
        })
    }
}

function Get-DependencyTreeSha256 {
    param([object[]]$Files)
    $stream = [IO.MemoryStream]::new()
    $writer = [IO.BinaryWriter]::new($stream, [Text.Encoding]::UTF8, $true)
    try {
        $writer.Write([Text.Encoding]::ASCII.GetBytes('raios.dependency_tree.v1'))
        $writer.Write([uint16]$Files.Count)
        foreach ($file in $Files) {
            Write-CanonicalString $writer $file.path
            Write-CanonicalString $writer $file.media_type
            $writer.Write([uint32]$file.bytes.Length)
            $writer.Write([byte[]](Convert-HexToBytes $file.sha256))
            $writer.Write([uint16]$file.chunks.Count)
            foreach ($chunk in $file.chunks) {
                $writer.Write([uint32]$chunk.bytes.Length)
                $writer.Write([byte[]](Convert-HexToBytes $chunk.sha256))
            }
        }
        $writer.Flush()
        return Get-ByteSha256Hex $stream.ToArray()
    }
    finally { $writer.Dispose(); $stream.Dispose() }
}

function Get-DependencyBundleSha256 {
    param(
        [string]$ProjectId, [string]$RevisionSha256, [string]$CargoLockSha256,
        [string]$Name, [string]$Version, [string]$Origin,
        [string]$LicenseExpression, [string]$LicensePath, [string]$LicenseSha256,
        [string]$TreeSha256
    )
    $stream = [IO.MemoryStream]::new()
    $writer = [IO.BinaryWriter]::new($stream, [Text.Encoding]::UTF8, $true)
    try {
        $writer.Write([Text.Encoding]::ASCII.GetBytes('raios.dependency_bundle.v1'))
        $writer.Write([byte[]](Convert-HexToBytes $ProjectId))
        $writer.Write([byte[]](Convert-HexToBytes $RevisionSha256))
        $writer.Write([byte[]](Convert-HexToBytes $CargoLockSha256))
        Write-CanonicalString $writer $Name
        Write-CanonicalString $writer $Version
        Write-CanonicalString $writer $Origin
        Write-CanonicalString $writer $LicenseExpression
        Write-CanonicalString $writer $LicensePath
        $writer.Write([byte[]](Convert-HexToBytes $LicenseSha256))
        $writer.Write([byte[]](Convert-HexToBytes $TreeSha256))
        $writer.Flush()
        return Get-ByteSha256Hex $stream.ToArray()
    }
    finally { $writer.Dispose(); $stream.Dispose() }
}

function Get-BuildSnapshotSha256 {
    param(
        [string]$ProjectId, [string]$RevisionSha256, [string]$SourceTreeSha256,
        [string]$CargoLockSha256, [object[]]$SourceFiles,
        [string]$BundleSha256, [string]$DependencyTreeSha256, [object[]]$DependencyFiles
    )
    $stream = [IO.MemoryStream]::new()
    $writer = [IO.BinaryWriter]::new($stream, [Text.Encoding]::UTF8, $true)
    try {
        $writer.Write([Text.Encoding]::ASCII.GetBytes('raios.project_build_snapshot.v1'))
        $writer.Write([Text.Encoding]::ASCII.GetBytes('RAIOBS01'))
        $writer.Write([byte[]](Convert-HexToBytes $ProjectId))
        $writer.Write([byte[]](Convert-HexToBytes $RevisionSha256))
        $writer.Write([byte[]](Convert-HexToBytes $SourceTreeSha256))
        $writer.Write([byte[]](Convert-HexToBytes $CargoLockSha256))
        $writer.Write([uint16]$SourceFiles.Count)
        foreach ($file in $SourceFiles) {
            Write-CanonicalString $writer $file.path
            $writer.Write([uint32]$file.bytes.Length)
            $writer.Write([byte[]](Convert-HexToBytes $file.sha256))
        }
        $writer.Write([uint16]1)
        $writer.Write([byte[]](Convert-HexToBytes $BundleSha256))
        $writer.Write([byte[]](Convert-HexToBytes $DependencyTreeSha256))
        $writer.Write([uint16]$DependencyFiles.Count)
        foreach ($file in $DependencyFiles) {
            Write-CanonicalString $writer $file.path
            $writer.Write([uint32]$file.bytes.Length)
            $writer.Write([byte[]](Convert-HexToBytes $file.sha256))
            $writer.Write([uint16]$file.chunks.Count)
            foreach ($chunk in $file.chunks) {
                $writer.Write([uint32]$chunk.bytes.Length)
                $writer.Write([byte[]](Convert-HexToBytes $chunk.sha256))
            }
        }
        $writer.Write([byte[]]::new(32))
        $writer.Flush()
        return Get-ByteSha256Hex $stream.ToArray()
    }
    finally { $writer.Dispose(); $stream.Dispose() }
}

function Get-BuildReceiptSha256 {
    param(
        [string]$InputManifestSha256, [string]$ToolchainSha256,
        [string]$FlagsSha256, [string]$EnvironmentSha256,
        [object]$RunOne, [object]$RunTwo, [string]$CandidateSha256, [int]$CandidateByteLen,
        [string]$ProjectId, [string]$RevisionSha256, [string]$SourceTreeSha256,
        [string]$CargoLockSha256, [object[]]$SourceFiles,
        [string]$BundleSha256, [string]$DependencyTreeSha256, [object[]]$DependencyFiles
    )
    $stream = [IO.MemoryStream]::new()
    $writer = [IO.BinaryWriter]::new($stream, [Text.Encoding]::UTF8, $true)
    try {
        $writer.Write([Text.Encoding]::ASCII.GetBytes('raios.project_build_receipt.v1'))
        $writer.Write([Text.Encoding]::ASCII.GetBytes('RAIOBS01'))
        $writer.Write([byte[]](Convert-HexToBytes $ProjectId))
        $writer.Write([byte[]](Convert-HexToBytes $RevisionSha256))
        $writer.Write([byte[]](Convert-HexToBytes $SourceTreeSha256))
        $writer.Write([byte[]](Convert-HexToBytes $CargoLockSha256))
        $writer.Write([uint16]$SourceFiles.Count)
        foreach ($file in $SourceFiles) {
            Write-CanonicalString $writer $file.path
            $writer.Write([uint32]$file.bytes.Length)
            $writer.Write([byte[]](Convert-HexToBytes $file.sha256))
        }
        $writer.Write([uint16]1)
        $writer.Write([byte[]](Convert-HexToBytes $BundleSha256))
        $writer.Write([byte[]](Convert-HexToBytes $DependencyTreeSha256))
        $writer.Write([uint16]$DependencyFiles.Count)
        foreach ($file in $DependencyFiles) {
            Write-CanonicalString $writer $file.path
            $writer.Write([uint32]$file.bytes.Length)
            $writer.Write([byte[]](Convert-HexToBytes $file.sha256))
            $writer.Write([uint16]$file.chunks.Count)
            foreach ($chunk in $file.chunks) {
                $writer.Write([uint32]$chunk.bytes.Length)
                $writer.Write([byte[]](Convert-HexToBytes $chunk.sha256))
            }
        }
        $writer.Write([byte[]](Convert-HexToBytes $InputManifestSha256))
        $writer.Write([byte[]](Convert-HexToBytes $ToolchainSha256))
        Write-CanonicalString $writer 'wasm32-unknown-unknown'
        $writer.Write([byte[]](Convert-HexToBytes $FlagsSha256))
        $writer.Write([byte[]](Convert-HexToBytes $EnvironmentSha256))
        Write-CanonicalString $writer 'deny_all_build_scripts_and_proc_macros_v1'
        Write-CanonicalString $writer 'pinned_dev_workstation_not_owner_sealed'
        foreach ($run in @($RunOne, $RunTwo)) {
            $writer.Write([int32]$run.exit_code)
            $writer.Write([byte[]](Convert-HexToBytes $run.stdout_sha256.Substring(7)))
            $writer.Write([byte[]](Convert-HexToBytes $run.stderr_sha256.Substring(7)))
            $writer.Write([uint32]$run.output_byte_len)
            $writer.Write([byte[]](Convert-HexToBytes $run.output_sha256.Substring(7)))
        }
        $writer.Write([byte[]](Convert-HexToBytes $CandidateSha256))
        $writer.Write([uint32]$CandidateByteLen)
        $writer.Write([byte]1)
        Write-CanonicalString $writer 'builder_attested_not_local_rebuild'
        foreach ($flag in @($true, $true, $false, $false, $false, $false, $false)) {
            $writer.Write([byte]$(if ($flag) { 1 } else { 0 }))
        }
        $writer.Flush()
        return Get-ByteSha256Hex $stream.ToArray()
    }
    finally { $writer.Dispose(); $stream.Dispose() }
}

function Assert-BuildPosture {
    param([object]$Result, [string]$Name)
    Add-BuildPredicate -Name "$Name`:posture" -Expected 'current-boot local-only non-authorizing workstation evidence' -Passed (
        $Result.scope -eq 'project_build' -and $Result.classification -eq 'local_only' -and
        $Result.persistence_posture -eq 'current_boot_ram_only' -and $Result.qemu_only -eq $true -and
        $Result.source_kind -eq 'owner_workstation_serial' -and $Result.target -eq 'wasm32-unknown-unknown' -and
        $Result.build_code_policy -eq 'deny_all_build_scripts_and_proc_macros_v1' -and
        $Result.toolchain_trust -eq 'pinned_dev_workstation_not_owner_sealed' -and
        $Result.generic_filesystem_exposed -eq $false -and
        $Result.network_attempted -eq $false -and
        $Result.provider_export_attempted -eq $false -and $Result.provider_export_authorized -eq $false -and
        $Result.guest_compiler_attempted -eq $false -and
        $Result.guest_build_attempted -eq $false -and $Result.guest_build_authorized -eq $false -and
        $Result.storage_write_attempted -eq $false -and $Result.writes_persistent_state -eq $false -and
        $Result.promotion_attempted -eq $false -and $Result.promotion_authorized -eq $false -and
        $Result.install_attempted -eq $false -and $Result.install_authorized -eq $false -and
        $Result.load_attempted -eq $false -and $Result.load_authorized -eq $false -and
        $Result.execution_attempted -eq $false -and $Result.execution_authorized -eq $false -and
        $Result.physical_media_attempted -eq $false -and $Result.physical_media_supported -eq $false
    ) -Actual $Result
}

function Assert-BuildDenied {
    param([object]$Result, [string]$Reason, [string]$Name)
    Add-BuildPredicate -Name "$Name`:denied" -Expected "$Reason without receipt or authority" -Passed (
        $Result.status -eq 'denied' -and $Result.reason -eq $Reason -and
        $Result.accepted -eq $false -and $Result.rejected -eq $true -and
        $Result.receipt_present -eq $false -and $null -eq $Result.receipt_sha256
    ) -Actual $Result
    Assert-BuildPosture $Result $Name
}

function Start-ProjectImport {
    param([string]$ProjectId, [string]$Name)
    $result = Send-BuildCommand "project.import_begin $ProjectId" 'project.import_begin' "$Name`:begin"
    Add-BuildPredicate "$Name`:begin" 'project import session accepted' ($result.accepted -eq $true -and $result.reason -eq 'project_import_started') $result
}

function Send-ProjectFile {
    param([object]$File, [string]$Name)
    $path64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($File.path))
    $begin = Send-BuildCommand "project.import_file_begin $path64 $($File.classification) $($File.media_type) $($File.bytes.Length) sha256:$($File.sha256)" 'project.import_file_begin' "$Name`:begin"
    Add-BuildPredicate "$Name`:begin" 'exact project file opened' ($begin.accepted -eq $true) $begin
    $base64 = [Convert]::ToBase64String($File.bytes)
    for ($offset = 0; $offset -lt $base64.Length; $offset += 256) {
        $chunk = $base64.Substring($offset, [Math]::Min(256, $base64.Length - $offset))
        $result = Send-BuildCommand "project.import_chunk $chunk" 'project.import_chunk' "$Name`:chunk_$offset"
        Add-BuildPredicate "$Name`:chunk_$offset" 'source chunk accepted RAM-only' ($result.accepted -eq $true) $result
    }
    $finalize = Send-BuildCommand 'project.import_file_finalize' 'project.import_file_finalize' "$Name`:finalize"
    Add-BuildPredicate "$Name`:finalize" 'exact project file finalized' ($finalize.accepted -eq $true -and $finalize.reason -eq 'project_import_file_finalized') $finalize
}

function Import-Project {
    param([string]$ProjectId, [object[]]$Files, [string]$TreeSha256, [string]$RevisionSha256, [string]$Name)
    Start-ProjectImport $ProjectId $Name
    foreach ($file in $Files) { Send-ProjectFile $file "$Name`_$($file.path)" }
    $commit = Send-BuildCommand 'project.import_commit' 'project.import_commit' "$Name`:commit"
    Add-BuildPredicate "$Name`:commit" 'exact immutable project committed' (
        $commit.accepted -eq $true -and $commit.reason -eq 'project_revision_committed' -and
        $commit.project_id -eq $ProjectId -and $commit.tree_sha256 -eq "sha256:$TreeSha256" -and
        $commit.revision_sha256 -eq "sha256:$RevisionSha256"
    ) $commit
}

function Import-Dependency {
    param(
        [string]$ProjectId, [string]$RevisionSha256, [string]$CargoLockSha256,
        [object[]]$Files, [string]$TreeSha256, [string]$BundleSha256,
        [string]$LicenseSha256, [string]$Name, [bool]$BuildScriptPresent = $false,
        [string]$PackageName = 'raios-local-helper', [string]$PackageVersion = '0.1.0',
        [string]$Origin = 'owner-local://serial/raios-local-helper/0.1.0'
    )
    $package64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($PackageName))
    $version64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($PackageVersion))
    $origin64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($Origin))
    $licenseExpression64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes('MIT'))
    $licensePath64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes('LICENSE'))
    $begin = Send-BuildCommand "project.dependency_begin $ProjectId sha256:$RevisionSha256 $package64 $version64 $origin64 $licenseExpression64 $licensePath64 sha256:$LicenseSha256" 'project.dependency_begin' "$Name`:begin"
    Add-BuildPredicate "$Name`:begin" 'dependency session bound to exact project revision and Cargo.lock' (
        $begin.accepted -eq $true -and $begin.reason -eq 'dependency_import_started' -and
        $begin.cargo_lock_sha256 -eq "sha256:$CargoLockSha256"
    ) $begin
    foreach ($file in $Files) {
        $path64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($file.path))
        $fileBegin = Send-BuildCommand "project.dependency_file_begin $path64 $($file.media_type) $($file.bytes.Length) sha256:$($file.sha256)" 'project.dependency_file_begin' "$Name`_$($file.path)`:begin"
        Add-BuildPredicate "$Name`_$($file.path)`:begin" 'dependency file opened' ($fileBegin.accepted -eq $true) $fileBegin
        foreach ($chunk in $file.chunks) {
            $chunk64 = [Convert]::ToBase64String($chunk.bytes)
            $chunkResult = Send-BuildCommand "project.dependency_chunk $chunk64" 'project.dependency_chunk' "$Name`_$($file.path)`:chunk"
            Add-BuildPredicate "$Name`_$($file.path)`:chunk" 'dependency chunk persisted but not visible' (
                $chunkResult.accepted -eq $true -and $chunkResult.chunk_persisted -eq $true -and $chunkResult.bundle_visible -eq $false
            ) $chunkResult
        }
        $finalize = Send-BuildCommand 'project.dependency_file_finalize' 'project.dependency_file_finalize' "$Name`_$($file.path)`:finalize"
        Add-BuildPredicate "$Name`_$($file.path)`:finalize" 'dependency whole-file hash verified' ($finalize.accepted -eq $true) $finalize
    }
    $commit = Send-BuildCommand 'project.dependency_commit' 'project.dependency_commit' "$Name`:commit"
    Add-BuildPredicate "$Name`:commit" 'exact inert dependency bundle committed manifest-last' (
        $commit.accepted -eq $true -and $commit.reason -eq 'dependency_bundle_committed' -and
        $commit.tree_sha256 -eq "sha256:$TreeSha256" -and $commit.bundle_sha256 -eq "sha256:$BundleSha256" -and
        $commit.build_script_present -eq $BuildScriptPresent -and $commit.bundle_visible -eq $true
    ) $commit
}

function Read-BuildBytes {
    param(
        [ValidateSet('source', 'dependency')][string]$Kind,
        [string]$ProjectId, [string]$RevisionSha256, [string]$BundleSha256,
        [object]$File, [string]$DestinationPath, [string]$Name
    )
    $path64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($File.path))
    $collected = [IO.MemoryStream]::new()
    try {
        for ($offset = 0; $offset -lt $File.bytes.Length; $offset += 512) {
            $length = [Math]::Min(512, $File.bytes.Length - $offset)
            $method = if ($Kind -eq 'source') { 'project.build_source_read' } else { 'project.build_dependency_read' }
            $command = if ($Kind -eq 'source') {
                "$method $ProjectId sha256:$RevisionSha256 $path64 $offset $length"
            } else {
                "$method $ProjectId sha256:$RevisionSha256 sha256:$BundleSha256 $path64 $offset $length"
            }
            $read = Send-BuildCommand $command $method "$Name`:$offset"
            $bytes = if ($read.bytes_b64) { [Convert]::FromBase64String($read.bytes_b64) } else { [byte[]]::new(0) }
            Add-BuildPredicate "$Name`:$offset`_exact" 'guest-reverified exact bounded source/dependency bytes' (
                $read.accepted -eq $true -and
                $read.reason -eq $(if ($Kind -eq 'source') { 'build_source_bytes_verified' } else { 'build_dependency_bytes_verified' }) -and
                $read.project_id -eq $ProjectId -and $read.project_revision_sha256 -eq "sha256:$RevisionSha256" -and
                $read.path -ceq $File.path -and [int]$read.file_byte_len -eq $File.bytes.Length -and
                $read.whole_sha256 -eq "sha256:$($File.sha256)" -and [int]$read.offset -eq $offset -and
                [int]$read.returned_len -eq $length -and
                (($Kind -eq 'source' -and $read.blob_sha256 -eq "sha256:$($File.sha256)") -or
                    ($Kind -eq 'dependency' -and $read.bundle_sha256 -eq "sha256:$BundleSha256")) -and
                ([Convert]::ToBase64String($bytes) -ceq [Convert]::ToBase64String([byte[]]$File.bytes[$offset..($offset + $length - 1)]))
            ) $read
            Assert-BuildPosture $read "$Name`:$offset"
            $collected.Write($bytes, 0, $bytes.Length)
        }
        $actual = $collected.ToArray()
        Add-BuildPredicate "$Name`:whole" 'materialized file rehashes to exact guest fact' (
            $actual.Length -eq $File.bytes.Length -and (Get-ByteSha256Hex $actual) -ceq $File.sha256
        ) (Get-ByteSha256Hex $actual)
        $directory = Split-Path -Parent $DestinationPath
        if ($directory) { [IO.Directory]::CreateDirectory($directory) | Out-Null }
        [IO.File]::WriteAllBytes($DestinationPath, $actual)
    }
    finally { $collected.Dispose() }
}

function Start-BuildSession {
    param([string]$ProjectId, [string]$RevisionSha256, [string]$ToolchainSha256, [string]$Name)
    return Send-BuildCommand "project.build_begin $ProjectId sha256:$RevisionSha256 sha256:$ToolchainSha256 sha256:$script:BuildFlagsContractSha256 sha256:$script:BuildEnvironmentContractSha256" 'project.build_begin' "$Name`:begin"
}

function Send-BuildRun {
    param([int]$Slot, [int]$ExitCode, [string]$StdoutSha256, [string]$StderrSha256, [int]$OutputByteLen, [string]$OutputSha256, [string]$Name)
    return Send-BuildCommand "project.build_run $Slot $ExitCode $StdoutSha256 $StderrSha256 $OutputByteLen $OutputSha256" 'project.build_run' "$Name`:run_$Slot"
}

foreach ($marker in @('C1_STRUCTURED_STORE_FIXTURE_ACCEPTED', 'C1_STRUCTURED_STORE_FORMAT_OPEN_OK')) {
    Assert-LogContains -Name "project-build:$marker" -Needle $marker -TimeoutSeconds $TimeoutSeconds
}

$projectId = '50000000000000000000000000000001'
$rootPackageName = 'raios-w4-app'
$rootPackageVersion = '0.1.0'
$projectFiles = @(
    [pscustomobject]@{
        path = 'Cargo.lock'; classification = 'local_only'; media_type = 'text/plain'
        bytes = [Text.Encoding]::UTF8.GetBytes("# This file is automatically @generated by Cargo.`nversion = 4`n`n[[package]]`nname = `"raios-local-helper`"`nversion = `"0.1.0`"`n`n[[package]]`nname = `"raios-w4-app`"`nversion = `"0.1.0`"`ndependencies = [`n `"raios-local-helper`",`n]`n")
    },
    [pscustomobject]@{
        path = 'Cargo.toml'; classification = 'local_only'; media_type = 'text/toml'
        bytes = [Text.Encoding]::UTF8.GetBytes("[package]`nname = `"raios-w4-app`"`nversion = `"0.1.0`"`nedition = `"2021`"`n`n[lib]`ncrate-type = [`"cdylib`"]`n`n[dependencies]`nraios-local-helper = { path = `"vendor/raios-local-helper`" }`n`n[profile.release]`npanic = `"abort`"`n")
    },
    [pscustomobject]@{
        path = 'src/lib.rs'; classification = 'local_only'; media_type = 'text/rust'
        bytes = [Text.Encoding]::UTF8.GetBytes("#![no_std]`nuse core::panic::PanicInfo;`n#[no_mangle]`npub extern `"C`" fn raios_service_main() -> i32 { raios_local_helper::answer() }`n#[panic_handler]`nfn panic(_: &PanicInfo) -> ! { core::arch::wasm32::unreachable() }`n")
    }
)
Add-HashesAndChunks $projectFiles
$projectTreeSha256 = Get-ProjectTreeSha256 $projectFiles
$projectRevisionSha256 = Get-ProjectRevisionSha256 $projectId $projectTreeSha256
$cargoLockSha256 = $projectFiles[0].sha256

$dependencyFiles = @(
    [pscustomobject]@{
        path = 'Cargo.toml'; media_type = 'text/toml'
        bytes = [Text.Encoding]::UTF8.GetBytes("[package]`nname = `"raios-local-helper`"`nversion = `"0.1.0`"`nedition = `"2021`"`n`n[lib]`n")
    },
    [pscustomobject]@{
        path = 'LICENSE'; media_type = 'text/plain'
        bytes = [Text.Encoding]::UTF8.GetBytes("MIT license for the inert local W4 fixture.`n")
    },
    [pscustomobject]@{
        path = 'src/lib.rs'; media_type = 'text/rust'
        bytes = [Text.Encoding]::UTF8.GetBytes("#![no_std]`npub const fn answer() -> i32 { 42 }`n")
    }
)
Add-HashesAndChunks $dependencyFiles
$dependencyTreeSha256 = Get-DependencyTreeSha256 $dependencyFiles
$dependencyBundleSha256 = Get-DependencyBundleSha256 `
    -ProjectId $projectId -RevisionSha256 $projectRevisionSha256 -CargoLockSha256 $cargoLockSha256 `
    -Name 'raios-local-helper' -Version '0.1.0' -Origin 'owner-local://serial/raios-local-helper/0.1.0' `
    -LicenseExpression 'MIT' -LicensePath 'LICENSE' -LicenseSha256 $dependencyFiles[1].sha256 `
    -TreeSha256 $dependencyTreeSha256
$materializedTreeSha256 = Get-MaterializedTreeSha256 $projectFiles $dependencyFiles

Import-Project $projectId $projectFiles $projectTreeSha256 $projectRevisionSha256 'positive_source'
Import-Dependency $projectId $projectRevisionSha256 $cargoLockSha256 $dependencyFiles $dependencyTreeSha256 $dependencyBundleSha256 $dependencyFiles[1].sha256 'positive_dependency'

$pinnedToolchainManifest = "schema=raios.owner_workstation_toolchain_manifest.v1`ncargo_version=cargo 1.84.0-nightly (15fbd2f60 2024-10-08)`ncargo_sha256=beb0e989a1597b624895907b918127bc08da919107b39bd9e954b3a1836aa13a`nrustc_version=rustc 1.84.0-nightly (9322d183f 2024-10-14)`nrustc_sha256=4fee4125dcdae28d01f0314a3820a09e9f9f717ee72494dc31ffe2d6de7eebf9`nrust_lld_sha256=375f02b38440bc5d7b0c3974abeb0fef080b654dc2fb87ef23887df7116129d2`nrustc_driver_sha256=e4aad069c7b57352c818aef521509ca42cc26cbfa5a20589ea9645bb4e111810`nhost_std_sha256=e54497532d6dcc306ada9a6086f30a2de5ed21740ab636414e569e017726fdfa`nwasm_target_manifest_sha256=032c9d596654633f8ab159236fff270bde984a15541763ebc6af5e2aaa560557`ntarget=wasm32-unknown-unknown`ntarget_lib_set_sha256=0ddff5952e3b6bbaeca08f78ea9e6841b833aef46fda393e7b4639e029267fde`n"
$pinnedToolchainSha256 = Get-ByteSha256Hex ([Text.Encoding]::UTF8.GetBytes($pinnedToolchainManifest))
$flagsContract = "cargo_command=cargo build --frozen --offline --locked --manifest-path=<input>/Cargo.toml --target=wasm32-unknown-unknown --release --target-dir=<fresh>`nrustflags=-Cpanic=abort$([char]0x1f)-Copt-level=z$([char]0x1f)-Ccodegen-units=1$([char]0x1f)-Cdebuginfo=0$([char]0x1f)-Cstrip=symbols$([char]0x1f)-Clinker=<toolchain>/lib/rustlib/x86_64-pc-windows-msvc/bin/rust-lld.exe$([char]0x1f)--remap-path-prefix=<input>=/raios/source`n"
$environmentContract = "CARGO_INCREMENTAL=0`nCARGO_NET_OFFLINE=true`nRUST_BACKTRACE=0`nSOURCE_DATE_EPOCH=0`nTZ=UTC`n"
$flagsContractSha256 = Get-ByteSha256Hex ([Text.Encoding]::UTF8.GetBytes($flagsContract))
$environmentContractSha256 = Get-ByteSha256Hex ([Text.Encoding]::UTF8.GetBytes($environmentContract))
$script:BuildFlagsContractSha256 = $flagsContractSha256
$script:BuildEnvironmentContractSha256 = $environmentContractSha256
Add-BuildPredicate 'host:fixed_contract_hashes' 'host canonical contracts exactly match pinned kernel constants' (
    $pinnedToolchainSha256 -eq '45723cc79127b688781d83eb3b042825fc2501c76ac02e4f8fee9f2904589e36' -and
    $flagsContractSha256 -eq '34ae7dd2f0bfdc316745e7306377b5b6252451b5aca16559be8aa2058e6e0ed1' -and
    $environmentContractSha256 -eq '2e9fc15d9adabd8e7c2a59b108ed91a70306e593ca08ad7475cba2b8465f79dc'
) "pinned=$pinnedToolchainSha256 flags=$flagsContractSha256 environment=$environmentContractSha256"
$inputManifestSha256 = Get-BuildSnapshotSha256 $projectId $projectRevisionSha256 $projectTreeSha256 $cargoLockSha256 $projectFiles $dependencyBundleSha256 $dependencyTreeSha256 $dependencyFiles

# Bounded fail-closed reads and begin denials need no workstation build.
$path64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes('Cargo.toml'))
$staleRead = Send-BuildCommand "project.build_source_read $projectId sha256:$('1' * 64) $path64 0 1" 'project.build_source_read' 'negative_stale_read'
Add-BuildPredicate 'negative_stale_read:denied' 'stale source revision returns no bytes' (
    $staleRead.status -eq 'denied' -and $staleRead.reason -eq 'build_project_revision_stale' -and $null -eq $staleRead.bytes_b64
) $staleRead
Assert-BuildPosture $staleRead 'negative_stale_read'
$dependencyPath64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes('src/lib.rs'))
$wrongBundleRead = Send-BuildCommand "project.build_dependency_read $projectId sha256:$projectRevisionSha256 sha256:$('2' * 64) $dependencyPath64 0 1" 'project.build_dependency_read' 'negative_wrong_bundle'
Add-BuildPredicate 'negative_wrong_bundle:denied' 'wrong dependency bundle returns no bytes' (
    $wrongBundleRead.status -eq 'denied' -and $wrongBundleRead.reason -eq 'build_dependency_path_not_found' -and $null -eq $wrongBundleRead.bytes_b64
) $wrongBundleRead
Assert-BuildPosture $wrongBundleRead 'negative_wrong_bundle'
$wrongToolchain = Start-BuildSession $projectId $projectRevisionSha256 ('3' * 64) 'negative_wrong_toolchain'
Assert-BuildDenied $wrongToolchain 'build_toolchain_manifest_mismatch' 'negative_wrong_toolchain'
$wrongFlags = Send-BuildCommand "project.build_begin $projectId sha256:$projectRevisionSha256 sha256:$pinnedToolchainSha256 sha256:$('6' * 64) sha256:$environmentContractSha256" 'project.build_begin' 'negative_wrong_flags:begin'
Assert-BuildDenied $wrongFlags 'build_flags_contract_mismatch' 'negative_wrong_flags'
$wrongEnvironment = Send-BuildCommand "project.build_begin $projectId sha256:$projectRevisionSha256 sha256:$pinnedToolchainSha256 sha256:$flagsContractSha256 sha256:$('7' * 64)" 'project.build_begin' 'negative_wrong_environment:begin'
Assert-BuildDenied $wrongEnvironment 'build_environment_contract_mismatch' 'negative_wrong_environment'

$buildScriptProjectId = '50000000000000000000000000000002'
$buildScriptFiles = @(
    [pscustomobject]@{ path = 'Build.RS'; classification = 'local_only'; media_type = 'text/rust'; bytes = [Text.Encoding]::UTF8.GetBytes("fn main() {}`n") },
    [pscustomobject]@{ path = 'Cargo.lock'; classification = 'local_only'; media_type = 'text/plain'; bytes = [Text.Encoding]::UTF8.GetBytes("version = 4`n") },
    [pscustomobject]@{ path = 'Cargo.toml'; classification = 'local_only'; media_type = 'text/toml'; bytes = [Text.Encoding]::UTF8.GetBytes("[package]`nname=`"denied-build-script`"`nversion=`"0.1.0`"`n") }
)
Add-HashesAndChunks $buildScriptFiles
$buildScriptTree = Get-ProjectTreeSha256 $buildScriptFiles
$buildScriptRevision = Get-ProjectRevisionSha256 $buildScriptProjectId $buildScriptTree
Import-Project $buildScriptProjectId $buildScriptFiles $buildScriptTree $buildScriptRevision 'negative_build_script_source'
$buildScriptDenied = Start-BuildSession $buildScriptProjectId $buildScriptRevision $pinnedToolchainSha256 'negative_build_script'
Assert-BuildDenied $buildScriptDenied 'build_source_build_script_denied' 'negative_build_script'

$dependencyBuildScriptProjectId = '50000000000000000000000000000003'
$dependencyBuildScriptProjectFiles = @(
    [pscustomobject]@{ path = 'Cargo.lock'; classification = 'local_only'; media_type = 'text/plain'; bytes = [Text.Encoding]::UTF8.GetBytes("# This file is automatically @generated by Cargo.`nversion = 4`n`n[[package]]`nname = `"raios-local-helper`"`nversion = `"0.1.0`"`n`n[[package]]`nname = `"raios-w4-app`"`nversion = `"0.1.0`"`ndependencies = [`n `"raios-local-helper`",`n]`n") },
    [pscustomobject]@{ path = 'Cargo.toml'; classification = 'local_only'; media_type = 'text/toml'; bytes = [Text.Encoding]::UTF8.GetBytes("[package]`nname = `"raios-w4-app`"`nversion = `"0.1.0`"`nedition = `"2021`"`n`n[lib]`ncrate-type = [`"cdylib`"]`n`n[dependencies]`nraios-local-helper = { path = `"vendor/raios-local-helper`" }`n") },
    [pscustomobject]@{ path = 'src/lib.rs'; classification = 'local_only'; media_type = 'text/rust'; bytes = [Text.Encoding]::UTF8.GetBytes("#![no_std]`n#[no_mangle] pub extern `"C`" fn raios_entry() -> i32 { raios_local_helper::answer() }`n") }
)
Add-HashesAndChunks $dependencyBuildScriptProjectFiles
$dependencyBuildScriptProjectTree = Get-ProjectTreeSha256 $dependencyBuildScriptProjectFiles
$dependencyBuildScriptProjectRevision = Get-ProjectRevisionSha256 $dependencyBuildScriptProjectId $dependencyBuildScriptProjectTree
Import-Project $dependencyBuildScriptProjectId $dependencyBuildScriptProjectFiles $dependencyBuildScriptProjectTree $dependencyBuildScriptProjectRevision 'negative_dependency_build_script_source'
$dependencyBuildScriptFiles = @(
    [pscustomobject]@{ path = 'Cargo.toml'; media_type = 'text/toml'; bytes = [Text.Encoding]::UTF8.GetBytes("[package]`nname = `"raios-local-helper`"`nversion = `"0.1.0`"`nedition = `"2021`"`nbuild = `"build.rs`"`n`n[lib]`npath = `"src/lib.rs`"`n") },
    [pscustomobject]@{ path = 'LICENSE'; media_type = 'text/plain'; bytes = [Text.Encoding]::UTF8.GetBytes("MIT License`n") },
    [pscustomobject]@{ path = 'build.rs'; media_type = 'text/rust'; bytes = [Text.Encoding]::UTF8.GetBytes("fn main() {}`n") },
    [pscustomobject]@{ path = 'src/lib.rs'; media_type = 'text/rust'; bytes = [Text.Encoding]::UTF8.GetBytes("#![no_std]`npub const fn answer() -> i32 { 42 }`n") }
)
Add-HashesAndChunks $dependencyBuildScriptFiles
$dependencyBuildScriptTree = Get-DependencyTreeSha256 $dependencyBuildScriptFiles
$dependencyBuildScriptBundle = Get-DependencyBundleSha256 `
    -ProjectId $dependencyBuildScriptProjectId -RevisionSha256 $dependencyBuildScriptProjectRevision `
    -CargoLockSha256 $dependencyBuildScriptProjectFiles[0].sha256 -Name 'raios-local-helper' -Version '0.1.0' `
    -Origin 'owner-local://serial/raios-local-helper/0.1.0' -LicenseExpression 'MIT' -LicensePath 'LICENSE' `
    -LicenseSha256 $dependencyBuildScriptFiles[1].sha256 -TreeSha256 $dependencyBuildScriptTree
Import-Dependency $dependencyBuildScriptProjectId $dependencyBuildScriptProjectRevision $dependencyBuildScriptProjectFiles[0].sha256 `
    $dependencyBuildScriptFiles $dependencyBuildScriptTree $dependencyBuildScriptBundle $dependencyBuildScriptFiles[1].sha256 `
    'negative_dependency_build_script_bundle' $true
$dependencyBuildScriptDenied = Start-BuildSession $dependencyBuildScriptProjectId $dependencyBuildScriptProjectRevision $pinnedToolchainSha256 'negative_dependency_build_script'
Assert-BuildDenied $dependencyBuildScriptDenied 'build_dependency_build_script_denied' 'negative_dependency_build_script'

# Exact guest reads are the only input bytes handed to the workstation builder.
$materializedInput = Join-Path $RunDir 'project-build-materialized-input'
$vendorRoot = Join-Path $materializedInput 'vendor\raios-local-helper'
foreach ($file in $projectFiles) {
    Read-BuildBytes source $projectId $projectRevisionSha256 '' $file (Join-Path $materializedInput $file.path) "materialize_source_$($file.path)"
}
foreach ($file in $dependencyFiles) {
    Read-BuildBytes dependency $projectId $projectRevisionSha256 $dependencyBundleSha256 $file (Join-Path $vendorRoot $file.path) "materialize_dependency_$($file.path)"
}

$builderOutput = Join-Path $RunDir 'project-build-workstation-output'
$builderScript = Join-Path $RepoRoot 'scripts\build-project-wasm.ps1'
& powershell -NoProfile -ExecutionPolicy Bypass -File $builderScript `
    -InputDirectory $materializedInput -OutputDirectory $builderOutput `
    -ProjectId $projectId -ProjectRevisionSha256 $projectRevisionSha256 `
    -ExpectedSourceTreeSha256 $projectTreeSha256 -ExpectedCargoLockSha256 $cargoLockSha256 `
    -ExpectedMaterializedTreeSha256 $materializedTreeSha256 `
    -ExpectedPackageName $rootPackageName -ExpectedPackageVersion $rootPackageVersion `
    -VendorDirectory $vendorRoot -ExpectedDependencyName 'raios-local-helper' `
    -ExpectedDependencyVersion '0.1.0' -ExpectedDependencyBundleSha256 $dependencyBundleSha256 `
    -TimeoutSeconds $TimeoutSeconds
if ($LASTEXITCODE -ne 0) { throw "project-build workstation builder failed with exit code $LASTEXITCODE" }
$buildResultPath = Join-Path $builderOutput 'build-result.json'
$candidatePath = Join-Path $builderOutput 'candidate.wasm'
$buildResult = Get-Content -LiteralPath $buildResultPath -Raw | ConvertFrom-Json
Add-BuildPredicate 'workstation:exact_result' 'offline two-build result binds exact source/dependency/toolchain facts' (
    $buildResult.format -eq 'project-build-workstation-output-1' -and
    $buildResult.status -eq 'accepted' -and $buildResult.reason -eq 'reproducible_wasm_built' -and
    $buildResult.project_id -eq $projectId -and $buildResult.project_revision_sha256 -eq "sha256:$projectRevisionSha256" -and
    $buildResult.source_tree_sha256 -eq "sha256:$projectTreeSha256" -and
    $buildResult.materialized_tree_sha256 -eq "sha256:$materializedTreeSha256" -and
    $buildResult.cargo_lock_sha256 -eq "sha256:$cargoLockSha256" -and
    $buildResult.package.name -eq $rootPackageName -and $buildResult.package.version -eq $rootPackageVersion -and
    $buildResult.dependency.bundle_sha256 -eq "sha256:$dependencyBundleSha256" -and
    $buildResult.target -eq 'wasm32-unknown-unknown' -and $buildResult.offline -eq $true -and
    $buildResult.network_attempted -eq $false -and $buildResult.build_script_attempted -eq $false -and
    $buildResult.proc_macro_attempted -eq $false -and $buildResult.normalization_attempted -eq $false -and
    $buildResult.flags_contract_sha256 -eq "sha256:$flagsContractSha256" -and
    $buildResult.environment_contract_sha256 -eq "sha256:$environmentContractSha256" -and
    $buildResult.pinned_contract_manifest_sha256 -eq "sha256:$pinnedToolchainSha256" -and
    $buildResult.measured_toolchain_manifest_sha256 -eq 'sha256:0cefad671146ab3b63b115c714cf0b6ccf5e45fcc65adf8068d5ed60417822ad' -and
    $buildResult.reproducible -eq $true -and @($buildResult.builds).Count -eq 2 -and
    $buildResult.builds[0].wasm_sha256 -eq $buildResult.builds[1].wasm_sha256 -and
    [int]$buildResult.builds[0].wasm_byte_len -eq [int]$buildResult.builds[1].wasm_byte_len
) $buildResult
$toolFiles = @($buildResult.toolchain.files)
Add-BuildPredicate 'workstation:measured_toolchain' 'pinned rustc/cargo/rust-lld and wasm target libraries measured' (
    $buildResult.toolchain.toolchain_id -eq 'nightly-2024-10-15-x86_64-pc-windows-msvc' -and
    $buildResult.toolchain.rustc_version -match 'commit-hash: 9322d183f45e0fd5a509820874cc5ff27744a479' -and
    @($toolFiles | Where-Object { $_.path -eq 'bin/cargo.exe' -and $_.sha256 -eq 'sha256:beb0e989a1597b624895907b918127bc08da919107b39bd9e954b3a1836aa13a' }).Count -eq 1 -and
    @($toolFiles | Where-Object { $_.path -eq 'bin/rustc.exe' -and $_.sha256 -eq 'sha256:4fee4125dcdae28d01f0314a3820a09e9f9f717ee72494dc31ffe2d6de7eebf9' }).Count -eq 1 -and
    @($toolFiles | Where-Object { $_.path -eq 'bin/rustc_driver-9a6caa80bb63f9f9.dll' -and $_.sha256 -eq 'sha256:e4aad069c7b57352c818aef521509ca42cc26cbfa5a20589ea9645bb4e111810' }).Count -eq 1 -and
    @($toolFiles | Where-Object { $_.path -eq 'bin/std-0cd72b70e9ef0a54.dll' -and $_.sha256 -eq 'sha256:e54497532d6dcc306ada9a6086f30a2de5ed21740ab636414e569e017726fdfa' }).Count -eq 1 -and
    @($toolFiles | Where-Object { $_.path -eq 'lib/rustlib/x86_64-pc-windows-msvc/bin/rust-lld.exe' -and $_.sha256 -eq 'sha256:375f02b38440bc5d7b0c3974abeb0fef080b654dc2fb87ef23887df7116129d2' }).Count -eq 1 -and
    @($toolFiles | Where-Object { $_.path -eq 'lib/rustlib/manifest-rust-std-wasm32-unknown-unknown' -and $_.sha256 -eq 'sha256:032c9d596654633f8ab159236fff270bde984a15541763ebc6af5e2aaa560557' }).Count -eq 1 -and
    [int]$buildResult.toolchain.target_lib_file_count -eq 27 -and
    @($buildResult.toolchain.target_lib_set).Count -eq 27 -and
    $buildResult.toolchain.target_lib_set_sha256 -eq 'sha256:0ddff5952e3b6bbaeca08f78ea9e6841b833aef46fda393e7b4639e029267fde'
) $buildResult.toolchain

$zeroHash = '4' * 64
$oneHash = '5' * 64
$missingBegin = Start-BuildSession $projectId $projectRevisionSha256 $pinnedToolchainSha256 'negative_missing_run'
Add-BuildPredicate 'negative_missing_run:started' 'real snapshot session started' ($missingBegin.accepted -eq $true) $missingBegin
$missingCommit = Send-BuildCommand 'project.build_commit' 'project.build_commit' 'negative_missing_run:commit'
Assert-BuildDenied $missingCommit 'build_run_missing' 'negative_missing_run'

$failedBegin = Start-BuildSession $projectId $projectRevisionSha256 $pinnedToolchainSha256 'negative_failed_run'
Add-BuildPredicate 'negative_failed_run:started' 'failed-run session started' ($failedBegin.accepted -eq $true) $failedBegin
Send-BuildRun 1 1 "sha256:$zeroHash" "sha256:$zeroHash" 8 "sha256:$zeroHash" 'negative_failed_run' | Out-Null
Send-BuildRun 2 0 "sha256:$zeroHash" "sha256:$zeroHash" 8 "sha256:$zeroHash" 'negative_failed_run' | Out-Null
$failedCommit = Send-BuildCommand 'project.build_commit' 'project.build_commit' 'negative_failed_run:commit'
Assert-BuildDenied $failedCommit 'build_run_failed' 'negative_failed_run'

$mismatchBegin = Start-BuildSession $projectId $projectRevisionSha256 $pinnedToolchainSha256 'negative_mismatch_run'
Add-BuildPredicate 'negative_mismatch_run:started' 'mismatch session started' ($mismatchBegin.accepted -eq $true) $mismatchBegin
Send-BuildRun 1 0 "sha256:$zeroHash" "sha256:$zeroHash" 8 "sha256:$zeroHash" 'negative_mismatch_run' | Out-Null
Send-BuildRun 2 0 "sha256:$zeroHash" "sha256:$zeroHash" 8 "sha256:$oneHash" 'negative_mismatch_run' | Out-Null
$mismatchCommit = Send-BuildCommand 'project.build_commit' 'project.build_commit' 'negative_mismatch_run:commit'
Assert-BuildDenied $mismatchCommit 'build_outputs_not_reproducible' 'negative_mismatch_run'

$delivery = Send-CandidateBytes -Path $candidatePath
$candidateFinalize = $delivery.finalize_response.body.result
$candidateSha256 = (Get-FileHash -LiteralPath $candidatePath -Algorithm SHA256).Hash.ToLowerInvariant()
$candidateByteLen = (Get-Item -LiteralPath $candidatePath).Length
Add-BuildPredicate 'candidate:retained_inert' 'workstation Wasm retained as exact inert current-boot candidate' (
    $candidateFinalize.rejected -eq $false -and $candidateFinalize.wasm_valid -eq $true -and
    $candidateFinalize.retained_in_ram -eq $true -and
    $candidateFinalize.artifact_sha256 -eq "sha256:$candidateSha256" -and
    [int]$candidateFinalize.byte_len -eq $candidateByteLen -and
    $candidateFinalize.load_attempted -eq $false -and $candidateFinalize.authorizes_load -eq $false -and
    $candidateFinalize.execution_attempted -eq $false -and $candidateFinalize.authorizes_execution -eq $false -and
    $candidateFinalize.writes_persistent_state -eq $false
) $candidateFinalize

$candidateMismatchBegin = Start-BuildSession $projectId $projectRevisionSha256 $pinnedToolchainSha256 'negative_candidate_mismatch'
Add-BuildPredicate 'negative_candidate_mismatch:started' 'candidate-mismatch session started' ($candidateMismatchBegin.accepted -eq $true) $candidateMismatchBegin
Send-BuildRun 1 0 "sha256:$zeroHash" "sha256:$zeroHash" $candidateByteLen "sha256:$zeroHash" 'negative_candidate_mismatch' | Out-Null
Send-BuildRun 2 0 "sha256:$zeroHash" "sha256:$zeroHash" $candidateByteLen "sha256:$zeroHash" 'negative_candidate_mismatch' | Out-Null
$candidateMismatchCommit = Send-BuildCommand 'project.build_commit' 'project.build_commit' 'negative_candidate_mismatch:commit'
Assert-BuildDenied $candidateMismatchCommit 'build_candidate_mismatch' 'negative_candidate_mismatch'

$positiveBegin = Start-BuildSession $projectId $projectRevisionSha256 $pinnedToolchainSha256 'positive'
Add-BuildPredicate 'positive:exact_begin' 'session snapshot binds exact source tree, lock, dependency and pinned toolchain' (
    $positiveBegin.status -eq 'accepted' -and $positiveBegin.reason -eq 'build_session_started' -and
    $positiveBegin.project_id -eq $projectId -and $positiveBegin.project_revision_sha256 -eq "sha256:$projectRevisionSha256" -and
    $positiveBegin.source_tree_sha256 -eq "sha256:$projectTreeSha256" -and
    $positiveBegin.cargo_lock_sha256 -eq "sha256:$cargoLockSha256" -and
    $positiveBegin.toolchain_manifest_sha256 -eq "sha256:$pinnedToolchainSha256" -and
    $positiveBegin.input_manifest_sha256 -eq "sha256:$inputManifestSha256" -and
    [int]$positiveBegin.source_file_count -eq $projectFiles.Count -and
    [int]$positiveBegin.dependency_bundle_count -eq 1 -and
    [int]$positiveBegin.dependency_chunk_count -eq $dependencyFiles.Count
) $positiveBegin
Assert-BuildPosture $positiveBegin 'positive_begin'
Add-BuildPredicate 'positive:contracts' 'guest derives exact fixed flags/environment/toolchain contract hashes' (
    $positiveBegin.flags_contract_sha256 -eq "sha256:$flagsContractSha256" -and
    $positiveBegin.environment_contract_sha256 -eq "sha256:$environmentContractSha256" -and
    $positiveBegin.pinned_toolchain_manifest_sha256 -eq "sha256:$pinnedToolchainSha256"
) $positiveBegin

$runOneFact = $buildResult.builds[0]
$runTwoFact = $buildResult.builds[1]
$receiptRunOne = [pscustomobject]@{
    exit_code = $runOneFact.run.exit_code
    stdout_sha256 = $runOneFact.run.stdout_sha256
    stderr_sha256 = $runOneFact.run.stderr_sha256
    output_byte_len = $runOneFact.wasm_byte_len
    output_sha256 = $runOneFact.wasm_sha256
}
$receiptRunTwo = [pscustomobject]@{
    exit_code = $runTwoFact.run.exit_code
    stdout_sha256 = $runTwoFact.run.stdout_sha256
    stderr_sha256 = $runTwoFact.run.stderr_sha256
    output_byte_len = $runTwoFact.wasm_byte_len
    output_sha256 = $runTwoFact.wasm_sha256
}
$runOne = Send-BuildRun 1 $runOneFact.run.exit_code $runOneFact.run.stdout_sha256 $runOneFact.run.stderr_sha256 $runOneFact.wasm_byte_len $runOneFact.wasm_sha256 'positive'
Add-BuildPredicate 'positive:run_one' 'first exact workstation build fact recorded' ($runOne.accepted -eq $true -and $runOne.reason -eq 'build_run_recorded') $runOne
Assert-BuildPosture $runOne 'positive_run_one'
$runTwo = Send-BuildRun 2 $runTwoFact.run.exit_code $runTwoFact.run.stdout_sha256 $runTwoFact.run.stderr_sha256 $runTwoFact.wasm_byte_len $runTwoFact.wasm_sha256 'positive'
Add-BuildPredicate 'positive:run_two' 'second byte-identical workstation build fact recorded' (
    $runTwo.accepted -eq $true -and $runTwo.reason -eq 'build_run_recorded' -and
    $runTwo.run_one.output_sha256 -eq "sha256:$candidateSha256" -and
    $runTwo.run_two.output_sha256 -eq "sha256:$candidateSha256"
) $runTwo
Assert-BuildPosture $runTwo 'positive_run_two'

$receiptSha256 = Get-BuildReceiptSha256 `
    -InputManifestSha256 $inputManifestSha256 -ToolchainSha256 $pinnedToolchainSha256 `
    -FlagsSha256 $flagsContractSha256 -EnvironmentSha256 $environmentContractSha256 `
    -RunOne $receiptRunOne -RunTwo $receiptRunTwo -CandidateSha256 $candidateSha256 -CandidateByteLen $candidateByteLen `
    -ProjectId $projectId -RevisionSha256 $projectRevisionSha256 -SourceTreeSha256 $projectTreeSha256 `
    -CargoLockSha256 $cargoLockSha256 -SourceFiles $projectFiles `
    -BundleSha256 $dependencyBundleSha256 -DependencyTreeSha256 $dependencyTreeSha256 -DependencyFiles $dependencyFiles
$commit = Send-BuildCommand 'project.build_commit' 'project.build_commit' 'positive:commit'
Add-BuildPredicate 'positive:exact_receipt' 'exact reproducible non-authorizing build receipt ready' (
    $commit.status -eq 'accepted' -and $commit.reason -eq 'build_receipt_ready' -and
    $commit.receipt_present -eq $true -and $commit.receipt_sha256 -eq "sha256:$receiptSha256" -and
    $commit.input_manifest_sha256 -eq "sha256:$inputManifestSha256" -and
    $commit.candidate_sha256 -eq "sha256:$candidateSha256" -and [int]$commit.candidate_byte_len -eq $candidateByteLen -and
    $commit.candidate_wasm_valid -eq $true -and $commit.host_build_attested -eq $true -and
    $commit.reproducible -eq $true -and $commit.independently_verified -eq $false -and
    $commit.evidence_quality -eq 'builder_attested_not_local_rebuild'
) $commit
Assert-BuildPosture $commit 'positive_commit'

$receipts = Send-BuildCommand "project.build_receipts $projectId sha256:$projectRevisionSha256" 'project.build_receipts' 'positive:receipts'
$receiptList = @($receipts.receipts)
Add-BuildPredicate 'positive:inspect_receipt' 'current-boot receipt inspection returns exact immutable hashes and no promotion/install/load/execute authority' (
    $receipts.status -eq 'accepted' -and $receipts.reason -eq 'build_receipts_present' -and
    $receipts.receipt_present -eq $true -and [int]$receipts.receipt_count -eq 1 -and $receiptList.Count -eq 1 -and
    $receipts.receipt_sha256 -eq "sha256:$receiptSha256" -and
    $receiptList[0].receipt_sha256 -eq "sha256:$receiptSha256" -and
    $receiptList[0].candidate_sha256 -eq "sha256:$candidateSha256" -and
    $receiptList[0].reproducible -eq $true -and $receiptList[0].independently_verified -eq $false -and
    $receiptList[0].authorizes_promotion -eq $false -and $receiptList[0].authorizes_install -eq $false -and
    $receiptList[0].authorizes_load -eq $false -and $receiptList[0].authorizes_execution -eq $false
) $receipts
Assert-BuildPosture $receipts 'positive_receipts'

# A later safe dependency changes the exact input snapshot; the retained receipt must fail closed.
$laterDependencyFiles = @(
    [pscustomobject]@{ path = 'Cargo.toml'; media_type = 'text/toml'; bytes = [Text.Encoding]::UTF8.GetBytes("[package]`nname = `"raios-second-helper`"`nversion = `"0.1.0`"`nedition = `"2021`"`n`n[lib]`npath = `"src/lib.rs`"`n") },
    [pscustomobject]@{ path = 'LICENSE'; media_type = 'text/plain'; bytes = [Text.Encoding]::UTF8.GetBytes("MIT License`n") },
    [pscustomobject]@{ path = 'src/lib.rs'; media_type = 'text/rust'; bytes = [Text.Encoding]::UTF8.GetBytes("#![no_std]`npub const fn second_answer() -> i32 { 7 }`n") }
)
Add-HashesAndChunks $laterDependencyFiles
$laterDependencyTree = Get-DependencyTreeSha256 $laterDependencyFiles
$laterDependencyBundle = Get-DependencyBundleSha256 `
    -ProjectId $projectId -RevisionSha256 $projectRevisionSha256 -CargoLockSha256 $cargoLockSha256 `
    -Name 'raios-second-helper' -Version '0.1.0' -Origin 'owner-local://serial/raios-second-helper/0.1.0' `
    -LicenseExpression 'MIT' -LicensePath 'LICENSE' -LicenseSha256 $laterDependencyFiles[1].sha256 `
    -TreeSha256 $laterDependencyTree
Import-Dependency -ProjectId $projectId -RevisionSha256 $projectRevisionSha256 -CargoLockSha256 $cargoLockSha256 `
    -Files $laterDependencyFiles -TreeSha256 $laterDependencyTree -BundleSha256 $laterDependencyBundle `
    -LicenseSha256 $laterDependencyFiles[1].sha256 -Name 'stale_receipt_later_dependency' `
    -PackageName 'raios-second-helper' -PackageVersion '0.1.0' `
    -Origin 'owner-local://serial/raios-second-helper/0.1.0'
$staleReceipts = Send-BuildCommand "project.build_receipts $projectId sha256:$projectRevisionSha256" 'project.build_receipts' 'negative_stale_receipt'
Assert-BuildDenied $staleReceipts 'build_receipt_input_snapshot_stale' 'negative_stale_receipt'
