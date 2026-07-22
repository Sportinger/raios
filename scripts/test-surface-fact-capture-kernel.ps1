param([switch]$SkipBuild)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot

function Require([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Require-Match([string]$Text, [string]$Pattern, [string]$Message) {
    Require ([regex]::IsMatch($Text, $Pattern, [Text.RegularExpressions.RegexOptions]::Singleline)) $Message
}

function Slice-Between([string]$Text, [string]$Start, [string]$End) {
    $startAt = $Text.IndexOf($Start, [StringComparison]::Ordinal)
    $endAt = $Text.IndexOf($End, $startAt + $Start.Length, [StringComparison]::Ordinal)
    Require (($startAt -ge 0) -and ($endAt -gt $startAt)) "missing source slice $Start"
    $Text.Substring($startAt, $endAt - $startAt)
}

$main = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "seed-kernel\src\main.rs")
$capture = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "seed-kernel\src\surface_fact_capture.rs")
$usb = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "seed-kernel\src\usb.rs")

Require-Match $main 'static SMBIOS_REQUEST: SmbiosRequest.*surface_fact_capture::capture\(.*SMBIOS_REQUEST\.get_response\(\).*MEMORY_MAP_REQUEST\.get_response\(\).*HHDM_REQUEST\.get_response\(\).*usb::init\(\).*append_surface_fact_capture' "boot ordering or SMBIOS request missing"
Require ([regex]::Matches($capture, 'pci::enumerate_functions_for_capture\(\)\?').Count -eq 1) "capture must use fail-closed PCI enumeration exactly once"
Require ([regex]::Matches($capture, 'pci::enumerate_functions\(').Count -eq 0) "infallible PCI enumeration remains reachable"
Require-Match $capture 'revision:\s*function\.revision' "captured PCI revision is not the accepted snapshot value"
Require ($capture -notmatch 'PciAddress|read_u8\(0x08\)') "capture performs a later PCI config reread"
Require-Match $capture 'cpuid\(0, 0\).*basic\.eax >= 1.*cpuid\(1, 0\).*basic\.eax >= 7.*cpuid\(7, 0\).*leaf7\.eax > CPUID_LEAF7_MAX_SUBLEAF.*1\.\.=leaf7\.eax.*cpuid\(7, subleaf\).*cpuid\(0x8000_0000, 0\).*extended\.eax >= 0x8000_0001.*cpuid\(0x8000_0001, 0\).*0x8000_0002\.\.=0x8000_0004' "bounded CPUID leaf set incomplete"
Require-Match $capture 'EntryType::USABLE.*EntryType::RESERVED.*EntryType::ACPI_RECLAIMABLE.*EntryType::ACPI_NVS.*EntryType::BAD_MEMORY.*EntryType::BOOTLOADER_RECLAIMABLE.*EntryType::EXECUTABLE_AND_MODULES.*EntryType::FRAMEBUFFER.*memory_type_unknown' "Limine map type mapping is not fail-closed"
Require-Match $capture 'SMBIOS_ENTRY_MAX_LEN:\s*usize = 256.*SMBIOS_SLICE_MAX_LEN:\s*usize = 1024 \* 1024.*entry_64\(\).*_SM3_.*SMBIOS64_MIN_LEN\.\.=SMBIOS_ENTRY_MAX_LEN.*checksum.*entry_32\(\).*_SM_.*SMBIOS32_MIN_LEN\.\.=SMBIOS_ENTRY_MAX_LEN.*_DMI_.*checksum' "SMBIOS entry validation surface incomplete"
Require-Match $capture 'declared_structure_count.*structure_count.*type17_handles.*smbios_end_missing.*smbios32_structure_count.*smbios32_type127_not_final' "SMBIOS count, final terminator, or duplicate-handle rules missing"
Require-Match $capture 'length > SMBIOS_SLICE_MAX_LEN \|\| length > isize::MAX as usize.*physical\s*\.checked_add\(length64\).*authorize_smbios_interval.*virtual_address\s*\.checked_add\(length64\).*is_canonical_x86_64.*from_raw_parts' "bounded physical slice authorization missing"
Require ([regex]::Matches($capture, 'checked_add\(hhdm\.offset\(\)\)').Count -eq 1) "SMBIOS physical pointers are translated more than once"
Require-Match $capture 'entry\.base\s*\.checked_add\(entry\.length\).*entry\.base <= physical && physical_end <= entry_end.*memory_map_ambiguous.*EntryType::RESERVED.*EntryType::ACPI_RECLAIMABLE.*EntryType::ACPI_NVS.*EntryType::BOOTLOADER_RECLAIMABLE.*memory_map_type' "Limine authorization is not one-entry fail-closed"
Require-Match $capture 'raw_size == 0x7fff.*read_u32\(formatted, 28\).*raw_speed == 0xffff.*read_u32\(formatted, 84\)' "SMBIOS extended size/speed handling missing"
Require-Match $capture 'capture_id\(nonce.*canonical_facts_sha256.*validate_capture' "capture identity/digest/final validation missing"

