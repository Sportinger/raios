param(
    [string[]]$ManifestPath = @(),
    [switch]$SelfTest
)

$ErrorActionPreference = "Stop"
$HardwareManifestRepoRoot = Split-Path -Parent $PSScriptRoot
$HardwareManifestSchemaPath = Join-Path $HardwareManifestRepoRoot "hardware\schemas\machine-manifest.v1.json"
$HardwareManifestDefaultPaths = @(
    (Join-Path $HardwareManifestRepoRoot "hardware\manifests\qemu-q35-shadow.v1.json"),
    (Join-Path $HardwareManifestRepoRoot "hardware\manifests\surface-pro-4.v1.json")
)

function Copy-JsonObject {
    param([object]$Value)
    return ($Value | ConvertTo-Json -Depth 100 -Compress | ConvertFrom-Json)
}

function Get-OrdinalJsonProperty {
    param([AllowNull()][object]$Object, [string]$Name)
    if ($null -eq $Object) { return $null }
    foreach ($property in $Object.PSObject.Properties) {
        if ([StringComparer]::Ordinal.Equals([string]$property.Name, $Name)) { return $property }
    }
    return $null
}

function Test-HasProperty {
    param([object]$Object, [string]$Name)
    return $null -ne (Get-OrdinalJsonProperty $Object $Name)
}

function Test-OrdinalStringIn {
    param([AllowNull()][string]$Value, [object[]]$Candidates)
    foreach ($candidate in @($Candidates)) {
        if ([StringComparer]::Ordinal.Equals($Value, [string]$candidate)) { return $true }
    }
    return $false
}

function Get-JsonType {
    param([AllowNull()][object]$Value)
    if ($null -eq $Value) { return "null" }
    if ($Value -is [bool]) { return "boolean" }
    if ($Value -is [string]) { return "string" }
    if ($Value -is [System.Array]) { return "array" }
    if ($Value -is [pscustomobject] -or $Value -is [System.Collections.IDictionary]) { return "object" }
    if ($Value -is [sbyte] -or $Value -is [byte] -or $Value -is [int16] -or $Value -is [uint16] -or
        $Value -is [int32] -or $Value -is [uint32] -or $Value -is [int64] -or $Value -is [uint64]) { return "integer" }
    if ($Value -is [decimal] -or $Value -is [double] -or $Value -is [single]) {
        if (-not [double]::IsNaN([double]$Value) -and -not [double]::IsInfinity([double]$Value) -and
            [decimal]$Value -eq [decimal]::Truncate([decimal]$Value)) { return "integer" }
        return "number"
    }
    return "invalid"
}

function Get-SchemaReference {
    param([object]$SchemaRoot, [string]$Reference)
    if ($Reference -notmatch '^#/') { throw "Unsupported non-local schema reference '$Reference'" }
    $node = $SchemaRoot
    foreach ($part in $Reference.Substring(2).Split('/')) {
        $name = $part.Replace('~1', '/').Replace('~0', '~')
        $property = Get-OrdinalJsonProperty $node $name
        if ($null -eq $property) { throw "Unresolved schema reference '$Reference'" }
        $node = $property.Value
    }
    return $node
}

function Assert-SupportedSchema {
    param([object]$Rule, [object]$SchemaRoot, [string]$Path = "#")
    $allowed = @('$schema', '$id', '$defs', '$ref', 'title', 'description', 'type', 'additionalProperties',
        'required', 'properties', 'items', 'const', 'enum', 'pattern', 'minLength', 'maxLength', 'minimum',
        'minItems', 'maxItems', 'maxProperties', 'uniqueItems', 'format')
    foreach ($property in $Rule.PSObject.Properties) {
        if (-not (Test-OrdinalStringIn $property.Name $allowed)) { throw "Unsupported schema keyword '$($property.Name)' at $Path" }
    }
    if (Test-HasProperty $Rule '$ref') { $null = Get-SchemaReference -SchemaRoot $SchemaRoot -Reference $Rule.'$ref' }
    if (Test-HasProperty $Rule 'properties') {
        foreach ($property in $Rule.properties.PSObject.Properties) {
            Assert-SupportedSchema -Rule $property.Value -SchemaRoot $SchemaRoot -Path "$Path/properties/$($property.Name)"
        }
    }
    if (Test-HasProperty $Rule '$defs') {
        foreach ($property in $Rule.'$defs'.PSObject.Properties) {
            Assert-SupportedSchema -Rule $property.Value -SchemaRoot $SchemaRoot -Path "$Path/`$defs/$($property.Name)"
        }
    }
    if (Test-HasProperty $Rule 'items') { Assert-SupportedSchema -Rule $Rule.items -SchemaRoot $SchemaRoot -Path "$Path/items" }
}

function Add-ManifestError {
    param([System.Collections.Generic.List[object]]$Errors, [string]$Code, [string]$Path, [string]$Message)
    $Errors.Add([ordered]@{ code = $Code; path = $Path; message = $Message })
}

function Test-JsonEqual {
    param([AllowNull()][object]$Left, [AllowNull()][object]$Right)
    if ((Get-JsonType $Left) -ne (Get-JsonType $Right)) { return $false }
    return ($Left | ConvertTo-Json -Depth 100 -Compress) -ceq ($Right | ConvertTo-Json -Depth 100 -Compress)
}

function Get-FirstJsonDifferencePath {
    param([AllowNull()][object]$Actual, [AllowNull()][object]$Expected, [string]$Path = "")
    $actualType = Get-JsonType $Actual; $expectedType = Get-JsonType $Expected
    if ($actualType -cne $expectedType) { return $(if ($Path) { $Path } else { "/" }) }
    if ($actualType -eq "array") {
        if (@($Actual).Count -ne @($Expected).Count) { return $(if ($Path) { $Path } else { "/" }) }
        for ($i = 0; $i -lt @($Expected).Count; $i++) {
            $difference = Get-FirstJsonDifferencePath @($Actual)[$i] @($Expected)[$i] "$Path/$i"
            if ($null -ne $difference) { return $difference }
        }
        return $null
    }
    if ($actualType -eq "object") {
        $actualProperties = @($Actual.PSObject.Properties); $expectedProperties = @($Expected.PSObject.Properties)
        if ($actualProperties.Count -ne $expectedProperties.Count) { return $(if ($Path) { $Path } else { "/" }) }
        foreach ($expectedProperty in $expectedProperties) {
            $actualProperty = Get-OrdinalJsonProperty $Actual $expectedProperty.Name
            if ($null -eq $actualProperty) { return "$Path/$($expectedProperty.Name)" }
            $difference = Get-FirstJsonDifferencePath $actualProperty.Value $expectedProperty.Value "$Path/$($expectedProperty.Name)"
            if ($null -ne $difference) { return $difference }
        }
        return $null
    }
    if (-not (Test-JsonEqual $Actual $Expected)) { return $(if ($Path) { $Path } else { "/" }) }
    return $null
}

