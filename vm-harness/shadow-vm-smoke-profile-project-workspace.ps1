if (-not $StructuredStoreDiskImage -or -not $StructuredStoreFixture -or -not $PersistDiskImage) {
    throw "project-workspace profile requires separate C1 store and valid-a BOOTCTL fixtures"
}

$fixtureReady = (
    [bool]$StructuredStoreFixture.valid -and
    [bool]$StructuredStoreFixture.disposable_qemu_only -and
    $StructuredStoreFixture.store_state -eq "empty_unformatted" -and
    (Test-Path -LiteralPath $PersistDiskImage)
)
Add-Predicate -Name "project-workspace:host_fixture_ready" -Expected "dedicated empty qemu-only C1 store and separate valid-a BOOTCTL fixture" -Passed $fixtureReady -Actual $(if ($fixtureReady) { $StructuredStoreDiskImage } else { $StructuredStoreFixture | ConvertTo-Json -Compress -Depth 6 })
if (-not $fixtureReady) {
    throw "Project-workspace fixture is not the dedicated empty QEMU test image"
}

# Long base64 command lines must be paced below the guest UART drain rate.
$script:SerialWriteChunkSize = 16
$script:SerialWriteDelayMilliseconds = 25

function Add-ProjectPredicate {
    param([string]$Name, [string]$Expected, [bool]$Passed, [object]$Actual)
    $actualText = if ($Actual -is [string]) { $Actual } else { $Actual | ConvertTo-Json -Compress -Depth 8 }
    Add-Predicate -Name "project-workspace:$Name" -Expected $Expected -Passed $Passed -Actual $actualText
    if (-not $Passed) { throw "Project-workspace predicate failed: $Name" }
}

function Get-ByteSha256Hex {
    param([byte[]]$Bytes)
    $sha = [System.Security.Cryptography.SHA256]::Create()
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

function Convert-BytesToHex {
    param([byte[]]$Bytes)
    return ([BitConverter]::ToString($Bytes) -replace '-', '').ToLowerInvariant()
}

function Write-CanonicalString {
    param([System.IO.BinaryWriter]$Writer, [string]$Value)
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($Value)
    $Writer.Write([uint16]$bytes.Length)
    $Writer.Write($bytes)
}

function Get-ProjectTreeSha256 {
    param([object[]]$Files)
    $stream = [System.IO.MemoryStream]::new()
    $writer = [System.IO.BinaryWriter]::new($stream, [System.Text.Encoding]::UTF8, $true)
    try {
        $writer.Write([System.Text.Encoding]::ASCII.GetBytes('RAIOSTR1'))
        $writer.Write([uint16]$Files.Count)
        foreach ($file in $Files) {
            Write-CanonicalString -Writer $writer -Value $file.path
            $writer.Write([byte]$(if ($file.classification -eq 'public') { 1 } else { 2 }))
            Write-CanonicalString -Writer $writer -Value $file.media_type
            $writer.Write([uint32]$file.bytes.Length)
            $writer.Write([byte[]](Convert-HexToBytes -Hex $file.sha256))
        }
        $writer.Flush()
        return Get-ByteSha256Hex -Bytes $stream.ToArray()
    }
    finally { $writer.Dispose(); $stream.Dispose() }
}

function Get-ProjectRevisionSha256 {
    param(
        [string]$ProjectId,
        [string]$TreeSha256,
        [string]$ParentRevisionSha256 = '',
        [string]$Action = 'owner_local_import',
        [string]$TimeBasis = 'none_deterministic'
    )
    $stream = [System.IO.MemoryStream]::new()
    $writer = [System.IO.BinaryWriter]::new($stream, [System.Text.Encoding]::UTF8, $true)
    try {
        $writer.Write([System.Text.Encoding]::ASCII.GetBytes('RAIOSREV1'))
        $writer.Write([byte[]](Convert-HexToBytes -Hex $ProjectId))
        $writer.Write([byte]$(if ($ParentRevisionSha256) { 1 } else { 0 }))
        $writer.Write([byte[]]$(if ($ParentRevisionSha256) { Convert-HexToBytes -Hex $ParentRevisionSha256 } else { [byte[]]::new(32) }))
        $writer.Write([byte[]](Convert-HexToBytes -Hex $TreeSha256))
        Write-CanonicalString -Writer $writer -Value $Action
        Write-CanonicalString -Writer $writer -Value $TimeBasis
        $writer.Flush()
        return Get-ByteSha256Hex -Bytes $stream.ToArray()
    }
    finally { $writer.Dispose(); $stream.Dispose() }
}

function Get-DependencyTreeSha256 {
    param([object[]]$Files)
    $stream = [System.IO.MemoryStream]::new()
    $writer = [System.IO.BinaryWriter]::new($stream, [System.Text.Encoding]::UTF8, $true)
    try {
        $writer.Write([System.Text.Encoding]::ASCII.GetBytes('raios.dependency_tree.v1'))
        $writer.Write([uint16]$Files.Count)
        foreach ($file in $Files) {
            Write-CanonicalString -Writer $writer -Value $file.path
            Write-CanonicalString -Writer $writer -Value $file.media_type
            $writer.Write([uint32]$file.bytes.Length)
            $writer.Write([byte[]](Convert-HexToBytes -Hex $file.sha256))
            $writer.Write([uint16]$file.chunks.Count)
            foreach ($chunk in $file.chunks) {
                $writer.Write([uint32]$chunk.bytes.Length)
                $writer.Write([byte[]](Convert-HexToBytes -Hex $chunk.sha256))
            }
        }
        $writer.Flush()
        return Get-ByteSha256Hex -Bytes $stream.ToArray()
    }
    finally { $writer.Dispose(); $stream.Dispose() }
}

function Get-DependencyBundleSha256 {
    param(
        [string]$ProjectId,
        [string]$RevisionSha256,
        [string]$CargoLockSha256,
        [string]$Name,
        [string]$Version,
        [string]$Origin,
        [string]$LicenseExpression,
        [string]$LicensePath,
        [string]$LicenseSha256,
        [string]$TreeSha256
    )
    $stream = [System.IO.MemoryStream]::new()
    $writer = [System.IO.BinaryWriter]::new($stream, [System.Text.Encoding]::UTF8, $true)
    try {
        $writer.Write([System.Text.Encoding]::ASCII.GetBytes('raios.dependency_bundle.v1'))
        $writer.Write([byte[]](Convert-HexToBytes -Hex $ProjectId))
        $writer.Write([byte[]](Convert-HexToBytes -Hex $RevisionSha256))
        $writer.Write([byte[]](Convert-HexToBytes -Hex $CargoLockSha256))
        Write-CanonicalString -Writer $writer -Value $Name
        Write-CanonicalString -Writer $writer -Value $Version
        Write-CanonicalString -Writer $writer -Value $Origin
        Write-CanonicalString -Writer $writer -Value $LicenseExpression
        Write-CanonicalString -Writer $writer -Value $LicensePath
        $writer.Write([byte[]](Convert-HexToBytes -Hex $LicenseSha256))
        $writer.Write([byte[]](Convert-HexToBytes -Hex $TreeSha256))
        $writer.Flush()
        return Get-ByteSha256Hex -Bytes $stream.ToArray()
    }
    finally { $writer.Dispose(); $stream.Dispose() }
}

function Send-ProjectCommand {
    param([string]$Command, [string]$Method, [string]$Name)
    Send-AgentCommand -Command $Command -ExpectedMarker "RAIOS_AGENT_END $Method" -Name "project-workspace:$Name"
    return (Get-LastAgentResponseJson -Method $Method).body.result
}

function ConvertTo-Base64Text {
    param([string]$Value)
    return [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($Value))
}

function Assert-DependencyPosture {
    param(
        [object]$Result,
        [bool]$StorageWriteAttempted,
        [bool]$WritesPersistentState,
        [string]$Name
    )
    Add-ProjectPredicate -Name "$Name`:dependency_posture" -Expected 'local-only inert dependency data with no network/archive/export/compiler/build-script/install/load/execute/secret/physical authority' -Passed (
        $Result.scope -eq 'project_dependency' -and $Result.classification -eq 'local_only' -and
        $Result.persistence_posture -eq 'qemu_disposable_structured_store_only' -and
        $Result.qemu_only -eq $true -and $Result.source_kind -eq 'owner_local_serial' -and
        $Result.origin_verified -eq $false -and $Result.origin_evidence -eq 'owner_declared_unverified' -and
        $Result.license_verified -eq $false -and $Result.license_evidence -eq 'owner_declared_unverified' -and
        $Result.storage_write_attempted -eq $StorageWriteAttempted -and
        $Result.writes_persistent_state -eq $WritesPersistentState -and
        $Result.network_fetch_attempted -eq $false -and $Result.archive_parse_attempted -eq $false -and
        $Result.provider_export_attempted -eq $false -and $Result.provider_export_authorized -eq $false -and
        $Result.compiler_attempted -eq $false -and
        $Result.build_attempted -eq $false -and $Result.build_authorized -eq $false -and
        $Result.build_script_execution_attempted -eq $false -and $Result.build_script_execution_authorized -eq $false -and
        $Result.install_attempted -eq $false -and $Result.install_authorized -eq $false -and
        $Result.load_attempted -eq $false -and $Result.load_authorized -eq $false -and
        $Result.execution_attempted -eq $false -and $Result.execution_authorized -eq $false -and
        $Result.secret_access_attempted -eq $false -and
        $Result.physical_media_attempted -eq $false -and $Result.physical_media_supported -eq $false
    ) -Actual $Result
}

function Assert-DependencyDenied {
    param(
        [object]$Result,
        [string]$Reason,
        [string]$Name,
        [bool]$StorageWriteAttempted = $false,
        [bool]$WritesPersistentState = $false
    )
    Add-ProjectPredicate -Name "$Name`:dependency_denied" -Expected "$Reason exposes no dependency bundle" -Passed (
        $Result.status -eq 'denied' -and $Result.reason -eq $Reason -and
        $Result.accepted -eq $false -and $Result.rejected -eq $true -and
        $Result.bundle_visible -eq $false -and $null -eq $Result.bundle_sha256 -and
        [int]$Result.bundle_count -eq 0 -and @($Result.bundles).Count -eq 0
    ) -Actual $Result
    Assert-DependencyPosture -Result $Result -StorageWriteAttempted $StorageWriteAttempted -WritesPersistentState $WritesPersistentState -Name $Name
}

function Assert-NoDependencyBundles {
    param(
        [string]$ProjectId,
        [string]$RevisionSha256,
        [string]$CargoLockSha256,
        [string]$Name
    )
    $result = Send-ProjectCommand -Command "project.dependencies $ProjectId sha256:$RevisionSha256" -Method 'project.dependencies' -Name "$Name`:dependencies"
    Add-ProjectPredicate -Name "$Name`:no_bundle_visible" -Expected 'verified source revision has no committed dependency manifest' -Passed (
        $result.status -eq 'accepted' -and $result.reason -eq 'dependency_bundles_verified' -and
        $result.accepted -eq $true -and $result.rejected -eq $false -and
        $result.project_id -eq $ProjectId -and $result.project_revision_sha256 -eq "sha256:$RevisionSha256" -and
        (($CargoLockSha256 -and $result.cargo_lock_sha256 -eq "sha256:$CargoLockSha256") -or
            (-not $CargoLockSha256 -and $null -eq $result.cargo_lock_sha256)) -and
        $result.bundle_visible -eq $false -and [int]$result.bundle_count -eq 0 -and @($result.bundles).Count -eq 0
    ) -Actual $result
    Assert-DependencyPosture -Result $result -StorageWriteAttempted $false -WritesPersistentState $false -Name "$Name`:dependencies"
    return $result
}

function Start-DependencyImport {
    param(
        [string]$ProjectId,
        [string]$RevisionSha256,
        [string]$PackageName,
        [string]$PackageVersion,
        [string]$Origin,
        [string]$LicenseExpression,
        [string]$LicensePath,
        [string]$LicenseSha256,
        [string]$Name
    )
    $command = 'project.dependency_begin {0} sha256:{1} {2} {3} {4} {5} {6} sha256:{7}' -f @(
        $ProjectId, $RevisionSha256,
        (ConvertTo-Base64Text -Value $PackageName),
        (ConvertTo-Base64Text -Value $PackageVersion),
        (ConvertTo-Base64Text -Value $Origin),
        (ConvertTo-Base64Text -Value $LicenseExpression),
        (ConvertTo-Base64Text -Value $LicensePath),
        $LicenseSha256
    )
    $result = Send-ProjectCommand -Command $command -Method 'project.dependency_begin' -Name "$Name`:begin"
    return $result
}

function Assert-DependencyBeginAccepted {
    param(
        [object]$Result,
        [string]$ProjectId,
        [string]$RevisionSha256,
        [string]$CargoLockSha256,
        [string]$PackageName,
        [string]$PackageVersion,
        [string]$LicensePath,
        [string]$LicenseSha256,
        [string]$Name
    )
    Add-ProjectPredicate -Name "$Name`:dependency_started" -Expected 'exact immutable project revision and Cargo.lock bound to a RAM import session' -Passed (
        $Result.status -eq 'accepted' -and $Result.reason -eq 'dependency_import_started' -and
        $Result.accepted -eq $true -and $Result.rejected -eq $false -and
        $Result.project_id -eq $ProjectId -and $Result.project_revision_sha256 -eq "sha256:$RevisionSha256" -and
        $Result.cargo_lock_sha256 -eq "sha256:$CargoLockSha256" -and
        $Result.name -ceq $PackageName -and $Result.version -ceq $PackageVersion -and
        $Result.origin -ceq $dependencyPackageOrigin -and
        $Result.license_expression -ceq 'MIT' -and $Result.license_path -ceq $LicensePath -and
        $Result.license_sha256 -eq "sha256:$LicenseSha256" -and
        $Result.bundle_visible -eq $false -and [int]$Result.file_count -eq 0 -and [int]$Result.chunk_count -eq 0
    ) -Actual $Result
    Assert-DependencyPosture -Result $Result -StorageWriteAttempted $false -WritesPersistentState $false -Name $Name
}

function Send-DependencyFile {
    param(
        [object]$File,
        [string]$Name,
        [bool]$ExpectedChunkStorageWrite = $true,
        [bool]$ExpectedChunkPersistentWrite = $true
    )
    $pathBase64 = ConvertTo-Base64Text -Value $File.path
    $begin = Send-ProjectCommand -Command "project.dependency_file_begin $pathBase64 $($File.media_type) $($File.bytes.Length) sha256:$($File.sha256)" -Method 'project.dependency_file_begin' -Name "$Name`:file_begin"
    Add-ProjectPredicate -Name "$Name`:file_started" -Expected "exact inert dependency file $($File.path) opened" -Passed (
        $begin.status -eq 'accepted' -and $begin.reason -eq 'dependency_file_started' -and
        $begin.accepted -eq $true -and $begin.active_path -ceq $File.path -and
        [int]$begin.active_byte_len -eq 0 -and [int]$begin.expected_byte_len -eq $File.bytes.Length -and
        $begin.bundle_visible -eq $false
    ) -Actual $begin
    Assert-DependencyPosture -Result $begin -StorageWriteAttempted $false -WritesPersistentState $false -Name "$Name`:file_begin"

    $chunkIndex = 0
    foreach ($chunk in $File.chunks) {
        $chunkIndex++
        $chunkBase64 = [Convert]::ToBase64String($chunk.bytes)
        $chunkResult = Send-ProjectCommand -Command "project.dependency_chunk $chunkBase64" -Method 'project.dependency_chunk' -Name "$Name`:chunk_$chunkIndex"
        Add-ProjectPredicate -Name "$Name`:chunk_$chunkIndex`_orphan" -Expected 'exact chunk verified for the uncommitted session without exposing a new candidate manifest' -Passed (
            $chunkResult.status -eq 'accepted' -and $chunkResult.reason -eq 'dependency_chunk_persisted' -and
            $chunkResult.accepted -eq $true -and
            $chunkResult.chunk_sha256 -eq "sha256:$($chunk.sha256)" -and
            [int]$chunkResult.chunk_byte_len -eq $chunk.bytes.Length -and
            $chunkResult.chunk_persisted -eq $true -and $chunkResult.bundle_visible -eq $false -and
            [int]$chunkResult.orphan_chunk_count -eq [int]$chunkResult.chunk_count -and
            [int]$chunkResult.orphan_chunk_count -ge 1
        ) -Actual $chunkResult
        Assert-DependencyPosture -Result $chunkResult -StorageWriteAttempted $ExpectedChunkStorageWrite -WritesPersistentState $ExpectedChunkPersistentWrite -Name "$Name`:chunk_$chunkIndex"
    }

    $finalize = Send-ProjectCommand -Command 'project.dependency_file_finalize' -Method 'project.dependency_file_finalize' -Name "$Name`:file_finalize"
    Add-ProjectPredicate -Name "$Name`:file_finalized" -Expected 'whole-file length/hash verified while dependency remains invisible' -Passed (
        $finalize.status -eq 'accepted' -and $finalize.reason -eq 'dependency_file_finalized' -and
        $finalize.accepted -eq $true -and $null -eq $finalize.active_path -and
        $finalize.bundle_visible -eq $false
    ) -Actual $finalize
    Assert-DependencyPosture -Result $finalize -StorageWriteAttempted $false -WritesPersistentState $false -Name "$Name`:file_finalize"
    return $finalize
}

function Test-ExactDependencyFiles {
    param([object[]]$ActualFiles, [object[]]$ExpectedFiles)
    if ($ActualFiles.Count -ne $ExpectedFiles.Count) { return $false }
    for ($fileIndex = 0; $fileIndex -lt $ExpectedFiles.Count; $fileIndex++) {
        $actualFile = $ActualFiles[$fileIndex]
        $expectedFile = $ExpectedFiles[$fileIndex]
        if (
            $actualFile.path -cne $expectedFile.path -or
            $actualFile.classification -ne 'local_only' -or
            $actualFile.media_type -ne $expectedFile.media_type -or
            [int]$actualFile.whole_byte_len -ne $expectedFile.bytes.Length -or
            $actualFile.whole_sha256 -ne "sha256:$($expectedFile.sha256)" -or
            [int]$actualFile.chunk_count -ne $expectedFile.chunks.Count
        ) { return $false }
        $actualChunks = @($actualFile.chunks)
        if ($actualChunks.Count -ne $expectedFile.chunks.Count) { return $false }
        for ($chunkIndex = 0; $chunkIndex -lt $expectedFile.chunks.Count; $chunkIndex++) {
            if (
                [int]$actualChunks[$chunkIndex].byte_len -ne $expectedFile.chunks[$chunkIndex].bytes.Length -or
                $actualChunks[$chunkIndex].sha256 -ne "sha256:$($expectedFile.chunks[$chunkIndex].sha256)"
            ) { return $false }
        }
    }
    return $true
}

function Assert-ExactDependencyBundleFields {
    param([object]$Actual, [object[]]$Files, [string]$Name)
    $filesExact = Test-ExactDependencyFiles -ActualFiles @($Actual.files) -ExpectedFiles $Files
    Add-ProjectPredicate -Name "$Name`:exact_bundle" -Expected 'exact sorted metadata/file/chunk/tree/bundle hashes including inert build.rs presence' -Passed (
        $Actual.name -ceq $dependencyPackageName -and
        $Actual.version -ceq $dependencyPackageVersion -and
        $Actual.origin -ceq $dependencyPackageOrigin -and
        $Actual.license_expression -ceq $dependencyLicenseExpression -and
        $Actual.license_path -ceq $dependencyLicensePath -and
        $Actual.license_sha256 -eq "sha256:$dependencyLicenseSha256" -and
        $Actual.tree_sha256 -eq "sha256:$dependencyPackageTreeSha256" -and
        $Actual.bundle_sha256 -eq "sha256:$dependencyBundleSha256" -and
        $Actual.build_script_present -eq $true -and $filesExact
    ) -Actual $Actual
}

function Assert-ExactDependencyInspection {
    param([object]$Result, [string]$Name)
    $bundles = @($Result.bundles)
    Add-ProjectPredicate -Name "$Name`:exact_inspection" -Expected 'one exact source-revision-bound visible dependency bundle' -Passed (
        $Result.status -eq 'accepted' -and $Result.reason -eq 'dependency_bundles_verified' -and
        $Result.accepted -eq $true -and $Result.rejected -eq $false -and
        $Result.project_id -eq $dependencyProjectId -and
        $Result.project_revision_sha256 -eq "sha256:$dependencyRevisionSha256" -and
        $Result.cargo_lock_sha256 -eq "sha256:$cargoLockSha256" -and
        $Result.bundle_visible -eq $true -and [int]$Result.bundle_count -eq 1 -and $bundles.Count -eq 1 -and
        $null -eq $Result.tree_sha256 -and $null -eq $Result.bundle_sha256 -and @($Result.files).Count -eq 0
    ) -Actual $Result
    if ($bundles.Count -eq 1) {
        Assert-ExactDependencyBundleFields -Actual $bundles[0] -Files $dependencyPackageFiles -Name "$Name`:bundle"
    }
    Assert-DependencyPosture -Result $Result -StorageWriteAttempted $false -WritesPersistentState $false -Name $Name
}

function New-SingleChunkDependencyFile {
    param([string]$Path, [string]$MediaType, [byte[]]$Bytes)
    $sha256 = Get-ByteSha256Hex -Bytes $Bytes
    return [pscustomobject]@{
        path = $Path
        media_type = $MediaType
        bytes = $Bytes
        sha256 = $sha256
        chunks = @([pscustomobject]@{ bytes = $Bytes; sha256 = $sha256 })
    }
}

function Start-ProjectImport {
    param([string]$ProjectId, [string]$Name)
    $result = Send-ProjectCommand -Command "project.import_begin $ProjectId" -Method 'project.import_begin' -Name "$Name`:begin"
    Add-ProjectPredicate -Name "$Name`:begin_fields" -Expected 'accepted local-only qemu-only non-authorizing import session' -Passed (
        $result.status -eq 'accepted' -and $result.reason -eq 'project_import_started' -and
        $result.accepted -eq $true -and $result.project_id -eq $ProjectId -and
        $result.classification -eq 'local_only' -and $result.qemu_only -eq $true -and
        $result.storage_write_attempted -eq $false -and $result.writes_persistent_state -eq $false
    ) -Actual $result
}

function Send-ProjectFile {
    param(
        [string]$Name,
        [string]$Path,
        [string]$Classification,
        [string]$MediaType,
        [byte[]]$Bytes,
        [string]$DeclaredSha256 = ''
    )
    $sha256 = if ($DeclaredSha256) { $DeclaredSha256 } else { Get-ByteSha256Hex -Bytes $Bytes }
    $pathBase64 = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($Path))
    $begin = Send-ProjectCommand -Command "project.import_file_begin $pathBase64 $Classification $MediaType $($Bytes.Length) sha256:$sha256" -Method 'project.import_file_begin' -Name "$Name`:file_begin"
    Add-ProjectPredicate -Name "$Name`:file_begin_fields" -Expected "active exact file $Path" -Passed (
        $begin.accepted -eq $true -and $begin.reason -eq 'project_import_file_started' -and
        $begin.active_path -eq $Path -and [int]$begin.expected_byte_len -eq $Bytes.Length -and
        $begin.storage_write_attempted -eq $false
    ) -Actual $begin

    $base64 = [Convert]::ToBase64String($Bytes)
    $chunkIndex = 0
    for ($offset = 0; $offset -lt $base64.Length; $offset += 256) {
        $chunkIndex++
        $count = [Math]::Min(256, $base64.Length - $offset)
        $chunk = $base64.Substring($offset, $count)
        $chunkResult = Send-ProjectCommand -Command "project.import_chunk $chunk" -Method 'project.import_chunk' -Name "$Name`:chunk_$chunkIndex"
        Add-ProjectPredicate -Name "$Name`:chunk_$chunkIndex`_accepted" -Expected 'paced source bytes accepted without storage authority' -Passed (
            $chunkResult.accepted -eq $true -and $chunkResult.reason -eq 'project_import_chunk_accepted' -and
            $chunkResult.storage_write_attempted -eq $false
        ) -Actual $chunkResult
    }

    return Send-ProjectCommand -Command 'project.import_file_finalize' -Method 'project.import_file_finalize' -Name "$Name`:file_finalize"
}

