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
$SurfaceFactMagic = [Text.Encoding]::ASCII.GetBytes("RAIOSSF0")

function Get-U32Le {
    param([byte[]]$Bytes, [int]$Offset)
    if ($Offset -lt 0 -or $Offset + 4 -gt $Bytes.Length) {
        throw "u32_out_of_bounds"
    }
    return [BitConverter]::ToUInt32($Bytes, $Offset)
}

function Get-U16Le {
    param([byte[]]$Bytes, [int]$Offset)
    if ($Offset -lt 0 -or $Offset + 2 -gt $Bytes.Length) { throw "u16_out_of_bounds" }
    return [BitConverter]::ToUInt16($Bytes, $Offset)
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
            $register -lt 0 -or $register -gt 9 -or
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

function Convert-ConnectionTimeoutFingerprint {
    param([uint32]$Value)

    # v1 mirrors the kernel's stable packed u32. All shifts and masks are kept
    # PS5.1-compatible; no binary literals or newer enum helpers are used.
    $formatTag = [int](($Value -shr 25) -band 0x7f)
    if ($formatTag -ne 0x53) {
        throw "connection_timeout_fingerprint_tag_invalid"
    }
    $stageCode = [int](($Value -shr 22) -band 0x07)
    $stage = switch ($stageCode) {
        0 { "idle" }
        1 { "supplicant_profile" }
        2 { "supplicant_pmk" }
        3 { "associate" }
        4 { "wait_port_release" }
        5 { "link_ready" }
        6 { "failed" }
        default { throw "connection_timeout_fingerprint_stage_invalid" }
    }
    $expectedCommand = [uint16](($Value -shr 14) -band 0xff)
    $publishedCommandLength = [uint16](($Value -shr 4) -band 0x03ff)
    $requestHeaderMatchesExpected = (($Value -band 0x08) -ne 0)
    $cleanupVerified = (($Value -band 0x04) -ne 0)
    $responseCode = [int]($Value -band 0x03)
    $responseClass = switch ($responseCode) {
        0 { "untouched_zero" }
        1 { "expected_header_seen" }
        2 { "nonempty_mismatch" }
        3 { "unavailable" }
        default { throw "connection_timeout_fingerprint_response_invalid" }
    }

    [pscustomobject][ordered]@{
        format_version = 1
        connection_stage = $stage
        connection_stage_code = $stageCode
        expected_command_id = $expectedCommand
        expected_command_hex = ("0x{0:x4}" -f $expectedCommand)
        published_command_len = $publishedCommandLength
        request_header_matches_expected = $requestHeaderMatchesExpected
        verified_quiesce_cleanup = $(if ($cleanupVerified) { "succeeded" } else { "failed" })
        response_class = $responseClass
        response_class_code = $responseCode
    }
}

function Convert-AssociateDoorbellAck {
    param([uint32]$Value)

    $tagMask = [Convert]::ToUInt32("ffff0000", 16)
    $expectedTag = [Convert]::ToUInt32("d2010000", 16)
    if (($Value -band $tagMask) -ne $expectedTag) {
        throw "associate_doorbell_ack_tag_invalid"
    }
    $classification = if ($Value -eq $expectedTag) {
        "cleared"
    }
    elseif ($Value -eq [Convert]::ToUInt32("d2010001", 16)) {
        "still_set"
    }
    elseif ($Value -eq [Convert]::ToUInt32("d2010002", 16)) {
        "unavailable"
    }
    else {
        throw "associate_doorbell_ack_value_invalid"
    }

    [pscustomobject][ordered]@{
        classification = $classification
    }
}

function Convert-PostPmkHwSpecCanaryResult {
    param([uint32]$Value)

    $tagMask = [Convert]::ToUInt32("ffffff00", 16)
    $expectedTag = [Convert]::ToUInt32("d2250000", 16)
    if (($Value -band $tagMask) -ne $expectedTag) {
        throw "post_pmk_hw_spec_canary_tag_invalid"
    }
    $outcomeCode = [int]($Value -band 0xff)
    $outcome = switch ($outcomeCode) {
        0 { "expected_completion" }
        1 { "firmware_result" }
        2 { "malformed_or_wrong_completion" }
        3 { "timeout_doorbell_cleared" }
        4 { "timeout_doorbell_still_set" }
        5 { "mmio_or_doorbell_unavailable" }
        6 { "stale_high_completion" }
        7 { "host_publication_failure" }
        default { throw "post_pmk_hw_spec_canary_value_invalid" }
    }

    [pscustomobject][ordered]@{
        format_version = 1
        outcome = $outcome
        outcome_code = $outcomeCode
        network_state_granted = $false
        cold_reboot_required = $true
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
        9 { "marvell_connection_timeout_fingerprint" }
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

function Convert-SurfaceFactRecord {
    param([byte[]]$Payload)
    if ($Payload.Length -lt 40) { throw "surface_fact_wire_truncated" }
    if (-not (Test-BytesEqual (Get-Slice $Payload 0 8) $SurfaceFactMagic)) { throw "surface_fact_wire_invalid_magic" }
    $wireVersion = Get-U16Le $Payload 8
    $headerLength = Get-U16Le $Payload 10
    $recordLength = Get-U16Le $Payload 12
    $payloadLength = Get-U16Le $Payload 14
    $schemaVersion = Get-U16Le $Payload 16
    $kind = $Payload[18]
    if ($wireVersion -ne 1) { throw "surface_fact_wire_unsupported_version" }
    if ($headerLength -ne 40) { throw "surface_fact_wire_invalid_header_length" }
    if ($recordLength -ne 40 + $payloadLength -or $recordLength -gt 170) { throw "surface_fact_wire_invalid_record_length" }
    if ($recordLength -gt $Payload.Length) { throw "surface_fact_wire_truncated" }
    if ($recordLength -lt $Payload.Length) { throw "surface_fact_wire_trailing_bytes" }
    if ($schemaVersion -ne 1) { throw "surface_fact_wire_schema_version_mismatch" }
    if ($Payload[19] -ne 0) { throw "surface_fact_wire_invalid_flags" }
    if ($kind -notin @(1, 2, 3, 4, 255)) { throw "surface_fact_wire_unknown_part_kind" }
    $captureId = Get-Slice $Payload 20 16
    $partIndex = Get-U16Le $Payload 36
    $partCount = Get-U16Le $Payload 38
    if ($partCount -eq 0 -or $partCount -gt 128) { throw "surface_fact_wire_invalid_part_count" }
    if ($partIndex -ge $partCount) { throw "surface_fact_wire_part_index_out_of_range" }
    $body = Get-Slice $Payload 40 $payloadLength
    $fact = $null
    switch ($kind) {
        1 {
            if ($body.Length -ne 24) { throw "surface_fact_wire_invalid_payload_length" }
            $fact = [pscustomobject][ordered]@{ leaf = Get-U32Le $body 0; subleaf = Get-U32Le $body 4
                eax = Get-U32Le $body 8; ebx = Get-U32Le $body 12; ecx = Get-U32Le $body 16; edx = Get-U32Le $body 20 }
        }
        2 {
            if ($body.Length -lt 19) { throw "surface_fact_wire_invalid_payload_length" }
            $locatorLength = $body[18]
            if ($locatorLength -gt 32 -or $body.Length -ne 19 + $locatorLength) { throw "surface_fact_wire_invalid_locator" }
            $locatorBytes = Get-Slice $body 19 $locatorLength
            foreach ($byte in $locatorBytes) { if ($byte -lt 0x20 -or $byte -gt 0x7e) { throw "surface_fact_wire_invalid_model" } }
            $size = Get-U64Le $body 4
            if ($size -eq 0) { throw "surface_fact_wire_invalid_model" }
            $fact = [pscustomobject][ordered]@{ array_handle = Get-U16Le $body 0; device_handle = Get-U16Le $body 2
                size_bytes = $size; speed_mt_s = Get-U32Le $body 12; memory_type = $body[16]
                form_factor = $body[17]; device_locator = [Text.Encoding]::ASCII.GetString($locatorBytes) }
        }
        3 {
            if ($body.Length -ne 20) { throw "surface_fact_wire_invalid_payload_length" }
            $base = Get-U64Le $body 0; $length = Get-U64Le $body 8
            if ($length -eq 0 -or $base -gt [uint64]::MaxValue - $length) { throw "surface_fact_wire_invalid_model" }
            $fact = [pscustomobject][ordered]@{ base = $base; length = $length; region_type = Get-U32Le $body 16 }
        }
        4 {
            if ($body.Length -lt 16) { throw "surface_fact_wire_invalid_payload_length" }
            $barCount = $body[15]
            if ($barCount -gt 6) { throw "surface_fact_wire_invalid_bar_count" }
            if ($body.Length -ne 16 + 19 * $barCount) { throw "surface_fact_wire_invalid_payload_length" }
            if ($body[3] -gt 31 -or $body[4] -gt 7 -or $body[14] -gt 4) { throw "surface_fact_wire_invalid_model" }
            $bars = @(); $seen = @{}; $reserved = @{}; $lastIndex = -1
            for ($barSlot = 0; $barSlot -lt $barCount; $barSlot++) {
                $barOffset = 16 + 19 * $barSlot; $barIndex = $body[$barOffset]; $barKind = $body[$barOffset + 1]
                $barBase = Get-U64Le $body ($barOffset + 2); $barLength = Get-U64Le $body ($barOffset + 10)
                $barStatus = $body[$barOffset + 18]
                if ($barIndex -le $lastIndex) { throw "surface_fact_wire_noncanonical_bar_order" }
                if ($barIndex -gt 5 -or $barKind -notin @(1, 2, 3) -or $barStatus -notin @(1, 2)) { throw "surface_fact_wire_invalid_bar_value" }
                if ($seen.ContainsKey($barIndex) -or $reserved.ContainsKey($barIndex) -or $barLength -eq 0 -or
                    $barBase -gt [uint64]::MaxValue - $barLength) { throw "surface_fact_wire_invalid_model" }
                if ($barKind -eq 3) {
                    if ($barIndex -eq 5 -or $seen.ContainsKey($barIndex + 1) -or $reserved.ContainsKey($barIndex + 1)) { throw "surface_fact_wire_invalid_model" }
                    $reserved[$barIndex + 1] = $true
                }
                if ($barKind -in @(1, 2) -and ($barBase + $barLength) -gt 4294967296) { throw "surface_fact_wire_invalid_model" }
                $seen[$barIndex] = $true; $lastIndex = $barIndex
                $bars += [pscustomobject][ordered]@{ index = $barIndex; kind = @("", "io", "memory32", "memory64")[$barKind]
                    base = $barBase; length = $barLength; status = @("", "assigned", "disabled")[$barStatus] }
            }
            $fact = [pscustomobject][ordered]@{ segment = Get-U16Le $body 0; bus = $body[2]; device = $body[3]
                function = $body[4]; vendor_id = Get-U16Le $body 5; device_id = Get-U16Le $body 7
                class_code = $body[9]; subclass = $body[10]; prog_if = $body[11]; revision = $body[12]
                irq_line = $body[13]; irq_pin = $body[14]; bars = @($bars) }
        }
        255 {
            if ($body.Length -ne 40) { throw "surface_fact_wire_invalid_payload_length" }
            $fact = [pscustomobject][ordered]@{ cpu_parts = Get-U16Le $body 0; smbios_memory_parts = Get-U16Le $body 2
                limine_memory_region_parts = Get-U16Le $body 4; pci_device_parts = Get-U16Le $body 6
                facts_sha256 = ConvertTo-Hex (Get-Slice $body 8 32) }
        }
    }
    $digestBytes = New-Object 'System.Collections.Generic.List[byte]'
    foreach ($range in @(@(16, 2), @(20, 16), @(18, 1), @(36, 4), @(40, $payloadLength))) {
        $digestBytes.AddRange([byte[]](Get-Slice $Payload $range[0] $range[1]))
    }
    return [pscustomobject][ordered]@{ wire_version = $wireVersion; schema_version = $schemaVersion
        capture_id = ConvertTo-Hex $captureId; part_kind = $kind; part_index = $partIndex; part_count = $partCount
        fact = $fact; digest_bytes = $digestBytes.ToArray() }
}

function Complete-SurfaceFactSeries {
    param([object[]]$Parts, [object[]]$Sources)
    if ($Parts.Count -eq 0) { return $null }
    $completion = $Parts[-1]
    if ($completion.part_kind -ne 255 -or $completion.part_index + 1 -ne $completion.part_count) { throw "surface_capture_completion_not_last" }
    $counts = [ordered]@{ cpu = 0; smbios_memory = 0; limine_memory_region = 0; pci_device = 0 }
    $cpu = @(); $smbios = @(); $limine = @(); $pci = @()
    $semantic = New-Object 'System.Collections.Generic.List[byte]'
    $semantic.AddRange([Text.Encoding]::ASCII.GetBytes("raios.surface_fact_capture.facts.v1`0"))
    foreach ($part in $Parts) {
        switch ($part.part_kind) {
            1 { $counts.cpu++; $cpu += $part.fact; $semantic.AddRange([byte[]]$part.digest_bytes) }
            2 { $counts.smbios_memory++; $smbios += $part.fact; $semantic.AddRange([byte[]]$part.digest_bytes) }
            3 { $counts.limine_memory_region++; $limine += $part.fact; $semantic.AddRange([byte[]]$part.digest_bytes) }
            4 { $counts.pci_device++; $pci += $part.fact; $semantic.AddRange([byte[]]$part.digest_bytes) }
        }
    }
    foreach ($name in @("cpu", "smbios_memory", "limine_memory_region", "pci_device")) {
        if ($counts[$name] -eq 0) { throw "surface_capture_missing_$name" }
    }
    $expected = $completion.fact
    if ($counts.cpu -ne $expected.cpu_parts -or $counts.smbios_memory -ne $expected.smbios_memory_parts -or
        $counts.limine_memory_region -ne $expected.limine_memory_region_parts -or $counts.pci_device -ne $expected.pci_device_parts) {
        throw "surface_capture_completion_count_mismatch"
    }
    $factsSha = ConvertTo-Hex (Get-Sha256Bytes $semantic.ToArray())
    if ($factsSha -cne $expected.facts_sha256) { throw "surface_capture_digest_mismatch" }
    $candidate = [ordered]@{ schema = "raios.surface_fact_manifest_candidate.v1"; capture_id = $Parts[0].capture_id
        wire_version = 1; fact_schema_version = 1; part_count = $Parts.Count; part_counts = [pscustomobject]$counts
        facts_sha256 = $factsSha; cpu = @($cpu); smbios_memory = @($smbios); limine_memory_map = @($limine)
        pci_devices = @($pci); raw_source = @($Sources) }
    $candidateJson = ([pscustomobject]$candidate | ConvertTo-Json -Depth 10 -Compress)
    $candidate["candidate_sha256"] = ConvertTo-Hex (Get-Sha256Bytes ([Text.Encoding]::UTF8.GetBytes($candidateJson)))
    return [pscustomobject]$candidate
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
    $surfaceParts = @()
    $surfaceSources = @()
    $surfaceComplete = $false
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
        $isSurface = $payload.Length -ge 8 -and (Test-BytesEqual (Get-Slice $payload 0 8) $SurfaceFactMagic)
        if (-not $isSurface -and $payload.Length -gt 0) {
            $prefixLength = [Math]::Min(8, $payload.Length)
            if (Test-BytesEqual (Get-Slice $payload 0 $prefixLength) (Get-Slice $SurfaceFactMagic 0 $prefixLength)) {
                throw "surface_fact_wire_truncated"
            }
            for ($probe = 1; $probe + 8 -le $payload.Length; $probe++) {
                if (Test-BytesEqual (Get-Slice $payload $probe 8) $SurfaceFactMagic) {
                    throw "surface_fact_wire_invalid_magic"
                }
            }
        }
        if ($isSurface) {
            if ($surfaceComplete) { throw "surface_capture_after_completion" }
            $part = Convert-SurfaceFactRecord $payload
            if ($surfaceParts.Count -eq 0) {
                if ($part.part_index -ne 0) { throw "surface_capture_missing_part" }
            }
            else {
                $first = $surfaceParts[0]
                if ($part.wire_version -ne $first.wire_version -or $part.schema_version -ne $first.schema_version -or
                    $part.capture_id -cne $first.capture_id -or $part.part_count -ne $first.part_count) { throw "surface_capture_mixed_series" }
                if ($part.part_index -ne $surfaceParts.Count) { throw "surface_capture_noncontiguous_part" }
            }
            $surfaceParts += $part
            $surfaceSources += [pscustomobject][ordered]@{ part_index = $part.part_index; reclog_sequence = $sequence
                absolute_lba = $AbsoluteStartLba + [uint64]($offset / $SectorSize); payload_sha256 = ConvertTo-Hex $payloadHash
                frame_sha256 = ConvertTo-Hex $frameHash }
            if ($part.part_kind -eq 255) {
                if ($surfaceParts.Count -ne $part.part_count) { throw "surface_capture_missing_part" }
                $surfaceComplete = $true
            }
        }
        elseif ($surfaceParts.Count -gt 0 -and -not $surfaceComplete) { throw "surface_capture_foreign_frame" }
        $schemaPrefix = [Text.Encoding]::UTF8.GetBytes('{"schema":"raios.hw_failure_trace.v0",')
        if ($payload.Length -ge $schemaPrefix.Length -and
            (Test-BytesEqual (Get-Slice $payload 0 $schemaPrefix.Length) $schemaPrefix)) {
            $trace = Convert-ValidatedTracePayload $payload
            $decodedSteps = @()
            $isK2Publication = $false
            foreach ($stepValue in @($trace.steps)) {
                $step = @($stepValue)
                $phase = [int]$step[1]
                $status = [int]$step[2]
                $register = [int]$step[3]
                if ($status -in @(110, 111, 112) -or
                    ($status -eq 1 -and $register -eq 7) -or
                    $register -eq 8) {
                    $isK2Publication = $true
                }
                $decodedStep = [ordered]@{
                    boot_ms = [uint32]$step[0]
                    phase = $phase
                    status = $status
                    status_name = Get-HwFailureStatusName $status
                    register = $register
                    register_name = Get-HwFailureRegisterName $register
                    register_value = [uint32]$step[4]
                }
                if ($register -eq 9) {
                    $decodedStep["connection_timeout_fingerprint"] =
                        Convert-ConnectionTimeoutFingerprint ([uint32]$step[4])
                }
                if ($phase -eq 5 -and $status -eq 100 -and $register -eq 8) {
                    $decodedStep["associate_doorbell_ack"] =
                        Convert-AssociateDoorbellAck ([uint32]$step[4])
                }
                if ($phase -eq 2 -and $status -eq 109 -and $register -eq 4) {
                    $decodedStep["post_pmk_hw_spec_canary"] =
                        Convert-PostPmkHwSpecCanaryResult ([uint32]$step[4])
                }
                $decodedSteps += [pscustomobject]$decodedStep
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
    if ($surfaceParts.Count -gt 0 -and -not $surfaceComplete) { throw "surface_capture_missing_completion" }
    $surfaceCandidate = Complete-SurfaceFactSeries -Parts $surfaceParts -Sources $surfaceSources
    return [pscustomobject]@{
        records = @($records)
        usb_diagnostics = @($usbDiagnostics)
        surface_fact_candidate = $surfaceCandidate
        valid_frame_count = [uint64]($expectedSeq - 1)
        tail_status = $tailStatus
    }
}

function New-SelfTestFrame {
    param([string]$PayloadText, [uint64]$Sequence, [byte[]]$PreviousFrameHash)
    $payload = [Text.Encoding]::UTF8.GetBytes($PayloadText)
    return New-SelfTestBinaryFrame -Payload $payload -Sequence $Sequence -PreviousFrameHash $PreviousFrameHash
}

function New-SelfTestBinaryFrame {
    param([byte[]]$Payload, [uint64]$Sequence, [byte[]]$PreviousFrameHash)
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

function Set-SelfTestBytes {
    param([byte[]]$Target, [int]$Offset, [byte[]]$Value)
    [Array]::Copy($Value, 0, $Target, $Offset, $Value.Length)
}

function New-SurfaceSelfTestRecord {
    param([byte]$Kind, [uint16]$Index, [uint16]$Count, [byte[]]$CaptureId, [byte[]]$Body)
    $record = New-Object byte[] (40 + $Body.Length)
    Set-SelfTestBytes $record 0 $SurfaceFactMagic
    Set-SelfTestBytes $record 8 ([BitConverter]::GetBytes([uint16]1))
    Set-SelfTestBytes $record 10 ([BitConverter]::GetBytes([uint16]40))
    Set-SelfTestBytes $record 12 ([BitConverter]::GetBytes([uint16]$record.Length))
    Set-SelfTestBytes $record 14 ([BitConverter]::GetBytes([uint16]$Body.Length))
    Set-SelfTestBytes $record 16 ([BitConverter]::GetBytes([uint16]1))
    $record[18] = $Kind
    Set-SelfTestBytes $record 20 $CaptureId
    Set-SelfTestBytes $record 36 ([BitConverter]::GetBytes($Index))
    Set-SelfTestBytes $record 38 ([BitConverter]::GetBytes($Count))
    Set-SelfTestBytes $record 40 $Body
    return $record
}

function Get-SurfaceSelfTestDigest {
    param([object[]]$Records)
    $bytes = New-Object 'System.Collections.Generic.List[byte]'
    $bytes.AddRange([Text.Encoding]::ASCII.GetBytes("raios.surface_fact_capture.facts.v1`0"))
    foreach ($item in $Records) {
        $record = [byte[]]$item.bytes
        $bodyLength = $record.Length - 40
        foreach ($range in @(@(16, 2), @(20, 16), @(18, 1), @(36, 4), @(40, $bodyLength))) {
            $bytes.AddRange([byte[]](Get-Slice $record $range[0] $range[1]))
        }
    }
    return Get-Sha256Bytes $bytes.ToArray()
}

function New-SurfaceSelfTestPayloads {
    $captureId = [byte[]](0..15 | ForEach-Object { 0x38 })
    $count = [uint16]5
    $cpu = New-Object byte[] 24
    foreach ($pair in @(@(0, 1), @(4, 0), @(8, 1), @(12, 2), @(16, 3), @(20, 4))) {
        Set-SelfTestBytes $cpu $pair[0] ([BitConverter]::GetBytes([uint32]$pair[1]))
    }
    $locator = [Text.Encoding]::ASCII.GetBytes("DIMM 0")
    $smbios = New-Object byte[] (19 + $locator.Length)
    Set-SelfTestBytes $smbios 0 ([BitConverter]::GetBytes([uint16]1)); Set-SelfTestBytes $smbios 2 ([BitConverter]::GetBytes([uint16]2))
    Set-SelfTestBytes $smbios 4 ([BitConverter]::GetBytes([uint64]8589934592)); Set-SelfTestBytes $smbios 12 ([BitConverter]::GetBytes([uint32]1600))
    $smbios[16] = 24; $smbios[17] = 9; $smbios[18] = $locator.Length; Set-SelfTestBytes $smbios 19 $locator
    $limine = New-Object byte[] 20
    Set-SelfTestBytes $limine 0 ([BitConverter]::GetBytes([uint64]1048576)); Set-SelfTestBytes $limine 8 ([BitConverter]::GetBytes([uint64]4194304))
    Set-SelfTestBytes $limine 16 ([BitConverter]::GetBytes([uint32]0))
    $pci = New-Object byte[] 54
    Set-SelfTestBytes $pci 0 ([BitConverter]::GetBytes([uint16]0)); $pci[2] = 1; $pci[3] = 0; $pci[4] = 0
    Set-SelfTestBytes $pci 5 ([BitConverter]::GetBytes([uint16]0x11ab)); Set-SelfTestBytes $pci 7 ([BitConverter]::GetBytes([uint16]0x2b38))
    $pci[9] = 2; $pci[10] = 0; $pci[11] = 0; $pci[12] = 1; $pci[13] = 11; $pci[14] = 1; $pci[15] = 2
    $pci[16] = 0; $pci[17] = 3; Set-SelfTestBytes $pci 18 ([BitConverter]::GetBytes([uint64]4294967296)); Set-SelfTestBytes $pci 26 ([BitConverter]::GetBytes([uint64]4096)); $pci[34] = 1
    $pci[35] = 2; $pci[36] = 2; Set-SelfTestBytes $pci 37 ([BitConverter]::GetBytes([uint64]4026531840)); Set-SelfTestBytes $pci 45 ([BitConverter]::GetBytes([uint64]4096)); $pci[53] = 2
    $records = @(
        [pscustomobject]@{ bytes = New-SurfaceSelfTestRecord 1 0 $count $captureId $cpu },
        [pscustomobject]@{ bytes = New-SurfaceSelfTestRecord 2 1 $count $captureId $smbios },
        [pscustomobject]@{ bytes = New-SurfaceSelfTestRecord 3 2 $count $captureId $limine },
        [pscustomobject]@{ bytes = New-SurfaceSelfTestRecord 4 3 $count $captureId $pci }
    )
    $completion = New-Object byte[] 40
    foreach ($offset in @(0, 2, 4, 6)) { Set-SelfTestBytes $completion $offset ([BitConverter]::GetBytes([uint16]1)) }
    Set-SelfTestBytes $completion 8 (Get-SurfaceSelfTestDigest $records)
    $records += [pscustomobject]@{ bytes = New-SurfaceSelfTestRecord 255 4 $count $captureId $completion }
    return ,$records
}

function New-SurfaceSelfTestRegion {
    param([object[]]$Payloads)
    $region = New-Object byte[] ($SectorSize * $Payloads.Count)
    $previous = New-Object byte[] 32
    for ($index = 0; $index -lt $Payloads.Count; $index++) {
        $frame = New-SelfTestBinaryFrame -Payload ([byte[]]$Payloads[$index].bytes) -Sequence ([uint64]($index + 1)) -PreviousFrameHash $previous
        [Array]::Copy($frame, 0, $region, $index * $SectorSize, $SectorSize)
        $previous = Get-Sha256Bytes $frame
    }
    return $region
}

function Assert-SurfaceSelfTestRejected {
    param([object[]]$Payloads, [string]$Expected)
    try {
        [void](Read-ReclogTraces -Region (New-SurfaceSelfTestRegion $Payloads) -AbsoluteStartLba 200)
        throw "selftest_surface_negative_not_rejected"
    }
    catch { if ($_.Exception.Message -cne $Expected) { throw } }
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

    $payload = '{"schema":"raios.hw_failure_trace.v0","classification":"local_only","scope":"current_boot","build_id":65536,"subsystem":1,"steps":[[10,1,1,7,1026],[20,1,111,8,2],[30,5,100,1,0],[30,5,100,9,2797897741]]}'
    $frame = New-SelfTestFrame -PayloadText $payload -Sequence 1 -PreviousFrameHash (New-Object byte[] 32)
    $doorbellPayload = '{"schema":"raios.hw_failure_trace.v0","classification":"local_only","scope":"current_boot","build_id":65536,"subsystem":1,"steps":[[30,5,100,1,0],[30,5,100,9,2797897804],[30,5,100,8,3523280896]]}'
    $doorbellFrame = New-SelfTestFrame -PayloadText $doorbellPayload -Sequence 2 -PreviousFrameHash (Get-Sha256Bytes $frame)
    $canaryValue = [Convert]::ToUInt32("d2250000", 16)
    $canaryPayload = '{"schema":"raios.hw_failure_trace.v0","classification":"local_only","scope":"current_boot","build_id":65536,"subsystem":1,"steps":[[40,2,109,4,' + $canaryValue + ']]}'
    $canaryFrame = New-SelfTestFrame -PayloadText $canaryPayload -Sequence 3 -PreviousFrameHash (Get-Sha256Bytes $doorbellFrame)
    $usbPayload = '{"schema":"raios.usb_diag.v0","classification":"local_only","scope":"current_boot","reason":"boot_probe","seq":3,"hub_count":1,"hub_ports":4,"hub_connected":2,"hub_reset":1,"hub_done":2,"recover":0,"reports":12,"errors":0,"last_int_cc":1,"last_xfer_cc":1,"last_cmd":42,"last_cc":1,"enum_vid":4660,"enum_pid":22136,"m_port":1,"m_chg":0,"m_ep":1}'
    $usbFrame = New-SelfTestFrame -PayloadText $usbPayload -Sequence 4 -PreviousFrameHash (Get-Sha256Bytes $canaryFrame)
    $region = New-Object byte[] ($SectorSize * 5)
    [Array]::Copy($frame, 0, $region, 0, $SectorSize)
    [Array]::Copy($doorbellFrame, 0, $region, $SectorSize, $SectorSize)
    [Array]::Copy($canaryFrame, 0, $region, $SectorSize * 2, $SectorSize)
    [Array]::Copy($usbFrame, 0, $region, $SectorSize * 3, $SectorSize)
    $parsed = Read-ReclogTraces -Region $region -AbsoluteStartLba 100
    if ($parsed.records.Count -ne 3 -or
        $parsed.records[0].kind -cne "k2_publication" -or
        $parsed.records[0].decoded_steps[0].register_name -cne "marvell_pci_command" -or
        $parsed.records[0].decoded_steps[1].status_name -cne "k2_publication_rejected" -or
        $parsed.records[0].decoded_steps[2].register_name -cne "marvell_host_interrupt_status" -or
        $parsed.records[0].decoded_steps[3].register_name -cne "marvell_connection_timeout_fingerprint" -or
        $parsed.records[0].decoded_steps[3].connection_timeout_fingerprint.connection_stage -cne "associate" -or
        $parsed.records[0].decoded_steps[3].connection_timeout_fingerprint.expected_command_hex -cne "0x0012" -or
        $parsed.records[0].decoded_steps[3].connection_timeout_fingerprint.published_command_len -ne 128 -or
        $parsed.records[0].decoded_steps[3].connection_timeout_fingerprint.request_header_matches_expected -ne $true -or
        $parsed.records[0].decoded_steps[3].connection_timeout_fingerprint.verified_quiesce_cleanup -cne "succeeded" -or
        $parsed.records[0].decoded_steps[3].connection_timeout_fingerprint.response_class -cne "expected_header_seen" -or
        $parsed.records[1].decoded_steps.Count -ne 3 -or
        $parsed.records[1].decoded_steps[0].register_name -cne "marvell_host_interrupt_status" -or
        $parsed.records[1].decoded_steps[1].register_name -cne "marvell_connection_timeout_fingerprint" -or
        $parsed.records[1].decoded_steps[1].connection_timeout_fingerprint.connection_stage -cne "associate" -or
        $parsed.records[1].decoded_steps[1].connection_timeout_fingerprint.expected_command_hex -cne "0x0012" -or
        $parsed.records[1].decoded_steps[1].connection_timeout_fingerprint.published_command_len -ne 132 -or
        $parsed.records[1].decoded_steps[1].connection_timeout_fingerprint.response_class -cne "untouched_zero" -or
        $parsed.records[1].decoded_steps[2].register_name -cne "marvell_publication_step" -or
        $parsed.records[1].decoded_steps[2].associate_doorbell_ack.classification -cne "cleared" -or
        $parsed.records[2].decoded_steps.Count -ne 1 -or
        $parsed.records[2].decoded_steps[0].post_pmk_hw_spec_canary.outcome -cne "expected_completion" -or
        $parsed.records[2].decoded_steps[0].post_pmk_hw_spec_canary.network_state_granted -ne $false -or
        $parsed.records[2].decoded_steps[0].post_pmk_hw_spec_canary.cold_reboot_required -ne $true -or
        $parsed.usb_diagnostics.Count -ne 1 -or
        $parsed.usb_diagnostics[0].diagnostic.reason -cne "boot_probe" -or
        $parsed.tail_status -cne "zero_tail") {
        throw "selftest_positive_failed"
    }

    $surfacePayloads = New-SurfaceSelfTestPayloads
    $surfaceRegion = New-SurfaceSelfTestRegion $surfacePayloads
    $surfaceParsed = Read-ReclogTraces -Region $surfaceRegion -AbsoluteStartLba 200
    $surfaceParsedAgain = Read-ReclogTraces -Region $surfaceRegion -AbsoluteStartLba 200
    $candidate = $surfaceParsed.surface_fact_candidate
    if ($candidate.schema -cne "raios.surface_fact_manifest_candidate.v1" -or
        $candidate.capture_id -cne ("38" * 16) -or $candidate.part_count -ne 5 -or
        $candidate.part_counts.cpu -ne 1 -or $candidate.part_counts.smbios_memory -ne 1 -or
        $candidate.part_counts.limine_memory_region -ne 1 -or $candidate.part_counts.pci_device -ne 1 -or
        $candidate.smbios_memory[0].device_locator -cne "DIMM 0" -or
        $candidate.pci_devices[0].bars[0].index -ne 0 -or $candidate.pci_devices[0].bars[1].index -ne 2 -or
        $candidate.raw_source.Count -ne 5 -or $candidate.raw_source[0].reclog_sequence -ne 1 -or
        $candidate.raw_source[4].reclog_sequence -ne 5 -or $candidate.candidate_sha256.Length -ne 64 -or
        $candidate.facts_sha256 -cne "c4206efc850258fdd067d9e5bce3433e402531477c4fb1f657f5ca102fa51148" -or
        $candidate.candidate_sha256 -cne "2ccdb89cb85baa807b089f36b9ec771047437c0c2d9046b80740472fbeaa57f8" -or
        $candidate.candidate_sha256 -cne $surfaceParsedAgain.surface_fact_candidate.candidate_sha256) {
        throw "selftest_surface_positive_failed"
    }

    $case = New-SurfaceSelfTestPayloads
    Assert-SurfaceSelfTestRejected @($case[0], $case[2], $case[3], $case[4]) "surface_capture_noncontiguous_part"
    $case = New-SurfaceSelfTestPayloads
    $case[2].bytes[36] = 1; $case[2].bytes[37] = 0
    Assert-SurfaceSelfTestRejected $case "surface_capture_noncontiguous_part"
    $case = New-SurfaceSelfTestPayloads
    $case[1].bytes[20] = $case[1].bytes[20] -bxor 1
    Assert-SurfaceSelfTestRejected $case "surface_capture_mixed_series"
    $case = New-SurfaceSelfTestPayloads
    $case[4].bytes = Get-Slice $case[4].bytes 0 ($case[4].bytes.Length - 1)
    Assert-SurfaceSelfTestRejected $case "surface_fact_wire_truncated"
    $case = New-SurfaceSelfTestPayloads
    $case[3].bytes[75] = 0
    Assert-SurfaceSelfTestRejected $case "surface_fact_wire_noncanonical_bar_order"
    $case = New-SurfaceSelfTestPayloads
    $case[0].bytes[40] = $case[0].bytes[40] -bxor 1
    Assert-SurfaceSelfTestRejected $case "surface_capture_digest_mismatch"
    $case = New-SurfaceSelfTestPayloads
    Assert-SurfaceSelfTestRejected @($case[0], [pscustomobject]@{ bytes = [Text.Encoding]::ASCII.GetBytes("foreign") }, $case[1], $case[2], $case[3], $case[4]) "surface_capture_foreign_frame"
    $case = New-SurfaceSelfTestPayloads
    Assert-SurfaceSelfTestRejected @($case + $case[0]) "surface_capture_after_completion"

    $validFingerprint = [Convert]::ToUInt32("A6C4880D", 16)
    $mutatedFingerprint = [uint32]($validFingerprint -bxor [Convert]::ToUInt32("80000000", 16))
    try {
        [void](Convert-ConnectionTimeoutFingerprint $mutatedFingerprint)
        throw "selftest_connection_timeout_fingerprint_mutation_not_rejected"
    }
    catch {
        if ($_.Exception.Message -cne "connection_timeout_fingerprint_tag_invalid") {
            throw
        }
    }

    $doorbellCases = [ordered]@{
        "d2010000" = "cleared"
        "d2010001" = "still_set"
        "d2010002" = "unavailable"
    }
    foreach ($case in $doorbellCases.GetEnumerator()) {
        $decoded = Convert-AssociateDoorbellAck ([Convert]::ToUInt32($case.Key, 16))
        if ($decoded.classification -cne $case.Value) {
            throw "selftest_associate_doorbell_ack_classification_failed"
        }
    }
    foreach ($mutation in @(
        @("c2010000", "associate_doorbell_ack_tag_invalid"),
        @("d2010003", "associate_doorbell_ack_value_invalid")
    )) {
        try {
            [void](Convert-AssociateDoorbellAck ([Convert]::ToUInt32($mutation[0], 16)))
            throw "selftest_associate_doorbell_ack_mutation_not_rejected"
        }
        catch {
            if ($_.Exception.Message -cne $mutation[1]) {
                throw
            }
        }
    }

    $canaryCases = [ordered]@{
        "d2250000" = "expected_completion"
        "d2250001" = "firmware_result"
        "d2250002" = "malformed_or_wrong_completion"
        "d2250003" = "timeout_doorbell_cleared"
        "d2250004" = "timeout_doorbell_still_set"
        "d2250005" = "mmio_or_doorbell_unavailable"
        "d2250006" = "stale_high_completion"
        "d2250007" = "host_publication_failure"
    }
    foreach ($case in $canaryCases.GetEnumerator()) {
        $decoded = Convert-PostPmkHwSpecCanaryResult ([Convert]::ToUInt32($case.Key, 16))
        if ($decoded.outcome -cne $case.Value -or $decoded.network_state_granted -ne $false -or
            $decoded.cold_reboot_required -ne $true) {
            throw "selftest_post_pmk_hw_spec_canary_classification_failed"
        }
    }
    foreach ($mutation in @(
        @("c2250000", "post_pmk_hw_spec_canary_tag_invalid"),
        @("d2250008", "post_pmk_hw_spec_canary_value_invalid")
    )) {
        try {
            [void](Convert-PostPmkHwSpecCanaryResult ([Convert]::ToUInt32($mutation[0], 16)))
            throw "selftest_post_pmk_hw_spec_canary_mutation_not_rejected"
        }
        catch {
            if ($_.Exception.Message -cne $mutation[1]) {
                throw
            }
        }
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
        checks = @("gpt_guid_known_vector", "gpt_guid_invalid_length_rejected", "powershell51_crc32", "crc32_mutation_rejected", "verified_readback", "k2_decoded", "connection_timeout_fingerprint_decoded", "connection_timeout_fingerprint_bit_mutation_rejected", "associate_doorbell_ack_decoded", "associate_doorbell_ack_tag_mutation_rejected", "associate_doorbell_ack_value_mutation_rejected", "post_pmk_hw_spec_canary_decoded", "post_pmk_hw_spec_canary_tag_mutation_rejected", "post_pmk_hw_spec_canary_value_mutation_rejected", "usb_diag_visible", "legacy_negative_usb_diag_rejected", "surface_fact_series_deterministic", "surface_fact_missing_part_rejected", "surface_fact_duplicate_part_rejected", "surface_fact_mixed_capture_rejected", "surface_fact_truncated_completion_rejected", "surface_fact_noncanonical_bar_order_rejected", "surface_fact_semantic_mutation_rejected", "surface_fact_foreign_frame_rejected", "surface_fact_after_completion_rejected", "torn_rejected", "secret_field_rejected", "full_region_reported")
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
        surface_fact_candidate = $scan.surface_fact_candidate
    } | ConvertTo-Json -Depth 10 -Compress
}
finally {
    $stream.Dispose()
}
