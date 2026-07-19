param(
    [string]$Query = "",
    [string]$Out = "shot.png",
    [ValidateRange(320, 7680)]
    [int]$Width = 1920,
    [ValidateRange(720, 4320)]
    [int]$Height = 1080,
    [switch]$IncludeLabChrome
)
$ErrorActionPreference = "Stop"
# Screenshot des UI-Labs im headless Edge.
# Beispiele:  .\shot.ps1                              -> Grundzustand
#             .\shot.ps1 -Query "scenario=genesis.dream.chat&chat=demo"
#             .\shot.ps1 -Query "scenario=wifi.selecting"
#             .\shot.ps1 -Query "scenario=vault.managing"
#             .\shot.ps1 -Query "scenario=personal.editor"
$edge = "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
if (-not (Test-Path $edge)) { $edge = "C:\Program Files\Microsoft\Edge\Application\msedge.exe" }
$html = Join-Path $PSScriptRoot "raios-ui-lab.html"
$url = "file:///" + $html.Replace('\', '/')
$queryParts = @()
if ($Query) { $queryParts += $Query }
if (-not $IncludeLabChrome -and $Query -notmatch '(^|&)chrome=') { $queryParts += "chrome=0" }
if ($queryParts.Count -gt 0) { $url += "?" + ($queryParts -join "&") }
$outPath = if ([IO.Path]::IsPathRooted($Out)) {
    [IO.Path]::GetFullPath($Out)
} else {
    [IO.Path]::GetFullPath((Join-Path (Get-Location) $Out))
}
$outParent = Split-Path -Parent $outPath
if (-not (Test-Path -LiteralPath $outParent -PathType Container)) {
    throw "Screenshot output directory does not exist: $outParent"
}
$shotRoot = Join-Path $env:TEMP ("raios-ui-lab-shot-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $shotRoot | Out-Null
try {
    $arguments = @(
        "--headless=new",
        "--no-first-run",
        "--disable-gpu",
        "--hide-scrollbars",
        "--force-device-scale-factor=1",
        "--window-size=$Width,$Height",
        "--user-data-dir=$shotRoot",
        "--screenshot=$outPath",
        $url
    )
    $process = Start-Process `
        -FilePath $edge `
        -ArgumentList $arguments `
        -WindowStyle Hidden `
        -Wait `
        -PassThru
    if ($process.ExitCode -ne 0) {
        throw "Edge screenshot failed with exit $($process.ExitCode)"
    }
    if (-not (Test-Path -LiteralPath $outPath -PathType Leaf)) {
        throw "Edge did not create the requested screenshot"
    }
    Get-Item -LiteralPath $outPath | Select-Object Length, FullName
}
finally {
    if (Test-Path -LiteralPath $shotRoot) {
        $resolvedTemp = (Resolve-Path -LiteralPath $env:TEMP).Path
        $resolvedShot = (Resolve-Path -LiteralPath $shotRoot).Path
        $expectedPrefix = $resolvedTemp + [IO.Path]::DirectorySeparatorChar + "raios-ui-lab-shot-"
        if (-not $resolvedShot.StartsWith($expectedPrefix, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing to remove unexpected screenshot profile: $resolvedShot"
        }
        Remove-Item -LiteralPath $resolvedShot -Recurse -Force
    }
}
