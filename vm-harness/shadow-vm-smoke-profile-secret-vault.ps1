# Secret Vault RR1 provisioning and recovery-unlock focused profile.
#
# The RR1 presentation exists on the host only as one transient PPM plus one
# byte array retained between the two boots. It never enters a serial command,
# predicate value, report field, permanent capture, or hash.

function Clear-Rr1Bytes {
    param([byte[]]$Bytes)

    if ($null -ne $Bytes -and $Bytes.Length -gt 0) {
        [Array]::Clear($Bytes, 0, $Bytes.Length)
    }
}

function Stop-Rr1VmForSecurityTripwire {
    if ($QemuPid) {
        Stop-Process -Id $QemuPid -Force -ErrorAction SilentlyContinue
    }
}

function Stop-Rr1VmForLogInspection {
    param([int]$QemuProcessId)

    Stop-Process -Id $QemuProcessId -Force -ErrorAction Stop
    if ($script:QemuProcess) {
        try {
            $script:QemuProcess.WaitForExit(5000) | Out-Null
        }
        catch {
        }
    }
    if (Get-Process -Id $QemuProcessId -ErrorAction SilentlyContinue) {
        throw "secret-vault QEMU process did not stop before log inspection"
    }
}

function Remove-Rr1SecretPpm {
    param([string]$Path)

    if (-not $Path -or -not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return
    }

    $stream = $null
    $zeroBuffer = [byte[]]::new(64KB)
    $erased = $false
    try {
        $stream = [System.IO.File]::Open(
            $Path,
            [System.IO.FileMode]::Open,
            [System.IO.FileAccess]::Write,
            [System.IO.FileShare]::None
        )
        $remaining = [int64]$stream.Length
        $stream.Position = 0
        while ($remaining -gt 0) {
            $count = [int][Math]::Min([int64]$zeroBuffer.Length, $remaining)
            $stream.Write($zeroBuffer, 0, $count)
            $remaining -= $count
        }
        $stream.Flush($true)
        $stream.SetLength(0)
        $stream.Flush($true)
        $erased = $true
    }
    catch {
        $erased = $false
    }
    finally {
        if ($stream) {
            $stream.Dispose()
        }
        Clear-Rr1Bytes -Bytes $zeroBuffer
    }

    Remove-Item -LiteralPath $Path -Force -ErrorAction SilentlyContinue
    if (-not $erased -or (Test-Path -LiteralPath $Path)) {
        Stop-Rr1VmForSecurityTripwire
        throw "SECURITY_TRIPWIRE_RR1_PPM_ERASE_FAILED"
    }
}

function Get-Rr1FontGlyphs {
    $fontPath = Join-Path $RepoRoot "seed-kernel\src\font_data.rs"
    $required = "-01R23456789abcdef"
    $glyphs = @{}
    $linePattern = '^\s*\[(?<bytes>(?:0x[0-9A-Fa-f]{2},?\s*){8})\],\s*// U\+(?<code>[0-9A-Fa-f]{4})'

    foreach ($line in Get-Content -LiteralPath $fontPath) {
        if ($line -notmatch $linePattern) {
            continue
        }
        $codeHex = $Matches.code
        $glyphText = $Matches.bytes
        $code = [Convert]::ToInt32($codeHex, 16)
        if ($required.IndexOf([char]$code) -lt 0) {
            continue
        }
        $byteMatches = [regex]::Matches($glyphText, '0x[0-9A-Fa-f]{2}')
        if ($byteMatches.Count -ne 8) {
            throw "RR1 font glyph has an invalid row count"
        }
        $rows = [byte[]]::new(8)
        for ($index = 0; $index -lt 8; $index++) {
            $rows[$index] = [Convert]::ToByte($byteMatches[$index].Value.Substring(2), 16)
        }
        $glyphs[$code] = $rows
    }

    foreach ($character in $required.ToCharArray()) {
        if (-not $glyphs.ContainsKey([int]$character)) {
            throw "RR1 font glyph source is incomplete"
        }
    }
    return $glyphs
}

function Get-Rr1MaskAlpha {
    param(
        [byte[]]$Rows,
        [int]$X,
        [int]$Y
    )

    if ((($Rows[$Y] -shr $X) -band 1) -eq 1) {
        return 255
    }
    return 0
}

