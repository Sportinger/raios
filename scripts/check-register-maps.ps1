[CmdletBinding()]
param(
    [switch]$SelfTest,
    [string]$SummaryPath
)

$ErrorActionPreference = 'Stop'
$expectedSchema = 'urn:raios:hardware:schema:register-map:v1'
$allowedAccess = @('ro', 'rw', 'wo', 'w1c', 'rwc', 'mixed')
$allowedWidths = @(8, 16, 32, 64)
$allowedStatuses = @('documented', 'source-derived', 'unknown')
$allowedProvenanceKinds = @('raios-source', 'linux-source', 'public-specification')
$allowedConstantKinds = @('command-opcode', 'shift', 'descriptor-mask', 'ring-mask', 'ring-parameter', 'buffer-parameter', 'region-offset', 'region-stride')
$allowedRootProperties = @('schema', 'mapVersion', 'id', 'device', 'provenance', 'constants', 'registers')
$root = Split-Path -Parent $PSScriptRoot
$mapDirectory = Join-Path $root 'hardware/register-maps'
$schemaPath = Join-Path $root 'hardware/schemas/register-map.v1.json'

function Has-Property([object]$Object, [string]$Name) {
    return $null -ne $Object -and $null -ne $Object.PSObject.Properties[$Name]
}

function Clone-Json([object]$Value) {
    return ($Value | ConvertTo-Json -Depth 30 -Compress | ConvertFrom-Json)
}

