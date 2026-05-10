param(
    [ValidateSet("debug", "release")]
    [string]$Profile = "release",
    [string]$Image = "$PSScriptRoot\..\release\seedos-stage0.img"
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$EspDir = Join-Path $RepoRoot "release\esp"
$KernelProfileDir = if ($Profile -eq "release") { "release" } else { "debug" }
$Kernel = Join-Path $RepoRoot "target\x86_64-seed\$KernelProfileDir\seed-kernel"
$LimineConfig = Join-Path $RepoRoot "seed-kernel\limine\limine.conf"
$BootConfig = Join-Path $EspDir "EFI\BOOT\limine.conf"
$ImageTool = Join-Path $RepoRoot "scripts\make-fat32-image.py"

powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $RepoRoot "scripts\build-seed-kernel.ps1") -Profile $Profile

New-Item -ItemType Directory -Force -Path (Join-Path $EspDir "EFI\BOOT") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $EspDir "kernel") | Out-Null

Copy-Item -LiteralPath $Kernel -Destination (Join-Path $EspDir "kernel\kernel.elf") -Force
Copy-Item -LiteralPath $Kernel -Destination (Join-Path $EspDir "kernel\seed-kernel.elf") -Force
Copy-Item -LiteralPath $LimineConfig -Destination (Join-Path $EspDir "limine.conf") -Force
Copy-Item -LiteralPath $LimineConfig -Destination $BootConfig -Force

if (-not (Test-Path (Join-Path $EspDir "EFI\BOOT\BOOTX64.EFI"))) {
    throw "Missing Limine bootloader at release\esp\EFI\BOOT\BOOTX64.EFI"
}

python $ImageTool --root $EspDir --output $Image --size 67108864