$series = Slice-Between $usb "// SAFETY: Reason: this reuses initialized xHCI DMA." "/// Appends one canonical RAIOSSF0"
$append = Slice-Between $usb "// SAFETY: Reason: MSC commands access static DMA." "/// Uses the append point proven"
$entry = Slice-Between $usb "pub fn append_surface_fact_capture" "pub fn rescan_if_input_missing"
Require-Match $series 'validate_capture\(records\).*part_index as usize != position.*part_count != validated\.part_count.*capture_id != validated\.capture_id.*CapturePayload::Completion.*for record in records.*encode_surface_fact.*decode_surface_fact.*wire_preflight_mismatch.*record_count.*checked_add\(record_count\).*for offset in 0\.\.record_count.*target_not_zero.*for \(payload, length\) in encoded.*append_surface_fact_payload' "series physical-order or all-record preflight missing"
Require-Match $append 'SURFACE_FACT_WIRE_MAGIC.*decode_surface_fact.*reclog_append_point\s*\.take\(\).*msc_write_sector_from_buffer\(point\.lba\).*msc_read_sector_copy\(point\.lba\).*readback != frame.*parse_reclog_frame.*decode_surface_fact\(.*readback.*readback_wire_parse_failed.*self\.reclog_append_point = Some' "narrow poison/readback/reparse append missing"
Require-Match $series 'if let Err\(reason\) = self\.append_surface_fact_payload.*self\.reclog_append_point = None' "series failure does not poison the shared append point"
Require-Match $usb 'SCSI_WRITE10_FUA:\s*u8 = 1 << 3.*0x2A,\s*SCSI_WRITE10_FUA' "WRITE(10) FUA missing"

function Sha256([byte[]]$Bytes) {
    $sha = [Security.Cryptography.SHA256]::Create()
    try { ,$sha.ComputeHash($Bytes) } finally { $sha.Dispose() }
}

function Write-Le32([byte[]]$Bytes, [int]$Offset, [uint32]$Value) {
    [BitConverter]::GetBytes($Value).CopyTo($Bytes, $Offset)
}

function Write-Le64([byte[]]$Bytes, [int]$Offset, [uint64]$Value) {
    [BitConverter]::GetBytes($Value).CopyTo($Bytes, $Offset)
}

function New-Frame([byte[]]$Payload, [uint64]$Sequence, [byte[]]$Previous) {
    $frame = [byte[]]::new(512)
    [Text.Encoding]::ASCII.GetBytes("RAIOSRC0").CopyTo($frame, 0)
    Write-Le32 $frame 8 512
    Write-Le32 $frame 12 $Payload.Length
    Write-Le64 $frame 16 $Sequence
    $Previous.CopyTo($frame, 24)
    (Sha256 $Payload).CopyTo($frame, 56)
    $Payload.CopyTo($frame, 88)
    $frame
}

