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
$netPath = Join-Path $RepoRoot "seed-kernel\src\net.rs"
$wifi = Get-Content -LiteralPath $wifiPath -Raw
$overlay = Get-Content -LiteralPath $overlayPath -Raw
$vault = Get-Content -LiteralPath $vaultPath -Raw
$driver = Get-Content -LiteralPath $driverPath -Raw
$net = Get-Content -LiteralPath $netPath -Raw

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
$linkLoss = Slice-Between $driver "fn handle_connection_event" "pub fn poll()"
Require-Match $linkLoss 'disable_bus_master' "link loss does not disable bus master"
Require-Match $linkLoss 'LinkLost' "link loss result missing"
Require-Match $linkLoss 'DATA_LINK_READY\.store\(false.*DeferredNetworkAction::Detach' "link loss does not plan network detach"
Require-NoMatch $linkLoss 'start_association|reconnect|begin_' "link loss gained host autoreconnect"

$receiveRing = Slice-Between $driver "pub fn receive_ethernet" "pub fn transmit_ethernet"
$transmitRing = Slice-Between $driver "pub fn transmit_ethernet" "pub fn start_bring_up_firmware"
$eventRing = Slice-Between $driver "pub fn poll_event_ring" "fn handle_connection_event"
$firmwarePoll = Slice-Between $driver "pub fn poll()" "fn finish_locked"
$hwSpecPoll = Slice-Between $driver "pub fn poll_hw_spec" "pub fn poll_scan_ext"
$scanPoll = Slice-Between $driver "pub fn poll_scan_ext" "fn clear_connection_response_mailbox"
$connectionPoll = Slice-Between $driver "pub fn poll_connection" "fn next_connection_phase"
$networkApply = Slice-Between $driver "fn apply_deferred_network_action" "pub fn start_association()"
$firmwareTrigger = Slice-Between $driver "pub fn start_bring_up_firmware" "pub fn start_scan_ext_24ghz"
$scanTrigger = Slice-Between $driver "pub fn start_scan_ext_24ghz" "pub fn poll_hw_spec"
$associationStart = Slice-Between $driver "fn start_association_inner" "fn same_connection_target"
$connectionArm = Slice-Between $driver "fn arm_connection_command" "pub fn poll_connection"
$eventArm = Slice-Between $driver "fn arm_event_ring_while_gated" "fn arm_rx_ring_while_gated"
$rxArm = Slice-Between $driver "fn arm_rx_ring_while_gated" "fn arm_tx_ring_while_gated"
$ringInit = Slice-Between $driver "fn arm_data_rings_while_gated" "fn plan_shared_rx_tx_pointer"
$sharedPointer = Slice-Between $driver "fn plan_shared_rx_tx_pointer" "fn finish_hw_spec_locked"
$initialPointers = Slice-Between $driver "fn publish_data_ring_pointers_while_gated" "fn finish_hw_spec_locked"

Require-NoMatch $driver '\bpoll_rx_ring\b' "dead competing RX consumer remains"
Require (([regex]::Matches($driver, 'write_reg\(\s*mmio_base,\s*PCIE_RX_RD_TX_WR_PTR,\s*next_raw\s*\)')).Count -eq 1) "shared 0xC05C register does not have exactly one explicit writer"
Require (([regex]::Matches($driver, 'publish_shared_rx_tx_pointer_while_gated\(')).Count -eq 3) "shared pointer publisher is not limited to definition plus RX/TX"
Require (([regex]::Matches($driver, 'publish_prepared_shared_rx_tx_pointer_while_gated\(')).Count -eq 3) "prepared shared pointer writer is not limited to definition plus validated runtime/init plans"
Require-Match $sharedPointer 'compose_rx_tx_pointer_register.*update_rx_pointer_preserving_tx.*update_tx_pointer_preserving_rx' "shared pointer publisher bypasses pure composer/update predicates"
Require-Match $receiveRing 'decode_rx_tx_pointer_register\(wrptr\).*parse_rx_ethernet\(index\).*arm_rx_desc\(index\).*publish_shared_rx_tx_pointer_while_gated' "RX device pointer is not validated before DMA/rearm/shared publication"
Require-Match $transmitRing 'decode_rx_tx_pointer_register\(rdptr\).*prepare_tx_buffer\(index.*publish_shared_rx_tx_pointer_while_gated.*CPU_INTR_DNLD_RDY' "TX device pointer is not validated before DMA/shared publication/doorbell"
Require-Match $eventRing 'decode_event_pointer\(wrptr\).*parse_event_buffer\(rd_index\).*arm_event_desc\(rd_index\)' "event device pointer is not validated before DMA/rearm"
Require-Match $eventArm 'decode_event_pointer\(wrptr\).*event_data_phys\(index\)' "event init touches DMA descriptors before pointer validation"
Require-Match $rxArm 'decode_rx_tx_pointer_register\(wrptr\).*rx_data_phys\(index\)' "RX init touches DMA descriptors before pointer validation"
Require-Match $initialPointers 'decode_event_pointer\(event_rdptr\).*plan_shared_rx_tx_pointer.*Err\(_\).*return false.*PCIE_EVT_RD_PTR.*publish_prepared_shared_rx_tx_pointer_while_gated' "ring init can publish EVENT/shared pointers before the complete host plan is valid"

