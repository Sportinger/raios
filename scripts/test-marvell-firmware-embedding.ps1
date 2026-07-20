param(
    [Parameter(Mandatory = $true)]
    [string]$FirmwarePath,
    [string]$WorkRoot = (Join-Path $env:TEMP ("raios-marvell-firmware-{0}" -f [Guid]::NewGuid().ToString("N"))),
    [string]$CargoHome = ""
)

$ErrorActionPreference = "Stop"

$ExpectedLength = 723540
$ExpectedSha256 = "CF4F51F41BD7EF4D7FE65FB76B8A2A0897BC70A0742BC4AEA13D93B03FFFD03A"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$PackageScript = Join-Path $PSScriptRoot "package-stage0.ps1"
$FirmwareFullPath = [IO.Path]::GetFullPath($FirmwarePath)
$WorkRoot = [IO.Path]::GetFullPath($WorkRoot)
$TargetRoot = Join-Path $WorkRoot "target"
$KernelPath = Join-Path $TargetRoot "x86_64-seed\release\seed-kernel"
$MissingKeyPath = Join-Path $WorkRoot "deliberately-absent-core-policy-owner.secret"
$ReportId = "H15-BUILD-EMBED-{0}" -f [Guid]::NewGuid().ToString("N")
$Results = [ordered]@{}

function Assert-Condition {
    param(
        [bool]$Condition,
        [string]$Message
    )
    if (-not $Condition) {
        throw $Message
    }
}

function Invoke-PackageCase {
    param(
        [string]$Name,
        [string[]]$Arguments,
        [switch]$ExpectFailure,
        [string]$ExpectedLogFragment = ""
    )

    $stdoutPath = Join-Path $WorkRoot "$Name.stdout.log"
    $stderrPath = Join-Path $WorkRoot "$Name.stderr.log"
    $oldErrorActionPreference = $ErrorActionPreference
    try {
        # Windows PowerShell surfaces native stderr as ErrorRecord objects.
        # Let the child exit code and retained logs define each case verdict.
        $ErrorActionPreference = "Continue"
        & powershell -NoProfile -ExecutionPolicy Bypass -File $PackageScript @Arguments `
            1> $stdoutPath 2> $stderrPath
        $exitCode = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $oldErrorActionPreference
    }
    if ($ExpectFailure) {
        Assert-Condition ($exitCode -ne 0) `
            "$Name unexpectedly succeeded. Logs: $stdoutPath, $stderrPath"
    }
    else {
        Assert-Condition ($exitCode -eq 0) `
            "$Name exited $exitCode. Logs: $stdoutPath, $stderrPath"
    }

    if (-not [string]::IsNullOrWhiteSpace($ExpectedLogFragment)) {
        $logText = ((Get-Content -LiteralPath $stdoutPath -Raw -ErrorAction SilentlyContinue) +
            (Get-Content -LiteralPath $stderrPath -Raw -ErrorAction SilentlyContinue))
        Assert-Condition ($logText.Contains($ExpectedLogFragment)) `
            "$Name did not report the expected failure '$ExpectedLogFragment'."
    }

    $Results[$Name] = [ordered]@{
        exit_code = $exitCode
        stdout = $stdoutPath
        stderr = $stderrPath
    }
}

function Get-PatternCount {
    param(
        [string]$ArtifactPath,
        [string]$PatternPath
    )
    $count = & python -c `
        "import pathlib,sys; print(pathlib.Path(sys.argv[1]).read_bytes().count(pathlib.Path(sys.argv[2]).read_bytes()))" `
        $ArtifactPath $PatternPath
    if ($LASTEXITCODE -ne 0) {
        throw "Binary pattern scan failed for $ArtifactPath"
    }
    return [int]$count
}

function Get-PathCount {
    param(
        [string]$ArtifactPath,
        [string]$HostPath
    )
    $count = & python -c `
        "import pathlib,sys; d=pathlib.Path(sys.argv[1]).read_bytes(); p=sys.argv[2]; print(d.count(p.encode('utf-8')) + d.count(p.encode('utf-16le')))" `
        $ArtifactPath $HostPath
    if ($LASTEXITCODE -ne 0) {
        throw "Host-path scan failed for $ArtifactPath"
    }
    return [int]$count
}

