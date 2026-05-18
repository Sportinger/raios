param(
    [string]$OutputDir = "$PSScriptRoot\..\docs\assets\screenshots",
    [string]$Prompt = "Reply with exactly five words: raisOS boots, networks, chats directly.",
    [int]$SerialTcpPort = 4591,
    [int]$MonitorTcpPort = 4592,
    [int]$VncDisplay = 79,
    [int]$TimeoutSeconds = 120,
    [switch]$KeepRunDir
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$RunId = "raisos-readme-screenshots-{0:yyyyMMdd-HHmmss}-{1}" -f (Get-Date), $PID
$RunDir = Join-Path $env:TEMP $RunId
$TempImage = Join-Path $RunDir "raisos-stage0-local-openai.img"
$SerialLog = Join-Path $RunDir "serial.log"
$ErrLog = Join-Path $RunDir "qemu.err"
$Code = Join-Path $RunDir "edk2-code.fd"
$Vars = Join-Path $RunDir "ovmf-vars.fd"
$QemuPid = $null
$Succeeded = $false

function Wait-ForLogText {
    param(
        [string]$Path,
        [string]$Needle,
        [int]$TimeoutSeconds
    )

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    do {
        if (Test-Path -LiteralPath $Path) {
            $content = Get-Content -Raw -LiteralPath $Path -ErrorAction SilentlyContinue
            if ($content -clike "*$Needle*") {
                return
            }
        }
        Start-Sleep -Milliseconds 250
    } while ([DateTime]::UtcNow -lt $deadline)

    throw "Timed out waiting for '$Needle' in $Path"
}

function Send-TcpText {
    param(
        [int]$Port,
        [string]$Text
    )

    $client = [System.Net.Sockets.TcpClient]::new()
    $connect = $client.BeginConnect("127.0.0.1", $Port, $null, $null)
    if (-not $connect.AsyncWaitHandle.WaitOne([TimeSpan]::FromSeconds(8))) {
        $client.Close()
        throw "Timed out connecting to TCP port $Port"
    }
    $client.EndConnect($connect)

    try {
        $client.SendTimeout = 4000
        $stream = $client.GetStream()
        $bytes = [Text.Encoding]::ASCII.GetBytes($Text)
        $stream.Write($bytes, 0, $bytes.Length)
        $stream.Flush()
    }
    finally {
        $client.Close()
    }
}

function Send-Serial {
    param([string]$Text)

    $client = [System.Net.Sockets.TcpClient]::new()
    $client.NoDelay = $true
    $connect = $client.BeginConnect("127.0.0.1", $SerialTcpPort, $null, $null)
    if (-not $connect.AsyncWaitHandle.WaitOne([TimeSpan]::FromSeconds(8))) {
        $client.Close()
        throw "Timed out connecting to serial TCP port $SerialTcpPort"
    }
    $client.EndConnect($connect)

    try {
        $client.SendTimeout = 4000
        $stream = $client.GetStream()
        $bytes = [Text.Encoding]::ASCII.GetBytes($Text)
        foreach ($byte in $bytes) {
            $stream.WriteByte($byte)
            Start-Sleep -Milliseconds 8
        }
        $stream.Flush()
        Start-Sleep -Milliseconds 500
    }
    finally {
        $client.Close()
    }
}

function Send-Hmp {
    param([string]$Command)

    $client = [System.Net.Sockets.TcpClient]::new()
    $connect = $client.BeginConnect("127.0.0.1", $MonitorTcpPort, $null, $null)
    if (-not $connect.AsyncWaitHandle.WaitOne([TimeSpan]::FromSeconds(8))) {
        $client.Close()
        throw "Timed out connecting to HMP monitor TCP port $MonitorTcpPort"
    }
    $client.EndConnect($connect)

    try {
        $client.SendTimeout = 4000
        $client.ReceiveTimeout = 500
        $stream = $client.GetStream()
        $drain = New-Object byte[] 4096
        try {
            while ($stream.DataAvailable) {
                $null = $stream.Read($drain, 0, $drain.Length)
            }
        }
        catch {
        }

        $bytes = [Text.Encoding]::ASCII.GetBytes("$Command`n")
        $stream.Write($bytes, 0, $bytes.Length)
        $stream.Flush()

        $reply = [Text.StringBuilder]::new()
        $deadline = [DateTime]::UtcNow.AddSeconds(3)
        do {
            try {
                if ($stream.DataAvailable) {
                    $count = $stream.Read($drain, 0, $drain.Length)
                    if ($count -gt 0) {
                        [void]$reply.Append([Text.Encoding]::ASCII.GetString($drain, 0, $count))
                    }
                }
            }
            catch {
            }
            Start-Sleep -Milliseconds 100
        } while ([DateTime]::UtcNow -lt $deadline)

        return $reply.ToString()
    }
    finally {
        $client.Close()
    }
}

function Convert-PpmToPng {
    param(
        [string]$PpmPath,
        [string]$PngPath
    )

    Add-Type -AssemblyName System.Drawing
    $bytes = [IO.File]::ReadAllBytes($PpmPath)
    $idx = 0

    function Read-Token {
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
        [Text.Encoding]::ASCII.GetString($Buffer, $start, $Index.Value - $start)
    }

    $ref = [ref]$idx
    $magic = Read-Token -Buffer $bytes -Index $ref
    if ($magic -ne "P6") {
        throw "Unsupported PPM magic '$magic'"
    }

    $width = [int](Read-Token -Buffer $bytes -Index $ref)
    $height = [int](Read-Token -Buffer $bytes -Index $ref)
    $max = [int](Read-Token -Buffer $bytes -Index $ref)
    if ($max -ne 255) {
        throw "Unsupported PPM max value '$max'"
    }

    while ($ref.Value -lt $bytes.Length -and [char]$bytes[$ref.Value] -match "\s") {
        $ref.Value++
    }
    $offset = $ref.Value

    $bitmap = [Drawing.Bitmap]::new($width, $height, [Drawing.Imaging.PixelFormat]::Format24bppRgb)
    $rect = [Drawing.Rectangle]::new(0, 0, $width, $height)
    $data = $bitmap.LockBits($rect, [Drawing.Imaging.ImageLockMode]::WriteOnly, $bitmap.PixelFormat)
    try {
        $stride = [Math]::Abs($data.Stride)
        $buffer = New-Object byte[] ($stride * $height)
        $src = $offset
        for ($y = 0; $y -lt $height; $y++) {
            $dst = $y * $stride
            for ($x = 0; $x -lt $width; $x++) {
                $buffer[$dst++] = $bytes[$src + 2]
                $buffer[$dst++] = $bytes[$src + 1]
                $buffer[$dst++] = $bytes[$src]
                $src += 3
            }
        }
        [Runtime.InteropServices.Marshal]::Copy($buffer, 0, $data.Scan0, $buffer.Length)
    }
    finally {
        $bitmap.UnlockBits($data)
    }

    try {
        $bitmap.Save($PngPath, [Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        $bitmap.Dispose()
    }
}

function Capture-Screen {
    param([string]$Name)

    $ppm = Join-Path $RunDir "$Name.ppm"
    $png = Join-Path $OutputDir "$Name.png"
    Remove-Item -LiteralPath $ppm, $png -Force -ErrorAction SilentlyContinue

    $reply = Send-Hmp -Command "screendump $($ppm.Replace('\', '/'))"
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    do {
        Start-Sleep -Milliseconds 250
    } while ((-not (Test-Path -LiteralPath $ppm)) -and [DateTime]::UtcNow -lt $deadline)

    if (-not (Test-Path -LiteralPath $ppm)) {
        throw "screendump did not create $ppm. HMP reply: $reply"
    }

    Convert-PpmToPng -PpmPath $ppm -PngPath $png
    Get-Item -LiteralPath $png | Select-Object FullName, Length
}

try {
    if ([string]::IsNullOrWhiteSpace([Environment]::GetEnvironmentVariable("OPENAI_API_KEY", "Process"))) {
        throw "OPENAI_API_KEY is not set in the current process."
    }

    New-Item -ItemType Directory -Force -Path $RunDir, $OutputDir | Out-Null

    & (Join-Path $RepoRoot "scripts\package-stage0.ps1") `
        -Profile release `
        -Image $TempImage `
        -UseTempEsp `
        -EmbedOpenAiApiKeyFromEnv `
        -AllowUnverifiedOpenAiTls
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    Copy-Item -LiteralPath "C:\Program Files\qemu\share\edk2-x86_64-code.fd" -Destination $Code -Force
    Copy-Item -LiteralPath (Join-Path $RepoRoot "release\ovmf_vars.fd") -Destination $Vars -Force
    Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force

    $qemu = "C:\Program Files\qemu\qemu-system-x86_64.exe"
    $qemuArgs = @(
        "-machine", "q35",
        "-m", "512M",
        "-drive", "if=pflash,format=raw,readonly=on,file=$Code",
        "-drive", "if=pflash,format=raw,file=$Vars",
        "-drive", "file=$TempImage,format=raw,if=ide",
        "-cpu", "max",
        "-netdev", "user,id=net0",
        "-device", "e1000,netdev=net0,mac=52:54:00:12:34:56",
        "-device", "qemu-xhci,id=xhci",
        "-device", "usb-kbd,bus=xhci.0",
        "-device", "usb-tablet,bus=xhci.0",
        "-chardev", "socket,id=seedserial,host=127.0.0.1,port=$SerialTcpPort,server=on,wait=off,logfile=$SerialLog,logappend=off",
        "-serial", "chardev:seedserial",
        "-display", "none",
        "-vnc", "127.0.0.1:$VncDisplay",
        "-monitor", "tcp:127.0.0.1:$MonitorTcpPort,server,nowait",
        "-no-reboot"
    )

    $process = Start-Process `
        -FilePath $qemu `
        -ArgumentList $qemuArgs `
        -PassThru `
        -RedirectStandardError $ErrLog `
        -WindowStyle Hidden
    $QemuPid = $process.Id

    Wait-ForLogText -Path $SerialLog -Needle "Default provider loaded: OPENAI API key set" -TimeoutSeconds $TimeoutSeconds
    Wait-ForLogText -Path $SerialLog -Needle "status NETWORK: CONFIGURED" -TimeoutSeconds $TimeoutSeconds
    Start-Sleep -Seconds 1
    Capture-Screen -Name "raisos-home"

    [void](Send-Hmp -Command "sendkey tab")
    [void](Send-Hmp -Command "sendkey tab")
    [void](Send-Hmp -Command "sendkey ret")
    Start-Sleep -Seconds 1
    Send-Serial -Text "help`r"
    Wait-ForLogText -Path $SerialLog -Needle "AGENT RAW: system.snapshot service.inventory module.load_ephemeral" -TimeoutSeconds 10
    Send-Serial -Text "status`r"
    Wait-ForLogText -Path $SerialLog -Needle "USB-XHCI: READY    WIFI: MISSING    NETWORK: CONFIGURED    INPUT: READY" -TimeoutSeconds 10
    Send-Serial -Text "devices`r"
    Wait-ForLogText -Path $SerialLog -Needle "FRAMEBUFFER: READY - 1280x800 PITCH 5120" -TimeoutSeconds 10
    Start-Sleep -Seconds 1
    Capture-Screen -Name "raisos-console-status"

    Send-Serial -Text "setup`r"
    Wait-ForLogText -Path $SerialLog -Needle "SETUP" -TimeoutSeconds 10
    Start-Sleep -Seconds 1
    Capture-Screen -Name "raisos-settings"

    Send-Serial -Text "q"
    Start-Sleep -Milliseconds 800
    Send-Serial -Text "ask $Prompt`r"
    Wait-ForLogText -Path $SerialLog -Needle "OPENAI DIRECT REQUEST 1 STARTED" -TimeoutSeconds 30
    Wait-ForLogText -Path $SerialLog -Needle "openai: HTTPS request sent" -TimeoutSeconds $TimeoutSeconds
    Wait-ForLogText -Path $SerialLog -Needle "OPENAI:" -TimeoutSeconds $TimeoutSeconds
    Start-Sleep -Seconds 2
    Capture-Screen -Name "raisos-openai-chat"
    $Succeeded = $true
}
finally {
    if ($QemuPid) {
        Stop-Process -Id $QemuPid -Force -ErrorAction SilentlyContinue
    }
    Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force
    Remove-Item -LiteralPath $TempImage -Force -ErrorAction SilentlyContinue
    if ((-not $KeepRunDir) -and $Succeeded) {
        Remove-Item -LiteralPath $RunDir -Recurse -Force -ErrorAction SilentlyContinue
    }
    Remove-Item -LiteralPath (Join-Path $RepoRoot "target\x86_64-seed") -Recurse -Force -ErrorAction SilentlyContinue
}