function Test-GateOrder([string]$Text, [string]$InnerLock) {
    $gateAt = $Text.IndexOf('MARVELL_DMA_GATE.lock', [StringComparison]::Ordinal)
    if ($gateAt -lt 0) { return $false }
    $innerAt = $Text.IndexOf($InnerLock, $gateAt + 1, [StringComparison]::Ordinal)
    if ($innerAt -le $gateAt) { return $false }
    $gatedPrefix = $Text.Substring($gateAt, $innerAt - $gateAt)
    if ([regex]::IsMatch($Text, 'drop\(_dma_gate\)')) { return $false }
    [regex]::IsMatch(
        $gatedPrefix,
        'if\s+connection_reboot_required\(\)(?:\s*\|\|[^\{]+)?\s*\{[^\{\}]*\breturn\b',
        [Text.RegularExpressions.RegexOptions]::Singleline
    )
}

foreach ($gateCheck in @(
    @{ Text = $receiveRing; Lock = 'RX_RING.lock'; Name = 'receive' },
    @{ Text = $transmitRing; Lock = 'TX_RING.lock'; Name = 'transmit' },
    @{ Text = $firmwareTrigger; Lock = 'BRINGUP.lock'; Name = 'firmware trigger' },
    @{ Text = $scanTrigger; Lock = 'SCAN.lock'; Name = 'scan trigger' },
    @{ Text = $associationStart; Lock = 'arm_data_rings_while_gated'; Name = 'association trigger' },
    @{ Text = $eventRing; Lock = 'EVENT_RING.lock'; Name = 'event' },
    @{ Text = $firmwarePoll; Lock = 'BRINGUP.lock'; Name = 'firmware init' },
    @{ Text = $hwSpecPoll; Lock = 'HWSPEC.lock'; Name = 'hardware-spec init' },
    @{ Text = $scanPoll; Lock = 'SCAN.lock'; Name = 'scan' },
    @{ Text = $connectionPoll; Lock = 'CONNECTION.lock'; Name = 'connection quarantine' },
    @{ Text = $linkLoss; Lock = 'CONNECTION.lock'; Name = 'event connection handling' }
)) {
    Require (Test-GateOrder $gateCheck.Text $gateCheck.Lock) "$($gateCheck.Name) does not hold DMA gate then check reboot latch before its inner lock/action"
}
foreach ($simpleGatedSurface in @($receiveRing, $transmitRing, $firmwareTrigger, $scanTrigger, $eventRing, $firmwarePoll, $hwSpecPoll, $scanPoll)) {
    Require-NoMatch $simpleGatedSurface 'net::(?:attach_wifi|detach_wifi)|apply_deferred_network_action' "simple DMA-gated path reaches NET_STATE"
}
Require-Match $eventRing '\};\s*if let Some\(parsed\) = serial_event\s*\{\s*handle_connection_event' "event processing can retain EVENT/DMA gate while acquiring CONNECTION"
Require-Match $scanTrigger 'connection_reboot_required\(\).*finish_scan_without_job\(ScanCmdResult::RebootRequired.*ScanCmdTriggerResult::Failed\(ScanCmdResult::RebootRequired\).*SCAN\.lock.*runtime\.job = Some.*wifi::note_scan_command_started' "scan trigger does not fail closed before job/start publication after reboot latch"
Require-NoMatch $networkApply 'MARVELL_DMA_GATE\.lock|CONNECTION\.lock' "deferred network executor can recreate a NET_STATE lock cycle"
$attachChecks = [regex]::Matches($networkApply, 'connection_reboot_required\(\) \|\| !data_link_ready\(\)')
Require ($attachChecks.Count -eq 2) "deferred attach lacks immediate pre/post reboot-ready checks"
$attachAt = $networkApply.IndexOf('net::attach_wifi(mac)', [StringComparison]::Ordinal)
Require (($attachAt -gt $attachChecks[0].Index) -and ($attachAt -lt $attachChecks[1].Index)) "deferred attach is not bracketed by reboot-ready checks"

