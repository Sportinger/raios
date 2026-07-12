# Support functions for shadow-vm-smoke.ps1.
# This file is dot-sourced and intentionally uses variables from the caller script scope.

function Resolve-OptionalPath {
    param([string]$Path)
    if (-not $Path) {
        return $null
    }
    if (-not (Test-Path -LiteralPath $Path)) {
        throw "Path does not exist: $Path"
    }
    return (Resolve-Path -LiteralPath $Path).Path
}

function Get-UnresolvedFullPath {
    param([string]$Path)

    if ([System.IO.Path]::IsPathRooted($Path)) {
        return [System.IO.Path]::GetFullPath($Path)
    }
    return [System.IO.Path]::GetFullPath((Join-Path (Get-Location) $Path))
}

function Test-PathUnderDirectory {
    param(
        [string]$Path,
        [string]$Directory
    )

    $fullPath = [System.IO.Path]::GetFullPath($Path).TrimEnd([char]'\', [char]'/')
    $fullDirectory = [System.IO.Path]::GetFullPath($Directory).TrimEnd([char]'\', [char]'/')
    return $fullPath.Equals($fullDirectory, [System.StringComparison]::OrdinalIgnoreCase) -or
        $fullPath.StartsWith("$fullDirectory$([System.IO.Path]::DirectorySeparatorChar)", [System.StringComparison]::OrdinalIgnoreCase)
}

function Assert-PersistDiskPathSafe {
    param([string]$Path)

    $fullPath = Get-UnresolvedFullPath -Path $Path
    $releaseRoot = [System.IO.Path]::GetFullPath((Join-Path $RepoRoot "release"))
    $stage0Image = [System.IO.Path]::GetFullPath((Join-Path $RepoRoot "release\raios-stage0.img"))
    if (Test-PathUnderDirectory -Path $fullPath -Directory $releaseRoot) {
        throw "PersistDiskPath must not be under release/: $fullPath"
    }
    if ($fullPath.Equals($stage0Image, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "PersistDiskPath must not be the production boot image: $fullPath"
    }
    return $fullPath
}

function Resolve-PersistDiskImage {
    param(
        [string]$PersistDiskPath,
        [string]$RunDir
    )

    $candidate = if ($PersistDiskPath) {
        Assert-PersistDiskPathSafe -Path $PersistDiskPath
    }
    else {
        Assert-PersistDiskPathSafe -Path (Join-Path $RunDir "raios-persist-gpt.img")
    }

    $builder = Join-Path $RepoRoot "scripts\make-gpt-persist-image.py"
    if (Test-Path -LiteralPath $candidate) {
        $inspectJson = & python $builder --inspect-json $candidate
        if ($LASTEXITCODE -ne 0) {
            throw "Existing persist disk failed validation: $candidate"
        }
        $inspection = ($inspectJson -join [Environment]::NewLine) | ConvertFrom-Json
        if (-not (
            [bool]$inspection.gpt_header_valid -and
            [bool]$inspection.gpt_crc_checked -and
            [bool]$inspection.gpt_seed_data_found -and
            [bool]$inspection.data_superblock_valid
        )) {
            throw "Existing persist disk failed validation: $candidate"
        }
    }
    else {
        # Discard the builder's chatty stdout ("wrote <path>", table, hex head)
        # so it does NOT leak into this function's return stream (which would make
        # the returned value an array whose first element is "wrote C:\...").
        $null = & python $builder --self-check $candidate 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "Persist disk build failed with exit code $LASTEXITCODE"
        }
    }
    return (Resolve-Path -LiteralPath $candidate).Path
}

function Get-FileSha256OrNull {
    param([string]$Path)
    if (-not $Path) {
        return $null
    }
    if (-not (Test-Path -LiteralPath $Path)) {
        return $null
    }
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-TextFileReport {
    param([string]$Path)

    $exists = $false
    $sizeBytes = $null
    if ($Path -and (Test-Path -LiteralPath $Path)) {
        $exists = $true
        try {
            $sizeBytes = [int64](Get-Item -LiteralPath $Path -ErrorAction Stop).Length
        }
        catch {
            $sizeBytes = $null
        }
    }

    return [ordered]@{
        path = $Path
        sha256 = (Get-FileSha256OrNull -Path $Path)
        exists = $exists
        size_bytes = $sizeBytes
        tail_present = ($null -ne $sizeBytes -and $sizeBytes -gt 0)
    }
}

function Get-TextSha256 {
    param([string]$Text)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $bytes = [System.Text.Encoding]::UTF8.GetBytes($Text)
        $hash = $sha.ComputeHash($bytes)
        return ([BitConverter]::ToString($hash) -replace "-", "").ToLowerInvariant()
    }
    finally {
        $sha.Dispose()
    }
}

function Send-QemuMonitorCommand {
    param(
        [string]$Command,
        [int]$ReplyWaitMilliseconds = 3000
    )

    if ($MonitorTcpPort -le 0) {
        throw "QEMU monitor is unavailable for profile '$Profile'"
    }
    if ($ReplyWaitMilliseconds -lt 0) {
        throw "QEMU monitor reply wait must not be negative"
    }
    $client = [System.Net.Sockets.TcpClient]::new()
    try {
        $connect = $client.BeginConnect("127.0.0.1", $MonitorTcpPort, $null, $null)
        try {
            if (-not $connect.AsyncWaitHandle.WaitOne([TimeSpan]::FromSeconds(8))) {
                throw "Timed out connecting to QEMU monitor TCP port $MonitorTcpPort"
            }
            $client.EndConnect($connect)
        }
        finally {
            $connect.AsyncWaitHandle.Dispose()
        }
        $client.SendTimeout = 4000
        $client.ReceiveTimeout = 500
        $stream = $client.GetStream()
        $buffer = [byte[]]::new(4096)
        while ($stream.DataAvailable) {
            $null = $stream.Read($buffer, 0, $buffer.Length)
        }
        $bytes = [System.Text.Encoding]::ASCII.GetBytes("$Command`n")
        $stream.Write($bytes, 0, $bytes.Length)
        $stream.Flush()
        $reply = [System.Text.StringBuilder]::new()
        $deadline = [DateTime]::UtcNow.AddMilliseconds($ReplyWaitMilliseconds)
        do {
            if ($stream.DataAvailable) {
                $count = $stream.Read($buffer, 0, $buffer.Length)
                if ($count -gt 0) {
                    [void]$reply.Append([System.Text.Encoding]::ASCII.GetString($buffer, 0, $count))
                }
            }
            Start-Sleep -Milliseconds 100
        } while ([DateTime]::UtcNow -lt $deadline)
        return $reply.ToString()
    }
    finally {
        $client.Dispose()
    }
}

function Send-QemuAbsolutePointerClick {
    param(
        [ValidateRange(0, 32767)][int]$X,
        [ValidateRange(0, 32767)][int]$Y
    )

    if ($QmpTcpPort -le 0) {
        throw "QEMU QMP is unavailable for profile '$Profile'"
    }
    $client = [System.Net.Sockets.TcpClient]::new()
    try {
        $client.Connect("127.0.0.1", $QmpTcpPort)
        $client.ReceiveTimeout = 4000
        $client.SendTimeout = 4000
        $stream = $client.GetStream()
        $reader = [System.IO.StreamReader]::new($stream, [System.Text.Encoding]::UTF8, $false, 4096, $true)
        $writer = [System.IO.StreamWriter]::new($stream, [System.Text.UTF8Encoding]::new($false), 4096, $true)
        $writer.NewLine = "`n"
        $writer.AutoFlush = $true

        $greeting = $reader.ReadLine() | ConvertFrom-Json
        if ($null -eq $greeting.QMP) {
            throw "QMP greeting was missing"
        }
        $writer.WriteLine('{"execute":"qmp_capabilities"}')
        $capabilities = $reader.ReadLine() | ConvertFrom-Json
        if ($null -ne $capabilities.error) {
            throw "QMP capabilities failed: $($capabilities.error | ConvertTo-Json -Compress)"
        }

        $moveAndPress = [ordered]@{
            execute = "input-send-event"
            arguments = [ordered]@{
                events = @(
                    [ordered]@{ type = "abs"; data = [ordered]@{ axis = "x"; value = $X } },
                    [ordered]@{ type = "abs"; data = [ordered]@{ axis = "y"; value = $Y } },
                    [ordered]@{ type = "btn"; data = [ordered]@{ down = $true; button = "left" } }
                )
            }
        } | ConvertTo-Json -Compress -Depth 8
        $writer.WriteLine($moveAndPress)
        $pressed = $reader.ReadLine() | ConvertFrom-Json
        if ($null -ne $pressed.error) {
            throw "QMP pointer press failed: $($pressed.error | ConvertTo-Json -Compress)"
        }

        Start-Sleep -Milliseconds 150
        $release = [ordered]@{
            execute = "input-send-event"
            arguments = [ordered]@{
                events = @(
                    [ordered]@{ type = "btn"; data = [ordered]@{ down = $false; button = "left" } }
                )
            }
        } | ConvertTo-Json -Compress -Depth 8
        $writer.WriteLine($release)
        $released = $reader.ReadLine() | ConvertFrom-Json
        if ($null -ne $released.error) {
            throw "QMP pointer release failed: $($released.error | ConvertTo-Json -Compress)"
        }
    }
    finally {
        $client.Dispose()
    }
}

function Save-QemuScreendump {
    param([string]$Name)

    if ($Name -notmatch '^[a-z0-9-]+$') {
        throw "Visual evidence name must be lowercase ASCII words: $Name"
    }
    $captureDir = Join-Path $RepoRoot "target\captures"
    New-Item -ItemType Directory -Force -Path $captureDir | Out-Null
    $path = Join-Path $captureDir ("$RunId-$Name.ppm")
    $hmpPath = $path.Replace("\", "/")
    $reply = Send-QemuMonitorCommand -Command "screendump $hmpPath"
    $deadline = [DateTime]::UtcNow.AddSeconds(20)
    while (-not (Test-Path -LiteralPath $path -PathType Leaf) -and [DateTime]::UtcNow -lt $deadline) {
        Start-Sleep -Milliseconds 100
    }
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "QEMU screendump '$Name' did not create $path. Reply: $reply"
    }
    $pngPath = [System.IO.Path]::ChangeExtension($path, ".png")
    Convert-QemuPpmToPng -PpmPath $path -PngPath $pngPath
    $evidence = [ordered]@{
        name = $Name
        path = (Resolve-Path -LiteralPath $pngPath).Path
        sha256 = Get-FileSha256OrNull -Path $pngPath
        bytes = [int64](Get-Item -LiteralPath $pngPath).Length
        format = "png_from_qemu_ppm_p6"
        source_ppm_sha256 = Get-FileSha256OrNull -Path $path
        secure_strip_sha256 = Get-QemuSecureStripSha256 -PngPath $pngPath
    }
    $script:VisualEvidence.Add($evidence) | Out-Null
    Add-Predicate -Name "visual:$Name" -Expected "QEMU screendump bound to this run" -Passed ($null -ne $evidence.sha256) -Actual $evidence.sha256
    return $evidence
}

function Get-QemuSecureStripSha256 {
    param([string]$PngPath)

    Add-Type -AssemblyName System.Drawing
    $bitmap = [System.Drawing.Bitmap]::new($PngPath)
    try {
        $height = [Math]::Min(76, $bitmap.Height)
        if ($bitmap.Width -le 0 -or $height -le 0) {
            throw "Invalid QEMU PNG dimensions for secure-strip evidence"
        }
        $bytes = [byte[]]::new($bitmap.Width * $height * 4)
        $index = 0
        for ($y = 0; $y -lt $height; $y++) {
            for ($x = 0; $x -lt $bitmap.Width; $x++) {
                $pixel = $bitmap.GetPixel($x, $y)
                $bytes[$index++] = $pixel.R
                $bytes[$index++] = $pixel.G
                $bytes[$index++] = $pixel.B
                $bytes[$index++] = $pixel.A
            }
        }
        $sha = [System.Security.Cryptography.SHA256]::Create()
        try {
            return ([BitConverter]::ToString($sha.ComputeHash($bytes)) -replace "-", "").ToLowerInvariant()
        }
        finally {
            $sha.Dispose()
        }
    }
    finally {
        $bitmap.Dispose()
    }
}

function Read-QemuPpmToken {
    param(
        [byte[]]$Buffer,
        [ref]$Index
    )

    while ($Index.Value -lt $Buffer.Length) {
        while ($Index.Value -lt $Buffer.Length -and [char]$Buffer[$Index.Value] -match "\s") {
            $Index.Value++
        }
        if ($Index.Value -lt $Buffer.Length -and [char]$Buffer[$Index.Value] -eq "#") {
            while ($Index.Value -lt $Buffer.Length -and $Buffer[$Index.Value] -ne 10) {
                $Index.Value++
            }
            continue
        }
        break
    }
    $start = $Index.Value
    while ($Index.Value -lt $Buffer.Length -and -not ([char]$Buffer[$Index.Value] -match "\s")) {
        $Index.Value++
    }
    if ($start -eq $Index.Value) {
        throw "Invalid QEMU PPM header token at byte $start"
    }
    return [System.Text.Encoding]::ASCII.GetString($Buffer, $start, $Index.Value - $start)
}

function Convert-QemuPpmToPng {
    param(
        [string]$PpmPath,
        [string]$PngPath
    )

    Add-Type -AssemblyName System.Drawing
    $bytes = [System.IO.File]::ReadAllBytes($PpmPath)
    $index = 0
    $indexRef = [ref]$index
    $magic = Read-QemuPpmToken -Buffer $bytes -Index $indexRef
    $width = [int](Read-QemuPpmToken -Buffer $bytes -Index $indexRef)
    $height = [int](Read-QemuPpmToken -Buffer $bytes -Index $indexRef)
    $maxValue = [int](Read-QemuPpmToken -Buffer $bytes -Index $indexRef)
    if ($magic -ne "P6" -or $width -le 0 -or $height -le 0 -or $maxValue -ne 255) {
        throw "Unsupported QEMU PPM: magic=$magic width=$width height=$height max=$maxValue"
    }
    if ($indexRef.Value -ge $bytes.Length -or -not ([char]$bytes[$indexRef.Value] -match "\s")) {
        throw "Invalid QEMU PPM raster delimiter"
    }
    $indexRef.Value++
    if ($bytes[$indexRef.Value - 1] -eq 13 -and $indexRef.Value -lt $bytes.Length -and $bytes[$indexRef.Value] -eq 10) {
        $indexRef.Value++
    }
    $offset = $indexRef.Value
    $expected = [int64]$width * [int64]$height * 3
    if (($bytes.Length - $offset) -ne $expected) {
        throw "Invalid QEMU PPM raster length"
    }
    $bitmap = [System.Drawing.Bitmap]::new($width, $height, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
    try {
        $rect = [System.Drawing.Rectangle]::new(0, 0, $width, $height)
        $data = $bitmap.LockBits($rect, [System.Drawing.Imaging.ImageLockMode]::WriteOnly, $bitmap.PixelFormat)
        try {
            $stride = [Math]::Abs($data.Stride)
            $pixels = [byte[]]::new($stride * $height)
            $source = $offset
            for ($y = 0; $y -lt $height; $y++) {
                $destination = $y * $stride
                for ($x = 0; $x -lt $width; $x++) {
                    $pixels[$destination++] = $bytes[$source + 2]
                    $pixels[$destination++] = $bytes[$source + 1]
                    $pixels[$destination++] = $bytes[$source]
                    $source += 3
                }
            }
            [Runtime.InteropServices.Marshal]::Copy($pixels, 0, $data.Scan0, $pixels.Length)
        }
        finally {
            $bitmap.UnlockBits($data)
        }
        $bitmap.Save($PngPath, [System.Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        $bitmap.Dispose()
    }
}

function New-ReliableDevPromotionSignatureHex {
    param(
        [string]$AttestationReferenceHash,
        [string]$SignerPath = ""
    )

    if ($AttestationReferenceHash -notmatch '^[0-9a-fA-F]{64}$') {
        throw "attestation_reference_hash must be 64 hex characters"
    }

    $resolvedSigner = $SignerPath
    if (-not $resolvedSigner) {
        $resolvedSigner = Join-Path $RepoRoot "target\debug\dev-promotion-signer.exe"
        if (-not (Test-Path -LiteralPath $resolvedSigner)) {
            $resolvedSigner = Join-Path $RepoRoot "target\debug\dev-promotion-signer"
        }
    }
    if (-not (Test-Path -LiteralPath $resolvedSigner)) {
        throw "dev-promotion-signer binary not found: $resolvedSigner"
    }

    $output = & $resolvedSigner $AttestationReferenceHash 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "dev-promotion-signer failed: $($output -join [Environment]::NewLine)"
    }
    $signature = [string](@($output)[0])
    $signature = $signature.Trim()
    if ($signature -notmatch '^[0-9a-f]+$') {
        throw "dev-promotion-signer returned non-hex output: $signature"
    }
    return $signature
}

function ConvertTo-ReportJson {
    param([object]$Value)
    return ($Value | ConvertTo-Json -Depth 20 -Compress)
}

function Register-ReportForbiddenDynamicValue {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][byte[]]$Value
    )

    if ($Label -notmatch '^[a-z0-9][a-z0-9._-]{0,63}$') {
        throw "Forbidden report value label must be bounded lowercase ASCII"
    }
    if ($Value.Length -eq 0 -or $Value.Length -gt 4096) {
        throw "Forbidden report value '$Label' must contain 1..4096 bytes"
    }

    $owned = [byte[]]::new($Value.Length)
    [Array]::Copy($Value, $owned, $Value.Length)
    try {
        $script:ReportForbiddenDynamicValues.Add([pscustomobject]@{
            Label = $Label
            Value = $owned
        }) | Out-Null
    }
    catch {
        [Array]::Clear($owned, 0, $owned.Length)
        throw
    }
}

function Test-ByteSequencePresent {
    param(
        [byte[]]$Bytes,
        [byte[]]$Needle
    )

    if ($null -eq $Bytes -or $null -eq $Needle -or $Needle.Length -eq 0 -or
        $Needle.Length -gt $Bytes.Length) {
        return $false
    }
    $lastStart = $Bytes.Length - $Needle.Length
    for ($offset = 0; $offset -le $lastStart; $offset++) {
        if ($Bytes[$offset] -ne $Needle[0]) {
            continue
        }
        $matches = $true
        for ($index = 1; $index -lt $Needle.Length; $index++) {
            if ($Bytes[$offset + $index] -ne $Needle[$index]) {
                $matches = $false
                break
            }
        }
        if ($matches) {
            return $true
        }
    }
    return $false
}

function Get-MatchedForbiddenDynamicLabels {
    param([byte[]]$Bytes)

    $labels = [System.Collections.Generic.List[string]]::new()
    foreach ($entry in $script:ReportForbiddenDynamicValues) {
        if (Test-ByteSequencePresent -Bytes $Bytes -Needle $entry.Value) {
            $labels.Add($entry.Label) | Out-Null
        }
    }
    return @($labels)
}

function Redact-ForbiddenDynamicValues {
    param([byte[]]$Bytes)

    foreach ($entry in $script:ReportForbiddenDynamicValues) {
        $needle = $entry.Value
        $allAsterisks = $true
        foreach ($value in $needle) {
            if ($value -ne [byte][char]'*') {
                $allAsterisks = $false
                break
            }
        }
        $replacement = [byte][char]$(if ($allAsterisks) { '#' } else { '*' })
        $lastStart = $Bytes.Length - $needle.Length
        for ($offset = 0; $offset -le $lastStart; $offset++) {
            $matches = $true
            for ($index = 0; $index -lt $needle.Length; $index++) {
                if ($Bytes[$offset + $index] -ne $needle[$index]) {
                    $matches = $false
                    break
                }
            }
            if ($matches) {
                for ($index = 0; $index -lt $needle.Length; $index++) {
                    $Bytes[$offset + $index] = $replacement
                }
                $offset += $needle.Length - 1
            }
        }
    }
}

function Clear-ReportForbiddenDynamicValues {
    foreach ($entry in $script:ReportForbiddenDynamicValues) {
        if ($null -ne $entry.Value -and $entry.Value.Length -gt 0) {
            [Array]::Clear($entry.Value, 0, $entry.Value.Length)
        }
    }
    $script:ReportForbiddenDynamicValues.Clear()
}

function Set-ForbiddenDynamicReportFailure {
    param(
        [object]$Report,
        [int]$PredicateIndex,
        [string[]]$Labels
    )

    $labelSummary = $Labels -join ','
    $script:Result = "failed"
    $script:ReportSecurityTripwire = $labelSummary
    $predicate = $Predicates[$PredicateIndex]
    $predicate.passed = $false
    $predicate.actual = "absent=false labels=$labelSummary"
    if (-not $Failures.Contains($predicate.name)) {
        $Failures.Add($predicate.name) | Out-Null
    }
    $Report.result = "failed"
    $Report.evidence_binding.result = "failed"
    $Report.evidence_binding.predicate_passed_count = @($Predicates.ToArray() | Where-Object { $_.passed }).Count
    $Report.evidence_binding.predicate_failed_count = @($Predicates.ToArray() | Where-Object { -not $_.passed }).Count
    $Report.failures = @($Failures.ToArray())
}

function Get-NullablePath {
    param([string]$Path)
    if ([string]::IsNullOrWhiteSpace($Path)) {
        return $null
    }
    return $Path
}

function Get-SerialLogContent {
    param([string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) {
        return $null
    }

    try {
        $item = Get-Item -LiteralPath $Path -ErrorAction Stop
    }
    catch {
        return $null
    }

    $length = [int64]$item.Length
    $writeTicks = [int64]$item.LastWriteTimeUtc.Ticks
    if (
        $script:SerialLogCachePath -eq $Path -and
        $script:SerialLogCacheLength -eq $length -and
        $script:SerialLogCacheWriteTicks -eq $writeTicks -and
        $null -ne $script:SerialLogCacheContent
    ) {
        return $script:SerialLogCacheContent
    }

    $content = Get-Content -Raw -LiteralPath $Path -ErrorAction SilentlyContinue
    if ($null -eq $content) {
        return $null
    }

    $script:SerialLogCachePath = $Path
    $script:SerialLogCacheLength = $length
    $script:SerialLogCacheWriteTicks = $writeTicks
    $script:SerialLogCacheContent = $content
    return $content
}

function Get-SerialLogTail {
    param([string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) {
        return "serial log not created"
    }

    $content = Get-SerialLogContent -Path $Path
    if ($null -eq $content) {
        return "serial log unreadable"
    }

    $limit = 1600
    if ($content.Length -le $limit) {
        return $content
    }
    return $content.Substring($content.Length - $limit)
}

function New-HardwareProfile {
    param(
        [string]$Nic,
        [bool]$ScratchDrive = $false,
        [bool]$AuditRollbackTargetDrive = $false,
        [bool]$PersistDrive = $false
    )

    $networkDevice = if ($Nic -eq "e1000") {
        "e1000_user"
    }
    else {
        "none"
    }

    $profile = [ordered]@{
        profile = "raios.shadow_vm.q35_xhci.v0"
        machine = "q35"
        memory = "512M"
        cpu = "max"
        firmware = "edk2-x86_64"
        boot_drive = "ide_raw_image"
        scratch_drive = if ($ScratchDrive) { "ide_raw_scratch_label_v0" } else { "none" }
        audit_rollback_target_drive = if ($AuditRollbackTargetDrive) { "ide_raw_audit_rollback_label_v0" } else { "none" }
        display = "none"
        serial = "tcp_chardev_with_log"
        input = @(
            "qemu-xhci",
            "usb-kbd",
            "usb-tablet"
        )
        network = $networkDevice
    }
    if ($PersistDrive) {
        $profile.persist_drive = "ide_raw_gpt_persist_v0"
    }
    return $profile
}

function Wait-ForLogText {
    param(
        [string]$Path,
        [string]$Needle,
        [int]$TimeoutSeconds
    )

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    do {
        $content = Get-SerialLogContent -Path $Path
        if ($null -ne $content -and $content -clike "*$Needle*") {
            return $true
        }
        Start-Sleep -Milliseconds 200
    } while ([DateTime]::UtcNow -lt $deadline)

    return $false
}

function Get-SerialLogOffset {
    $content = Get-SerialLogContent -Path $SerialLog
    if ($null -ne $content) {
        return [int64]$content.Length
    }
    return [int64]0
}

function Wait-ForLogTextAfterOffset {
    param(
        [string]$Path,
        [string]$Needle,
        [int64]$Offset,
        [int]$TimeoutSeconds
    )

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    do {
        Drain-SerialTcpOutput -Stream $script:SerialTcpDrainStream
        $content = Get-SerialLogContent -Path $Path
        if ($null -ne $content) {
            $start = [int64]$Offset
            if ($start -lt 0) {
                $start = [int64]0
            }
            if ($start -gt [int64]$content.Length) {
                $start = [int64]$content.Length
            }
            $after = if ($start -eq [int64]0) {
                $content
            }
            elseif ($start -lt [int64]$content.Length) {
                $content.Substring([int]$start)
            }
            else {
                ""
            }
            if ($after -clike "*$Needle*") {
                return $true
            }
        }
        Start-Sleep -Milliseconds 200
    } while ([DateTime]::UtcNow -lt $deadline)

    return $false
}

function Drain-SerialTcpOutput {
    param(
        [System.Net.Sockets.NetworkStream]$Stream
    )

    if ($null -eq $Stream) {
        return
    }

    $buffer = New-Object byte[] 4096
    try {
        while ($Stream.DataAvailable) {
            $null = $Stream.Read($buffer, 0, $buffer.Length)
        }
    }
    catch {
        return
    }
}

function Close-SerialTcpConnection {
    try {
        if ($null -ne $script:SerialTcpStream) {
            Drain-SerialTcpOutput -Stream $script:SerialTcpStream
        }
    }
    catch {
    }
    if ($null -ne $script:SerialTcpClient) {
        $script:SerialTcpClient.Close()
    }
    $script:SerialTcpClient = $null
    $script:SerialTcpStream = $null
    $script:SerialTcpDrainStream = $null
}

function Get-QemuProcessState {
    $qpid = $script:QemuPid
    if (-not $qpid) {
        return [ordered]@{ pid = $null; state = "unknown_no_pid"; exit_code = $null }
    }

    $proc = $script:QemuProcess
    $exitCode = $null
    $state = $null
    if ($null -ne $proc) {
        try {
            if (-not $proc.HasExited) {
                $state = "running"
            }
            else {
                $state = "exited"
                try { $exitCode = $proc.ExitCode } catch { $exitCode = $null }
            }
        }
        catch {
            $state = $null
        }
    }
    if ($null -eq $state) {
        try {
            Get-Process -Id $qpid -ErrorAction Stop | Out-Null
            $state = "running"
        }
        catch {
            $state = "exited"
        }
    }
    return [ordered]@{ pid = $qpid; state = $state; exit_code = $exitCode }
}

function Get-QemuProcessSnapshot {
    param([string]$Observation)

    $state = Get-QemuProcessState
    return [ordered]@{
        observation = $Observation
        pid = $state.pid
        state = $state.state
        exit_code = $state.exit_code
        checked_at_utc = ([DateTime]::UtcNow.ToString("o"))
    }
}

function Test-SerialPortListener {
    param([int]$Port)

    try {
        $listeners = @(Get-NetTCPConnection -State Listen -LocalPort $Port -ErrorAction Stop)
        return ($listeners.Count -gt 0)
    }
    catch {
        return $false
    }
}

function Register-SerialTransportFailure {
    param(
        [string]$Classification,
        [int]$Port,
        $QemuState,
        [bool]$ListenerPresent,
        [double]$ElapsedSeconds
    )

    $record = [ordered]@{
        kind = "serial_transport_failure"
        classification = $Classification
        port = $Port
        listener_present = $ListenerPresent
        qemu_pid = $QemuState.pid
        qemu_state = $QemuState.state
        qemu_exit_code = $QemuState.exit_code
        elapsed_seconds = [Math]::Round($ElapsedSeconds, 1)
        observed_at_utc = ([DateTime]::UtcNow.ToString("o"))
    }
    $script:SerialTransportFailure = $record
    $Failures.Add($record) | Out-Null
    return $record
}

function Get-SerialTcpStream {
    param(
        [int]$Port,
        [int]$TimeoutSeconds
    )

    if ($null -ne $script:SerialTcpClient -and
        $null -ne $script:SerialTcpStream -and
        $script:SerialTcpClient.Connected -and
        $script:SerialTcpStream.CanWrite) {
        return $script:SerialTcpStream
    }

    Close-SerialTcpConnection

    $connectStarted = [DateTime]::UtcNow
    $deadline = $connectStarted.AddSeconds($TimeoutSeconds)
    while ([DateTime]::UtcNow -lt $deadline) {
        $candidate = [System.Net.Sockets.TcpClient]::new()
        $candidate.NoDelay = $true
        $attemptFailed = $false
        try {
            $connect = $candidate.BeginConnect("127.0.0.1", $Port, $null, $null)
            if ($connect.AsyncWaitHandle.WaitOne([TimeSpan]::FromMilliseconds(500))) {
                $candidate.EndConnect($connect)
                $script:SerialTcpClient = $candidate
                $script:SerialTcpStream = $candidate.GetStream()
                return $script:SerialTcpStream
            }
        }
        catch {
            $attemptFailed = $true
        }
        $candidate.Close()

        # A dead QEMU can never come back within this run: abort immediately
        # with a structured classification instead of burning the whole
        # timeout budget (see failure classification log in PROJECT_STATUS).
        $qemuState = Get-QemuProcessState
        if ($qemuState.state -eq "exited") {
            $elapsed = ([DateTime]::UtcNow - $connectStarted).TotalSeconds
            $listener = Test-SerialPortListener -Port $Port
            Register-SerialTransportFailure -Classification "qemu_exited" -Port $Port -QemuState $qemuState -ListenerPresent $listener -ElapsedSeconds $elapsed | Out-Null
            throw "Serial transport failure (qemu_exited): QEMU pid $($qemuState.pid) exited (exit_code=$($qemuState.exit_code)) while reconnecting to serial TCP port $Port after $([Math]::Round($elapsed, 1))s"
        }

        Start-Sleep -Milliseconds 100
        if ($attemptFailed) { continue }
    }

    $elapsed = ([DateTime]::UtcNow - $connectStarted).TotalSeconds
    $qemuState = Get-QemuProcessState
    $listener = Test-SerialPortListener -Port $Port
    $classification =
        if ($qemuState.state -eq "exited") { "qemu_exited" }
        elseif (-not $listener) { "listener_missing_process_alive" }
        else { "connect_timeout_listener_present" }
    Register-SerialTransportFailure -Classification $classification -Port $Port -QemuState $qemuState -ListenerPresent $listener -ElapsedSeconds $elapsed | Out-Null
    throw "Serial transport failure ($classification): no connection to QEMU serial TCP port $Port after $([Math]::Round($elapsed, 1))s (qemu pid=$($qemuState.pid) state=$($qemuState.state) exit_code=$($qemuState.exit_code) listener_present=$listener)"
}

function Add-Predicate {
    param(
        [string]$Name,
        [string]$Expected,
        [bool]$Passed,
        [string]$Actual = ""
    )

    $Predicates.Add([ordered]@{
        name = $Name
        expected = $Expected
        passed = $Passed
        actual = $Actual
    }) | Out-Null

    if (-not $Passed) {
        $Failures.Add($Name) | Out-Null
    }
}

function Assert-LogContains {
    param(
        [string]$Name,
        [string]$Needle,
        [int]$TimeoutSeconds
    )

    $passed = Wait-ForLogText -Path $SerialLog -Needle $Needle -TimeoutSeconds $TimeoutSeconds
    $actual = if ($passed) { "found" } else { Get-SerialLogTail -Path $SerialLog }
    Add-Predicate -Name $Name -Expected "serial_contains:$Needle" -Passed $passed -Actual $actual
    if (-not $passed) {
        throw "Timed out waiting for '$Needle' in $SerialLog"
    }
}

function Assert-LogContainsFields {
    param(
        [string]$NamePrefix,
        [object[]]$Fields,
        [int]$TimeoutSeconds
    )

    foreach ($field in $Fields) {
        Assert-LogContains -Name "$NamePrefix$($field.Suffix)" -Needle $field.Needle -TimeoutSeconds $TimeoutSeconds
    }
}

function Assert-LogDoesNotContain {
    param(
        [string]$Name,
        [string]$Needle
    )

    $content = Get-SerialLogContent -Path $SerialLog
    if ($null -eq $content) {
        $content = ""
    }
    $passed = -not ($content.Contains($Needle))
    $actual = if ($passed) { "absent" } else { "found" }
    Add-Predicate -Name $Name -Expected "serial_not_contains:$Needle" -Passed $passed -Actual $actual
    if (-not $passed) {
        throw "Unexpected '$Needle' in $SerialLog"
    }
}

function Write-SerialTcpText {
    param(
        [System.Net.Sockets.NetworkStream]$Stream,
        [string]$Text
    )

    $bytes = [System.Text.Encoding]::ASCII.GetBytes($Text)
    $chunkSize = [Math]::Max(1, $SerialWriteChunkSize)
    for ($offset = 0; $offset -lt $bytes.Length; $offset += $chunkSize) {
        $count = [Math]::Min($chunkSize, $bytes.Length - $offset)
        $Stream.Write($bytes, $offset, $count)
        if ($SerialWriteDelayMilliseconds -gt 0) {
            Start-Sleep -Milliseconds $SerialWriteDelayMilliseconds
        }
    }
}

function Send-SerialText {
    param(
        [int]$Port,
        [string]$Text,
        [int]$TimeoutSeconds
    )

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    $client = $null
    while ([DateTime]::UtcNow -lt $deadline) {
        $candidate = [System.Net.Sockets.TcpClient]::new()
        $candidate.NoDelay = $true
        try {
            $connect = $candidate.BeginConnect("127.0.0.1", $Port, $null, $null)
            if ($connect.AsyncWaitHandle.WaitOne([TimeSpan]::FromMilliseconds(500))) {
                $candidate.EndConnect($connect)
                $client = $candidate
                break
            }
        }
        catch {
            $candidate.Close()
            Start-Sleep -Milliseconds 100
            continue
        }
        $candidate.Close()
        Start-Sleep -Milliseconds 100
    }
    if (-not $client) {
        throw "Timed out connecting to QEMU serial TCP port $Port"
    }

    try {
        $stream = $client.GetStream()
        Write-SerialTcpText -Stream $stream -Text $Text
        $stream.Flush()
        Start-Sleep -Milliseconds 250
    }
    finally {
        $client.Close()
    }
}

function Send-AgentCommand {
    param(
        [string]$Command,
        [string]$ExpectedMarker,
        [string]$Name = ""
    )

    $commandStartedAt = [DateTime]::UtcNow
    $startOffset = Get-SerialLogOffset
    $predicateName = if ($Name.Length -gt 0) { $Name } else { "command:$Command" }
    $passed = $false
    $sent = $false
    $stream = $null
    try {
        $stream = Get-SerialTcpStream -Port $SerialTcpPort -TimeoutSeconds $TimeoutSeconds
        $script:SerialTcpDrainStream = $stream
        Write-SerialTcpText -Stream $stream -Text "$Command`r"
        $sent = $true
        $stream.Flush()
        Start-Sleep -Milliseconds 50

        $passed = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle $ExpectedMarker -Offset $startOffset -TimeoutSeconds $TimeoutSeconds
        $actual = if ($passed) { "found_after_offset:$startOffset" } else { Get-SerialLogTail -Path $SerialLog }
        Add-Predicate -Name $predicateName -Expected "serial_contains_after_offset:$ExpectedMarker" -Passed $passed -Actual $actual
        if (-not $passed) {
            throw "Timed out waiting for '$ExpectedMarker' in $SerialLog after offset $startOffset"
        }
        Start-Sleep -Milliseconds 150
        Drain-SerialTcpOutput -Stream $stream
    }
    finally {
        $commandEndedAt = [DateTime]::UtcNow
        if ($sent) {
            $ExecutedCommands.Add([ordered]@{
                command = $Command
                name = $predicateName
                expected_marker = $ExpectedMarker
                response_offset = $startOffset
                duration_ms = ([int][Math]::Round(($commandEndedAt - $commandStartedAt).TotalMilliseconds))
                sent = $true
                passed = $passed
            }) | Out-Null
        }
        $script:SerialTcpDrainStream = $null
        if ($null -ne $stream) {
            Drain-SerialTcpOutput -Stream $stream
        }
        Close-SerialTcpConnection
    }
}

function Send-CandidateBytes {
    param(
        [string]$Path
    )

    if (-not (Test-Path -LiteralPath $Path)) {
        throw "Candidate artifact does not exist: $Path"
    }

    $bytes = [System.IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Path).Path)
    $base64 = [Convert]::ToBase64String($bytes)
    $chunkChars = 3000
    $chunkIndex = 0
    for ($offset = 0; $offset -lt $base64.Length; $offset += $chunkChars) {
        $count = [Math]::Min($chunkChars, $base64.Length - $offset)
        $chunk = $base64.Substring($offset, $count)
        $chunkIndex += 1
        Send-AgentCommand `
            -Command "module.submit_candidate_chunk $chunk" `
            -ExpectedMarker "RAIOS_AGENT_END module.submit_candidate_chunk" `
            -Name "candidate_delivery:chunk_$chunkIndex"
    }

    Send-AgentCommand `
        -Command "module.submit_candidate_finalize" `
        -ExpectedMarker "RAIOS_AGENT_END module.submit_candidate_finalize" `
        -Name "candidate_delivery:finalize"

    return [pscustomobject]@{
        path = (Resolve-Path -LiteralPath $Path).Path
        byte_len = $bytes.Length
        chunk_count = $chunkIndex
        finalize_response = (Get-LastAgentResponseJson -Method "module.submit_candidate_finalize")
    }
}

function Get-LastAgentResponseJson {
    param(
        [string]$Method
    )

    $content = Get-SerialLogContent -Path $SerialLog
    if ($null -eq $content) {
        throw "No serial log content found in $SerialLog"
    }
    $begin = "RAIOS_AGENT_BEGIN $Method"
    $end = "RAIOS_AGENT_END $Method"
    $beginIndex = $content.LastIndexOf($begin, [System.StringComparison]::Ordinal)
    if ($beginIndex -lt 0) {
        throw "No agent response for method '$Method' found in $SerialLog"
    }
    $jsonStart = $content.IndexOf("{", $beginIndex, [System.StringComparison]::Ordinal)
    $endIndex = $content.IndexOf($end, $jsonStart, [System.StringComparison]::Ordinal)
    if ($jsonStart -lt 0 -or $endIndex -lt 0) {
        throw "Incomplete agent response for method '$Method' found in $SerialLog"
    }
    $json = $content.Substring($jsonStart, $endIndex - $jsonStart).Trim()
    return $json | ConvertFrom-Json
}

function Assert-CurrentBootEventId {
    param(
        [string]$Name,
        [string]$Value
    )

    $passed = $Value -match '^event\.current_boot\.[0-9]{8}$'
    Add-Predicate -Name $Name -Expected "current_boot_event_id" -Passed $passed -Actual $Value
    if (-not $passed) {
        throw "Expected current-boot event id for '$Name', got '$Value'"
    }
}

function Write-Report {
    param(
        [string]$FinalResult,
        [string]$ResolvedImage,
        [string]$ResolvedArtifact,
        [string]$ResolvedManifest,
        [string[]]$QemuArgList,
        [object]$HardwareProfile,
        [DateTime]$StartedAt
    )

    New-Item -ItemType Directory -Force -Path $ReportDir | Out-Null

    $forbiddenValuesRegistered = $script:ReportForbiddenDynamicValues.Count -gt 0
    $forbiddenPredicateIndex = -1
    if ($forbiddenValuesRegistered) {
        $forbiddenPredicateIndex = $Predicates.Count
        Add-Predicate `
            -Name "report:forbidden_dynamic_values_absent" `
            -Expected "registered dynamic values absent from final report bytes before write and after readback" `
            -Passed $true `
            -Actual "absent=true scope=report_json"
    }

    $serialHash = Get-FileSha256OrNull -Path $SerialLog
    $errLog = [System.IO.Path]::ChangeExtension($SerialLog, ".err.txt")
    $endedAt = [DateTime]::UtcNow
    $networkMode = if ($Network) { "e1000_user_enabled" } else { "disabled" }
    $baseImageSha256 = Get-FileSha256OrNull -Path $ResolvedImage
    $artifactSha256 = Get-FileSha256OrNull -Path $ResolvedArtifact
    $manifestSha256 = Get-FileSha256OrNull -Path $ResolvedManifest
    $qemuArgsCanonical = ConvertTo-ReportJson -Value @($QemuArgList)
    $qemuArgsSha256 = Get-TextSha256 -Text $qemuArgsCanonical
    $hardwareProfileSha256 = Get-TextSha256 -Text (ConvertTo-ReportJson -Value $HardwareProfile)
    $executedCommandDetails = @($ExecutedCommands.ToArray())
    $executedCommandNames = @($executedCommandDetails | ForEach-Object { $_.command })

    $report = [ordered]@{
        schema = "raios.vm_test_report.v0"
        result = $FinalResult
        profile = $Profile
        generated_at_utc = ($endedAt.ToString("o"))
        started_at_utc = ($StartedAt.ToString("o"))
        duration_ms = ([int][Math]::Round(($endedAt - $StartedAt).TotalMilliseconds))
        run_id = $RunId
        sandbox_policy = [ordered]@{
            hypervisor = "qemu-system-x86_64"
            headless = $true
            shared_folders = "none"
            host_filesystem_mounts = "none"
            network = $networkMode
            boot_media = "temporary_or_explicit_image"
            qemu_killed_after_run = $true
        }
        base_image = [ordered]@{
            path = (Get-NullablePath -Path $ResolvedImage)
            sha256 = $baseImageSha256
            temporary = $TempImage
        }
        candidate_artifact = [ordered]@{
            path = (Get-NullablePath -Path $ResolvedArtifact)
            sha256 = $artifactSha256
        }
        candidate_manifest = [ordered]@{
            path = (Get-NullablePath -Path $ResolvedManifest)
            sha256 = $manifestSha256
            validation = $ManifestValidation
        }
        hardware_profile = $HardwareProfile
        qemu = [ordered]@{
            script = $RunScript
            args = @($QemuArgList)
            args_canonical_json = $qemuArgsCanonical
            args_sha256 = $qemuArgsSha256
            serial_tcp_port = $SerialTcpPort
            serial_write = [ordered]@{
                chunk_bytes = $SerialWriteChunkSize
                inter_chunk_delay_ms = $SerialWriteDelayMilliseconds
            }
            pid = $QemuPid
        }
        qemu_process = [ordered]@{
            before_teardown = $script:QemuProcessBeforeTeardown
            after_teardown = $script:QemuProcessAfterTeardown
            teardown_action = $script:QemuTeardownAction
        }
        serial_transport_failure = $script:SerialTransportFailure
        evidence_binding = [ordered]@{
            base_image_sha256 = $baseImageSha256
            candidate_artifact_sha256 = $artifactSha256
            candidate_manifest_sha256 = $manifestSha256
            hardware_profile_sha256 = $hardwareProfileSha256
            qemu_args_sha256 = $qemuArgsSha256
            serial_log_sha256 = $serialHash
            predicate_count = $Predicates.Count
            predicate_passed_count = @($Predicates.ToArray() | Where-Object { $_.passed }).Count
            predicate_failed_count = @($Predicates.ToArray() | Where-Object { -not $_.passed }).Count
            visual_evidence_count = $script:VisualEvidence.Count
            result = $FinalResult
        }
        commands = $executedCommandNames
        executed_commands = $executedCommandDetails
        predicates = @($Predicates.ToArray())
        serial_log = [ordered]@{
            path = $SerialLog
            sha256 = $serialHash
        }
        visual_evidence = @($script:VisualEvidence.ToArray())
        stderr_log = (Get-TextFileReport -Path $errLog)
        failures = @($Failures.ToArray())
    }

    $json = $report | ConvertTo-Json -Depth 20
    if (-not $forbiddenValuesRegistered) {
        Set-Content -LiteralPath $ReportPath -Value $json -Encoding UTF8
        $reportHash = Get-FileSha256OrNull -Path $ReportPath
        Set-Content -LiteralPath $ReportHashPath -Value "$reportHash  $ReportPath" -Encoding ASCII
        return
    }

    $jsonBytes = $null
    $readback = $null
    try {
        $encoding = [System.Text.UTF8Encoding]::new($false)
        $jsonBytes = $encoding.GetBytes($json)
        $matchedLabels = @(Get-MatchedForbiddenDynamicLabels -Bytes $jsonBytes)
        if ($matchedLabels.Count -gt 0) {
            Set-ForbiddenDynamicReportFailure `
                -Report $report `
                -PredicateIndex $forbiddenPredicateIndex `
                -Labels $matchedLabels
            [Array]::Clear($jsonBytes, 0, $jsonBytes.Length)
            $jsonBytes = $encoding.GetBytes(($report | ConvertTo-Json -Depth 20))
            Redact-ForbiddenDynamicValues -Bytes $jsonBytes
        }

        $remainingLabels = @(Get-MatchedForbiddenDynamicLabels -Bytes $jsonBytes)
        if ($remainingLabels.Count -gt 0) {
            Remove-Item -LiteralPath $ReportPath -Force -ErrorAction SilentlyContinue
            Remove-Item -LiteralPath $ReportHashPath -Force -ErrorAction SilentlyContinue
            if (-not $script:ReportSecurityTripwire) {
                $script:ReportSecurityTripwire = $remainingLabels -join ','
            }
            throw "Registered dynamic report value could not be redacted"
        }

        [System.IO.File]::WriteAllBytes($ReportPath, $jsonBytes)
        $readback = [System.IO.File]::ReadAllBytes($ReportPath)
        $readbackLabels = @(Get-MatchedForbiddenDynamicLabels -Bytes $readback)
        if ($readbackLabels.Count -gt 0) {
            Set-ForbiddenDynamicReportFailure `
                -Report $report `
                -PredicateIndex $forbiddenPredicateIndex `
                -Labels $readbackLabels
            [Array]::Clear($jsonBytes, 0, $jsonBytes.Length)
            $jsonBytes = $encoding.GetBytes(($report | ConvertTo-Json -Depth 20))
            Redact-ForbiddenDynamicValues -Bytes $jsonBytes
            [System.IO.File]::WriteAllBytes($ReportPath, $jsonBytes)
            [Array]::Clear($readback, 0, $readback.Length)
            $readback = [System.IO.File]::ReadAllBytes($ReportPath)
            $remainingLabels = @(Get-MatchedForbiddenDynamicLabels -Bytes $readback)
            if ($remainingLabels.Count -gt 0) {
                Remove-Item -LiteralPath $ReportPath -Force -ErrorAction SilentlyContinue
                Remove-Item -LiteralPath $ReportHashPath -Force -ErrorAction SilentlyContinue
                throw "Registered dynamic report value remained after redaction"
            }
        }

        $reportHash = Get-FileSha256OrNull -Path $ReportPath
        Set-Content -LiteralPath $ReportHashPath -Value "$reportHash  $ReportPath" -Encoding ASCII
    }
    finally {
        if ($null -ne $jsonBytes -and $jsonBytes.Length -gt 0) {
            [Array]::Clear($jsonBytes, 0, $jsonBytes.Length)
        }
        if ($null -ne $readback -and $readback.Length -gt 0) {
            [Array]::Clear($readback, 0, $readback.Length)
        }
        Clear-ReportForbiddenDynamicValues
    }
}