function Get-QemuQ35ShadowControlledContract {
    # This independent value/status/provenance allowlist is launch truth. The
    # digest supplied by a lane order binds bytes, but cannot change this truth.
    return @'
{
  "schema_version": "raios.machine_manifest.v1",
  "curated_context_ready": true,
  "missing_required_facts": [],
  "machine": { "id": "qemu-q35-shadow", "kind": "virtual", "display_name": "QEMU q35 shadow VM", "provenance": { "machine_id": "qemu-q35-shadow", "source_kind": "launch_configuration", "source_ref": "scripts/run-stage0-qemu.ps1:-machine q35", "observed_at_utc": "2026-07-19T00:00:00Z" } },
  "cpu": {
    "architecture": { "status": "observed", "value": "x86_64", "provenance": { "machine_id": "qemu-q35-shadow", "source_kind": "launch_configuration", "source_ref": "scripts/run-stage0-qemu.ps1:qemu-system-x86_64", "observed_at_utc": "2026-07-19T00:00:00Z" } },
    "model": { "status": "observed", "value": "max", "provenance": { "machine_id": "qemu-q35-shadow", "source_kind": "launch_configuration", "source_ref": "vm-harness/shadow-vm-smoke.ps1:Cpu=max", "observed_at_utc": "2026-07-19T00:00:00Z" } },
    "features": [
      { "name": "qemu-max-feature-set", "status": "observed", "value": true, "provenance": { "machine_id": "qemu-q35-shadow", "source_kind": "launch_configuration", "source_ref": "vm-harness/shadow-vm-smoke.ps1:Cpu=max -> scripts/run-stage0-qemu.ps1:-cpu max", "observed_at_utc": "2026-07-19T00:00:00Z" } }
    ]
  },
  "memory": {
    "total_bytes": { "status": "observed", "value": 536870912, "provenance": { "machine_id": "qemu-q35-shadow", "source_kind": "launch_configuration", "source_ref": "vm-harness/shadow-vm-smoke.ps1:GuestMemoryMB=512", "observed_at_utc": "2026-07-19T00:00:00Z" } },
    "topology": { "status": "observed", "value": "single guest address space; no explicit NUMA node, DIMM, or memory-backend configuration", "provenance": { "machine_id": "qemu-q35-shadow", "source_kind": "launch_configuration", "source_ref": "scripts/run-stage0-qemu.ps1:qemu args contain -m only; no -numa, -object memory-backend, or -device pc-dimm", "observed_at_utc": "2026-07-19T00:00:00Z" } },
    "regions": [
      { "id": "guest-ram-capacity", "kind": "ram", "start_bytes": 0, "length_bytes": 536870912, "status": "observed", "provenance": { "machine_id": "qemu-q35-shadow", "source_kind": "launch_configuration", "source_ref": "vm-harness/shadow-vm-smoke.ps1:GuestMemoryMB=512 -> scripts/run-stage0-qemu.ps1:-m ${MemoryMB}M", "observed_at_utc": "2026-07-19T00:00:00Z" } }
    ]
  },
  "devices": [
    { "id": "q35", "type": "chipset", "identity": { "status": "observed", "value": "q35", "provenance": { "machine_id": "qemu-q35-shadow", "source_kind": "launch_configuration", "source_ref": "scripts/run-stage0-qemu.ps1:-machine q35", "observed_at_utc": "2026-07-19T00:00:00Z" } }, "bdf": { "status": "unknown", "value": null, "provenance": { "machine_id": "qemu-q35-shadow", "source_kind": "documentation_gap", "source_ref": "chipset BDF not pinned", "observed_at_utc": "2026-07-19T00:00:00Z" } }, "resources": [], "provenance": { "machine_id": "qemu-q35-shadow", "source_kind": "launch_configuration", "source_ref": "scripts/run-stage0-qemu.ps1", "observed_at_utc": "2026-07-19T00:00:00Z" } },
    { "id": "xhci", "type": "usb_controller", "identity": { "status": "observed", "value": "qemu-xhci", "provenance": { "machine_id": "qemu-q35-shadow", "source_kind": "launch_configuration", "source_ref": "scripts/run-stage0-qemu.ps1:-device qemu-xhci,id=xhci", "observed_at_utc": "2026-07-19T00:00:00Z" } }, "bdf": { "status": "unknown", "value": null, "provenance": { "machine_id": "qemu-q35-shadow", "source_kind": "documentation_gap", "source_ref": "BDF is assigned by QEMU and not pinned", "observed_at_utc": "2026-07-19T00:00:00Z" } }, "resources": [], "provenance": { "machine_id": "qemu-q35-shadow", "source_kind": "launch_configuration", "source_ref": "scripts/run-stage0-qemu.ps1", "observed_at_utc": "2026-07-19T00:00:00Z" } },
    { "id": "bootkbd", "type": "usb_input", "identity": { "status": "observed", "value": "usb-kbd", "provenance": { "machine_id": "qemu-q35-shadow", "source_kind": "launch_configuration", "source_ref": "scripts/run-stage0-qemu.ps1:-device usb-kbd", "observed_at_utc": "2026-07-19T00:00:00Z" } }, "bdf": { "status": "unknown", "value": null, "provenance": { "machine_id": "qemu-q35-shadow", "source_kind": "documentation_gap", "source_ref": "USB devices have no PCI BDF", "observed_at_utc": "2026-07-19T00:00:00Z" } }, "resources": [], "provenance": { "machine_id": "qemu-q35-shadow", "source_kind": "launch_configuration", "source_ref": "scripts/run-stage0-qemu.ps1", "observed_at_utc": "2026-07-19T00:00:00Z" } },
    { "id": "pointer", "type": "usb_input", "identity": { "status": "observed", "value": "usb-tablet", "provenance": { "machine_id": "qemu-q35-shadow", "source_kind": "launch_configuration", "source_ref": "scripts/run-stage0-qemu.ps1:-device usb-tablet", "observed_at_utc": "2026-07-19T00:00:00Z" } }, "bdf": { "status": "unknown", "value": null, "provenance": { "machine_id": "qemu-q35-shadow", "source_kind": "documentation_gap", "source_ref": "USB devices have no PCI BDF", "observed_at_utc": "2026-07-19T00:00:00Z" } }, "resources": [], "provenance": { "machine_id": "qemu-q35-shadow", "source_kind": "launch_configuration", "source_ref": "scripts/run-stage0-qemu.ps1", "observed_at_utc": "2026-07-19T00:00:00Z" } }
  ]
}
'@ | ConvertFrom-Json
}

