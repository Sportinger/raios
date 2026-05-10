param(
    [string]$Image = "$PSScriptRoot\..\release\seedos-stage0.img",
    [string]$SerialLog = "$env:TEMP\seedos-stage0-baremetal.serial.txt",
    [ValidateSet("file", "tcp")]
    [string]$SerialMode = "tcp",
    [int]$SerialTcpPort = 4555,
    [int]$MonitorTcpPort = 0,
    [switch]$Headless,
    [switch]$StopExisting
)

$ErrorActionPreference = "Stop"

$args = @(
    "-ExecutionPolicy", "Bypass",
    "-File", (Join-Path $PSScriptRoot "run-stage0-qemu.ps1"),
    "-Image", $Image,
    "-SerialLog", $SerialLog,
    "-SerialMode", $SerialMode,
    "-SerialTcpPort", $SerialTcpPort,
    "-BareMetalVm"
)

if ($MonitorTcpPort -gt 0) {
    $args += @("-MonitorTcpPort", $MonitorTcpPort)
}
if ($Headless) {
    $args += "-Headless"
}
if ($StopExisting) {
    $args += "-StopExisting"
}

powershell -NoProfile @args
