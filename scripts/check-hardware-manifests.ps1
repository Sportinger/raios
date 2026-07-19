param(
    [string[]]$ManifestPath = @(),
    [switch]$SelfTest
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$SchemaPath = Join-Path $RepoRoot "hardware\schemas\machine-manifest.v1.json"
$DefaultManifestPaths = @(
    (Join-Path $RepoRoot "hardware\manifests\qemu-q35-shadow.v1.json"),
    (Join-Path $RepoRoot "hardware\manifests\surface-pro-4.v1.json")
)

function Copy-JsonObject {
    param([object]$Value)
    return ($Value | ConvertTo-Json -Depth 100 | ConvertFrom-Json)
}

function Test-HasProperty {
    param([object]$Object, [string]$Name)
    return $null -ne $Object -and $null -ne $Object.PSObject.Properties[$Name]
}

function New-ValidationResult {
    param([object]$Manifest, [int]$LaunchMemoryMB = 0, [switch]$SkipHarnessBinding)

    $errors = New-Object System.Collections.Generic.List[object]
    function Add-Error([string]$Code, [string]$Path, [string]$Message) {
        $errors.Add([ordered]@{ code = $Code; path = $Path; message = $Message })
    }

    if (-not (Test-HasProperty $Manifest "schema_version") -or $Manifest.schema_version -ne "raios.machine_manifest.v1") {
        Add-Error "schema_version_mismatch" "/schema_version" "Expected raios.machine_manifest.v1"
    }
    foreach ($required in @("machine", "cpu", "memory", "devices")) {
        if (-not (Test-HasProperty $Manifest $required) -or $null -eq $Manifest.$required) {
            Add-Error "missing_$required" "/$required" "Required manifest section is missing"
        }
    }
    if ($errors.Count -gt 0 -and (-not (Test-HasProperty $Manifest "machine"))) {
        return [ordered]@{ valid = $false; machine_id = $null; errors = [object[]]$errors }
    }

    $machineId = [string]$Manifest.machine.id
    if ($machineId -notmatch '^[a-z0-9][a-z0-9.-]+$') {
        Add-Error "invalid_machine_id" "/machine/id" "Machine id is not a stable lower-case identifier"
    }
    if ($Manifest.machine.kind -notin @("virtual", "physical")) {
        Add-Error "invalid_machine_kind" "/machine/kind" "Machine kind must be virtual or physical"
    }
    foreach ($name in @("id", "kind", "display_name", "provenance")) {
        if (-not (Test-HasProperty $Manifest.machine $name)) { Add-Error "invalid_machine_shape" "/machine/$name" "Required machine field is missing" }
    }

    # Every fact/resource/region provenance is recursively machine-bound.
    function Walk-Provenance([object]$Node, [string]$Path) {
        if ($null -eq $Node) { return }
        if ($Node -is [System.Array]) {
            for ($i = 0; $i -lt $Node.Count; $i++) { Walk-Provenance $Node[$i] "$Path/$i" }
            return
        }
        if ($Node -isnot [pscustomobject]) { return }
        foreach ($property in $Node.PSObject.Properties) {
            $propertyPath = "$Path/$($property.Name)"
            if ($property.Name -eq "provenance") {
                $p = $property.Value
                $validTimestamp = $false
                if ($null -ne $p -and (Test-HasProperty $p "observed_at_utc")) {
                    $parsed = [DateTimeOffset]::MinValue
                    $validTimestamp = [DateTimeOffset]::TryParse([string]$p.observed_at_utc, [ref]$parsed)
                }
                if ($null -eq $p -or
                    -not (Test-HasProperty $p "machine_id") -or
                    -not (Test-HasProperty $p "source_kind") -or
                    -not (Test-HasProperty $p "source_ref") -or
                    -not $validTimestamp -or
                    [string]::IsNullOrWhiteSpace([string]$p.source_ref) -or
                    $p.source_kind -notin @("launch_configuration", "boot_observation", "documentation_gap")) {
                    Add-Error "invalid_provenance" $propertyPath "Provenance requires machine, typed source, source reference, and timestamp"
                }
                elseif ($p.machine_id -ne $machineId) {
                    Add-Error "provenance_machine_mismatch" $propertyPath "Provenance belongs to '$($p.machine_id)', not '$machineId'"
                }
                if ($machineId -eq "surface-pro-4" -and $null -ne $p -and
                    ($p.machine_id -ne "surface-pro-4" -or $p.source_kind -eq "launch_configuration" -or [string]$p.source_ref -match '(?i)qemu|shadow-vm|run-stage0')) {
                    Add-Error "cross_machine_provenance" $propertyPath "QEMU provenance cannot satisfy a Surface fact"
                }
            }
            else { Walk-Provenance $property.Value $propertyPath }
        }
    }
    Walk-Provenance $Manifest ""

    function Check-Fact([object]$Fact, [string]$Path) {
        if ($null -eq $Fact -or -not (Test-HasProperty $Fact "status") -or -not (Test-HasProperty $Fact "value") -or -not (Test-HasProperty $Fact "provenance")) {
            Add-Error "invalid_fact_shape" $Path "Fact requires status, value, and provenance"
            return
        }
        if ($Fact.status -eq "unknown" -and $null -ne $Fact.value) {
            Add-Error "unknown_fact_has_value" $Path "Unknown facts must have a null value"
        }
        elseif ($Fact.status -eq "observed" -and $null -eq $Fact.value) {
            Add-Error "observed_fact_missing_value" $Path "Observed facts require a value"
        }
        elseif ($Fact.status -notin @("observed", "unknown")) {
            Add-Error "invalid_fact_status" $Path "Fact status must be observed or unknown"
        }
    }

    if (Test-HasProperty $Manifest "cpu") {
        foreach ($name in @("architecture", "model", "features")) {
            if (-not (Test-HasProperty $Manifest.cpu $name)) { Add-Error "missing_cpu_$name" "/cpu/$name" "Required CPU field is missing" }
        }
        if (Test-HasProperty $Manifest.cpu "architecture") { Check-Fact $Manifest.cpu.architecture "/cpu/architecture" }
        if (Test-HasProperty $Manifest.cpu "model") { Check-Fact $Manifest.cpu.model "/cpu/model" }
        if (Test-HasProperty $Manifest.cpu "features") {
            $featureNames = @{}
            for ($i = 0; $i -lt @($Manifest.cpu.features).Count; $i++) {
                $feature = @($Manifest.cpu.features)[$i]
                Check-Fact $feature "/cpu/features/$i"
                if ([string]::IsNullOrWhiteSpace([string]$feature.name)) { Add-Error "invalid_cpu_feature_name" "/cpu/features/$i/name" "Feature name is required" }
                elseif ($featureNames.ContainsKey([string]$feature.name)) { Add-Error "duplicate_cpu_feature" "/cpu/features/$i/name" "CPU feature names must be unique" }
                else { $featureNames[[string]$feature.name] = $true }
            }
        }
    }

    if (Test-HasProperty $Manifest "memory") {
        foreach ($name in @("total_bytes", "topology", "regions")) {
            if (-not (Test-HasProperty $Manifest.memory $name)) { Add-Error "missing_memory_$name" "/memory/$name" "Required memory field is missing" }
        }
        if (Test-HasProperty $Manifest.memory "total_bytes") { Check-Fact $Manifest.memory.total_bytes "/memory/total_bytes" }
        if (Test-HasProperty $Manifest.memory "topology") { Check-Fact $Manifest.memory.topology "/memory/topology" }
        $observedRegions = New-Object System.Collections.Generic.List[object]
        if (Test-HasProperty $Manifest.memory "regions") {
            for ($i = 0; $i -lt @($Manifest.memory.regions).Count; $i++) {
                $region = @($Manifest.memory.regions)[$i]
                if ($region.status -eq "unknown") {
                    if ($null -ne $region.start_bytes -or $null -ne $region.length_bytes) { Add-Error "unknown_region_has_range" "/memory/regions/$i" "Unknown region range must be null" }
                    continue
                }
                if ($region.status -ne "observed" -or $null -eq $region.start_bytes -or $null -eq $region.length_bytes -or [int64]$region.start_bytes -lt 0 -or [int64]$region.length_bytes -le 0) {
                    Add-Error "invalid_memory_region" "/memory/regions/$i" "Observed region requires a non-negative start and positive length"
                    continue
                }
                $end = [int64]$region.start_bytes + [int64]$region.length_bytes
                if ($Manifest.memory.total_bytes.status -eq "observed" -and $end -gt [int64]$Manifest.memory.total_bytes.value) {
                    Add-Error "memory_region_out_of_range" "/memory/regions/$i" "Region exceeds observed total memory"
                }
                $observedRegions.Add([ordered]@{ index = $i; start = [int64]$region.start_bytes; end = $end })
            }
            for ($i = 0; $i -lt $observedRegions.Count; $i++) {
                for ($j = $i + 1; $j -lt $observedRegions.Count; $j++) {
                    $a = $observedRegions[$i]; $b = $observedRegions[$j]
                    if ($a.start -lt $b.end -and $b.start -lt $a.end) {
                        Add-Error "overlapping_memory_regions" "/memory/regions/$($b.index)" "Observed memory regions overlap"
                    }
                }
            }
        }
    }

    if (Test-HasProperty $Manifest "devices") {
        if (@($Manifest.devices).Count -eq 0) { Add-Error "missing_devices" "/devices" "At least one observed or explicitly unknown device is required" }
        $ids = @{}; $bdfs = @{}
        for ($i = 0; $i -lt @($Manifest.devices).Count; $i++) {
            $device = @($Manifest.devices)[$i]
            foreach ($name in @("id", "type", "identity", "bdf", "resources", "provenance")) {
                if (-not (Test-HasProperty $device $name)) { Add-Error "invalid_device_shape" "/devices/$i/$name" "Required device field is missing" }
            }
            $id = [string]$device.id
            if ($ids.ContainsKey($id)) { Add-Error "duplicate_device_id" "/devices/$i/id" "Device ids must be unique" } else { $ids[$id] = $true }
            if ($device.type -notin @("chipset", "storage", "network", "usb_controller", "usb_input", "display", "rng", "firmware", "other")) {
                Add-Error "invalid_device_type" "/devices/$i/type" "Device type is not in the v1 vocabulary"
            }
            if (Test-HasProperty $device "identity") { Check-Fact $device.identity "/devices/$i/identity" }
            if (Test-HasProperty $device "bdf") {
                Check-Fact $device.bdf "/devices/$i/bdf"
                if ($device.bdf.status -eq "observed") {
                    $bdf = [string]$device.bdf.value
                    if ($bdf -notmatch '^[0-9a-fA-F]{4}:[0-9a-fA-F]{2}:[0-1][0-9a-fA-F]\.[0-7]$') { Add-Error "invalid_device_bdf" "/devices/$i/bdf/value" "BDF must use dddd:bb:ss.f syntax" }
                    elseif ($bdfs.ContainsKey($bdf)) { Add-Error "duplicate_device_bdf" "/devices/$i/bdf/value" "Observed BDFs must be unique" } else { $bdfs[$bdf] = $true }
                }
            }
            if (Test-HasProperty $device "resources") {
                for ($j = 0; $j -lt @($device.resources).Count; $j++) {
                    $resource = @($device.resources)[$j]
                    if ($resource.kind -notin @("mmio", "io_port", "irq", "dma", "unknown") -or $resource.status -notin @("observed", "unknown") -or -not (Test-HasProperty $resource "provenance")) {
                        Add-Error "invalid_device_resource" "/devices/$i/resources/$j" "Resource requires typed kind, status, and provenance"
                    }
                    elseif ($resource.status -eq "unknown" -and ($null -ne $resource.start -or $null -ne $resource.length)) {
                        Add-Error "unknown_resource_has_range" "/devices/$i/resources/$j" "Unknown resource range must be null"
                    }
                    elseif ($resource.status -eq "observed" -and ($null -eq $resource.start -or $null -eq $resource.length -or [int64]$resource.start -lt 0 -or [int64]$resource.length -le 0)) {
                        Add-Error "invalid_device_resource_range" "/devices/$i/resources/$j" "Observed resource requires a valid range"
                    }
                }
            }
        }
    }

    if (-not $SkipHarnessBinding -and $machineId -eq "qemu-q35-shadow") {
        $smokeSource = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "vm-harness\shadow-vm-smoke.ps1")
        $supportSource = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "vm-harness\shadow-vm-smoke-support.ps1")
        $launchSource = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "scripts\run-stage0-qemu.ps1")
        if ($LaunchMemoryMB -le 0) {
            $match = [regex]::Match($smokeSource, '\[int\]\$GuestMemoryMB\s*=\s*(\d+)')
            if (-not $match.Success) { Add-Error "harness_memory_source_missing" "/memory/total_bytes" "Could not resolve GuestMemoryMB from harness" }
            else { $LaunchMemoryMB = [int]$match.Groups[1].Value }
        }
        if ($supportSource -notmatch 'memory\s*=\s*"\$\{MemoryMB\}M"' -or $supportSource -notmatch 'Get-Variable\s+-Name\s+GuestMemoryMB' -or
            $smokeSource -notmatch 'MemoryMB\s*=\s*\$GuestMemoryMB' -or $launchSource -notmatch '"-m",\s*"\$\{MemoryMB\}M"') {
            Add-Error "harness_memory_binding_missing" "/memory/total_bytes" "Profile, smoke args, and QEMU -m must share GuestMemoryMB"
        }
        if ($Manifest.memory.total_bytes.status -ne "observed" -or [int64]$Manifest.memory.total_bytes.value -ne ([int64]$LaunchMemoryMB * 1MB)) {
            Add-Error "qemu_profile_launch_drift" "/memory/total_bytes" "Manifest memory does not match QEMU launch memory ${LaunchMemoryMB} MiB"
        }
        if ($launchSource -notmatch '"-machine",\s*"q35"' -or $Manifest.machine.kind -ne "virtual") {
            Add-Error "qemu_machine_launch_drift" "/machine" "Manifest does not bind the q35 virtual launch"
        }
        if ($Manifest.cpu.model.value -ne "max" -or $smokeSource -notmatch 'Cpu\s*=\s*"max"') {
            Add-Error "qemu_cpu_launch_drift" "/cpu/model" "Manifest CPU model does not match harness launch"
        }
    }

    return [ordered]@{ valid = ($errors.Count -eq 0); machine_id = $machineId; errors = [object[]]$errors }
}

