[CmdletBinding(DefaultParameterSetName = "Disk")]
param(
    [Parameter(Mandatory = $true, ParameterSetName = "Disk")]
    [int]$DiskNumber,

    [Parameter(Mandatory = $true, ParameterSetName = "Disk")]
    [ValidateNotNullOrEmpty()]
    [string]$ExpectedFriendlyName,

    [Parameter(Mandatory = $true, ParameterSetName = "Disk")]
    [ValidatePattern('^[0-9a-fA-F]{64}$')]
    [string]$ExpectedImagePrefixSha256,

    [Parameter(ParameterSetName = "Disk")]
    [ValidateRange(512, 16777216)]
    [int]$ImagePrefixBytes = 1048576,

    [Parameter(Mandatory = $true, ParameterSetName = "SelfTest")]
    [switch]$SelfTest
)

$ErrorActionPreference = "Stop"

$SectorSize = 512
$GptEntryCount = 128
$GptEntrySize = 128
$GptEntriesLba = 2
$SeedDataTypeGuid = "5eedda7a-c0de-4a55-9a15-000000000001"
$EspTypeGuid = "c12a7328-f81f-11d2-ba4b-00a0c93ec93b"
$SeedDataMagic = [Text.Encoding]::ASCII.GetBytes("RAIOS_DATA_SB_V0")
$ReclogMagic = [Text.Encoding]::ASCII.GetBytes("RAIOSRC0")
$ReclogHeaderLength = 88
$ReclogStartLba = 16
$ReclogLbaCount = 4096
$TraceSchema = "raios.hw_failure_trace.v0"
$UsbDiagSchema = "raios.usb_diag.v0"

function Get-U32Le {
    param([byte[]]$Bytes, [int]$Offset)
    if ($Offset -lt 0 -or $Offset + 4 -gt $Bytes.Length) {
        throw "u32_out_of_bounds"
    }
    return [BitConverter]::ToUInt32($Bytes, $Offset)
}

function Get-U64Le {
    param([byte[]]$Bytes, [int]$Offset)
    if ($Offset -lt 0 -or $Offset + 8 -gt $Bytes.Length) {
        throw "u64_out_of_bounds"
    }
    return [BitConverter]::ToUInt64($Bytes, $Offset)
}

function Test-BytesEqual {
    param([byte[]]$Left, [byte[]]$Right)
    if ($Left.Length -ne $Right.Length) {
        return $false
    }
    for ($index = 0; $index -lt $Left.Length; $index++) {
        if ($Left[$index] -ne $Right[$index]) {
            return $false
        }
    }
    return $true
}

function Test-AllZero {
    param([byte[]]$Bytes, [int]$Offset, [int]$Length)
    if ($Offset -lt 0 -or $Length -lt 0 -or $Offset + $Length -gt $Bytes.Length) {
        throw "zero_range_out_of_bounds"
    }
    for ($index = $Offset; $index -lt $Offset + $Length; $index++) {
        if ($Bytes[$index] -ne 0) {
            return $false
        }
    }
    return $true
}

function Get-Slice {
    param([byte[]]$Bytes, [int]$Offset, [int]$Length)
    if ($Offset -lt 0 -or $Length -lt 0 -or $Offset + $Length -gt $Bytes.Length) {
        throw "slice_out_of_bounds"
    }
    $result = New-Object byte[] $Length
    [Array]::Copy($Bytes, $Offset, $result, 0, $Length)
    return $result
}

function Get-Sha256Bytes {
    param([byte[]]$Bytes)
    $sha = [Security.Cryptography.SHA256]::Create()
    try {
        return $sha.ComputeHash($Bytes)
    }
    finally {
        $sha.Dispose()
    }
}

function ConvertTo-Hex {
    param([byte[]]$Bytes)
    return ([BitConverter]::ToString($Bytes)).Replace("-", "").ToLowerInvariant()
}

function ConvertFrom-GptTypeGuidBytes {
    param([byte[]]$Bytes)
    if ($null -eq $Bytes -or $Bytes.Length -ne 16) {
        throw "gpt_type_guid_length_invalid"
    }
    return ([Guid]::new($Bytes)).ToString().ToLowerInvariant()
}

