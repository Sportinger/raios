param(
    [string]$Image = "$PSScriptRoot\..\release\seedos-stage0.img",
    [string]$SerialLog = "$env:TEMP\seedos-stage0.serial.txt",
    [switch]$StopExisting
)

$ErrorActionPreference = "Stop"

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
    "-drive", "file=$((Resolve-Path $Image).Path),format=raw,if=ide",
    "-netdev", "user,id=net0",
    "-device", "virtio-net-pci,netdev=net0",
    "-device", "virtio-rng-pci",
    "-device", "virtio-keyboard-pci",
    "-device", "virtio-mouse-pci",
    "-serial", "file:$SerialLog",
    "-display", "gtk",
    "-no-reboot"
)

$process = Start-Process -FilePath $Qemu -ArgumentList $qemuArgs -PassThru -RedirectStandardError $ErrLog
Write-Output "qemu pid: $($process.Id)"
Write-Output "serial log: $SerialLog"