function Assert-ProjectAbsent {
    param([string]$ProjectId, [string]$Name)
    $inspect = Send-ProjectCommand -Command "project.inspect $ProjectId" -Method 'project.inspect' -Name "$Name`:inspect_absent"
    Add-ProjectPredicate -Name "$Name`:no_committed_revision" -Expected 'project.inspect absent with no file/tree/revision facts' -Passed (
        $inspect.status -eq 'absent' -and $inspect.reason -eq 'project_not_found' -and
        $inspect.present -eq $false -and $null -eq $inspect.project_id -and
        $null -eq $inspect.tree_sha256 -and $null -eq $inspect.revision_sha256 -and
        [int]$inspect.file_count -eq 0 -and @($inspect.files).Count -eq 0 -and
        $inspect.builder_attempted -eq $false -and $inspect.install_attempted -eq $false -and
        $inspect.load_attempted -eq $false -and $inspect.execution_attempted -eq $false -and
        $inspect.provider_export_attempted -eq $false
    ) -Actual $inspect
}

function Assert-ProjectDenied {
    param([object]$Result, [string]$Reason, [string]$Name)
    Add-ProjectPredicate -Name "$Name`:denied" -Expected "fail-closed $Reason without persistent write" -Passed (
        $Result.status -eq 'denied' -and $Result.reason -eq $Reason -and
        $Result.accepted -eq $false -and $Result.rejected -eq $true -and
        $Result.storage_write_attempted -eq $false -and $Result.writes_persistent_state -eq $false -and
        $Result.builder_attempted -eq $false -and $Result.install_attempted -eq $false -and
        $Result.load_attempted -eq $false -and $Result.execution_attempted -eq $false -and
        $Result.provider_export_attempted -eq $false
    ) -Actual $Result
}