function Get-Crc32 {
    param([byte[]]$Bytes)
    # Windows PowerShell 5.1 parses 0xffffffff and 0xedb88320 as negative
    # Int32 values before applying the UInt32 cast. Construct both unsigned
    # values without signed hexadecimal literals so GPT validation works on
    # the same shell used for the physical-disk safety gate.
    [uint32]$crc = [uint32]::MaxValue
    [uint32]$polynomial = [Convert]::ToUInt32("edb88320", 16)
    foreach ($byte in $Bytes) {
        $crc = $crc -bxor [uint32]$byte
        for ($bit = 0; $bit -lt 8; $bit++) {
            if (($crc -band 1) -ne 0) {
                $crc = [uint32](($crc -shr 1) -bxor $polynomial)
            }
            else {
                $crc = [uint32]($crc -shr 1)
            }
        }
    }
    return [uint32]($crc -bxor [uint32]::MaxValue)
}

function Read-ExactAt {
    param(
        [IO.Stream]$Stream,
        [long]$Offset,
        [int]$Length
    )
    if (-not $Stream.CanRead -or $Stream.CanWrite) {
        throw "source_stream_must_be_read_only"
    }
    [void]$Stream.Seek($Offset, [IO.SeekOrigin]::Begin)
    $bytes = New-Object byte[] $Length
    $readTotal = 0
    while ($readTotal -lt $Length) {
        $read = $Stream.Read($bytes, $readTotal, $Length - $readTotal)
        if ($read -le 0) {
            throw "short_read"
        }
        $readTotal += $read
    }
    return $bytes
}