function Test-AgainstSchema {
    param([AllowNull()][object]$Value, [object]$Rule, [object]$SchemaRoot,
        [System.Collections.Generic.List[object]]$Errors, [string]$Path = "")

    if (Test-HasProperty $Rule '$ref') {
        $reference = (Get-OrdinalJsonProperty $Rule '$ref').Value
        Test-AgainstSchema -Value $Value -Rule (Get-SchemaReference $SchemaRoot $reference) -SchemaRoot $SchemaRoot -Errors $Errors -Path $Path
        return
    }
    if (Test-HasProperty $Rule 'const') {
        if (-not (Test-JsonEqual $Value $Rule.const)) {
            $code = if ($Path -eq "/schema_version") { "schema_version_mismatch" } else { "const_mismatch" }
            Add-ManifestError $Errors $code $Path "Value does not match the schema constant"
        }
    }
    if (Test-HasProperty $Rule 'enum') {
        $matched = $false
        foreach ($candidate in @($Rule.enum)) { if (Test-JsonEqual $Value $candidate) { $matched = $true; break } }
        if (-not $matched) { Add-ManifestError $Errors "enum_mismatch" $Path "Value is not in the allowed vocabulary" }
    }
    if (Test-HasProperty $Rule 'type') {
        $actualType = Get-JsonType $Value
        $allowedTypes = @($Rule.type)
        if (-not (Test-OrdinalStringIn $actualType $allowedTypes)) {
            Add-ManifestError $Errors "type_mismatch" $Path "Expected JSON type $($allowedTypes -join '|'), got $actualType"
            return
        }
    }
    $jsonType = Get-JsonType $Value
    if ($jsonType -eq "string") {
        if ((Test-HasProperty $Rule 'minLength') -and $Value.Length -lt [int]$Rule.minLength) {
            Add-ManifestError $Errors "string_too_short" $Path "String is shorter than minLength"
        }
        if ((Test-HasProperty $Rule 'maxLength') -and $Value.Length -gt [int]$Rule.maxLength) {
            Add-ManifestError $Errors "string_too_long" $Path "String is longer than maxLength"
        }
        if ((Test-HasProperty $Rule 'pattern') -and $Value -cnotmatch $Rule.pattern) {
            Add-ManifestError $Errors "pattern_mismatch" $Path "String does not match the required pattern"
        }
        if ((Test-HasProperty $Rule 'format') -and $Rule.format -eq "date-time") {
            $parsed = [DateTimeOffset]::MinValue
            if ($Value -notmatch '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})$' -or
                -not [DateTimeOffset]::TryParse($Value, [Globalization.CultureInfo]::InvariantCulture,
                    [Globalization.DateTimeStyles]::RoundtripKind, [ref]$parsed)) {
                Add-ManifestError $Errors "format_mismatch" $Path "String is not an RFC 3339 date-time"
            }
        }
    }
    if ($jsonType -in @("integer", "number") -and (Test-HasProperty $Rule 'minimum') -and
        [decimal]$Value -lt [decimal]$Rule.minimum) {
        Add-ManifestError $Errors "below_minimum" $Path "Number is below the schema minimum"
    }
    if ($jsonType -eq "array") {
        if ((Test-HasProperty $Rule 'minItems') -and @($Value).Count -lt [int]$Rule.minItems) {
            $code = if ($Path -eq "/devices") { "missing_devices" } else { "array_too_short" }
            Add-ManifestError $Errors $code $Path "Array is shorter than minItems"
        }
        if ((Test-HasProperty $Rule 'maxItems') -and @($Value).Count -gt [int]$Rule.maxItems) {
            Add-ManifestError $Errors "array_too_long" $Path "Array is longer than maxItems"
        }
        if ((Test-HasProperty $Rule 'uniqueItems') -and $Rule.uniqueItems) {
            $seen = New-Object 'System.Collections.Generic.HashSet[string]' ([StringComparer]::Ordinal)
            for ($i = 0; $i -lt @($Value).Count; $i++) {
                $key = @($Value)[$i] | ConvertTo-Json -Depth 100 -Compress
                if (-not $seen.Add($key)) { Add-ManifestError $Errors "duplicate_array_item" "$Path/$i" "Array items must be unique" }
            }
        }
        if (Test-HasProperty $Rule 'items') {
            for ($i = 0; $i -lt @($Value).Count; $i++) {
                Test-AgainstSchema -Value @($Value)[$i] -Rule $Rule.items -SchemaRoot $SchemaRoot -Errors $Errors -Path "$Path/$i"
            }
        }
    }
    if ($jsonType -eq "object") {
        if ((Test-HasProperty $Rule 'maxProperties') -and @($Value.PSObject.Properties).Count -gt [int]$Rule.maxProperties) {
            Add-ManifestError $Errors "object_too_large" $Path "Object has more properties than maxProperties"
        }
        if (Test-HasProperty $Rule 'required') {
            foreach ($name in @($Rule.required)) {
                if (-not (Test-HasProperty $Value $name)) {
                    $code = if ($Path -ceq "" -and (Test-OrdinalStringIn $name @("machine", "cpu", "memory", "devices"))) { "missing_$name" } else { "required_property_missing" }
                    Add-ManifestError $Errors $code "$Path/$name" "Required property '$name' is missing"
                }
            }
        }
        if ((Test-HasProperty $Rule 'additionalProperties') -and $Rule.additionalProperties -eq $false) {
            $allowedNames = if (Test-HasProperty $Rule 'properties') { @($Rule.properties.PSObject.Properties.Name) } else { @() }
            foreach ($property in $Value.PSObject.Properties) {
                if (-not (Test-OrdinalStringIn $property.Name $allowedNames)) { Add-ManifestError $Errors "additional_property" "$Path/$($property.Name)" "Property is not allowed" }
            }
        }
        if (Test-HasProperty $Rule 'properties') {
            foreach ($property in $Rule.properties.PSObject.Properties) {
                $valueProperty = Get-OrdinalJsonProperty $Value $property.Name
                if ($null -ne $valueProperty) {
                    Test-AgainstSchema -Value $valueProperty.Value -Rule $property.Value -SchemaRoot $SchemaRoot -Errors $Errors -Path "$Path/$($property.Name)"
                }
            }
        }
    }
}

function Find-NamedManifestItem {
    param([object[]]$Items, [string]$Name)
    foreach ($item in @($Items)) { if ([string]$item.id -ceq $Name -or [string]$item.name -ceq $Name) { return $item } }
    return $null
}

function Resolve-MachineManifestFact {
    param([object]$Manifest, [string]$Path)
    $node = $null; $value = $null
    if ($Path -cmatch '^/cpu/(architecture|model)$') { $node = (Get-OrdinalJsonProperty $Manifest.cpu $Matches[1]).Value; $value = $node.value }
    elseif ($Path -cmatch '^/cpu/features/([a-z0-9][a-z0-9._-]*)$') { $node = Find-NamedManifestItem @($Manifest.cpu.features) $Matches[1]; if ($null -ne $node) { $value = $node.value } }
    elseif ($Path -cmatch '^/memory/(total_bytes|topology)$') { $node = (Get-OrdinalJsonProperty $Manifest.memory $Matches[1]).Value; $value = $node.value }
    elseif ($Path -cmatch '^/memory/regions/([a-z0-9][a-z0-9._-]*)$') {
        $node = Find-NamedManifestItem @($Manifest.memory.regions) $Matches[1]
        if ($null -ne $node) { $value = [ordered]@{ id = $node.id; kind = $node.kind; start_bytes = $node.start_bytes; length_bytes = $node.length_bytes } }
    }
    elseif ($Path -cmatch '^/devices/([a-z0-9][a-z0-9._-]*)/(identity|bdf)$') {
        $device = Find-NamedManifestItem @($Manifest.devices) $Matches[1]
        if ($null -ne $device) { $node = (Get-OrdinalJsonProperty $device $Matches[2]).Value; $value = $node.value }
    }
    elseif ($Path -cmatch '^/devices/([a-z0-9][a-z0-9._-]*)/resources/([0-9]+)$') {
        $device = Find-NamedManifestItem @($Manifest.devices) $Matches[1]; $index = [int]$Matches[2]
        if ($null -ne $device -and $index -lt @($device.resources).Count) {
            $node = @($device.resources)[$index]
            $value = [ordered]@{ kind = $node.kind; start = $node.start; length = $node.length }
        }
    }
    if ($null -eq $node) { return [ordered]@{ found = $false; path = $Path } }
    return [ordered]@{ found = $true; path = $Path; status = $node.status; value = $value; provenance = $node.provenance }
}

