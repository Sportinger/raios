param(
    [Parameter(Mandatory = $true)]
    [string]$ManifestPath,
    [Parameter(Mandatory = $true)]
    [string]$ArtifactPath,
    [Parameter(Mandatory = $true)]
    [string]$VmReportPath,
    [Parameter(Mandatory = $true)]
    [string]$Approval,
    [ValidateSet("ram_only", "proposal_only")]
    [string]$LoadMode = "ram_only",
    [string]$OutputDir = ""
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$ValidateManifestScript = Join-Path $PSScriptRoot "validate-module-manifest.ps1"
if (-not $OutputDir) {
    $OutputDir = Join-Path $RepoRoot "release\attestations"
}

function Get-FileSha256 {
    param([string]$Path)
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-TextSha256 {
    param([string]$Text)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $bytes = [System.Text.Encoding]::UTF8.GetBytes($Text)
        $hash = $sha.ComputeHash($bytes)
        return ([BitConverter]::ToString($hash) -replace "-", "").ToLowerInvariant()
    }
    finally {
        $sha.Dispose()
    }
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

$resolvedManifest = (Resolve-Path -LiteralPath $ManifestPath).Path
$resolvedArtifact = (Resolve-Path -LiteralPath $ArtifactPath).Path
$resolvedReport = (Resolve-Path -LiteralPath $VmReportPath).Path

$validationJson = & $ValidateManifestScript -ManifestPath $resolvedManifest -ArtifactPath $resolvedArtifact
$validation = ($validationJson -join [Environment]::NewLine) | ConvertFrom-Json
if (-not $validation.valid) {
    throw "Manifest validation failed"
}

$report = Get-Content -Raw -LiteralPath $resolvedReport | ConvertFrom-Json
if ($report.schema -ne "seedos.vm_test_report.v0") {
    throw "VM report schema must be seedos.vm_test_report.v0"
}
if ($report.result -ne "passed") {
    throw "VM report result must be passed"
}

$manifestHash = Get-FileSha256 -Path $resolvedManifest
$artifactHash = Get-FileSha256 -Path $resolvedArtifact
$reportHash = Get-FileSha256 -Path $resolvedReport
$reportManifestHash = Normalize-HashRef -Value $report.candidate_manifest.sha256
$reportArtifactHash = Normalize-HashRef -Value $report.candidate_artifact.sha256
$reportBaseImageHash = Normalize-HashRef -Value $report.base_image.sha256
$reportQemuArgsHash = Normalize-HashRef -Value $report.qemu.args_sha256

if (-not $reportManifestHash) {
    throw "VM report is not bound to a candidate manifest"
}
if (-not $reportArtifactHash) {
    throw "VM report is not bound to a candidate artifact"
}
if ($reportManifestHash -ne $manifestHash) {
    throw "VM report manifest hash does not match ManifestPath"
}
if ($reportArtifactHash -ne $artifactHash) {
    throw "VM report artifact hash does not match ArtifactPath"
}

$expectedApproval = "APPROVE $($LoadMode.ToUpperInvariant()) $($artifactHash.Substring(0, 12))"
if ($Approval -cne $expectedApproval) {
    throw "Approval must be exactly: $expectedApproval"
}

$attestationId = "attest-{0:yyyyMMdd-HHmmss}-{1}" -f (Get-Date), $PID
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$outputPath = Join-Path $OutputDir "$attestationId.json"
$outputHashPath = "$outputPath.sha256"
$rollbackPlan = if ($LoadMode -eq "ram_only") { "drop_on_reboot_or_kill_service" } else { "proposal_record_only" }

$attestation = [ordered]@{
    schema = "seedos.local_attestation.v0"
    attestation_id = $attestationId
    created_at_utc = ([DateTime]::UtcNow.ToString("o"))
    result = "evidence_recorded_load_still_denied_in_stage0"
    load_mode = $LoadMode
    manifest = [ordered]@{
        path = $resolvedManifest
        sha256 = $manifestHash
        name = $validation.declared.name
        version = $validation.declared.version
        kind = $validation.declared.kind
        risk = $validation.declared.risk
        requested_caps = @($validation.declared.requested_caps)
    }
    artifact = [ordered]@{
        path = $resolvedArtifact
        sha256 = $artifactHash
    }
    vm_report = [ordered]@{
        path = $resolvedReport
        sha256 = $reportHash
        run_id = $report.run_id
        base_image_sha256 = $reportBaseImageHash
        qemu_args_sha256 = $reportQemuArgsHash
        result = $report.result
    }
    approval = [ordered]@{
        source = "local_user_cli"
        phrase_sha256 = (Get-TextSha256 -Text $Approval)
        expected_phrase = $expectedApproval
        recorded_at_utc = ([DateTime]::UtcNow.ToString("o"))
    }
    evidence_binding = [ordered]@{
        manifest_sha256 = $manifestHash
        artifact_sha256 = $artifactHash
        vm_report_sha256 = $reportHash
        base_image_sha256 = $reportBaseImageHash
        qemu_args_sha256 = $reportQemuArgsHash
    }
    limits = [ordered]@{
        grants_load_now = $false
        requires_guest_loader = $true
        requires_kernel_policy_check = $true
        rollback_plan = $rollbackPlan
    }
}

$json = $attestation | ConvertTo-Json -Depth 12
Set-Content -LiteralPath $outputPath -Value $json -Encoding UTF8
$attestationHash = Get-FileSha256 -Path $outputPath
Set-Content -LiteralPath $outputHashPath -Value "$attestationHash  $outputPath" -Encoding ASCII

Write-Host "local attestation written: $outputPath"
Write-Host "local attestation sha256: $outputHashPath"
