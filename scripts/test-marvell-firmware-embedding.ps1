param(
    [Parameter(Mandatory = $true)]
    [string]$FirmwarePath,
    [string]$WorkRoot = (Join-Path $env:TEMP ("raios-marvell-firmware-{0}" -f [Guid]::NewGuid().ToString("N"))),
    [string]$CargoHome = "",
    [string]$CorePolicyPrivateKey = (Join-Path $env:LOCALAPPDATA "raiOS\keys\core-policy-owner.p256.secret")
)

$ErrorActionPreference = "Stop"

$ExpectedLength = 723540
$ExpectedSha256 = "CF4F51F41BD7EF4D7FE65FB76B8A2A0897BC70A0742BC4AEA13D93B03FFFD03A"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$PackageScript = Join-Path $PSScriptRoot "package-stage0.ps1"
$WriterScript = Join-Path $PSScriptRoot "write-stage0-usb.ps1"
$FirmwareFullPath = [IO.Path]::GetFullPath($FirmwarePath)
$CorePolicyPrivateKeyFullPath = [IO.Path]::GetFullPath($CorePolicyPrivateKey)
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

function Invoke-WriterCase {
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
        $ErrorActionPreference = "Continue"
        & powershell -NoProfile -ExecutionPolicy Bypass -File $WriterScript @Arguments `
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

    $logText = ((Get-Content -LiteralPath $stdoutPath -Raw -ErrorAction SilentlyContinue) +
        (Get-Content -LiteralPath $stderrPath -Raw -ErrorAction SilentlyContinue))
    if (-not [string]::IsNullOrWhiteSpace($ExpectedLogFragment)) {
        Assert-Condition ($logText.Contains($ExpectedLogFragment)) `
            "$Name did not report the expected result '$ExpectedLogFragment'."
    }
    Assert-Condition (-not $logText.Contains("Run this script from an elevated PowerShell window.")) `
        "$Name reached the administrator/disk boundary instead of terminating in source-only validation."

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
if (-not (Test-Path -LiteralPath $CorePolicyPrivateKeyFullPath -PathType Leaf)) {
    throw "Core Policy owner key is required for the exported ESP/writer fixture: $CorePolicyPrivateKeyFullPath"
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

    $existingExportDir = Join-Path $WorkRoot "existing-export"
    New-Item -ItemType Directory -Path $existingExportDir | Out-Null
    Set-Content -LiteralPath (Join-Path $existingExportDir "sentinel.txt") -Value "do not replace"
    Invoke-PackageCase -Name "export-existing-target" -ExpectFailure `
        -ExpectedLogFragment "Refusing pre-existing ESP export path" `
        -Arguments @(
            "-Profile", "release",
            "-UseTempEsp",
            "-Image", (Join-Path $WorkRoot "existing-export.img"),
            "-RequireMarvellFirmware",
            "-MarvellFirmwarePath", $FirmwareFullPath,
            "-CorePolicyPrivateKey", $CorePolicyPrivateKeyFullPath,
            "-ExportEspDir", $existingExportDir
        )
    Assert-Condition ((Get-Content -LiteralPath (Join-Path $existingExportDir "sentinel.txt") -Raw).Trim() -eq "do not replace") `
        "The rejected export target was modified."

    $emptyExportDir = Join-Path $WorkRoot "empty-existing-export"
    New-Item -ItemType Directory -Path $emptyExportDir | Out-Null
    Invoke-PackageCase -Name "export-empty-existing-target" -ExpectFailure `
        -ExpectedLogFragment "Refusing pre-existing ESP export path" `
        -Arguments @(
            "-Profile", "release",
            "-UseTempEsp",
            "-Image", (Join-Path $WorkRoot "empty-existing-export.img"),
            "-RequireMarvellFirmware",
            "-MarvellFirmwarePath", $FirmwareFullPath,
            "-CorePolicyPrivateKey", $CorePolicyPrivateKeyFullPath,
            "-ExportEspDir", $emptyExportDir
        )

    $unsafeExportDir = Join-Path $env:LOCALAPPDATA ("raios-unsafe-export-{0}" -f $PID)
    Invoke-PackageCase -Name "export-unsafe-root" -ExpectFailure `
        -ExpectedLogFragment "outside the host temp root" `
        -Arguments @(
            "-Profile", "release",
            "-UseTempEsp",
            "-Image", (Join-Path $WorkRoot "unsafe-root.img"),
            "-RequireMarvellFirmware",
            "-MarvellFirmwarePath", $FirmwareFullPath,
            "-CorePolicyPrivateKey", $CorePolicyPrivateKeyFullPath,
            "-ExportEspDir", $unsafeExportDir
        )
    Assert-Condition (-not (Test-Path -LiteralPath $unsafeExportDir)) `
        "The rejected unsafe export path was created."

    $collisionPath = Join-Path $WorkRoot "export-image-collision"
    Invoke-PackageCase -Name "export-image-collision" -ExpectFailure `
        -ExpectedLogFragment "overlaps the image path" `
        -Arguments @(
            "-Profile", "release",
            "-UseTempEsp",
            "-Image", $collisionPath,
            "-RequireMarvellFirmware",
            "-MarvellFirmwarePath", $FirmwareFullPath,
            "-CorePolicyPrivateKey", $CorePolicyPrivateKeyFullPath,
            "-ExportEspDir", $collisionPath
        )

    $exportedImage = Join-Path $WorkRoot "exported.img"
    $exportedEsp = Join-Path $WorkRoot "exported-esp"
    Invoke-PackageCase -Name "exported" -Arguments @(
        "-Profile", "release",
        "-UseTempEsp",
        "-Image", $exportedImage,
        "-RequireMarvellFirmware",
        "-MarvellFirmwarePath", $FirmwareFullPath,
        "-CorePolicyPrivateKey", $CorePolicyPrivateKeyFullPath,
        "-CorePolicySlot", "A",
        "-CorePolicyGeneration", "1",
        "-ExportEspDir", $exportedEsp
    )
    $exportedKernel = Join-Path $exportedEsp "kernel\kernel.elf"
    $exportedPolicy = Join-Path $exportedEsp "raios\core-policy.bin"
    Assert-Condition (Test-Path -LiteralPath $exportedEsp -PathType Container) `
        "The successful package did not preserve its exported ESP payload."
    Assert-Condition (Test-Path -LiteralPath $exportedKernel -PathType Leaf) `
        "The exported ESP payload is missing kernel\kernel.elf."
    Assert-Condition (Test-Path -LiteralPath $exportedPolicy -PathType Leaf) `
        "The exported ESP payload is missing its kernel-bound Core Policy."
    Assert-Condition ((Get-PatternCount $exportedKernel $FirmwareFullPath) -eq 1) `
        "The exported ESP kernel does not contain pcie8897_uapsta.bin exactly once."
    Assert-Condition ((Get-PatternCount $exportedImage $FirmwareFullPath) -eq 1) `
        "The exported image does not contain pcie8897_uapsta.bin exactly once."

    $writerCommonArgs = @(
        "-DiskNumber", "2147483647",
        "-SkipBuild",
        "-UsePersistLayout",
        "-RequireMarvellFirmware",
        "-MarvellFirmwarePath", $FirmwareFullPath,
        "-CorePolicySlot", "A",
        "-CorePolicyGeneration", "1",
        "-ValidateSourceOnly"
    )
    Invoke-WriterCase -Name "writer-exported-positive" `
        -ExpectedLogFragment "exported ESP source validation: green" `
        -Arguments ($writerCommonArgs + @("-SourceEspDir", $exportedEsp))

    Invoke-WriterCase -Name "writer-source-without-skip" -ExpectFailure `
        -ExpectedLogFragment "-SourceEspDir is accepted only together with -SkipBuild" `
        -Arguments @(
            "-DiskNumber", "2147483647",
            "-UsePersistLayout",
            "-RequireMarvellFirmware",
            "-MarvellFirmwarePath", $FirmwareFullPath,
            "-SourceEspDir", $exportedEsp
        )

    Invoke-WriterCase -Name "writer-missing-source" -ExpectFailure `
        -ExpectedLogFragment "Exported ESP source directory is missing or invalid" `
        -Arguments ($writerCommonArgs + @("-SourceEspDir", (Join-Path $WorkRoot "missing-export")))

    $writerMissingFirmwareArgs = @(
        "-DiskNumber", "2147483647",
        "-SkipBuild",
        "-UsePersistLayout",
        "-RequireMarvellFirmware",
        "-MarvellFirmwarePath", $missingPath,
        "-ValidateSourceOnly",
        "-SourceEspDir", $exportedEsp
    )
    Invoke-WriterCase -Name "writer-missing-firmware" -ExpectFailure `
        -ExpectedLogFragment "Pinned Marvell firmware is missing for writer validation" `
        -Arguments $writerMissingFirmwareArgs

    $writerWrongFirmwareArgs = @(
        "-DiskNumber", "2147483647",
        "-SkipBuild",
        "-UsePersistLayout",
        "-RequireMarvellFirmware",
        "-MarvellFirmwarePath", $wrongShaPath,
        "-ValidateSourceOnly",
        "-SourceEspDir", $exportedEsp
    )
    Invoke-WriterCase -Name "writer-wrong-firmware" -ExpectFailure `
        -ExpectedLogFragment "Pinned Marvell firmware SHA-256 mismatch for writer validation" `
        -Arguments $writerWrongFirmwareArgs

    $missingPolicyEsp = Join-Path $WorkRoot "missing-policy-esp"
    Copy-Item -LiteralPath $exportedEsp -Destination $missingPolicyEsp -Recurse
    Remove-Item -LiteralPath (Join-Path $missingPolicyEsp "raios\core-policy.bin") -Force
    Invoke-WriterCase -Name "writer-missing-policy" -ExpectFailure `
        -ExpectedLogFragment "missing required file 'raios\core-policy.bin'" `
        -Arguments ($writerCommonArgs + @("-SourceEspDir", $missingPolicyEsp))

    $wrongPolicyEsp = Join-Path $WorkRoot "wrong-policy-esp"
    Copy-Item -LiteralPath $exportedEsp -Destination $wrongPolicyEsp -Recurse
    $wrongPolicyPath = Join-Path $wrongPolicyEsp "raios\core-policy.bin"
    $wrongPolicyBytes = [IO.File]::ReadAllBytes($wrongPolicyPath)
    $wrongPolicyBytes[0] = $wrongPolicyBytes[0] -bxor 0xff
    [IO.File]::WriteAllBytes($wrongPolicyPath, $wrongPolicyBytes)
    Invoke-WriterCase -Name "writer-wrong-policy-binding" -ExpectFailure `
        -ExpectedLogFragment "missing a valid binding to the packaged kernel" `
        -Arguments ($writerCommonArgs + @("-SourceEspDir", $wrongPolicyEsp))

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