function Test-MachineManifest {
    param([object]$Manifest, [object]$Schema, [int]$LaunchMemoryMB = 0, [switch]$SkipHarnessBinding, [string]$ControlledMachineId = "")
    $errors = New-Object System.Collections.Generic.List[object]
    Test-AgainstSchema -Value $Manifest -Rule $Schema -SchemaRoot $Schema -Errors $errors
    if ($errors.Count -gt 0) { return [ordered]@{ valid = $false; machine_id = $null; curated_context_ready = $false; errors = [object[]]$errors } }

    $machineId = [string]$Manifest.machine.id
    function Check-ProvenanceNode([object]$Node, [string]$Path) {
        if ($null -eq $Node) { return }
        if ($Node -is [System.Array]) { for ($i = 0; $i -lt $Node.Count; $i++) { Check-ProvenanceNode $Node[$i] "$Path/$i" }; return }
        if ($Node -isnot [pscustomobject]) { return }
        foreach ($property in $Node.PSObject.Properties) {
            if ($property.Name -eq "provenance") {
                if ($property.Value.machine_id -cne $machineId) { Add-ManifestError $errors "provenance_machine_mismatch" "$Path/provenance" "Provenance is not bound to '$machineId'" }
                if ($machineId -eq "surface-pro-4" -and ($property.Value.source_kind -eq "launch_configuration" -or [string]$property.Value.source_ref -match '(?i)qemu|shadow-vm|run-stage0')) {
                    Add-ManifestError $errors "cross_machine_provenance" "$Path/provenance" "QEMU provenance cannot satisfy a Surface fact"
                }
            } else { Check-ProvenanceNode $property.Value "$Path/$($property.Name)" }
        }
    }
    Check-ProvenanceNode $Manifest ""

    function Check-Fact([object]$Fact, [string]$Path) {
        if ($Fact.status -eq "unknown" -and $null -ne $Fact.value) { Add-ManifestError $errors "unknown_fact_has_value" $Path "Unknown facts must have null value" }
        elseif ($Fact.status -eq "observed" -and $null -eq $Fact.value) { Add-ManifestError $errors "observed_fact_missing_value" $Path "Observed facts require a value" }
    }
    Check-Fact $Manifest.cpu.architecture "/cpu/architecture"; Check-Fact $Manifest.cpu.model "/cpu/model"
    $featureNames = @{}; foreach ($feature in @($Manifest.cpu.features)) {
        Check-Fact $feature "/cpu/features/$($feature.name)"
        if ($featureNames.ContainsKey($feature.name)) { Add-ManifestError $errors "duplicate_cpu_feature" "/cpu/features/$($feature.name)" "CPU feature names must be unique" } else { $featureNames[$feature.name] = $true }
    }
    Check-Fact $Manifest.memory.total_bytes "/memory/total_bytes"; Check-Fact $Manifest.memory.topology "/memory/topology"
    $regionIds = @{}; $observedRegions = @()
    foreach ($region in @($Manifest.memory.regions)) {
        if ($regionIds.ContainsKey($region.id)) { Add-ManifestError $errors "duplicate_memory_region" "/memory/regions/$($region.id)" "Memory region ids must be unique" } else { $regionIds[$region.id] = $true }
        if ($region.status -eq "unknown" -and ($null -ne $region.start_bytes -or $null -ne $region.length_bytes -or $region.kind -ne "unknown")) {
            Add-ManifestError $errors "unknown_region_has_range" "/memory/regions/$($region.id)" "Unknown regions require unknown kind and null range"
        } elseif ($region.status -eq "observed" -and ($null -eq $region.start_bytes -or $null -eq $region.length_bytes)) {
            Add-ManifestError $errors "observed_region_missing_range" "/memory/regions/$($region.id)" "Observed regions require start and length"
        } elseif ($region.status -eq "observed" -and $region.kind -eq "unknown") {
            Add-ManifestError $errors "observed_region_unknown_kind" "/memory/regions/$($region.id)" "Observed regions require a known kind"
        } elseif ($region.status -eq "observed") {
            $end = [decimal]$region.start_bytes + [decimal]$region.length_bytes
            if ($Manifest.memory.total_bytes.status -eq "observed" -and $end -gt [decimal]$Manifest.memory.total_bytes.value) { Add-ManifestError $errors "memory_region_out_of_range" "/memory/regions/$($region.id)" "Region exceeds observed total memory" }
            $observedRegions += [ordered]@{ id = $region.id; start = [decimal]$region.start_bytes; end = $end }
        }
    }
    for ($i = 0; $i -lt $observedRegions.Count; $i++) { for ($j = $i + 1; $j -lt $observedRegions.Count; $j++) {
        if ($observedRegions[$i].start -lt $observedRegions[$j].end -and $observedRegions[$j].start -lt $observedRegions[$i].end) { Add-ManifestError $errors "overlapping_memory_regions" "/memory/regions/$($observedRegions[$j].id)" "Observed memory regions overlap" }
    } }
    $deviceIds = @{}; $bdfs = @{}
    foreach ($device in @($Manifest.devices)) {
        if ($deviceIds.ContainsKey($device.id)) { Add-ManifestError $errors "duplicate_device_id" "/devices/$($device.id)" "Device ids must be unique" } else { $deviceIds[$device.id] = $true }
        Check-Fact $device.identity "/devices/$($device.id)/identity"; Check-Fact $device.bdf "/devices/$($device.id)/bdf"
        if ($device.bdf.status -eq "observed") { if ($bdfs.ContainsKey($device.bdf.value)) { Add-ManifestError $errors "duplicate_device_bdf" "/devices/$($device.id)/bdf" "Observed BDFs must be unique" } else { $bdfs[$device.bdf.value] = $true } }
        for ($i = 0; $i -lt @($device.resources).Count; $i++) {
            $resource = @($device.resources)[$i]
            if ($resource.status -eq "unknown" -and ($null -ne $resource.start -or $null -ne $resource.length -or $resource.kind -ne "unknown")) { Add-ManifestError $errors "unknown_resource_has_range" "/devices/$($device.id)/resources/$i" "Unknown resources require unknown kind and null range" }
            elseif ($resource.status -eq "observed" -and ($null -eq $resource.start -or $null -eq $resource.length)) { Add-ManifestError $errors "observed_resource_missing_range" "/devices/$($device.id)/resources/$i" "Observed resources require start and length" }
            elseif ($resource.status -eq "observed" -and $resource.kind -eq "unknown") { Add-ManifestError $errors "observed_resource_unknown_kind" "/devices/$($device.id)/resources/$i" "Observed resources require a known kind" }
        }
    }

    if ($machineId -ceq "qemu-q35-shadow" -or $ControlledMachineId -ceq "qemu-q35-shadow") {
        $controlledContract = Get-QemuQ35ShadowControlledContract
        $controlledDifference = Get-FirstJsonDifferencePath $Manifest $controlledContract
        if ($null -ne $controlledDifference) {
            Add-ManifestError $errors "qemu_controlled_truth_drift" $controlledDifference "QEMU manifest differs from the explicit repository-controlled launch truth"
        }
    }

    $baselineMissing = @()
    foreach ($path in @("/cpu/architecture", "/cpu/model", "/memory/total_bytes", "/memory/topology")) { if ((Resolve-MachineManifestFact $Manifest $path).status -ne "observed") { $baselineMissing += $path } }
    foreach ($feature in @($Manifest.cpu.features)) { if ($feature.status -ne "observed") { $baselineMissing += "/cpu/features/$($feature.name)" } }
    foreach ($region in @($Manifest.memory.regions)) { if ($region.status -ne "observed") { $baselineMissing += "/memory/regions/$($region.id)" } }
    foreach ($device in @($Manifest.devices)) { if ($device.identity.status -ne "observed") { $baselineMissing += "/devices/$($device.id)/identity" } }
    $declaredMissing = @($Manifest.missing_required_facts)
    if ($Manifest.curated_context_ready) {
        if ($declaredMissing.Count -ne 0) { Add-ManifestError $errors "ready_manifest_declares_missing_facts" "/missing_required_facts" "Ready manifests cannot declare missing facts" }
        foreach ($path in $baselineMissing) { Add-ManifestError $errors "ready_fact_unknown" $path "Ready manifest baseline facts must be observed" }
    } else {
        if ($declaredMissing.Count -eq 0) { Add-ManifestError $errors "not_ready_without_missing_facts" "/missing_required_facts" "Non-ready manifests must declare missing facts" }
        foreach ($path in $declaredMissing) {
            $fact = Resolve-MachineManifestFact $Manifest $path
            if (-not $fact.found) { Add-ManifestError $errors "missing_fact_path_unknown" $path "Declared missing fact does not exist" }
            elseif ($fact.status -ne "unknown") { Add-ManifestError $errors "missing_fact_is_observed" $path "Declared missing fact is already observed" }
        }
        foreach ($path in $baselineMissing) { if (-not (Test-OrdinalStringIn $path $declaredMissing)) { Add-ManifestError $errors "missing_fact_not_declared" $path "Baseline unknown fact is not declared" } }
        foreach ($path in $declaredMissing) { if (-not (Test-OrdinalStringIn $path $baselineMissing)) { Add-ManifestError $errors "nonbaseline_missing_fact" $path "Declared missing fact is not a baseline readiness fact" } }
    }

    if (-not $SkipHarnessBinding -and $machineId -eq "qemu-q35-shadow") {
        $smokeSource = Get-Content -Raw -LiteralPath (Join-Path $HardwareManifestRepoRoot "vm-harness\shadow-vm-smoke.ps1")
        $supportSource = Get-Content -Raw -LiteralPath (Join-Path $HardwareManifestRepoRoot "vm-harness\shadow-vm-smoke-support.ps1")
        $launchSource = Get-Content -Raw -LiteralPath (Join-Path $HardwareManifestRepoRoot "scripts\run-stage0-qemu.ps1")
        if ($LaunchMemoryMB -le 0) { $match = [regex]::Match($smokeSource, '\[int\]\$GuestMemoryMB\s*=\s*(\d+)'); if ($match.Success) { $LaunchMemoryMB = [int]$match.Groups[1].Value } else { Add-ManifestError $errors "harness_memory_source_missing" "/memory/total_bytes" "Could not resolve GuestMemoryMB" } }
        if ($supportSource -notmatch 'memory\s*=\s*"\$\{MemoryMB\}M"' -or $supportSource -notmatch 'Get-Variable\s+-Name\s+GuestMemoryMB' -or $smokeSource -notmatch 'MemoryMB\s*=\s*\$GuestMemoryMB' -or $launchSource -notmatch '"-m",\s*"\$\{MemoryMB\}M"') { Add-ManifestError $errors "harness_memory_binding_missing" "/memory/total_bytes" "Harness and QEMU memory are not bound" }
        if ($Manifest.memory.total_bytes.value -ne ([int64]$LaunchMemoryMB * 1MB)) { Add-ManifestError $errors "qemu_profile_launch_drift" "/memory/total_bytes" "Manifest memory does not match ${LaunchMemoryMB} MiB" }
        if ($launchSource -cnotmatch '"-machine",\s*"q35"' -or $Manifest.machine.kind -cne "virtual") { Add-ManifestError $errors "qemu_machine_launch_drift" "/machine" "Manifest does not bind q35 virtual launch" }
        $architecture = $Manifest.cpu.architecture
        if ($architecture.status -cne "observed" -or $architecture.value -cne "x86_64" -or
            $architecture.provenance.source_kind -cne "launch_configuration" -or
            $architecture.provenance.source_ref -cne "scripts/run-stage0-qemu.ps1:qemu-system-x86_64" -or
            $launchSource -cnotmatch 'qemu-system-x86_64') {
            Add-ManifestError $errors "qemu_architecture_launch_drift" "/cpu/architecture" "Manifest architecture or provenance does not match the controlled QEMU executable"
        }
        if ($Manifest.cpu.model.value -cne "max" -or $smokeSource -cnotmatch 'Cpu\s*=\s*"max"' -or $launchSource -cnotmatch '@\("-cpu",\s*\$Cpu\)') { Add-ManifestError $errors "qemu_cpu_launch_drift" "/cpu/model" "Manifest CPU does not bind -cpu max" }
        $feature = Find-NamedManifestItem @($Manifest.cpu.features) "qemu-max-feature-set"
        if ($null -eq $feature -or $feature.status -ne "observed" -or $feature.value -ne $true) { Add-ManifestError $errors "qemu_feature_launch_drift" "/cpu/features/qemu-max-feature-set" "Configured max feature set is not observed" }
        $expectedTopology = "single guest address space; no explicit NUMA node, DIMM, or memory-backend configuration"
        if ($Manifest.memory.topology.value -cne $expectedTopology -or $launchSource -cmatch '"-numa"|memory-backend|pc-dimm') { Add-ManifestError $errors "qemu_topology_launch_drift" "/memory/topology" "Manifest topology does not match launch arguments" }
        $qemuRegions = @($Manifest.memory.regions)
        $expectedRegionSource = "vm-harness/shadow-vm-smoke.ps1:GuestMemoryMB=$LaunchMemoryMB -> scripts/run-stage0-qemu.ps1:-m `${MemoryMB}M"
        if ($qemuRegions.Count -cne 1) {
            Add-ManifestError $errors "qemu_ram_region_launch_drift" "/memory/regions" "Controlled QEMU profile exposes exactly one rendered RAM-capacity region"
        }
        foreach ($region in $qemuRegions) {
            if ($region.id -cne "guest-ram-capacity" -or $region.kind -cne "ram" -or $region.status -cne "observed" -or
                [int64]$region.start_bytes -ne 0 -or [int64]$region.length_bytes -ne ([int64]$LaunchMemoryMB * 1MB) -or
                $region.provenance.source_kind -cne "launch_configuration" -or $region.provenance.source_ref -cne $expectedRegionSource) {
                Add-ManifestError $errors "qemu_ram_region_launch_drift" "/memory/regions/$($region.id)" "RAM-region value or provenance does not match the controlled QEMU memory profile"
            }
        }
        foreach ($binding in @(@("q35", "q35", '"-machine",\s*"q35"'), @("xhci", "qemu-xhci", '"-device",\s*"qemu-xhci,id=xhci"'), @("bootkbd", "usb-kbd", '"-device",\s*"usb-kbd,bus=xhci\.0,id=bootkbd"'), @("pointer", "usb-tablet", '"-device",\s*"usb-tablet,bus=xhci\.0"'))) {
            $device = Find-NamedManifestItem @($Manifest.devices) $binding[0]
            if ($null -eq $device -or $device.identity.value -cne $binding[1] -or $launchSource -cnotmatch $binding[2]) { Add-ManifestError $errors "qemu_device_launch_drift" "/devices/$($binding[0])/identity" "Manifest device does not match QEMU launch" }
        }
    }
    return [ordered]@{ valid = ($errors.Count -eq 0); machine_id = $machineId; curated_context_ready = [bool]$Manifest.curated_context_ready; errors = [object[]]$errors }
}