function Assert-GptAndGetSeedData {
    param([IO.Stream]$Stream)

    $mbr = Read-ExactAt -Stream $Stream -Offset 0 -Length $SectorSize
    if ($mbr[510] -ne 0x55 -or $mbr[511] -ne 0xaa -or $mbr[450] -ne 0xee) {
        throw "protective_mbr_invalid"
    }

    $header = Read-ExactAt -Stream $Stream -Offset $SectorSize -Length $SectorSize
    $signature = [Text.Encoding]::ASCII.GetString($header, 0, 8)
    if ($signature -cne "EFI PART") {
        throw "gpt_header_missing"
    }
    $headerSize = [int](Get-U32Le $header 12)
    if ($headerSize -lt 92 -or $headerSize -gt $SectorSize) {
        throw "gpt_header_size_invalid"
    }
    $storedHeaderCrc = Get-U32Le $header 16
    $headerForCrc = Get-Slice $header 0 $headerSize
    for ($index = 16; $index -lt 20; $index++) {
        $headerForCrc[$index] = 0
    }
    if ((Get-Crc32 $headerForCrc) -ne $storedHeaderCrc) {
        throw "gpt_header_crc32_mismatch"
    }

    $entryLba = Get-U64Le $header 72
    $entryCount = Get-U32Le $header 80
    $entrySize = Get-U32Le $header 84
    if ($entryLba -ne $GptEntriesLba -or $entryCount -ne $GptEntryCount -or $entrySize -ne $GptEntrySize) {
        throw "gpt_entry_geometry_mismatch"
    }
    $entryBytes = Read-ExactAt `
        -Stream $Stream `
        -Offset ([long]$entryLba * $SectorSize) `
        -Length ([int]$entryCount * [int]$entrySize)
    if ((Get-Crc32 $entryBytes) -ne (Get-U32Le $header 88)) {
        throw "gpt_entry_array_crc32_mismatch"
    }

    $required = @{}
    for ($index = 0; $index -lt $entryCount; $index++) {
        $offset = $index * $entrySize
        $typeBytes = Get-Slice $entryBytes $offset 16
        $allZero = $true
        foreach ($byte in $typeBytes) {
            if ($byte -ne 0) {
                $allZero = $false
                break
            }
        }
        if ($allZero) {
            continue
        }
        $typeGuid = ConvertFrom-GptTypeGuidBytes -Bytes $typeBytes
        $name = [Text.Encoding]::Unicode.GetString($entryBytes, $offset + 56, 72).Trim([char]0)
        if ($name -in @("SEED_ESP_A", "SEED_ESP_B", "SEED_DATA")) {
            if ($required.ContainsKey($name)) {
                throw "gpt_required_partition_duplicate"
            }
            $required[$name] = [pscustomobject]@{
                name = $name
                type_guid = $typeGuid
                first_lba = Get-U64Le $entryBytes ($offset + 32)
                last_lba = Get-U64Le $entryBytes ($offset + 40)
            }
        }
    }
    foreach ($name in @("SEED_ESP_A", "SEED_ESP_B", "SEED_DATA")) {
        if (-not $required.ContainsKey($name)) {
            throw "gpt_required_partition_missing"
        }
    }
    if ($required.SEED_ESP_A.type_guid -cne $EspTypeGuid -or
        $required.SEED_ESP_B.type_guid -cne $EspTypeGuid -or
        $required.SEED_DATA.type_guid -cne $SeedDataTypeGuid) {
        throw "gpt_required_partition_type_mismatch"
    }
    if ($required.SEED_ESP_A.last_lba -ge $required.SEED_ESP_B.first_lba -or
        $required.SEED_ESP_B.last_lba -ge $required.SEED_DATA.first_lba -or
        $required.SEED_DATA.last_lba -lt $required.SEED_DATA.first_lba) {
        throw "gpt_required_partition_span_invalid"
    }
    return $required.SEED_DATA
}

function Assert-SeedDataSuperblock {
    param([IO.Stream]$Stream, $SeedData)

    $dataLbaCount = [uint64]($SeedData.last_lba - $SeedData.first_lba + 1)
    $superblock0 = Read-ExactAt -Stream $Stream -Offset ([long]$SeedData.first_lba * $SectorSize) -Length $SectorSize
    $superblock1 = Read-ExactAt -Stream $Stream -Offset ([long]($SeedData.first_lba + 1) * $SectorSize) -Length $SectorSize
    if (-not (Test-BytesEqual $superblock0 $superblock1)) {
        throw "superblock_lba1_copy_mismatch"
    }
    if (-not (Test-BytesEqual (Get-Slice $superblock0 0 16) $SeedDataMagic)) {
        throw "superblock_magic_mismatch"
    }
    if ((Get-U32Le $superblock0 16) -ne 0 -or
        (Get-U32Le $superblock0 20) -ne 128 -or
        (Get-U32Le $superblock0 24) -ne 3 -or
        (Get-U32Le $superblock0 28) -ne 24 -or
        (Get-U64Le $superblock0 32) -ne $dataLbaCount) {
        throw "superblock_header_mismatch"
    }
    if (-not (Test-BytesEqual (Get-Sha256Bytes (Get-Slice $superblock0 0 128)) (Get-Slice $superblock0 128 32))) {
        throw "superblock_header_sha256_mismatch"
    }

    $expectedRegions = @(
        @("BOOTCTL`0", [uint64]2, [uint64]8),
        @("RECLOG`0`0", [uint64]16, [uint64]4096),
        @("ARTSTOR`0", [uint64]8192, [uint64]($dataLbaCount - 8192))
    )
    for ($index = 0; $index -lt $expectedRegions.Count; $index++) {
        $offset = 48 + $index * 24
        $tag = [Text.Encoding]::ASCII.GetString($superblock0, $offset, 8)
        if ($tag -cne $expectedRegions[$index][0] -or
            (Get-U64Le $superblock0 ($offset + 8)) -ne $expectedRegions[$index][1] -or
            (Get-U64Le $superblock0 ($offset + 16)) -ne $expectedRegions[$index][2]) {
            throw "superblock_region_mismatch"
        }
    }
    if ($dataLbaCount -lt ($ReclogStartLba + $ReclogLbaCount)) {
        throw "reclog_region_out_of_seed_data"
    }
}

