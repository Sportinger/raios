param(
    [string]$OutputDirectory = "pages-dist"
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
New-Item -ItemType Directory -Path $siteAssets -Force | Out-Null
New-Item -ItemType Directory -Path $surfaceAssets -Force | Out-Null
Copy-Item -LiteralPath (Join-Path (Join-Path $assetsSource "site") "fortnite.gif") -Destination $siteAssets
Copy-Item -LiteralPath (Join-Path (Join-Path $assetsSource "surface") "surface-photo-portrait-q88.webp") -Destination $surfaceAssets
Copy-Item -LiteralPath (Join-Path (Join-Path $assetsSource "surface") "surface-reflection-portrait-crop.webp") -Destination $surfaceAssets
Copy-Item -LiteralPath (Join-Path (Join-Path $repoRoot "cloudflare") "pages-worker.mjs") -Destination (Join-Path $outputPath "_worker.js")

$html = Get-Content -Raw -LiteralPath (Join-Path $outputPath "index.html")
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
}