function Validate-RegisterMap([object]$Map, [string]$Label) {
    $errors = [System.Collections.Generic.List[string]]::new()
    $add = { param([string]$Message) $errors.Add("${Label}: $Message") }

    if ($null -eq $Map) { & $add 'document is null'; return @($errors) }
    foreach ($property in $Map.PSObject.Properties.Name) { if ($allowedRootProperties -notcontains $property) { & $add "unknown root property '$property'" } }
    if (-not (Has-Property $Map 'schema') -or $Map.schema -ne $expectedSchema) { & $add 'unknown or missing schema' }
    if (-not (Has-Property $Map 'mapVersion') -or [int]$Map.mapVersion -ne 1) { & $add 'mapVersion must be 1' }
    if (-not (Has-Property $Map 'id') -or [string]::IsNullOrWhiteSpace([string]$Map.id)) { & $add 'id is required' }

    if (-not (Has-Property $Map 'device') -or $null -eq $Map.device) {
        & $add 'device binding is required'
    } else {
        if ($Map.device.bus -ne 'pci') { & $add 'device.bus must be pci' }
        if (-not (Has-Property $Map.device 'identity') -or $null -eq $Map.device.identity -or $Map.device.identity.PSObject.Properties.Count -eq 0) { & $add 'device.identity must bind the device' }
        if (-not (Has-Property $Map.device 'bar') -or $null -eq $Map.device.bar) {
            & $add 'device.bar binding is required'
        } else {
            if (-not (Has-Property $Map.device.bar 'index') -or [int]$Map.device.bar.index -lt 0 -or [int]$Map.device.bar.index -gt 5) { & $add 'BAR index must be 0..5' }
            if ($Map.device.bar.addressSpace -ne 'mmio') { & $add 'BAR addressSpace must be mmio' }
        }
    }

    $provenanceIds = @{}
    if (-not (Has-Property $Map 'provenance') -or @($Map.provenance).Count -eq 0) {
        & $add 'top-level provenance is required'
    } else {
        foreach ($source in @($Map.provenance)) {
            if ([string]::IsNullOrWhiteSpace([string]$source.id) -or [string]::IsNullOrWhiteSpace([string]$source.kind) -or [string]::IsNullOrWhiteSpace([string]$source.path) -or [string]::IsNullOrWhiteSpace([string]$source.claim)) {
                & $add 'each provenance entry requires id, kind, path, and claim'
            } elseif ($allowedProvenanceKinds -notcontains [string]$source.kind) {
                & $add "provenance '$($source.id)' has unknown kind '$($source.kind)'"
            } elseif ($provenanceIds.ContainsKey([string]$source.id)) {
                & $add "duplicate provenance id '$($source.id)'"
            } else { $provenanceIds[[string]$source.id] = $true }
        }
    }

    $regions = @{}
    if (-not (Has-Property $Map.device 'ownedRegions') -or @($Map.device.ownedRegions).Count -eq 0) {
        & $add 'at least one owned region is required'
    } else {
        foreach ($region in @($Map.device.ownedRegions)) {
            $regionName = [string]$region.name
            if ([string]::IsNullOrWhiteSpace($regionName) -or $regions.ContainsKey($regionName)) { & $add "region name is missing or duplicate '$regionName'"; continue }
            if (-not (Has-Property $region 'base') -or [string]::IsNullOrWhiteSpace([string]$region.base.kind)) { & $add "region '$regionName' has no base binding" }
            if (-not (Has-Property $region 'length') -or [long]$region.length -le 0) { & $add "region '$regionName' length must be positive" }
            $regions[$regionName] = $region
        }
    }

    $constantNames = @{}
    if (Has-Property $Map 'constants') {
        foreach ($constant in @($Map.constants)) {
            $constantName = [string]$constant.name
            if ([string]::IsNullOrWhiteSpace($constantName) -or $constantNames.ContainsKey($constantName)) { & $add "map constant name is missing or duplicate '$constantName'" } else { $constantNames[$constantName] = $true }
            if (-not (Has-Property $constant 'value') -or [long]$constant.value -lt 0 -or $allowedConstantKinds -notcontains [string]$constant.kind -or [string]::IsNullOrWhiteSpace([string]$constant.meaning)) { & $add "map constant '$constantName' requires non-negative value, allowed kind, and meaning" }
            if (-not (Has-Property $constant 'provenance') -or @($constant.provenance).Count -eq 0) { & $add "map constant '$constantName' requires provenance" }
            else { foreach ($sourceId in @($constant.provenance)) { if (-not $provenanceIds.ContainsKey([string]$sourceId)) { & $add "map constant '$constantName' references unknown provenance '$sourceId'" } } }
        }
    }

    $names = @{}
    $tuples = @{}
    if (-not (Has-Property $Map 'registers') -or @($Map.registers).Count -eq 0) {
        & $add 'at least one register is required'
        return @($errors)
    }
    foreach ($register in @($Map.registers)) {
        $name = [string]$register.name
        if ([string]::IsNullOrWhiteSpace($name)) { & $add 'register name is required'; $name = '<missing>' }
        if ($names.ContainsKey($name)) { & $add "duplicate register name '$name'" } else { $names[$name] = $true }

        $regionName = [string]$register.region
        if (-not $regions.ContainsKey($regionName)) { & $add "register '$name' references unknown region '$regionName'"; continue }
        if (-not (Has-Property $register 'offset') -or [long]$register.offset -lt 0) { & $add "register '$name' offset must be non-negative"; continue }
        if (-not (Has-Property $register 'width') -or $allowedWidths -notcontains [int]$register.width) { & $add "register '$name' has invalid width"; continue }
        if (-not (Has-Property $register 'access') -or $allowedAccess -notcontains [string]$register.access) { & $add "register '$name' has invalid access '$($register.access)'" }

        $bytes = [int]$register.width / 8
        $offset = [long]$register.offset
        if (($offset % $bytes) -ne 0) { & $add "register '$name' is not naturally aligned" }
        $lastEnd = $offset + $bytes
        if (Has-Property $register 'instances') {
            if (Has-Property $register.instances 'count') {
                $count = [long]$register.instances.count
                $stride = [long]$register.instances.stride
                if ($count -le 0 -or $stride -lt $bytes) { & $add "register '$name' has invalid instance count/stride" }
                else { $lastEnd = $offset + (($count - 1) * $stride) + $bytes }
            } elseif (-not (Has-Property $register.instances 'countSource') -or -not (Has-Property $register.instances 'stride')) {
                & $add "register '$name' instances require count+stride or countSource+stride"
            }
        }
        if ($lastEnd -gt [long]$regions[$regionName].length) { & $add "register '$name' exceeds owned region '$regionName'" }

        $tuple = "$regionName/$offset/$($register.width)"
        if ($tuples.ContainsKey($tuple)) {
            $prior = $tuples[$tuple]
            $alias = if (Has-Property $register 'aliasGroup') { [string]$register.aliasGroup } else { '' }
            $priorAlias = if (Has-Property $prior 'aliasGroup') { [string]$prior.aliasGroup } else { '' }
            if ([string]::IsNullOrWhiteSpace($alias) -or $alias -ne $priorAlias) { & $add "duplicate offset tuple '$tuple' lacks a shared aliasGroup" }
        } else { $tuples[$tuple] = $register }

        if (-not (Has-Property $register 'semantics') -or $null -eq $register.semantics -or $allowedStatuses -notcontains [string]$register.semantics.status -or [string]::IsNullOrWhiteSpace([string]$register.semantics.description)) { & $add "register '$name' requires allowed semantics status and description" }
        if (-not (Has-Property $register 'provenance') -or @($register.provenance).Count -eq 0) {
            & $add "register '$name' requires provenance"
        } else {
            foreach ($sourceId in @($register.provenance)) { if (-not $provenanceIds.ContainsKey([string]$sourceId)) { & $add "register '$name' references unknown provenance '$sourceId'" } }
        }

        $fields = @(if ($null -ne $register.semantics -and (Has-Property $register.semantics 'bitfields')) { $register.semantics.bitfields })
        for ($i = 0; $i -lt $fields.Count; $i++) {
            $field = $fields[$i]
            if ([string]::IsNullOrWhiteSpace([string]$field.name) -or [string]::IsNullOrWhiteSpace([string]$field.meaning)) { & $add "register '$name' has an unnamed/undescribed bitfield" }
            try { $mask = [uint64]$field.mask } catch { & $add "register '$name' has a non-integer bit mask"; continue }
            if ($mask -eq 0) { & $add "register '$name' bitfield '$($field.name)' has zero mask" }
            if ([int]$register.width -lt 64 -and $mask -ge ([uint64]1 -shl [int]$register.width)) { & $add "register '$name' bitfield '$($field.name)' exceeds width" }
            for ($j = 0; $j -lt $i; $j++) {
                $other = $fields[$j]
                $otherMask = [uint64]$other.mask
                if (($mask -band $otherMask) -ne 0) {
                    $group = if (Has-Property $field 'overlapGroup') { [string]$field.overlapGroup } else { '' }
                    $otherGroup = if (Has-Property $other 'overlapGroup') { [string]$other.overlapGroup } else { '' }
                    if ([string]::IsNullOrWhiteSpace($group) -or $group -ne $otherGroup) { & $add "register '$name' bitfields '$($field.name)' and '$($other.name)' overlap without a shared overlapGroup" }
                }
            }
        }
    }
    return @($errors)
}

