param(
    [string]$Image = "$PSScriptRoot\..\release\seedos-stage0.img",
    [string]$SerialLog = "$env:TEMP\seedos-stage0.serial.txt",
    [ValidateSet("file", "tcp")]
    [string]$SerialMode = "file",
    [int]$SerialTcpPort = 4555,
    [int]$MonitorTcpPort = 0,
    [string]$Cpu = "",
    [ValidateSet("", "none", "e1000")]
    [string]$Nic = "",
    [switch]$BareMetalVm,
    [switch]$Headless,
    [switch]$UsbXhciInput,
    [switch]$StopExisting
)

$ErrorActionPreference = "Stop"

if ($BareMetalVm) {
    $UsbXhciInput = $true
    if (-not $Nic) {
        $Nic = "e1000"
    }
    if (-not $Cpu) {
        $Cpu = "max"
    }
}

if (-not $Nic) {
    $Nic = "e1000"
}

if ($StopExisting) {
    Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force
}

$RepoRoot = Split-Path -Parent $PSScriptRoot
$Qemu = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$CodeSrc = "C:\Program Files\qemu\share\edk2-x86_64-code.fd"
$Code = Join-Path $env:TEMP "seedos-edk2-code-run.fd"
$Vars = Join-Path $env:TEMP "seedos-ovmf-vars-run.fd"
$ErrLog = [System.IO.Path]::ChangeExtension($SerialLog, ".err.txt")

Copy-Item -LiteralPath $CodeSrc -Destination $Code -Force
Copy-Item -LiteralPath (Join-Path $RepoRoot "release\ovmf_vars.fd") -Destination $Vars -Force
Remove-Item -LiteralPath $SerialLog, $ErrLog -Force -ErrorAction SilentlyContinue

$qemuArgs = @(
    "-machine", "q35",
    "-m", "512M",
    "-drive", "if=pflash,format=raw,readonly=on,file=$Code",
    "-drive", "if=pflash,format=raw,file=$Vars",
    "-drive", "file=$((Resolve-Path $Image).Path),format=raw,if=ide"
)

if ($Cpu) {
    $qemuArgs += @("-cpu", $Cpu)
}

if ($Nic -eq "e1000") {
    $qemuArgs += @(
        "-netdev", "user,id=net0",
        "-device", "e1000,netdev=net0,mac=52:54:00:12:34:56"
    )
}

if ($UsbXhciInput) {
    $qemuArgs += @(
        "-device", "qemu-xhci,id=xhci",
        "-device", "usb-kbd,bus=xhci.0",
        "-device", "usb-mouse,bus=xhci.0"
    )
}

if ($SerialMode -eq "tcp") {
    $qemuArgs += @(
        "-chardev", "socket,id=seedserial,host=127.0.0.1,port=$SerialTcpPort,server=on,wait=off,logfile=$SerialLog,logappend=off",
        "-serial", "chardev:seedserial"
    )
}
else {
    $qemuArgs += @("-serial", "file:$SerialLog")
}

if ($Headless) {
    $qemuArgs += @("-display", "none")
}
else {
    $qemuArgs += @("-display", "gtk")
}

if ($MonitorTcpPort -gt 0) {
    $qemuArgs += @("-monitor", "tcp:127.0.0.1:$MonitorTcpPort,server,nowait")
}

$qemuArgs += @("-no-reboot")

$process = Start-Process -FilePath $Qemu -ArgumentList $qemuArgs -PassThru -RedirectStandardError $ErrLog
Write-Output "qemu pid: $($process.Id)"
Write-Output "serial log: $SerialLog"
if ($SerialMode -eq "tcp") {
    Write-Output "serial tcp: 127.0.0.1:$SerialTcpPort"
}
if ($MonitorTcpPort -gt 0) {
    Write-Output "monitor tcp: 127.0.0.1:$MonitorTcpPort"
}