function Get-LatestBuildScriptOutput {
    $buildRoot = Join-Path $TargetRoot "x86_64-seed\release\build"
    $output = Get-ChildItem -LiteralPath $buildRoot -Directory -Filter "seed-kernel-*" |
        ForEach-Object { Join-Path $_.FullName "output" } |
        Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } |
        Get-Item |
        Sort-Object LastWriteTimeUtc -Descending |
        Select-Object -First 1
    Assert-Condition ($null -ne $output) "Cargo did not retain a seed-kernel build-script output file."
    return $output.FullName
}

if (-not (Test-Path -LiteralPath $FirmwareFullPath -PathType Leaf)) {
    throw "Pinned Marvell firmware is missing: $FirmwareFullPath"
}
if (Test-Path -LiteralPath $WorkRoot) {
    throw "Refusing pre-existing fixture work root: $WorkRoot"
}

$firmwareItem = Get-Item -LiteralPath $FirmwareFullPath
Assert-Condition ($firmwareItem.Length -eq $ExpectedLength) `
    "Pinned Marvell firmware length is $($firmwareItem.Length), expected $ExpectedLength."
$firmwareSha256 = (Get-FileHash -LiteralPath $FirmwareFullPath -Algorithm SHA256).Hash
Assert-Condition ($firmwareSha256 -eq $ExpectedSha256) `
    "Pinned Marvell firmware SHA-256 does not match the repository predicate."

New-Item -ItemType Directory -Path $WorkRoot | Out-Null
$wrongLengthPath = Join-Path $WorkRoot "wrong-length.bin"
$wrongShaPath = Join-Path $WorkRoot "wrong-sha.bin"
$missingPath = Join-Path $WorkRoot "missing.bin"
[IO.File]::WriteAllBytes($wrongLengthPath, [byte[]]@(0))
$wrongShaBytes = [IO.File]::ReadAllBytes($FirmwareFullPath)
$wrongShaBytes[0] = $wrongShaBytes[0] -bxor 0xff
[IO.File]::WriteAllBytes($wrongShaPath, $wrongShaBytes)

$oldCargoTargetDir = $env:CARGO_TARGET_DIR
$oldCargoHome = $env:CARGO_HOME
$oldRequireMarvellFirmware = $env:RAIOS_REQUIRE_MARVELL_FIRMWARE
$oldMarvellFirmwarePath = $env:RAIOS_MARVELL_FIRMWARE_PATH

