[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$source = Get-Content -Raw -LiteralPath (Join-Path $repoRoot "seed-kernel\src\usb.rs")

function Get-SourceBlock([string]$Start, [string]$End) {
    $first = $source.IndexOf($Start, [StringComparison]::Ordinal)
    $last = $source.IndexOf($End, $first + $Start.Length, [StringComparison]::Ordinal)
    if ($first -lt 0 -or $last -le $first) { throw "source block missing: $Start .. $End" }
    $source.Substring($first, $last - $first)
}
function Assert-Occurrences([string]$Text, [string]$Needle, [int]$Expected) {
    $actual = ([regex]::Matches($Text, [regex]::Escape($Needle))).Count
    if ($actual -ne $Expected) { throw "'$Needle': expected $Expected occurrence(s), found $actual" }
}

$pure = Get-SourceBlock "// USB_COMPOSITE_HID_PURE_BEGIN" "// USB_COMPOSITE_HID_PURE_END"
$control = Get-SourceBlock "let ring_indices =" 'self.set_enum_stage("cfg-ep")'
Assert-Occurrences $control "USB_REQ_SET_CONFIGURATION" 1
Assert-Occurrences $control "HID_REQ_SET_PROTOCOL" 1
Assert-Occurrences $control "HID_REQ_SET_IDLE" 1
$configure = Get-SourceBlock "unsafe fn configure_interrupt_endpoints" "unsafe fn control_in"
Assert-Occurrences $configure "TRB_TYPE_CONFIGURE_ENDPOINT" 1
Assert-Occurrences $configure "clear_input_context();" 1
Assert-Occurrences $configure "queue_keyboard_report" 0
Assert-Occurrences $configure "queue_mouse_report" 0
$command = $configure.IndexOf("TRB_TYPE_CONFIGURE_ENDPOINT", [StringComparison]::Ordinal)
if ($configure.IndexOf("self.keyboard = keyboard", [StringComparison]::Ordinal) -le $command -or
    $configure.IndexOf("self.mouse = pointer", [StringComparison]::Ordinal) -le $command) {
    throw "HID state was published before atomic Configure Endpoint completed"
}
foreach ($disconnect in @(
    (Get-SourceBlock "fn clear_devices_for_root_port" "fn clear_devices_for_hub_port"),
    (Get-SourceBlock "fn clear_devices_for_hub_port" "fn deactivate_hubs_for_root_port")
)) {
    Assert-Occurrences $disconnect "self.keyboard = None" 1
    Assert-Occurrences $disconnect "self.mouse = None" 1
}
Write-Output "composite HID integration source predicates: PASS"

$tempRoot = [IO.Path]::GetFullPath($env:TEMP).TrimEnd([char]'\', [char]'/') + [IO.Path]::DirectorySeparatorChar
$fixtureDir = [IO.Path]::GetFullPath((Join-Path $env:TEMP ("raios-usb-composite-hid-{0}-{1}" -f $PID, [Guid]::NewGuid().ToString("N"))))
if (-not $fixtureDir.StartsWith($tempRoot, [StringComparison]::OrdinalIgnoreCase)) {
    throw "refusing fixture directory outside TEMP: $fixtureDir"
}
New-Item -ItemType Directory -Path $fixtureDir | Out-Null
try {
    $harness = @"
const KEYBOARD_REPORT_LEN: usize = 8;
const MOUSE_REPORT_LEN: usize = 4;
const TABLET_REPORT_LEN: usize = 8;
$pure

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct I { n:u8, alt:u8, class:u8, sub:u8, proto:u8, ep:u8, attr:u8, mps:u16, interval:u8 }
    const fn hid(n:u8, ep:u8, sub:u8, proto:u8)->I {
        I { n,alt:0,class:3,sub,proto,ep,attr:3,mps:8,interval:10 }
    }
    const fn kb(n:u8,ep:u8)->I { hid(n,ep,1,1) }
    const fn mouse(n:u8,ep:u8)->I { hid(n,ep,1,2) }
    const fn generic(n:u8,ep:u8)->I { hid(n,ep,0,0) }
    fn descriptor(items:&[I])->Vec<u8> {
        let len=9+items.len()*16;
        let mut d=vec![9,2,len as u8,(len>>8) as u8,items.len() as u8,1,0,0x80,50];
        for i in items {
            d.extend_from_slice(&[9,4,i.n,i.alt,1,i.class,i.sub,i.proto,0]);
            d.extend_from_slice(&[7,5,i.ep,i.attr,i.mps as u8,(i.mps>>8) as u8,i.interval]);
        }
        d
    }

    #[test]
    fn rapoo_order_independent_and_extras_do_not_displace() {
        let a=descriptor(&[kb(2,0x83),generic(3,0x84),mouse(0,0x81),kb(1,0x82)]);
        let b=descriptor(&[kb(1,0x82),mouse(0,0x81),generic(3,0x84),kb(2,0x83)]);
        let (a,b)=(parse_boot_hid_endpoints(&a,true,true).unwrap(),parse_boot_hid_endpoints(&b,true,true).unwrap());
        assert_eq!(a,b);
        assert_eq!((a.pointer.unwrap().interface_number,a.pointer.unwrap().dci),(0,3));
        assert_eq!((a.keyboard.unwrap().interface_number,a.keyboard.unwrap().dci),(1,5));
        assert_eq!(a.len(),2);
    }

    #[test]
    fn duplicate_roles_choose_lowest_alt_zero_valid_candidate() {
        let mut alt=kb(0,0x81); alt.alt=1;
        let mut invalid=kb(7,0x87); invalid.mps=4;
        let d=descriptor(&[invalid,alt,mouse(5,0x86),mouse(2,0x82),kb(3,0x83)]);
        let got=parse_boot_hid_endpoints(&d,true,true).unwrap();
        assert_eq!(got.keyboard.unwrap().interface_number,3);
        assert_eq!(got.pointer.unwrap().interface_number,2);
    }

    #[test]
    fn generic_fallback_is_single_only_and_boot_wins() {
        let d=descriptor(&[generic(0,0x81)]);
        assert_eq!(parse_boot_hid_endpoints(&d,true,true).unwrap().keyboard.unwrap().kind,HidKind::GenericKeyboard);
        assert_eq!(parse_boot_hid_endpoints(&d,false,true).unwrap().pointer.unwrap().kind,HidKind::Tablet);
        let mut bad=generic(0,0x80); bad.mps=1;
        let d=descriptor(&[bad,kb(1,0x82)]);
        let got=parse_boot_hid_endpoints(&d,true,true).unwrap();
        assert!(got.pointer.is_none()); assert_eq!(got.keyboard.unwrap().kind,HidKind::Keyboard);
        assert!(parse_boot_hid_endpoints(&d,false,true).is_err());
        let d=descriptor(&[generic(0,0x81),generic(1,0x82)]);
        assert_eq!(parse_boot_hid_endpoints(&d,true,true),Err("generic HID fallback requires exactly one interface"));
    }

    #[test]
    fn single_boot_devices_remain_supported() {
        let k=parse_boot_hid_endpoints(&descriptor(&[kb(0,0x81)]),true,true).unwrap();
        assert!(k.keyboard.is_some() && k.pointer.is_none());
        let m=parse_boot_hid_endpoints(&descriptor(&[mouse(0,0x81)]),true,true).unwrap();
        assert!(m.keyboard.is_none() && m.pointer.is_some());
    }

    #[test]
    fn malformed_and_truncated_fail_closed() {
        let mut d=descriptor(&[kb(0,0x81)]); d.pop();
        assert_eq!(parse_boot_hid_endpoints(&d,true,true),Err("configuration descriptor truncated"));
        let mut d=descriptor(&[kb(0,0x81)]); d[9]=1;
        assert_eq!(parse_boot_hid_endpoints(&d,true,true),Err("configuration descriptor entry malformed"));
    }

    #[test]
    fn invalid_endpoints_never_reach_ring_plan() {
        let mut i=kb(0,0x01);
        assert!(parse_boot_hid_endpoints(&descriptor(&[i]),true,true).is_err());
        i.ep=0x81; i.attr=2;
        assert!(parse_boot_hid_endpoints(&descriptor(&[i]),true,true).is_err());
        assert_eq!(parse_boot_hid_endpoints(&descriptor(&[kb(0,0x80)]),true,true),Err("HID interrupt endpoint uses EP0"));
        let mut i=kb(0,0x81); i.mps=4;
        assert_eq!(parse_boot_hid_endpoints(&descriptor(&[i]),true,true),Err("HID interrupt endpoint packet is smaller than its boot report"));
    }

    #[test]
    fn duplicate_cross_role_dci_is_rejected() {
        let d=descriptor(&[mouse(0,0x81),kb(1,0x81)]);
        assert_eq!(parse_boot_hid_endpoints(&d,true,true),Err("HID interrupt endpoints share a DCI"));
    }

    #[test]
    fn composite_rings_and_transfer_routing_are_separate() {
        let ep=parse_boot_hid_endpoints(&descriptor(&[mouse(0,0x81),kb(1,0x82)]),true,true).unwrap();
        let mut next=1;
        let rings=plan_hid_ring_indices(ep,0,&mut next,16).unwrap();
        assert_eq!((rings.pointer,rings.keyboard,next),(Some(0),Some(1),2));
        let slot=9; let k=ep.keyboard.unwrap(); let p=ep.pointer.unwrap();
        assert_eq!(hid_transfer_target(slot,k.dci,Some((slot,k.dci)),Some((slot,p.dci))),Some(HidTransferTarget::Keyboard));
        assert_eq!(hid_transfer_target(slot,p.dci,Some((slot,k.dci)),Some((slot,p.dci))),Some(HidTransferTarget::Pointer));
        assert_eq!(hid_transfer_target(slot+1,p.dci,Some((slot,k.dci)),Some((slot,p.dci))),None);
        let mut exhausted=16;
        assert_eq!(plan_hid_ring_indices(ep,0,&mut exhausted,16),Err("composite HID interrupt ring unavailable"));
    }
}
"@
    $harnessPath = Join-Path $fixtureDir "usb-composite-hid-fixture.rs"
    $binaryPath = Join-Path $fixtureDir "usb-composite-hid-fixture.exe"
    [IO.File]::WriteAllText($harnessPath, $harness, [Text.UTF8Encoding]::new($false))
    & rustup run nightly-2024-10-15 rustc --edition=2021 --test $harnessPath -o $binaryPath
    if ($LASTEXITCODE -ne 0) { throw "fixture compile failed: $LASTEXITCODE" }
    & $binaryPath
    if ($LASTEXITCODE -ne 0) { throw "fixture failed: $LASTEXITCODE" }
}
finally {
    if (Test-Path -LiteralPath $fixtureDir) {
        $resolved = [IO.Path]::GetFullPath($fixtureDir)
        if (-not $resolved.StartsWith($tempRoot, [StringComparison]::OrdinalIgnoreCase)) {
            throw "refusing cleanup outside TEMP: $resolved"
        }
        Remove-Item -LiteralPath $resolved -Recurse -Force
    }
}