function Get-Rr1SourceCoordinate {
    param(
        [int]$Pixel,
        [int]$Scale,
        [int]$Cells
    )

    $coordinate = [int]([Math]::Floor(($Pixel * 256 + 128) / $Scale)) - 128
    if ($coordinate -le 0) {
        return [pscustomobject]@{ Cell = 0; Fraction = 0 }
    }
    $clamped = [Math]::Min($coordinate, ($Cells - 1) * 256)
    return [pscustomobject]@{
        Cell = [int][Math]::Floor($clamped / 256)
        Fraction = [int]($clamped % 256)
    }
}

function New-Rr1GlyphTemplate {
    param(
        [byte[]]$Rows,
        [int]$Scale
    )

    $foreground = @(238, 241, 245)
    $background = @(29, 32, 38)
    $width = 8 * $Scale
    $height = 8 * $Scale
    $template = [byte[]]::new($width * $height * 3)
    $output = 0

    for ($pixelY = 0; $pixelY -lt $height; $pixelY++) {
        $sourceY = Get-Rr1SourceCoordinate -Pixel $pixelY -Scale $Scale -Cells 8
        for ($pixelX = 0; $pixelX -lt $width; $pixelX++) {
            $sourceX = Get-Rr1SourceCoordinate -Pixel $pixelX -Scale $Scale -Cells 8
            $x0 = $sourceX.Cell
            $y0 = $sourceY.Cell
            $x1 = [Math]::Min($x0 + 1, 7)
            $y1 = [Math]::Min($y0 + 1, 7)
            $fx = $sourceX.Fraction
            $fy = $sourceY.Fraction
            $ix = 255 - $fx
            $iy = 255 - $fy
            $sum = [int64](Get-Rr1MaskAlpha -Rows $Rows -X $x0 -Y $y0) * $ix * $iy
            $sum += [int64](Get-Rr1MaskAlpha -Rows $Rows -X $x1 -Y $y0) * $fx * $iy
            $sum += [int64](Get-Rr1MaskAlpha -Rows $Rows -X $x0 -Y $y1) * $ix * $fy
            $sum += [int64](Get-Rr1MaskAlpha -Rows $Rows -X $x1 -Y $y1) * $fx * $fy
            $alpha = [int][Math]::Floor($sum / (255 * 255))
            if ($alpha -ge 1 -and $alpha -le 48) {
                $alpha = [Math]::Min(255, $alpha * 2)
            }
            elseif ($alpha -gt 192) {
                $alpha = 255
            }

            for ($channel = 0; $channel -lt 3; $channel++) {
                $value = [int][Math]::Floor(
                    ($foreground[$channel] * $alpha + $background[$channel] * (255 - $alpha) + 127) / 255
                )
                $template[$output++] = [byte]$value
            }
        }
    }
    return ,$template
}

function Get-Rr1CellScore {
    param(
        [byte[]]$Ppm,
        [int]$RasterOffset,
        [int]$FrameWidth,
        [int]$PhysicalX,
        [int]$PhysicalY,
        [byte[]]$Template,
        [int]$Scale
    )

    $glyphWidth = 8 * $Scale
    $glyphHeight = 8 * $Scale
    $score = [int64]0
    $templateIndex = 0
    for ($row = 0; $row -lt $glyphHeight; $row++) {
        $source = $RasterOffset + (($PhysicalY + $row) * $FrameWidth + $PhysicalX) * 3
        for ($column = 0; $column -lt $glyphWidth * 3; $column++) {
            $score += [Math]::Abs([int]$Ppm[$source + $column] - [int]$Template[$templateIndex++])
        }
    }
    return $score
}

function Convert-Rr1HexNibble {
    param([byte]$Value)

    if ($Value -ge [byte][char]'0' -and $Value -le [byte][char]'9') {
        return $Value - [byte][char]'0'
    }
    if ($Value -ge [byte][char]'a' -and $Value -le [byte][char]'f') {
        return $Value - [byte][char]'a' + 10
    }
    throw "RR1 visual decoder produced a non-hex glyph"
}