function Convert-ValidatedTracePayload {
    param([byte[]]$Payload)

    try {
        $text = [Text.UTF8Encoding]::new($false, $true).GetString($Payload)
        $trace = $text | ConvertFrom-Json
    }
    catch {
        throw "hw_failure_trace_json_invalid"
    }
    $actualFields = @($trace.PSObject.Properties.Name | Sort-Object)
    $expectedFields = @("build_id", "classification", "schema", "scope", "steps", "subsystem") | Sort-Object
    if (($actualFields -join "|") -cne ($expectedFields -join "|")) {
        throw "hw_failure_trace_field_set_invalid"
    }
    try {
        [void][uint64]$trace.build_id
    }
    catch {
        throw "hw_failure_trace_build_id_invalid"
    }
    if ($trace.schema -cne $TraceSchema -or
        $trace.classification -cne "local_only" -or
        $trace.scope -cne "current_boot" -or
        [int]$trace.subsystem -ne 1) {
        throw "hw_failure_trace_identity_invalid"
    }
    $steps = @($trace.steps)
    if ($steps.Count -lt 1 -or $steps.Count -gt 4) {
        throw "hw_failure_trace_step_count_invalid"
    }
    [uint64]$previousTime = 0
    for ($index = 0; $index -lt $steps.Count; $index++) {
        $step = @($steps[$index])
        if ($step.Count -ne 5) {
            throw "hw_failure_trace_step_shape_invalid"
        }
        [uint64]$bootMs = $step[0]
        [int]$phase = $step[1]
        [int]$status = $step[2]
        [int]$register = $step[3]
        [uint64]$registerValue = $step[4]
        if (($index -gt 0 -and $bootMs -lt $previousTime) -or
            $bootMs -gt [uint32]::MaxValue -or
            $phase -lt 1 -or $phase -gt 8 -or
            $status -notin @(1, 2, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112) -or
            $register -lt 0 -or $register -gt 8 -or
            $registerValue -gt [uint32]::MaxValue) {
            throw "hw_failure_trace_step_value_invalid"
        }
        $previousTime = $bootMs
    }
    return $trace
}

function Get-HwFailureStatusName {
    param([int]$Status)
    switch ($Status) {
        1 { "started" }
        2 { "completed" }
        100 { "timeout" }
        101 { "transport_fault" }
        102 { "firmware_rejected" }
        103 { "unsupported_security" }
        104 { "authentication_rejected" }
        105 { "association_rejected" }
        106 { "key_exchange_failed" }
        107 { "link_lost" }
        108 { "boot_posture_denied" }
        109 { "network_state_not_granted" }
        110 { "k2_checkpoint_persist_failed" }
        111 { "k2_publication_rejected" }
        112 { "k2_pci_command_rejected" }
        default { throw "hw_failure_trace_status_name_missing" }
    }
}

function Get-HwFailureRegisterName {
    param([int]$Register)
    switch ($Register) {
        0 { "none" }
        1 { "marvell_host_interrupt_status" }
        2 { "marvell_host_interrupt_mask" }
        3 { "marvell_firmware_status" }
        4 { "marvell_command_response_status" }
        5 { "xhci_usb_status" }
        6 { "xhci_port_status_change" }
        7 { "marvell_pci_command" }
        8 { "marvell_publication_step" }
        default { throw "hw_failure_trace_register_name_missing" }
    }
}

function Convert-ValidatedUsbDiagnosticPayload {
    param([byte[]]$Payload)

    try {
        $text = [Text.UTF8Encoding]::new($false, $true).GetString($Payload)
        $record = $text | ConvertFrom-Json
    }
    catch {
        throw "usb_diag_json_invalid"
    }
    $actualFields = @($record.PSObject.Properties.Name | Sort-Object)
    $expectedFields = @(
        "classification", "enum_pid", "enum_vid", "errors", "hub_connected", "hub_count",
        "hub_done", "hub_ports", "hub_reset", "last_cc", "last_cmd", "last_int_cc",
        "last_xfer_cc", "m_chg", "m_ep", "m_port", "reason", "recover", "reports",
        "schema", "scope", "seq"
    ) | Sort-Object
    if (($actualFields -join "|") -cne ($expectedFields -join "|")) {
        throw "usb_diag_field_set_invalid"
    }
    if ($record.schema -cne $UsbDiagSchema -or
        $record.classification -cne "local_only" -or
        $record.scope -cne "current_boot" -or
        [string]$record.reason -notmatch '^[a-z0-9_]{1,32}$') {
        throw "usb_diag_identity_invalid"
    }
    foreach ($field in $expectedFields) {
        if ($field -notin @("classification", "reason", "schema", "scope")) {
            try { [void][uint64]($record.$field) }
            catch { throw "usb_diag_numeric_field_invalid" }
        }
    }
    return $record
}

