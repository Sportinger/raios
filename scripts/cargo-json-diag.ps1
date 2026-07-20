[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Package,

    [ValidateSet('build', 'check', 'test')]
    [string]$Command = 'check',

    [string]$Out,

    [string[]]$CargoArgs = @(),

    [string]$ManifestPath,

    [string]$CargoHome,

    [string]$TargetDir
)

Set-StrictMode -Version 2.0
$ErrorActionPreference = 'Stop'

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))

function Resolve-RepoPath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    if ([System.IO.Path]::IsPathRooted($Path)) {
        return [System.IO.Path]::GetFullPath($Path)
    }

    return [System.IO.Path]::GetFullPath((Join-Path $repoRoot $Path))
}

if ([string]::IsNullOrWhiteSpace($Out)) {
    $Out = Join-Path 'release\diagnostics' ($Package + '-latest.json')
}
$outPath = Resolve-RepoPath -Path $Out

if ([string]::IsNullOrWhiteSpace($CargoHome)) {
    $CargoHome = Join-Path $env:USERPROFILE '.cargo'
}
$cargoHomePath = Resolve-RepoPath -Path $CargoHome

if ([string]::IsNullOrWhiteSpace($TargetDir)) {
    $TargetDir = Join-Path $repoRoot 'target'
}
$targetDirPath = Resolve-RepoPath -Path $TargetDir

$cargoInvocation = @($Command, '-p', $Package, '--message-format=json')
if (-not [string]::IsNullOrWhiteSpace($ManifestPath)) {
    $cargoInvocation += @('--manifest-path', (Resolve-RepoPath -Path $ManifestPath))
}
$cargoInvocation += @($CargoArgs)

$hadCargoHome = Test-Path Env:CARGO_HOME
$previousCargoHome = $env:CARGO_HOME
$hadTargetDir = Test-Path Env:CARGO_TARGET_DIR
$previousTargetDir = $env:CARGO_TARGET_DIR

$cargoOutput = @()
$cargoExitCode = 1

try {
    $env:CARGO_HOME = $cargoHomePath
    $env:CARGO_TARGET_DIR = $targetDirPath

    Push-Location -LiteralPath $repoRoot
    try {
        $previousErrorActionPreference = $ErrorActionPreference
        try {
            # Windows PowerShell 5.1 wraps native stderr as error records. Cargo
            # writes progress there even on success, so it must not trigger Stop.
            $ErrorActionPreference = 'Continue'
            $cargoOutput = @(& cargo @cargoInvocation 2>$null)
            $cargoExitCode = [int]$LASTEXITCODE
        }
        finally {
            $ErrorActionPreference = $previousErrorActionPreference
        }
    }
    catch {
        $cargoOutput = @()
        $cargoExitCode = 1
    }
    finally {
        Pop-Location
    }
}
finally {
    if ($hadCargoHome) {
        $env:CARGO_HOME = $previousCargoHome
    }
    else {
        Remove-Item Env:CARGO_HOME -ErrorAction SilentlyContinue
    }

    if ($hadTargetDir) {
        $env:CARGO_TARGET_DIR = $previousTargetDir
    }
    else {
        Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    }
}

$diagnostics = @()
foreach ($line in $cargoOutput) {
    if ([string]::IsNullOrWhiteSpace($line)) {
        continue
    }

    try {
        $entry = $line | ConvertFrom-Json
    }
    catch {
        continue
    }

    if ($entry.reason -ne 'compiler-message' -or $null -eq $entry.message) {
        continue
    }

    $message = $entry.message
    $primarySpan = $message.spans |
        Where-Object { $_.is_primary } |
        Select-Object -First 1

    $code = $null
    if ($null -ne $message.code) {
        $code = $message.code.code
    }

    $file = $null
    $lineStart = $null
    $columnStart = $null
    if ($null -ne $primarySpan) {
        $file = $primarySpan.file_name
        $lineStart = $primarySpan.line_start
        $columnStart = $primarySpan.column_start
    }

    $diagnostics += [pscustomobject][ordered]@{
        level = $message.level
        message = $message.message
        code = $code
        file = $file
        line = $lineStart
        column = $columnStart
        rendered = $message.rendered
    }
}

$errors = @($diagnostics | Where-Object { $_.level -eq 'error' })
$warnings = @($diagnostics | Where-Object { $_.level -eq 'warning' })
$otherDiagnostics = @($diagnostics | Where-Object {
        $_.level -ne 'error' -and $_.level -ne 'warning'
    })
$orderedDiagnostics = @($errors) + @($warnings) + @($otherDiagnostics)

$scriptExitCode = 1
if ($cargoExitCode -eq 0) {
    $scriptExitCode = 0
}

$document = [pscustomobject][ordered]@{
    schema = 'raios.cargo_diag.v0'
    package = $Package
    command = $Command
    exit_code = $scriptExitCode
    error_count = $errors.Count
    warning_count = $warnings.Count
    diagnostics = @($orderedDiagnostics)
}

$outDirectory = Split-Path -Parent $outPath
if (-not (Test-Path -LiteralPath $outDirectory)) {
    New-Item -ItemType Directory -Path $outDirectory -Force | Out-Null
}

$json = $document | ConvertTo-Json -Depth 8
$json = $json.Replace("`r`n", "`n").TrimEnd("`r", "`n") + "`n"
$utf8WithoutBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText($outPath, $json, $utf8WithoutBom)

Write-Output ('CARGO_DIAG package={0} exit={1} errors={2} warnings={3} out={4}' -f `
        $Package, $scriptExitCode, $errors.Count, $warnings.Count, $outPath)
exit $scriptExitCode