function Assert-ExactProjectRevision {
    param([object]$Result, [object[]]$Files, [string]$ProjectId, [string]$TreeSha256, [string]$RevisionSha256, [string]$ParentRevisionSha256 = '', [string]$Action = 'owner_local_import', [string]$Name)
    $actualFiles = @($Result.files)
    $filesExact = $actualFiles.Count -eq $Files.Count
    for ($index = 0; $filesExact -and $index -lt $Files.Count; $index++) {
        $filesExact = (
            $actualFiles[$index].path -ceq $Files[$index].path -and
            $actualFiles[$index].classification -eq $Files[$index].classification -and
            $actualFiles[$index].media_type -eq $Files[$index].media_type -and
            [int]$actualFiles[$index].byte_len -eq $Files[$index].bytes.Length -and
            $actualFiles[$index].blob_sha256 -eq "sha256:$($Files[$index].sha256)"
        )
    }
    Add-ProjectPredicate -Name "$Name`:exact_revision" -Expected 'host-computed sorted files, blob hashes, tree hash, and revision hash' -Passed (
        $Result.status -eq 'present' -and $Result.reason -eq 'project_revision_verified' -and
        $Result.present -eq $true -and $Result.project_id -eq $ProjectId -and
        (($ParentRevisionSha256 -and $Result.parent_revision_sha256 -eq "sha256:$ParentRevisionSha256") -or
            (-not $ParentRevisionSha256 -and $null -eq $Result.parent_revision_sha256)) -and
        $Result.revision_action -eq $Action -and
        $Result.tree_sha256 -eq "sha256:$TreeSha256" -and
        $Result.revision_sha256 -eq "sha256:$RevisionSha256" -and
        [int]$Result.file_count -eq $Files.Count -and
        [int]$Result.total_byte_len -eq (($Files | ForEach-Object { $_.bytes.Length } | Measure-Object -Sum).Sum) -and
        $filesExact
    ) -Actual $Result
    Add-ProjectPredicate -Name "$Name`:non_authority" -Expected 'inspection grants no builder/install/load/execute/provider-export authority' -Passed (
        $Result.qemu_only -eq $true -and $Result.physical_media_supported -eq $false -and
        $Result.builder_attempted -eq $false -and $Result.build_authorized -eq $false -and
        $Result.install_attempted -eq $false -and $Result.install_authorized -eq $false -and
        $Result.load_attempted -eq $false -and $Result.load_authorized -eq $false -and
        $Result.execution_attempted -eq $false -and $Result.execution_authorized -eq $false -and
        $Result.provider_export_attempted -eq $false -and $Result.provider_export_authorized -eq $false
    ) -Actual $Result
}

function Assert-ProjectQueryPosture {
    param([object]$Result, [string]$Name)
    Add-ProjectPredicate -Name "$Name`:non_authority" -Expected 'read-only local-only QEMU query with no write/export/build/install/load/execute authority' -Passed (
        $Result.classification -eq 'local_only' -and $Result.qemu_only -eq $true -and
        $Result.physical_media_supported -eq $false -and
        $Result.storage_write_attempted -eq $false -and $Result.writes_persistent_state -eq $false -and
        $Result.provider_export_attempted -eq $false -and $Result.provider_export_authorized -eq $false -and
        $Result.builder_attempted -eq $false -and $Result.build_authorized -eq $false -and
        $Result.install_attempted -eq $false -and $Result.install_authorized -eq $false -and
        $Result.load_attempted -eq $false -and $Result.load_authorized -eq $false -and
        $Result.execution_attempted -eq $false -and $Result.execution_authorized -eq $false
    ) -Actual $Result
}

function Assert-ProjectReadDenied {
    param([string]$Command, [string]$Reason, [string]$Name)
    $result = Send-ProjectCommand -Command $Command -Method 'project.read' -Name "$Name`:command"
    Add-ProjectPredicate -Name "$Name`:denied" -Expected "project.read denies $Reason without returning bytes" -Passed (
        $result.status -eq 'denied' -and $result.reason -eq $Reason -and
        [int]$result.returned_len -eq 0 -and $null -eq $result.bytes_hex
    ) -Actual $result
    Assert-ProjectQueryPosture -Result $result -Name $Name
}

function Assert-ProjectSearchDenied {
    param([string]$Command, [string]$Reason, [string]$Name)
    $result = Send-ProjectCommand -Command $Command -Method 'project.search' -Name "$Name`:command"
    Add-ProjectPredicate -Name "$Name`:denied" -Expected "project.search denies $Reason without returning locators or source bytes" -Passed (
        $result.status -eq 'denied' -and $result.reason -eq $Reason -and
        [int]$result.match_count -eq 0 -and @($result.matches).Count -eq 0 -and
        $null -eq $result.PSObject.Properties['snippet'] -and
        $null -eq $result.PSObject.Properties['bytes_hex']
    ) -Actual $result
    Assert-ProjectQueryPosture -Result $result -Name $Name
}

function Assert-ProjectReadAndSearch {
    param(
        [string]$ProjectId,
        [string]$RevisionSha256,
        [object]$SourceFile,
        [string]$Name
    )
    $pathBase64 = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($SourceFile.path))
    $sourceText = [System.Text.Encoding]::UTF8.GetString($SourceFile.bytes)
    $readOffset = $sourceText.IndexOf('println!', [StringComparison]::Ordinal)
    if ($readOffset -lt 0) { throw 'project-workspace fixture lost expected println! source range' }
    $readLength = $SourceFile.bytes.Length - $readOffset
    $expectedRead = [byte[]]$SourceFile.bytes[$readOffset..($SourceFile.bytes.Length - 1)]
    $read = Send-ProjectCommand -Command "project.read $ProjectId $pathBase64 $readOffset $readLength" -Method 'project.read' -Name "$Name`:read"
    Add-ProjectPredicate -Name "$Name`:read_exact_range" -Expected 'exact src/main.rs tail bytes plus immutable file/revision metadata and eof' -Passed (
        $read.status -eq 'present' -and $read.reason -eq 'project_read_verified' -and
        $read.project_id -eq $ProjectId -and $read.revision_sha256 -eq "sha256:$RevisionSha256" -and
        $read.path -ceq $SourceFile.path -and $read.file_classification -eq $SourceFile.classification -and
        $read.media_type -eq $SourceFile.media_type -and $read.blob_sha256 -eq "sha256:$($SourceFile.sha256)" -and
        [int]$read.file_byte_len -eq $SourceFile.bytes.Length -and [int]$read.offset -eq $readOffset -and
        [int]$read.requested_len -eq $readLength -and [int]$read.returned_len -eq $readLength -and
        $read.bytes_hex -ceq (Convert-BytesToHex -Bytes $expectedRead) -and $read.eof -eq $true
    ) -Actual $read
    Assert-ProjectQueryPosture -Result $read -Name "$Name`:read"

    $queryBytes = [System.Text.Encoding]::UTF8.GetBytes('hello from raiOS')
    $queryBase64 = [Convert]::ToBase64String($queryBytes)
    $expectedMatchOffset = $sourceText.IndexOf('hello from raiOS', [StringComparison]::Ordinal)
    $search = Send-ProjectCommand -Command "project.search $ProjectId $queryBase64 4" -Method 'project.search' -Name "$Name`:search"
    $matches = @($search.matches)
    Add-ProjectPredicate -Name "$Name`:search_exact_locator" -Expected 'one exact path/offset/length locator, with no snippet or source bytes' -Passed (
        $search.status -eq 'present' -and $search.reason -eq 'project_search_verified' -and
        $search.project_id -eq $ProjectId -and $search.revision_sha256 -eq "sha256:$RevisionSha256" -and
        $search.query_sha256 -eq "sha256:$(Get-ByteSha256Hex -Bytes $queryBytes)" -and
        [int]$search.query_byte_len -eq $queryBytes.Length -and [int]$search.searched_file_count -eq 2 -and
        [int]$search.limit -eq 4 -and
        [int]$search.match_count -eq 1 -and $search.truncated -eq $false -and $matches.Count -eq 1 -and
        $matches[0].path -ceq $SourceFile.path -and [int]$matches[0].byte_offset -eq $expectedMatchOffset -and
        [int]$matches[0].match_len -eq $queryBytes.Length -and
        $null -eq $search.PSObject.Properties['snippet'] -and
        $null -eq $search.PSObject.Properties['bytes_hex'] -and
        $null -eq $matches[0].PSObject.Properties['snippet'] -and
        $null -eq $matches[0].PSObject.Properties['bytes_hex']
    ) -Actual $search
    Assert-ProjectQueryPosture -Result $search -Name "$Name`:search"
}

function Assert-ProjectQueryDenials {
    param([string]$ProjectId, [object]$SourceFile, [string]$Name)
    $pathBase64 = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($SourceFile.path))
    $missingPathBase64 = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes('src/missing.rs'))
    $missingProjectId = 'ffeeddccbbaa99887766554433221100'
    $queryBase64 = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes('hello'))

    Assert-ProjectReadDenied -Command "project.read $missingProjectId $pathBase64 0 1" -Reason 'project_not_found' -Name "$Name`:read_wrong_project"
    Assert-ProjectReadDenied -Command "project.read $ProjectId $missingPathBase64 0 1" -Reason 'project_path_not_found' -Name "$Name`:read_wrong_path"
    Assert-ProjectReadDenied -Command "project.read $ProjectId $pathBase64 $($SourceFile.bytes.Length + 1) 1" -Reason 'project_read_offset_out_of_bounds' -Name "$Name`:read_offset_out_of_range"
    Assert-ProjectReadDenied -Command "project.read $ProjectId $pathBase64 0 513" -Reason 'project_read_limit_exceeded' -Name "$Name`:read_oversize"

    Assert-ProjectSearchDenied -Command "project.search $missingProjectId $queryBase64 4" -Reason 'project_not_found' -Name "$Name`:search_wrong_project"
    Assert-ProjectSearchDenied -Command "project.search $ProjectId  4" -Reason 'project_query_malformed' -Name "$Name`:search_empty_query"
    $oversizeQueryBase64 = [Convert]::ToBase64String([byte[]]::new(129))
    Assert-ProjectSearchDenied -Command "project.search $ProjectId $oversizeQueryBase64 4" -Reason 'project_search_query_limit_exceeded' -Name "$Name`:search_oversize_query"
    Assert-ProjectSearchDenied -Command "project.search $ProjectId $queryBase64 17" -Reason 'project_search_limit_invalid' -Name "$Name`:search_limit_overflow"
}

function Assert-ProjectEditPosture {
    param([object]$Result, [bool]$MayWrite, [string]$Name)
    Add-ProjectPredicate -Name "$Name`:edit_posture" -Expected $(if ($MayWrite) { 'only successful edit_commit may write the QEMU store' } else { 'overlay operation is RAM-only and non-authorizing' }) -Passed (
        $Result.classification -eq 'local_only' -and $Result.qemu_only -eq $true -and
        $Result.physical_media_supported -eq $false -and $Result.physical_media_attempted -eq $false -and
        $Result.provider_export_attempted -eq $false -and $Result.provider_export_authorized -eq $false -and
        $Result.builder_attempted -eq $false -and $Result.build_authorized -eq $false -and
        $Result.install_attempted -eq $false -and $Result.install_authorized -eq $false -and
        $Result.load_attempted -eq $false -and $Result.load_authorized -eq $false -and
        $Result.execution_attempted -eq $false -and $Result.execution_authorized -eq $false -and
        ($MayWrite -or ($Result.storage_write_attempted -eq $false -and $Result.writes_persistent_state -eq $false))
    ) -Actual $Result
}

function Start-ProjectEdit {
    param([string]$ProjectId, [string]$BaseRevisionSha256, [string]$Name)
    $result = Send-ProjectCommand -Command "project.edit_begin $ProjectId" -Method 'project.edit_begin' -Name "$Name`:edit_begin"
    Add-ProjectPredicate -Name "$Name`:edit_begin_bound" -Expected 'RAM-only overlay bound to the exact latest verified revision' -Passed (
        $result.status -eq 'accepted' -and $result.reason -eq 'project_edit_started' -and
        $result.accepted -eq $true -and $result.rejected -eq $false -and
        $result.project_id -eq $ProjectId -and $result.base_revision_sha256 -eq "sha256:$BaseRevisionSha256" -and
        $result.action -eq 'agent_overlay_commit'
    ) -Actual $result
    Assert-ProjectEditPosture -Result $result -MayWrite $false -Name "$Name`:edit_begin"
    return $result
}

function Send-ProjectEditFile {
    param(
        [string]$Name,
        [string]$Path,
        [string]$Classification,
        [string]$MediaType,
        [byte[]]$Bytes,
        [string]$DeclaredSha256 = ''
    )
    $sha256 = if ($DeclaredSha256) { $DeclaredSha256 } else { Get-ByteSha256Hex -Bytes $Bytes }
    $pathBase64 = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($Path))
    $begin = Send-ProjectCommand -Command "project.edit_file_begin $pathBase64 $Classification $MediaType $($Bytes.Length) sha256:$sha256" -Method 'project.edit_file_begin' -Name "$Name`:file_begin"
    Add-ProjectPredicate -Name "$Name`:file_begin" -Expected "bounded overlay file $Path opened without durable write" -Passed (
        $begin.status -eq 'accepted' -and $begin.reason -eq 'project_edit_file_started' -and
        $begin.accepted -eq $true -and $begin.active_path -ceq $Path -and
        [int]$begin.expected_byte_len -eq $Bytes.Length
    ) -Actual $begin
    Assert-ProjectEditPosture -Result $begin -MayWrite $false -Name "$Name`:file_begin"

    $base64 = [Convert]::ToBase64String($Bytes)
    for ($offset = 0; $offset -lt $base64.Length; $offset += 256) {
        $count = [Math]::Min(256, $base64.Length - $offset)
        $chunk = $base64.Substring($offset, $count)
        $chunkResult = Send-ProjectCommand -Command "project.edit_chunk $chunk" -Method 'project.edit_chunk' -Name "$Name`:chunk_$offset"
        Add-ProjectPredicate -Name "$Name`:chunk_$offset" -Expected 'overlay bytes accepted in RAM only' -Passed (
            $chunkResult.status -eq 'accepted' -and $chunkResult.reason -eq 'project_edit_chunk_accepted' -and
            $chunkResult.accepted -eq $true
        ) -Actual $chunkResult
        Assert-ProjectEditPosture -Result $chunkResult -MayWrite $false -Name "$Name`:chunk_$offset"
    }
    return Send-ProjectCommand -Command 'project.edit_file_finalize' -Method 'project.edit_file_finalize' -Name "$Name`:file_finalize"
}

