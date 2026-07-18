[CmdletBinding()]
param(
    [switch]$SelfTest
)

$ErrorActionPreference = "Stop"

function Get-DocsHygieneDisplayPath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RootPath,

        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $rootFullPath = [System.IO.Path]::GetFullPath($RootPath).TrimEnd('\', '/')
    $itemFullPath = [System.IO.Path]::GetFullPath($Path)
    if ($itemFullPath.Length -gt $rootFullPath.Length) {
        $relativePath = $itemFullPath.Substring($rootFullPath.Length).TrimStart('\', '/')
        return $relativePath.Replace('\', '/')
    }

    return "."
}

function Add-DocsHygieneViolation {
    param(
        [Parameter(Mandatory = $true)]
        [AllowEmptyCollection()]
        [System.Collections.Generic.List[object]]$Violations,

        [Parameter(Mandatory = $true)]
        [string]$Code,

        [Parameter(Mandatory = $true)]
        [string]$Path,

        [Parameter(Mandatory = $true)]
        [string]$Detail
    )

    [void]$Violations.Add([pscustomobject]@{
        Code = $Code
        Path = $Path
        Detail = $Detail
    })
}

function Invoke-DocsHygieneCheck {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$RootPath
    )

    $checkCount = 6
    $rootFullPath = [System.IO.Path]::GetFullPath($RootPath)
    $docsPath = Join-Path $rootFullPath "docs"
    $violations = New-Object "System.Collections.Generic.List[object]"
    $warnings = New-Object "System.Collections.Generic.List[string]"

    # Rule 1: only the fixed set of files and directories may live in docs/.
    if (-not (Test-Path -LiteralPath $docsPath -PathType Container)) {
        Add-DocsHygieneViolation -Violations $violations -Code "docs_root_missing" -Path "docs" -Detail "required_directory_missing"
    }
    else {
        $allowedFiles = @("SCOPE.md", "README.md")
        $allowedDirectories = @("scope", "architecture", "agents", "plans", "status", "assets", "_archive")
        foreach ($entry in @(Get-ChildItem -LiteralPath $docsPath -Force)) {
            $isAllowed = $false
            if ($entry -is [System.IO.FileInfo]) {
                $isAllowed = $allowedFiles -ccontains $entry.Name
            }
            elseif ($entry -is [System.IO.DirectoryInfo]) {
                $isAllowed = $allowedDirectories -ccontains $entry.Name
            }

            if (-not $isAllowed) {
                $displayPath = Get-DocsHygieneDisplayPath -RootPath $rootFullPath -Path $entry.FullName
                $entryType = "other"
                if ($entry -is [System.IO.FileInfo]) {
                    $entryType = "file"
                }
                elseif ($entry -is [System.IO.DirectoryInfo]) {
                    $entryType = "directory"
                }
                Add-DocsHygieneViolation -Violations $violations -Code "docs_root_entry" -Path $displayPath -Detail ("unexpected_" + $entryType)
            }
        }
    }

    # Rule 2: HANDOFF.md is required, warns above 2560 bytes, and fails above 4096 bytes.
    $handoffPath = Join-Path $docsPath "status\HANDOFF.md"
    if (-not (Test-Path -LiteralPath $handoffPath -PathType Leaf)) {
        Add-DocsHygieneViolation -Violations $violations -Code "handoff_missing" -Path "docs/status/HANDOFF.md" -Detail "required_file_missing"
    }
    else {
        $handoffLength = (Get-Item -LiteralPath $handoffPath -Force).Length
        if ($handoffLength -gt 4096) {
            Add-DocsHygieneViolation -Violations $violations -Code "handoff_too_large" -Path "docs/status/HANDOFF.md" -Detail ("size_bytes=" + $handoffLength + " limit_bytes=4096")
        }
        elseif ($handoffLength -gt 2560) {
            [void]$warnings.Add("DOCS_HYGIENE warning=handoff_size path=docs/status/HANDOFF.md detail=size_bytes=$handoffLength warning_bytes=2560 limit_bytes=4096")
        }
    }

    # Rule 3: STATUS.md is optional, but limited to 30720 bytes when present.
    $statusPath = Join-Path $docsPath "status\STATUS.md"
    if (Test-Path -LiteralPath $statusPath -PathType Leaf) {
        $statusLength = (Get-Item -LiteralPath $statusPath -Force).Length
        if ($statusLength -gt 30720) {
            Add-DocsHygieneViolation -Violations $violations -Code "status_too_large" -Path "docs/status/STATUS.md" -Detail ("size_bytes=" + $statusLength + " limit_bytes=30720")
        }
    }

    # Rule 4: scope/ contains one markdown file for each prefix 01 through 07 and nothing else.
    $scopePath = Join-Path $docsPath "scope"
    if (-not (Test-Path -LiteralPath $scopePath -PathType Container)) {
        Add-DocsHygieneViolation -Violations $violations -Code "scope_directory_missing" -Path "docs/scope" -Detail "required_directory_missing"
    }
    else {
        $prefixCounts = @{}
        foreach ($number in 1..7) {
            $prefixCounts[('{0:D2}' -f $number)] = 0
        }

        foreach ($entry in @(Get-ChildItem -LiteralPath $scopePath -Force)) {
            $displayPath = Get-DocsHygieneDisplayPath -RootPath $rootFullPath -Path $entry.FullName
            if ($entry -isnot [System.IO.FileInfo]) {
                Add-DocsHygieneViolation -Violations $violations -Code "scope_unexpected_entry" -Path $displayPath -Detail "expected_file"
                continue
            }

            if ($entry.Name -cmatch '^(0[1-7])-.*\.md$') {
                $prefixCounts[$Matches[1]] = $prefixCounts[$Matches[1]] + 1
            }
            else {
                Add-DocsHygieneViolation -Violations $violations -Code "scope_unexpected_file" -Path $displayPath -Detail "expected_name=01-*.md_through_07-*.md"
            }
        }

        foreach ($number in 1..7) {
            $prefix = '{0:D2}' -f $number
            $actualCount = $prefixCounts[$prefix]
            if ($actualCount -ne 1) {
                Add-DocsHygieneViolation -Violations $violations -Code "scope_prefix_count" -Path "docs/scope" -Detail ("prefix=" + $prefix + " expected=1 actual=" + $actualCount)
            }
        }
    }

    # Rule 5: every file directly in plans/ follows plan-*.md.
    $plansPath = Join-Path $docsPath "plans"
    if (Test-Path -LiteralPath $plansPath -PathType Container) {
        foreach ($entry in @(Get-ChildItem -LiteralPath $plansPath -Force)) {
            if (($entry -is [System.IO.FileInfo]) -and ($entry.Name -cnotmatch '^plan-.*\.md$')) {
                $displayPath = Get-DocsHygieneDisplayPath -RootPath $rootFullPath -Path $entry.FullName
                Add-DocsHygieneViolation -Violations $violations -Code "plan_filename" -Path $displayPath -Detail "expected_name=plan-*.md"
            }
        }
    }

    # Rule 6: ADR files are numbered with four digits; invariant-choices.md is the sole exception.
    $decisionsPath = Join-Path $docsPath "architecture\decisions"
    if (Test-Path -LiteralPath $decisionsPath -PathType Container) {
        foreach ($entry in @(Get-ChildItem -LiteralPath $decisionsPath -Force)) {
            if ($entry -is [System.IO.FileInfo]) {
                $isNumberedDecision = $entry.Name -cmatch '^\d{4}-.*\.md$'
                $isInvariantChoices = $entry.Name -ceq "invariant-choices.md"
                if ((-not $isNumberedDecision) -and (-not $isInvariantChoices)) {
                    $displayPath = Get-DocsHygieneDisplayPath -RootPath $rootFullPath -Path $entry.FullName
                    Add-DocsHygieneViolation -Violations $violations -Code "decision_filename" -Path $displayPath -Detail "expected_name=NNNN-*.md_or_invariant-choices.md"
                }
            }
        }
    }

    $result = "green"
    if ($violations.Count -gt 0) {
        $result = "red"
    }

    return [pscustomobject]@{
        Result = $result
        CheckCount = $checkCount
        Violations = $violations
        Warnings = $warnings
    }
}

function Write-DocsHygieneResult {
    param(
        [Parameter(Mandatory = $true)]
        [object]$CheckResult
    )

    foreach ($warning in $CheckResult.Warnings) {
        Write-Output $warning
    }
    foreach ($violation in $CheckResult.Violations) {
        Write-Output ("DOCS_HYGIENE violation=" + $violation.Code + " path=" + $violation.Path + " detail=" + $violation.Detail)
    }
    Write-Output ("DOCS_HYGIENE result=" + $CheckResult.Result + " checks=" + $CheckResult.CheckCount + " violations=" + $CheckResult.Violations.Count)
}

if ($SelfTest) {
    $tempBasePath = [System.IO.Path]::GetFullPath($env:TEMP)
    $fixtureRoot = Join-Path $tempBasePath ("raios-docs-hygiene-" + [System.Guid]::NewGuid().ToString("N"))
    try {
        $fixtureDocsPath = Join-Path $fixtureRoot "docs"
        foreach ($directory in @("scope", "architecture\decisions", "agents", "plans", "status", "assets", "_archive")) {
            [void](New-Item -ItemType Directory -Path (Join-Path $fixtureDocsPath $directory) -Force)
        }

        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "SCOPE.md") -Value "self-test" -Encoding utf8
        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "README.md") -Value "self-test" -Encoding utf8
        foreach ($number in 1..7) {
            $scopeFileName = ('{0:D2}-self-test.md' -f $number)
            Set-Content -LiteralPath (Join-Path $fixtureDocsPath ("scope\" + $scopeFileName)) -Value "self-test" -Encoding utf8
        }

        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "foreign.txt") -Value "planted violation" -Encoding utf8
        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "status\HANDOFF.md") -Value ("x" * 5000) -Encoding utf8
        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "plans\wrong-name.md") -Value "planted violation" -Encoding utf8

        $selfTestResult = Invoke-DocsHygieneCheck -RootPath $fixtureRoot
        $detectedCodes = @($selfTestResult.Violations | ForEach-Object { $_.Code })
        $requiredCodes = @("docs_root_entry", "handoff_too_large", "plan_filename")
        $missingCodes = @($requiredCodes | Where-Object { $detectedCodes -cnotcontains $_ })

        if ($missingCodes.Count -eq 0) {
            Write-Output "DOCS_HYGIENE selftest=green planted=3 detected=3"
            Write-Output ("DOCS_HYGIENE result=green checks=" + $selfTestResult.CheckCount + " violations=0")
            exit 0
        }

        Write-Output ("DOCS_HYGIENE violation=selftest_detection path=temporary_fixture detail=missing_codes=" + ($missingCodes -join ','))
        Write-Output ("DOCS_HYGIENE result=red checks=" + $selfTestResult.CheckCount + " violations=1")
        exit 1
    }
    finally {
        if (Test-Path -LiteralPath $fixtureRoot) {
            Remove-Item -LiteralPath $fixtureRoot -Recurse -Force
        }
    }
}

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$repositoryResult = Invoke-DocsHygieneCheck -RootPath $repositoryRoot
Write-DocsHygieneResult -CheckResult $repositoryResult
if ($repositoryResult.Violations.Count -gt 0) {
    exit 1
}
exit 0
