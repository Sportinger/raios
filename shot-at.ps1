param(
    # Virtueller Zeitpunkt in ms nach Navigationsstart, an dem der Shot landet.
    # Chromium laesst Timer/rAF/CSS-Animationen deterministisch bis zu diesem
    # Punkt laufen (--virtual-time-budget) und feuert dann den Screenshot.
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateRange(0, 120000)]
    [int]$AtMs,
    [string]$Query = "site=1",
    [string]$Out = "shot-at.png",
    [ValidateRange(320, 7680)]
    [int]$Width = 1920,
    [ValidateRange(720, 4320)]
    [int]$Height = 1080,
    [switch]$IncludeLabChrome
)
$ErrorActionPreference = "Stop"
# Zeitpraeziser Screenshot des UI-Labs im headless Edge, ohne offenen Browser.
# Beispiele:  .\shot-at.ps1 -AtMs 0
#             .\shot-at.ps1 -AtMs 1200 -Query "site=1" -Out shot-1200.jpg
#             .\shot-at.ps1 2500 -Query "site=1&scenario=genesis.dream.chat"
# Hinweis: --virtual-time-budget fast-forwarded die Page-Uhr; Netzwerk pausiert
# die virtuelle Zeit, d.h. der Shot ist reproduzierbar am selben Zeitpunkt.
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
# Edge schreibt bei --screenshot immer PNG; fuer .jpg/.jpeg wird konvertiert.
$wantJpeg = $outPath -match '\.jpe?g$'
$capturePath = $outPath
if ($wantJpeg) { $capturePath = [IO.Path]::ChangeExtension($outPath, ".png") + ".tmp.png" }
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
        "--virtual-time-budget=$AtMs",
        "--user-data-dir=$shotRoot",
        "--screenshot=$capturePath",
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
    if (-not (Test-Path -LiteralPath $capturePath -PathType Leaf)) {
        throw "Edge did not create the requested screenshot"
    }
    if ($wantJpeg) {
        Add-Type -AssemblyName System.Drawing
        $image = [System.Drawing.Image]::FromFile($capturePath)
        try {
            $image.Save($outPath, [System.Drawing.Imaging.ImageFormat]::Jpeg)
        }
        finally {
            $image.Dispose()
            Remove-Item -LiteralPath $capturePath -Force
        }
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