function Assert-ProjectEditDenied {
    param([object]$Result, [string]$Reason, [string]$Name)
    Add-ProjectPredicate -Name "$Name`:edit_denied" -Expected "$Reason leaves no durable overlay mutation" -Passed (
        $Result.status -eq 'denied' -and $Result.reason -eq $Reason -and
        $Result.accepted -eq $false -and $Result.rejected -eq $true
    ) -Actual $Result
    Assert-ProjectEditPosture -Result $Result -MayWrite $false -Name $Name
}

foreach ($marker in @('C1_STRUCTURED_STORE_FIXTURE_ACCEPTED', 'C1_STRUCTURED_STORE_FORMAT_OPEN_OK')) {
    Assert-LogContains -Name "project-workspace:$marker" -Needle $marker -TimeoutSeconds $TimeoutSeconds
}

$oneByte = [System.Text.Encoding]::UTF8.GetBytes('x')
$oneByteHash = Get-ByteSha256Hex -Bytes $oneByte

foreach ($negative in @(
    @{ name = 'absolute_path'; id = '10000000000000000000000000000001'; path = '/src/main.rs'; reason = 'project_path_invalid' },
    @{ name = 'parent_path'; id = '10000000000000000000000000000002'; path = '../src/main.rs'; reason = 'project_path_invalid' }
)) {
    Start-ProjectImport -ProjectId $negative.id -Name $negative.name
    $path64 = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($negative.path))
    $denied = Send-ProjectCommand -Command "project.import_file_begin $path64 local_only text/rust 1 sha256:$oneByteHash" -Method 'project.import_file_begin' -Name "$($negative.name)`:file_begin"
    Assert-ProjectDenied -Result $denied -Reason $negative.reason -Name $negative.name
    Assert-ProjectAbsent -ProjectId $negative.id -Name $negative.name
}

$collisionId = '10000000000000000000000000000003'
Start-ProjectImport -ProjectId $collisionId -Name 'case_collision'
foreach ($path in @('Source.rs', 'source.RS')) {
    $finalized = Send-ProjectFile -Name "case_collision_$path" -Path $path -Classification public -MediaType text/rust -Bytes $oneByte
    Add-ProjectPredicate -Name "case_collision_$path`:finalized" -Expected 'case alias staged but not committed' -Passed ($finalized.accepted -eq $true -and $finalized.reason -eq 'project_import_file_finalized') -Actual $finalized
}
$collisionCommit = Send-ProjectCommand -Command 'project.import_commit' -Method 'project.import_commit' -Name 'case_collision:commit'
Assert-ProjectDenied -Result $collisionCommit -Reason 'project_path_collision' -Name 'case_collision'
Assert-ProjectAbsent -ProjectId $collisionId -Name 'case_collision'

$wrongHashId = '10000000000000000000000000000004'
Start-ProjectImport -ProjectId $wrongHashId -Name 'wrong_hash'
$wrongFinalize = Send-ProjectFile -Name 'wrong_hash' -Path 'src/main.rs' -Classification local_only -MediaType text/rust -Bytes $oneByte -DeclaredSha256 ('0' * 64)
Assert-ProjectDenied -Result $wrongFinalize -Reason 'project_file_hash_mismatch' -Name 'wrong_hash'
Assert-ProjectAbsent -ProjectId $wrongHashId -Name 'wrong_hash'

$quotaId = '10000000000000000000000000000005'
Start-ProjectImport -ProjectId $quotaId -Name 'file_quota'
$quotaPath64 = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes('large.bin'))
$quotaDenied = Send-ProjectCommand -Command "project.import_file_begin $quotaPath64 local_only application/octet-stream 32769 sha256:$('0' * 64)" -Method 'project.import_file_begin' -Name 'file_quota:file_begin'
Assert-ProjectDenied -Result $quotaDenied -Reason 'project_file_quota_exceeded' -Name 'file_quota'
Assert-ProjectAbsent -ProjectId $quotaId -Name 'file_quota'

$projectId = '00112233445566778899aabbccddeeff'
$files = @(
    [pscustomobject]@{
        path = 'Cargo.toml'; classification = 'public'; media_type = 'text/toml'
        bytes = [System.Text.Encoding]::UTF8.GetBytes("[package]`nname = `"genesis-workspace`"`nversion = `"0.1.0`"`nedition = `"2021`"`n")
    },
    [pscustomobject]@{
        path = 'src/main.rs'; classification = 'local_only'; media_type = 'text/rust'
        bytes = [System.Text.Encoding]::UTF8.GetBytes("fn main() {`n    println!(`"hello from raiOS`"`);`n}`n")
    }
)
foreach ($file in $files) { $file | Add-Member -NotePropertyName sha256 -NotePropertyValue (Get-ByteSha256Hex -Bytes $file.bytes) }
$treeSha256 = Get-ProjectTreeSha256 -Files $files
$revisionSha256 = Get-ProjectRevisionSha256 -ProjectId $projectId -TreeSha256 $treeSha256

Start-ProjectImport -ProjectId $projectId -Name 'positive'
foreach ($file in $files) {
    $finalized = Send-ProjectFile -Name "positive_$($file.path)" -Path $file.path -Classification $file.classification -MediaType $file.media_type -Bytes $file.bytes
    Add-ProjectPredicate -Name "positive_$($file.path)`:finalized" -Expected "exact host hash sha256:$($file.sha256) finalized" -Passed (
        $finalized.accepted -eq $true -and $finalized.reason -eq 'project_import_file_finalized' -and
        $finalized.storage_write_attempted -eq $false
    ) -Actual $finalized
}

$commit = Send-ProjectCommand -Command 'project.import_commit' -Method 'project.import_commit' -Name 'positive:commit'
Add-ProjectPredicate -Name 'positive:durable_commit' -Expected 'qemu-only durable revision commit with no build/install/load/execute/provider-export authority' -Passed (
    $commit.status -eq 'accepted' -and $commit.reason -eq 'project_revision_committed' -and
    $commit.accepted -eq $true -and $commit.project_id -eq $projectId -and
    $commit.retention -eq 'durable_when_committed' -and $commit.qemu_only -eq $true -and
    $commit.storage_write_attempted -eq $true -and $commit.writes_persistent_state -eq $true -and
    $commit.tree_sha256 -eq "sha256:$treeSha256" -and $commit.revision_sha256 -eq "sha256:$revisionSha256" -and
    $commit.builder_attempted -eq $false -and $commit.build_authorized -eq $false -and
    $commit.install_attempted -eq $false -and $commit.install_authorized -eq $false -and
    $commit.load_attempted -eq $false -and $commit.load_authorized -eq $false -and
    $commit.execution_attempted -eq $false -and $commit.execution_authorized -eq $false -and
    $commit.provider_export_attempted -eq $false -and $commit.provider_export_authorized -eq $false
) -Actual $commit

$firstInspect = Send-ProjectCommand -Command "project.inspect $projectId" -Method 'project.inspect' -Name 'positive:inspect_first_boot'
Assert-ExactProjectRevision -Result $firstInspect -Files $files -ProjectId $projectId -TreeSha256 $treeSha256 -RevisionSha256 $revisionSha256 -Name 'first_boot'
Assert-ProjectReadAndSearch -ProjectId $projectId -RevisionSha256 $revisionSha256 -SourceFile $files[1] -Name 'first_boot'
Assert-ProjectQueryDenials -ProjectId $projectId -SourceFile $files[1] -Name 'first_boot'
$firstRevisionJson = $firstInspect | ConvertTo-Json -Compress -Depth 8

# W2b negative overlays use a separate durable base. Each failed overlay must leave
# the last committed revision byte-identical; only the explicit stale-control import
# below is allowed to advance that separate base.
$negativeEditProjectId = '20000000000000000000000000000001'
$negativeEditFiles = @(
    [pscustomobject]@{
        path = 'base.txt'; classification = 'local_only'; media_type = 'text/plain'
        bytes = [System.Text.Encoding]::UTF8.GetBytes("immutable negative base`n")
    }
)
foreach ($file in $negativeEditFiles) { $file | Add-Member -NotePropertyName sha256 -NotePropertyValue (Get-ByteSha256Hex -Bytes $file.bytes) }
$negativeEditTree = Get-ProjectTreeSha256 -Files $negativeEditFiles
$negativeEditRevision = Get-ProjectRevisionSha256 -ProjectId $negativeEditProjectId -TreeSha256 $negativeEditTree
Start-ProjectImport -ProjectId $negativeEditProjectId -Name 'w2b_negative_base'
$negativeBaseFinalize = Send-ProjectFile -Name 'w2b_negative_base' -Path $negativeEditFiles[0].path -Classification $negativeEditFiles[0].classification -MediaType $negativeEditFiles[0].media_type -Bytes $negativeEditFiles[0].bytes
Add-ProjectPredicate -Name 'w2b_negative_base:file_finalized' -Expected 'separate exact negative-overlay base file' -Passed ($negativeBaseFinalize.accepted -eq $true) -Actual $negativeBaseFinalize
$negativeBaseCommit = Send-ProjectCommand -Command 'project.import_commit' -Method 'project.import_commit' -Name 'w2b_negative_base:commit'
Add-ProjectPredicate -Name 'w2b_negative_base:committed' -Expected 'separate negative-overlay base committed once' -Passed (
    $negativeBaseCommit.accepted -eq $true -and $negativeBaseCommit.revision_sha256 -eq "sha256:$negativeEditRevision"
) -Actual $negativeBaseCommit
$negativeBaseInspect = Send-ProjectCommand -Command "project.inspect $negativeEditProjectId" -Method 'project.inspect' -Name 'w2b_negative_base:inspect'
Assert-ExactProjectRevision -Result $negativeBaseInspect -Files $negativeEditFiles -ProjectId $negativeEditProjectId -TreeSha256 $negativeEditTree -RevisionSha256 $negativeEditRevision -Name 'w2b_negative_base'
$negativeBaseJson = $negativeBaseInspect | ConvertTo-Json -Compress -Depth 8

$negativeReplacement = [System.Text.Encoding]::UTF8.GetBytes("substituted`n")
Start-ProjectEdit -ProjectId $negativeEditProjectId -BaseRevisionSha256 $negativeEditRevision -Name 'w2b_wrong_hash' | Out-Null
$wrongEditFinalize = Send-ProjectEditFile -Name 'w2b_wrong_hash' -Path 'base.txt' -Classification local_only -MediaType text/plain -Bytes $negativeReplacement -DeclaredSha256 ('0' * 64)
Assert-ProjectEditDenied -Result $wrongEditFinalize -Reason 'project_edit_file_hash_mismatch' -Name 'w2b_wrong_hash'
$afterWrongHash = Send-ProjectCommand -Command "project.inspect $negativeEditProjectId" -Method 'project.inspect' -Name 'w2b_wrong_hash:inspect'
Add-ProjectPredicate -Name 'w2b_wrong_hash:base_unchanged' -Expected 'wrong hash cannot change stored base' -Passed (($afterWrongHash | ConvertTo-Json -Compress -Depth 8) -ceq $negativeBaseJson) -Actual $afterWrongHash

Start-ProjectEdit -ProjectId $negativeEditProjectId -BaseRevisionSha256 $negativeEditRevision -Name 'w2b_malformed_chunk' | Out-Null
$negativePathBase64 = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes('base.txt'))
$malformedBegin = Send-ProjectCommand -Command "project.edit_file_begin $negativePathBase64 local_only text/plain 1 sha256:$oneByteHash" -Method 'project.edit_file_begin' -Name 'w2b_malformed_chunk:file_begin'
Add-ProjectPredicate -Name 'w2b_malformed_chunk:file_started' -Expected 'malformed case opens RAM-only file first' -Passed ($malformedBegin.accepted -eq $true) -Actual $malformedBegin
$malformedChunk = Send-ProjectCommand -Command 'project.edit_chunk %%%' -Method 'project.edit_chunk' -Name 'w2b_malformed_chunk:chunk'
Assert-ProjectEditDenied -Result $malformedChunk -Reason 'project_edit_chunk_invalid' -Name 'w2b_malformed_chunk'
$afterMalformed = Send-ProjectCommand -Command "project.inspect $negativeEditProjectId" -Method 'project.inspect' -Name 'w2b_malformed_chunk:inspect'
Add-ProjectPredicate -Name 'w2b_malformed_chunk:base_unchanged' -Expected 'malformed chunk cannot change stored base' -Passed (($afterMalformed | ConvertTo-Json -Compress -Depth 8) -ceq $negativeBaseJson) -Actual $afterMalformed

Start-ProjectEdit -ProjectId $negativeEditProjectId -BaseRevisionSha256 $negativeEditRevision -Name 'w2b_invalid_delete' | Out-Null
$missingDeletePath = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes('missing.txt'))
$invalidDelete = Send-ProjectCommand -Command "project.edit_delete $missingDeletePath" -Method 'project.edit_delete' -Name 'w2b_invalid_delete:delete'
Assert-ProjectEditDenied -Result $invalidDelete -Reason 'project_edit_delete_missing' -Name 'w2b_invalid_delete'
$afterInvalidDelete = Send-ProjectCommand -Command "project.inspect $negativeEditProjectId" -Method 'project.inspect' -Name 'w2b_invalid_delete:inspect'
Add-ProjectPredicate -Name 'w2b_invalid_delete:base_unchanged' -Expected 'invalid delete cannot change stored base' -Passed (($afterInvalidDelete | ConvertTo-Json -Compress -Depth 8) -ceq $negativeBaseJson) -Actual $afterInvalidDelete

