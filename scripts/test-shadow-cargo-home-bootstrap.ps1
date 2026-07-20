$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$HarnessPath = Join-Path $RepoRoot "vm-harness\shadow-vm-smoke.ps1"
$TempRoot = Join-Path $env:TEMP ("raios-cargo-home-bootstrap-{0}" -f [Guid]::NewGuid().ToString("N"))
$tempPrefix = [IO.Path]::GetFullPath($env:TEMP).TrimEnd('\') + '\'
$tempFullPath = [IO.Path]::GetFullPath($TempRoot)
if (-not $tempFullPath.StartsWith($tempPrefix, [StringComparison]::OrdinalIgnoreCase)) {
    throw "fixture temp root escaped host temp directory"
}

function Require {
    param([bool]$Condition, [string]$Message)
    if (-not $Condition) {
        throw $Message
    }
}

function Require-Failure {
    param([scriptblock]$Action, [string]$ExpectedMessage)
    try {
        & $Action
        throw "expected failure was not raised: $ExpectedMessage"
    }
    catch {
        if ($_.Exception.Message -cne $ExpectedMessage) {
            throw
        }
    }
}

$tokens = $null
$parseErrors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile(
    $HarnessPath,
    [ref]$tokens,
    [ref]$parseErrors
)
Require ($parseErrors.Count -eq 0) "shadow-vm-smoke.ps1 has PowerShell parse errors"
$functionAst = $ast.Find(
    {
        param($node)
        $node -is [Management.Automation.Language.FunctionDefinitionAst] -and
            $node.Name -ceq "Resolve-ShadowCargoHome"
    },
    $true
)
Require ($null -ne $functionAst) "Resolve-ShadowCargoHome function is missing"
. ([scriptblock]::Create($functionAst.Extent.Text))

try {
    New-Item -ItemType Directory -Path $TempRoot | Out-Null

    $freshRepo = Join-Path $TempRoot "fresh-repo"
    New-Item -ItemType Directory -Path $freshRepo | Out-Null
    $freshResolved = Resolve-ShadowCargoHome -RepositoryRoot $freshRepo
    $expectedFresh = [IO.Path]::GetFullPath((Join-Path $freshRepo ".cargo-home"))
    Require ([string]::Equals($freshResolved, $expectedFresh, [StringComparison]::OrdinalIgnoreCase)) `
        "fresh checkout resolved the wrong Cargo home"
    Require (Test-Path -LiteralPath $freshResolved -PathType Container) `
        "fresh checkout did not create the repo-local Cargo home"

    $repoSentinel = Join-Path $freshResolved "preserve.txt"
    Set-Content -LiteralPath $repoSentinel -Value "keep" -NoNewline
    $freshAgain = Resolve-ShadowCargoHome -RepositoryRoot $freshRepo
    Require ([string]::Equals($freshAgain, $freshResolved, [StringComparison]::OrdinalIgnoreCase)) `
        "existing repo-local Cargo home was not idempotent"
    Require ((Get-Content -LiteralPath $repoSentinel -Raw) -ceq "keep") `
        "existing repo-local Cargo home was modified"

    $customHome = Join-Path $TempRoot "custom-home"
    New-Item -ItemType Directory -Path $customHome | Out-Null
    $customSentinel = Join-Path $customHome "preserve.txt"
    Set-Content -LiteralPath $customSentinel -Value "custom" -NoNewline
    $customRepo = Join-Path $TempRoot "custom-repo"
    New-Item -ItemType Directory -Path $customRepo | Out-Null
    $customResolved = Resolve-ShadowCargoHome `
        -RepositoryRoot $customRepo `
        -ConfiguredCargoHome $customHome
    Require ([string]::Equals($customResolved, $customHome, [StringComparison]::OrdinalIgnoreCase)) `
        "custom absolute Cargo home was not preserved"
    Require ((Get-Content -LiteralPath $customSentinel -Raw) -ceq "custom") `
        "custom Cargo home was modified"
    Require (-not (Test-Path -LiteralPath (Join-Path $customRepo ".cargo-home"))) `
        "custom Cargo home unexpectedly created a repo-local directory"

    $newCustomHome = Join-Path $TempRoot "new-custom-home"
    $newCustomResolved = Resolve-ShadowCargoHome `
        -RepositoryRoot $customRepo `
        -ConfiguredCargoHome $newCustomHome
    Require (Test-Path -LiteralPath $newCustomResolved -PathType Container) `
        "missing custom absolute Cargo home was not created"

    $fileRepo = Join-Path $TempRoot "file-repo"
    New-Item -ItemType Directory -Path $fileRepo | Out-Null
    $defaultFile = Join-Path $fileRepo ".cargo-home"
    Set-Content -LiteralPath $defaultFile -Value "not-a-directory" -NoNewline
    Require-Failure `
        -Action { Resolve-ShadowCargoHome -RepositoryRoot $fileRepo } `
        -ExpectedMessage "CARGO_HOME path exists but is not a directory: $defaultFile"
    Require ((Get-Content -LiteralPath $defaultFile -Raw) -ceq "not-a-directory") `
        "file at default Cargo home path was modified"

    Require-Failure `
        -Action {
            Resolve-ShadowCargoHome `
                -RepositoryRoot $customRepo `
                -ConfiguredCargoHome "relative\cargo-home"
        } `
        -ExpectedMessage "Configured CARGO_HOME must be an absolute path"

    $customFile = Join-Path $TempRoot "custom-file"
    Set-Content -LiteralPath $customFile -Value "not-a-directory" -NoNewline
    Require-Failure `
        -Action {
            Resolve-ShadowCargoHome `
                -RepositoryRoot $customRepo `
                -ConfiguredCargoHome $customFile
        } `
        -ExpectedMessage "CARGO_HOME path exists but is not a directory: $customFile"

    $blockedParent = Join-Path $TempRoot "blocked-parent"
    Set-Content -LiteralPath $blockedParent -Value "file-parent" -NoNewline
    $blockedChild = Join-Path $blockedParent "cargo-home"
    Require-Failure `
        -Action {
            Resolve-ShadowCargoHome `
                -RepositoryRoot $customRepo `
                -ConfiguredCargoHome $blockedChild
        } `
        -ExpectedMessage "Failed to create CARGO_HOME directory: $blockedChild"

    Write-Output "SHADOW_CARGO_HOME_BOOTSTRAP status=pass fresh_created=true existing_preserved=true custom_preserved=true negatives=4"
}
finally {
    if (Test-Path -LiteralPath $TempRoot) {
        Remove-Item -LiteralPath $TempRoot -Recurse -Force
    }
}