function Test-Rr1StructureAndChecksum {
    param([byte[]]$Rr1)

    if ($Rr1.Length -ne 80 -or
        $Rr1[0] -ne [byte][char]'R' -or
        $Rr1[1] -ne [byte][char]'R' -or
        $Rr1[2] -ne [byte][char]'1' -or
        $Rr1[3] -ne [byte][char]'-') {
        return $false
    }

    $separators = @(12, 21, 30, 39, 48, 57, 66, 75)
    foreach ($separator in $separators) {
        if ($Rr1[$separator] -ne [byte][char]'-') {
            return $false
        }
    }

    $hex = [byte[]]::new(64)
    $key = [byte[]]::new(32)
    $digest = $null
    $hasher = $null
    try {
        $hexIndex = 0
        for ($index = 4; $index -le 74; $index++) {
            if ($separators -contains $index) {
                continue
            }
            $hex[$hexIndex++] = $Rr1[$index]
        }
        if ($hexIndex -ne 64) {
            return $false
        }
        for ($index = 0; $index -lt 32; $index++) {
            $high = Convert-Rr1HexNibble -Value $hex[$index * 2]
            $low = Convert-Rr1HexNibble -Value $hex[$index * 2 + 1]
            $key[$index] = [byte](($high -shl 4) -bor $low)
        }

        $expected0 = ((Convert-Rr1HexNibble -Value $Rr1[76]) -shl 4) -bor (Convert-Rr1HexNibble -Value $Rr1[77])
        $expected1 = ((Convert-Rr1HexNibble -Value $Rr1[78]) -shl 4) -bor (Convert-Rr1HexNibble -Value $Rr1[79])
        $hasher = [System.Security.Cryptography.IncrementalHash]::CreateHash(
            [System.Security.Cryptography.HashAlgorithmName]::SHA256
        )
        $hasher.AppendData([System.Text.Encoding]::ASCII.GetBytes("raios-recovery-v1"))
        $hasher.AppendData($key)
        $digest = $hasher.GetHashAndReset()
        return $digest[0] -eq $expected0 -and $digest[1] -eq $expected1
    }
    finally {
        if ($hasher) {
            $hasher.Dispose()
        }
        Clear-Rr1Bytes -Bytes $hex
        Clear-Rr1Bytes -Bytes $key
        Clear-Rr1Bytes -Bytes $digest
    }
}