Start-ProjectEdit -ProjectId $negativeEditProjectId -BaseRevisionSha256 $negativeEditRevision -Name 'w2b_no_op' | Out-Null
$noOpFinalize = Send-ProjectEditFile -Name 'w2b_no_op' -Path 'base.txt' -Classification local_only -MediaType text/plain -Bytes $negativeEditFiles[0].bytes
Assert-ProjectEditDenied -Result $noOpFinalize -Reason 'project_edit_diff_empty' -Name 'w2b_no_op'
$afterNoOp = Send-ProjectCommand -Command "project.inspect $negativeEditProjectId" -Method 'project.inspect' -Name 'w2b_no_op:inspect'
Add-ProjectPredicate -Name 'w2b_no_op:base_unchanged' -Expected 'no-op cannot create a revision' -Passed (($afterNoOp | ConvertTo-Json -Compress -Depth 8) -ceq $negativeBaseJson) -Actual $afterNoOp

Start-ProjectEdit -ProjectId $negativeEditProjectId -BaseRevisionSha256 $negativeEditRevision -Name 'w2b_collision' | Out-Null
$collisionFinalize = Send-ProjectEditFile -Name 'w2b_collision' -Path 'BASE.TXT' -Classification local_only -MediaType text/plain -Bytes $negativeReplacement
Assert-ProjectEditDenied -Result $collisionFinalize -Reason 'project_path_collision' -Name 'w2b_collision'
$afterEditCollision = Send-ProjectCommand -Command "project.inspect $negativeEditProjectId" -Method 'project.inspect' -Name 'w2b_collision:inspect'
Add-ProjectPredicate -Name 'w2b_collision:base_unchanged' -Expected 'case collision cannot change stored base' -Passed (($afterEditCollision | ConvertTo-Json -Compress -Depth 8) -ceq $negativeBaseJson) -Actual $afterEditCollision

# A stale overlay is real only if another valid transaction advances its base.
Start-ProjectEdit -ProjectId $negativeEditProjectId -BaseRevisionSha256 $negativeEditRevision -Name 'w2b_stale' | Out-Null
$staleOverlayFinalize = Send-ProjectEditFile -Name 'w2b_stale' -Path 'base.txt' -Classification local_only -MediaType text/plain -Bytes $negativeReplacement
Add-ProjectPredicate -Name 'w2b_stale:overlay_staged' -Expected 'candidate edit staged only in RAM' -Passed ($staleOverlayFinalize.accepted -eq $true) -Actual $staleOverlayFinalize
$advancedNegativeFiles = @(
    [pscustomobject]@{
        path = 'base.txt'; classification = 'local_only'; media_type = 'text/plain'
        bytes = [System.Text.Encoding]::UTF8.GetBytes("legitimate concurrent revision`n")
    }
)
foreach ($file in $advancedNegativeFiles) { $file | Add-Member -NotePropertyName sha256 -NotePropertyValue (Get-ByteSha256Hex -Bytes $file.bytes) }
$advancedNegativeTree = Get-ProjectTreeSha256 -Files $advancedNegativeFiles
$advancedNegativeRevision = Get-ProjectRevisionSha256 -ProjectId $negativeEditProjectId -TreeSha256 $advancedNegativeTree -ParentRevisionSha256 $negativeEditRevision
Start-ProjectImport -ProjectId $negativeEditProjectId -Name 'w2b_stale_control'
$advancedFinalize = Send-ProjectFile -Name 'w2b_stale_control' -Path 'base.txt' -Classification local_only -MediaType text/plain -Bytes $advancedNegativeFiles[0].bytes
Add-ProjectPredicate -Name 'w2b_stale_control:file_finalized' -Expected 'independent valid base advance staged' -Passed ($advancedFinalize.accepted -eq $true) -Actual $advancedFinalize
$advancedCommit = Send-ProjectCommand -Command 'project.import_commit' -Method 'project.import_commit' -Name 'w2b_stale_control:commit'
Add-ProjectPredicate -Name 'w2b_stale_control:committed' -Expected 'independent valid transaction advances latest revision' -Passed ($advancedCommit.revision_sha256 -eq "sha256:$advancedNegativeRevision") -Actual $advancedCommit
$staleCommit = Send-ProjectCommand -Command 'project.edit_commit' -Method 'project.edit_commit' -Name 'w2b_stale:commit'
Assert-ProjectEditDenied -Result $staleCommit -Reason 'project_edit_base_stale' -Name 'w2b_stale'
$afterStale = Send-ProjectCommand -Command "project.inspect $negativeEditProjectId" -Method 'project.inspect' -Name 'w2b_stale:inspect'
Assert-ExactProjectRevision -Result $afterStale -Files $advancedNegativeFiles -ProjectId $negativeEditProjectId -TreeSha256 $advancedNegativeTree -RevisionSha256 $advancedNegativeRevision -ParentRevisionSha256 $negativeEditRevision -Name 'w2b_stale_latest'

# W3 uses a separate immutable source project. The exact Cargo.lock is source
# evidence, while the dependency bytes below remain a distinct inert bundle.
$dependencyProjectId = '30000000000000000000000000000001'
$dependencyFiles = @(
    [pscustomobject]@{
        path = 'Cargo.lock'; classification = 'local_only'; media_type = 'text/plain'
        bytes = [System.Text.Encoding]::UTF8.GetBytes("# This file is automatically @generated by Cargo.`n# It is not intended for manual editing.`nversion = 3`n`n[[package]]`nname = `"genesis-dependency-root`"`nversion = `"0.1.0`"`ndependencies = [`n `"raios-local-helper`",`n]`n`n[[package]]`nname = `"raios-local-helper`"`nversion = `"0.1.0`"`n")
    },
    [pscustomobject]@{
        path = 'Cargo.toml'; classification = 'local_only'; media_type = 'text/toml'
        bytes = [System.Text.Encoding]::UTF8.GetBytes("[package]`nname = `"genesis-dependency-root`"`nversion = `"0.1.0`"`nedition = `"2021`"`n`n[dependencies]`nraios-local-helper = `"=0.1.0`"`n")
    }
)
foreach ($file in $dependencyFiles) { $file | Add-Member -NotePropertyName sha256 -NotePropertyValue (Get-ByteSha256Hex -Bytes $file.bytes) }
$dependencyTreeSha256 = Get-ProjectTreeSha256 -Files $dependencyFiles
$dependencyRevisionSha256 = Get-ProjectRevisionSha256 -ProjectId $dependencyProjectId -TreeSha256 $dependencyTreeSha256
$cargoLockSha256 = $dependencyFiles[0].sha256

Start-ProjectImport -ProjectId $dependencyProjectId -Name 'w3_source'
foreach ($file in $dependencyFiles) {
    $finalize = Send-ProjectFile -Name "w3_source_$($file.path)" -Path $file.path -Classification $file.classification -MediaType $file.media_type -Bytes $file.bytes
    Add-ProjectPredicate -Name "w3_source_$($file.path)`:finalized" -Expected 'exact W3 source file staged before manifest-last commit' -Passed (
        $finalize.status -eq 'accepted' -and $finalize.reason -eq 'project_import_file_finalized' -and $finalize.accepted -eq $true
    ) -Actual $finalize
}
$dependencySourceCommit = Send-ProjectCommand -Command 'project.import_commit' -Method 'project.import_commit' -Name 'w3_source:commit'
Add-ProjectPredicate -Name 'w3_source:committed' -Expected 'separate exact Cargo.lock source revision committed on boot 1' -Passed (
    $dependencySourceCommit.status -eq 'accepted' -and $dependencySourceCommit.reason -eq 'project_revision_committed' -and
    $dependencySourceCommit.project_id -eq $dependencyProjectId -and
    $dependencySourceCommit.tree_sha256 -eq "sha256:$dependencyTreeSha256" -and
    $dependencySourceCommit.revision_sha256 -eq "sha256:$dependencyRevisionSha256"
) -Actual $dependencySourceCommit
$dependencySourceInspect = Send-ProjectCommand -Command "project.inspect $dependencyProjectId" -Method 'project.inspect' -Name 'w3_source:inspect'
Assert-ExactProjectRevision -Result $dependencySourceInspect -Files $dependencyFiles -ProjectId $dependencyProjectId -TreeSha256 $dependencyTreeSha256 -RevisionSha256 $dependencyRevisionSha256 -Name 'w3_source'
$dependencySourceJson = $dependencySourceInspect | ConvertTo-Json -Compress -Depth 8

# A second real source revision intentionally lacks Cargo.lock. W3 must deny it
# without confusing "missing lock" with a malformed synthetic request.
$dependencyNoLockProjectId = '30000000000000000000000000000002'
$dependencyNoLockFiles = @(
    [pscustomobject]@{
        path = 'Cargo.toml'; classification = 'local_only'; media_type = 'text/toml'
        bytes = [System.Text.Encoding]::UTF8.GetBytes("[package]`nname = `"genesis-unlocked-root`"`nversion = `"0.1.0`"`nedition = `"2021`"`n")
    }
)
foreach ($file in $dependencyNoLockFiles) { $file | Add-Member -NotePropertyName sha256 -NotePropertyValue (Get-ByteSha256Hex -Bytes $file.bytes) }
$dependencyNoLockTreeSha256 = Get-ProjectTreeSha256 -Files $dependencyNoLockFiles
$dependencyNoLockRevisionSha256 = Get-ProjectRevisionSha256 -ProjectId $dependencyNoLockProjectId -TreeSha256 $dependencyNoLockTreeSha256
Start-ProjectImport -ProjectId $dependencyNoLockProjectId -Name 'w3_no_lock_source'
$noLockFinalize = Send-ProjectFile -Name 'w3_no_lock_source' -Path $dependencyNoLockFiles[0].path -Classification $dependencyNoLockFiles[0].classification -MediaType $dependencyNoLockFiles[0].media_type -Bytes $dependencyNoLockFiles[0].bytes
Add-ProjectPredicate -Name 'w3_no_lock_source:file_finalized' -Expected 'real no-lock source file staged' -Passed ($noLockFinalize.accepted -eq $true) -Actual $noLockFinalize
$noLockCommit = Send-ProjectCommand -Command 'project.import_commit' -Method 'project.import_commit' -Name 'w3_no_lock_source:commit'
Add-ProjectPredicate -Name 'w3_no_lock_source:committed' -Expected 'separate no-lock revision exists for exact W3 denial' -Passed ($noLockCommit.revision_sha256 -eq "sha256:$dependencyNoLockRevisionSha256") -Actual $noLockCommit

# The positive inert package uses several files and a >24 KiB Rust source so
# project.dependency_chunk must prove ordered multi-chunk assembly. build.rs is
# deliberately harmless input, but its mere presence must never trigger it.
$dependencyPackageFiles = @(
    [pscustomobject]@{
        path = 'LICENSE'; media_type = 'text/plain'
        bytes = [System.Text.Encoding]::UTF8.GetBytes("Permission is hereby granted to use this inert raiOS test package.`n")
    },
    [pscustomobject]@{
        path = 'build.rs'; media_type = 'text/rust'
        bytes = [System.Text.Encoding]::UTF8.GetBytes("fn main() { println!(`"cargo:rerun-if-changed=build.rs`"`); }`n")
    },
    [pscustomobject]@{
        path = 'src/lib.rs'; media_type = 'text/rust'
        bytes = [System.Text.Encoding]::UTF8.GetBytes("pub fn answer() -> u32 { 42 }`n" + ((0..749 | ForEach-Object { "// inert ordered chunk {0:D4} padding`n" -f $_ }) -join ''))
    }
)
foreach ($file in $dependencyPackageFiles) { $file | Add-Member -NotePropertyName sha256 -NotePropertyValue (Get-ByteSha256Hex -Bytes $file.bytes) }
Add-ProjectPredicate -Name 'w3_fixture:multi_chunk_source' -Expected 'bounded inert Rust source exceeds 24 KiB' -Passed (
    $dependencyPackageFiles[2].bytes.Length -gt (24 * 1024) -and $dependencyPackageFiles[2].bytes.Length -le (32 * 1024)
) -Actual $dependencyPackageFiles[2].bytes.Length
# The serial console command buffer is 4096 bytes, narrower than the 24 KiB
# dependency chunk contract. Keep raw chunks below the base64-expanded limit.
$dependencyChunkBytes = 2048
foreach ($file in $dependencyPackageFiles) {
    $chunks = @()
    for ($offset = 0; $offset -lt $file.bytes.Length; $offset += $dependencyChunkBytes) {
        $count = [Math]::Min($dependencyChunkBytes, $file.bytes.Length - $offset)
        $bytes = [byte[]]::new($count)
        [Array]::Copy($file.bytes, $offset, $bytes, 0, $count)
        $chunks += [pscustomobject]@{
            bytes = $bytes
            sha256 = Get-ByteSha256Hex -Bytes $bytes
        }
    }
    $file | Add-Member -NotePropertyName chunks -NotePropertyValue $chunks
}
$dependencyChunkHashes = @($dependencyPackageFiles | ForEach-Object { $_.chunks } | ForEach-Object { $_.sha256 })
Add-ProjectPredicate -Name 'w3_fixture:ordered_unique_chunks' -Expected 'multi-file package requires multiple uniquely hashed ordered chunks within the 64-chunk bound' -Passed (
    $dependencyPackageFiles[2].chunks.Count -gt 1 -and
    $dependencyChunkHashes.Count -le 64 -and
    @($dependencyChunkHashes | Sort-Object -Unique).Count -eq $dependencyChunkHashes.Count
) -Actual $dependencyChunkHashes.Count
$dependencyPackageName = 'raios-local-helper'
$dependencyPackageVersion = '0.1.0'
$dependencyPackageOrigin = 'owner-local://serial/raios-local-helper/0.1.0'
$dependencyLicenseExpression = 'MIT'
$dependencyLicensePath = 'LICENSE'
$dependencyLicenseSha256 = $dependencyPackageFiles[0].sha256
$dependencyPackageTreeSha256 = Get-DependencyTreeSha256 -Files $dependencyPackageFiles
$dependencyBundleSha256 = Get-DependencyBundleSha256 `
    -ProjectId $dependencyProjectId `
    -RevisionSha256 $dependencyRevisionSha256 `
    -CargoLockSha256 $cargoLockSha256 `
    -Name $dependencyPackageName `
    -Version $dependencyPackageVersion `
    -Origin $dependencyPackageOrigin `
    -LicenseExpression $dependencyLicenseExpression `
    -LicensePath $dependencyLicensePath `
    -LicenseSha256 $dependencyLicenseSha256 `
    -TreeSha256 $dependencyPackageTreeSha256

