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
    param([string]$ProjectId, [string]$TreeSha256)
    $stream = [System.IO.MemoryStream]::new()
    $writer = [System.IO.BinaryWriter]::new($stream, [System.Text.Encoding]::UTF8, $true)
    try {
        $writer.Write([System.Text.Encoding]::ASCII.GetBytes('RAIOSREV1'))
        $writer.Write([byte[]](Convert-HexToBytes -Hex $ProjectId))
        $writer.Write([byte]0)
        $writer.Write([byte[]]::new(32))
        $writer.Write([byte[]](Convert-HexToBytes -Hex $TreeSha256))
        Write-CanonicalString -Writer $writer -Value 'owner_local_import'
        Write-CanonicalString -Writer $writer -Value 'none_deterministic'
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
    param([object]$Result, [object[]]$Files, [string]$ProjectId, [string]$TreeSha256, [string]$RevisionSha256, [string]$Name)
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
        $null -eq $Result.parent_revision_sha256 -and
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
$firstRevisionJson = $firstInspect | ConvertTo-Json -Compress -Depth 8

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
$rebootRevisionJson = $rebootInspect | ConvertTo-Json -Compress -Depth 8
Add-ProjectPredicate -Name 'reboot:byte_identical_revision_facts' -Expected 'project/tree/revision/file facts byte-identical across two boots' -Passed ($rebootRevisionJson -ceq $firstRevisionJson) -Actual $(if ($rebootRevisionJson -ceq $firstRevisionJson) { $revisionSha256 } else { 'response_mismatch' })

$combinedLog = Join-Path $RunDir 'serial-project-workspace-combined.log'
$firstBootContent = Get-Content -LiteralPath $firstBootLog -Raw -ErrorAction Stop
$rebootContent = Get-Content -LiteralPath $rebootLog -Raw -ErrorAction Stop
Set-Content -LiteralPath $combinedLog -Value ($firstBootContent + [Environment]::NewLine + $rebootContent) -Encoding UTF8
$SerialLog = $combinedLog