function Invoke-SelfTests {
    param([object]$Qemu, [object]$Surface)
    $cases = New-Object System.Collections.Generic.List[object]
    function Run-Case([string]$Name, [object]$Mutant, [string]$ExpectedCode, [int]$LaunchMemoryMB = 0) {
        $result = New-ValidationResult -Manifest $Mutant -LaunchMemoryMB $LaunchMemoryMB
        $codes = @($result.errors | ForEach-Object { $_.code })
        $cases.Add([ordered]@{ name = $Name; expected = "rejected:$ExpectedCode"; passed = (-not $result.valid -and $ExpectedCode -in $codes); actual_codes = $codes })
    }

    $m = Copy-JsonObject $Qemu; $m.schema_version = "raios.machine_manifest.v0"; Run-Case "schema-version" $m "schema_version_mismatch"
    $m = Copy-JsonObject $Qemu; $m.PSObject.Properties.Remove("cpu"); Run-Case "missing-cpu" $m "missing_cpu"
    $m = Copy-JsonObject $Qemu; $m.PSObject.Properties.Remove("memory"); Run-Case "missing-memory" $m "missing_memory"
    $m = Copy-JsonObject $Qemu; $m.devices = @(); Run-Case "missing-devices" $m "missing_devices"
    $m = Copy-JsonObject $Qemu; $r = Copy-JsonObject $m.memory.regions[0]; $r.id = "overlap"; $r.start_bytes = 1; $r.length_bytes = 16; $m.memory.regions = @($m.memory.regions) + @($r); Run-Case "overlapping-memory" $m "overlapping_memory_regions"
    $m = Copy-JsonObject $Qemu; $m.memory.regions[0].length_bytes = [int64]$m.memory.total_bytes.value + 1; Run-Case "out-of-range-memory" $m "memory_region_out_of_range"
    $m = Copy-JsonObject $Qemu; $d = Copy-JsonObject $m.devices[0]; $m.devices = @($m.devices) + @($d); Run-Case "duplicate-device-id" $m "duplicate_device_id"
    $m = Copy-JsonObject $Qemu; foreach ($i in @(0,1)) { $m.devices[$i].bdf.status = "observed"; $m.devices[$i].bdf.value = "0000:00:01.0" }; Run-Case "duplicate-device-bdf" $m "duplicate_device_bdf"
    $m = Copy-JsonObject $Qemu; $m.cpu.model.provenance.source_ref = ""; Run-Case "invalid-provenance" $m "invalid_provenance"
    $m = Copy-JsonObject $Qemu; Run-Case "qemu-512-vs-8192-drift" $m "qemu_profile_launch_drift" 8192
    $m = Copy-JsonObject $Surface; $m.cpu.architecture.provenance = Copy-JsonObject $Qemu.cpu.architecture.provenance; Run-Case "cross-machine-provenance" $m "cross_machine_provenance"
    $m = Copy-JsonObject $Surface; $m.memory.total_bytes.value = 536870912; Run-Case "unknown-cannot-carry-qemu-value" $m "unknown_fact_has_value"

    $originalQemu = New-ValidationResult -Manifest $Qemu
    $originalSurface = New-ValidationResult -Manifest $Surface
    return [ordered]@{
        original_manifests_green = ($originalQemu.valid -and $originalSurface.valid)
        mutation_count = $cases.Count
        mutation_passed_count = @($cases | Where-Object { $_.passed }).Count
        passed = ($originalQemu.valid -and $originalSurface.valid -and @($cases | Where-Object { -not $_.passed }).Count -eq 0)
        cases = [object[]]$cases
    }
}

