param(
    [string]$Image = "$PSScriptRoot\..\release\raios-stage0.img",
    [string]$ScratchImage = "",
    [string]$AuditRollbackTargetImage = "",
    [string]$PersistDiskPath = "",
    [string]$StructuredStoreDiskPath = "",
    [string]$UsbStorageImage = "",
    [ValidateSet("writethrough", "directsync", "writeback", "none", "unsafe")]
    [string]$PersistCacheMode = "writethrough",
    [string]$SerialLog = "$env:TEMP\raios-stage0.serial.txt",
    [ValidateSet("file", "tcp")]
    [string]$SerialMode = "file",
    [int]$SerialTcpPort = 4555,
    [int]$MonitorTcpPort = 0,
    [int]$QmpTcpPort = 0,
    [string]$Cpu = "",
    [ValidateSet("", "none", "e1000")]
    [string]$Nic = "",
    # NET-8 live W7 fetch: bridge a guest TCP address to a host loopback fixture, e.g.
    # "tcp:10.0.2.100:8443-tcp:127.0.0.1:8443". Must NOT target the slirp gateway 10.0.2.2
    # (QEMU rejects guestfwd to the gateway). Empty = no bridge.
    [string]$GuestFwd = "",
    [switch]$BareMetalVm,
    [switch]$Headless,
    [switch]$MouseGrab,
    [switch]$RelativeMouse,
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
$Code = Join-Path $env:TEMP "raios-edk2-code-run.fd"
$Vars = Join-Path $env:TEMP "raios-ovmf-vars-run.fd"
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

if ($StructuredStoreDiskPath) {
    # C1 only accepts a disposable regular fixture. It never selects a host
    # disk and deliberately has no relationship to the SEED_DATA path.
    if ($StructuredStoreDiskPath -match '^(\\\\[.?]\\|\\Device\\)') {
        throw "StructuredStoreDiskPath must name a disposable regular image file, not a device"
    }
    $structuredStoreItem = Get-Item -LiteralPath $StructuredStoreDiskPath -Force
    if ($structuredStoreItem -isnot [System.IO.FileInfo] -or
        $structuredStoreItem.Extension -notin ".img", ".raw" -or
        $structuredStoreItem.Name -notmatch "structured-store") {
        throw "StructuredStoreDiskPath must be a regular .img/.raw file named structured-store*"
    }
    $StructuredStoreDiskPath = $structuredStoreItem.FullName
}

if ($ScratchImage) {
    $qemuArgs += @(
        "-drive", "file=$((Resolve-Path $ScratchImage).Path),format=raw,if=none,id=raiosscratch0",
        "-device", "ide-hd,drive=raiosscratch0,bus=ide.1,unit=0"
    )
}

if ($AuditRollbackTargetImage) {
    $qemuArgs += @(
        "-drive", "file=$((Resolve-Path $AuditRollbackTargetImage).Path),format=raw,if=none,id=raiosauditrollback0",
        "-device", "ide-hd,drive=raiosauditrollback0,bus=ide.2,unit=0"
    )
}

if ($PersistDiskPath) {
    $qemuArgs += @(
        "-drive", "file=$((Resolve-Path $PersistDiskPath).Path),format=raw,if=none,id=raiospersist0,cache=$PersistCacheMode",
        "-device", "ide-hd,drive=raiospersist0,bus=ide.3,unit=0"
    )
}

if ($StructuredStoreDiskPath) {
    $qemuArgs += @(
        "-drive", "file=$StructuredStoreDiskPath,format=raw,if=none,id=raiosstructuredstore0,cache=writethrough",
        "-device", "ide-hd,drive=raiosstructuredstore0,bus=ide.4,unit=0"
    )
}

if ($Cpu) {
    $qemuArgs += @("-cpu", $Cpu)
}

if ($Nic -eq "e1000") {
    $netdev = "user,id=net0"
    if ($GuestFwd) {
        $netdev += ",guestfwd=$GuestFwd"
    }
    $qemuArgs += @(
        "-netdev", $netdev,
        "-device", "e1000,netdev=net0,mac=52:54:00:12:34:56"
    )
}

if ($UsbXhciInput) {
    $qemuArgs += @(
        "-device", "qemu-xhci,id=xhci",
        "-device", "usb-kbd,bus=xhci.0,id=bootkbd"
    )
    if ($RelativeMouse) {
        $qemuArgs += @("-device", "usb-mouse,bus=xhci.0")
    }
    else {
        $qemuArgs += @("-device", "usb-tablet,bus=xhci.0")
    }
}
elseif ($UsbStorageImage) {
    $qemuArgs += @("-device", "qemu-xhci,id=xhci")
}

if ($UsbStorageImage) {
    $qemuArgs += @(
        "-drive", "file=$((Resolve-Path $UsbStorageImage).Path),format=raw,if=none,id=raiosusb0",
        "-device", "usb-storage,bus=xhci.0,drive=raiosusb0"
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
elseif ($MouseGrab) {
    $qemuArgs += @("-display", "gtk,grab-on-hover=on,show-cursor=off")
}
else {
    $qemuArgs += @("-display", "gtk,show-cursor=off")
}

if ($MonitorTcpPort -gt 0) {
    $qemuArgs += @("-monitor", "tcp:127.0.0.1:$MonitorTcpPort,server,nowait")
}
if ($QmpTcpPort -gt 0) {
    $qemuArgs += @("-qmp", "tcp:127.0.0.1:$QmpTcpPort,server=on,wait=off")
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
if ($QmpTcpPort -gt 0) {
    Write-Output "qmp tcp: 127.0.0.1:$QmpTcpPort"
}