function Read-ReclogTraces {
    param(
        [byte[]]$Region,
        [uint64]$AbsoluteStartLba
    )
    if ($Region.Length -eq 0 -or ($Region.Length % $SectorSize) -ne 0) {
        throw "reclog_region_size_invalid"
    }
    $records = @()
    $usbDiagnostics = @()
    $offset = 0
    [uint64]$expectedSeq = 1
    $previousFrameHash = New-Object byte[] 32
    $tailStatus = "full"
    while ($offset -lt $Region.Length) {
        $sectorIsZero = Test-AllZero $Region $offset $SectorSize
        if ($sectorIsZero) {
            if (-not (Test-AllZero $Region $offset ($Region.Length - $offset))) {
                throw "reclog_nonzero_after_zero_tail"
            }
            $tailStatus = "zero_tail"
            break
        }
        if (-not (Test-BytesEqual (Get-Slice $Region $offset 8) $ReclogMagic)) {
            throw "reclog_bad_magic"
        }
        $frameLength = [int](Get-U32Le $Region ($offset + 8))
        $payloadLength = [int](Get-U32Le $Region ($offset + 12))
        $sequence = Get-U64Le $Region ($offset + 16)
        if ($frameLength -lt $SectorSize -or ($frameLength % $SectorSize) -ne 0 -or
            $frameLength -gt 4096 -or $offset + $frameLength -gt $Region.Length -or
            $payloadLength -lt 0 -or $payloadLength -gt ($frameLength - $ReclogHeaderLength) -or
            $sequence -ne $expectedSeq) {
            throw "reclog_frame_shape_invalid"
        }
        if (-not (Test-BytesEqual (Get-Slice $Region ($offset + 24) 32) $previousFrameHash)) {
            throw "reclog_previous_hash_mismatch"
        }
        $payload = Get-Slice $Region ($offset + $ReclogHeaderLength) $payloadLength
        $payloadHash = Get-Sha256Bytes $payload
        if (-not (Test-BytesEqual $payloadHash (Get-Slice $Region ($offset + 56) 32))) {
            throw "reclog_payload_hash_mismatch"
        }
        $paddingLength = $frameLength - $ReclogHeaderLength - $payloadLength
        if ($paddingLength -gt 0) {
            if (-not (Test-AllZero $Region ($offset + $ReclogHeaderLength + $payloadLength) $paddingLength)) {
                throw "reclog_nonzero_padding"
            }
        }
        $frame = Get-Slice $Region $offset $frameLength
        $frameHash = Get-Sha256Bytes $frame
        $schemaPrefix = [Text.Encoding]::UTF8.GetBytes('{"schema":"raios.hw_failure_trace.v0",')
        if ($payload.Length -ge $schemaPrefix.Length -and
            (Test-BytesEqual (Get-Slice $payload 0 $schemaPrefix.Length) $schemaPrefix)) {
            $trace = Convert-ValidatedTracePayload $payload
            $decodedSteps = @()
            $isK2Publication = $false
            foreach ($stepValue in @($trace.steps)) {
                $step = @($stepValue)
                $status = [int]$step[2]
                $register = [int]$step[3]
                if ($status -in @(110, 111, 112) -or
                    ($status -eq 1 -and $register -eq 7) -or
                    $register -eq 8) {
                    $isK2Publication = $true
                }
                $decodedSteps += [pscustomobject][ordered]@{
                    boot_ms = [uint32]$step[0]
                    phase = [int]$step[1]
                    status = $status
                    status_name = Get-HwFailureStatusName $status
                    register = $register
                    register_name = Get-HwFailureRegisterName $register
                    register_value = [uint32]$step[4]
                }
            }
            $records += [pscustomobject][ordered]@{
                kind = $(if ($isK2Publication) { "k2_publication" } else { "hw_failure_trace" })
                seq = $sequence
                absolute_lba = $AbsoluteStartLba + [uint64]($offset / $SectorSize)
                frame_sha256 = ConvertTo-Hex $frameHash
                payload_sha256 = ConvertTo-Hex $payloadHash
                trace = $trace
                decoded_steps = @($decodedSteps)
            }
        }
        $usbSchemaPrefix = [Text.Encoding]::UTF8.GetBytes('{"schema":"raios.usb_diag.v0",')
        if ($payload.Length -ge $usbSchemaPrefix.Length -and
            (Test-BytesEqual (Get-Slice $payload 0 $usbSchemaPrefix.Length) $usbSchemaPrefix)) {
            $usbDiagnostics += [pscustomobject][ordered]@{
                kind = "usb_diag"
                seq = $sequence
                absolute_lba = $AbsoluteStartLba + [uint64]($offset / $SectorSize)
                frame_sha256 = ConvertTo-Hex $frameHash
                payload_sha256 = ConvertTo-Hex $payloadHash
                diagnostic = Convert-ValidatedUsbDiagnosticPayload $payload
            }
        }
        $previousFrameHash = $frameHash
        $expectedSeq++
        $offset += $frameLength
    }
    return [pscustomobject]@{
        records = @($records)
        usb_diagnostics = @($usbDiagnostics)
        valid_frame_count = [uint64]($expectedSeq - 1)
        tail_status = $tailStatus
    }
}

