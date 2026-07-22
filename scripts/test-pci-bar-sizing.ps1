param(
    [switch]$MetadataOnly
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$TargetRoot = Join-Path $RepoRoot 'target'
$RunRoot = Join-Path $TargetRoot ("pci-bar-sizing-{0}" -f [Guid]::NewGuid().ToString('N'))
$SourceRoot = Join-Path $RunRoot 'src'
$HarnessPath = Join-Path $SourceRoot 'lib.rs'
$TestBinary = Join-Path $RunRoot 'pci-bar-sizing-tests.exe'
$MetadataPath = Join-Path $RunRoot 'pci-bar-sizing-tests.rmeta'
$AssemblyPath = Join-Path $RunRoot 'pci-bar-transport.s'
$MutatedAssemblyPath = Join-Path $RunRoot 'pci-bar-transport-mutated.s'
$MutatedPciPath = Join-Path $RunRoot 'pci-mutated.rs'
$AssociationMutatedAssemblyPath = Join-Path $RunRoot 'pci-bar-transport-association-mutated.s'
$AssociationMutatedPciPath = Join-Path $RunRoot 'pci-association-mutated.rs'
$CaptureMutatedPciPath = Join-Path $RunRoot 'pci-capture-mutated.rs'
$CaptureMutatedTestBinary = Join-Path $RunRoot 'pci-capture-mutated-tests.exe'
$EmptyPath = Join-Path $RunRoot 'empty'
$PredicatePath = Join-Path $RepoRoot 'scripts\test-pci-bar-sizing.ps1'
$PciPath = (Join-Path $RepoRoot 'seed-kernel\src\pci.rs').Replace('\', '/')
$PreviousTemp = $env:TEMP
$PreviousTmp = $env:TMP
$RustSysroot = (& rustup run nightly-2024-10-15 rustc --print sysroot).Trim()
if ($LASTEXITCODE -ne 0) {
    throw 'querying pinned Rust sysroot failed'
}
$RustLldSource = Join-Path $RustSysroot 'lib\rustlib\x86_64-pc-windows-msvc\bin\rust-lld.exe'
$RustLld = Join-Path $RunRoot 'rust-lld.exe'
if (-not $MetadataOnly -and -not (Test-Path -LiteralPath $RustLldSource -PathType Leaf)) {
    throw "pinned Rust linker missing: $RustLldSource"
}

function Write-Harness([string]$Path, [string]$IncludedPciPath) {
    $normalizedPciPath = $IncludedPciPath.Replace('\', '/')
    $harness = @"
#![allow(dead_code)]
extern crate alloc;
extern crate self as spin;

pub struct Mutex<T>(std::sync::Mutex<T>);

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self(std::sync::Mutex::new(value))
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, T> {
        self.0.lock().expect("test mutex poisoned")
    }
}

#[path = "$normalizedPciPath"]
mod pci;
"@
    [IO.File]::WriteAllText($Path, $harness, (New-Object Text.UTF8Encoding($false)))
}

function Compile-TransportAssembly(
    [string]$IncludedPciPath,
    [string]$OutputPath
) {
    Write-Harness $HarnessPath $IncludedPciPath
    & rustup run nightly-2024-10-15 rustc `
        --edition=2021 `
        --crate-type=lib `
        --cfg pci_bar_transport_proof `
        --emit=asm `
        -C opt-level=3 `
        -o $OutputPath `
        $HarnessPath
    if ($LASTEXITCODE -ne 0) {
        throw 'emitting retained production x86 transport assembly failed'
    }
}

function Get-RetainedTransportBody([string]$Path) {
    $assembly = [IO.File]::ReadAllText($Path)
    $marker = 'retained_pci_config_write_u16_transport:'
    $start = $assembly.IndexOf($marker, [StringComparison]::Ordinal)
    if ($start -lt 0) {
        throw 'retained transport symbol missing from emitted assembly'
    }
    $tail = $assembly.Substring($start)
    $nextSymbol = [regex]::Match(
        $tail.Substring($marker.Length),
        '(?m)^\s*(?:\.section|\.globl|\.def)\s+'
    )
    if ($nextSymbol.Success) {
        return $tail.Substring(0, $marker.Length + $nextSymbol.Index)
    }
    return $tail
}

function Get-TransportPortWidthAssociations([string]$Body) {
    $result = @()
    $port = $null
    foreach ($line in [regex]::Split($Body, '\r?\n')) {
        if ($line -match '(?i)(?:\$(332[046])\s*,\s*%e?dx\b|\be?dx\s*,\s*(332[046])\b)') {
            $port = if ($Matches[1]) { $Matches[1] } else { $Matches[2] }
        }
        $width = $null
        if ($line -match '(?i)(?:\bout\s+dx\s*,\s*eax\b|\boutl\s+%eax\s*,\s*%dx\b)') {
            $width = 'dword'
        }
        elseif ($line -match '(?i)(?:\bout\s+dx\s*,\s*ax\b|\boutw\s+%ax\s*,\s*%dx\b)') {
            $width = 'word'
        }
        if ($width) {
            if (-not $port) { throw 'retained transport output has no bound port' }
            $result += ('{0}:{1}' -f $port, $width)
            $port = $null
        }
    }
    return $result
}

function Assert-TransportAssembly([string]$Path) {
    $body = Get-RetainedTransportBody $Path
    $associations = @(Get-TransportPortWidthAssociations $body)
    $expected = @('3320:dword', '3324:word', '3320:dword', '3326:word')
    if (($associations -join ',') -ne ($expected -join ',')) {
        throw ('retained transport port/width associations invalid: {0}' -f ($associations -join ','))
    }
    $dwordOuts = [regex]::Matches(
        $body,
        '(?im)(?:\bout\s+dx\s*,\s*eax\b|\boutl\s+%eax\s*,\s*%dx\b)'
    ).Count
    $wordOuts = [regex]::Matches(
        $body,
        '(?im)(?:\bout\s+dx\s*,\s*ax\b|\boutw\s+%ax\s*,\s*%dx\b)'
    ).Count
    if ($dwordOuts -ne 2 -or $wordOuts -ne 2) {
        throw "retained transport widths invalid: dword=$dwordOuts word=$wordOuts"
    }
    foreach ($port in @('3320', '3324', '3326')) {
        if ($body -notmatch "(?i)(?<![0-9])$port(?![0-9])") {
            throw "retained transport does not materialize required port $port"
        }
    }
}

try {
    New-Item -ItemType Directory -Path $SourceRoot -Force | Out-Null
    $env:TEMP = $RunRoot
    $env:TMP = $RunRoot
    if (-not $MetadataOnly) {
        Copy-Item -LiteralPath $RustLldSource -Destination $RustLld
    }

    Compile-TransportAssembly $PciPath $AssemblyPath
    Assert-TransportAssembly $AssemblyPath
    Write-Host 'PCI-BAR-ASM: retained CF8 dword plus CFC/CFE word transport passed.'

    $pciSource = [IO.File]::ReadAllText($PciPath)
    $dwordWrite = '        outl(port, value);'
    $wordWrite = '        outw(port, value);'
    if ($pciSource.IndexOf($dwordWrite, [StringComparison]::Ordinal) -lt 0 -or
        $pciSource.IndexOf($dwordWrite, [StringComparison]::Ordinal) -ne
        $pciSource.LastIndexOf($dwordWrite, [StringComparison]::Ordinal) -or
        $pciSource.IndexOf($wordWrite, [StringComparison]::Ordinal) -lt 0 -or
        $pciSource.IndexOf($wordWrite, [StringComparison]::Ordinal) -ne
        $pciSource.LastIndexOf($wordWrite, [StringComparison]::Ordinal)) {
        throw 'concrete x86 transport mutation targets are not unique'
    }
    $mutatedSource = $pciSource.Replace($wordWrite, '        outl(port, value as u32);')
    [IO.File]::WriteAllText(
        $MutatedPciPath,
        $mutatedSource,
        (New-Object Text.UTF8Encoding($false))
    )
    Compile-TransportAssembly $MutatedPciPath $MutatedAssemblyPath
    $mutationRejected = $false
    try {
        Assert-TransportAssembly $MutatedAssemblyPath
    }
    catch {
        $mutationRejected = $true
    }
    if (-not $mutationRejected) {
        throw 'dword data-port transport mutation did not make emitted-code proof fail'
    }
    Write-Host 'PCI-BAR-ASM-MUTATION: dword data-port mutation rejected.'

    $associationMutatedSource = $pciSource.Replace(
        $dwordWrite,
        '        outw(port, value as u16);'
    ).Replace(
        $wordWrite,
        '        outl(port, value as u32);'
    )
    [IO.File]::WriteAllText(
        $AssociationMutatedPciPath,
        $associationMutatedSource,
        (New-Object Text.UTF8Encoding($false))
    )
    Compile-TransportAssembly $AssociationMutatedPciPath $AssociationMutatedAssemblyPath
    $associationMutationRejected = $false
    try {
        Assert-TransportAssembly $AssociationMutatedAssemblyPath
    }
    catch {
        $associationMutationRejected = $true
    }
    if (-not $associationMutationRejected) {
        throw 'swapped port/width transport mutation did not make exact emitted-code proof fail'
    }
    Write-Host 'PCI-BAR-ASM-ASSOCIATION-MUTATION: swapped port/width mutation rejected.'

    Write-Harness $HarnessPath $PciPath

    if ($MetadataOnly) {
        & rustup run nightly-2024-10-15 rustc `
            --edition=2021 `
            --test `
            --emit=metadata `
            -o $MetadataPath `
            $HarnessPath
        if ($LASTEXITCODE -ne 0) {
            throw 'type-checking shared PCI BAR production logic failed'
        }
    }
    else {
        & rustup run nightly-2024-10-15 rustc `
            --edition=2021 `
            --test `
            -C opt-level=0 `
            -C "linker=$RustLld" `
            -C linker-flavor=lld-link `
            -o $TestBinary `
            $HarnessPath
        if ($LASTEXITCODE -ne 0) {
            throw 'compiling shared PCI BAR production logic failed'
        }

        & $TestBinary --nocapture
        if ($LASTEXITCODE -ne 0) {
            throw 'PCI BAR sizing production-logic tests failed'
        }

        $captureReject = '            BarDisposition::Unavailable => return Err("pci_capture_bar_unavailable"),'
        if ($pciSource.IndexOf($captureReject, [StringComparison]::Ordinal) -lt 0 -or
            $pciSource.IndexOf($captureReject, [StringComparison]::Ordinal) -ne
            $pciSource.LastIndexOf($captureReject, [StringComparison]::Ordinal)) {
            throw 'capture rejection mutation target is not unique'
        }
        $captureMutatedSource = $pciSource.Replace(
            $captureReject,
            '            BarDisposition::Unavailable => {}'
        )
        [IO.File]::WriteAllText(
            $CaptureMutatedPciPath,
            $captureMutatedSource,
            (New-Object Text.UTF8Encoding($false))
        )
        Write-Harness $HarnessPath $CaptureMutatedPciPath
        $captureRustcArgs = @(
            '--edition=2021',
            '--test',
            '-C', 'opt-level=0',
            '-C', "linker=$RustLld",
            '-C', 'linker-flavor=lld-link',
            '-o', $CaptureMutatedTestBinary,
            $HarnessPath
        )
        & rustup run nightly-2024-10-15 rustc @captureRustcArgs
        if ($LASTEXITCODE -ne 0) {
            throw 'compiling capture rejection mutation failed'
        }

        $previousErrorActionPreference = $ErrorActionPreference
        try {
            $ErrorActionPreference = 'Continue'
            $captureMutationOutput = @(& $CaptureMutatedTestBinary --nocapture 2>&1)
            $captureMutationExit = $LASTEXITCODE
        }
        finally {
            $ErrorActionPreference = $previousErrorActionPreference
        }
        if ($captureMutationExit -eq 0) {
            throw 'capture rejection-to-empty mutation did not make the predicate fail'
        }
        $captureMutationText = $captureMutationOutput -join [Environment]::NewLine
        if ($captureMutationText -notmatch
            'capture_enumeration_distinguishes_empty_headers_from_probe_rejection') {
            throw 'capture mutation failed for an unexpected reason'
        }
        Write-Host 'PCI-BAR-CAPTURE-MUTATION: rejection-to-empty mutation rejected.'
    }

    & git -C $RepoRoot diff --check -- seed-kernel/src/pci.rs scripts/test-pci-bar-sizing.ps1
    if ($LASTEXITCODE -ne 0) {
        throw 'git diff --check failed for lane files'
    }
    $predicateSource = [IO.File]::ReadAllText($PredicatePath)
    if ($predicateSource -notmatch 'Assert-TransportAssembly' -or
        $predicateSource -notmatch 'PCI-BAR-CAPTURE-MUTATION' -or
        $predicateSource -notmatch 'diff --no-index --check') {
        throw 'predicate self-content checks are missing'
    }
    [IO.File]::WriteAllText($EmptyPath, '', (New-Object Text.UTF8Encoding($false)))
    $selfCheck = @(
        & git -C $RepoRoot -c core.autocrlf=false diff --no-index --check -- `
            $EmptyPath $PredicatePath 2>&1
    )
    $selfCheckExit = $LASTEXITCODE
    if ($selfCheckExit -notin @(0, 1) -or $selfCheck.Count -ne 0) {
        throw "git whitespace check failed for untracked predicate: $($selfCheck -join '; ')"
    }
}
finally {
    $env:TEMP = $PreviousTemp
    $env:TMP = $PreviousTmp
    if (Test-Path -LiteralPath $RunRoot) {
        Remove-Item -LiteralPath $RunRoot -Recurse -Force
    }
}

if (Test-Path -LiteralPath $RunRoot) {
    throw "transient test directory remains: $RunRoot"
}

if ($MetadataOnly) {
    Write-Host 'PCI BAR sizing production-logic metadata check passed; runtime predicate not executed.'
}
else {
    Write-Host 'PCI BAR sizing production-logic predicate passed.'
}
