param(
    [string]$HostName = "127.0.0.1",
    [int]$Port = 4555,
    [string]$Ask = "",
    [string]$ResponsePrefix = "HOST BRIDGE OK",
    [switch]$Once,
    [int]$ConnectTimeoutMs = 10000,
    [int]$AskStartupDelayMs = 3000,
    [int]$RequestTimeoutSeconds = 30
)

$ErrorActionPreference = "Stop"

function ConvertTo-AsciiHex {
    param([string]$Text)

    $bytes = [System.Text.Encoding]::ASCII.GetBytes($Text)
    $parts = foreach ($byte in $bytes) {
        $byte.ToString("X2")
    }
    -join $parts
}

function ConvertFrom-AsciiHex {
    param([string]$Hex)

    if (($Hex.Length % 2) -ne 0) {
        throw "Odd-length bridge hex payload"
    }

    $bytes = New-Object byte[] ($Hex.Length / 2)
    for ($idx = 0; $idx -lt $bytes.Length; $idx++) {
        $bytes[$idx] = [Convert]::ToByte($Hex.Substring($idx * 2, 2), 16)
    }
    [System.Text.Encoding]::ASCII.GetString($bytes)
}

function Write-SerialText {
    param(
        [System.Net.Sockets.NetworkStream]$Stream,
        [string]$Text
    )

    $bytes = [System.Text.Encoding]::ASCII.GetBytes($Text)
    $Stream.Write($bytes, 0, $bytes.Length)
    $Stream.Flush()
}

function Connect-WithRetry {
    param(
        [string]$TargetHost,
        [int]$TargetPort,
        [int]$TimeoutMs
    )

    $deadline = [DateTime]::UtcNow.AddMilliseconds($TimeoutMs)
    do {
        $client = [System.Net.Sockets.TcpClient]::new()
        try {
            $connect = $client.BeginConnect($TargetHost, $TargetPort, $null, $null)
            if ($connect.AsyncWaitHandle.WaitOne(500)) {
                $client.EndConnect($connect)
                return $client
            }
        }
        catch {
            $client.Close()
        }

        $client.Close()
        Start-Sleep -Milliseconds 150
    } while ([DateTime]::UtcNow -lt $deadline)

    throw "Timed out connecting to $TargetHost`:$TargetPort"
}

$client = $null

try {
    $client = Connect-WithRetry -TargetHost $HostName -TargetPort $Port -TimeoutMs $ConnectTimeoutMs
    $stream = $client.GetStream()
    $stream.ReadTimeout = 200

    Write-Host "connected to serial bridge at $HostName`:$Port"

    $line = [System.Text.StringBuilder]::new()
    $buffer = New-Object byte[] 1
    $handled = 0
    $askText = $Ask.Trim()
    $askPending = $askText.Length -gt 0
    $askSendDeadline = [DateTime]::UtcNow.AddMilliseconds($AskStartupDelayMs)
    $requestDeadline = [DateTime]::UtcNow.AddSeconds($RequestTimeoutSeconds)

    while ($true) {
        try {
            $read = $stream.Read($buffer, 0, 1)
        }
        catch [System.IO.IOException] {
            if ($askPending -and [DateTime]::UtcNow -gt $askSendDeadline) {
                $clearLine = -join (1..80 | ForEach-Object { [char]8 })
                Write-SerialText -Stream $stream -Text ($clearLine + ("ask {0}`r" -f $askText))
                $askPending = $false
                $requestDeadline = [DateTime]::UtcNow.AddSeconds($RequestTimeoutSeconds)
            }
            if ($Once -and [DateTime]::UtcNow -gt $requestDeadline) {
                throw "Timed out waiting for one bridge request"
            }
            continue
        }

        if ($read -le 0) {
            break
        }

        $byte = $buffer[0]
        if ($byte -eq 10 -or $byte -eq 13) {
            $text = $line.ToString()
            [void]$line.Clear()

            if ($askPending -and $text -like "*SERIAL CONSOLE READY*") {
                $clearLine = -join (1..80 | ForEach-Object { [char]8 })
                Write-SerialText -Stream $stream -Text ($clearLine + ("ask {0}`r" -f $askText))
                $askPending = $false
                $requestDeadline = [DateTime]::UtcNow.AddSeconds($RequestTimeoutSeconds)
            }

            if ($text -match "SEEDOS_BRIDGE_REQ\s+([0-9]+)\s+([0-9A-Fa-f]+)") {
                $id = $Matches[1]
                $requestText = ConvertFrom-AsciiHex -Hex $Matches[2]
                $responseText = "{0}: {1}" -f $ResponsePrefix, $requestText
                $responseHex = ConvertTo-AsciiHex -Text $responseText
                $frame = [char]2 + "SEEDOS_BRIDGE_RESP $id $responseHex`n"

                Write-Host "request #${id}: $requestText"
                Write-Host "response #${id}: $responseText"
                Write-SerialText -Stream $stream -Text $frame

                $handled++
                if ($Once) {
                    Start-Sleep -Milliseconds 500
                    break
                }
            }
        }
        elseif ($byte -eq 8 -or $byte -eq 127) {
            if ($line.Length -gt 0) {
                $line.Length = $line.Length - 1
            }
        }
        elseif ($byte -ge 32 -and $byte -le 126) {
            [void]$line.Append([char]$byte)
            if ($line.Length -gt 4096) {
                [void]$line.Clear()
            }
        }
    }

    Write-Host "bridge handled $handled request(s)"
}
finally {
    if ($client -ne $null) {
        $client.Close()
    }
}
