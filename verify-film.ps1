param(
    [string]$HtmlPath = (Join-Path $PSScriptRoot "raios-ui-lab.html")
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$requiredTimes = @(3, 10, 20, 30, 38, 46, 57, 67, 76, 84, 92, 101, 110, 117)
$expectedScenes = @(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14)
$requiredComponents = @(
    "film-block",
    "film-door",
    "film-key",
    "film-vein",
    "film-capsule",
    "film-stamp",
    "film-ring-segment",
    "film-gate",
    "film-crystal"
)
$emDash = [char]0x2014
$middleDot = [char]0x00B7
$notEqual = [char]0x2260
$requiredCopy = @(
    "In two minutes",
    "The factory moves in.",
    "> build me a music player",
    ("OWNS CPU {0} RAM {0} USB {0} DISPLAY {0} NET" -f $middleDot),
    ("BUILDS ABOVE KERNEL {0} GRANTS DOORS" -f $middleDot),
    ("RUST-KERNEL {0} tr{1}gt beide B{2}den" -f $emDash, [char]0x00E4, [char]0x00F6),
    "GENESIS DECK",
    "BUILDER DECK",
    "build.request",
    "ALL BUILD DOORS UNLOCKED",
    "KEY GRANTED: NET.HTTPS",
    ("the connection stays {0} and stays visible" -f $emDash),
    "NO NET SOCKET.",
    "BUILDS ARE OFFLINE BY DESIGN.",
    ("HASHED {0} CONTENT-ADDRESSED {0} IMMUTABLE" -f $middleDot),
    ("STORED {0} EXECUTABLE" -f $notEqual),
    "rustc 1.83.0-dev",
    "Cargo.lock missing",
    "EQUAL",
    "NO SUCH DOOR",
    "Approve + run program",
    "REMOTE START DENIED",
    ("F12 {0} REVOKE ALL" -f $middleDot),
    "TOMBSTONE",
    ("raiOS {0} the OS that builds its own software." -f $emDash),
    "Every gate you just saw is a real test."
)

function Assert-Condition {
    param(
        [Parameter(Mandatory = $true)]
        [bool]$Condition,
        [Parameter(Mandatory = $true)]
        [string]$Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

function Get-AttributeValue {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Tag,
        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    $attributePattern = '(?is)\b' + [regex]::Escape($Name) + '\s*=\s*(["''])(.*?)\1'
    $match = [regex]::Match($Tag, $attributePattern)
    if (-not $match.Success) {
        return $null
    }
    return [Net.WebUtility]::HtmlDecode($match.Groups[2].Value)
}

function Get-ElementTagById {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Document,
        [Parameter(Mandatory = $true)]
        [string]$TagName,
        [Parameter(Mandatory = $true)]
        [string]$Id
    )

    $pattern = '(?is)<' + [regex]::Escape($TagName) + '\b(?=[^>]*\bid\s*=\s*(["''])' +
        [regex]::Escape($Id) + '\1)[^>]*>'
    $match = [regex]::Match($Document, $pattern)
    if (-not $match.Success) {
        return $null
    }
    return $match.Value
}

function Test-InTemplate {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Document,
        [Parameter(Mandatory = $true)]
        [int]$Offset
    )

    $prefix = $Document.Substring(0, $Offset)
    $openCount = [regex]::Matches($prefix, '(?is)<template\b').Count
    $closeCount = [regex]::Matches($prefix, '(?is)</template\s*>').Count
    return $openCount -gt $closeCount
}

function Get-EdgePath {
    $candidates = @(
        "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        "C:\Program Files\Microsoft\Edge\Application\msedge.exe"
    )
    foreach ($candidate in $candidates) {
        if (Test-Path -LiteralPath $candidate -PathType Leaf) {
            return $candidate
        }
    }
    throw "Microsoft Edge was not found"
}

$resolvedHtml = (Resolve-Path -LiteralPath $HtmlPath).Path
$filmScript = Join-Path (Split-Path -Parent $resolvedHtml) "ui-lab\site\film.js"
Assert-Condition (Test-Path -LiteralPath $filmScript -PathType Leaf) "Missing film timeline: $filmScript"

$strictUtf8 = [Text.UTF8Encoding]::new($false, $true)
$htmlBytes = [IO.File]::ReadAllBytes($resolvedHtml)
$html = $strictUtf8.GetString($htmlBytes)
$filmJs = $strictUtf8.GetString([IO.File]::ReadAllBytes($filmScript))

$sectionPattern = '(?is)<section\b[^>]*class\s*=\s*(["''])[^"'']*\bwebsite-story\b[^"'']*\1[^>]*>'
$storySections = [regex]::Matches($html, $sectionPattern)
Assert-Condition ($storySections.Count -ge 2) "Expected the film and legacy opener as the first two website story sections"

$firstStoryTag = $storySections[0].Value
$secondStoryTag = $storySections[1].Value
Assert-Condition ($firstStoryTag -match '(?i)\bstory-opener\b') "The active film is not the first website story section"
Assert-Condition ($firstStoryTag -match '(?i)\bfilm-section\b') "The first website story section is not marked as the active film"
Assert-Condition ($secondStoryTag -match '(?i)\bstory-opener-legacy\b') "The legacy opener is not the second website story section"
Assert-Condition (-not (Test-InTemplate $html $storySections[0].Index)) "The active film is inside a template"
Assert-Condition (-not (Test-InTemplate $html $storySections[1].Index)) "The legacy opener is inside a template"

$filmEnd = $html.IndexOf('</section>', $storySections[0].Index, [StringComparison]::OrdinalIgnoreCase)
Assert-Condition ($filmEnd -gt $storySections[0].Index) "The active film section has no closing tag"
$filmSection = $html.Substring($storySections[0].Index, $filmEnd + '</section>'.Length - $storySections[0].Index)
$filmSectionBytes = [Text.Encoding]::UTF8.GetByteCount($filmSection)
Assert-Condition ($filmSectionBytes -le (450 * 1024)) "Film section is $filmSectionBytes bytes; limit is 460800 bytes"

$sceneMatches = [regex]::Matches(
    $filmSection,
    '(?is)<g\b(?=[^>]*\bid\s*=\s*(["''])film-scene-(\d{1,2})\1)[^>]*>'
)
Assert-Condition ($sceneMatches.Count -eq 14) "Expected exactly 14 film scene groups, found $($sceneMatches.Count)"
$sceneNumbers = @($sceneMatches | ForEach-Object { [int]$_.Groups[2].Value } | Sort-Object)
Assert-Condition (($sceneNumbers -join ',') -eq (($expectedScenes | Sort-Object) -join ',')) "Film scene IDs must be exactly film-scene-1 through film-scene-14"

$smilMatches = [regex]::Matches($filmSection, '(?is)<\s*(animate|animateMotion|animateTransform|set)\b')
Assert-Condition ($smilMatches.Count -eq 0) "The active film contains $($smilMatches.Count) SMIL elements"
Assert-Condition ($filmSection -match '(?is)<defs\b.*?</defs>') "The active film has no defs block"
Assert-Condition ($filmSection -match '(?is)<g\b[^>]*\bid\s*=\s*(["''])(cam|film-cam)\1') "The active film has no camera rig"

foreach ($component in $requiredComponents) {
    $componentPattern = '(?is)<symbol\b(?=[^>]*\bid\s*=\s*(["''])' + [regex]::Escape($component) + '\1)[^>]*>'
    Assert-Condition ([regex]::IsMatch($filmSection, $componentPattern)) "Missing film component symbol: $component"
}

$decodedFilm = [Net.WebUtility]::HtmlDecode($filmSection)
foreach ($copy in $requiredCopy) {
    Assert-Condition ($decodedFilm.Contains($copy)) "Missing required film copy: $copy"
}

$filmScriptReferences = [regex]::Matches(
    $html,
    '(?is)<script\b[^>]*\bsrc\s*=\s*(["''])ui-lab/site/film\.js\1[^>]*>\s*</script>'
)
Assert-Condition ($filmScriptReferences.Count -eq 1) "Expected exactly one ui-lab/site/film.js script reference"
Assert-Condition ($filmJs -match '(?m)^\s*const DURATION\s*=\s*120\s*;') "film.js does not declare the 120-second master duration"
Assert-Condition ($filmJs -match '(?m)^\s*const POSTER_TIME\s*=\s*118\s*;') "film.js does not declare the t=118 poster frame"
Assert-Condition ($filmJs.Contains('requestAnimationFrame')) "film.js does not use requestAnimationFrame"
Assert-Condition ($filmJs.Contains('prefers-reduced-motion')) "film.js does not implement reduced-motion handling"

$legacyEnd = $html.IndexOf('</section>', $storySections[1].Index, [StringComparison]::OrdinalIgnoreCase)
Assert-Condition ($legacyEnd -gt $storySections[1].Index) "The legacy opener section has no closing tag"
$legacySection = $html.Substring($storySections[1].Index, $legacyEnd + '</section>'.Length - $storySections[1].Index)
Assert-Condition ($legacySection -match '(?is)<svg\b[^>]*class\s*=\s*(["''])[^"'']*\baib-svg\b[^"'']*\1') "The second story section does not contain the legacy .aib-svg opener"
Assert-Condition ($secondStoryTag -notmatch '(?i)\bhidden\b') "The legacy opener section is hidden"
Assert-Condition ($secondStoryTag -notmatch '(?i)aria-hidden\s*=\s*(["''])true\1') "The legacy opener section is aria-hidden"
Assert-Condition ($secondStoryTag -notmatch '(?i)style\s*=\s*(["''])[^"'']*(display\s*:\s*none|visibility\s*:\s*hidden)') "The legacy opener section is hidden by inline style"

$edge = Get-EdgePath
$tempRoot = Join-Path $env:TEMP ("raios-film-verify-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $tempRoot | Out-Null
$baseUrl = "file:///" + $resolvedHtml.Replace('\', '/')
$runtimeChecks = [Collections.Generic.List[object]]::new()

function Get-DomSnapshot {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Label,
        [Parameter(Mandatory = $true)]
        [string]$Query,
        [switch]$ForceReducedMotion
    )

    $profile = Join-Path $tempRoot ("profile-" + $Label)
    New-Item -ItemType Directory -Path $profile | Out-Null
    $stdout = Join-Path $tempRoot ($Label + ".dom.html")
    $stderr = Join-Path $tempRoot ($Label + ".stderr.txt")
    $url = $baseUrl
    if ($Query) {
        $url += "?" + $Query
    }
    $arguments = @(
        "--headless=new",
        "--no-first-run",
        "--disable-gpu",
        "--hide-scrollbars",
        "--user-data-dir=$profile",
        "--dump-dom"
    )
    if ($ForceReducedMotion) {
        $arguments += "--force-prefers-reduced-motion=reduce"
    }
    $arguments += $url

    $process = Start-Process `
        -FilePath $edge `
        -ArgumentList $arguments `
        -WindowStyle Hidden `
        -Wait `
        -PassThru `
        -RedirectStandardOutput $stdout `
        -RedirectStandardError $stderr
    if ($process.ExitCode -ne 0) {
        $detail = if (Test-Path -LiteralPath $stderr) {
            Get-Content -Raw -LiteralPath $stderr
        } else {
            "no stderr"
        }
        throw "Edge DOM load '$Label' failed with exit $($process.ExitCode): $detail"
    }
    Assert-Condition (Test-Path -LiteralPath $stdout -PathType Leaf) "Edge produced no DOM for '$Label'"
    return Get-Content -Raw -LiteralPath $stdout -Encoding utf8
}

function Assert-RuntimeFrame {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Dom,
        [Parameter(Mandatory = $true)]
        [string]$Label,
        [Parameter(Mandatory = $true)]
        [double]$ExpectedTime,
        [Parameter(Mandatory = $true)]
        [int]$ExpectedScene
    )

    $selftestTag = Get-ElementTagById $Dom "output" "lab-selftest"
    Assert-Condition ($null -ne $selftestTag) "Runtime '$Label': missing lab-selftest"
    $selftestStatus = Get-AttributeValue $selftestTag "data-status"
    Assert-Condition ($selftestStatus -eq "pass") "Runtime '$Label': lab selftest is '$selftestStatus'"

    $filmTag = Get-ElementTagById $Dom "svg" "film-svg"
    Assert-Condition ($null -ne $filmTag) "Runtime '$Label': missing #film-svg"
    $scene = Get-AttributeValue $filmTag "data-film-scene"
    $sceneId = Get-AttributeValue $filmTag "data-film-scene-id"
    $playing = Get-AttributeValue $filmTag "data-film-playing"
    Assert-Condition ($scene -eq [string]$ExpectedScene) "Runtime '$Label': expected scene $ExpectedScene, got '$scene'"
    Assert-Condition (-not [string]::IsNullOrWhiteSpace($sceneId)) "Runtime '$Label': missing data-film-scene-id"
    Assert-Condition ($playing -eq "false") "Runtime '$Label': deterministic/reduced frame is playing"

    $style = Get-AttributeValue $filmTag "style"
    Assert-Condition (-not [string]::IsNullOrWhiteSpace($style)) "Runtime '$Label': missing rendered film style state"
    $timeMatch = [regex]::Match($style, '(?i)--film-time\s*:\s*([0-9]+(?:\.[0-9]+)?)')
    Assert-Condition $timeMatch.Success "Runtime '$Label': missing --film-time state"
    $renderedTime = [double]::Parse($timeMatch.Groups[1].Value, [Globalization.CultureInfo]::InvariantCulture)
    Assert-Condition ([Math]::Abs($renderedTime - $ExpectedTime) -le 0.001) "Runtime '$Label': expected t=$ExpectedTime, got t=$renderedTime"

    $activeSceneTags = [regex]::Matches(
        $Dom,
        '(?is)<g\b(?=[^>]*\bid\s*=\s*(["''])film-scene-(\d{1,2})\1)(?=[^>]*\bdata-film-active\s*=\s*(["''])true\3)[^>]*>'
    )
    Assert-Condition ($activeSceneTags.Count -eq 1) "Runtime '$Label': expected one active scene group, found $($activeSceneTags.Count)"
    $activeScene = [int]$activeSceneTags[0].Groups[2].Value
    Assert-Condition ($activeScene -eq $ExpectedScene) "Runtime '$Label': active SVG group is scene $activeScene, expected $ExpectedScene"

    $hudTag = Get-ElementTagById $Dom "text" "film-scene-number"
    Assert-Condition ($null -ne $hudTag) "Runtime '$Label': missing film scene HUD"
    $hudPattern = '(?is)' + [regex]::Escape($hudTag) + '\s*0?' + [string]$ExpectedScene + '\s*/\s*14\s*</text>'
    Assert-Condition ([regex]::IsMatch($Dom, $hudPattern)) "Runtime '$Label': HUD does not report scene $ExpectedScene / 14"

    $runtimeChecks.Add([pscustomobject]@{
        Label = $Label
        Time = $renderedTime
        Scene = $activeScene
        SceneId = $sceneId
        Selftest = $selftestStatus
    }) | Out-Null
}

try {
    for ($index = 0; $index -lt $requiredTimes.Count; $index += 1) {
        $time = $requiredTimes[$index]
        $scene = $expectedScenes[$index]
        $label = "p7-{0:D3}" -f $time
        $dom = Get-DomSnapshot -Label $label -Query "site=1&anim=0&animt=$time"
        Assert-RuntimeFrame -Dom $dom -Label $label -ExpectedTime $time -ExpectedScene $scene
    }

    $posterDom = Get-DomSnapshot -Label "poster-118" -Query "site=1&anim=0&animt=118"
    Assert-RuntimeFrame -Dom $posterDom -Label "poster-118" -ExpectedTime 118 -ExpectedScene 14

    $reducedDom = Get-DomSnapshot -Label "reduced-motion" -Query "site=1" -ForceReducedMotion
    Assert-RuntimeFrame -Dom $reducedDom -Label "reduced-motion" -ExpectedTime 118 -ExpectedScene 14

    [pscustomobject]@{
        Predicate = "film-masterplan-p7"
        Status = "PASS"
        FilmBytes = $filmSectionBytes
        FilmLimitBytes = 450 * 1024
        SceneGroups = $sceneMatches.Count
        P7Frames = $requiredTimes.Count
        PosterFrame = 118
        ReducedMotionFrame = 118
        LegacyOpener = "second-visible-story-section"
        RuntimeSelftests = ($runtimeChecks | Where-Object { $_.Selftest -eq "pass" }).Count
    }
}
finally {
    if (Test-Path -LiteralPath $tempRoot -PathType Container) {
        $resolvedTemp = (Resolve-Path -LiteralPath $env:TEMP).Path
        $resolvedVerify = (Resolve-Path -LiteralPath $tempRoot).Path
        $expectedPrefix = $resolvedTemp + [IO.Path]::DirectorySeparatorChar + "raios-film-verify-"
        if (-not $resolvedVerify.StartsWith($expectedPrefix, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing to remove unexpected verification path: $resolvedVerify"
        }
        Remove-Item -LiteralPath $resolvedVerify -Recurse -Force
    }
}