$firstBootLog = $SerialLog
Close-SerialTcpConnection
if (-not $QemuPid) { throw 'project-workspace profile cannot reboot without the first QEMU process' }
$firstQemuPid = $QemuPid
Stop-Process -Id $firstQemuPid -Force -ErrorAction Stop
if ($script:QemuProcess) { try { $script:QemuProcess.WaitForExit(5000) | Out-Null } catch {} }
if (Get-Process -Id $firstQemuPid -ErrorAction SilentlyContinue) { throw 'project-workspace first QEMU process did not stop before reboot' }

$rebootLog = Join-Path $RunDir 'serial-project-workspace-reboot.log'
$rebootParams = $runParams.Clone()
$rebootParams.StopExisting = $false
$rebootParams.SerialLog = $rebootLog
$rebootOutput = & $RunScript @rebootParams
$SerialLog = $rebootLog
$QemuPid = $null
foreach ($line in $rebootOutput) { if ($line -match '^qemu pid:\s*(\d+)') { $QemuPid = [int]$Matches[1] } }
if (-not $QemuPid) { throw 'project-workspace reboot did not return a QEMU pid' }
try { $script:QemuProcess = Get-Process -Id $QemuPid -ErrorAction Stop } catch { $script:QemuProcess = $null }

Assert-LogContains -Name 'project-workspace:reboot_serial_console_ready' -Needle 'SERIAL CONSOLE READY' -TimeoutSeconds $TimeoutSeconds
$rebootInspect = Send-ProjectCommand -Command "project.inspect $projectId" -Method 'project.inspect' -Name 'positive:inspect_reboot'
Assert-ExactProjectRevision -Result $rebootInspect -Files $files -ProjectId $projectId -TreeSha256 $treeSha256 -RevisionSha256 $revisionSha256 -Name 'reboot'
Assert-ProjectReadAndSearch -ProjectId $projectId -RevisionSha256 $revisionSha256 -SourceFile $files[1] -Name 'reboot'
Assert-ProjectQueryDenials -ProjectId $projectId -SourceFile $files[1] -Name 'reboot'
$rebootRevisionJson = $rebootInspect | ConvertTo-Json -Compress -Depth 8
Add-ProjectPredicate -Name 'reboot:byte_identical_revision_facts' -Expected 'project/tree/revision/file facts byte-identical across two boots' -Passed ($rebootRevisionJson -ceq $firstRevisionJson) -Actual $(if ($rebootRevisionJson -ceq $firstRevisionJson) { $revisionSha256 } else { 'response_mismatch' })

$dependencySourceRebootInspect = Send-ProjectCommand -Command "project.inspect $dependencyProjectId" -Method 'project.inspect' -Name 'w3_source:inspect_reboot'
Assert-ExactProjectRevision -Result $dependencySourceRebootInspect -Files $dependencyFiles -ProjectId $dependencyProjectId -TreeSha256 $dependencyTreeSha256 -RevisionSha256 $dependencyRevisionSha256 -Name 'w3_source_reboot'
$dependencySourceRebootJson = $dependencySourceRebootInspect | ConvertTo-Json -Compress -Depth 8
Add-ProjectPredicate -Name 'w3_source:boot2_revision_unchanged' -Expected 'exact Cargo.lock source revision byte-identical and unchanged on boot 2' -Passed (
    $dependencySourceRebootJson -ceq $dependencySourceJson
) -Actual $(if ($dependencySourceRebootJson -ceq $dependencySourceJson) { $dependencyRevisionSha256 } else { 'response_mismatch' })

$dependencyNoLockInspect = Send-ProjectCommand -Command "project.inspect $dependencyNoLockProjectId" -Method 'project.inspect' -Name 'w3_no_lock_source:inspect_reboot'
Assert-ExactProjectRevision -Result $dependencyNoLockInspect -Files $dependencyNoLockFiles -ProjectId $dependencyNoLockProjectId -TreeSha256 $dependencyNoLockTreeSha256 -RevisionSha256 $dependencyNoLockRevisionSha256 -Name 'w3_no_lock_source_reboot'

# W3 begin denials bind to real stored project facts and write nothing.
$wrongDependencyProject = Start-DependencyImport `
    -ProjectId '40000000000000000000000000000001' -RevisionSha256 $dependencyRevisionSha256 `
    -PackageName $dependencyPackageName -PackageVersion $dependencyPackageVersion -Origin $dependencyPackageOrigin `
    -LicenseExpression $dependencyLicenseExpression -LicensePath $dependencyLicensePath -LicenseSha256 $dependencyLicenseSha256 `
    -Name 'w3_wrong_project'
Assert-DependencyDenied -Result $wrongDependencyProject -Reason 'project_not_found' -Name 'w3_wrong_project'

$wrongDependencyRevision = Start-DependencyImport `
    -ProjectId $dependencyProjectId -RevisionSha256 ('1' * 64) `
    -PackageName $dependencyPackageName -PackageVersion $dependencyPackageVersion -Origin $dependencyPackageOrigin `
    -LicenseExpression $dependencyLicenseExpression -LicensePath $dependencyLicensePath -LicenseSha256 $dependencyLicenseSha256 `
    -Name 'w3_wrong_revision'
Assert-DependencyDenied -Result $wrongDependencyRevision -Reason 'dependency_project_revision_stale' -Name 'w3_wrong_revision'

$missingCargoLock = Start-DependencyImport `
    -ProjectId $dependencyNoLockProjectId -RevisionSha256 $dependencyNoLockRevisionSha256 `
    -PackageName $dependencyPackageName -PackageVersion $dependencyPackageVersion -Origin $dependencyPackageOrigin `
    -LicenseExpression $dependencyLicenseExpression -LicensePath $dependencyLicensePath -LicenseSha256 $dependencyLicenseSha256 `
    -Name 'w3_missing_cargo_lock'
Assert-DependencyDenied -Result $missingCargoLock -Reason 'dependency_cargo_lock_missing' -Name 'w3_missing_cargo_lock'

$invalidMetadata = Start-DependencyImport `
    -ProjectId $dependencyProjectId -RevisionSha256 $dependencyRevisionSha256 `
    -PackageName "bad`nname" -PackageVersion $dependencyPackageVersion -Origin $dependencyPackageOrigin `
    -LicenseExpression $dependencyLicenseExpression -LicensePath $dependencyLicensePath -LicenseSha256 $dependencyLicenseSha256 `
    -Name 'w3_invalid_metadata'
Assert-DependencyDenied -Result $invalidMetadata -Reason 'dependency_metadata_invalid' -Name 'w3_invalid_metadata'

$invalidLicensePath = Start-DependencyImport `
    -ProjectId $dependencyProjectId -RevisionSha256 $dependencyRevisionSha256 `
    -PackageName $dependencyPackageName -PackageVersion $dependencyPackageVersion -Origin $dependencyPackageOrigin `
    -LicenseExpression $dependencyLicenseExpression -LicensePath '../LICENSE' -LicenseSha256 $dependencyLicenseSha256 `
    -Name 'w3_invalid_license_path'
Assert-DependencyDenied -Result $invalidLicensePath -Reason 'dependency_path_invalid' -Name 'w3_invalid_license_path'

$invalidLicenseHash = Start-DependencyImport `
    -ProjectId $dependencyProjectId -RevisionSha256 $dependencyRevisionSha256 `
    -PackageName $dependencyPackageName -PackageVersion $dependencyPackageVersion -Origin $dependencyPackageOrigin `
    -LicenseExpression $dependencyLicenseExpression -LicensePath $dependencyLicensePath -LicenseSha256 'xyz' `
    -Name 'w3_invalid_license_hash'
Assert-DependencyDenied -Result $invalidLicenseHash -Reason 'dependency_license_sha256_invalid' -Name 'w3_invalid_license_hash'

# Positive package: every valid chunk is a durable orphan until the exact
# manifest is committed last. Inspection before commit must still be empty.
$dependencyBegin = Start-DependencyImport `
    -ProjectId $dependencyProjectId -RevisionSha256 $dependencyRevisionSha256 `
    -PackageName $dependencyPackageName -PackageVersion $dependencyPackageVersion -Origin $dependencyPackageOrigin `
    -LicenseExpression $dependencyLicenseExpression -LicensePath $dependencyLicensePath -LicenseSha256 $dependencyLicenseSha256 `
    -Name 'w3_positive'
Assert-DependencyBeginAccepted -Result $dependencyBegin `
    -ProjectId $dependencyProjectId -RevisionSha256 $dependencyRevisionSha256 -CargoLockSha256 $cargoLockSha256 `
    -PackageName $dependencyPackageName -PackageVersion $dependencyPackageVersion `
    -LicensePath $dependencyLicensePath -LicenseSha256 $dependencyLicenseSha256 -Name 'w3_positive'
foreach ($file in $dependencyPackageFiles) {
    Send-DependencyFile -File $file -Name "w3_positive_$($file.path)" | Out-Null
}
$dependencyBeforeCommit = Assert-NoDependencyBundles `
    -ProjectId $dependencyProjectId -RevisionSha256 $dependencyRevisionSha256 -CargoLockSha256 $cargoLockSha256 `
    -Name 'w3_positive_before_commit'

$dependencyCommit = Send-ProjectCommand -Command 'project.dependency_commit' -Method 'project.dependency_commit' -Name 'w3_positive:commit'
Add-ProjectPredicate -Name 'w3_positive:manifest_last_commit' -Expected 'manifest becomes visible only after all exact chunk and whole-file hashes verify' -Passed (
    $dependencyCommit.status -eq 'accepted' -and $dependencyCommit.reason -eq 'dependency_bundle_committed' -and
    $dependencyCommit.accepted -eq $true -and $dependencyCommit.rejected -eq $false -and
    $dependencyCommit.project_id -eq $dependencyProjectId -and
    $dependencyCommit.project_revision_sha256 -eq "sha256:$dependencyRevisionSha256" -and
    $dependencyCommit.cargo_lock_sha256 -eq "sha256:$cargoLockSha256" -and
    [int]$dependencyCommit.file_count -eq $dependencyPackageFiles.Count -and
    [int]$dependencyCommit.chunk_count -eq $dependencyChunkHashes.Count -and
    $dependencyCommit.bundle_visible -eq $true -and
    $dependencyCommit.build_script_present -eq $true
) -Actual $dependencyCommit
Assert-ExactDependencyBundleFields -Actual $dependencyCommit -Files $dependencyPackageFiles -Name 'w3_positive_commit'
Assert-DependencyPosture -Result $dependencyCommit -StorageWriteAttempted $true -WritesPersistentState $true -Name 'w3_positive_commit'

$dependencyInspect = Send-ProjectCommand -Command "project.dependencies $dependencyProjectId sha256:$dependencyRevisionSha256" -Method 'project.dependencies' -Name 'w3_positive:inspect'
Assert-ExactDependencyInspection -Result $dependencyInspect -Name 'w3_positive_inspect'
$dependencyInspectJson = $dependencyInspect | ConvertTo-Json -Compress -Depth 12

# Re-importing the byte-identical package proves content-addressed idempotence:
# existing chunks and the existing manifest verify, but neither claims a write.
$idempotentDependencyBegin = Start-DependencyImport `
    -ProjectId $dependencyProjectId -RevisionSha256 $dependencyRevisionSha256 `
    -PackageName $dependencyPackageName -PackageVersion $dependencyPackageVersion -Origin $dependencyPackageOrigin `
    -LicenseExpression $dependencyLicenseExpression -LicensePath $dependencyLicensePath -LicenseSha256 $dependencyLicenseSha256 `
    -Name 'w3_idempotent'
Assert-DependencyBeginAccepted -Result $idempotentDependencyBegin `
    -ProjectId $dependencyProjectId -RevisionSha256 $dependencyRevisionSha256 -CargoLockSha256 $cargoLockSha256 `
    -PackageName $dependencyPackageName -PackageVersion $dependencyPackageVersion `
    -LicensePath $dependencyLicensePath -LicenseSha256 $dependencyLicenseSha256 -Name 'w3_idempotent'