function Test-DeferredNetworkAfterGateIife(
    [string]$Text,
    [string]$Start,
    [string]$End,
    [string]$IifeStart
) {
    $surface = Slice-Between $Text $Start $End
    $iifeAt = $surface.IndexOf($IifeStart, [StringComparison]::Ordinal)
    if ($iifeAt -lt 0) { return $false }
    $gateAt = $surface.IndexOf('MARVELL_DMA_GATE.lock', $iifeAt, [StringComparison]::Ordinal)
    $dropNeedle = '    })();'
    $dropAt = $surface.IndexOf($dropNeedle, $gateAt, [StringComparison]::Ordinal)
    if (($gateAt -le $iifeAt) -or ($dropAt -le $gateAt)) { return $false }
    if (([regex]::Matches($surface, 'MARVELL_DMA_GATE\.lock')).Count -ne 1) { return $false }
    if (([regex]::Matches($surface, 'apply_deferred_network_action\(network_action\)')).Count -ne 1) { return $false }
    $inside = $surface.Substring($iifeAt, $dropAt - $iifeAt)
    if ([regex]::IsMatch($inside, 'apply_deferred_network_action|net::(?:attach_wifi|detach_wifi)')) { return $false }
    $tail = $surface.Substring($dropAt + $dropNeedle.Length)
    [regex]::IsMatch($tail, '^\s*apply_deferred_network_action\(network_action\);')
}

Require (Test-DeferredNetworkAfterGateIife $driver "pub fn poll_connection" "fn next_connection_phase" 'let (changed, network_action) = (|| {') "connection apply is not uniquely after its gate IIFE"
Require (Test-DeferredNetworkAfterGateIife $driver "fn handle_connection_event" "pub fn poll()" 'let network_action = (|| {') "event apply is not uniquely after its gate IIFE"
Require (([regex]::Matches($driver, 'apply_deferred_network_action\(')).Count -eq 3) "deferred network helper allowlist changed"
Require (([regex]::Matches($networkApply, 'apply_deferred_network_action\(')).Count -eq 1) "deferred network helper definition count changed"

function Test-AssociationGatedPublication([string]$Text) {
    $surface = Slice-Between $Text "fn start_association_inner" "fn same_connection_target"
    $iifeAt = $surface.IndexOf('let gated_start = (move || {', [StringComparison]::Ordinal)
    if ($iifeAt -lt 0) { return $false }
    $gateAt = $surface.IndexOf('MARVELL_DMA_GATE.lock', $iifeAt, [StringComparison]::Ordinal)
    if ($gateAt -le $iifeAt) { return $false }
    $latchAt = $surface.IndexOf('if connection_reboot_required()', $gateAt, [StringComparison]::Ordinal)
    if ($latchAt -le $gateAt) { return $false }
    $ringsAt = $surface.IndexOf('arm_data_rings_while_gated(mmio_base)', $latchAt, [StringComparison]::Ordinal)
    if ($ringsAt -le $latchAt) { return $false }
    $dmaAt = $surface.IndexOf('(connect_cmd_phys(), connect_rsp_phys())', $ringsAt, [StringComparison]::Ordinal)
    if ($dmaAt -le $ringsAt) { return $false }
    $connectionAt = $surface.IndexOf('CONNECTION.lock', $dmaAt, [StringComparison]::Ordinal)
    if ($connectionAt -le $dmaAt) { return $false }
    $jobAt = $surface.IndexOf('runtime.job = Some(ConnectionJob', $connectionAt, [StringComparison]::Ordinal)
    if ($jobAt -le $connectionAt) { return $false }
    $dropAt = $surface.IndexOf('    })();', $jobAt, [StringComparison]::Ordinal)
    if (-not (($iifeAt -ge 0) -and ($gateAt -gt $iifeAt) -and ($latchAt -gt $gateAt) -and ($ringsAt -gt $latchAt) -and ($dmaAt -gt $ringsAt) -and ($connectionAt -gt $dmaAt) -and ($jobAt -gt $connectionAt) -and ($dropAt -gt $jobAt))) { return $false }
    if (([regex]::Matches($surface, 'MARVELL_DMA_GATE\.lock')).Count -ne 1) { return $false }
    $inside = $surface.Substring($iifeAt, $dropAt - $iifeAt)
    if ([regex]::IsMatch($inside, 'net::(?:attach_wifi|detach_wifi)|fail_connection_start|\barm_data_rings\(')) { return $false }
    $tail = $surface.Substring($dropAt + '    })();'.Length)
    [regex]::IsMatch($tail, 'match gated_start.*net::detach_wifi.*fail_connection_start', [Text.RegularExpressions.RegexOptions]::Singleline)
}

