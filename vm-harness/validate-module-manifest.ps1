param(
    [Parameter(Mandatory = $true)]
    [string]$ManifestPath,
    [string]$ArtifactPath = "",
    [switch]$AllowInvalid
)

$ErrorActionPreference = "Stop"

function Get-FileSha256 {
    param([string]$Path)
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-PropertyValue {
    param(
        [object]$Object,
        [string]$Name
    )
    if ($Object.PSObject.Properties.Name -contains $Name) {
        return $Object.$Name
    }
    return $null
}

function Add-Issue {
    param(
        [System.Collections.Generic.List[string]]$List,
        [string]$Message
    )
    $List.Add($Message) | Out-Null
}

function Test-StringField {
    param(
        [object]$Manifest,
        [string]$Name,
        [System.Collections.Generic.List[string]]$Errors
    )
    $value = Get-PropertyValue -Object $Manifest -Name $Name
    if ($null -eq $value -or -not ($value -is [string]) -or -not $value.Trim()) {
        Add-Issue -List $Errors -Message "$Name must be a non-empty string"
        return $null
    }
    return $value
}

function Test-StringArrayField {
    param(
        [object]$Manifest,
        [string]$Name,
        [System.Collections.Generic.List[string]]$Errors,
        [System.Collections.Generic.List[string]]$Warnings
    )
    $value = Get-PropertyValue -Object $Manifest -Name $Name
    if ($null -eq $value) {
        if ($script:ManifestRaw -match ('"' + [regex]::Escape($Name) + '"\s*:\s*\[\s*\]')) {
            return @()
        }
        Add-Issue -List $Errors -Message "$Name must be an array"
        return @()
    }
    if ($value -is [string]) {
        if ($script:ManifestRaw -match ('"' + [regex]::Escape($Name) + '"\s*:\s*\[')) {
            Add-Issue -List $Warnings -Message "$Name parsed as scalar by PowerShell but raw JSON contains an array; accepted as one item"
            return @($value)
        }
        Add-Issue -List $Errors -Message "$Name must be an array, not a string"
        return @()
    }

    $items = @($value)
    foreach ($item in $items) {
        if (-not ($item -is [string]) -or -not $item.Trim()) {
            Add-Issue -List $Errors -Message "$Name entries must be non-empty strings"
            break
        }
    }
    return $items
}

function Normalize-HashRef {
    param([string]$Value)
    if (-not $Value) {
        return $null
    }
    $trimmed = $Value.Trim().ToLowerInvariant()
    if ($trimmed.StartsWith("sha256:")) {
        $trimmed = $trimmed.Substring(7)
    }
    return $trimmed
}

function Test-HashField {
    param(
        [object]$Manifest,
        [string]$Name,
        [bool]$Required,
        [System.Collections.Generic.List[string]]$Errors
    )
    $value = Get-PropertyValue -Object $Manifest -Name $Name
    if ($null -eq $value -or $value -eq "") {
        if ($Required) {
            Add-Issue -List $Errors -Message "$Name must be a sha256 hash reference"
        }
        return $null
    }
    if (-not ($value -is [string])) {
        Add-Issue -List $Errors -Message "$Name must be a string sha256 hash reference"
        return $null
    }
    $hash = Normalize-HashRef -Value $value
    if ($hash -notmatch '^[0-9a-f]{64}$') {
        Add-Issue -List $Errors -Message "$Name must be sha256:<64 hex chars> or 64 hex chars"
        return $null
    }
    return $hash
}

function Test-Enum {
    param(
        [string]$Name,
        [string]$Value,
        [string[]]$Allowed,
        [System.Collections.Generic.List[string]]$Errors
    )
    if ($Allowed -notcontains $Value) {
        Add-Issue -List $Errors -Message "$Name must be one of: $($Allowed -join ', ')"
    }
}

$resolvedManifest = (Resolve-Path -LiteralPath $ManifestPath).Path
$resolvedArtifact = $null
if ($ArtifactPath) {
    $resolvedArtifact = (Resolve-Path -LiteralPath $ArtifactPath).Path
}

$errors = [System.Collections.Generic.List[string]]::new()
$warnings = [System.Collections.Generic.List[string]]::new()
$script:ManifestRaw = ""
$manifestHash = Get-FileSha256 -Path $resolvedManifest
$artifactHash = if ($resolvedArtifact) { Get-FileSha256 -Path $resolvedArtifact } else { $null }

try {
    $script:ManifestRaw = Get-Content -Raw -LiteralPath $resolvedManifest
    $manifest = $script:ManifestRaw | ConvertFrom-Json
}
catch {
    Add-Issue -List $errors -Message "manifest must be valid JSON: $($_.Exception.Message)"
    $manifest = [pscustomobject]@{}
}

$schema = Test-StringField -Manifest $manifest -Name "schema" -Errors $errors
if ($schema -and $schema -ne "seedos.module_manifest.v0") {
    Add-Issue -List $errors -Message "schema must be seedos.module_manifest.v0"
}

$name = Test-StringField -Manifest $manifest -Name "name" -Errors $errors
if ($name -and $name -notmatch '^[a-z0-9][a-z0-9._-]{1,62}$') {
    Add-Issue -List $errors -Message "name must be 2-63 chars: lowercase letters, digits, dot, underscore, hyphen"
}

$version = Test-StringField -Manifest $manifest -Name "version" -Errors $errors
$kind = Test-StringField -Manifest $manifest -Name "kind" -Errors $errors
$target = Test-StringField -Manifest $manifest -Name "target" -Errors $errors
$abi = Test-StringField -Manifest $manifest -Name "abi" -Errors $errors
$builtBy = Test-StringField -Manifest $manifest -Name "built_by" -Errors $errors
$risk = Test-StringField -Manifest $manifest -Name "risk" -Errors $errors
$loadMode = Test-StringField -Manifest $manifest -Name "load_mode" -Errors $errors

if ($kind) {
    Test-Enum -Name "kind" -Value $kind -Allowed @("workstation_capability", "guest_diagnostic", "service", "driver", "ui_tool") -Errors $errors
}
if ($target -and $target -ne "seedos-stage0") {
    Add-Issue -List $errors -Message "target must be seedos-stage0"
}
if ($abi) {
    Test-Enum -Name "abi" -Value $abi -Allowed @("none", "seedos-agent-v0", "seedos-service-v0", "seedos-driver-v0") -Errors $errors
}
if ($risk) {
    Test-Enum -Name "risk" -Value $risk -Allowed @("observe", "diagnose", "simulate", "modify_ram", "persist", "hardware") -Errors $errors
}
if ($loadMode) {
    Test-Enum -Name "load_mode" -Value $loadMode -Allowed @("proposal_only", "vm_test_only", "ram_only", "persistent") -Errors $errors
}

$provides = Test-StringArrayField -Manifest $manifest -Name "provides" -Errors $errors -Warnings $warnings
$requestedCaps = Test-StringArrayField -Manifest $manifest -Name "requested_caps" -Errors $errors -Warnings $warnings
$grantedCaps = Test-StringArrayField -Manifest $manifest -Name "granted_caps" -Errors $errors -Warnings $warnings
$tests = Test-StringArrayField -Manifest $manifest -Name "tests" -Errors $errors -Warnings $warnings

if (@($grantedCaps).Count -ne 0) {
    Add-Issue -List $errors -Message "granted_caps must be empty; grants are computed by local policy"
}

$artifactHashDeclared = Test-HashField -Manifest $manifest -Name "artifact_hash" -Required $true -Errors $errors
$baseImageHashDeclared = Test-HashField -Manifest $manifest -Name "base_image_hash" -Required $false -Errors $errors
$testReportHashDeclared = Test-HashField -Manifest $manifest -Name "test_report_hash" -Required $false -Errors $errors
$manifestHashDeclared = Test-HashField -Manifest $manifest -Name "manifest_hash" -Required $false -Errors $errors

if ($manifestHashDeclared) {
    Add-Issue -List $warnings -Message "manifest_hash is externally computed; embedded values are informational only"
    if ($manifestHashDeclared -ne $manifestHash) {
        Add-Issue -List $errors -Message "manifest_hash does not match this file"
    }
}

if ($artifactHashDeclared -and $artifactHash -and $artifactHashDeclared -ne $artifactHash) {
    Add-Issue -List $errors -Message "artifact_hash does not match ArtifactPath"
}
if ($artifactHashDeclared -and -not $artifactHash) {
    Add-Issue -List $warnings -Message "artifact_hash declared but ArtifactPath was not provided for byte-level verification"
}

$rollbackId = Get-PropertyValue -Object $manifest -Name "rollback_id"
if ($null -ne $rollbackId -and -not ($rollbackId -is [string])) {
    Add-Issue -List $errors -Message "rollback_id must be null or a string"
}

$valid = $errors.Count -eq 0
$result = [ordered]@{
    schema = "seedos.module_manifest_validation.v0"
    valid = $valid
    manifest = [ordered]@{
        path = $resolvedManifest
        sha256 = $manifestHash
        declared_sha256 = $manifestHashDeclared
    }
    artifact = [ordered]@{
        path = $resolvedArtifact
        sha256 = $artifactHash
        declared_sha256 = $artifactHashDeclared
    }
    declared = [ordered]@{
        name = $name
        version = $version
        kind = $kind
        target = $target
        abi = $abi
        built_by = $builtBy
        risk = $risk
        load_mode = $loadMode
        provides = @($provides)
        requested_caps = @($requestedCaps)
        granted_caps = @($grantedCaps)
        tests = @($tests)
        base_image_sha256 = $baseImageHashDeclared
        test_report_sha256 = $testReportHashDeclared
        rollback_id = $rollbackId
    }
    errors = @($errors.ToArray())
    warnings = @($warnings.ToArray())
}

$result | ConvertTo-Json -Depth 12

if (-not $valid -and -not $AllowInvalid) {
    throw "Module manifest validation failed: $($errors -join '; ')"
}
