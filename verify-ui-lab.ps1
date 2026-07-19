param(
    [string]$Query = "suite=all&scenario=genesis.dream.closed"
)

$ErrorActionPreference = "Stop"

$edge = "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
if (-not (Test-Path -LiteralPath $edge -PathType Leaf)) {
    $edge = "C:\Program Files\Microsoft\Edge\Application\msedge.exe"
}
if (-not (Test-Path -LiteralPath $edge -PathType Leaf)) {
    throw "Microsoft Edge was not found"
}

$verifyRoot = Join-Path $env:TEMP ("raios-ui-lab-verify-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $verifyRoot | Out-Null

try {
    $stdout = Join-Path $verifyRoot "dom.txt"
    $stderr = Join-Path $verifyRoot "edge.err.txt"
    $html = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "raios-ui-lab.html")).Path
    $url = "file:///" + $html.Replace('\', '/')
    if ($Query) {
        $url += "?" + $Query
    }
    $arguments = @(
        "--headless=new",
        "--no-first-run",
        "--disable-gpu",
        "--user-data-dir=$verifyRoot",
        "--dump-dom",
        $url
    )
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
        throw "Edge verification failed with exit $($process.ExitCode): $detail"
    }

    $dom = Get-Content -Raw -LiteralPath $stdout
    $match = [regex]::Match(
        $dom,
        '<output id="lab-selftest" data-status="([^"]+)">([^<]*)</output>'
    )
    if (-not $match.Success) {
        throw "UI Lab selftest marker is missing"
    }
    if ($match.Groups[1].Value -ne "pass") {
        throw "UI Lab selftest failed: $($match.Groups[2].Value)"
    }

    $canvasMatch = [regex]::Match(
        $dom,
        '<canvas id="screen" width="(\d+)" height="(\d+)"'
    )
    if (-not $canvasMatch.Success) {
        throw "UI Lab canvas dimensions are missing"
    }
    $physicalWidth = [int]$canvasMatch.Groups[1].Value
    $physicalHeight = [int]$canvasMatch.Groups[2].Value

    [pscustomobject]@{
        Predicate = if ($Query -match '(^|&)suite=all(&|$)') { "structured-ui-mirror" } else { "geometry-parity" }
        Status = "PASS"
        Detail = $match.Groups[2].Value
        Physical = "${physicalWidth}x${physicalHeight}"
        Logical = "$([math]::Floor($physicalWidth / 2))x$([math]::Floor($physicalHeight / 2))"
    }
}
finally {
    if (Test-Path -LiteralPath $verifyRoot) {
        $resolvedTemp = (Resolve-Path -LiteralPath $env:TEMP).Path
        $resolvedVerify = (Resolve-Path -LiteralPath $verifyRoot).Path
        $expectedPrefix = $resolvedTemp + [IO.Path]::DirectorySeparatorChar + "raios-ui-lab-verify-"
        if (-not $resolvedVerify.StartsWith($expectedPrefix, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing to remove unexpected verification path: $resolvedVerify"
        }
        Remove-Item -LiteralPath $resolvedVerify -Recurse -Force
    }
}