function Get-MachineManifestSchema {
    $schema = Get-Content -Raw -LiteralPath $HardwareManifestSchemaPath | ConvertFrom-Json
    Assert-SupportedSchema -Rule $schema -SchemaRoot $schema
    if ($schema.properties.schema_version.const -ne "raios.machine_manifest.v1") { throw "Schema const does not bind v1" }
    return $schema
}

function Invoke-HardwareManifestSelfTests {
    param([object]$Qemu, [object]$Surface, [object]$Schema)
    $cases = New-Object System.Collections.Generic.List[object]
    function Run-Case([string]$Name, [object]$Mutant, [string]$ExpectedCode, [int]$LaunchMemoryMB = 0, [switch]$UseHarness, [string]$ControlledMachineId = "") {
        $result = Test-MachineManifest $Mutant $Schema -LaunchMemoryMB $LaunchMemoryMB -SkipHarnessBinding:(-not $UseHarness) -ControlledMachineId $ControlledMachineId
        $codes = @($result.errors | ForEach-Object { $_.code })
        $cases.Add([ordered]@{ name = $Name; expected = "rejected:$ExpectedCode"; passed = (-not $result.valid -and $ExpectedCode -in $codes); actual_codes = $codes })
    }
    $m = Copy-JsonObject $Qemu; $m.schema_version = "raios.machine_manifest.v0"; Run-Case "schema-version" $m "schema_version_mismatch"
    $m = Copy-JsonObject $Qemu; $m.PSObject.Properties.Remove("cpu"); Run-Case "missing-cpu" $m "missing_cpu"
    $m = Copy-JsonObject $Qemu; $cpu = Copy-JsonObject $m.cpu; $m.PSObject.Properties.Remove("cpu"); $m | Add-Member -MemberType NoteProperty -Name "CPU" -Value $cpu; Run-Case "property-name-case-required" $m "missing_cpu"; Run-Case "property-name-case-additional" $m "additional_property"
    $m = Copy-JsonObject $Qemu; $m.PSObject.Properties.Remove("memory"); Run-Case "missing-memory" $m "missing_memory"
    $m = Copy-JsonObject $Qemu; $m.devices = @(); Run-Case "missing-devices" $m "missing_devices"
    $m = Copy-JsonObject $Qemu; $r = Copy-JsonObject $m.memory.regions[0]; $r.id = "overlap"; $r.start_bytes = 1; $r.length_bytes = 16; $m.memory.regions = @($m.memory.regions) + @($r); Run-Case "overlapping-memory" $m "overlapping_memory_regions"
    $m = Copy-JsonObject $Qemu; $m.memory.regions[0].length_bytes = [int64]$m.memory.total_bytes.value + 1; Run-Case "out-of-range-memory" $m "memory_region_out_of_range"
    $m = Copy-JsonObject $Qemu; $d = Copy-JsonObject $m.devices[0]; $m.devices = @($m.devices) + @($d); Run-Case "duplicate-device-id" $m "duplicate_device_id"
    $m = Copy-JsonObject $Qemu; foreach ($i in @(0,1)) { $m.devices[$i].bdf.status = "observed"; $m.devices[$i].bdf.value = "0000:00:01.0" }; Run-Case "duplicate-device-bdf" $m "duplicate_device_bdf"
    $m = Copy-JsonObject $Qemu; $m.cpu.model.provenance.source_ref = ""; Run-Case "invalid-provenance" $m "string_too_short"
    $m = Copy-JsonObject $Qemu; Run-Case "qemu-512-vs-8192-drift" $m "qemu_profile_launch_drift" 8192 -UseHarness
    $m = Copy-JsonObject $Qemu; $m.cpu.architecture.value = "aarch64"; Run-Case "qemu-architecture-drift" $m "qemu_architecture_launch_drift" 512 -UseHarness
    $m = Copy-JsonObject $Qemu; $m.cpu.architecture.provenance.source_ref = "uncontrolled-observation"; Run-Case "qemu-architecture-provenance-drift" $m "qemu_architecture_launch_drift" 512 -UseHarness
    $m = Copy-JsonObject $Qemu; $m.memory.regions[0].length_bytes = [int64]$m.memory.regions[0].length_bytes - 1; Run-Case "qemu-ram-region-drift" $m "qemu_ram_region_launch_drift" 512 -UseHarness
    $m = Copy-JsonObject $Qemu; $m.memory.regions[0].provenance.source_ref = "uncontrolled-observation"; Run-Case "qemu-ram-region-provenance-drift" $m "qemu_ram_region_launch_drift" 512 -UseHarness
    $m = Copy-JsonObject $Surface; $m.cpu.architecture.provenance = Copy-JsonObject $Qemu.cpu.architecture.provenance; Run-Case "cross-machine-provenance" $m "provenance_machine_mismatch"
    $m = Copy-JsonObject $Surface; $m.memory.total_bytes.value = 536870912; Run-Case "unknown-cannot-carry-qemu-value" $m "unknown_fact_has_value"

    function Run-QemuControlledCase([string]$Name, [scriptblock]$Mutation) {
        $controlledMutant = Copy-JsonObject $Qemu
        & $Mutation $controlledMutant
        Run-Case $Name $controlledMutant "qemu_controlled_truth_drift"
    }
    Run-QemuControlledCase "qemu-machine-kind-controlled" { param($x) $x.machine.kind = "physical" }
    Run-QemuControlledCase "qemu-machine-name-controlled" { param($x) $x.machine.display_name = "repinned impostor" }
    $m = Copy-JsonObject $Qemu; $m.machine.id = "repinned-qemu"
    function Rewrite-ProvenanceMachineId([object]$Node) {
        if ($null -eq $Node) { return }
        if ($Node -is [System.Array]) { foreach ($item in $Node) { Rewrite-ProvenanceMachineId $item }; return }
        if ($Node -isnot [pscustomobject]) { return }
        foreach ($property in $Node.PSObject.Properties) {
            if ($property.Name -ceq "provenance") { $property.Value.machine_id = "repinned-qemu" }
            else { Rewrite-ProvenanceMachineId $property.Value }
        }
    }
    Rewrite-ProvenanceMachineId $m
    Run-Case "qemu-canonical-path-machine-binding" $m "qemu_controlled_truth_drift" -ControlledMachineId "qemu-q35-shadow"
    Run-QemuControlledCase "qemu-architecture-status-controlled" { param($x) $x.cpu.architecture.status = "unknown"; $x.cpu.architecture.value = $null }
    Run-QemuControlledCase "qemu-model-value-controlled" { param($x) $x.cpu.model.value = "host" }
    Run-QemuControlledCase "qemu-model-status-controlled" { param($x) $x.cpu.model.status = "unknown"; $x.cpu.model.value = $null }
    Run-QemuControlledCase "qemu-feature-value-controlled" { param($x) $x.cpu.features[0].value = $false }
    Run-QemuControlledCase "qemu-feature-status-controlled" { param($x) $x.cpu.features[0].status = "unknown"; $x.cpu.features[0].value = $null }
    Run-QemuControlledCase "qemu-additional-feature-controlled" { param($x) $f = Copy-JsonObject $x.cpu.features[0]; $f.name = "invented-feature"; $x.cpu.features = @($x.cpu.features) + @($f) }
    Run-QemuControlledCase "qemu-memory-value-controlled" { param($x) $x.memory.total_bytes.value = 536870911 }
    Run-QemuControlledCase "qemu-memory-status-controlled" { param($x) $x.memory.total_bytes.status = "unknown"; $x.memory.total_bytes.value = $null }
    Run-QemuControlledCase "qemu-topology-value-controlled" { param($x) $x.memory.topology.value = "invented numa topology" }
    Run-QemuControlledCase "qemu-topology-status-controlled" { param($x) $x.memory.topology.status = "unknown"; $x.memory.topology.value = $null }
    Run-QemuControlledCase "qemu-region-id-controlled" { param($x) $x.memory.regions[0].id = "invented-region" }
    Run-QemuControlledCase "qemu-region-kind-controlled" { param($x) $x.memory.regions[0].kind = "reserved" }
    Run-QemuControlledCase "qemu-region-start-controlled" { param($x) $x.memory.regions[0].start_bytes = 1; $x.memory.regions[0].length_bytes = 536870911 }
    Run-QemuControlledCase "qemu-region-length-controlled" { param($x) $x.memory.regions[0].length_bytes = 536870911 }
    Run-QemuControlledCase "qemu-region-status-controlled" { param($x) $x.memory.regions[0].status = "unknown"; $x.memory.regions[0].kind = "unknown"; $x.memory.regions[0].start_bytes = $null; $x.memory.regions[0].length_bytes = $null }
    Run-QemuControlledCase "qemu-additional-observed-region-controlled" { param($x) $x.memory.regions[0].length_bytes = 268435456; $r = Copy-JsonObject $x.memory.regions[0]; $r.id = "invented-region"; $r.start_bytes = 268435456; $x.memory.regions = @($x.memory.regions) + @($r) }
    foreach ($deviceIndex in 0..(@($Qemu.devices).Count - 1)) {
        $deviceName = [string]$Qemu.devices[$deviceIndex].id
        Run-QemuControlledCase "qemu-$deviceName-id-controlled" { param($x) $x.devices[$deviceIndex].id = "invented-$deviceName" }.GetNewClosure()
        Run-QemuControlledCase "qemu-$deviceName-type-controlled" { param($x) $x.devices[$deviceIndex].type = "other" }.GetNewClosure()
        Run-QemuControlledCase "qemu-$deviceName-identity-value-controlled" { param($x) $x.devices[$deviceIndex].identity.value = "invented-$deviceName" }.GetNewClosure()
    }
    Run-QemuControlledCase "qemu-device-identity-status-controlled" { param($x) $x.devices[1].identity.status = "unknown"; $x.devices[1].identity.value = $null }
    Run-QemuControlledCase "qemu-bdf-unknown-to-observed-controlled" { param($x) $x.devices[1].bdf.status = "observed"; $x.devices[1].bdf.value = "0000:00:01.0" }
    Run-QemuControlledCase "qemu-additional-observed-resource-controlled" { param($x) $x.devices[1].resources = @([pscustomobject]@{ kind = "mmio"; start = 4096; length = 4096; status = "observed"; provenance = Copy-JsonObject $x.devices[1].provenance }) }
    Run-QemuControlledCase "qemu-resource-provenance-drift" { param($x) $x.devices[1].resources = @([pscustomobject]@{ kind = "mmio"; start = 4096; length = 4096; status = "observed"; provenance = [pscustomobject]@{ machine_id = "qemu-q35-shadow"; source_kind = "boot_observation"; source_ref = "invented resource"; observed_at_utc = "2026-07-19T00:00:00Z" } }) }
    Run-QemuControlledCase "qemu-additional-device-controlled" { param($x) $d = Copy-JsonObject $x.devices[1]; $d.id = "invented-device"; $x.devices = @($x.devices) + @($d) }

    $provenanceTargets = @(
        @{ name = "machine"; get = { param($x) $x.machine.provenance } },
        @{ name = "architecture"; get = { param($x) $x.cpu.architecture.provenance } },
        @{ name = "model"; get = { param($x) $x.cpu.model.provenance } },
        @{ name = "feature"; get = { param($x) $x.cpu.features[0].provenance } },
        @{ name = "memory"; get = { param($x) $x.memory.total_bytes.provenance } },
        @{ name = "topology"; get = { param($x) $x.memory.topology.provenance } },
        @{ name = "region"; get = { param($x) $x.memory.regions[0].provenance } }
    )
    for ($deviceIndex = 0; $deviceIndex -lt @($Qemu.devices).Count; $deviceIndex++) {
        $deviceName = [string]$Qemu.devices[$deviceIndex].id
        $provenanceTargets += @{ name = "$deviceName-device"; get = { param($x) $x.devices[$deviceIndex].provenance }.GetNewClosure() }
        $provenanceTargets += @{ name = "$deviceName-identity"; get = { param($x) $x.devices[$deviceIndex].identity.provenance }.GetNewClosure() }
        $provenanceTargets += @{ name = "$deviceName-bdf"; get = { param($x) $x.devices[$deviceIndex].bdf.provenance }.GetNewClosure() }
    }
    foreach ($target in $provenanceTargets) {
        foreach ($field in @("machine_id", "source_kind", "source_ref", "observed_at_utc")) {
            $targetName = [string]$target.name; $getTarget = $target.get
            Run-QemuControlledCase "qemu-$targetName-$field-controlled" {
                param($x)
                $provenance = & $getTarget $x
                switch ($field) {
                    "machine_id" { $provenance.machine_id = "surface-pro-4" }
                    "source_kind" { $provenance.source_kind = $(if ($provenance.source_kind -ceq "documentation_gap") { "launch_configuration" } else { "boot_observation" }) }
                    "source_ref" { $provenance.source_ref = "invented controlled source" }
                    "observed_at_utc" { $provenance.observed_at_utc = "2026-07-20T00:00:00Z" }
                }
            }.GetNewClosure()
        }
    }

    $m = Copy-JsonObject $Qemu; $m | Add-Member unexpected $true; Run-Case "root-additional-property" $m "additional_property"
    $m = Copy-JsonObject $Qemu; $m.curated_context_ready = "true"; Run-Case "readiness-wrong-type" $m "type_mismatch"
    $m = Copy-JsonObject $Qemu; $m.machine.id = "QEMU"; Run-Case "machine-id-pattern" $m "pattern_mismatch"
    $m = Copy-JsonObject $Qemu; $m.machine.kind = "emulated"; Run-Case "machine-kind-enum" $m "enum_mismatch"
    $m = Copy-JsonObject $Qemu; $m.machine.display_name = ""; Run-Case "display-name-empty" $m "string_too_short"
    $m = Copy-JsonObject $Qemu; $m.machine.display_name = "x" * 161; Run-Case "display-name-bounded" $m "string_too_long"
    $m = Copy-JsonObject $Qemu; $m.machine.PSObject.Properties.Remove("display_name"); Run-Case "nested-required-property" $m "required_property_missing"
    $m = Copy-JsonObject $Qemu; $m.machine.provenance.observed_at_utc = "yesterday"; Run-Case "provenance-date-time" $m "format_mismatch"
    $m = Copy-JsonObject $Qemu; $m.cpu.features = @(); Run-Case "features-non-empty" $m "array_too_short"
    $m = Copy-JsonObject $Qemu; $m.cpu.features = @(1..129 | ForEach-Object { $f = Copy-JsonObject $Qemu.cpu.features[0]; $f.name = "feature-$_"; $f }); Run-Case "features-bounded" $m "array_too_long"
    $m = Copy-JsonObject $Qemu; $m.cpu.features[0].value = "true"; Run-Case "boolean-fact-type" $m "type_mismatch"
    $m = Copy-JsonObject $Qemu; $m.cpu.model | Add-Member extra "x"; Run-Case "fact-additional-property" $m "additional_property"
    $m = Copy-JsonObject $Qemu; $m.cpu.model.status = "guessed"; Run-Case "fact-status-enum" $m "enum_mismatch"
    $m = Copy-JsonObject $Qemu; $m.cpu.model.value = $null; Run-Case "observed-fact-needs-value" $m "observed_fact_missing_value"
    $m = Copy-JsonObject $Qemu; $f = Copy-JsonObject $m.cpu.features[0]; $m.cpu.features = @($m.cpu.features) + @($f); Run-Case "duplicate-feature-name" $m "duplicate_cpu_feature"
    $m = Copy-JsonObject $Qemu; $m.memory.total_bytes.value = -1; Run-Case "integer-minimum" $m "below_minimum"
    $m = Copy-JsonObject $Qemu; $m.memory.regions = @(); Run-Case "regions-non-empty" $m "array_too_short"
    $m = Copy-JsonObject $Qemu; $m.memory.regions[0].start_bytes = "0"; Run-Case "region-integer-type" $m "type_mismatch"
    $m = Copy-JsonObject $Qemu; $m.memory.regions[0].start_bytes = $null; Run-Case "observed-region-needs-range" $m "observed_region_missing_range"
    $m = Copy-JsonObject $Qemu; $r = Copy-JsonObject $m.memory.regions[0]; $m.memory.regions = @($m.memory.regions) + @($r); Run-Case "duplicate-region-id" $m "duplicate_memory_region"
    $m = Copy-JsonObject $Qemu; $m.devices[0].type = "bus"; Run-Case "device-type-enum" $m "enum_mismatch"
    $m = Copy-JsonObject $Qemu; $m.devices[0].bdf.status = "observed"; $m.devices[0].bdf.value = "00:01.0"; Run-Case "bdf-pattern" $m "pattern_mismatch"
    $m = Copy-JsonObject $Qemu; $m.devices[0].resources = @([pscustomobject]@{ kind = "unknown"; start = $null; length = $null; status = "unknown"; provenance = Copy-JsonObject $m.devices[0].provenance }); $m.devices[0].resources[0] | Add-Member extra 1; Run-Case "resource-additional-property" $m "additional_property"
    $m = Copy-JsonObject $Qemu; $m.devices[0].resources = @([pscustomobject]@{ kind = "irq"; start = 11; length = $null; status = "observed"; provenance = Copy-JsonObject $m.devices[0].provenance }); Run-Case "observed-resource-needs-range" $m "observed_resource_missing_range"
    $m = Copy-JsonObject $Qemu; $m.missing_required_facts = @("/cpu/model", "/cpu/model"); Run-Case "missing-paths-unique" $m "duplicate_array_item"
    $m = Copy-JsonObject $Qemu; $m.missing_required_facts = @("bad-path"); Run-Case "fact-path-pattern" $m "pattern_mismatch"
    $m = Copy-JsonObject $Qemu; $m.missing_required_facts = @("/cpu/model"); Run-Case "ready-no-missing" $m "ready_manifest_declares_missing_facts"
    $m = Copy-JsonObject $Surface; $m.missing_required_facts = @(); Run-Case "not-ready-declares-missing" $m "not_ready_without_missing_facts"
    $m = Copy-JsonObject $Surface; $m.missing_required_facts[0] = "/devices/xhci/identity"; Run-Case "missing-path-must-be-unknown" $m "missing_fact_is_observed"
    $m = Copy-JsonObject $Qemu; $m.cpu.model.provenance.machine_id = "surface-pro-4"; Run-Case "provenance-machine-binding" $m "provenance_machine_mismatch"
    $m = Copy-JsonObject $Qemu; $m.machine.provenance | Add-Member extra "x"; Run-Case "object-property-bound" $m "object_too_large"

    $pointerCase = Resolve-MachineManifestFact $Qemu "/CPU/model"
    $cases.Add([ordered]@{ name = "json-pointer-case-sensitive"; expected = "not-found:/CPU/model"; passed = (-not $pointerCase.found); actual_codes = @("found=$($pointerCase.found)") })

    $qemuResult = Test-MachineManifest $Qemu $Schema
    $surfaceResult = Test-MachineManifest $Surface $Schema
    return [ordered]@{ original_manifests_green = ($qemuResult.valid -and $surfaceResult.valid); mutation_count = $cases.Count;
        mutation_passed_count = @($cases | Where-Object { $_.passed }).Count;
        passed = ($qemuResult.valid -and $surfaceResult.valid -and @($cases | Where-Object { -not $_.passed }).Count -eq 0); cases = [object[]]$cases }
}

