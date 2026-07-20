param(
    [string]$OutputPath = "$PSScriptRoot\..\release\genesis-shell.png",
    [int]$TimeoutSeconds = 120,
    [switch]$KeepRunDir
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$Image = Join-Path $RepoRoot "release\raios-stage0.img"
$Qemu = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$FirmwareCode = "C:\Program Files\qemu\share\edk2-x86_64-code.fd"
$FirmwareVars = Join-Path $RepoRoot "release\ovmf_vars.fd"
$RunDir = Join-Path ([IO.Path]::GetTempPath()) ("raios-genesis-capture-{0}-{1}" -f $PID, [Guid]::NewGuid().ToString("N"))
$SerialLog = Join-Path $RunDir "serial.log"
$QemuErrorLog = Join-Path $RunDir "qemu.err"
$CodeCopy = Join-Path $RunDir "edk2-code.fd"
$VarsCopy = Join-Path $RunDir "ovmf-vars.fd"
$ImageCopy = Join-Path $RunDir "raios-stage0.img"
$PpmPath = Join-Path $RunDir "genesis-shell.ppm"
$PngPath = Join-Path $RunDir "genesis-shell.png"
$QemuProcess = $null
$Succeeded = $false
$ExitCode = 0

function Get-FreeTcpPort {
    $listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
    try {
        $listener.Start()
        return ([Net.IPEndPoint]$listener.LocalEndpoint).Port
    }
    finally {
        $listener.Stop()
    }
}

function Wait-ForLogText {
    param(
        [string]$Path,
        [string]$Needle,
        [int]$TimeoutSeconds
    )

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    do {
        if ($QemuProcess) {
            $QemuProcess.Refresh()
            if ($QemuProcess.HasExited) {
                $stderr = if (Test-Path -LiteralPath $QemuErrorLog) {
                    Get-Content -Raw -LiteralPath $QemuErrorLog -ErrorAction SilentlyContinue
                }
                else {
                    ""
                }
                throw "QEMU exited with code $($QemuProcess.ExitCode) before Genesis was ready. $stderr"
            }
        }

        if (Test-Path -LiteralPath $Path) {
            $content = Get-Content -Raw -LiteralPath $Path -ErrorAction SilentlyContinue
            if ($content -clike "*$Needle*") {
                return
            }
        }
        Start-Sleep -Milliseconds 250
    } while ([DateTime]::UtcNow -lt $deadline)

    throw "Timed out after $TimeoutSeconds seconds waiting for '$Needle' in $Path."
}

function Send-HmpCommand {
    param(
        [int]$Port,
        [string]$Command
    )

    $client = [Net.Sockets.TcpClient]::new()
    $connect = $client.BeginConnect("127.0.0.1", $Port, $null, $null)
    try {
        if (-not $connect.AsyncWaitHandle.WaitOne([TimeSpan]::FromSeconds(8))) {
            throw "Timed out connecting to HMP monitor TCP port $Port."
        }
        $client.EndConnect($connect)
        $client.SendTimeout = 4000
        $client.ReceiveTimeout = 500
        $stream = $client.GetStream()
        $buffer = [byte[]]::new(4096)

        while ($stream.DataAvailable) {
            $null = $stream.Read($buffer, 0, $buffer.Length)
        }

        $commandBytes = [Text.Encoding]::ASCII.GetBytes("$Command`n")
        $stream.Write($commandBytes, 0, $commandBytes.Length)
        $stream.Flush()

        $reply = [Text.StringBuilder]::new()
        $deadline = [DateTime]::UtcNow.AddSeconds(3)
        do {
            if ($stream.DataAvailable) {
                $count = $stream.Read($buffer, 0, $buffer.Length)
                if ($count -gt 0) {
                    [void]$reply.Append([Text.Encoding]::ASCII.GetString($buffer, 0, $count))
                }
            }
            Start-Sleep -Milliseconds 100
        } while ([DateTime]::UtcNow -lt $deadline)

        return $reply.ToString()
    }
    finally {
        $connect.AsyncWaitHandle.Dispose()
        $client.Dispose()
    }
}

function Read-PpmToken {
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
        throw "Invalid PPM header: expected a token at byte $start."
    }
    return [Text.Encoding]::ASCII.GetString($Buffer, $start, $Index.Value - $start)
}