function Test-Frame([byte[]]$Frame, [uint64]$Sequence, [byte[]]$Previous) {
    if ([Text.Encoding]::ASCII.GetString($Frame, 0, 8) -cne "RAIOSRC0") { return $false }
    if ([BitConverter]::ToUInt64($Frame, 16) -ne $Sequence) { return $false }
    for ($i = 0; $i -lt 32; $i++) { if ($Frame[24 + $i] -ne $Previous[$i]) { return $false } }
    $length = [BitConverter]::ToUInt32($Frame, 12)
    $payload = [byte[]]::new($length)
    [Array]::Copy($Frame, 88, $payload, 0, $length)
    $hash = Sha256 $payload
    for ($i = 0; $i -lt 32; $i++) { if ($Frame[56 + $i] -ne $hash[$i]) { return $false } }
    for ($i = 88 + $length; $i -lt 512; $i++) { if ($Frame[$i] -ne 0) { return $false } }
    $true
}

# Positive series model: each verified record advances both sequence and hash.
$previous = [byte[]]::new(32)
$sequence = [uint64]1
foreach ($kind in 1, 2, 3, 4, 255) {
    $payload = [byte[]]::new(40)
    [Text.Encoding]::ASCII.GetBytes("RAIOSSF0").CopyTo($payload, 0)
    $payload[18] = $kind
    $frame = New-Frame $payload $sequence $previous
    Require (Test-Frame $frame $sequence $previous) "positive RECLOG frame rejected"
    $next = Sha256 $frame
    Require ([Convert]::ToBase64String($next) -cne [Convert]::ToBase64String($previous)) "RECLOG hash did not advance"
    $previous = $next
    $sequence++
}
Require ($sequence -eq 6) "positive series did not append all records"

# A missing or checksum-invalid entry point cannot produce any append calls.
function Capture-AppendCount([AllowNull()][byte[]]$Entry) {
    if (($null -eq $Entry) -or ($Entry.Length -lt 24)) { return 0 }
    $sum = 0
    foreach ($byte in $Entry) { $sum = ($sum + $byte) -band 0xff }
    if ($sum -ne 0) { return 0 }
    1
}
Require ((Capture-AppendCount $null) -eq 0) "missing SMBIOS produced an append"
$invalidEntry = [byte[]]::new(24); $invalidEntry[0] = 1
Require ((Capture-AppendCount $invalidEntry) -eq 0) "checksum-invalid SMBIOS produced an append"

function Test-CanonicalAddress([uint64]$Address) {
    $upper = $Address -shr 48
    if (($Address -band ([uint64]1 -shl 47)) -eq 0) { return $upper -eq 0 }
    $upper -eq 0xffff
}

function Test-PhysicalInterval(
    [uint64]$Physical,
    [uint64]$Length,
    [uint64]$HhdmOffset,
    [object[]]$Entries
) {
    if ($Physical -eq 0 -or $Length -eq 0 -or $Length -gt (1024 * 1024)) { return $false }
    if ($Physical -gt [uint64]::MaxValue - $Length) { return $false }
    if ($Physical -gt [uint64]::MaxValue - $HhdmOffset) { return $false }
    $physicalEnd = $Physical + $Length
    $virtual = $Physical + $HhdmOffset
    if ($virtual -gt [uint64]::MaxValue - $Length) { return $false }
    if (-not (Test-CanonicalAddress $virtual) -or
        -not (Test-CanonicalAddress ($virtual + $Length - 1))) { return $false }
    $containing = @($Entries | Where-Object {
        $_.Base -le $Physical -and $_.Length -le ([uint64]::MaxValue - $_.Base) -and
        $physicalEnd -le ($_.Base + $_.Length)
    })
    if ($containing.Count -ne 1) { return $false }
    $containing[0].Type -in @("reserved", "acpi_reclaimable", "acpi_nvs", "bootloader_reclaimable")
}