Require (Test-AssociationGatedPublication $driver) "association job publication is not atomic with fresh latch/ring/DMA validation"
Require (([regex]::Matches($driver, 'arm_data_rings_while_gated\(')).Count -eq 3) "gated ring initialization gained an unreviewed caller"

$readyQuarantine = Slice-Between $driver "fn quarantine_retained_ready_connection" "fn fail_connection_start"
$failedStart = Slice-Between $driver "fn fail_connection_start" "pub fn receive_ethernet"
Require-Match $readyQuarantine 'MARVELL_DMA_GATE\.lock.*CONNECTION\.lock.*\}\s*net::detach_wifi' "ready-connection quarantine does not release DMA/CONNECTION before NET_STATE"
Require-Match $failedStart 'MARVELL_DMA_GATE\.lock.*CONNECTION\.lock.*\}\s*net::detach_wifi' "failed connection start does not release DMA/CONNECTION before NET_STATE"

$writeOwners = @{}
$currentFunction = ''
foreach ($line in ($driver -split '\r?\n')) {
    if ($line -match '^(?:pub(?:\([^)]*\))?\s+)?fn\s+([A-Za-z0-9_]+)') {
        $currentFunction = $Matches[1]
    }
    if (($line -match '\bwrite_reg\(') -and $currentFunction -ne 'write_reg') {
        $writeOwners[$currentFunction] = $true
    }
}
$expectedWriteOwners = @(
    'arm_connection_command',
    'clear_connection_response_mailbox',
    'poll',
    'poll_connection',
    'poll_event_ring',
    'poll_hw_spec',
    'poll_scan_ext',
    'publish_data_ring_pointers_while_gated',
    'publish_prepared_shared_rx_tx_pointer_while_gated',
    'receive_ethernet',
    'transmit_ethernet',
    'write_drv_ready_pre_quarantined'
) | Sort-Object
$actualWriteOwners = @($writeOwners.Keys | Sort-Object)
Require (($actualWriteOwners -join ',') -eq ($expectedWriteOwners -join ',')) "MMIO writer ownership changed: $($actualWriteOwners -join ',')"
Require (([regex]::Matches($driver, 'clear_connection_response_mailbox\(')).Count -eq 6) "mailbox clear gained an unreviewed caller"
Require (([regex]::Matches($driver, 'arm_connection_command\(')).Count -eq 2) "connection DMA arming gained an unreviewed caller"
Require (([regex]::Matches($driver, 'write_drv_ready_pre_quarantined\(')).Count -eq 2) "DRV_READY writer gained an unreviewed caller"
Require (([regex]::Matches($driver, 'publish_data_ring_pointers_while_gated\(')).Count -eq 2) "ring-pointer publication gained an unreviewed caller"
Require (([regex]::Matches($driver, 'pci::enable_bus_master\(')).Count -eq 2) "direct bus-master enable surface changed"
Require-Match $firmwareTrigger 'connection_reboot_required\(\).*BRINGUP\.lock.*pci::enable_bus_master' "firmware trigger can enable bus mastering outside its reboot-checked gate"
Require-Match $scanPoll 'connection_reboot_required\(\).*SCAN\.lock.*pci::enable_bus_master.*CPU_INTR_DOOR_BELL' "scan can enable bus mastering or doorbell outside its reboot-checked gate"
Require (([regex]::Matches($driver, 'ensure_pci_memory_bus_master\(')).Count -eq 3) "checked bus-master enable gained an unreviewed caller"
Require-Match $connectionArm 'ensure_pci_memory_bus_master.*CPU_INTR_DOOR_BELL' "connection DMA helper no longer bounds bus-master enable before doorbell"