foreach ($file in $dependencyPackageFiles) {
    Send-DependencyFile -File $file -Name "w3_idempotent_$($file.path)" `
        -ExpectedChunkStorageWrite $false -ExpectedChunkPersistentWrite $false | Out-Null
}
$idempotentDependencyCommit = Send-ProjectCommand -Command 'project.dependency_commit' -Method 'project.dependency_commit' -Name 'w3_idempotent:commit'
Add-ProjectPredicate -Name 'w3_idempotent:manifest_verified_without_write' -Expected 'identical existing manifest accepted and visible without claiming a storage write' -Passed (
    $idempotentDependencyCommit.status -eq 'accepted' -and
    $idempotentDependencyCommit.reason -eq 'dependency_bundle_already_present' -and
    $idempotentDependencyCommit.accepted -eq $true -and
    $idempotentDependencyCommit.bundle_visible -eq $true -and
    $idempotentDependencyCommit.bundle_sha256 -eq "sha256:$dependencyBundleSha256"
) -Actual $idempotentDependencyCommit
Assert-ExactDependencyBundleFields -Actual $idempotentDependencyCommit -Files $dependencyPackageFiles -Name 'w3_idempotent_commit'
Assert-DependencyPosture -Result $idempotentDependencyCommit -StorageWriteAttempted $false -WritesPersistentState $false -Name 'w3_idempotent_commit'
$dependencyAfterIdempotent = Send-ProjectCommand -Command "project.dependencies $dependencyProjectId sha256:$dependencyRevisionSha256" -Method 'project.dependencies' -Name 'w3_idempotent:inspect'
Assert-ExactDependencyInspection -Result $dependencyAfterIdempotent -Name 'w3_idempotent_inspect'
$dependencyAfterIdempotentJson = $dependencyAfterIdempotent | ConvertTo-Json -Compress -Depth 12
Add-ProjectPredicate -Name 'w3_idempotent:inspection_unchanged' -Expected 'idempotent re-import leaves exact inspection byte-identical' -Passed (
    $dependencyAfterIdempotentJson -ceq $dependencyInspectJson
) -Actual $(if ($dependencyAfterIdempotentJson -ceq $dependencyInspectJson) { $dependencyBundleSha256 } else { 'response_mismatch' })

# Malformed and overflow requests discard only their RAM session.
$malformedDependencyBegin = Start-DependencyImport `
    -ProjectId $dependencyProjectId -RevisionSha256 $dependencyRevisionSha256 `
    -PackageName 'negative-malformed' -PackageVersion '0.1.0' -Origin $dependencyPackageOrigin `
    -LicenseExpression 'MIT' -LicensePath 'LICENSE' -LicenseSha256 $dependencyLicenseSha256 `
    -Name 'w3_malformed_chunk'
Assert-DependencyBeginAccepted -Result $malformedDependencyBegin `
    -ProjectId $dependencyProjectId -RevisionSha256 $dependencyRevisionSha256 -CargoLockSha256 $cargoLockSha256 `
    -PackageName 'negative-malformed' -PackageVersion '0.1.0' -LicensePath 'LICENSE' -LicenseSha256 $dependencyLicenseSha256 `
    -Name 'w3_malformed_chunk'
$malformedPath = ConvertTo-Base64Text -Value 'malformed.bin'
$malformedFileBegin = Send-ProjectCommand -Command "project.dependency_file_begin $malformedPath application/octet-stream 1 sha256:$oneByteHash" -Method 'project.dependency_file_begin' -Name 'w3_malformed_chunk:file_begin'
Add-ProjectPredicate -Name 'w3_malformed_chunk:file_started' -Expected 'malformed chunk case has one active bounded file' -Passed ($malformedFileBegin.accepted -eq $true) -Actual $malformedFileBegin
$malformedChunk = Send-ProjectCommand -Command 'project.dependency_chunk' -Method 'project.dependency_chunk' -Name 'w3_malformed_chunk:chunk'
Assert-DependencyDenied -Result $malformedChunk -Reason 'dependency_chunk_malformed' -Name 'w3_malformed_chunk'

$overflowDependencyBegin = Start-DependencyImport `
    -ProjectId $dependencyProjectId -RevisionSha256 $dependencyRevisionSha256 `
    -PackageName 'negative-overflow' -PackageVersion '0.1.0' -Origin $dependencyPackageOrigin `
    -LicenseExpression 'MIT' -LicensePath 'LICENSE' -LicenseSha256 $dependencyLicenseSha256 `
    -Name 'w3_file_overflow'
Assert-DependencyBeginAccepted -Result $overflowDependencyBegin `
    -ProjectId $dependencyProjectId -RevisionSha256 $dependencyRevisionSha256 -CargoLockSha256 $cargoLockSha256 `
    -PackageName 'negative-overflow' -PackageVersion '0.1.0' -LicensePath 'LICENSE' -LicenseSha256 $dependencyLicenseSha256 `
    -Name 'w3_file_overflow'
$overflowPath = ConvertTo-Base64Text -Value 'overflow.bin'
$overflowFile = Send-ProjectCommand -Command "project.dependency_file_begin $overflowPath application/octet-stream 524289 sha256:$('2' * 64)" -Method 'project.dependency_file_begin' -Name 'w3_file_overflow:file_begin'
Assert-DependencyDenied -Result $overflowFile -Reason 'dependency_file_too_large' -Name 'w3_file_overflow'

# A correct persisted chunk followed by a wrong whole-file hash must remain an
# orphan and must not alter the already visible exact bundle.
$hashMismatchBytes = [System.Text.Encoding]::UTF8.GetBytes('unique negative hash mismatch bytes')
$hashMismatchActualSha = Get-ByteSha256Hex -Bytes $hashMismatchBytes
$hashMismatchBegin = Start-DependencyImport `
    -ProjectId $dependencyProjectId -RevisionSha256 $dependencyRevisionSha256 `
    -PackageName 'negative-hash' -PackageVersion '0.1.0' -Origin $dependencyPackageOrigin `
    -LicenseExpression 'MIT' -LicensePath 'hash.bin' -LicenseSha256 ('3' * 64) `
    -Name 'w3_file_hash_mismatch'
Assert-DependencyBeginAccepted -Result $hashMismatchBegin `
    -ProjectId $dependencyProjectId -RevisionSha256 $dependencyRevisionSha256 -CargoLockSha256 $cargoLockSha256 `
    -PackageName 'negative-hash' -PackageVersion '0.1.0' -LicensePath 'hash.bin' -LicenseSha256 ('3' * 64) `
    -Name 'w3_file_hash_mismatch'
$hashMismatchPath = ConvertTo-Base64Text -Value 'hash.bin'
$hashMismatchFileBegin = Send-ProjectCommand -Command "project.dependency_file_begin $hashMismatchPath application/octet-stream $($hashMismatchBytes.Length) sha256:$('4' * 64)" -Method 'project.dependency_file_begin' -Name 'w3_file_hash_mismatch:file_begin'
Add-ProjectPredicate -Name 'w3_file_hash_mismatch:file_started' -Expected 'wrong declared whole hash accepted only as an unverified pending claim' -Passed ($hashMismatchFileBegin.accepted -eq $true) -Actual $hashMismatchFileBegin
$hashMismatchChunk = Send-ProjectCommand -Command "project.dependency_chunk $([Convert]::ToBase64String($hashMismatchBytes))" -Method 'project.dependency_chunk' -Name 'w3_file_hash_mismatch:chunk'
Add-ProjectPredicate -Name 'w3_file_hash_mismatch:orphan_written' -Expected 'valid unique chunk persists truthfully while bundle remains invisible' -Passed (
    $hashMismatchChunk.accepted -eq $true -and $hashMismatchChunk.reason -eq 'dependency_chunk_persisted' -and
    $hashMismatchChunk.chunk_sha256 -eq "sha256:$hashMismatchActualSha" -and
    $hashMismatchChunk.chunk_persisted -eq $true -and $hashMismatchChunk.bundle_visible -eq $false
) -Actual $hashMismatchChunk
Assert-DependencyPosture -Result $hashMismatchChunk -StorageWriteAttempted $true -WritesPersistentState $true -Name 'w3_file_hash_mismatch:chunk'
$hashMismatchFinalize = Send-ProjectCommand -Command 'project.dependency_file_finalize' -Method 'project.dependency_file_finalize' -Name 'w3_file_hash_mismatch:finalize'
Assert-DependencyDenied -Result $hashMismatchFinalize -Reason 'dependency_file_hash_mismatch' -Name 'w3_file_hash_mismatch'

# A valid license file with a different declared license hash reaches the
# exact commit-time license binding and still cannot publish a manifest.
$wrongLicenseFile = New-SingleChunkDependencyFile -Path 'LICENSE' -MediaType 'text/plain' -Bytes ([System.Text.Encoding]::UTF8.GetBytes("unique wrong-license fixture`n"))
$wrongLicenseBegin = Start-DependencyImport `
    -ProjectId $dependencyProjectId -RevisionSha256 $dependencyRevisionSha256 `
    -PackageName 'negative-license' -PackageVersion '0.1.0' -Origin $dependencyPackageOrigin `
    -LicenseExpression 'MIT' -LicensePath 'LICENSE' -LicenseSha256 ('5' * 64) `
    -Name 'w3_license_hash_mismatch'
Assert-DependencyBeginAccepted -Result $wrongLicenseBegin `
    -ProjectId $dependencyProjectId -RevisionSha256 $dependencyRevisionSha256 -CargoLockSha256 $cargoLockSha256 `
    -PackageName 'negative-license' -PackageVersion '0.1.0' -LicensePath 'LICENSE' -LicenseSha256 ('5' * 64) `
    -Name 'w3_license_hash_mismatch'
Send-DependencyFile -File $wrongLicenseFile -Name 'w3_license_hash_mismatch' | Out-Null
$wrongLicenseCommit = Send-ProjectCommand -Command 'project.dependency_commit' -Method 'project.dependency_commit' -Name 'w3_license_hash_mismatch:commit'
Assert-DependencyDenied -Result $wrongLicenseCommit -Reason 'dependency_license_hash_mismatch' -Name 'w3_license_hash_mismatch'

# Case aliases persist only content-addressed orphan chunks; the sorted bundle
# validator refuses to publish an ambiguous path tree.
$caseFileA = New-SingleChunkDependencyFile -Path 'Case.rs' -MediaType 'text/rust' -Bytes ([System.Text.Encoding]::UTF8.GetBytes("// unique case A`n"))
$caseFileB = New-SingleChunkDependencyFile -Path 'case.RS' -MediaType 'text/rust' -Bytes ([System.Text.Encoding]::UTF8.GetBytes("// unique case B`n"))
$caseCollisionBegin = Start-DependencyImport `
    -ProjectId $dependencyProjectId -RevisionSha256 $dependencyRevisionSha256 `
    -PackageName 'negative-collision' -PackageVersion '0.1.0' -Origin $dependencyPackageOrigin `
    -LicenseExpression 'MIT' -LicensePath 'Case.rs' -LicenseSha256 $caseFileA.sha256 `
    -Name 'w3_case_collision'
Assert-DependencyBeginAccepted -Result $caseCollisionBegin `
    -ProjectId $dependencyProjectId -RevisionSha256 $dependencyRevisionSha256 -CargoLockSha256 $cargoLockSha256 `
    -PackageName 'negative-collision' -PackageVersion '0.1.0' -LicensePath 'Case.rs' -LicenseSha256 $caseFileA.sha256 `
    -Name 'w3_case_collision'
Send-DependencyFile -File $caseFileA -Name 'w3_case_collision_a' | Out-Null
Send-DependencyFile -File $caseFileB -Name 'w3_case_collision_b' | Out-Null
$caseCollisionCommit = Send-ProjectCommand -Command 'project.dependency_commit' -Method 'project.dependency_commit' -Name 'w3_case_collision:commit'
Assert-DependencyDenied -Result $caseCollisionCommit -Reason 'dependency_path_collision' -Name 'w3_case_collision'

$dependencyAfterDenials = Send-ProjectCommand -Command "project.dependencies $dependencyProjectId sha256:$dependencyRevisionSha256" -Method 'project.dependencies' -Name 'w3_after_denials:inspect'
Assert-ExactDependencyInspection -Result $dependencyAfterDenials -Name 'w3_after_denials'
$dependencyAfterDenialsJson = $dependencyAfterDenials | ConvertTo-Json -Compress -Depth 12
Add-ProjectPredicate -Name 'w3_after_denials:manifest_unchanged' -Expected 'all denied sessions leave the exact committed dependency inspection byte-identical' -Passed (
    $dependencyAfterDenialsJson -ceq $dependencyInspectJson
) -Actual $(if ($dependencyAfterDenialsJson -ceq $dependencyInspectJson) { $dependencyBundleSha256 } else { 'response_mismatch' })
$dependencySourceAfterW3 = Send-ProjectCommand -Command "project.inspect $dependencyProjectId" -Method 'project.inspect' -Name 'w3_source:inspect_after_dependency'
Add-ProjectPredicate -Name 'w3_source:unchanged_after_dependency' -Expected 'dependency quarantine cannot mutate the immutable source revision' -Passed (
    ($dependencySourceAfterW3 | ConvertTo-Json -Compress -Depth 8) -ceq $dependencySourceJson
) -Actual $dependencySourceAfterW3

# W2b positive child revision: one replacement, one addition, one deletion.
$childFiles = @(
    [pscustomobject]@{
        path = 'README.md'; classification = 'public'; media_type = 'text/markdown'
        bytes = [System.Text.Encoding]::UTF8.GetBytes("# Genesis workspace`n`nEdited safely by the raiOS agent.`n")
    },
    [pscustomobject]@{
        path = 'src/main.rs'; classification = 'local_only'; media_type = 'text/rust'
        bytes = [System.Text.Encoding]::UTF8.GetBytes("fn main() {`n    println!(`"hello from raiOS child`"`);`n}`n")
    }
)
foreach ($file in $childFiles) { $file | Add-Member -NotePropertyName sha256 -NotePropertyValue (Get-ByteSha256Hex -Bytes $file.bytes) }
$childTreeSha256 = Get-ProjectTreeSha256 -Files $childFiles
$childRevisionSha256 = Get-ProjectRevisionSha256 -ProjectId $projectId -TreeSha256 $childTreeSha256 -ParentRevisionSha256 $revisionSha256 -Action 'agent_overlay_commit'

