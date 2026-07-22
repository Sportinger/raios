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

function Get-CapturePhaseCalls([string]$Source) {
    @([regex]::Matches(
        $Source,
        'progress\(CapturePhase::([A-Za-z]+)\);'
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

$script:ExpectedBootBackgrounds = @("APP_BLUE", "SURFACE_BG", "HAIRLINE", "APP_AMBER", "APP_GREEN", "SURFACE_ALT", "APP_BG", "APP_RED")
$script:ExpectedCaptureMethods = @(
    "render_early_boot_capture_cpuid",
    "render_early_boot_capture_smbios",
    "render_early_boot_capture_memory_map",
    "render_early_boot_capture_pci",
    "render_early_boot_capture_finalize"
)
$script:ExpectedCapturePhases = @("Cpuid", "Smbios", "MemoryMap", "Pci", "Finalize")
$script:ExpectedCaptureBackgrounds = @("APP_BLUE", "SURFACE_BG", "HAIRLINE", "APP_AMBER", "APP_GREEN")
$script:ExpectedMethods = @(
    "render_early_boot_before_surface",
    "render_early_boot_after_provider_config",
    "render_early_boot_after_console",
    "render_early_boot_capture_cpuid",
    "render_early_boot_capture_smbios",
    "render_early_boot_capture_memory_map",
    "render_early_boot_capture_pci",
    "render_early_boot_capture_finalize",
    "render_early_boot_before_usb",
    "render_early_boot_after_usb",
    "render_early_boot_after_persist",
    "render_early_boot_persist_failed",
    "render_early_boot_surface_failed"
)
$script:ExpectedCodes = @("EB1", "EB1P", "EB1C", "SC", "SS", "SM", "SP", "SV", "EB2", "EB3", "EB4P", "EB4E", "EB4F")
$script:ExpectedBackgrounds = @("APP_BLUE", "SURFACE_BG", "HAIRLINE", "APP_BLUE", "SURFACE_BG", "HAIRLINE", "APP_AMBER", "APP_GREEN", "APP_AMBER", "APP_GREEN", "SURFACE_ALT", "APP_BG", "APP_RED")
$script:MaxScale = 4
$script:GlyphWidth = 8
$script:GlyphHeight = 8
$script:GlyphAdvance = 9

function Assert-CheckpointSources(
    [string]$MainSource,
    [string]$GenesisSource,
    [string]$CaptureSource
) {
    $earlyMain = Slice-Between $MainSource "fn early_main() -> !" "fn init_framebuffer()"
    $checkpointSurface = Slice-Between $GenesisSource "    const EARLY_BOOT_MAX_SCALE" "    pub fn render(&mut self"
    $persistPhase = Slice-Between $earlyMain "    if let Ok(records) = surface_capture {" "    let iommu_report = iommu_vtd::probe();"
    $captureMapping = Slice-Between $earlyMain "    let surface_capture = surface_fact_capture::capture(" "    status_ui.render_early_boot_before_usb();"
    $captureFunction = Slice-Between $CaptureSource "pub fn capture(" "fn capture_cpuid("

    Require ([regex]::Matches($CaptureSource, '\bpub enum CapturePhase\b').Count -eq 1) "CapturePhase public enum count changed"
    Require-Match $CaptureSource 'pub enum CapturePhase\s*\{\s*Cpuid,\s*Smbios,\s*MemoryMap,\s*Pci,\s*Finalize,\s*\}' "CapturePhase is not the exact ordered fieldless public enum"
    Require-Match $captureFunction 'pub fn capture\(.*mut progress: impl FnMut\(CapturePhase\),\s*\)\s*->\s*Result<Vec<CaptureRecord>, &''static str>' "capture progress callback signature is missing or dynamic"
    $observedCapturePhases = @(Get-CapturePhaseCalls $captureFunction)
    Require (Test-ExactSequence $observedCapturePhases $script:ExpectedCapturePhases) "capture progress callbacks are missing, duplicated, or reordered"
    Require ([regex]::Matches($captureFunction, '\bprogress\(').Count -eq $script:ExpectedCapturePhases.Count) "capture invokes progress outside the five phase boundaries"
    Require-Match $captureFunction 'progress\(CapturePhase::Cpuid\);\s*capture_cpuid\(&mut payloads\)\?;' "CPUID progress is not immediately before CPUID capture"
    Require-Match $captureFunction 'progress\(CapturePhase::Smbios\);\s*capture_smbios\(smbios, memory_map, hhdm, &mut payloads\)\?;' "SMBIOS progress is not immediately before SMBIOS capture"
    Require-Match $captureFunction 'progress\(CapturePhase::MemoryMap\);\s*capture_memory_map\(memory_map, &mut payloads\)\?;' "memory-map progress is not immediately before memory-map capture"
    Require-Match $captureFunction 'progress\(CapturePhase::Pci\);\s*let functions = pci::enumerate_functions_for_capture\(\)\?;' "PCI progress is not immediately before PCI capture"
    Require-Match $captureFunction 'progress\(CapturePhase::Finalize\);\s*let capture_id = capture_id\(nonce, &payloads\);' "finalize progress is not immediately before finalization"

    $mappingMatches = @([regex]::Matches(
        $captureMapping,
        'surface_fact_capture::CapturePhase::([A-Za-z]+)\s*=>\s*\{\s*status_ui\.(render_early_boot_capture_[a-z_]+)\(\);\s*\}'
    ))
    Require ($mappingMatches.Count -eq $script:ExpectedCapturePhases.Count) "main CapturePhase mapping arm count changed"
    for ($index = 0; $index -lt $script:ExpectedCapturePhases.Count; $index++) {
        Require ($mappingMatches[$index].Groups[1].Value -ceq $script:ExpectedCapturePhases[$index]) "main CapturePhase variants are missing or reordered"
        Require ($mappingMatches[$index].Groups[2].Value -ceq $script:ExpectedCaptureMethods[$index]) "main CapturePhase ShellHost mapping is missing or reordered"
    }
    Require ([regex]::Matches($captureMapping, '=>').Count -eq $script:ExpectedCapturePhases.Count) "main CapturePhase match contains a fallback or extra arm"
    Require-Match $captureMapping '\|phase\|\s*match phase\s*\{' "capture callback is not an exhaustive phase match"
    Require ($captureMapping -notmatch '(?i)format!|format_args!|serial::|secret|vault|reason|error|unsafe') "capture progress mapping can consume values, secrets, or errors"

    $observedMethods = @(Get-CheckpointCalls $earlyMain)
    Require (Test-ExactSequence $observedMethods $script:ExpectedMethods) "early_main checkpoints are missing, duplicated, or reordered"

    Require-UniqueOrderedTokens $earlyMain @(
        "let mut status_ui = shell_host::ShellHost::new(framebuffer_surface);",
        "status_ui.render_early_boot_before_surface();",
        "let default_provider_loaded = provider_config::init_default_config();",
        "status_ui.render_early_boot_after_provider_config();",
        "if default_provider_loaded {",
        "console::init();",
        "status_ui.render_early_boot_after_console();",
        "let surface_capture = surface_fact_capture::capture(",
        "status_ui.render_early_boot_capture_cpuid()",
        "status_ui.render_early_boot_capture_smbios()",
        "status_ui.render_early_boot_capture_memory_map()",
        "status_ui.render_early_boot_capture_pci()",
        "status_ui.render_early_boot_capture_finalize()",
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
    Require-Match $earlyMain 'ShellHost::new\(framebuffer_surface\);\s*status_ui\.render_early_boot_before_surface\(\);\s*let default_provider_loaded = provider_config::init_default_config\(\);\s*status_ui\.render_early_boot_after_provider_config\(\);\s*if default_provider_loaded' "EB1/EB1P do not immediately bracket provider configuration independently of its bool result"
    Require-Match $earlyMain 'console::init\(\);\s*status_ui\.render_early_boot_after_console\(\);\s*let surface_capture = surface_fact_capture::capture\(.*?\|phase\|\s*match phase\s*\{.*?\},\s*\);\s*status_ui\.render_early_boot_before_usb\(\);' "EB1C, capture phase mapping, and post-return EB2 boundary changed"
    Require-Match $earlyMain 'status_ui\.render_early_boot_before_usb\(\);.*?usb::init\(\);\s*status_ui\.render_early_boot_after_usb\(\);' "EB2 is not before USB init or EB3 is not immediate after USB return"
    Require ([regex]::Matches($earlyMain, 'provider_config::init_default_config\(\)').Count -eq 1) "provider configuration call count changed"
    Require ([regex]::Matches($earlyMain, 'console::init\(\)').Count -eq 1) "console initialization call count changed"
    Require ([regex]::Matches($earlyMain, 'surface_fact_capture::capture\(').Count -eq 1) "Surface capture call count changed"
    Require ([regex]::Matches($earlyMain, 'usb::init\(\)').Count -eq 1) "USB initialization call count changed"
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

    $bootColorKeys = @($script:ExpectedBootBackgrounds | ForEach-Object {
        Get-ColorKey $GenesisSource $_
    })
    Require ((@($bootColorKeys | Select-Object -Unique)).Count -eq $script:ExpectedBootBackgrounds.Count) "existing checkpoint backgrounds are not diagnostically distinct below the glyph visibility bound"
    $captureColorKeys = @($script:ExpectedCaptureBackgrounds | ForEach-Object {
        Get-ColorKey $GenesisSource $_
    })
    Require ((@($captureColorKeys | Select-Object -Unique)).Count -eq $script:ExpectedCaptureBackgrounds.Count) "capture phase backgrounds are not diagnostically distinct below the glyph visibility bound"
    $orderedColorKeys = @($script:ExpectedBackgrounds | ForEach-Object {
        Get-ColorKey $GenesisSource $_
    })
    for ($index = 1; $index -lt $orderedColorKeys.Count; $index++) {
        Require ($orderedColorKeys[$index] -cne $orderedColorKeys[$index - 1]) "adjacent checkpoints collapse to the same color fallback"
    }

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
    [string]$CaptureSource,
    [string]$ReportId
) {
    $rejected = $false
    try {
        Assert-CheckpointSources $MainSource $GenesisSource $CaptureSource
    } catch {
        $rejected = $true
    }
    Require $rejected "$ReportId accepted a mutated source tree"
    Write-Output ($ReportId + ": PASS")
}

$mainPath = Join-Path $RepoRoot "seed-kernel\src\main.rs"
$genesisPath = Join-Path $RepoRoot "seed-kernel\src\shell_host\genesis.rs"
$capturePath = Join-Path $RepoRoot "seed-kernel\src\surface_fact_capture.rs"
$main = [IO.File]::ReadAllText($mainPath)
$genesis = [IO.File]::ReadAllText($genesisPath)
$capture = [IO.File]::ReadAllText($capturePath)

Assert-CheckpointSources $main $genesis $capture
Write-Output "SURFACE-EARLY-BOOT-BOUNDS: PASS"

# Negative boundary: mutate the complete source input and run the same source
# contract used for the real files.
$newCheckpointTokens = @(
    "    status_ui.render_early_boot_after_provider_config();",
    "    status_ui.render_early_boot_after_console();"
)
for ($index = 0; $index -lt $newCheckpointTokens.Count; $index++) {
    $mutationNumber = $index + 1
    $token = $newCheckpointTokens[$index]
    Require ([regex]::Matches($main, [regex]::Escape($token)).Count -eq 1) "new-checkpoint source token is not unique"
    $missingMain = $main.Replace($token, "")
    Assert-RejectedMutation $missingMain $genesis $capture "SURFACE-EARLY-BOOT-NEG-MISSING-NEW-$mutationNumber"
    $duplicateMain = $main.Replace(
        $token,
        $token + [Environment]::NewLine + $token
    )
    Assert-RejectedMutation $duplicateMain $genesis $capture "SURFACE-EARLY-BOOT-NEG-DUPLICATE-NEW-$mutationNumber"
}

$cpuidMappingToken = "status_ui.render_early_boot_capture_cpuid()"
$smbiosMappingToken = "status_ui.render_early_boot_capture_smbios()"
Require ([regex]::Matches($main, [regex]::Escape($cpuidMappingToken)).Count -eq 1) "CPUID main mapping token is not unique"
Require ([regex]::Matches($main, [regex]::Escape($smbiosMappingToken)).Count -eq 1) "SMBIOS main mapping token is not unique"
$missingMainMapping = $main.Replace($cpuidMappingToken, "()")
Require ($missingMainMapping -cne $main) "missing-main-mapping negative did not mutate the source"
Assert-RejectedMutation $missingMainMapping $genesis $capture "SURFACE-EARLY-BOOT-NEG-MAIN-MAPPING-MISSING"

$mainMappingSentinel = "status_ui.render_early_boot_capture_swap()"
Require (-not $main.Contains($mainMappingSentinel)) "main-mapping reorder sentinel already exists"
$reorderedMainMapping = $main.Replace($cpuidMappingToken, $mainMappingSentinel).
    Replace($smbiosMappingToken, $cpuidMappingToken).
    Replace($mainMappingSentinel, $smbiosMappingToken)
Require ($reorderedMainMapping -cne $main) "main-mapping reorder negative did not mutate the source"
Assert-RejectedMutation $reorderedMainMapping $genesis $capture "SURFACE-EARLY-BOOT-NEG-MAIN-MAPPING-REORDER"

$cpuidProgressToken = "progress(CapturePhase::Cpuid);"
$smbiosProgressToken = "progress(CapturePhase::Smbios);"
Require ([regex]::Matches($capture, [regex]::Escape($cpuidProgressToken)).Count -eq 1) "CPUID progress token is not unique"
Require ([regex]::Matches($capture, [regex]::Escape($smbiosProgressToken)).Count -eq 1) "SMBIOS progress token is not unique"
$missingCaptureProgress = $capture.Replace($cpuidProgressToken, "")
Require ($missingCaptureProgress -cne $capture) "missing-capture-progress negative did not mutate the source"
Assert-RejectedMutation $main $genesis $missingCaptureProgress "SURFACE-EARLY-BOOT-NEG-CAPTURE-CALL-MISSING"

$captureProgressSentinel = "progress(CapturePhase::SwapSentinel);"
Require (-not $capture.Contains($captureProgressSentinel)) "capture-progress reorder sentinel already exists"
$reorderedCaptureProgress = $capture.Replace($cpuidProgressToken, $captureProgressSentinel).
    Replace($smbiosProgressToken, $cpuidProgressToken).
    Replace($captureProgressSentinel, $smbiosProgressToken)
Require ($reorderedCaptureProgress -cne $capture) "capture-progress reorder negative did not mutate the source"
Assert-RejectedMutation $main $genesis $reorderedCaptureProgress "SURFACE-EARLY-BOOT-NEG-CAPTURE-CALL-REORDER"

$afterProviderName = "render_early_boot_after_provider_config"
$afterConsoleName = "render_early_boot_after_console"
$swapSentinel = "render_early_boot_swap_checkpoint"
Require (-not $main.Contains($swapSentinel)) "reorder-negative sentinel already exists"
$reorderedMain = $main.Replace($afterProviderName, $swapSentinel).
    Replace($afterConsoleName, $afterProviderName).
    Replace($swapSentinel, $afterConsoleName)
Require ($reorderedMain -cne $main) "reorder-negative did not mutate the source"
Assert-RejectedMutation $reorderedMain $genesis $capture "SURFACE-EARLY-BOOT-NEG-REORDER"

$collapsedErrorToken = "                status_ui.render_early_boot_persist_failed();"
Require ([regex]::Matches($main, [regex]::Escape($collapsedErrorToken)).Count -eq 1) "collapsed-error negative source token is not unique"
$collapsedErrorMain = $main.Replace(
    $collapsedErrorToken,
    "                status_ui.render_early_boot_after_persist();"
)
Require ($collapsedErrorMain -cne $main) "collapsed-error negative did not mutate the source"
Assert-RejectedMutation $collapsedErrorMain $genesis $capture "SURFACE-EARLY-BOOT-NEG-ERR-COLLAPSED"

Push-Location $RepoRoot
try {
    $rustDiff = @(git diff HEAD --unified=0 -- seed-kernel/src/main.rs seed-kernel/src/shell_host/genesis.rs seed-kernel/src/surface_fact_capture.rs)
    if ($LASTEXITCODE -ne 0) { throw "git diff HEAD failed" }
    $addedRust = @($rustDiff | Where-Object {
        $_.StartsWith("+", [StringComparison]::Ordinal) -and
        -not $_.StartsWith("+++", [StringComparison]::Ordinal)
    })
    $removedRust = @($rustDiff | Where-Object {
        $_.StartsWith("-", [StringComparison]::Ordinal) -and
        -not $_.StartsWith("---", [StringComparison]::Ordinal)
    })
    Require ($removedRust.Count -eq 0) "existing Rust capture operations, errors, or ordering changed"
    $forbiddenRust = '\bunsafe\b|\bpci::|\bwifi::|program_persistence::|structured_store|memory_store|usb::append|append_surface_fact_capture|reclog|msc_'
    Require (($addedRust -join [Environment]::NewLine) -notmatch $forbiddenRust) "Rust additions introduce unsafe, PCI, Wi-Fi, or persistence behavior"

    $expectedPaths = @(
        "scripts/test-surface-early-boot-checkpoints.ps1",
        "seed-kernel/src/main.rs",
        "seed-kernel/src/shell_host/genesis.rs",
        "seed-kernel/src/surface_fact_capture.rs"
    ) | Sort-Object
    $statusLines = @(git status --short --untracked-files=all)
    if ($LASTEXITCODE -ne 0) { throw "git status failed" }
    $actualPaths = @($statusLines | ForEach-Object {
        $_.Substring(3).Replace('\', '/')
    }) | Sort-Object
    Require ((@(Compare-Object $expectedPaths $actualPaths)).Count -eq 0) "dirty file set differs from the exact four-file lane"

    git diff HEAD --check -- seed-kernel/src/main.rs seed-kernel/src/shell_host/genesis.rs seed-kernel/src/surface_fact_capture.rs scripts/test-surface-early-boot-checkpoints.ps1
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