function Test-InitialPointerPlanBeforeMmio([string]$Text) {
    $surface = Slice-Between $Text "fn publish_data_ring_pointers_while_gated" "fn finish_hw_spec_locked"
    $eventPlanAt = $surface.IndexOf('decode_event_pointer(event_rdptr)', [StringComparison]::Ordinal)
    $sharedPlanAt = $surface.IndexOf('let shared_raw = match plan_shared_rx_tx_pointer', [StringComparison]::Ordinal)
    $sharedRejectAt = $surface.IndexOf('Err(_) =>', $sharedPlanAt, [StringComparison]::Ordinal)
    $rejectReturnAt = $surface.IndexOf('return false', $sharedRejectAt, [StringComparison]::Ordinal)
    $eventWriteAt = $surface.IndexOf('PCIE_EVT_RD_PTR', [StringComparison]::Ordinal)
    $sharedWriteAt = $surface.IndexOf('publish_prepared_shared_rx_tx_pointer_while_gated(mmio, shared_raw)', [StringComparison]::Ordinal)
    ($eventPlanAt -ge 0) -and
        ($sharedPlanAt -gt $eventPlanAt) -and
        ($sharedRejectAt -gt $sharedPlanAt) -and
        ($rejectReturnAt -gt $sharedRejectAt) -and
        ($eventWriteAt -gt $rejectReturnAt) -and
        ($sharedWriteAt -gt $eventWriteAt)
}

Require (Test-InitialPointerPlanBeforeMmio $driver) "initial EVENT/shared pointer publication can start before complete host validation"

$netPoll = Slice-Between $net "pub fn poll()" "pub fn ui_snapshot"
$wifiPhy = Slice-Between $net "struct WifiPhy;" "struct DnsParseResult"
Require-Match $netPoll 'NET_STATE\.lock.*NetBackend::Wifi.*iface\.poll' "NET_STATE -> WifiPhy poll counter-edge is no longer explicit"
Require-Match $wifiPhy 'fn receive.*marvell_wifi_pcie::receive_ethernet.*fn transmit.*WifiTxToken.*impl TxToken for WifiTxToken.*marvell_wifi_pcie::transmit_ethernet' "WifiPhy RX/TX no longer proves NET_STATE -> DMA_GATE counter-edge"

function Test-NoDirectNetworkInGatedPolls([string]$Text) {
    $connection = Slice-Between $Text "pub fn poll_connection" "fn next_connection_phase"
    $event = Slice-Between $Text "fn handle_connection_event" "pub fn poll()"
    -not [regex]::IsMatch(
        $connection + $event,
        'net::(?:attach_wifi|detach_wifi)',
        [Text.RegularExpressions.RegexOptions]::Singleline
    )
}

$scanStart = $driver.IndexOf('pub fn poll_scan_ext', [StringComparison]::Ordinal)
$scanGateNeedle = '    let _dma_gate = MARVELL_DMA_GATE.lock();'
$scanGateAt = $driver.IndexOf($scanGateNeedle, $scanStart, [StringComparison]::Ordinal)
Require ($scanGateAt -gt $scanStart) "scan gate mutation fixture could not find gate"
$scanWithoutGate = $driver.Remove($scanGateAt, $scanGateNeedle.Length)
$mutatedScanPoll = Slice-Between $scanWithoutGate "pub fn poll_scan_ext" "fn clear_connection_response_mailbox"
Require (-not (Test-GateOrder $mutatedScanPoll 'SCAN.lock')) "missing scan gate mutation was accepted"

$connectionStart = $driver.IndexOf('pub fn poll_connection', [StringComparison]::Ordinal)
$connectionGateDrop = $driver.IndexOf('    })();', $connectionStart, [StringComparison]::Ordinal)
Require ($connectionGateDrop -gt $connectionStart) "network lock-order mutation fixture could not find gate drop"
foreach ($networkCall in @('net::attach_wifi([0; 6]);', 'net::detach_wifi();')) {
    $networkBeforeGateDrop = $driver.Insert(
        $connectionGateDrop,
        "        $networkCall" + [Environment]::NewLine
    )
    Require (-not (Test-NoDirectNetworkInGatedPolls $networkBeforeGateDrop)) "network-before-gate-drop mutation was accepted: $networkCall"
}