Start-ProjectEdit -ProjectId $projectId -BaseRevisionSha256 $revisionSha256 -Name 'w2b_positive' | Out-Null
$replaceFinalize = Send-ProjectEditFile -Name 'w2b_positive_replace' -Path $childFiles[1].path -Classification $childFiles[1].classification -MediaType $childFiles[1].media_type -Bytes $childFiles[1].bytes
Add-ProjectPredicate -Name 'w2b_positive:replace_staged' -Expected 'src/main.rs replacement staged in RAM' -Passed (
    $replaceFinalize.status -eq 'accepted' -and $replaceFinalize.reason -eq 'project_edit_file_finalized' -and $replaceFinalize.accepted -eq $true
) -Actual $replaceFinalize
Assert-ProjectEditPosture -Result $replaceFinalize -MayWrite $false -Name 'w2b_positive:replace'
$addFinalize = Send-ProjectEditFile -Name 'w2b_positive_add' -Path $childFiles[0].path -Classification $childFiles[0].classification -MediaType $childFiles[0].media_type -Bytes $childFiles[0].bytes
Add-ProjectPredicate -Name 'w2b_positive:add_staged' -Expected 'README.md addition staged in RAM' -Passed (
    $addFinalize.status -eq 'accepted' -and $addFinalize.reason -eq 'project_edit_file_finalized' -and $addFinalize.accepted -eq $true
) -Actual $addFinalize
Assert-ProjectEditPosture -Result $addFinalize -MayWrite $false -Name 'w2b_positive:add'
$cargoPathBase64 = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes('Cargo.toml'))
$deleteResult = Send-ProjectCommand -Command "project.edit_delete $cargoPathBase64" -Method 'project.edit_delete' -Name 'w2b_positive:delete'
Add-ProjectPredicate -Name 'w2b_positive:delete_staged' -Expected 'Cargo.toml deletion staged in RAM' -Passed (
    $deleteResult.status -eq 'accepted' -and $deleteResult.reason -eq 'project_edit_delete_accepted' -and $deleteResult.accepted -eq $true
) -Actual $deleteResult
Assert-ProjectEditPosture -Result $deleteResult -MayWrite $false -Name 'w2b_positive:delete'

$diffResult = Send-ProjectCommand -Command 'project.edit_diff' -Method 'project.edit_diff' -Name 'w2b_positive:diff'
$actualDiff = @($diffResult.diff)
$expectedDiff = @(
    [pscustomobject]@{ kind = 'delete'; path = 'Cargo.toml'; old = $files[0]; new = $null },
    [pscustomobject]@{ kind = 'add'; path = 'README.md'; old = $null; new = $childFiles[0] },
    [pscustomobject]@{ kind = 'replace'; path = 'src/main.rs'; old = $files[1]; new = $childFiles[1] }
)
$diffExact = $actualDiff.Count -eq $expectedDiff.Count
for ($index = 0; $diffExact -and $index -lt $expectedDiff.Count; $index++) {
    $actual = $actualDiff[$index]
    $expected = $expectedDiff[$index]
    $oldExact = if ($null -eq $expected.old) {
        $null -eq $actual.old_blob_sha256 -and $null -eq $actual.old_classification -and
        $null -eq $actual.old_media_type -and $null -eq $actual.old_byte_len
    } else {
        $actual.old_blob_sha256 -eq "sha256:$($expected.old.sha256)" -and
        $actual.old_classification -eq $expected.old.classification -and
        $actual.old_media_type -eq $expected.old.media_type -and
        [int]$actual.old_byte_len -eq $expected.old.bytes.Length
    }
    $newExact = if ($null -eq $expected.new) {
        $null -eq $actual.new_blob_sha256 -and $null -eq $actual.new_classification -and
        $null -eq $actual.new_media_type -and $null -eq $actual.new_byte_len
    } else {
        $actual.new_blob_sha256 -eq "sha256:$($expected.new.sha256)" -and
        $actual.new_classification -eq $expected.new.classification -and
        $actual.new_media_type -eq $expected.new.media_type -and
        [int]$actual.new_byte_len -eq $expected.new.bytes.Length
    }
    $diffExact = (
        $actual.kind -eq $expected.kind -and $actual.path -ceq $expected.path -and
        $oldExact -and $newExact
    )
}
Add-ProjectPredicate -Name 'w2b_positive:exact_sorted_diff' -Expected 'sorted delete/add/replace diff binds exact old/new metadata and proposed child hashes' -Passed (
    $diffResult.status -eq 'present' -and $diffResult.reason -eq 'project_edit_diff_verified' -and
    $diffResult.action -eq 'agent_overlay_commit' -and
    $diffResult.project_id -eq $projectId -and $diffResult.base_revision_sha256 -eq "sha256:$revisionSha256" -and
    $diffResult.proposed_tree_sha256 -eq "sha256:$childTreeSha256" -and
    $diffResult.proposed_revision_sha256 -eq "sha256:$childRevisionSha256" -and
    [int]$diffResult.diff_count -eq 3 -and $diffExact
) -Actual $diffResult
Assert-ProjectEditPosture -Result $diffResult -MayWrite $false -Name 'w2b_positive:diff'

$childCommit = Send-ProjectCommand -Command 'project.edit_commit' -Method 'project.edit_commit' -Name 'w2b_positive:commit'
Add-ProjectPredicate -Name 'w2b_positive:exact_child_commit' -Expected 'one exact child revision commits with parent/action/tree/revision binding' -Passed (
    $childCommit.status -eq 'accepted' -and $childCommit.reason -eq 'project_edit_revision_committed' -and
    $childCommit.accepted -eq $true -and $childCommit.action -eq 'agent_overlay_commit' -and
    $childCommit.project_id -eq $projectId -and
    $childCommit.parent_revision_sha256 -eq "sha256:$revisionSha256" -and
    $childCommit.tree_sha256 -eq "sha256:$childTreeSha256" -and
    $childCommit.revision_sha256 -eq "sha256:$childRevisionSha256" -and
    $childCommit.storage_write_attempted -eq $true -and $childCommit.writes_persistent_state -eq $true
) -Actual $childCommit
Assert-ProjectEditPosture -Result $childCommit -MayWrite $true -Name 'w2b_positive:commit'

$childInspect = Send-ProjectCommand -Command "project.inspect $projectId" -Method 'project.inspect' -Name 'w2b_positive:inspect_child'
Assert-ExactProjectRevision -Result $childInspect -Files $childFiles -ProjectId $projectId -TreeSha256 $childTreeSha256 -RevisionSha256 $childRevisionSha256 -ParentRevisionSha256 $revisionSha256 -Action 'agent_overlay_commit' -Name 'w2b_child_first_boot'
Add-ProjectPredicate -Name 'w2b_positive:child_parent_bound' -Expected 'child inspect retains exact original parent revision' -Passed ($childInspect.parent_revision_sha256 -eq "sha256:$revisionSha256") -Actual $childInspect
Assert-ProjectReadAndSearch -ProjectId $projectId -RevisionSha256 $childRevisionSha256 -SourceFile $childFiles[1] -Name 'w2b_child_first_boot'
$childRevisionJson = $childInspect | ConvertTo-Json -Compress -Depth 8

# Discard is a second real overlay, not a synthetic no-op: stage one replacement,
# discard it, then prove the committed child remains exact.
Start-ProjectEdit -ProjectId $projectId -BaseRevisionSha256 $childRevisionSha256 -Name 'w2b_discard' | Out-Null
$discardBytes = [System.Text.Encoding]::UTF8.GetBytes("# discarded candidate`n")
$discardFinalize = Send-ProjectEditFile -Name 'w2b_discard' -Path 'README.md' -Classification public -MediaType text/markdown -Bytes $discardBytes
Add-ProjectPredicate -Name 'w2b_discard:change_staged' -Expected 'second overlay contains a real replacement before discard' -Passed ($discardFinalize.accepted -eq $true) -Actual $discardFinalize
$discardResult = Send-ProjectCommand -Command 'project.edit_discard' -Method 'project.edit_discard' -Name 'w2b_discard:discard'
Add-ProjectPredicate -Name 'w2b_discard:accepted' -Expected 'discard drops only RAM overlay and writes nothing' -Passed (
    $discardResult.status -eq 'accepted' -and $discardResult.reason -eq 'project_edit_discarded' -and
    $discardResult.accepted -eq $true
) -Actual $discardResult
Assert-ProjectEditPosture -Result $discardResult -MayWrite $false -Name 'w2b_discard'
$afterDiscard = Send-ProjectCommand -Command "project.inspect $projectId" -Method 'project.inspect' -Name 'w2b_discard:inspect'
Add-ProjectPredicate -Name 'w2b_discard:child_unchanged' -Expected 'discard leaves exact committed child revision unchanged' -Passed (($afterDiscard | ConvertTo-Json -Compress -Depth 8) -ceq $childRevisionJson) -Actual $afterDiscard

# Third boot proves the W2b child; the existing first/reboot W1+W2a evidence above
# remains byte-identical and unchanged.
Close-SerialTcpConnection
if (-not $QemuPid) { throw 'project-workspace W2b cannot reboot without the second QEMU process' }
$secondQemuPid = $QemuPid
Stop-Process -Id $secondQemuPid -Force -ErrorAction Stop
if ($script:QemuProcess) { try { $script:QemuProcess.WaitForExit(5000) | Out-Null } catch {} }
if (Get-Process -Id $secondQemuPid -ErrorAction SilentlyContinue) { throw 'project-workspace second QEMU process did not stop before W2b reboot' }

$childRebootLog = Join-Path $RunDir 'serial-project-workspace-child-reboot.log'
$childRebootParams = $runParams.Clone()
$childRebootParams.StopExisting = $false
$childRebootParams.SerialLog = $childRebootLog
$childRebootOutput = & $RunScript @childRebootParams
$SerialLog = $childRebootLog
$QemuPid = $null
foreach ($line in $childRebootOutput) { if ($line -match '^qemu pid:\s*(\d+)') { $QemuPid = [int]$Matches[1] } }
if (-not $QemuPid) { throw 'project-workspace W2b reboot did not return a QEMU pid' }
try { $script:QemuProcess = Get-Process -Id $QemuPid -ErrorAction Stop } catch { $script:QemuProcess = $null }

Assert-LogContains -Name 'project-workspace:w2b_child_reboot_serial_console_ready' -Needle 'SERIAL CONSOLE READY' -TimeoutSeconds $TimeoutSeconds
$childRebootInspect = Send-ProjectCommand -Command "project.inspect $projectId" -Method 'project.inspect' -Name 'w2b_child_reboot:inspect'
Assert-ExactProjectRevision -Result $childRebootInspect -Files $childFiles -ProjectId $projectId -TreeSha256 $childTreeSha256 -RevisionSha256 $childRevisionSha256 -ParentRevisionSha256 $revisionSha256 -Action 'agent_overlay_commit' -Name 'w2b_child_reboot'
Add-ProjectPredicate -Name 'w2b_child_reboot:parent_bound' -Expected 'rebooted child retains exact original parent' -Passed ($childRebootInspect.parent_revision_sha256 -eq "sha256:$revisionSha256") -Actual $childRebootInspect
Assert-ProjectReadAndSearch -ProjectId $projectId -RevisionSha256 $childRevisionSha256 -SourceFile $childFiles[1] -Name 'w2b_child_reboot'
$childRebootJson = $childRebootInspect | ConvertTo-Json -Compress -Depth 8
Add-ProjectPredicate -Name 'w2b_child_reboot:byte_identical_child_facts' -Expected 'child project/tree/revision/file facts byte-identical across reboot' -Passed ($childRebootJson -ceq $childRevisionJson) -Actual $(if ($childRebootJson -ceq $childRevisionJson) { $childRevisionSha256 } else { 'response_mismatch' })

$dependencySourceChildRebootInspect = Send-ProjectCommand -Command "project.inspect $dependencyProjectId" -Method 'project.inspect' -Name 'w3_source:inspect_child_reboot'
Assert-ExactProjectRevision -Result $dependencySourceChildRebootInspect -Files $dependencyFiles -ProjectId $dependencyProjectId -TreeSha256 $dependencyTreeSha256 -RevisionSha256 $dependencyRevisionSha256 -Name 'w3_source_child_reboot'
$dependencySourceChildRebootJson = $dependencySourceChildRebootInspect | ConvertTo-Json -Compress -Depth 8
Add-ProjectPredicate -Name 'w3_source:boot3_revision_unchanged' -Expected 'dependency import cannot mutate its exact source revision across three boots' -Passed (
    $dependencySourceChildRebootJson -ceq $dependencySourceJson
) -Actual $(if ($dependencySourceChildRebootJson -ceq $dependencySourceJson) { $dependencyRevisionSha256 } else { 'response_mismatch' })

$dependencyChildRebootInspect = Send-ProjectCommand -Command "project.dependencies $dependencyProjectId sha256:$dependencyRevisionSha256" -Method 'project.dependencies' -Name 'w3_child_reboot:inspect'
Assert-ExactDependencyInspection -Result $dependencyChildRebootInspect -Name 'w3_child_reboot'
$dependencyChildRebootJson = $dependencyChildRebootInspect | ConvertTo-Json -Compress -Depth 12
Add-ProjectPredicate -Name 'w3_child_reboot:byte_identical_inspection' -Expected 'metadata/file/chunk/tree/bundle hashes and build.rs inertness replay byte-identically on boot 3' -Passed (
    $dependencyChildRebootJson -ceq $dependencyInspectJson
) -Actual $(if ($dependencyChildRebootJson -ceq $dependencyInspectJson) { $dependencyBundleSha256 } else { 'response_mismatch' })

$combinedLog = Join-Path $RunDir 'serial-project-workspace-combined.log'
$firstBootContent = Get-Content -LiteralPath $firstBootLog -Raw -ErrorAction Stop
$rebootContent = Get-Content -LiteralPath $rebootLog -Raw -ErrorAction Stop
$childRebootContent = Get-Content -LiteralPath $childRebootLog -Raw -ErrorAction Stop
Set-Content -LiteralPath $combinedLog -Value ($firstBootContent + [Environment]::NewLine + $rebootContent + [Environment]::NewLine + $childRebootContent) -Encoding UTF8
$SerialLog = $combinedLog