function Read-Rr1FromEphemeralScreen {
    param(
        [int]$LogicalX,
        [int]$LogicalY,
        [int]$Scale
    )

    $ppmPath = Join-Path $RunDir "rr1-once.ppm"
    $script:Rr1SecretPpmPath = $ppmPath
    $ppm = $null
    $rr1 = $null
    try {
        Start-Sleep -Milliseconds 250
        $hmpPath = $ppmPath.Replace("\", "/")
        Send-QemuMonitorCommand -Command "screendump $hmpPath" -ReplyWaitMilliseconds 1000 | Out-Null
        $deadline = [DateTime]::UtcNow.AddSeconds(20)
        while (-not (Test-Path -LiteralPath $ppmPath -PathType Leaf) -and [DateTime]::UtcNow -lt $deadline) {
            Start-Sleep -Milliseconds 50
        }
        if (-not (Test-Path -LiteralPath $ppmPath -PathType Leaf)) {
            throw "RR1 transient screendump was not created"
        }

        $ppm = [System.IO.File]::ReadAllBytes($ppmPath)
        Remove-Rr1SecretPpm -Path $ppmPath

        $headerIndex = 0
        $headerIndexRef = [ref]$headerIndex
        $magic = Read-QemuPpmToken -Buffer $ppm -Index $headerIndexRef
        $frameWidth = [int](Read-QemuPpmToken -Buffer $ppm -Index $headerIndexRef)
        $frameHeight = [int](Read-QemuPpmToken -Buffer $ppm -Index $headerIndexRef)
        $maxValue = [int](Read-QemuPpmToken -Buffer $ppm -Index $headerIndexRef)
        if ($magic -ne "P6" -or $frameWidth -le 0 -or $frameHeight -le 0 -or $maxValue -ne 255) {
            throw "RR1 transient screendump has an unsupported PPM header"
        }
        if ($headerIndexRef.Value -ge $ppm.Length -or -not ([char]$ppm[$headerIndexRef.Value] -match "\s")) {
            throw "RR1 transient screendump is missing its raster delimiter"
        }
        $headerIndexRef.Value++
        if ($ppm[$headerIndexRef.Value - 1] -eq 13 -and
            $headerIndexRef.Value -lt $ppm.Length -and
            $ppm[$headerIndexRef.Value] -eq 10) {
            $headerIndexRef.Value++
        }
        $rasterOffset = $headerIndexRef.Value
        $expectedRasterLength = [int64]$frameWidth * [int64]$frameHeight * 3
        if (($ppm.Length - $rasterOffset) -ne $expectedRasterLength) {
            throw "RR1 transient screendump has an invalid raster length"
        }

        $fontGlyphs = Get-Rr1FontGlyphs
        $allowedHex = [System.Text.Encoding]::ASCII.GetBytes("0123456789abcdef")
        $templates = @{}
        foreach ($code in @([byte][char]'-', [byte][char]'1', [byte][char]'R') + $allowedHex) {
            if (-not $templates.ContainsKey([int]$code)) {
                $templates[[int]$code] = New-Rr1GlyphTemplate -Rows $fontGlyphs[[int]$code] -Scale $Scale
            }
        }

        $physicalX = $LogicalX * $Scale
        $physicalY = $LogicalY * $Scale
        $lastGlyphX = $physicalX + 39 * 9 * $Scale + 8 * $Scale
        $lastGlyphY = $physicalY + 16 * $Scale + 8 * $Scale
        if ($physicalX -lt 0 -or $physicalY -lt 0 -or $lastGlyphX -gt $frameWidth -or $lastGlyphY -gt $frameHeight) {
            throw "RR1 visual layout lies outside the framebuffer"
        }

        $rr1 = [byte[]]::new(80)
        $separatorPositions = @(3, 12, 21, 30, 39, 48, 57, 66, 75)
        for ($index = 0; $index -lt 80; $index++) {
            if ($index -eq 0 -or $index -eq 1) {
                $candidates = [byte[]]@([byte][char]'R')
            }
            elseif ($index -eq 2) {
                $candidates = [byte[]]@([byte][char]'1')
            }
            elseif ($separatorPositions -contains $index) {
                $candidates = [byte[]]@([byte][char]'-')
            }
            else {
                $candidates = $allowedHex
            }

            $row = [int][Math]::Floor($index / 40)
            $column = $index % 40
            $cellX = $physicalX + $column * 9 * $Scale
            $cellY = $physicalY + $row * 16 * $Scale
            $bestScore = [int64]::MaxValue
            $bestByte = [byte]0
            $bestCount = 0
            foreach ($candidate in $candidates) {
                $score = Get-Rr1CellScore `
                    -Ppm $ppm `
                    -RasterOffset $rasterOffset `
                    -FrameWidth $frameWidth `
                    -PhysicalX $cellX `
                    -PhysicalY $cellY `
                    -Template $templates[[int]$candidate] `
                    -Scale $Scale
                if ($score -lt $bestScore) {
                    $bestScore = $score
                    $bestByte = $candidate
                    $bestCount = 1
                }
                elseif ($score -eq $bestScore) {
                    $bestCount++
                }
            }
            if ($bestScore -ne 0 -or $bestCount -ne 1) {
                throw "RR1 bitmap-font decode failed at glyph position $index score=$bestScore ties=$bestCount"
            }
            $rr1[$index] = $bestByte
        }

        $valid = Test-Rr1StructureAndChecksum -Rr1 $rr1
        Add-Predicate `
            -Name "secret-vault:rr1_visual_structure_checksum" `
            -Expected "exact 80-glyph RR1 V1 bitmap-font presentation with valid checksum" `
            -Passed $valid `
            -Actual $(if ($valid) { "valid=true glyphs=80" } else { "valid=false glyphs=80" })
        if (-not $valid) {
            throw "RR1 visual structure or checksum validation failed"
        }
        return ,$rr1
    }
    catch {
        Clear-Rr1Bytes -Bytes $rr1
        throw
    }
    finally {
        Clear-Rr1Bytes -Bytes $ppm
        Remove-Rr1SecretPpm -Path $ppmPath
    }
}

function Send-Rr1HmpKey {
    param([string]$KeyName)

    Send-QemuMonitorCommand -Command "sendkey $KeyName 60" -ReplyWaitMilliseconds 125 | Out-Null
}

function Invoke-VaultVisibleAction {
    for ($index = 0; $index -lt 3; $index++) {
        Send-Rr1HmpKey -KeyName "tab"
    }
    Send-Rr1HmpKey -KeyName "ret"
    Start-Sleep -Milliseconds 150
    Send-Rr1HmpKey -KeyName "tab"
    Send-Rr1HmpKey -KeyName "ret"
    Start-Sleep -Milliseconds 250
    Send-Rr1HmpKey -KeyName "ret"
}

function Send-Rr1ViaUsbKeyboard {
    param([byte[]]$Rr1)

    if ($null -eq $Rr1 -or $Rr1.Length -ne 80) {
        throw "RR1 keyboard input requires the complete in-memory value"
    }
    foreach ($value in $Rr1) {
        if ($value -eq [byte][char]'R') {
            $keyName = "shift-r"
        }
        elseif ($value -eq [byte][char]'-') {
            $keyName = "minus"
        }
        elseif (($value -ge [byte][char]'0' -and $value -le [byte][char]'9') -or
            ($value -ge [byte][char]'a' -and $value -le [byte][char]'f')) {
            $keyName = ([char]$value).ToString()
        }
        else {
            throw "RR1 keyboard input contains an unsupported byte"
        }
        Send-Rr1HmpKey -KeyName $keyName
    }
    Send-Rr1HmpKey -KeyName "ret"
}

function Get-SerialMarkerCount {
    param(
        [string]$Path,
        [string]$Marker
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return 0
    }
    $content = Get-Content -LiteralPath $Path -Raw -ErrorAction Stop
    return [regex]::Matches($content, [regex]::Escape($Marker)).Count
}

function Assert-Rr1NotInSerial {
    param(
        [string]$Name,
        [string]$Path
    )

    $bytes = [System.IO.File]::ReadAllBytes($Path)
    $prefix = [System.Text.Encoding]::ASCII.GetBytes("RR1-")
    $matches = New-Object System.Collections.Generic.List[int]
    try {
        for ($offset = 0; $offset -le $bytes.Length - $prefix.Length; $offset++) {
            $found = $true
            for ($index = 0; $index -lt $prefix.Length; $index++) {
                if ($bytes[$offset + $index] -ne $prefix[$index]) {
                    $found = $false
                    break
                }
            }
            if ($found) {
                $matches.Add($offset) | Out-Null
            }
        }

        if ($matches.Count -gt 0) {
            foreach ($offset in $matches) {
                $count = [Math]::Min(80, $bytes.Length - $offset)
                for ($index = 0; $index -lt $count; $index++) {
                    $bytes[$offset + $index] = [byte][char]'*'
                }
            }
            try {
                [System.IO.File]::WriteAllBytes($Path, $bytes)
            }
            catch {
                Stop-Rr1VmForSecurityTripwire
                throw "SECURITY_TRIPWIRE_RR1_SERIAL_REDACTION_FAILED"
            }
            $script:SerialLogCachePath = $null
            $script:SerialLogCacheLength = [int64]-1
            $script:SerialLogCacheWriteTicks = [int64]-1
            $script:SerialLogCacheContent = $null
        }

        $passed = $matches.Count -eq 0
        Add-Predicate `
            -Name $Name `
            -Expected "serial contains no RR1 presentation prefix" `
            -Passed $passed `
            -Actual $(if ($passed) { "absent=true count=0" } else { "absent=false redacted=true count=$($matches.Count)" })
        if (-not $passed) {
            throw "SECURITY_TRIPWIRE_RR1_SERIAL_LEAK_REDACTED"
        }
    }
    finally {
        Clear-Rr1Bytes -Bytes $bytes
        Clear-Rr1Bytes -Bytes $prefix
    }
}

function Assert-VaultSerialOutcome {
    param(
        [string]$Name,
        [string]$SuccessMarker,
        [string]$RejectedMarker,
        [int]$TimeoutSeconds
    )

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    do {
        $content = [string](Get-SerialLogContent -Path $SerialLog)
        if ($content.IndexOf($SuccessMarker, [StringComparison]::Ordinal) -ge 0) {
            Add-Predicate -Name $Name -Expected "serial_contains:$SuccessMarker" -Passed $true -Actual "accepted=true"
            return
        }
        $match = [regex]::Match(
            $content,
            [regex]::Escape($RejectedMarker) + ' reason=(?<reason>[a-z_]+)'
        )
        if (-not $match.Success) {
            $match = [regex]::Match(
                $content,
                'VAULT_RR1_INPUT_REJECTED reason=(?<reason>[a-z_]+)'
            )
        }
        if ($match.Success) {
            $reason = $match.Groups['reason'].Value
            Add-Predicate -Name $Name -Expected "serial_contains:$SuccessMarker" -Passed $false -Actual "accepted=false reason=$reason"
            throw "Vault guest rejected the physical RR1 input: $reason"
        }
        Start-Sleep -Milliseconds 100
    } while ([DateTime]::UtcNow -lt $deadline)

    Add-Predicate -Name $Name -Expected "serial_contains:$SuccessMarker" -Passed $false -Actual "accepted=false reason=timeout"
    throw "Timed out waiting for the Vault RR1 outcome"
}

function Get-Rr1DisplayLayoutFromSerial {
    param([string]$Path)

    $content = Get-Content -LiteralPath $Path -Raw -ErrorAction Stop
    $pattern = 'VAULT_RR1_DISPLAY_READY layout=v1 x=(\d+) y=(\d+) scale=(\d+) rows=(\d+) cols=(\d+)'
    $found = [regex]::Matches($content, $pattern)
    if ($found.Count -ne 1) {
        Add-Predicate `
            -Name "secret-vault:rr1_displayed_exactly_once" `
            -Expected "one non-secret RR1 display-ready marker" `
            -Passed $false `
            -Actual "count=$($found.Count)"
        throw "Expected exactly one RR1 display-ready marker"
    }
    $match = $found[0]
    $layout = [pscustomobject]@{
        X = [int]$match.Groups[1].Value
        Y = [int]$match.Groups[2].Value
        Scale = [int]$match.Groups[3].Value
        Rows = [int]$match.Groups[4].Value
        Columns = [int]$match.Groups[5].Value
    }
    $valid = $layout.X -ge 0 -and $layout.Y -ge 0 -and $layout.Scale -eq 2 -and
        $layout.Rows -eq 2 -and $layout.Columns -eq 40
    Add-Predicate `
        -Name "secret-vault:rr1_displayed_exactly_once" `
        -Expected "one V1 marker for two 40-glyph rows at scale 2" `
        -Passed $valid `
        -Actual "valid=$($valid.ToString().ToLowerInvariant()) count=1 rows=$($layout.Rows) cols=$($layout.Columns) scale=$($layout.Scale)"
    if (-not $valid) {
        throw "RR1 display-ready layout is not V1"
    }
    return $layout
}

if (-not $StructuredStoreDiskImage -or -not $StructuredStoreFixture -or -not $PersistDiskImage) {
    throw "secret-vault profile requires separate C1 store and valid-a BOOTCTL fixtures"
}

$fixtureReady = (
    [bool]$StructuredStoreFixture.valid -and
    [bool]$StructuredStoreFixture.disposable_qemu_only -and
    $StructuredStoreFixture.store_state -eq "empty_unformatted" -and
    (Test-Path -LiteralPath $PersistDiskImage)
)
Add-Predicate `
    -Name "secret-vault:host_fixture_ready" `
    -Expected "dedicated empty C1 QEMU store plus separate valid-a BOOTCTL" `
    -Passed $fixtureReady `
    -Actual $(if ($fixtureReady) { "ready=true count=2" } else { "ready=false count=0" })
if (-not $fixtureReady) {
    throw "Secret Vault fixtures are not the dedicated disposable pair"
}

$rr1 = $null
$script:Rr1SecretPpmPath = Join-Path $RunDir "rr1-once.ppm"
try {
    Assert-LogContains `
        -Name "secret-vault:boot1:usb_keyboard_ready" `
        -Needle "usb-hid: keyboard ready on slot" `
        -TimeoutSeconds $TimeoutSeconds
    foreach ($marker in @(
        "C1_STRUCTURED_STORE_FIXTURE_ACCEPTED",
        "C1_STRUCTURED_STORE_FORMAT_OPEN_OK",
        "C1_VAULT_COMPLETE_REPLAY_BOUND",
        "C1_VAULT_CORE_POLICY_BOUND"
    )) {
        Assert-LogContains -Name "secret-vault:boot1:$marker" -Needle $marker -TimeoutSeconds $TimeoutSeconds
    }

    $usbBeforeStart = Get-SerialMarkerCount -Path $SerialLog -Marker "usb input batch:"
    Invoke-VaultVisibleAction
    Assert-LogContains `
        -Name "secret-vault:boot1:rr1_display_ready" `
        -Needle "VAULT_RR1_DISPLAY_READY layout=v1" `
        -TimeoutSeconds $TimeoutSeconds
    $usbAfterStart = Get-SerialMarkerCount -Path $SerialLog -Marker "usb input batch:"
    $startUsedUsb = $usbAfterStart -gt $usbBeforeStart
    Add-Predicate `
        -Name "secret-vault:boot1:visible_action_uses_usb_hid" `
        -Expected "Tab/Enter Vault navigation increments USB input batches" `
        -Passed $startUsedUsb `
        -Actual "passed=$($startUsedUsb.ToString().ToLowerInvariant()) before=$usbBeforeStart after=$usbAfterStart"
    if (-not $startUsedUsb) {
        throw "Vault visible action did not traverse USB HID input"
    }

    $layout = Get-Rr1DisplayLayoutFromSerial -Path $SerialLog
    try {
        $rr1 = Read-Rr1FromEphemeralScreen -LogicalX $layout.X -LogicalY $layout.Y -Scale $layout.Scale
    }
    catch {
        if ($_.Exception.Message -notlike "RR1 bitmap-font decode failed at glyph position *") {
            throw
        }
        Start-Sleep -Milliseconds 250
        $rr1 = Read-Rr1FromEphemeralScreen -LogicalX $layout.X -LogicalY $layout.Y -Scale $layout.Scale
    }
    if (Test-Path -LiteralPath $script:Rr1SecretPpmPath) {
        throw "SECURITY_TRIPWIRE_RR1_PPM_RETAINED"
    }

    $usbBeforeReentry = Get-SerialMarkerCount -Path $SerialLog -Marker "usb input batch:"
    # A distinct physical Space acknowledges the already captured one-time
    # display; trailing Enter reports from opening Vault cannot dismiss it.
    Send-Rr1HmpKey -KeyName "spc"
    Assert-LogContains `
        -Name "secret-vault:boot1:confirmation_ready" `
        -Needle "VAULT_RR1_CONFIRMATION_READY" `
        -TimeoutSeconds $TimeoutSeconds
    Start-Sleep -Milliseconds 150
    Send-Rr1ViaUsbKeyboard -Rr1 $rr1
    Assert-VaultSerialOutcome `
        -Name "secret-vault:boot1:rr1_confirmed" `
        -SuccessMarker "VAULT_RR1_CONFIRMED" `
        -RejectedMarker "VAULT_RR1_REJECTED" `
        -TimeoutSeconds $TimeoutSeconds
    Assert-LogContains `
        -Name "secret-vault:boot1:wrapper_committed_readback" `
        -Needle "C1_VAULT_RECOVERY_WRAPPER_COMMITTED generation=1 readback=verified" `
        -TimeoutSeconds $TimeoutSeconds
    $usbAfterReentry = Get-SerialMarkerCount -Path $SerialLog -Marker "usb input batch:"
    $reentryUsedUsb = $usbAfterReentry -gt $usbBeforeReentry
    Add-Predicate `
        -Name "secret-vault:boot1:rr1_reentry_uses_usb_hid" `
        -Expected "RR1 re-entry increments USB input batches" `
        -Passed $reentryUsedUsb `
        -Actual "passed=$($reentryUsedUsb.ToString().ToLowerInvariant()) before=$usbBeforeReentry after=$usbAfterReentry"
    if (-not $reentryUsedUsb) {
        throw "RR1 confirmation did not traverse USB HID input"
    }
    $firstBootLog = $SerialLog
    if (-not $QemuPid) {
        throw "secret-vault profile cannot reboot without the first QEMU process"
    }
    $firstQemuPid = $QemuPid
    Stop-Rr1VmForLogInspection -QemuProcessId $firstQemuPid
    Assert-Rr1NotInSerial -Name "secret-vault:boot1:rr1_absent_from_serial" -Path $firstBootLog

    $rebootLog = Join-Path $RunDir "serial-secret-vault-reboot.log"
    $rebootParams = $runParams.Clone()
    $rebootParams.StopExisting = $false
    $rebootParams.SerialLog = $rebootLog
    $rebootOutput = & $RunScript @rebootParams
    $SerialLog = $rebootLog
    $script:SerialLogCachePath = $null
    $script:SerialLogCacheLength = [int64]-1
    $script:SerialLogCacheWriteTicks = [int64]-1
    $script:SerialLogCacheContent = $null
    $QemuPid = $null
    foreach ($line in $rebootOutput) {
        if ($line -match '^qemu pid:\s*(\d+)') {
            $QemuPid = [int]$Matches[1]
        }
    }
    if (-not $QemuPid) {
        throw "secret-vault reboot did not return a QEMU pid"
    }
    try {
        $script:QemuProcess = Get-Process -Id $QemuPid -ErrorAction Stop
    }
    catch {
        $script:QemuProcess = $null
    }

    Assert-LogContains -Name "secret-vault:boot2:serial_ready" -Needle "SERIAL CONSOLE READY" -TimeoutSeconds $TimeoutSeconds
    Assert-LogContains `
        -Name "secret-vault:boot2:usb_keyboard_ready" `
        -Needle "usb-hid: keyboard ready on slot" `
        -TimeoutSeconds $TimeoutSeconds
    Assert-LogContains `
        -Name "secret-vault:boot2:complete_replay_bound" `
        -Needle "C1_VAULT_COMPLETE_REPLAY_BOUND" `
        -TimeoutSeconds $TimeoutSeconds
    Assert-LogContains `
        -Name "secret-vault:boot2:core_policy_bound" `
        -Needle "C1_VAULT_CORE_POLICY_BOUND" `
        -TimeoutSeconds $TimeoutSeconds
    Assert-LogContains `
        -Name "secret-vault:boot2:wrapper_replayed" `
        -Needle "C1_VAULT_RECOVERY_WRAPPER_REPLAYED generation=1" `
        -TimeoutSeconds $TimeoutSeconds

    $usbBeforeUnlockAction = Get-SerialMarkerCount -Path $SerialLog -Marker "usb input batch:"
    Invoke-VaultVisibleAction
    Assert-LogContains `
        -Name "secret-vault:boot2:recovery_unlock_ready" `
        -Needle "VAULT_RECOVERY_UNLOCK_READY" `
        -TimeoutSeconds $TimeoutSeconds
    $displayCountBoot2 = Get-SerialMarkerCount -Path $SerialLog -Marker "VAULT_RR1_DISPLAY_READY"
    $notRedisplayed = $displayCountBoot2 -eq 0
    Add-Predicate `
        -Name "secret-vault:boot2:rr1_not_redisplayed" `
        -Expected "recovery unlock never redisplays RR1" `
        -Passed $notRedisplayed `
        -Actual "passed=$($notRedisplayed.ToString().ToLowerInvariant()) count=$displayCountBoot2"
    if (-not $notRedisplayed) {
        throw "RR1 was redisplayed during recovery unlock"
    }
    $usbAfterUnlockAction = Get-SerialMarkerCount -Path $SerialLog -Marker "usb input batch:"
    $unlockActionUsedUsb = $usbAfterUnlockAction -gt $usbBeforeUnlockAction
    Add-Predicate `
        -Name "secret-vault:boot2:visible_action_uses_usb_hid" `
        -Expected "Tab/Enter recovery-unlock navigation increments USB input batches" `
        -Passed $unlockActionUsedUsb `
        -Actual "passed=$($unlockActionUsedUsb.ToString().ToLowerInvariant()) before=$usbBeforeUnlockAction after=$usbAfterUnlockAction"
    if (-not $unlockActionUsedUsb) {
        throw "Recovery unlock visible action did not traverse USB HID input"
    }

    $usbBeforeUnlock = Get-SerialMarkerCount -Path $SerialLog -Marker "usb input batch:"
    Send-Rr1ViaUsbKeyboard -Rr1 $rr1
    Assert-VaultSerialOutcome `
        -Name "secret-vault:boot2:broker_unlocked" `
        -SuccessMarker "VAULT_RR1_UNLOCKED" `
        -RejectedMarker "VAULT_RR1_UNLOCK_REJECTED" `
        -TimeoutSeconds $TimeoutSeconds
    $usbAfterUnlock = Get-SerialMarkerCount -Path $SerialLog -Marker "usb input batch:"
    $unlockUsedUsb = $usbAfterUnlock -gt $usbBeforeUnlock
    Add-Predicate `
        -Name "secret-vault:boot2:rr1_unlock_uses_usb_hid" `
        -Expected "recovery RR1 input increments USB input batches" `
        -Passed $unlockUsedUsb `
        -Actual "passed=$($unlockUsedUsb.ToString().ToLowerInvariant()) before=$usbBeforeUnlock after=$usbAfterUnlock"
    if (-not $unlockUsedUsb) {
        throw "Recovery RR1 input did not traverse USB HID input"
    }
    Stop-Rr1VmForLogInspection -QemuProcessId $QemuPid
    Assert-Rr1NotInSerial -Name "secret-vault:boot2:rr1_absent_from_serial" -Path $SerialLog

    Clear-Rr1Bytes -Bytes $rr1
    $rr1 = $null

    $combinedLog = Join-Path $RunDir "serial-secret-vault-combined.log"
    $firstBootContent = Get-Content -LiteralPath $firstBootLog -Raw -ErrorAction Stop
    $rebootContent = Get-Content -LiteralPath $rebootLog -Raw -ErrorAction Stop
    Set-Content -LiteralPath $combinedLog -Value ($firstBootContent + [Environment]::NewLine + $rebootContent) -Encoding UTF8
    $SerialLog = $combinedLog
    $script:SerialLogCachePath = $null
    $script:SerialLogCacheLength = [int64]-1
    $script:SerialLogCacheWriteTicks = [int64]-1
    $script:SerialLogCacheContent = $null
}
finally {
    Clear-Rr1Bytes -Bytes $rr1
    $rr1 = $null
    Remove-Rr1SecretPpm -Path $script:Rr1SecretPpmPath
}