$helperBeforeGateDrop = $driver.Insert(
    $connectionGateDrop,
    '        apply_deferred_network_action(DeferredNetworkAction::None);' + [Environment]::NewLine
)
Require (-not (Test-DeferredNetworkAfterGateIife $helperBeforeGateDrop "pub fn poll_connection" "fn next_connection_phase" 'let (changed, network_action) = (|| {')) "deferred helper inside connection gate IIFE mutation was accepted"

$receiveStart = $driver.IndexOf('pub fn receive_ethernet', [StringComparison]::Ordinal)
$receiveLockAt = $driver.IndexOf('    let mut runtime = RX_RING.lock();', $receiveStart, [StringComparison]::Ordinal)
Require ($receiveLockAt -gt $receiveStart) "receive guard-lifetime mutation fixture could not find RX lock"
$receiveWithEarlyDrop = $driver.Insert(
    $receiveLockAt,
    '    drop(_dma_gate);' + [Environment]::NewLine
)
$mutatedReceive = Slice-Between $receiveWithEarlyDrop "pub fn receive_ethernet" "pub fn transmit_ethernet"
Require (-not (Test-GateOrder $mutatedReceive 'RX_RING.lock')) "early DMA gate drop mutation was accepted"

$receiveLatchNeedle = 'if connection_reboot_required() || !data_link_ready()'
$receiveLatchAt = $driver.IndexOf($receiveLatchNeedle, $receiveStart, [StringComparison]::Ordinal)
Require ($receiveLatchAt -gt $receiveStart) "receive latch mutation fixture could not find effective check"
$receiveWithIgnoredLatch = $driver.Remove($receiveLatchAt, $receiveLatchNeedle.Length).Insert(
    $receiveLatchAt,
    'let _ignored = connection_reboot_required(); if !data_link_ready()'
)
$mutatedReceive = Slice-Between $receiveWithIgnoredLatch "pub fn receive_ethernet" "pub fn transmit_ethernet"
Require (-not (Test-GateOrder $mutatedReceive 'RX_RING.lock')) "control-flow-ineffective reboot check mutation was accepted"

$associationStartAt = $driver.IndexOf('fn start_association_inner', [StringComparison]::Ordinal)
$associationGateAt = $driver.IndexOf('MARVELL_DMA_GATE.lock', $associationStartAt, [StringComparison]::Ordinal)
$associationLatchNeedle = 'if connection_reboot_required()'
$associationFreshLatchAt = $driver.IndexOf($associationLatchNeedle, $associationGateAt, [StringComparison]::Ordinal)
Require ($associationFreshLatchAt -gt $associationGateAt) "association latch mutation fixture could not find fresh gated check"
$associationWithoutFreshLatch = $driver.Remove($associationFreshLatchAt, $associationLatchNeedle.Length).Insert($associationFreshLatchAt, 'if false')
Require (-not (Test-AssociationGatedPublication $associationWithoutFreshLatch)) "association publication without fresh gated latch check was accepted"

$scanTriggerStart = $driver.IndexOf('pub fn start_scan_ext_24ghz', [StringComparison]::Ordinal)
$scanTriggerGateAt = $driver.IndexOf($scanGateNeedle, $scanTriggerStart, [StringComparison]::Ordinal)
Require ($scanTriggerGateAt -gt $scanTriggerStart) "scan trigger gate mutation fixture could not find gate"
$scanTriggerWithoutGate = $driver.Remove($scanTriggerGateAt, $scanGateNeedle.Length)
$mutatedScanTrigger = Slice-Between $scanTriggerWithoutGate "pub fn start_scan_ext_24ghz" "pub fn poll_hw_spec"
Require (-not (Test-GateOrder $mutatedScanTrigger 'SCAN.lock')) "missing scan trigger gate mutation was accepted"

$initialPointerStart = $driver.IndexOf('fn publish_data_ring_pointers_while_gated', [StringComparison]::Ordinal)
$sharedPlanAt = $driver.IndexOf('let shared_raw = match plan_shared_rx_tx_pointer', $initialPointerStart, [StringComparison]::Ordinal)
Require ($sharedPlanAt -gt $initialPointerStart) "initial pointer mutation fixture could not find shared plan"
$eventBeforeSharedPlan = $driver.Insert(
    $sharedPlanAt,
    '    write_reg(mmio_base as *mut u8, PCIE_EVT_RD_PTR, event_rdptr);' + [Environment]::NewLine
)
Require (-not (Test-InitialPointerPlanBeforeMmio $eventBeforeSharedPlan)) "invalid shared plan can publish EVENT pointer mutation was accepted"

