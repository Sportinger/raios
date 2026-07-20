[CmdletBinding()]
param(
    [string]$InputDirectory,
    [string]$OutputDirectory,
    [string]$ProjectId,
    [string]$ProjectRevisionSha256,
    [string]$ExpectedSourceTreeSha256,
    [string]$ExpectedCargoLockSha256,
    [string]$ExpectedPackageName,
    [string]$ExpectedPackageVersion,
    [string]$ExpectedMaterializedTreeSha256,
    [string]$VendorDirectory,
    [string]$ExpectedDependencyName,
    [string]$ExpectedDependencyVersion,
    [string]$ExpectedDependencyBundleSha256,
    [ValidateRange(30, 1800)]
    [int]$TimeoutSeconds = 300,
    [switch]$SelfTest
)

$ErrorActionPreference = 'Stop'
$script:ResultPath = $null
$script:CandidatePath = $null
$script:FailureContext = @{}

function Get-Sha256Hex {
    param([Parameter(Mandatory)][AllowEmptyCollection()][byte[]]$Bytes)
    $sha = [Security.Cryptography.SHA256]::Create()
    try { return ([BitConverter]::ToString($sha.ComputeHash($Bytes)) -replace '-', '').ToLowerInvariant() }
    finally { $sha.Dispose() }
}