$reserved = [pscustomobject]@{ Base = [uint64]0x1000; Length = [uint64]0x1000; Type = "reserved" }
Require (Test-PhysicalInterval 0x1200 256 0 @($reserved)) "authorized SMBIOS interval rejected"
Require (-not (Test-PhysicalInterval 0x1200 ((1024 * 1024) + 1) 0 @($reserved))) "oversized SMBIOS interval accepted"
Require (-not (Test-PhysicalInterval 0x1200 256 0 @())) "unmapped SMBIOS interval accepted"
$usable = [pscustomobject]@{ Base = [uint64]0x1000; Length = [uint64]0x1000; Type = "usable" }
Require (-not (Test-PhysicalInterval 0x1200 256 0 @($usable))) "disallowed SMBIOS map type accepted"
Require (-not (Test-PhysicalInterval 0x1200 256 0 @($reserved, $reserved))) "ambiguous SMBIOS interval accepted"
Require (-not (Test-PhysicalInterval 0x1200 256 0x0000800000000000 @($reserved))) "noncanonical HHDM interval accepted"
Require (-not (Test-PhysicalInterval ([uint64]::MaxValue - 1) 4 0 @($reserved))) "overflowing SMBIOS interval accepted"

function Test-SmbiosSeries([int[]]$Kinds, [int[]]$Type17Handles, [int]$DeclaredCount, [bool]$RequireFinal) {
    if ($Kinds.Count -eq 0 -or -not $Kinds.Contains(127)) { return $false }
    if ($Type17Handles.Count -ne (@($Type17Handles | Sort-Object -Unique)).Count) { return $false }
    $terminator = [Array]::IndexOf($Kinds, 127)
    if ($DeclaredCount -ge 0 -and $Kinds.Count -ne $DeclaredCount) { return $false }
    if ($RequireFinal -and $terminator -ne ($Kinds.Count - 1)) { return $false }
    $true
}

Require (Test-SmbiosSeries @(0, 17, 127) @(0x10) 3 $true) "valid SMBIOS 2 model rejected"
Require (-not (Test-SmbiosSeries @(0, 17) @(0x10) 2 $true)) "missing Type 127 accepted"
Require (-not (Test-SmbiosSeries @(17, 17, 127) @(0x10, 0x10) 3 $true)) "duplicate Type-17 handle accepted"
Require (-not (Test-SmbiosSeries @(0, 127, 17) @(0x10) 3 $true)) "non-final SMBIOS 2 terminator accepted"
Require (Test-SmbiosSeries @(0, 127, 17) @(0x10) -1 $false) "bounded SMBIOS 3 tail rejected"
Require (32 -le 32) "CPUID leaf-7 cap model failed"
Require (-not (33 -le 32)) "CPUID leaf-7 cap overflow was truncated"

function Test-PhysicalOrder([int[]]$Indexes, [int]$Count, [int]$CompletionPosition) {
    if ($Indexes.Count -ne $Count -or $CompletionPosition -ne ($Count - 1)) { return $false }
    for ($position = 0; $position -lt $Indexes.Count; $position++) {
        if ($Indexes[$position] -ne $position) { return $false }
    }
    $true
}

Require (Test-PhysicalOrder @(0, 1, 2, 3) 4 3) "ordered physical series rejected"
Require (-not (Test-PhysicalOrder @(1, 0, 2, 3) 4 3)) "shuffled physical series accepted"
Require (-not (Test-PhysicalOrder @(0, 1, 2, 3) 4 2)) "non-final physical completion accepted"

function Invoke-AppendFailureModel([string]$Stage) {
    $point = "available"
    if ($Stage -eq "preflight") {
        return [pscustomobject]@{ Point = $point; CompletionWritten = $false }
    }
    $point = $null
    if ($Stage -in @("write", "readback", "outer_reparse", "inner_reparse")) {
        return [pscustomobject]@{ Point = $point; CompletionWritten = $false }
    }
    [pscustomobject]@{ Point = "advanced"; CompletionWritten = $true }
}

$preflightFailure = Invoke-AppendFailureModel "preflight"
Require ($preflightFailure.Point -eq "available") "preflight failure poisoned H25 append state"
foreach ($stage in "write", "readback", "outer_reparse", "inner_reparse") {
    $failure = Invoke-AppendFailureModel $stage
    Require (($null -eq $failure.Point) -and -not $failure.CompletionWritten) "$stage failure did not poison append state"
}