$totalQuarantineCalls = ([regex]::Matches($driver, 'quarantine_connection_job\(&job\)')).Count
$gatedQuarantineCalls =
    ([regex]::Matches($connectionPoll, 'quarantine_connection_job\(&job\)')).Count +
    ([regex]::Matches($linkLoss, 'quarantine_connection_job\(&job\)')).Count
Require (($totalQuarantineCalls -gt 0) -and ($totalQuarantineCalls -eq $gatedQuarantineCalls)) "quarantine call exists outside gated CONNECTION paths"
$quarantineBody = Slice-Between $driver "fn quarantine_connection_job" "fn clear_connection_secret_dma"
Require-Match $quarantineBody 'CONNECTION_REBOOT_REQUIRED\.store\(true' "quarantine no longer latches reboot-required"
Require (([regex]::Matches($driver, 'CONNECTION_REBOOT_REQUIRED\.store\(true')).Count -eq 1) "reboot latch gained an unreviewed direct store"

function Enter-ModeledDmaGate([hashtable]$State, [string]$Actor) {
    if ($State.Gate -ne '') {
        $State.Sequence += "$Actor.wait"
        return $false
    }
    $State.Gate = $Actor
    $State.Sequence += "$Actor.acquire"
    $true
}

function Exit-ModeledDmaGate([hashtable]$State, [string]$Actor) {
    Require ($State.Gate -eq $Actor) "$Actor released a gate it did not own"
    $State.Gate = ''
    $State.Sequence += "$Actor.release"
}

function Complete-ModeledTx([hashtable]$State) {
    Require ($State.Gate -eq 'tx') "TX actions ran outside DMA gate"
    $State.Sequence += 'tx.check_latch'
    if ($State.Reboot) {
        $State.Sequence += 'tx.stop'
        Exit-ModeledDmaGate $State 'tx'
        return
    }
    $State.PointerWrites += 1
    $State.Sequence += 'tx.pointer'
    $State.Doorbells += 1
    $State.Sequence += 'tx.doorbell'
    Exit-ModeledDmaGate $State 'tx'
}

function Complete-ModeledQuarantine([hashtable]$State) {
    Require (Enter-ModeledDmaGate $State 'quarantine') "quarantine could not acquire released DMA gate"
    $State.Reboot = $true
    $State.Sequence += 'quarantine.latch'
    Exit-ModeledDmaGate $State 'quarantine'
}

function Invoke-DmaGateInterleaving([bool]$TxFirst) {
    $state = @{ Gate = ''; Reboot = $false; PointerWrites = 0; Doorbells = 0; Sequence = @() }
    if ($TxFirst) {
        Require (Enter-ModeledDmaGate $state 'tx') "TX could not acquire initial DMA gate"
        Require (-not (Enter-ModeledDmaGate $state 'quarantine')) "quarantine overlapped TX critical section"
        Complete-ModeledTx $state
        Complete-ModeledQuarantine $state
    }
    else {
        Complete-ModeledQuarantine $state
        Require (Enter-ModeledDmaGate $state 'tx') "TX could not acquire released DMA gate"
        Complete-ModeledTx $state
    }
    $state
}
$txFirst = Invoke-DmaGateInterleaving $true
$quarantineFirst = Invoke-DmaGateInterleaving $false
Require ($txFirst.Reboot -and $txFirst.PointerWrites -eq 1 -and $txFirst.Doorbells -eq 1) "TX-first DMA gate model is not atomic before latch"
Require (($txFirst.Sequence -join ',') -eq 'tx.acquire,quarantine.wait,tx.check_latch,tx.pointer,tx.doorbell,tx.release,quarantine.acquire,quarantine.latch,quarantine.release') "TX-first interleaving did not serialize quarantine after pointer/doorbell"
Require ($quarantineFirst.Reboot -and $quarantineFirst.PointerWrites -eq 0 -and $quarantineFirst.Doorbells -eq 0) "quarantine-first DMA gate model permits post-latch pointer/doorbell"
Require (($quarantineFirst.Sequence -join ',') -eq 'quarantine.acquire,quarantine.latch,quarantine.release,tx.acquire,tx.check_latch,tx.stop,tx.release') "quarantine-first interleaving did not stop TX after latch"

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