if (-not (Test-Path -LiteralPath $schemaPath)) { throw "Missing schema: $schemaPath" }
$null = Get-Content -Raw -LiteralPath $schemaPath | ConvertFrom-Json
$mapFiles = @(Get-ChildItem -LiteralPath $mapDirectory -Filter '*.v1.json' -File | Sort-Object Name)
$allErrors = [System.Collections.Generic.List[string]]::new()
$maps = @()
$registerCount = 0
foreach ($file in $mapFiles) {
    try { $map = Get-Content -Raw -LiteralPath $file.FullName | ConvertFrom-Json } catch { $allErrors.Add("$($file.Name): invalid JSON: $($_.Exception.Message)"); continue }
    $maps += $map
    $registerCount += @($map.registers).Count
    foreach ($problem in @(Validate-RegisterMap $map $file.Name)) { $allErrors.Add([string]$problem) }
}

$negativeResults = @()
if ($SelfTest -and $maps.Count -gt 0 -and $allErrors.Count -eq 0) {
    $cases = @(
        @{ name = 'duplicate-name'; mutate = { param($m) $m.registers[1].name = $m.registers[0].name } },
        @{ name = 'duplicate-offset'; mutate = { param($m) $m.registers[1].region = $m.registers[0].region; $m.registers[1].offset = $m.registers[0].offset } },
        @{ name = 'out-of-range'; mutate = { param($m) $m.registers[0].offset = [long]$m.device.ownedRegions[0].length } },
        @{ name = 'bad-width'; mutate = { param($m) $m.registers[0].width = 24 } },
        @{ name = 'bad-access'; mutate = { param($m) $m.registers[0].access = 'sometimes' } },
        @{ name = 'overlapping-mask'; mutate = { param($m) $m.registers[0].semantics | Add-Member -NotePropertyName bitfields -NotePropertyValue @(@{ name='A'; mask=3; meaning='mutation' }, @{ name='B'; mask=1; meaning='mutation' }) -Force } },
        @{ name = 'out-of-width-mask'; mutate = { param($m) $m.registers[0].semantics | Add-Member -NotePropertyName bitfields -NotePropertyValue $null -Force; $m.registers[0].semantics.bitfields = [pscustomobject]@{ name='OOB'; mask=4294967296; meaning='mutation' } } },
        @{ name = 'missing-register-provenance'; mutate = { param($m) $m.registers[0].provenance = @() } },
        @{ name = 'missing-device-binding'; mutate = { param($m) $m.device.identity = $null } },
        @{ name = 'unknown-schema'; mutate = { param($m) $m.schema = 'urn:raios:hardware:schema:register-map:v2' } }
    )
    foreach ($case in $cases) {
        $mutant = Clone-Json $maps[0]
        & $case.mutate $mutant
        $rejections = @(Validate-RegisterMap $mutant "selftest/$($case.name)")
        $rejected = $rejections.Count -gt 0
        $negativeResults += [ordered]@{ name = $case.name; rejected = $rejected; errorCount = $rejections.Count }
        if (-not $rejected) { $allErrors.Add("selftest/$($case.name): unsafe mutation was accepted") }
    }
}

$summary = [ordered]@{
    predicate = 'register-maps-v1'
    ok = ($allErrors.Count -eq 0)
    schema = $expectedSchema
    mapsChecked = $mapFiles.Count
    registersChecked = $registerCount
    selfTest = [bool]$SelfTest
    negativeCases = $negativeResults.Count
    negativeCasesRejected = @($negativeResults | Where-Object { $_.rejected }).Count
    negativeResults = $negativeResults
    errors = @($allErrors)
}
$json = $summary | ConvertTo-Json -Depth 10 -Compress
if (-not [string]::IsNullOrWhiteSpace($SummaryPath)) { [System.IO.File]::WriteAllText((Join-Path (Get-Location) $SummaryPath), $json + [Environment]::NewLine) }
Write-Output $json
if ($allErrors.Count -ne 0) { exit 1 }