function Get-FileSha256Hex {
    param([Parameter(Mandatory)][string]$Path)
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Test-Hex {
    param([string]$Value, [int]$Length)
    return $Value -and $Value.Length -eq $Length -and $Value -cmatch '^[0-9a-f]+$'
}

function Write-CanonicalString {
    param([IO.BinaryWriter]$Writer, [string]$Value)
    $bytes = [Text.Encoding]::UTF8.GetBytes($Value)
    if ($bytes.Length -gt [uint16]::MaxValue) { throw 'canonical string exceeds u16 length' }
    $Writer.Write([uint16]$bytes.Length)
    $Writer.Write($bytes)
}

function Convert-HexToBytes {
    param([string]$Hex)
    $bytes = [byte[]]::new($Hex.Length / 2)
    for ($index = 0; $index -lt $bytes.Length; $index++) {
        $bytes[$index] = [Convert]::ToByte($Hex.Substring($index * 2, 2), 16)
    }
    return $bytes
}

function ConvertTo-NativeArgument {
    param([string]$Value)
    if ($null -eq $Value -or $Value.Length -eq 0) { return '""' }
    if ($Value -notmatch '[\s"]') { return $Value }
    $builder = [Text.StringBuilder]::new()
    [void]$builder.Append('"')
    $slashes = 0
    foreach ($character in $Value.ToCharArray()) {
        if ($character -eq '\') {
            $slashes++
            continue
        }
        if ($character -eq '"') {
            [void]$builder.Append(('\' * (($slashes * 2) + 1)))
            [void]$builder.Append('"')
            $slashes = 0
            continue
        }
        if ($slashes) { [void]$builder.Append(('\' * $slashes)); $slashes = 0 }
        [void]$builder.Append($character)
    }
    if ($slashes) { [void]$builder.Append(('\' * ($slashes * 2))) }
    [void]$builder.Append('"')
    return $builder.ToString()
}

function Invoke-IsolatedTool {
    param(
        [string]$FilePath,
        [string[]]$Arguments,
        [string]$WorkingDirectory,
        [System.Collections.IDictionary]$Environment,
        [string]$LogDirectory,
        [string]$Label,
        [int]$Timeout
    )
    $stdoutPath = Join-Path $LogDirectory "$Label.stdout"
    $stderrPath = Join-Path $LogDirectory "$Label.stderr"
    $start = [Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $FilePath
    $start.Arguments = (($Arguments | ForEach-Object { ConvertTo-NativeArgument $_ }) -join ' ')
    $start.WorkingDirectory = $WorkingDirectory
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.EnvironmentVariables.Clear()
    foreach ($key in ($Environment.Keys | Sort-Object)) {
        $start.EnvironmentVariables[$key] = [string]$Environment[$key]
    }

    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $start
    $stdout = [IO.MemoryStream]::new()
    $stderr = [IO.MemoryStream]::new()
    try {
        if (-not $process.Start()) { throw "failed to start $Label" }
        $stdoutCopy = $process.StandardOutput.BaseStream.CopyToAsync($stdout)
        $stderrCopy = $process.StandardError.BaseStream.CopyToAsync($stderr)
        if (-not $process.WaitForExit($Timeout * 1000)) {
            try { $process.Kill() } catch {}
            throw "$Label timed out after $Timeout seconds"
        }
        [Threading.Tasks.Task]::WaitAll(@($stdoutCopy, $stderrCopy))
        $process.WaitForExit()
        $stdoutBytes = $stdout.ToArray()
        $stderrBytes = $stderr.ToArray()
        [IO.File]::WriteAllBytes($stdoutPath, $stdoutBytes)
        [IO.File]::WriteAllBytes($stderrPath, $stderrBytes)
        return [pscustomobject][ordered]@{
            label = $Label
            exit_code = $process.ExitCode
            stdout_path = [IO.Path]::GetFileName($stdoutPath)
            stdout_byte_len = $stdoutBytes.Length
            stdout_sha256 = "sha256:$(Get-Sha256Hex $stdoutBytes)"
            stderr_path = [IO.Path]::GetFileName($stderrPath)
            stderr_byte_len = $stderrBytes.Length
            stderr_sha256 = "sha256:$(Get-Sha256Hex $stderrBytes)"
            stdout_bytes = $stdoutBytes
            stderr_bytes = $stderrBytes
        }
    }
    finally {
        $stdout.Dispose()
        $stderr.Dispose()
        $process.Dispose()
    }
}

function Get-ValidatedInputTree {
    param([string]$Root, [string]$VendorRoot)
    $rootPath = [IO.Path]::GetFullPath($Root).TrimEnd('\')
    $rootItem = Get-Item -LiteralPath $rootPath -Force
    if (-not $rootItem.PSIsContainer) { throw 'input path is not a directory' }
    if (($rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'input root is a reparse point' }

    $caseKeys = @{}
    $files = @()
    foreach ($item in Get-ChildItem -LiteralPath $rootPath -Force -Recurse) {
        if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "reparse point denied: $($item.FullName)"
        }
        $full = [IO.Path]::GetFullPath($item.FullName)
        if (-not $full.StartsWith($rootPath + '\', [StringComparison]::OrdinalIgnoreCase)) {
            throw "input path escaped root: $full"
        }
        $relative = $full.Substring($rootPath.Length + 1).Replace('\', '/')
        if ([IO.Path]::IsPathRooted($relative) -or $relative.Split('/') -contains '..') {
            throw "invalid relative input path: $relative"
        }
        $caseKey = $relative.ToLowerInvariant()
        if ($caseKeys.ContainsKey($caseKey)) { throw "case-colliding input path: $relative" }
        $caseKeys[$caseKey] = $true
        if ($item.PSIsContainer) { continue }
        if ([IO.Path]::GetFileName($relative).Equals('build.rs', [StringComparison]::OrdinalIgnoreCase)) {
            throw "build script denied: $relative"
        }
        if ($relative -match '(^|/)\.cargo/config(\.toml)?$') {
            throw "Cargo configuration denied inside materialized input: $relative"
        }
        $files += [pscustomobject][ordered]@{
            path = $relative
            byte_len = [uint64]$item.Length
            sha256 = Get-FileSha256Hex $full
            full_path = $full
        }
    }
    $files = @($files | Sort-Object -Property @{ Expression = { $_.path }; Ascending = $true })
    if (-not $files.Count) { throw 'input tree is empty' }

    $cargoTomls = @($files | Where-Object { [IO.Path]::GetFileName($_.path).Equals('Cargo.toml', [StringComparison]::OrdinalIgnoreCase) })
    $cargoLocks = @($files | Where-Object { [IO.Path]::GetFileName($_.path).Equals('Cargo.lock', [StringComparison]::OrdinalIgnoreCase) })
    $expectedTomlCount = if ($VendorRoot) { 2 } else { 1 }
    if ($cargoTomls.Count -ne $expectedTomlCount -or @($cargoTomls | Where-Object { $_.path -ceq 'Cargo.toml' }).Count -ne 1) {
        throw "expected root Cargo.toml plus at most one vendor Cargo.toml"
    }
    if ($VendorRoot) {
        $vendorManifests = @($cargoTomls | Where-Object { $_.path -cne 'Cargo.toml' })
        if ($vendorManifests.Count -ne 1 -or -not $vendorManifests[0].full_path.StartsWith($VendorRoot + '\', [StringComparison]::OrdinalIgnoreCase)) {
            throw 'vendor Cargo.toml escaped the provided vendor tree'
        }
    }
    if ($cargoLocks.Count -ne 1 -or @($cargoLocks | Where-Object { $_.path -ceq 'Cargo.lock' }).Count -ne 1) {
        throw 'exactly one root Cargo.lock is required'
    }

    $stream = [IO.MemoryStream]::new()
    $writer = [IO.BinaryWriter]::new($stream, [Text.Encoding]::UTF8, $true)
    try {
        $writer.Write([Text.Encoding]::ASCII.GetBytes('raios.host_builder_input_tree.v1'))
        $writer.Write([uint32]$files.Count)
        foreach ($file in $files) {
            Write-CanonicalString $writer $file.path
            $writer.Write([uint64]$file.byte_len)
            $writer.Write([byte[]](Convert-HexToBytes $file.sha256))
        }
        $writer.Flush()
        $treeSha256 = Get-Sha256Hex $stream.ToArray()
    }
    finally { $writer.Dispose(); $stream.Dispose() }

    return [pscustomobject]@{ root = $rootPath; files = $files; tree_sha256 = $treeSha256 }
}

function Get-ToolchainManifest {
    param([string]$ToolchainRoot, [System.Collections.IDictionary]$Environment, [string]$LogDirectory, [int]$Timeout)
    $cargo = Join-Path $ToolchainRoot 'bin\cargo.exe'
    $rustc = Join-Path $ToolchainRoot 'bin\rustc.exe'
    $lld = Join-Path $ToolchainRoot 'lib\rustlib\x86_64-pc-windows-msvc\bin\rust-lld.exe'
    foreach ($path in @($cargo, $rustc, $lld)) { if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "missing toolchain file: $path" } }

    $rustcVersionRun = Invoke-IsolatedTool $rustc @('-vV') $ToolchainRoot $Environment $LogDirectory 'toolchain-rustc-version' $Timeout
    $cargoVersionRun = Invoke-IsolatedTool $cargo @('-vV') $ToolchainRoot $Environment $LogDirectory 'toolchain-cargo-version' $Timeout
    if ($rustcVersionRun.exit_code -ne 0 -or $cargoVersionRun.exit_code -ne 0) { throw 'toolchain version query failed' }
    $rustcVersion = [Text.Encoding]::UTF8.GetString($rustcVersionRun.stdout_bytes).Trim()
    $cargoVersion = [Text.Encoding]::UTF8.GetString($cargoVersionRun.stdout_bytes).Trim()
    if ($rustcVersion -notmatch 'release: 1\.84\.0-nightly' -or $rustcVersion -notmatch 'commit-hash: 9322d183f45e0fd5a509820874cc5ff27744a479') {
        throw 'pinned nightly rustc identity mismatch'
    }

    $targetLib = Join-Path $ToolchainRoot 'lib\rustlib\wasm32-unknown-unknown\lib'
    if (-not (Test-Path -LiteralPath $targetLib -PathType Container)) { throw 'wasm32-unknown-unknown target is not installed' }
    $targetLibItemsByName = @{}
    foreach ($item in @(Get-ChildItem -LiteralPath $targetLib | Where-Object { -not $_.PSIsContainer })) {
        $targetLibItemsByName[$item.Name] = $item
    }
    $targetLibNames = [string[]]@($targetLibItemsByName.Keys)
    [Array]::Sort($targetLibNames, [StringComparer]::Ordinal)
    $targetLibSetPreimage = New-Object Text.StringBuilder
    $targetLibSet = @($targetLibNames | ForEach-Object {
        $item = $targetLibItemsByName[$_]
        $sha256 = Get-FileSha256Hex $item.FullName
        [void]$targetLibSetPreimage.Append("$($_)=$sha256`n")
        [pscustomobject][ordered]@{
            name = $_
            sha256 = "sha256:$sha256"
        }
    })
    $targetLibSetSha256 = Get-Sha256Hex ([Text.Encoding]::UTF8.GetBytes($targetLibSetPreimage.ToString()))
    if ($targetLibSetSha256 -cne '0ddff5952e3b6bbaeca08f78ea9e6841b833aef46fda393e7b4639e029267fde') {
        throw 'pinned wasm target library set mismatch'
    }
    $paths = @($cargo, $rustc, $lld)
    foreach ($filter in @('rustc_driver-*.dll', 'std-*.dll')) {
        $matches = @(Get-ChildItem (Join-Path $ToolchainRoot 'bin') -Filter $filter -File)
        if ($matches.Count -ne 1) { throw "expected one $filter toolchain file" }
        $paths += $matches[0].FullName
    }
    $paths += Join-Path $ToolchainRoot 'lib\rustlib\manifest-rustc-x86_64-pc-windows-msvc'
    $paths += Join-Path $ToolchainRoot 'lib\rustlib\manifest-rust-std-wasm32-unknown-unknown'
    foreach ($filter in @('libcore-*.rlib', 'libcompiler_builtins-*.rlib', 'librustc_std_workspace_core-*.rlib', 'libpanic_abort-*.rlib')) {
        $matches = @(Get-ChildItem $targetLib -Filter $filter -File)
        if ($matches.Count -ne 1) { throw "expected one $filter target library" }
        $paths += $matches[0].FullName
    }
    $files = @($paths | ForEach-Object {
        $item = Get-Item -LiteralPath $_
        [pscustomobject][ordered]@{
            path = $item.FullName.Substring($ToolchainRoot.Length + 1).Replace('\', '/')
            byte_len = [uint64]$item.Length
            sha256 = "sha256:$(Get-FileSha256Hex $item.FullName)"
        }
    } | Sort-Object path)
    $manifest = [pscustomobject][ordered]@{
        toolchain_id = 'nightly-2024-10-15-x86_64-pc-windows-msvc'
        target = 'wasm32-unknown-unknown'
        rustc_version = $rustcVersion
        cargo_version = $cargoVersion
        target_lib_file_count = [uint32]$targetLibSet.Count
        target_lib_set_sha256 = "sha256:$targetLibSetSha256"
        target_lib_set = $targetLibSet
        files = $files
    }
    $bytes = [Text.Encoding]::UTF8.GetBytes(($manifest | ConvertTo-Json -Compress -Depth 8))
    return [pscustomobject]@{
        manifest = $manifest
        manifest_sha256 = Get-Sha256Hex $bytes
        cargo = $cargo
        rustc = $rustc
        rust_lld = $lld
        version_runs = @($rustcVersionRun, $cargoVersionRun)
        target_lib_set_sha256 = $targetLibSetSha256
    }
}

function Get-RunFact {
    param([object]$Run)
    return [pscustomobject][ordered]@{
        exit_code = $Run.exit_code
        stdout_path = $Run.stdout_path
        stdout_byte_len = $Run.stdout_byte_len
        stdout_sha256 = $Run.stdout_sha256
        stderr_path = $Run.stderr_path
        stderr_byte_len = $Run.stderr_byte_len
        stderr_sha256 = $Run.stderr_sha256
    }
}

function Write-ResultJson {
    param([object]$Value, [string]$Path)
    $json = $Value | ConvertTo-Json -Depth 16
    [IO.File]::WriteAllText($Path, $json + [Environment]::NewLine, [Text.UTF8Encoding]::new($false))
}

function Invoke-SelfTest {
    $root = Join-Path ([IO.Path]::GetTempPath()) ("raios-w4-selftest-" + [Guid]::NewGuid().ToString('N'))
    $inputPath = Join-Path $root 'input'
    $outputPath = Join-Path $root 'output'
    try {
        [IO.Directory]::CreateDirectory((Join-Path $inputPath 'src')) | Out-Null
        [IO.File]::WriteAllText((Join-Path $inputPath 'Cargo.toml'), "[package]`nname = `"raios-w4-selftest`"`nversion = `"0.1.0`"`nedition = `"2021`"`n`n[lib]`ncrate-type = [`"cdylib`"]`n`n[profile.release]`npanic = `"abort`"`n", [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText((Join-Path $inputPath 'Cargo.lock'), "# This file is automatically @generated by Cargo.`nversion = 4`n`n[[package]]`nname = `"raios-w4-selftest`"`nversion = `"0.1.0`"`n", [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText((Join-Path $inputPath 'src\lib.rs'), "#![no_std]`nuse core::panic::PanicInfo;`n#[no_mangle] pub extern `"C`" fn raios_service_main() -> i32 { 42 }`n#[panic_handler] fn panic(_: &PanicInfo) -> ! { core::arch::wasm32::unreachable() }`n", [Text.UTF8Encoding]::new($false))
        & $PSCommandPath -InputDirectory $inputPath -OutputDirectory $outputPath `
            -ProjectId ('a' * 32) -ProjectRevisionSha256 ('b' * 64) `
            -ExpectedSourceTreeSha256 ('c' * 64) `
            -ExpectedCargoLockSha256 (Get-FileSha256Hex (Join-Path $inputPath 'Cargo.lock')) `
            -ExpectedPackageName 'raios-w4-selftest' -ExpectedPackageVersion '0.1.0' `
            -TimeoutSeconds $TimeoutSeconds
        $result = Get-Content (Join-Path $outputPath 'build-result.json') -Raw | ConvertFrom-Json
        if ($result.status -ne 'accepted' -or -not $result.reproducible -or -not (Test-Path (Join-Path $outputPath 'candidate.wasm'))) {
            throw 'W4 self-test result mismatch'
        }
        Write-Output 'build-project-wasm self-test passed'
    }
    finally { if (Test-Path $root) { Remove-Item -LiteralPath $root -Recurse -Force } }
}

if ($SelfTest) {
    Invoke-SelfTest
    return
}

try {
    foreach ($parameter in @('InputDirectory', 'OutputDirectory', 'ProjectId', 'ProjectRevisionSha256', 'ExpectedSourceTreeSha256', 'ExpectedCargoLockSha256', 'ExpectedPackageName', 'ExpectedPackageVersion')) {
        if (-not (Get-Variable -Name $parameter -ValueOnly)) { throw "missing required parameter: $parameter" }
    }
    if (-not (Test-Hex $ProjectId 32)) { throw 'ProjectId must be 32 lowercase hex characters' }
    foreach ($hash in @($ProjectRevisionSha256, $ExpectedSourceTreeSha256, $ExpectedCargoLockSha256)) {
        if (-not (Test-Hex $hash 64)) { throw 'expected hashes must be 64 lowercase hex characters' }
    }
    if ($ExpectedMaterializedTreeSha256 -and -not (Test-Hex $ExpectedMaterializedTreeSha256 64)) { throw 'ExpectedMaterializedTreeSha256 is invalid' }
    $dependencyParameters = @($VendorDirectory, $ExpectedDependencyName, $ExpectedDependencyVersion, $ExpectedDependencyBundleSha256)
    $dependencyEnabled = [bool]$VendorDirectory
    if (($dependencyParameters | Where-Object { [bool]$_ }).Count -notin @(0, 4)) { throw 'vendor dependency parameters must be supplied together' }
    if ($dependencyEnabled -and -not (Test-Hex $ExpectedDependencyBundleSha256 64)) { throw 'ExpectedDependencyBundleSha256 is invalid' }

    $inputRoot = (Resolve-Path -LiteralPath $InputDirectory).Path.TrimEnd('\')
    $outputRoot = [IO.Path]::GetFullPath($OutputDirectory).TrimEnd('\')
    if ($outputRoot.StartsWith($inputRoot + '\', [StringComparison]::OrdinalIgnoreCase) -or $inputRoot.StartsWith($outputRoot + '\', [StringComparison]::OrdinalIgnoreCase)) {
        throw 'input and output directories must not contain one another'
    }
    if (Test-Path $outputRoot) {
        if (-not (Get-Item $outputRoot).PSIsContainer -or @(Get-ChildItem $outputRoot -Force).Count) { throw 'output directory must be absent or empty' }
    } else { [IO.Directory]::CreateDirectory($outputRoot) | Out-Null }
    $script:ResultPath = Join-Path $outputRoot 'build-result.json'
    $script:CandidatePath = Join-Path $outputRoot 'candidate.wasm'
    $logs = Join-Path $outputRoot 'logs'
    $temp = Join-Path $outputRoot 'tmp'
    [IO.Directory]::CreateDirectory($logs) | Out-Null
    [IO.Directory]::CreateDirectory($temp) | Out-Null

    $vendorRoot = $null
    if ($dependencyEnabled) {
        $vendorRoot = (Resolve-Path -LiteralPath $VendorDirectory).Path.TrimEnd('\')
        if (-not $vendorRoot.StartsWith($inputRoot + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'vendor directory must be under input root' }
    }
    $tree = Get-ValidatedInputTree $inputRoot $vendorRoot
    if ($ExpectedMaterializedTreeSha256 -and $tree.tree_sha256 -cne $ExpectedMaterializedTreeSha256) { throw 'materialized tree hash mismatch' }
    $cargoLockPath = Join-Path $inputRoot 'Cargo.lock'
    $cargoLockSha256 = Get-FileSha256Hex $cargoLockPath
    if ($cargoLockSha256 -cne $ExpectedCargoLockSha256) { throw 'Cargo.lock hash mismatch' }

    $repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
    $cargoHome = (Resolve-Path (Join-Path $repoRoot '.cargo-home')).Path
    foreach ($configPath in @(
        (Join-Path $cargoHome 'config'), (Join-Path $cargoHome 'config.toml')
    )) {
        if (Test-Path -LiteralPath $configPath) { throw "Cargo home configuration denied: $configPath" }
    }
    $ancestor = Get-Item -LiteralPath $inputRoot
    while ($ancestor) {
        foreach ($configName in @('.cargo\config', '.cargo\config.toml')) {
            $configPath = Join-Path $ancestor.FullName $configName
            if (Test-Path -LiteralPath $configPath) { throw "ancestor Cargo configuration denied: $configPath" }
        }
        $ancestor = $ancestor.Parent
    }
    $toolchainRoot = Join-Path $env:USERPROFILE '.rustup\toolchains\nightly-2024-10-15-x86_64-pc-windows-msvc'
    $rustcPath = Join-Path $toolchainRoot 'bin\rustc.exe'
    $lldPath = Join-Path $toolchainRoot 'lib\rustlib\x86_64-pc-windows-msvc\bin\rust-lld.exe'
    $rustFlags = @(
        '-Cpanic=abort', '-Copt-level=z', '-Ccodegen-units=1', '-Cdebuginfo=0', '-Cstrip=symbols',
        "-Clinker=$lldPath", "--remap-path-prefix=$inputRoot=/raios/source"
    )
    $encodedRustFlags = $rustFlags -join [char]0x1f
    $environment = [ordered]@{
        CARGO_HOME = $cargoHome
        CARGO_NET_OFFLINE = 'true'
        CARGO_INCREMENTAL = '0'
        SOURCE_DATE_EPOCH = '0'
        TZ = 'UTC'
        CARGO_ENCODED_RUSTFLAGS = $encodedRustFlags
        RUSTC = $rustcPath
        RUST_BACKTRACE = '0'
        TEMP = $temp
        TMP = $temp
        SystemRoot = $env:SystemRoot
        WINDIR = $env:WINDIR
        ComSpec = $env:ComSpec
        PATH = "$toolchainRoot\bin;$env:SystemRoot\System32"
    }
    $normalizedRustFlags = $encodedRustFlags.Replace($inputRoot, '<input>').Replace($toolchainRoot, '<toolchain>').Replace('\', '/')
    $flagsContract = "cargo_command=cargo build --frozen --offline --locked --manifest-path=<input>/Cargo.toml --target=wasm32-unknown-unknown --release --target-dir=<fresh>`nrustflags=$normalizedRustFlags`n"
    $environmentContract = "CARGO_INCREMENTAL=0`nCARGO_NET_OFFLINE=true`nRUST_BACKTRACE=0`nSOURCE_DATE_EPOCH=0`nTZ=UTC`n"
    $flagsContractSha256 = Get-Sha256Hex ([Text.Encoding]::UTF8.GetBytes($flagsContract))
    $environmentContractSha256 = Get-Sha256Hex ([Text.Encoding]::UTF8.GetBytes($environmentContract))
    if ($flagsContractSha256 -cne '34ae7dd2f0bfdc316745e7306377b5b6252451b5aca16559be8aa2058e6e0ed1') { throw 'fixed flags contract hash mismatch' }
    if ($environmentContractSha256 -cne '2e9fc15d9adabd8e7c2a59b108ed91a70306e593ca08ad7475cba2b8465f79dc') { throw 'fixed environment contract hash mismatch' }

    $toolchain = Get-ToolchainManifest $toolchainRoot $environment $logs $TimeoutSeconds
    $measuredFiles = @{}
    foreach ($file in $toolchain.manifest.files) { $measuredFiles[$file.path] = $file.sha256.Substring(7) }
    foreach ($expected in @(
        @{ path = 'bin/cargo.exe'; sha256 = 'beb0e989a1597b624895907b918127bc08da919107b39bd9e954b3a1836aa13a' },
        @{ path = 'bin/rustc.exe'; sha256 = '4fee4125dcdae28d01f0314a3820a09e9f9f717ee72494dc31ffe2d6de7eebf9' },
        @{ path = 'bin/rustc_driver-9a6caa80bb63f9f9.dll'; sha256 = 'e4aad069c7b57352c818aef521509ca42cc26cbfa5a20589ea9645bb4e111810' },
        @{ path = 'bin/std-0cd72b70e9ef0a54.dll'; sha256 = 'e54497532d6dcc306ada9a6086f30a2de5ed21740ab636414e569e017726fdfa' },
        @{ path = 'lib/rustlib/x86_64-pc-windows-msvc/bin/rust-lld.exe'; sha256 = '375f02b38440bc5d7b0c3974abeb0fef080b654dc2fb87ef23887df7116129d2' },
        @{ path = 'lib/rustlib/manifest-rust-std-wasm32-unknown-unknown'; sha256 = '032c9d596654633f8ab159236fff270bde984a15541763ebc6af5e2aaa560557' },
        @{ path = 'lib/rustlib/wasm32-unknown-unknown/lib/libcore-192e883b194683b6.rlib'; sha256 = 'b4b44cb5ed0b1f0928cf192d6f1f2985f3f8c9d0c49224cc34b5cd1cf3241df2' },
        @{ path = 'lib/rustlib/wasm32-unknown-unknown/lib/libcompiler_builtins-2422556c21c1b8e3.rlib'; sha256 = '7c6d40c3f358e980a0144edd71af39f111aa64574470ea330f785664c3f8ac32' },
        @{ path = 'lib/rustlib/wasm32-unknown-unknown/lib/librustc_std_workspace_core-6ec6ff816f15bedf.rlib'; sha256 = 'f52a86ec391b5faf6ba2bec28735cce77bc81f831f6e933d85d84373b76c6b92' },
        @{ path = 'lib/rustlib/wasm32-unknown-unknown/lib/libpanic_abort-489ea0fd8f9fb87b.rlib'; sha256 = '9a91adbf4db8087a3657732665325cb3339d92e818bf92ca3560b1bf86337019' }
    )) {
        if (-not $measuredFiles.ContainsKey($expected.path) -or $measuredFiles[$expected.path] -cne $expected.sha256) {
            throw "pinned toolchain component mismatch: $($expected.path)"
        }
    }
    $pinnedContractManifest = "schema=raios.owner_workstation_toolchain_manifest.v1`ncargo_version=cargo 1.84.0-nightly (15fbd2f60 2024-10-08)`ncargo_sha256=beb0e989a1597b624895907b918127bc08da919107b39bd9e954b3a1836aa13a`nrustc_version=rustc 1.84.0-nightly (9322d183f 2024-10-14)`nrustc_sha256=4fee4125dcdae28d01f0314a3820a09e9f9f717ee72494dc31ffe2d6de7eebf9`nrust_lld_sha256=375f02b38440bc5d7b0c3974abeb0fef080b654dc2fb87ef23887df7116129d2`nrustc_driver_sha256=e4aad069c7b57352c818aef521509ca42cc26cbfa5a20589ea9645bb4e111810`nhost_std_sha256=e54497532d6dcc306ada9a6086f30a2de5ed21740ab636414e569e017726fdfa`nwasm_target_manifest_sha256=032c9d596654633f8ab159236fff270bde984a15541763ebc6af5e2aaa560557`ntarget=wasm32-unknown-unknown`ntarget_lib_set_sha256=0ddff5952e3b6bbaeca08f78ea9e6841b833aef46fda393e7b4639e029267fde`n"
    $pinnedContractManifestSha256 = Get-Sha256Hex ([Text.Encoding]::UTF8.GetBytes($pinnedContractManifest))
    if ($pinnedContractManifestSha256 -cne '45723cc79127b688781d83eb3b042825fc2501c76ac02e4f8fee9f2904589e36') { throw 'pinned contract manifest hash mismatch' }
    if ($toolchain.target_lib_set_sha256 -cne '0ddff5952e3b6bbaeca08f78ea9e6841b833aef46fda393e7b4639e029267fde') { throw 'measured target library set does not match pinned contract' }
    $cargo = $toolchain.cargo
    $manifestPath = Join-Path $inputRoot 'Cargo.toml'
    $commonCargoArgs = @('--frozen', '--offline', '--locked', '--manifest-path', $manifestPath)
    $metadataRun = Invoke-IsolatedTool $cargo (@('metadata', '--format-version', '1') + $commonCargoArgs) $inputRoot $environment $logs 'cargo-metadata' $TimeoutSeconds
    if ($metadataRun.exit_code -ne 0) { throw 'cargo metadata denied the materialized project' }
    try { $metadata = [Text.Encoding]::UTF8.GetString($metadataRun.stdout_bytes) | ConvertFrom-Json }
    catch { throw 'cargo metadata returned malformed JSON' }
    $packages = @($metadata.packages)
    $expectedPackageCount = if ($dependencyEnabled) { 2 } else { 1 }
    if ($packages.Count -ne $expectedPackageCount -or @($metadata.resolve.nodes).Count -ne $expectedPackageCount) { throw 'cargo package graph count mismatch' }
    if (@($packages | Where-Object { $null -ne $_.source }).Count) { throw 'registry or non-local Cargo source denied' }
    if (@($packages.targets | Where-Object { @($_.kind) -contains 'custom-build' -or @($_.kind) -contains 'proc-macro' }).Count) { throw 'custom-build or proc-macro target denied' }
    $rootPackage = @($packages | Where-Object { $_.name -ceq $ExpectedPackageName -and $_.version -ceq $ExpectedPackageVersion })
    if ($rootPackage.Count -ne 1 -or [IO.Path]::GetFullPath($rootPackage[0].manifest_path) -cne $manifestPath) { throw 'root package metadata mismatch' }
    $cdylibTargets = @($rootPackage[0].targets | Where-Object { @($_.crate_types) -contains 'cdylib' })
    if ($cdylibTargets.Count -ne 1 -or @($rootPackage[0].targets).Count -ne 1) { throw 'exactly one root cdylib target is required' }
    $rootDependencies = @($rootPackage[0].dependencies)
    if ($dependencyEnabled) {
        if ($rootDependencies.Count -ne 1 -or $rootDependencies[0].name -cne $ExpectedDependencyName -or $rootDependencies[0].source) { throw 'root path dependency mismatch' }
        $dependencyPackage = @($packages | Where-Object { $_.name -ceq $ExpectedDependencyName -and $_.version -ceq $ExpectedDependencyVersion })
        if ($dependencyPackage.Count -ne 1 -or @($dependencyPackage[0].dependencies).Count) { throw 'dependency package graph mismatch' }
        $dependencyManifest = [IO.Path]::GetFullPath($dependencyPackage[0].manifest_path)
        if (-not $dependencyManifest.StartsWith($vendorRoot + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'dependency escaped vendor root' }
    } elseif ($rootDependencies.Count) { throw 'dependencies are denied for dependency-free build' }

    $buildFacts = @()
    foreach ($buildId in @('a', 'b')) {
        $targetDir = Join-Path $outputRoot "target-$buildId"
        [IO.Directory]::CreateDirectory($targetDir) | Out-Null
        $run = Invoke-IsolatedTool $cargo (@('build') + $commonCargoArgs + @('--target', 'wasm32-unknown-unknown', '--release', '--target-dir', $targetDir)) $inputRoot $environment $logs "cargo-build-$buildId" $TimeoutSeconds
        if ($run.exit_code -ne 0) { throw "cargo build $buildId failed" }
        $releaseDir = Join-Path $targetDir 'wasm32-unknown-unknown\release'
        $wasmFiles = @(Get-ChildItem -LiteralPath $releaseDir -Filter '*.wasm' -File -ErrorAction SilentlyContinue)
        if ($wasmFiles.Count -ne 1) { throw "build $buildId did not produce exactly one top-level Wasm artifact" }
        if ($wasmFiles[0].Length -le 0 -or $wasmFiles[0].Length -gt 262144) { throw "build $buildId Wasm artifact exceeds the 256 KiB bound" }
        $buildFacts += [pscustomobject][ordered]@{
            build_id = $buildId
            run = Get-RunFact $run
            wasm_path = $wasmFiles[0].FullName
            wasm_byte_len = [uint64]$wasmFiles[0].Length
            wasm_sha256 = "sha256:$(Get-FileSha256Hex $wasmFiles[0].FullName)"
        }
    }
    if ($buildFacts[0].wasm_byte_len -ne $buildFacts[1].wasm_byte_len -or $buildFacts[0].wasm_sha256 -cne $buildFacts[1].wasm_sha256) {
        throw 'two-build Wasm output is not byte-identical'
    }
    $bytesA = [IO.File]::ReadAllBytes($buildFacts[0].wasm_path)
    $bytesB = [IO.File]::ReadAllBytes($buildFacts[1].wasm_path)
    if (-not [System.Collections.StructuralComparisons]::StructuralEqualityComparer.Equals($bytesA, $bytesB)) { throw 'two-build Wasm bytes differ' }
    [IO.File]::Copy($buildFacts[0].wasm_path, $script:CandidatePath, $false)
    if ((Get-FileSha256Hex $script:CandidatePath) -cne $buildFacts[0].wasm_sha256.Substring(7)) { throw 'candidate copy hash mismatch' }

    $inputFiles = @($tree.files | ForEach-Object { [pscustomobject][ordered]@{ path = $_.path; byte_len = $_.byte_len; sha256 = "sha256:$($_.sha256)" } })
    $result = [pscustomobject][ordered]@{
        format = 'project-build-workstation-output-1'
        status = 'accepted'
        reason = 'reproducible_wasm_built'
        builder_kind = 'owner_controlled_windows_workstation'
        authority = 'non_authorizing_receipt'
        project_id = $ProjectId
        project_revision_sha256 = "sha256:$ProjectRevisionSha256"
        source_tree_sha256 = "sha256:$ExpectedSourceTreeSha256"
        materialized_tree_sha256 = "sha256:$($tree.tree_sha256)"
        cargo_lock_sha256 = "sha256:$cargoLockSha256"
        package = [pscustomobject][ordered]@{ name = $ExpectedPackageName; version = $ExpectedPackageVersion }
        dependency = if ($dependencyEnabled) { [pscustomobject][ordered]@{ name = $ExpectedDependencyName; version = $ExpectedDependencyVersion; bundle_sha256 = "sha256:$ExpectedDependencyBundleSha256" } } else { $null }
        input_files = $inputFiles
        target = 'wasm32-unknown-unknown'
        profile = 'release'
        offline = $true
        network_attempted = $false
        build_script_attempted = $false
        proc_macro_attempted = $false
        normalization_attempted = $false
        flags_contract_sha256 = "sha256:$flagsContractSha256"
        environment_contract_sha256 = "sha256:$environmentContractSha256"
        measured_toolchain_manifest_sha256 = "sha256:$($toolchain.manifest_sha256)"
        pinned_contract_manifest_sha256 = "sha256:$pinnedContractManifestSha256"
        toolchain = $toolchain.manifest
        metadata_run = Get-RunFact $metadataRun
        builds = @($buildFacts | ForEach-Object { [pscustomobject][ordered]@{ build_id = $_.build_id; run = $_.run; wasm_byte_len = $_.wasm_byte_len; wasm_sha256 = $_.wasm_sha256 } })
        reproducible = $true
        candidate = [pscustomobject][ordered]@{ path = 'candidate.wasm'; byte_len = [uint64]$bytesA.Length; sha256 = $buildFacts[0].wasm_sha256 }
    }
    Write-ResultJson $result $script:ResultPath
    Write-Output "build_result=$script:ResultPath"
    Write-Output "candidate=$script:CandidatePath"
    Write-Output "candidate_sha256=$($buildFacts[0].wasm_sha256.Substring(7))"
}
catch {
    if ($script:CandidatePath -and (Test-Path -LiteralPath $script:CandidatePath)) { Remove-Item -LiteralPath $script:CandidatePath -Force }
    if ($script:ResultPath) {
        $failure = [pscustomobject][ordered]@{
            format = 'project-build-workstation-output-1'
            status = 'denied'
            reason = 'host_builder_failed'
            diagnostic = $_.Exception.Message
            reproducible = $false
            candidate = $null
            network_attempted = $false
            normalization_attempted = $false
        }
        try { Write-ResultJson $failure $script:ResultPath } catch {}
    }
    throw
}