function Invoke-HardwareManifestCheck {
    param([string[]]$Paths, [switch]$RunSelfTest)
    try {
        $schema = Get-MachineManifestSchema
        $selectedPaths = if ($Paths.Count -gt 0) { $Paths } else { $HardwareManifestDefaultPaths }
        $results = @(); $loaded = @{}
        foreach ($path in $selectedPaths) {
            $resolved = (Resolve-Path -LiteralPath $path).Path
            $manifest = Get-Content -Raw -LiteralPath $resolved | ConvertFrom-Json
            $canonicalQemuPath = [IO.Path]::GetFullPath($HardwareManifestDefaultPaths[0])
            $controlledMachineId = if ([IO.Path]::GetFullPath($resolved) -ceq $canonicalQemuPath) { "qemu-q35-shadow" } else { "" }
            $validation = Test-MachineManifest $manifest $schema -ControlledMachineId $controlledMachineId
            if ($null -ne $validation.machine_id) { $loaded[[string]$validation.machine_id] = $manifest }
            $results += [ordered]@{ path = $resolved; machine_id = $validation.machine_id; valid = $validation.valid; curated_context_ready = $validation.curated_context_ready; errors = $validation.errors }
        }
        $selfTestResult = $null
        if ($RunSelfTest) {
            if (-not $loaded.ContainsKey("qemu-q35-shadow") -or -not $loaded.ContainsKey("surface-pro-4")) { throw "SelfTest requires both canonical manifests" }
            $selfTestResult = Invoke-HardwareManifestSelfTests $loaded["qemu-q35-shadow"] $loaded["surface-pro-4"] $schema
        }
        $valid = @($results | Where-Object { -not $_.valid }).Count -eq 0 -and ($null -eq $selfTestResult -or $selfTestResult.passed)
        [ordered]@{ schema = "raios.hardware_manifest_validation.v1"; generated_at_utc = [DateTime]::UtcNow.ToString("o"); valid = $valid;
            manifest_count = $results.Count; results = $results; self_test = $selfTestResult } | ConvertTo-Json -Depth 100
        if (-not $valid) { return 1 }; return 0
    } catch {
        [ordered]@{ schema = "raios.hardware_manifest_validation.v1"; generated_at_utc = [DateTime]::UtcNow.ToString("o"); valid = $false;
            fatal = $_.Exception.Message; location = $_.InvocationInfo.PositionMessage } | ConvertTo-Json -Depth 10
        return 1
    }
}

if ($MyInvocation.InvocationName -ne '.') {
    $runResult = @(Invoke-HardwareManifestCheck -Paths $ManifestPath -RunSelfTest:$SelfTest)
    for ($i = 0; $i -lt $runResult.Count - 1; $i++) { Write-Output $runResult[$i] }
    exit [int]$runResult[$runResult.Count - 1]
}
