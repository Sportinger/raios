param(
    [string]$OutputDirectory = "pages-dist",
    [string]$BuildVersion = $env:RAIOS_BUILD_VERSION
)

$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
$outputPath = if ([IO.Path]::IsPathRooted($OutputDirectory)) {
    [IO.Path]::GetFullPath($OutputDirectory)
} else {
    [IO.Path]::GetFullPath((Join-Path $repoRoot $OutputDirectory))
}
$repoPrefix = $repoRoot.TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
if (-not $outputPath.StartsWith($repoPrefix, [StringComparison]::OrdinalIgnoreCase)) {
    throw "Pages output must stay inside the repository: $outputPath"
}
if ($outputPath -eq $repoRoot) {
    throw "Refusing to use the repository root as Pages output"
}

if (Test-Path -LiteralPath $outputPath) {
    $resolvedOutput = (Resolve-Path -LiteralPath $outputPath).Path
    if (-not $resolvedOutput.StartsWith($repoPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to remove unexpected Pages output: $resolvedOutput"
    }
    Remove-Item -LiteralPath $resolvedOutput -Recurse -Force
}

New-Item -ItemType Directory -Path $outputPath | Out-Null
Copy-Item -LiteralPath (Join-Path $repoRoot "raios-ui-lab.html") -Destination (Join-Path $outputPath "index.html")

$uiOutput = Join-Path $outputPath "ui-lab"
New-Item -ItemType Directory -Path $uiOutput | Out-Null
$uiSource = Join-Path $repoRoot "ui-lab"
Copy-Item -LiteralPath (Join-Path $uiSource "lab.css") -Destination $uiOutput
foreach ($directory in @("core", "lab", "site", "surfaces")) {
    Copy-Item -LiteralPath (Join-Path $uiSource $directory) -Destination $uiOutput -Recurse
}

$assetsSource = Join-Path $uiSource "assets"
$assetsOutput = Join-Path $uiOutput "assets"
$siteAssets = Join-Path $assetsOutput "site"
$surfaceAssets = Join-Path $assetsOutput "surface"
$audioAssets = Join-Path $assetsOutput "audio"
New-Item -ItemType Directory -Path $siteAssets -Force | Out-Null
New-Item -ItemType Directory -Path $surfaceAssets -Force | Out-Null
New-Item -ItemType Directory -Path $audioAssets -Force | Out-Null
Copy-Item -LiteralPath (Join-Path (Join-Path $assetsSource "site") "fortnite.gif") -Destination $siteAssets
Copy-Item -LiteralPath (Join-Path (Join-Path $assetsSource "surface") "surface-photo-portrait-q88.webp") -Destination $surfaceAssets
Copy-Item -LiteralPath (Join-Path (Join-Path $assetsSource "surface") "surface-reflection-portrait-crop.webp") -Destination $surfaceAssets
Copy-Item -Path (Join-Path $assetsSource "audio\*") -Destination $audioAssets
Copy-Item -LiteralPath (Join-Path (Join-Path $repoRoot "cloudflare") "pages-worker.mjs") -Destination (Join-Path $outputPath "_worker.js")

$effectiveBuildVersion = $BuildVersion.Trim()
if (-not $effectiveBuildVersion) {
    $manifestLines = @(
        Get-ChildItem -LiteralPath $outputPath -Recurse -File |
            Sort-Object FullName |
            ForEach-Object {
                $relativePath = $_.FullName.Substring($outputPath.Length).TrimStart([char[]]"\/").Replace("\", "/")
                $fileHash = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
                "$relativePath=$fileHash"
            }
    )
    $manifestBytes = [Text.Encoding]::UTF8.GetBytes($manifestLines -join "`n")
    $sha256 = [Security.Cryptography.SHA256]::Create()
    try {
        $effectiveBuildVersion = ([BitConverter]::ToString($sha256.ComputeHash($manifestBytes))).Replace("-", "").ToLowerInvariant()
    } finally {
        $sha256.Dispose()
    }
}
$effectiveBuildVersion = [regex]::Replace($effectiveBuildVersion, '[^A-Za-z0-9._-]', '-').Trim('-')
if (-not $effectiveBuildVersion) {
    throw "Build version must contain at least one URL-safe character"
}
if ($effectiveBuildVersion.Length -gt 12) {
    $effectiveBuildVersion = $effectiveBuildVersion.Substring(0, 12)
}

$html = Get-Content -Raw -LiteralPath (Join-Path $outputPath "index.html")
$buildVersionMetaPattern = '(<meta\s+name="raios-build-version"\s+content=")[^"]*(">)'
$buildVersionMetaRegex = [regex]::new($buildVersionMetaPattern)
if (-not $buildVersionMetaRegex.IsMatch($html)) {
    throw "HTML is missing the raiOS build-version meta tag"
}
$html = $buildVersionMetaRegex.Replace(
    $html,
    "`$1$effectiveBuildVersion`$2",
    1
)
$html = [regex]::Replace(
    $html,
    '(?<prefix>\b(?:src|href)=")(?<url>[^"]+)(?<suffix>")',
    {
        param($match)
        $reference = $match.Groups['url'].Value
        if ($reference -match '^(?:https?:|#|data:|mailto:|javascript:)' ) {
            return $match.Value
        }
        $pathOnly = ($reference -split '[?#]', 2)[0]
        if (-not [IO.Path]::GetExtension($pathOnly)) {
            return $match.Value
        }
        $fragmentIndex = $reference.IndexOf('#')
        $referenceWithoutFragment = if ($fragmentIndex -ge 0) {
            $reference.Substring(0, $fragmentIndex)
        } else {
            $reference
        }
        $fragment = if ($fragmentIndex -ge 0) { $reference.Substring($fragmentIndex) } else { "" }
        $separator = if ($referenceWithoutFragment.Contains('?')) { '&' } else { '?' }
        $versionedReference = "$referenceWithoutFragment${separator}v=$effectiveBuildVersion$fragment"
        return "$($match.Groups['prefix'].Value)$versionedReference$($match.Groups['suffix'].Value)"
    }
)
[IO.File]::WriteAllText(
    (Join-Path $outputPath "index.html"),
    $html,
    [Text.UTF8Encoding]::new($false)
)
[IO.File]::WriteAllText(
    (Join-Path $outputPath "version.json"),
    "{`"version`":`"$effectiveBuildVersion`"}",
    [Text.UTF8Encoding]::new($false)
)
$references = @(
    [regex]::Matches($html, '(?:src|href)="([^"]+)"') |
        ForEach-Object { $_.Groups[1].Value } |
        Where-Object { $_ -notmatch '^(?:https?:|#|data:|mailto:|javascript:)' } |
        Sort-Object -Unique
)
$missingReferences = @(
    foreach ($reference in $references) {
        $relativePath = ($reference -split '[?#]')[0]
        if ($relativePath -and -not (Test-Path -LiteralPath (Join-Path $outputPath $relativePath))) {
            $reference
        }
    }
)
if ($missingReferences.Count -gt 0) {
    throw "Pages output is missing referenced assets: $($missingReferences -join ', ')"
}

$files = @(Get-ChildItem -LiteralPath $outputPath -Recurse -File)
$largest = $files | Sort-Object Length -Descending | Select-Object -First 1
if ($largest.Length -gt 25MB) {
    throw "Pages file exceeds the 25 MiB limit: $($largest.FullName)"
}

[pscustomobject]@{
    Output = $outputPath
    Files = $files.Count
    Bytes = ($files | Measure-Object -Property Length -Sum).Sum
    LargestFile = $largest.Name
    LargestBytes = $largest.Length
    ReferencedAssets = $references.Count
    BuildVersion = $effectiveBuildVersion
}