function New-SelfTestFrame {
    param([string]$PayloadText, [uint64]$Sequence, [byte[]]$PreviousFrameHash)
    $payload = [Text.Encoding]::UTF8.GetBytes($PayloadText)
    if ($payload.Length -gt ($SectorSize - $ReclogHeaderLength)) {
        throw "selftest_payload_too_large"
    }
    $frame = New-Object byte[] $SectorSize
    [Array]::Copy($ReclogMagic, 0, $frame, 0, 8)
    [Array]::Copy([BitConverter]::GetBytes([uint32]$SectorSize), 0, $frame, 8, 4)
    [Array]::Copy([BitConverter]::GetBytes([uint32]$payload.Length), 0, $frame, 12, 4)
    [Array]::Copy([BitConverter]::GetBytes($Sequence), 0, $frame, 16, 8)
    [Array]::Copy($PreviousFrameHash, 0, $frame, 24, 32)
    [Array]::Copy((Get-Sha256Bytes $payload), 0, $frame, 56, 32)
    [Array]::Copy($payload, 0, $frame, $ReclogHeaderLength, $payload.Length)
    return $frame
}

function Invoke-SelfTest {
    $knownEspTypeBytes = [byte[]]@(
        0x28, 0x73, 0x2a, 0xc1, 0x1f, 0xf8, 0xd2, 0x11,
        0xba, 0x4b, 0x00, 0xa0, 0xc9, 0x3e, 0xc9, 0x3b
    )
    $knownEspEntry = New-Object byte[] $GptEntrySize
    [Array]::Copy($knownEspTypeBytes, 0, $knownEspEntry, 0, 16)
    $actualEspTypeGuid = ConvertFrom-GptTypeGuidBytes -Bytes (Get-Slice $knownEspEntry 0 16)
    if ($actualEspTypeGuid -cne $EspTypeGuid) {
        throw "selftest_gpt_guid_known_vector_failed"
    }
    foreach ($invalidGuidLength in @(15, 17)) {
        try {
            [void](ConvertFrom-GptTypeGuidBytes -Bytes (New-Object byte[] $invalidGuidLength))
            throw "selftest_gpt_guid_invalid_length_not_rejected"
        }
        catch {
            if ($_.Exception.Message -cne "gpt_type_guid_length_invalid") {
                throw
            }
        }
    }

    $legacyCrcVector = [Text.Encoding]::ASCII.GetBytes("123456789")
    $expectedLegacyCrc = [Convert]::ToUInt32("cbf43926", 16)
    if ((Get-Crc32 $legacyCrcVector) -ne $expectedLegacyCrc) {
        throw "selftest_powershell51_crc32_failed"
    }
    $mutatedCrcVector = Get-Slice $legacyCrcVector 0 $legacyCrcVector.Length
    $mutatedCrcVector[0] = $mutatedCrcVector[0] -bxor 1
    if ((Get-Crc32 $mutatedCrcVector) -eq $expectedLegacyCrc) {
        throw "selftest_crc32_mutation_not_rejected"
    }

    $payload = '{"schema":"raios.hw_failure_trace.v0","classification":"local_only","scope":"current_boot","build_id":65536,"subsystem":1,"steps":[[10,1,1,7,1026],[20,1,111,8,2]]}'
    $frame = New-SelfTestFrame -PayloadText $payload -Sequence 1 -PreviousFrameHash (New-Object byte[] 32)
    $usbPayload = '{"schema":"raios.usb_diag.v0","classification":"local_only","scope":"current_boot","reason":"boot_probe","seq":2,"hub_count":1,"hub_ports":4,"hub_connected":2,"hub_reset":1,"hub_done":2,"recover":0,"reports":12,"errors":0,"last_int_cc":1,"last_xfer_cc":1,"last_cmd":42,"last_cc":1,"enum_vid":4660,"enum_pid":22136,"m_port":1,"m_chg":0,"m_ep":1}'
    $usbFrame = New-SelfTestFrame -PayloadText $usbPayload -Sequence 2 -PreviousFrameHash (Get-Sha256Bytes $frame)
    $region = New-Object byte[] ($SectorSize * 3)
    [Array]::Copy($frame, 0, $region, 0, $SectorSize)
    [Array]::Copy($usbFrame, 0, $region, $SectorSize, $SectorSize)
    $parsed = Read-ReclogTraces -Region $region -AbsoluteStartLba 100
    if ($parsed.records.Count -ne 1 -or
        $parsed.records[0].kind -cne "k2_publication" -or
        $parsed.records[0].decoded_steps[0].register_name -cne "marvell_pci_command" -or
        $parsed.records[0].decoded_steps[1].status_name -cne "k2_publication_rejected" -or
        $parsed.usb_diagnostics.Count -ne 1 -or
        $parsed.usb_diagnostics[0].diagnostic.reason -cne "boot_probe" -or
        $parsed.tail_status -cne "zero_tail") {
        throw "selftest_positive_failed"
    }

    $torn = Get-Slice $region 0 $region.Length
    $torn[$ReclogHeaderLength + 1] = $torn[$ReclogHeaderLength + 1] -bxor 1
    try {
        [void](Read-ReclogTraces -Region $torn -AbsoluteStartLba 100)
        throw "selftest_torn_not_rejected"
    }
    catch {
        if ($_.Exception.Message -cne "reclog_payload_hash_mismatch") {
            throw
        }
    }

    $contaminatedPayload = '{"schema":"raios.hw_failure_trace.v0","classification":"local_only","scope":"current_boot","build_id":65536,"subsystem":1,"steps":[[10,1,100,3,1]],"ssid":"HomeWifi"}'
    $contaminated = New-SelfTestFrame -PayloadText $contaminatedPayload -Sequence 1 -PreviousFrameHash (New-Object byte[] 32)
    try {
        [void](Read-ReclogTraces -Region $contaminated -AbsoluteStartLba 100)
        throw "selftest_secret_field_not_rejected"
    }
    catch {
        if ($_.Exception.Message -cne "hw_failure_trace_field_set_invalid") {
            throw
        }
    }

    $legacyNegativeUsbPayload = '{"schema":"raios.usb_diag.v0","classification":"local_only","scope":"current_boot","reason":"boot_probe","seq":1,"hub_count":1,"hub_ports":4,"hub_connected":2,"hub_reset":1,"hub_done":2,"recover":0,"reports":12,"errors":0,"last_int_cc":-1,"last_xfer_cc":1,"last_cmd":42,"last_cc":1,"enum_vid":4660,"enum_pid":22136,"m_port":1,"m_chg":0,"m_ep":1}'
    $legacyNegativeUsbFrame = New-SelfTestFrame -PayloadText $legacyNegativeUsbPayload -Sequence 1 -PreviousFrameHash (New-Object byte[] 32)
    try {
        [void](Read-ReclogTraces -Region $legacyNegativeUsbFrame -AbsoluteStartLba 100)
        throw "selftest_legacy_negative_usb_diag_not_rejected"
    }
    catch {
        if ($_.Exception.Message -cne "usb_diag_numeric_field_invalid") {
            throw
        }
    }

    $full = New-Object byte[] ($SectorSize * 2)
    [Array]::Copy($frame, 0, $full, 0, $SectorSize)
    $second = New-SelfTestFrame -PayloadText $payload -Sequence 2 -PreviousFrameHash (Get-Sha256Bytes $frame)
    [Array]::Copy($second, 0, $full, $SectorSize, $SectorSize)
    $fullParsed = Read-ReclogTraces -Region $full -AbsoluteStartLba 100
    if ($fullParsed.valid_frame_count -ne 2 -or $fullParsed.tail_status -cne "full") {
        throw "selftest_full_region_failed"
    }

    [pscustomobject][ordered]@{
        schema = "raios.hw_failure_trace.extractor_selftest.v0"
        status = "passed"
        checks = @("gpt_guid_known_vector", "gpt_guid_invalid_length_rejected", "powershell51_crc32", "crc32_mutation_rejected", "verified_readback", "k2_decoded", "usb_diag_visible", "legacy_negative_usb_diag_rejected", "torn_rejected", "secret_field_rejected", "full_region_reported")
    } | ConvertTo-Json -Depth 4 -Compress
}