function Convert-PpmToPng {
    param(
        [string]$PpmPath,
        [string]$PngPath
    )

    Add-Type -AssemblyName System.Drawing
    $bytes = [IO.File]::ReadAllBytes($PpmPath)
    $index = 0
    $indexRef = [ref]$index

    $magic = Read-PpmToken -Buffer $bytes -Index $indexRef
    $width = [int](Read-PpmToken -Buffer $bytes -Index $indexRef)
    $height = [int](Read-PpmToken -Buffer $bytes -Index $indexRef)
    $maxValue = [int](Read-PpmToken -Buffer $bytes -Index $indexRef)
    if ($magic -ne "P6" -or $width -le 0 -or $height -le 0 -or $maxValue -ne 255) {
        throw "Unsupported PPM header: magic=$magic width=$width height=$height max=$maxValue."
    }
    if ($indexRef.Value -ge $bytes.Length -or -not ([char]$bytes[$indexRef.Value] -match "\s")) {
        throw "Invalid PPM header: missing raster delimiter."
    }

    $indexRef.Value++
    if ($bytes[$indexRef.Value - 1] -eq 13 -and $indexRef.Value -lt $bytes.Length -and $bytes[$indexRef.Value] -eq 10) {
        $indexRef.Value++
    }
    $offset = $indexRef.Value
    $expectedBytes = [int64]$width * [int64]$height * 3
    if (($bytes.Length - $offset) -ne $expectedBytes) {
        throw "Invalid PPM raster length: expected $expectedBytes bytes, found $($bytes.Length - $offset)."
    }

    $bitmap = [Drawing.Bitmap]::new($width, $height, [Drawing.Imaging.PixelFormat]::Format24bppRgb)
    try {
        $rect = [Drawing.Rectangle]::new(0, 0, $width, $height)
        $data = $bitmap.LockBits($rect, [Drawing.Imaging.ImageLockMode]::WriteOnly, $bitmap.PixelFormat)
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
        $bitmap.Save($PngPath, [Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        $bitmap.Dispose()
    }
}

try {
    foreach ($requiredPath in @($Image, $Qemu, $FirmwareCode, $FirmwareVars)) {
        if (-not (Test-Path -LiteralPath $requiredPath -PathType Leaf)) {
            throw "Required capture input is missing: $requiredPath"
        }
    }
    if ($TimeoutSeconds -le 0) {
        throw "TimeoutSeconds must be greater than zero."
    }

    $SerialTcpPort = Get-FreeTcpPort
    do {
        $MonitorTcpPort = Get-FreeTcpPort
    } while ($MonitorTcpPort -eq $SerialTcpPort)

    New-Item -ItemType Directory -Path $RunDir | Out-Null
    Copy-Item -LiteralPath $FirmwareCode -Destination $CodeCopy
    Copy-Item -LiteralPath $FirmwareVars -Destination $VarsCopy
    Copy-Item -LiteralPath $Image -Destination $ImageCopy

    # The normal release image is fixed here deliberately. This harness never packages
    # provider credentials and never enables an unverified TLS path.
    $qemuArgs = @(
        "-machine", "q35",
        "-m", "512M",
        "-drive", "if=pflash,format=raw,readonly=on,file=$CodeCopy",
        "-drive", "if=pflash,format=raw,file=$VarsCopy",
        "-drive", "file=$ImageCopy,format=raw,if=ide",
        "-cpu", "max",
        "-netdev", "user,id=net0",
        "-device", "e1000,netdev=net0,mac=52:54:00:12:34:56",
        "-device", "qemu-xhci,id=xhci",
        "-device", "usb-kbd,bus=xhci.0,id=bootkbd",
        "-device", "usb-tablet,bus=xhci.0",
        "-chardev", "socket,id=seedserial,host=127.0.0.1,port=$SerialTcpPort,server=on,wait=off,logfile=$SerialLog,logappend=off",
        "-serial", "chardev:seedserial",
        "-display", "none",
        "-monitor", "tcp:127.0.0.1:$MonitorTcpPort,server,nowait",
        "-no-reboot"
    )

    $QemuProcess = Start-Process -FilePath $Qemu -ArgumentList $qemuArgs -PassThru -RedirectStandardError $QemuErrorLog -WindowStyle Hidden
    Wait-ForLogText -Path $SerialLog -Needle "status FRAMEBUFFER: READY" -TimeoutSeconds $TimeoutSeconds
    Start-Sleep -Seconds 1

    $hmpPath = $PpmPath.Replace("\", "/")
    $reply = Send-HmpCommand -Port $MonitorTcpPort -Command "screendump $hmpPath"
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    while (-not (Test-Path -LiteralPath $PpmPath) -and [DateTime]::UtcNow -lt $deadline) {
        Start-Sleep -Milliseconds 250
    }
    if (-not (Test-Path -LiteralPath $PpmPath -PathType Leaf)) {
        throw "HMP screendump did not create $PpmPath. Reply: $reply"
    }

    Convert-PpmToPng -PpmPath $PpmPath -PngPath $PngPath
    $outputDirectory = Split-Path -Parent $OutputPath
    if (-not [string]::IsNullOrWhiteSpace($outputDirectory)) {
        New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null
    }
    Copy-Item -LiteralPath $PngPath -Destination $OutputPath -Force

    $Succeeded = $true
    Write-Output "Genesis capture: $((Resolve-Path -LiteralPath $OutputPath).Path)"
    Write-Output "Image: $((Resolve-Path -LiteralPath $Image).Path)"
    Write-Output "QEMU PID: $($QemuProcess.Id); serial port: $SerialTcpPort; monitor port: $MonitorTcpPort"
}
catch {
    $ExitCode = 1
    Write-Error "Genesis capture failed: $($_.Exception.Message) Run directory preserved at $RunDir"
}
finally {
    if ($QemuProcess) {
        $QemuProcess.Refresh()
        if (-not $QemuProcess.HasExited) {
            Stop-Process -InputObject $QemuProcess -Force -ErrorAction SilentlyContinue
            $null = $QemuProcess.WaitForExit(5000)
        }
        $QemuProcess.Dispose()
    }

    if ($Succeeded -and -not $KeepRunDir -and (Test-Path -LiteralPath $RunDir)) {
        Remove-Item -LiteralPath $RunDir -Recurse -Force
    }
}

exit $ExitCode