try {
    $env:CARGO_TARGET_DIR = $TargetRoot
    if (-not [string]::IsNullOrWhiteSpace($CargoHome)) {
        $env:CARGO_HOME = [IO.Path]::GetFullPath($CargoHome)
    }

    $commonArgs = @(
        "-Profile", "release",
        "-UseTempEsp",
        "-CorePolicyPrivateKey", $MissingKeyPath
    )

    $missingImage = Join-Path $WorkRoot "missing.img"
    Invoke-PackageCase -Name "missing" -ExpectFailure `
        -ExpectedLogFragment "required Marvell firmware could not be read" `
        -Arguments ($commonArgs + @(
            "-Image", $missingImage,
            "-RequireMarvellFirmware",
            "-MarvellFirmwarePath", $missingPath
        ))
    Assert-Condition (-not (Test-Path -LiteralPath $KernelPath)) `
        "The missing-firmware case produced a kernel artifact."
    Assert-Condition (-not (Test-Path -LiteralPath $missingImage)) `
        "The missing-firmware case produced an image."

    $wrongLengthImage = Join-Path $WorkRoot "wrong-length.img"
    Invoke-PackageCase -Name "wrong-length" -ExpectFailure `
        -ExpectedLogFragment "required Marvell firmware length mismatch" `
        -Arguments ($commonArgs + @(
            "-Image", $wrongLengthImage,
            "-RequireMarvellFirmware",
            "-MarvellFirmwarePath", $wrongLengthPath
        ))
    Assert-Condition (-not (Test-Path -LiteralPath $KernelPath)) `
        "The wrong-length case produced a kernel artifact."
    Assert-Condition (-not (Test-Path -LiteralPath $wrongLengthImage)) `
        "The wrong-length case produced an image."

    $wrongShaImage = Join-Path $WorkRoot "wrong-sha.img"
    Invoke-PackageCase -Name "wrong-sha" -ExpectFailure `
        -ExpectedLogFragment "required Marvell firmware SHA-256 mismatch" `
        -Arguments ($commonArgs + @(
            "-Image", $wrongShaImage,
            "-RequireMarvellFirmware",
            "-MarvellFirmwarePath", $wrongShaPath
        ))
    Assert-Condition (-not (Test-Path -LiteralPath $KernelPath)) `
        "The wrong-SHA case produced a kernel artifact."
    Assert-Condition (-not (Test-Path -LiteralPath $wrongShaImage)) `
        "The wrong-SHA case produced an image."

    $embeddedImage = Join-Path $WorkRoot "embedded.img"
    Invoke-PackageCase -Name "embedded" -Arguments ($commonArgs + @(
        "-Image", $embeddedImage,
        "-RequireMarvellFirmware",
        "-MarvellFirmwarePath", $FirmwareFullPath
    ))
    Assert-Condition (Test-Path -LiteralPath $KernelPath -PathType Leaf) `
        "The valid firmware case did not produce a kernel."
    Assert-Condition (Test-Path -LiteralPath $embeddedImage -PathType Leaf) `
        "The valid firmware case did not produce an image."
    Assert-Condition ((Get-PatternCount $KernelPath $FirmwareFullPath) -eq 1) `
        "The valid firmware was not embedded exactly once in the kernel."
    Assert-Condition ((Get-PatternCount $embeddedImage $FirmwareFullPath) -eq 1) `
        "The valid firmware was not packaged exactly once in the image."
    Assert-Condition ((Get-PathCount $KernelPath $FirmwareFullPath) -eq 0) `
        "The local firmware input path leaked into the kernel."
    Assert-Condition ((Get-PathCount $embeddedImage $FirmwareFullPath) -eq 0) `
        "The local firmware input path leaked into the image."
    $embeddedBuildOutput = Get-Content -LiteralPath (Get-LatestBuildScriptOutput) -Raw
    Assert-Condition (($embeddedBuildOutput -split "`n" |
        Where-Object { $_.Trim() -eq "cargo:rustc-cfg=marvell_fw_present" }).Count -eq 1) `
        "The firmware cfg was not emitted exactly once."
    Assert-Condition (($embeddedBuildOutput -split "`n" |
        Where-Object { $_ -like "cargo:rustc-env=MARVELL_FW_PATH=*" }).Count -eq 1) `
        "The firmware path environment was not emitted exactly once."

    # A caller-provided path and leaked process environment must remain inert
    # unless package-stage0 receives the explicit require switch. Reusing the
    # target also proves that Cargo invalidates the prior cfg-bearing build.
    $env:RAIOS_REQUIRE_MARVELL_FIRMWARE = "1"
    $env:RAIOS_MARVELL_FIRMWARE_PATH = $FirmwareFullPath
    $withoutOptInImage = Join-Path $WorkRoot "without-opt-in.img"
    Invoke-PackageCase -Name "without-opt-in" -Arguments ($commonArgs + @(
        "-Image", $withoutOptInImage,
        "-MarvellFirmwarePath", $FirmwareFullPath
    ))
    Assert-Condition ((Get-PatternCount $KernelPath $FirmwareFullPath) -eq 0) `
        "Firmware remained embedded without the explicit require switch."
    Assert-Condition ((Get-PatternCount $withoutOptInImage $FirmwareFullPath) -eq 0) `
        "Firmware was packaged without the explicit require switch."
    $withoutOptInBuildOutput = Get-Content -LiteralPath (Get-LatestBuildScriptOutput) -Raw
    Assert-Condition (-not $withoutOptInBuildOutput.Contains("cargo:rustc-cfg=marvell_fw_present")) `
        "The no-opt-in build emitted the firmware cfg."
    Assert-Condition (-not $withoutOptInBuildOutput.Contains("cargo:rustc-env=MARVELL_FW_PATH=")) `
        "The no-opt-in build emitted the firmware path environment."

    $reportPath = Join-Path $WorkRoot "$ReportId.json"
    [ordered]@{
        report_id = $ReportId
        verdict = "green"
        firmware_length = $firmwareItem.Length
        firmware_sha256 = $firmwareSha256
        cache_mode = "fresh target followed by same-target invalidation in fresh PowerShell processes"
        cases = $Results
    } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $reportPath -Encoding UTF8
    Write-Output "marvell firmware embedding predicate: green"
    Write-Output "report_id=$ReportId"
    Write-Output "report_path=$reportPath"
}
finally {
    $env:CARGO_TARGET_DIR = $oldCargoTargetDir
    $env:CARGO_HOME = $oldCargoHome
    $env:RAIOS_REQUIRE_MARVELL_FIRMWARE = $oldRequireMarvellFirmware
    $env:RAIOS_MARVELL_FIRMWARE_PATH = $oldMarvellFirmwarePath
}
