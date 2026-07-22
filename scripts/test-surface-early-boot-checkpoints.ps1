param([switch]$SkipBuild)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot

function Require([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Require-Match([string]$Text, [string]$Pattern, [string]$Message) {
    $options = [Text.RegularExpressions.RegexOptions]::Singleline
    Require ([regex]::IsMatch($Text, $Pattern, $options)) $Message
}

function Slice-Between([string]$Text, [string]$Start, [string]$End) {
    $startAt = $Text.IndexOf($Start, [StringComparison]::Ordinal)
    $endAt = $Text.IndexOf($End, $startAt + $Start.Length, [StringComparison]::Ordinal)
    Require (($startAt -ge 0) -and ($endAt -gt $startAt)) "missing source slice $Start"
    $Text.Substring($startAt, $endAt - $startAt)
}

function Test-ExactSequence([string[]]$Actual, [string[]]$Expected) {
    if ($Actual.Count -ne $Expected.Count) { return $false }
    for ($index = 0; $index -lt $Expected.Count; $index++) {
        if ($Actual[$index] -cne $Expected[$index]) { return $false }
    }
    $true
}

function Get-CheckpointCalls([string]$Source) {
    @([regex]::Matches(
        $Source,
        'status_ui\.(render_early_boot_[a-z_]+)\(\);'
    ) | ForEach-Object { $_.Groups[1].Value })
}

function Require-UniqueOrderedTokens([string]$Source, [string[]]$Tokens) {
    $previous = -1
    foreach ($token in $Tokens) {
        $count = [regex]::Matches($Source, [regex]::Escape($token)).Count
        Require ($count -eq 1) "boot-order token is not unique: $token"
        $position = $Source.IndexOf($token, [StringComparison]::Ordinal)
        Require ($position -gt $previous) "boot-order token is missing or reordered: $token"
        $previous = $position
    }
}

function Get-ColorKey([string]$Source, [string]$Name) {
    $pattern = 'const\s+' + [regex]::Escape($Name) +
        ':\s*Color\s*=\s*Color::new\((\d+),\s*(\d+),\s*(\d+)\);'
    $match = [regex]::Match($Source, $pattern)
    Require $match.Success "checkpoint background color is not a static RGB constant: $Name"
    "$($match.Groups[1].Value),$($match.Groups[2].Value),$($match.Groups[3].Value)"
}

function Get-ModeledCheckpointLayout([int]$Width, [int]$Height, [string]$Code) {
    $textWidth = (($Code.Length - 1) * $script:GlyphAdvance) + $script:GlyphWidth
    $widthScale = [int][Math]::Floor($Width / $textWidth)
    $heightScale = [int][Math]::Floor($Height / $script:GlyphHeight)
    $fitScale = [Math]::Min($widthScale, $heightScale)
    $scale = [int][Math]::Max(1, [Math]::Min($script:MaxScale, $fitScale))
    $remainingWidth = [Math]::Max(0, $Width - ($textWidth * $scale))
    $remainingHeight = [Math]::Max(0, $Height - ($script:GlyphHeight * $scale))
    $x = [int][Math]::Floor([Math]::Floor($remainingWidth / $scale) / 2)
    $y = [int][Math]::Floor([Math]::Floor($remainingHeight / $scale) / 2)
    [pscustomobject]@{
        Scale = $scale
        X = $x
        Y = $y
        TextWidth = $textWidth
        TextFits = (($Width -ge $textWidth) -and ($Height -ge $script:GlyphHeight))
    }
}

$script:ExpectedMethods = @(
    "render_early_boot_before_surface",
    "render_early_boot_before_usb",
    "render_early_boot_after_usb",
    "render_early_boot_after_persist",
    "render_early_boot_persist_failed",
    "render_early_boot_surface_failed"
)
$script:ExpectedCodes = @("EB1", "EB2", "EB3", "EB4P", "EB4E", "EB4F")
$script:ExpectedBackgrounds = @("APP_BLUE", "APP_AMBER", "APP_GREEN", "SURFACE_ALT", "APP_BG", "APP_RED")
$script:MaxScale = 4
$script:GlyphWidth = 8
$script:GlyphHeight = 8
$script:GlyphAdvance = 9

function Assert-CheckpointSources([string]$MainSource, [string]$GenesisSource) {
    $earlyMain = Slice-Between $MainSource "fn early_main() -> !" "fn init_framebuffer()"
    $checkpointSurface = Slice-Between $GenesisSource "    const EARLY_BOOT_MAX_SCALE" "    pub fn render(&mut self"
    $persistPhase = Slice-Between $earlyMain "    if let Ok(records) = surface_capture {" "    let iommu_report = iommu_vtd::probe();"

    $observedMethods = @(Get-CheckpointCalls $earlyMain)
    Require (Test-ExactSequence $observedMethods $script:ExpectedMethods) "early_main checkpoints are missing, duplicated, or reordered"

    Require-UniqueOrderedTokens $earlyMain @(
        "let mut status_ui = shell_host::ShellHost::new(framebuffer_surface);",
        "status_ui.render_early_boot_before_surface();",
        "let surface_capture = surface_fact_capture::capture(",
        "status_ui.render_early_boot_before_usb();",
        "usb::init();",
        "status_ui.render_early_boot_after_usb();",
        "if let Ok(records) = surface_capture {",
        "usb::append_surface_fact_capture(&records)",
        "status_ui.render_early_boot_after_persist();",
        "status_ui.render_early_boot_persist_failed();",
        "status_ui.render_early_boot_surface_failed();",
        "let iommu_report = iommu_vtd::probe();"
    )
    Require-Match $earlyMain 'ShellHost::new\(framebuffer_surface\);\s*status_ui\.render_early_boot_before_surface\(\);' "first checkpoint is not immediate after ShellHost framebuffer ownership"
    Require-Match $earlyMain 'status_ui\.render_early_boot_before_usb\(\);\s*usb::init\(\);\s*status_ui\.render_early_boot_after_usb\(\);' "USB enter checkpoint is not presented immediately before the potentially hanging call"
    $branchPattern = 'if let Ok\(records\) = surface_capture\s*\{\s*match usb::append_surface_fact_capture\(&records\)\s*\{\s*Ok\(\(\)\)\s*=>\s*\{(?<success>.*?)\}\s*Err\(reason\)\s*=>\s*\{(?<error>.*?)\}\s*\}\s*\}\s*else\s*\{(?<measurement>.*?)\}\s*$'
    $branchMatch = [regex]::Match(
        $persistPhase,
        $branchPattern,
        [Text.RegularExpressions.RegexOptions]::Singleline
    )
    Require $branchMatch.Success "persistence and measurement outcomes are not exclusive static branches"
    Require (Test-ExactSequence @(Get-CheckpointCalls $branchMatch.Groups["success"].Value) @(
        "render_early_boot_after_persist"
    )) "append success does not exclusively select EB4P"
    Require (Test-ExactSequence @(Get-CheckpointCalls $branchMatch.Groups["error"].Value) @(
        "render_early_boot_persist_failed"
    )) "append error does not exclusively select EB4E"
    Require (Test-ExactSequence @(Get-CheckpointCalls $branchMatch.Groups["measurement"].Value) @(
        "render_early_boot_surface_failed"
    )) "surface measurement error does not exclusively select EB4F"

    for ($index = 0; $index -lt $script:ExpectedMethods.Count; $index++) {
        $method = [regex]::Escape($script:ExpectedMethods[$index])
        $code = [regex]::Escape($script:ExpectedCodes[$index])
        $background = [regex]::Escape($script:ExpectedBackgrounds[$index])
        $mappingPattern = 'pub fn ' + $method +
            '\(&mut self\)\s*\{\s*self\.render_early_boot_checkpoint\("' +
            $code + '",\s*' + $background + '\);\s*\}'
        Require-Match $checkpointSurface $mappingPattern "checkpoint method/code/background mapping is missing or dynamic: $($script:ExpectedMethods[$index])"
        $quotedCode = '"' + $script:ExpectedCodes[$index] + '"'
        Require ([regex]::Matches($GenesisSource, [regex]::Escape($quotedCode)).Count -eq 1) "checkpoint code is not unique: $($script:ExpectedCodes[$index])"
        Require ([Text.Encoding]::ASCII.GetByteCount($script:ExpectedCodes[$index]) -eq $script:ExpectedCodes[$index].Length) "checkpoint code is not ASCII: $($script:ExpectedCodes[$index])"
    }

    Require ([regex]::Matches($checkpointSurface, 'self\.render_early_boot_checkpoint\(').Count -eq $script:ExpectedMethods.Count) "checkpoint renderer accepts a non-static or extra caller"
    $renderPattern = "fn render_early_boot_checkpoint\(&mut self, code: &'static str, background: Color\).*let \(scale, x, y, text_fits\) =.*Self::early_boot_checkpoint_layout\(surface\.info\(\), code\.len\(\)\);.*surface\.set_draw_scale\(scale\);.*surface\.fill\(background\);.*if text_fits\s*\{\s*text::draw_text\(surface, x, y, code, TEXT_MAIN, None\);\s*\}.*surface\.present\(\);"
    Require-Match $checkpointSurface $renderPattern "ShellHost checkpoint path does not use the bounded framebuffer/text/present primitives"
    $layoutPattern = 'const EARLY_BOOT_MAX_SCALE: usize = 4;.*const EARLY_BOOT_GLYPH_WIDTH: usize = 8;.*const EARLY_BOOT_GLYPH_HEIGHT: usize = 8;.*const EARLY_BOOT_GLYPH_ADVANCE: usize = 9;.*fn early_boot_checkpoint_layout\(.*code_len: usize,.*\(usize, usize, usize, bool\).*let text_width = code_len.*saturating_sub\(1\).*saturating_mul\(Self::EARLY_BOOT_GLYPH_ADVANCE\).*saturating_add\(Self::EARLY_BOOT_GLYPH_WIDTH\);.*let fit_scale = usize::min\(.*width / text_width\.max\(1\),.*height / Self::EARLY_BOOT_GLYPH_HEIGHT,.*\);.*let text_fits =.*width >= text_width && height >= Self::EARLY_BOOT_GLYPH_HEIGHT;.*let scale = usize::min\(Self::EARLY_BOOT_MAX_SCALE, fit_scale\)\.max\(1\);.*let x = width\.saturating_sub\(text_width\.saturating_mul\(scale\)\) / scale / 2;.*let y =.*height\.saturating_sub\(Self::EARLY_BOOT_GLYPH_HEIGHT\.saturating_mul\(scale\)\).* / scale.* / 2;.*\(scale, x, y, text_fits\)'
    Require-Match $checkpointSurface $layoutPattern "checkpoint scale/position are not derived from the modeled framebuffer and glyph bounds"
    Require ([regex]::Matches($checkpointSurface, 'text::draw_text\(').Count -eq 1) "checkpoint path has an unexpected text-rendering route"
    Require ([regex]::Matches($checkpointSurface, 'surface\.fill\(').Count -eq 1) "checkpoint path has an unexpected framebuffer fill route"
    Require ([regex]::Matches($checkpointSurface, 'surface\.present\(\)').Count -eq 1) "checkpoint path does not present exactly once"
    Require ($checkpointSurface -notmatch '(?i)format!|format_args!|serial::|console::|provider::|secret|vault|pci::|wifi::|usb::|append|write_|unsafe') "checkpoint surface can consume dynamic, secret, device, or persistence data"

    $literalValues = @([regex]::Matches(
        $checkpointSurface,
        '"([^"\\]*(?:\\.[^"\\]*)*)"'
    ) | ForEach-Object { $_.Groups[1].Value })
    Require (Test-ExactSequence $literalValues $script:ExpectedCodes) "checkpoint renderer contains non-code or dynamic text"

    $colorKeys = @($script:ExpectedBackgrounds | ForEach-Object {
        Get-ColorKey $GenesisSource $_
    })
    Require ((@($colorKeys | Select-Object -Unique)).Count -eq $script:ExpectedBackgrounds.Count) "checkpoint backgrounds are not diagnostically distinct below the glyph visibility bound"

    $visibleModes = @(
        @{ Name = "minimum-full-tag"; Width = 35; Height = 8 },
        @{ Name = "small"; Width = 80; Height = 25 },
        @{ Name = "review-height-boundary"; Width = 676; Height = 32 },
        @{ Name = "surface-like"; Width = 2736; Height = 1824 }
    )
    foreach ($mode in $visibleModes) {
        foreach ($code in $script:ExpectedCodes) {
            $layout = Get-ModeledCheckpointLayout $mode.Width $mode.Height $code
            $right = ($layout.X + $layout.TextWidth) * $layout.Scale
            $bottom = ($layout.Y + $script:GlyphHeight) * $layout.Scale
            Require $layout.TextFits "checkpoint tag does not fit modeled accepted mode: $($mode.Name)/$code"
            Require (($layout.Scale -ge 1) -and ($layout.Scale -le $script:MaxScale)) "modeled scale is outside bounds: $($mode.Name)/$code"
            Require (($right -le $mode.Width) -and ($bottom -le $mode.Height)) "checkpoint tag is clipped in modeled accepted mode: $($mode.Name)/$code"
        }
    }
    foreach ($code in $script:ExpectedCodes) {
        $tiny = Get-ModeledCheckpointLayout 1 1 $code
        Require ((-not $tiny.TextFits) -and ($tiny.Scale -eq 1) -and ($tiny.X -eq 0) -and ($tiny.Y -eq 0)) "tiny accepted framebuffer does not use the bounded color fallback: $code"
    }
}

function Assert-RejectedMutation(
    [string]$MainSource,
    [string]$GenesisSource,
    [string]$ReportId
) {
    $rejected = $false
    try {
        Assert-CheckpointSources $MainSource $GenesisSource
    } catch {
        $rejected = $true
    }
    Require $rejected "$ReportId accepted a mutated source tree"
    Write-Output ($ReportId + ": PASS")
}

$mainPath = Join-Path $RepoRoot "seed-kernel\src\main.rs"
$genesisPath = Join-Path $RepoRoot "seed-kernel\src\shell_host\genesis.rs"
$main = [IO.File]::ReadAllText($mainPath)
$genesis = [IO.File]::ReadAllText($genesisPath)

Assert-CheckpointSources $main $genesis
Write-Output "SURFACE-EARLY-BOOT-BOUNDS: PASS"

# Negative boundary: mutate the complete source input and run the same source
# contract used for the real files.
$missingToken = "    status_ui.render_early_boot_before_usb();"
Require ([regex]::Matches($main, [regex]::Escape($missingToken)).Count -eq 1) "missing-negative source token is not unique"
$missingMain = $main.Replace($missingToken, "")
Assert-RejectedMutation $missingMain $genesis "SURFACE-EARLY-BOOT-NEG-MISSING"

$duplicateToken = "    status_ui.render_early_boot_after_usb();"
Require ([regex]::Matches($main, [regex]::Escape($duplicateToken)).Count -eq 1) "duplicate-negative source token is not unique"
$duplicateMain = $main.Replace(
    $duplicateToken,
    $duplicateToken + [Environment]::NewLine + $duplicateToken
)
Assert-RejectedMutation $duplicateMain $genesis "SURFACE-EARLY-BOOT-NEG-DUPLICATE"

$beforeUsbName = "render_early_boot_before_usb"
$afterUsbName = "render_early_boot_after_usb"
$swapSentinel = "render_early_boot_swap_checkpoint"
Require (-not $main.Contains($swapSentinel)) "reorder-negative sentinel already exists"
$reorderedMain = $main.Replace($beforeUsbName, $swapSentinel).
    Replace($afterUsbName, $beforeUsbName).
    Replace($swapSentinel, $afterUsbName)
Require ($reorderedMain -cne $main) "reorder-negative did not mutate the source"
Assert-RejectedMutation $reorderedMain $genesis "SURFACE-EARLY-BOOT-NEG-REORDER"

$collapsedErrorToken = "                status_ui.render_early_boot_persist_failed();"
Require ([regex]::Matches($main, [regex]::Escape($collapsedErrorToken)).Count -eq 1) "collapsed-error negative source token is not unique"
$collapsedErrorMain = $main.Replace(
    $collapsedErrorToken,
    "                status_ui.render_early_boot_after_persist();"
)
Require ($collapsedErrorMain -cne $main) "collapsed-error negative did not mutate the source"
Assert-RejectedMutation $collapsedErrorMain $genesis "SURFACE-EARLY-BOOT-NEG-ERR-COLLAPSED"

Push-Location $RepoRoot
try {
    $rustDiff = @(git diff HEAD --unified=0 -- seed-kernel/src/main.rs seed-kernel/src/shell_host/genesis.rs)
    if ($LASTEXITCODE -ne 0) { throw "git diff HEAD failed" }
    $addedRust = @($rustDiff | Where-Object {
        $_.StartsWith("+", [StringComparison]::Ordinal) -and
        -not $_.StartsWith("+++", [StringComparison]::Ordinal)
    })
    $forbiddenRust = '\bunsafe\b|\bpci::|\bwifi::|program_persistence::|structured_store|memory_store|usb::append|append_surface_fact_capture|reclog|msc_'
    Require (($addedRust -join [Environment]::NewLine) -notmatch $forbiddenRust) "Rust additions introduce unsafe, PCI, Wi-Fi, or persistence behavior"

    $expectedPaths = @(
        "scripts/test-surface-early-boot-checkpoints.ps1",
        "seed-kernel/src/main.rs",
        "seed-kernel/src/shell_host/genesis.rs"
    ) | Sort-Object
    $statusLines = @(git status --short --untracked-files=all)
    if ($LASTEXITCODE -ne 0) { throw "git status failed" }
    $actualPaths = @($statusLines | ForEach-Object {
        $_.Substring(3).Replace('\', '/')
    }) | Sort-Object
    Require ((@(Compare-Object $expectedPaths $actualPaths)).Count -eq 0) "dirty file set differs from the exact three-file lane"

    git diff HEAD --check -- seed-kernel/src/main.rs seed-kernel/src/shell_host/genesis.rs
    if ($LASTEXITCODE -ne 0) { throw "tracked diff whitespace check failed" }
    $selfCheck = @(git -c core.autocrlf=false diff --no-index --check -- NUL scripts/test-surface-early-boot-checkpoints.ps1 2>&1)
    $selfCheckExit = $LASTEXITCODE
    Require (($selfCheckExit -in @(0, 1)) -and ($selfCheck.Count -eq 0)) "predicate whitespace check failed"
} finally {
    Pop-Location
}

if (-not $SkipBuild) {
    & (Join-Path $RepoRoot "scripts\build-seed-kernel.ps1") -Profile release
    if ($LASTEXITCODE -ne 0) { throw "freestanding release seed-kernel build failed" }
}

Write-Output "surface early-boot checkpoint predicate: PASS"
exit 0
