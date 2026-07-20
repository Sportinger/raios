param([switch]$SkipBuild)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot

function Require([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Require-Match([string]$Text, [string]$Pattern, [string]$Message) {
    Require ([regex]::IsMatch($Text, $Pattern, [Text.RegularExpressions.RegexOptions]::Singleline)) $Message
}

function Require-NoMatch([string]$Text, [string]$Pattern, [string]$Message) {
    Require (-not [regex]::IsMatch($Text, $Pattern, [Text.RegularExpressions.RegexOptions]::Singleline)) $Message
}

function Slice-Between([string]$Text, [string]$Start, [string]$End) {
    $startAt = $Text.IndexOf($Start, [StringComparison]::Ordinal)
    Require ($startAt -ge 0) "missing fixture start: $Start"
    $endAt = $Text.IndexOf($End, $startAt + $Start.Length, [StringComparison]::Ordinal)
    Require ($endAt -gt $startAt) "missing fixture end: $End"
    $Text.Substring($startAt, $endAt - $startAt)
}

$wifiPath = Join-Path $RepoRoot "seed-kernel\src\shell_host\wifi_flow.rs"
$overlayPath = Join-Path $RepoRoot "seed-kernel\src\secure_overlay.rs"
$vaultPath = Join-Path $RepoRoot "seed-kernel\src\secret_vault\mod.rs"
$driverPath = Join-Path $RepoRoot "seed-kernel\src\marvell_wifi_pcie.rs"
$wifi = Get-Content -LiteralPath $wifiPath -Raw
$overlay = Get-Content -LiteralPath $overlayPath -Raw
$vault = Get-Content -LiteralPath $vaultPath -Raw
$driver = Get-Content -LiteralPath $driverPath -Raw

$submit = Slice-Between $wifi "fn submit_ephemeral_physical_password" "fn cancel_secure_password"
Require-Match $submit 'BootPosture::PersistenceUnavailable' "submit lacks PU posture gate"
Require-Match $submit 'same_ephemeral_live_target' "submit lacks fresh target check"
Require-Match $submit 'SecureOverlayInput::Submit' "submit bypasses SecureOverlay"
Require-Match $submit 'begin_ephemeral_physical_wifi_use' "submit lacks linear facade join"
Require-Match $submit 'start_ephemeral_association_from_physical_genesis' "submit lacks typed driver join"
Require-NoMatch $submit 'set_passphrase|set_remember|save_or_replace|wifi_status|provider|grant|audit' "submit reaches forbidden state"

$selection = Slice-Between $wifi "fn select_pointer" "pub fn draw"
$puAt = $selection.IndexOf("BootPosture::PersistenceUnavailable", [StringComparison]::Ordinal)
$openAt = $selection.IndexOf("Dot11Security::Open", [StringComparison]::Ordinal)
Require (($puAt -ge 0) -and ($openAt -gt $puAt)) "Open network is considered before PU deny"
Require-Match $selection 'association_ready\(\).*ScanSource::LiveRadio.*supports_wpa2_psk_ccmp' "selection lacks exact live WPA2 gate"
Require-NoMatch (Slice-Between $selection "BootPosture::PersistenceUnavailable" "if network.security == Dot11Security::Open") 'LegacyRamOnly|set_passphrase|set_remember|VaultSecretStatus' "PU selection can reach legacy or Vault"

$authorityDecl = Slice-Between $vault "pub(crate) struct EphemeralPhysicalWifiUse" "/// Non-secret proof retained"
Require-NoMatch $authorityDecl '\bpub(?:\(crate\))?\s+[a-zA-Z_][a-zA-Z0-9_]*\s*:' "authority exposes a field"
Require-NoMatch $vault 'impl\s+(?:Clone|Copy|core::fmt::Debug|Debug)\s+for\s+EphemeralPhysicalWifiUse' "authority gained copy/debug implementation"
Require-Match $vault 'attempt_id:\s*u64.*boot_scope:\s*EphemeralWifiBootScope.*secret:\s*SecretPlaintext' "authority lacks attempt/current-boot/secret binding"
Require-Match $vault 'target\.ssid\.as_bytes\(\).*target\.bssid.*target\.channel.*EPHEMERAL_WIFI_SECURITY_WPA2_PSK_CCMP.*target\.security_ie\(\)' "target hash omits SSID/BSSID/channel/RSN"

$constructors = Get-ChildItem -LiteralPath (Join-Path $RepoRoot "seed-kernel\src") -Recurse -Filter "*.rs" |
    Select-String -Pattern 'EphemeralPhysicalWifiUse\s*\{'
Require ($constructors.Count -eq 3) "authority construction/destructure surface changed"
Require (($constructors | Where-Object { $_.Path -ne $vaultPath }).Count -eq 0) "authority can be constructed outside facade"
Require-Match $overlay 'into_plaintext_for_ephemeral_physical_wifi.*WifiPassphrase.*SecretKind::WifiPassphrase' "overlay transfer is not WiFi-specific"

$entry = Slice-Between $driver "pub(crate) fn start_ephemeral_association_from_physical_genesis" "fn start_association_inner"
Require-Match $entry '!= BootPosture::PersistenceUnavailable' "driver entry lacks exact posture gate"
Require-Match $entry 'revalidate_ephemeral_physical_wifi_use' "driver entry lacks target revalidation"
Require-NoMatch $entry 'Normal|Probation|Safe|ExplicitSafeWifiReconnect|wifi_status|format_legacy' "driver entry accepts another authority"

$pmk = Slice-Between $driver "ConnectionSecretSource::EphemeralPhysical { pending, receipt } =>" "ConnectionSecretSource::Ordinary =>"
Require-Match $pmk 'pending\s*\.take\(\).*write_ephemeral_physical_wifi_pmk.*\*receipt\s*=\s*Some' "PMK path is not single-use"
Require-NoMatch $pmk 'wifi_status|write_wifi_pmk_for_association|write_wifi_pmk_for_safe_association|format_legacy' "ephemeral PMK can fall back"
Require (([regex]::Matches($driver, 'revalidate_ephemeral_release\(&job\)')).Count -eq 2) "port-release/net-attach revalidation count changed"

$generic = Slice-Between $driver "pub fn start_association()" "/// The only SAFE association entrypoint"
Require-Match $generic 'BootPosture::PersistenceUnavailable.*BootPostureDenied' "generic PU deny was weakened"
$linkLoss = Slice-Between $driver "fn handle_connection_event" "pub fn poll_rx_ring"
Require-Match $linkLoss 'disable_bus_master' "link loss does not disable bus master"
Require-Match $linkLoss 'LinkLost' "link loss result missing"
Require-Match $linkLoss 'DATA_LINK_READY\.store\(false.*net::detach_wifi' "link loss does not detach network"
Require-NoMatch $linkLoss 'start_association|reconnect|begin_' "link loss gained host autoreconnect"

$agentSurface = Get-ChildItem -LiteralPath (Join-Path $RepoRoot "seed-kernel\src") -Filter "agent_protocol*.rs" |
    Get-Content -Raw
Require-NoMatch ($agentSurface -join "`n") 'start_ephemeral_association|begin_ephemeral_physical_wifi_use' "agent protocol exposes ephemeral authority"
Require-NoMatch ($wifi + $vault + $driver) 'serial::[^\r\n]*(?:target_binding_sha256|attempt_id|secret\.len|submission\.len)' "secret metadata reaches serial"

if (-not $SkipBuild) {
    $oldCargoHome = $env:CARGO_HOME
    $oldTarget = $env:CARGO_TARGET_DIR
    $oldRustFlags = $env:RUSTFLAGS
    try {
        if ([string]::IsNullOrWhiteSpace($env:CARGO_HOME) -or -not (Test-Path -LiteralPath $env:CARGO_HOME)) {
            $env:CARGO_HOME = Split-Path -Parent (Split-Path -Parent (Get-Command cargo).Source)
        }
        if ([string]::IsNullOrWhiteSpace($env:CARGO_TARGET_DIR) -or -not (Test-Path -LiteralPath (Split-Path -Parent $env:CARGO_TARGET_DIR))) {
            $env:CARGO_TARGET_DIR = Join-Path $env:TEMP "raios-wifi-ephemeral-fixture-target"
        }
        $linker = (Resolve-Path (Join-Path $RepoRoot "seed-kernel\linker.ld")).Path
        $env:RUSTFLAGS = "-C link-arg=-T$linker -C relocation-model=static -C code-model=kernel -C force-frame-pointers=yes -C link-arg=--gc-sections"
        $cargoArgs = @('+nightly-2024-10-15', '-Zbuild-std=core,compiler_builtins,alloc', 'build', '--locked', '--target', (Join-Path $RepoRoot 'seed-kernel\x86_64-seed.json'), '-p', 'seed-kernel')
        & cargo @cargoArgs
        if ($LASTEXITCODE -ne 0) { throw "freestanding seed-kernel build failed" }
    }
    finally {
        $env:CARGO_HOME = $oldCargoHome
        $env:CARGO_TARGET_DIR = $oldTarget
        $env:RUSTFLAGS = $oldRustFlags
    }
}

Write-Output "WIFI_EPHEMERAL_PHYSICAL_FIXTURE status=pass physical_only=true current_boot=true persistent=false autoreconnect=false"
