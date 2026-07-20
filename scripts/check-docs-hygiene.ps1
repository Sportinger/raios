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

    $checkCount = 12
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

    # Rule 7: README.md names SCOPE.md as the conflict-resolving source for what raiOS is.
    $readmePath = Join-Path $docsPath "README.md"
    $singleSourcePhrase = "conflicts resolve in favor of SCOPE.md"
    if (-not (Test-Path -LiteralPath $readmePath -PathType Leaf)) {
        Add-DocsHygieneViolation -Violations $violations -Code "single_source" -Path "docs/README.md" -Detail "required_file_missing"
    }
    else {
        $readmeContent = Get-Content -LiteralPath $readmePath -Raw
        if ($readmeContent.IndexOf($singleSourcePhrase, [System.StringComparison]::Ordinal) -lt 0) {
            Add-DocsHygieneViolation -Violations $violations -Code "single_source" -Path "docs/README.md" -Detail "required_phrase_missing"
        }
    }

    # Rule 8: AGENTS.md is the single live root instruction source. It may
    # reference only existing, non-glob docs paths; legacy agent-control
    # surfaces fail closed (ADR 0025).
    $instructionRelativePaths = @("AGENTS.md")
    foreach ($instructionRelativePath in $instructionRelativePaths) {
        $instructionPath = Join-Path $rootFullPath $instructionRelativePath
        $instructionDisplayPath = $instructionRelativePath.Replace('\', '/')
        if (-not (Test-Path -LiteralPath $instructionPath -PathType Leaf)) {
            Add-DocsHygieneViolation -Violations $violations -Code "root_instruction_missing" -Path $instructionDisplayPath -Detail "required_file_missing"
            continue
        }

        $instructionContent = Get-Content -LiteralPath $instructionPath -Raw
        $docsReferences = [System.Text.RegularExpressions.Regex]::Matches($instructionContent, '(?<![A-Za-z0-9_./-])docs/[A-Za-z0-9_./*<>?-]+')
        foreach ($docsReferenceMatch in $docsReferences) {
            $docsReference = $docsReferenceMatch.Value.TrimEnd('.', ',', ':', ';')
            if (($docsReference.IndexOf('*') -ge 0) -or ($docsReference.IndexOf('<') -ge 0) -or ($docsReference.IndexOf('?') -ge 0)) {
                continue
            }

            $referencedPath = Join-Path $rootFullPath $docsReference.Replace('/', '\')
            if (-not (Test-Path -LiteralPath $referencedPath)) {
                Add-DocsHygieneViolation -Violations $violations -Code "root_instruction_path" -Path $instructionDisplayPath -Detail ("missing_reference=" + $docsReference)
            }
        }
    }

    foreach ($legacyInstructionRelativePath in @("CLAUDE.md", ".claude")) {
        $legacyInstructionPath = Join-Path $rootFullPath $legacyInstructionRelativePath
        if (Test-Path -LiteralPath $legacyInstructionPath) {
            Add-DocsHygieneViolation -Violations $violations -Code "legacy_agent_instruction" -Path $legacyInstructionRelativePath -Detail "AGENTS.md_is_single_control_plane"
        }
    }

    # Rule 9: each plan maps by suffix to exactly one scope category, with no duplicates.
    $scopeCategoryFiles = @()
    if (Test-Path -LiteralPath $scopePath -PathType Container) {
        $scopeCategoryFiles = @(Get-ChildItem -LiteralPath $scopePath -File | Where-Object { $_.Name -cmatch '^0[1-7]-.*\.md$' })
    }

    $planCategoryCounts = @{}
    if (Test-Path -LiteralPath $plansPath -PathType Container) {
        foreach ($entry in @(Get-ChildItem -LiteralPath $plansPath -File)) {
            if ($entry.Name -cnotmatch '^plan-(.+)\.md$') {
                continue
            }

            $planSlug = $Matches[1]
            $scopeNamePattern = '^0[1-7]-' + [System.Text.RegularExpressions.Regex]::Escape($planSlug) + '\.md$'
            $matchingScopeFiles = @($scopeCategoryFiles | Where-Object { $_.Name -cmatch $scopeNamePattern })
            if ($matchingScopeFiles.Count -ne 1) {
                $displayPath = Get-DocsHygieneDisplayPath -RootPath $rootFullPath -Path $entry.FullName
                Add-DocsHygieneViolation -Violations $violations -Code "plan_category" -Path $displayPath -Detail ("scope_matches=" + $matchingScopeFiles.Count + " expected=1")
                continue
            }

            $scopeCategoryName = $matchingScopeFiles[0].Name
            if (-not $planCategoryCounts.ContainsKey($scopeCategoryName)) {
                $planCategoryCounts[$scopeCategoryName] = 0
            }
            $planCategoryCounts[$scopeCategoryName] = $planCategoryCounts[$scopeCategoryName] + 1
        }
    }

    foreach ($scopeCategoryName in $planCategoryCounts.Keys) {
        $planCount = $planCategoryCounts[$scopeCategoryName]
        if ($planCount -gt 1) {
            Add-DocsHygieneViolation -Violations $violations -Code "plan_category_duplicate" -Path "docs/plans" -Detail ("scope_category=" + $scopeCategoryName + " expected_max=1 actual=" + $planCount)
        }
    }

    # Rule 10 (adr-form): numbered ADRs are contiguous from 0001 and carry date/status metadata.
    $adrFormAllowedNonNumberedFiles = @("invariant-choices.md")
    if (Test-Path -LiteralPath $decisionsPath -PathType Container) {
        $adrNumberCounts = @{}
        foreach ($entry in @(Get-ChildItem -LiteralPath $decisionsPath -Force)) {
            $displayPath = Get-DocsHygieneDisplayPath -RootPath $rootFullPath -Path $entry.FullName
            if ($entry -isnot [System.IO.FileInfo]) {
                Add-DocsHygieneViolation -Violations $violations -Code "adr_unexpected_entry" -Path $displayPath -Detail "expected_file"
                continue
            }

            if ($adrFormAllowedNonNumberedFiles -ccontains $entry.Name) {
                continue
            }

            if ($entry.Name -cnotmatch '^(\d{4})-.*\.md$') {
                Add-DocsHygieneViolation -Violations $violations -Code "adr_unexpected_file" -Path $displayPath -Detail ("expected_name=NNNN-*.md_or_whitelisted=" + ($adrFormAllowedNonNumberedFiles -join ','))
                continue
            }

            $adrNumber = [int]$Matches[1]
            if ($adrNumber -lt 1) {
                Add-DocsHygieneViolation -Violations $violations -Code "adr_number_start" -Path $displayPath -Detail "expected_start=0001"
            }
            if (-not $adrNumberCounts.ContainsKey($adrNumber)) {
                $adrNumberCounts[$adrNumber] = 0
            }
            $adrNumberCounts[$adrNumber] = $adrNumberCounts[$adrNumber] + 1

            $adrContent = Get-Content -LiteralPath $entry.FullName -Raw
            $dateMatches = [System.Text.RegularExpressions.Regex]::Matches($adrContent, '(?m)^Date:\s*(\d{4}-\d{2}-\d{2})(?=\s|[.]|$)')
            $hasValidDate = $false
            foreach ($dateMatch in $dateMatches) {
                [datetime]$parsedDate = [datetime]::MinValue
                if ([datetime]::TryParseExact($dateMatch.Groups[1].Value, "yyyy-MM-dd", [System.Globalization.CultureInfo]::InvariantCulture, [System.Globalization.DateTimeStyles]::None, [ref]$parsedDate)) {
                    $hasValidDate = $true
                    break
                }
            }
            if (-not $hasValidDate) {
                Add-DocsHygieneViolation -Violations $violations -Code "adr_date" -Path $displayPath -Detail "expected_line=Date:_YYYY-MM-DD"
            }

            $hasStatusMarker = $adrContent -cmatch 'Status:'
            if (-not $hasStatusMarker) {
                Add-DocsHygieneViolation -Violations $violations -Code "adr_status" -Path $displayPath -Detail "expected_marker=Status:"
            }
        }

        if ($adrNumberCounts.Count -eq 0) {
            Add-DocsHygieneViolation -Violations $violations -Code "adr_number_gap" -Path "docs/architecture/decisions" -Detail "expected=0001 actual=none"
        }
        else {
            $highestAdrNumber = ($adrNumberCounts.Keys | Measure-Object -Maximum).Maximum
            foreach ($expectedAdrNumber in 1..$highestAdrNumber) {
                if (-not $adrNumberCounts.ContainsKey($expectedAdrNumber)) {
                    Add-DocsHygieneViolation -Violations $violations -Code "adr_number_gap" -Path "docs/architecture/decisions" -Detail ("missing=" + ('{0:D4}' -f $expectedAdrNumber))
                }
            }

            foreach ($adrNumber in @($adrNumberCounts.Keys | Sort-Object)) {
                $adrNumberCount = $adrNumberCounts[$adrNumber]
                if ($adrNumberCount -gt 1) {
                    Add-DocsHygieneViolation -Violations $violations -Code "adr_number_duplicate" -Path "docs/architecture/decisions" -Detail ("number=" + ('{0:D4}' -f $adrNumber) + " expected=1 actual=" + $adrNumberCount)
                }
            }
        }
    }

    # Rule 11 (archive-dated): archived files are date-prefixed; other entry types fail loudly.
    $archivePath = Join-Path $docsPath "_archive"
    if (Test-Path -LiteralPath $archivePath -PathType Container) {
        foreach ($entry in @(Get-ChildItem -LiteralPath $archivePath -Force)) {
            $displayPath = Get-DocsHygieneDisplayPath -RootPath $rootFullPath -Path $entry.FullName
            if ($entry -isnot [System.IO.FileInfo]) {
                Add-DocsHygieneViolation -Violations $violations -Code "archive_unexpected_entry" -Path $displayPath -Detail "expected_file"
                continue
            }

            if ($entry.Name -cnotmatch '^\d{4}-\d{2}-\d{2}_') {
                Add-DocsHygieneViolation -Violations $violations -Code "archive_dated" -Path $displayPath -Detail "expected_prefix=YYYY-MM-DD_"
            }
        }
    }

    # Rule 12 (breakdown-consistency): checked top-level scope boxes require fully green mapped groups.
    $breakdownMappings = @(
        [pscustomobject]@{
            ScopeSubstring = "structured report"
            BreakdownFile = "03-security-trust-pipeline.md"
            GroupHeadingSubstring = "report pipeline (ARTSTOR)"
        },
        [pscustomobject]@{
            ScopeSubstring = "Compiler diagnostics as JSON"
            BreakdownFile = "04-agent-fabric.md"
            GroupHeadingSubstring = "Feedback loop"
        },
        [pscustomobject]@{
            ScopeSubstring = "single source for"
            BreakdownFile = "07-docs-hygiene.md"
            GroupHeadingSubstring = "Single source"
        },
        [pscustomobject]@{
            ScopeSubstring = "Docs structure"
            BreakdownFile = "07-docs-hygiene.md"
            GroupHeadingSubstring = "Structure"
        },
        [pscustomobject]@{
            ScopeSubstring = "Every architecture decision is an ADR"
            BreakdownFile = "07-docs-hygiene.md"
            GroupHeadingSubstring = "Decisions & history"
        },
        [pscustomobject]@{
            ScopeSubstring = "never silently deleted"
            BreakdownFile = "07-docs-hygiene.md"
            GroupHeadingSubstring = "Decisions & history"
        },
        [pscustomobject]@{
            ScopeExactFirstLine = "**Floor interface narrow & kernel-agnostic:** ADR 0015 chooses the custom"
            BreakdownFile = "02-genesis-layer.md"
            GroupHeadingSubstring = "Floor contract"
        }
    )

    $scopeDocumentPath = Join-Path $docsPath "SCOPE.md"
    if (-not (Test-Path -LiteralPath $scopeDocumentPath -PathType Leaf)) {
        Add-DocsHygieneViolation -Violations $violations -Code "breakdown_consistency" -Path "docs/SCOPE.md" -Detail "required_file_missing"
    }
    else {
        $scopeLineNumber = 0
        foreach ($scopeLine in @(Get-Content -LiteralPath $scopeDocumentPath)) {
            $scopeLineNumber = $scopeLineNumber + 1
            if ($scopeLine -cnotmatch '^- \[x\]\s+(.+)$') {
                continue
            }

            $scopeBoxText = $Matches[1]
            $matchingMappings = @($breakdownMappings | Where-Object {
                if ($_.PSObject.Properties.Name -ccontains "ScopeExactFirstLine") {
                    return $scopeBoxText.Equals($_.ScopeExactFirstLine, [System.StringComparison]::Ordinal)
                }
                return $scopeBoxText.IndexOf($_.ScopeSubstring, [System.StringComparison]::OrdinalIgnoreCase) -ge 0
            })
            if ($matchingMappings.Count -ne 1) {
                Add-DocsHygieneViolation -Violations $violations -Code "breakdown_consistency" -Path "docs/SCOPE.md" -Detail ("line=" + $scopeLineNumber + " mapping_matches=" + $matchingMappings.Count + " expected=1 checkbox=" + $scopeBoxText)
                continue
            }

            $mapping = $matchingMappings[0]
            $breakdownRelativePath = "docs/scope/" + $mapping.BreakdownFile
            $breakdownFilePath = Join-Path $scopePath $mapping.BreakdownFile
            if (-not (Test-Path -LiteralPath $breakdownFilePath -PathType Leaf)) {
                Add-DocsHygieneViolation -Violations $violations -Code "breakdown_consistency" -Path $breakdownRelativePath -Detail ("mapped_file_missing scope_checkbox=" + $mapping.ScopeSubstring)
                continue
            }

            $breakdownLines = @(Get-Content -LiteralPath $breakdownFilePath)
            $matchingHeadingIndexes = New-Object "System.Collections.Generic.List[int]"
            for ($lineIndex = 0; $lineIndex -lt $breakdownLines.Count; $lineIndex++) {
                if (($breakdownLines[$lineIndex] -cmatch '^##\s+(.+)$') -and
                    ($Matches[1].IndexOf($mapping.GroupHeadingSubstring, [System.StringComparison]::OrdinalIgnoreCase) -ge 0)) {
                    [void]$matchingHeadingIndexes.Add($lineIndex)
                }
            }

            if ($matchingHeadingIndexes.Count -ne 1) {
                Add-DocsHygieneViolation -Violations $violations -Code "breakdown_consistency" -Path $breakdownRelativePath -Detail ("group_heading=" + $mapping.GroupHeadingSubstring + " matches=" + $matchingHeadingIndexes.Count + " expected=1")
                continue
            }

            $groupCheckboxCount = 0
            for ($lineIndex = $matchingHeadingIndexes[0] + 1; $lineIndex -lt $breakdownLines.Count; $lineIndex++) {
                $groupLine = $breakdownLines[$lineIndex]
                if ($groupLine -cmatch '^##\s+') {
                    break
                }
                if ($groupLine -cmatch '^\s*-\s+\[([ xX])\]') {
                    $groupCheckboxCount = $groupCheckboxCount + 1
                    if ($Matches[1] -cne "x") {
                        Add-DocsHygieneViolation -Violations $violations -Code "breakdown_consistency" -Path $breakdownRelativePath -Detail ("group_heading=" + $mapping.GroupHeadingSubstring + " open_checkbox_line=" + ($lineIndex + 1))
                    }
                }
            }

            if ($groupCheckboxCount -eq 0) {
                Add-DocsHygieneViolation -Violations $violations -Code "breakdown_consistency" -Path $breakdownRelativePath -Detail ("group_heading=" + $mapping.GroupHeadingSubstring + " checkboxes=0 expected_at_least=1")
            }
        }
    }

    # Every numbered breakdown identifies docs/SCOPE.md near the top.
    foreach ($scopeCategoryFile in $scopeCategoryFiles) {
        $breakdownDisplayPath = Get-DocsHygieneDisplayPath -RootPath $rootFullPath -Path $scopeCategoryFile.FullName
        $firstFiveLines = @(Get-Content -LiteralPath $scopeCategoryFile.FullName -TotalCount 5)
        $firstFiveText = $firstFiveLines -join "`n"
        if ($firstFiveText.IndexOf("docs/SCOPE.md", [System.StringComparison]::Ordinal) -lt 0) {
            Add-DocsHygieneViolation -Violations $violations -Code "breakdown_backlink" -Path $breakdownDisplayPath -Detail "required_phrase=docs/SCOPE.md location=first_5_lines"
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
        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "SCOPE.md") -Value @(
            "# self-test scope",
            "",
            "## 2. Genesis Layer",
            "- [x] **Floor interface narrow & kernel-agnostic:** ADR 0015 chooses the custom",
            "      kernel; the substitutable floor is the documented Wasm-import + service-",
            "      capability contract, with no kernel-internal types. A contract test rejects",
            "      any service that depends on kernel internals",
            "",
            "## 3. Security",
            "- [x] Report pipeline: every build/test emits a structured report (ARTSTOR)"
        ) -Encoding utf8
        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "README.md") -Value "conflicts resolve in favor of SCOPE.md" -Encoding utf8
        Set-Content -LiteralPath (Join-Path $fixtureRoot "AGENTS.md") -Value "reference docs/SCOPE.md" -Encoding utf8
        foreach ($number in 1..7) {
            $scopeFileName = ('{0:D2}-self-test-{0:D2}.md' -f $number)
            $scopeFileContent = @(
                ("# " + ('{0:D2}' -f $number) + " self-test"),
                '> Breakdown of `docs/SCOPE.md` self-test.',
                "",
                "## Self-test group",
                "- [x] baseline"
            )
            if ($number -eq 2) {
                $scopeFileName = "02-genesis-layer.md"
                $scopeFileContent = @(
                    "# 02 self-test",
                    '> Breakdown of `docs/SCOPE.md` self-test.',
                    "",
                    "## Floor contract",
                    "- [x] The full Wasm import + service-capability floor fits in one document",
                    "- [x] No kernel-internal types leak through the import/service interface",
                    "- [x] Contract conformance rejects undeclared or kernel-internal dependencies"
                )
            }
            elseif ($number -eq 3) {
                $scopeFileContent = @(
                    "# 03 self-test",
                    '> Breakdown of `docs/SCOPE.md` self-test.',
                    "",
                    "## Day 1 - report pipeline (ARTSTOR)",
                    "- [ ] planted state divergence"
                )
            }
            elseif ($number -eq 7) {
                $scopeFileContent = @(
                    "# 07 self-test",
                    "> Breakdown self-test with the backlink removed.",
                    "",
                    "## Self-test group",
                    "- [x] baseline"
                )
            }
            Set-Content -LiteralPath (Join-Path $fixtureDocsPath ("scope\" + $scopeFileName)) -Value $scopeFileContent -Encoding utf8
        }
        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "plans\plan-self-test-01.md") -Value "self-test" -Encoding utf8
        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "architecture\decisions\0001-valid.md") -Value "Date: 2026-07-18`r`nStatus: active" -Encoding utf8
        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "architecture\decisions\0003-first.md") -Value "Date: 2026-07-18`r`nStatus: active" -Encoding utf8
        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "architecture\decisions\0003-duplicate.md") -Value "Date: 2026-07-18`r`nStatus: active" -Encoding utf8
        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "architecture\decisions\0004-dateless.md") -Value "Status: active" -Encoding utf8
        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "architecture\decisions\invariant-choices.md") -Value "self-test" -Encoding utf8
        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "architecture\decisions\stray.md") -Value "planted violation" -Encoding utf8
        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "_archive\undated.md") -Value "planted violation" -Encoding utf8

        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "foreign.txt") -Value "planted violation" -Encoding utf8
        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "status\HANDOFF.md") -Value ("x" * 5000) -Encoding utf8
        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "status\STATUS.md") -Value ("x" * 31000) -Encoding utf8
        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "plans\wrong-name.md") -Value "planted violation" -Encoding utf8
        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "plans\plan-nonexistent-thing.md") -Value "planted violation" -Encoding utf8
        Set-Content -LiteralPath (Join-Path $fixtureDocsPath "README.md") -Value "planted statement without the required phrase" -Encoding utf8
        Add-Content -LiteralPath (Join-Path $fixtureRoot "AGENTS.md") -Value "reference docs/nonexistent-root-reference.md" -Encoding utf8
        Set-Content -LiteralPath (Join-Path $fixtureRoot "CLAUDE.md") -Value "planted legacy instruction" -Encoding utf8
        [void](New-Item -ItemType Directory -Path (Join-Path $fixtureRoot ".claude") -Force)

        $selfTestResult = Invoke-DocsHygieneCheck -RootPath $fixtureRoot
        $floorBreakdownPath = "docs/scope/02-genesis-layer.md"
        $floorGreenViolations = @($selfTestResult.Violations | Where-Object {
            ($_.Code -ceq "breakdown_consistency") -and ($_.Path -ceq $floorBreakdownPath)
        })

        $floorFixturePath = Join-Path $fixtureDocsPath "scope\02-genesis-layer.md"
        $floorFixtureLines = @(Get-Content -LiteralPath $floorFixturePath)
        $floorFixtureLines[6] = "- [ ] Contract conformance rejects undeclared or kernel-internal dependencies"
        Set-Content -LiteralPath $floorFixturePath -Value $floorFixtureLines -Encoding utf8
        $floorRedResult = Invoke-DocsHygieneCheck -RootPath $fixtureRoot
        $floorRedViolations = @($floorRedResult.Violations | Where-Object {
            ($_.Code -ceq "breakdown_consistency") -and
            ($_.Path -ceq $floorBreakdownPath) -and
            ($_.Detail -ceq "group_heading=Floor contract open_checkbox_line=7")
        })

        $detectedCodes = @($selfTestResult.Violations | ForEach-Object { $_.Code })
        $requiredCodes = @("docs_root_entry", "handoff_too_large", "status_too_large", "plan_filename", "single_source", "root_instruction_path", "legacy_agent_instruction", "plan_category", "adr_number_gap", "adr_number_duplicate", "adr_date", "adr_unexpected_file", "archive_dated", "breakdown_consistency", "breakdown_backlink")
        $missingCodes = @($requiredCodes | Where-Object { $detectedCodes -cnotcontains $_ })
        $legacyInstructionViolations = @($selfTestResult.Violations | Where-Object { $_.Code -ceq "legacy_agent_instruction" })

        if (($missingCodes.Count -eq 0) -and ($legacyInstructionViolations.Count -eq 2) -and ($floorGreenViolations.Count -eq 0) -and ($floorRedViolations.Count -eq 1)) {
            Write-Output "DOCS_HYGIENE selftest=green planted=16 detected=16"
            Write-Output "DOCS_HYGIENE floor_mapping=green checked_breakdown_boxes=3 red_path=open_checkbox_detected"
            Write-Output ("DOCS_HYGIENE result=green checks=" + $selfTestResult.CheckCount + " violations=0")
            exit 0
        }

        Write-Output ("DOCS_HYGIENE violation=selftest_detection path=temporary_fixture detail=missing_codes=" + ($missingCodes -join ',') + " legacy_instruction_violations=" + $legacyInstructionViolations.Count + " floor_green_violations=" + $floorGreenViolations.Count + " floor_red_violations=" + $floorRedViolations.Count)
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
