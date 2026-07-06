if (-not $PersistDiskImage) {
    throw "Persistence profile requires PersistDiskImage"
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
    -Name "gpt-header-valid" `
    -Expected 'signature "EFI PART"' `
    -Passed $gptHeaderValid `
    -Actual $(if ($gptHeaderValid) { "matched" } else { ($inspection.gpt | ConvertTo-Json -Compress -Depth 6) })

Add-Predicate `
    -Name "gpt-crc-checked" `
    -Expected "primary and backup GPT header plus entry-array CRC32 match" `
    -Passed $gptCrcChecked `
    -Actual $(if ($gptCrcChecked) { "matched" } else { ($inspection.gpt | ConvertTo-Json -Compress -Depth 6) })

Add-Predicate `
    -Name "gpt-seed-data-found" `
    -Expected "SEED_DATA partition with raiOS DATA type GUID" `
    -Passed $seedDataFound `
    -Actual $(if ($seedDataFound) { "matched" } else { ($inspection.partitions | ConvertTo-Json -Compress -Depth 4) })

Add-Predicate `
    -Name "data-superblock-valid" `
    -Expected "RAIOS_DATA_SB_V0 magic plus matching LBA0/LBA1 header sha256" `
    -Passed $superblockValid `
    -Actual $(if ($superblockValid) { "matched" } else { ($inspection.superblock | ConvertTo-Json -Compress -Depth 6) })

if (-not ($gptHeaderValid -and $gptCrcChecked -and $seedDataFound -and $superblockValid)) {
    throw "Persistence host-side disk validation failed"
}