try {
    $schema = Get-Content -Raw -LiteralPath $SchemaPath | ConvertFrom-Json
    if ($schema.properties.schema_version.const -ne "raios.machine_manifest.v1") { throw "Schema const does not bind v1" }
    $paths = if ($ManifestPath.Count -gt 0) { $ManifestPath } else { $DefaultManifestPaths }
    $results = @()
    $loaded = @{}
    foreach ($path in $paths) {
        $resolved = (Resolve-Path -LiteralPath $path).Path
        $manifest = Get-Content -Raw -LiteralPath $resolved | ConvertFrom-Json
        $loaded[[string]$manifest.machine.id] = $manifest
        $validation = New-ValidationResult -Manifest $manifest
        $results += [ordered]@{ path = $resolved; machine_id = $validation.machine_id; valid = $validation.valid; errors = $validation.errors }
    }
    $selfTestResult = $null
    if ($SelfTest) {
        if (-not $loaded.ContainsKey("qemu-q35-shadow") -or -not $loaded.ContainsKey("surface-pro-4")) { throw "SelfTest requires both canonical manifests" }
        $selfTestResult = Invoke-SelfTests -Qemu $loaded["qemu-q35-shadow"] -Surface $loaded["surface-pro-4"]
    }
    $valid = @($results | Where-Object { -not $_.valid }).Count -eq 0 -and ($null -eq $selfTestResult -or $selfTestResult.passed)
    [ordered]@{
        schema = "raios.hardware_manifest_validation.v1"
        generated_at_utc = [DateTime]::UtcNow.ToString("o")
        valid = $valid
        manifest_count = $results.Count
        results = $results
        self_test = $selfTestResult
    } | ConvertTo-Json -Depth 100
    if (-not $valid) { exit 1 }
}
catch {
    [ordered]@{ schema = "raios.hardware_manifest_validation.v1"; generated_at_utc = [DateTime]::UtcNow.ToString("o"); valid = $false; fatal = $_.Exception.Message; location = $_.InvocationInfo.PositionMessage } | ConvertTo-Json -Depth 10
    exit 1
}