# Readback mismatch on a fact stops the loop before the final completion kind.
$writtenKinds = [Collections.Generic.List[int]]::new()
foreach ($kind in 1, 2, 3, 255) {
    $frame = New-Frame ([byte[]]@(82,65,73,79,83,83,70,48,$kind)) 1 ([byte[]]::new(32))
    $readback = [byte[]]$frame.Clone()
    if ($kind -eq 2) { $readback[100] = 1 }
    if ([Convert]::ToBase64String($frame) -cne [Convert]::ToBase64String($readback)) { break }
    $writtenKinds.Add($kind)
}
Require (-not $writtenKinds.Contains(255)) "readback mismatch allowed completion"

$unsafeBlocks = [regex]::Matches($capture, 'unsafe\s*\{').Count
Require ($unsafeBlocks -eq 2) "unexpected Surface capture unsafe inventory ($unsafeBlocks)"
$newUnsafeText = $capture + $entry + $series + $append
$newUnsafeSites = [regex]::Matches($newUnsafeText, 'unsafe\s+(?:fn|\{)').Count
$reasonedSites = [regex]::Matches($newUnsafeText, 'SAFETY: Reason:.*Invariant:').Count
Require ($reasonedSites -ge $newUnsafeSites) "new unsafe lacks explicit Reason/Invariant"

$pciPredicate = Join-Path $RepoRoot "scripts\test-pci-bar-sizing.ps1"
if ($SkipBuild) {
    & $pciPredicate -MetadataOnly
} else {
    & $pciPredicate
}
$pciPredicateSucceeded = $?
if (-not $pciPredicateSucceeded) { throw "accepted PCI capture predicate failed" }

if (-not $SkipBuild) {
    $env:CARGO_HOME = "C:\Users\admin\.cargo"
    $env:CARGO_TARGET_DIR = Join-Path $RepoRoot "target\surface-k3-cargo"
    $linker = (Resolve-Path (Join-Path $RepoRoot "seed-kernel\linker.ld")).Path
    $env:RUSTFLAGS = "-C link-arg=-T$linker -C relocation-model=static -C code-model=kernel"
    $cargoArgs = @('+nightly-2024-10-15', '-Zbuild-std=core,compiler_builtins,alloc', 'build', '--release', '--locked', '--target', (Join-Path $RepoRoot 'seed-kernel\x86_64-seed.json'), '-p', 'seed-kernel')
    & cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) { throw "freestanding release seed-kernel build failed" }
}

python -B (Join-Path $RepoRoot "scripts\unsafe-inventory.py") --check (Join-Path $RepoRoot "docs\architecture\unsafe-inventory-baseline-v2.json")
if ($LASTEXITCODE -ne 0) { throw "unsafe inventory baseline check failed" }

Push-Location $RepoRoot
try {
    $expectedPaths = @(
        "docs/architecture/unsafe-inventory-baseline-v2.json",
        "scripts/test-surface-fact-capture-kernel.ps1",
        "seed-kernel/src/main.rs",
        "seed-kernel/src/surface_fact_capture.rs",
        "seed-kernel/src/usb.rs"
    ) | Sort-Object
    $statusLines = @(git status --short --untracked-files=all)
    if ($LASTEXITCODE -ne 0) { throw "git status failed" }
    $actualPaths = @($statusLines | ForEach-Object { $_.Substring(3).Replace('\', '/') }) | Sort-Object
    $pathDifference = @(Compare-Object $expectedPaths $actualPaths)
    Require ($pathDifference.Count -eq 0) "dirty file set differs from the exact five-file lane"

    git diff HEAD --check -- docs/architecture/unsafe-inventory-baseline-v2.json seed-kernel/src/main.rs seed-kernel/src/usb.rs
    if ($LASTEXITCODE -ne 0) { throw "tracked diff whitespace check failed" }
    foreach ($untracked in "scripts/test-surface-fact-capture-kernel.ps1", "seed-kernel/src/surface_fact_capture.rs") {
        $noIndexOutput = @(git -c core.autocrlf=false diff --no-index --check -- NUL $untracked 2>&1)
        $noIndexExit = $LASTEXITCODE
        Require (($noIndexExit -in @(0, 1)) -and $noIndexOutput.Count -eq 0) "untracked diff whitespace check failed for $untracked"
    }
} finally { Pop-Location }

Write-Output "surface fact capture kernel predicates: PASS"