if ($PSCmdlet.ParameterSetName -eq "SelfTest") {
    Invoke-SelfTest
    return
}

if (($ImagePrefixBytes % $SectorSize) -ne 0) {
    throw "ImagePrefixBytes must be a multiple of 512"
}
$disk = Get-Disk -Number $DiskNumber -ErrorAction Stop
if ($disk.FriendlyName -cne $ExpectedFriendlyName -or $disk.FriendlyName -notlike "*SanDisk*") {
    throw "disk_friendly_name_mismatch"
}
if ($disk.BusType -ne "USB" -or $disk.IsBoot -or $disk.IsSystem -or $disk.Size -lt $ImagePrefixBytes) {
    throw "disk_safety_posture_denied"
}

$physicalPath = "\\.\PhysicalDrive$DiskNumber"
$stream = [IO.File]::Open(
    $physicalPath,
    [IO.FileMode]::Open,
    [IO.FileAccess]::Read,
    [IO.FileShare]::ReadWrite
)
try {
    if ($stream.CanWrite) {
        throw "physical_drive_opened_writable"
    }
    $prefix = Read-ExactAt -Stream $stream -Offset 0 -Length $ImagePrefixBytes
    $actualPrefixSha256 = ConvertTo-Hex (Get-Sha256Bytes $prefix)
    if ($actualPrefixSha256 -cne $ExpectedImagePrefixSha256.ToLowerInvariant()) {
        throw "image_prefix_sha256_mismatch"
    }
    $seedData = Assert-GptAndGetSeedData -Stream $stream
    Assert-SeedDataSuperblock -Stream $stream -SeedData $seedData
    $absoluteReclogLba = [uint64]$seedData.first_lba + $ReclogStartLba
    $reclog = Read-ExactAt `
        -Stream $stream `
        -Offset ([long]$absoluteReclogLba * $SectorSize) `
        -Length ($ReclogLbaCount * $SectorSize)
    $scan = Read-ReclogTraces -Region $reclog -AbsoluteStartLba $absoluteReclogLba

    [pscustomobject][ordered]@{
        schema = "raios.hw_failure_trace.extract.v0"
        classification = "local_only"
        source = [pscustomobject][ordered]@{
            disk_number = $DiskNumber
            friendly_name = $disk.FriendlyName
            size_bytes = [uint64]$disk.Size
            image_prefix_bytes = $ImagePrefixBytes
            image_prefix_sha256 = $actualPrefixSha256
            seed_data_first_lba = [uint64]$seedData.first_lba
            seed_data_lba_count = [uint64]($seedData.last_lba - $seedData.first_lba + 1)
            reclog_first_lba = $absoluteReclogLba
            read_only = $true
        }
        reclog = [pscustomobject][ordered]@{
            valid_frame_count = $scan.valid_frame_count
            tail_status = $scan.tail_status
        }
        traces = @($scan.records)
        usb_diagnostics = @($scan.usb_diagnostics)
    } | ConvertTo-Json -Depth 10 -Compress
}
finally {
    $stream.Dispose()
}
