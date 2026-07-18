//! Pure driver hardware-capability decisions for ADR 0010 Option A.
//!
//! This module grants nothing by itself. It is the reference-monitor predicate
//! the future trusted hardware "hand" will consult before touching MMIO or DMA.

pub const DRIVER_HW_CAPABILITY_TRUST_TIER: &str = "dev_key_not_owner_sealed";
pub const DRIVER_HW_CAPABILITY_MAX_MMIO_WINDOWS: usize = 4;
pub const DRIVER_HW_CAPABILITY_MAX_DMA_REGION_IDS: usize = 4;
pub const DRIVER_HW_CAPABILITY_MANIFEST_MAGIC: &[u8; 8] = b"RAIOSDH0";
pub const DRIVER_HW_CAPABILITY_MANIFEST_VERSION: u16 = 0;
pub const DRIVER_HW_CAPABILITY_MANIFEST_LEN: usize = 96;

const SELECTOR_HAS_BDF: u8 = 1 << 0;
const SELECTOR_HAS_SUBSYSTEM: u8 = 1 << 1;
const SELECTOR_KNOWN_FLAGS: u8 = SELECTOR_HAS_BDF | SELECTOR_HAS_SUBSYSTEM;
const TRUST_TIER_DEV_KEY_NOT_OWNER_SEALED: u8 = 1;
const WINDOW_ENTRY_LEN: usize = 12;
const WINDOW_TABLE_OFFSET: usize = 32;
const DMA_REGION_TABLE_OFFSET: usize =
    WINDOW_TABLE_OFFSET + DRIVER_HW_CAPABILITY_MAX_MMIO_WINDOWS * WINDOW_ENTRY_LEN;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PciBdf {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PciSubsystemId {
    pub vendor_id: u16,
    pub device_id: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DriverDeviceSelector {
    pub vendor_id: u16,
    pub device_id: u16,
    pub bdf: Option<PciBdf>,
    pub subsystem: Option<PciSubsystemId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MmioWindow {
    pub bar_index: u8,
    pub offset_start: u32,
    pub offset_end: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DriverHwCapability {
    pub device: DriverDeviceSelector,
    pub mmio_windows: [MmioWindow; DRIVER_HW_CAPABILITY_MAX_MMIO_WINDOWS],
    pub mmio_window_count: usize,
    pub max_dma_bytes: u32,
    pub dma_region_ids: [u32; DRIVER_HW_CAPABILITY_MAX_DMA_REGION_IDS],
    pub dma_region_id_count: usize,
    pub dma_confined: bool,
    pub trust_tier: &'static str,
}

impl DriverHwCapability {
    pub const fn empty() -> Self {
        Self {
            device: DriverDeviceSelector {
                vendor_id: 0,
                device_id: 0,
                bdf: None,
                subsystem: None,
            },
            mmio_windows: [EMPTY_MMIO_WINDOW; DRIVER_HW_CAPABILITY_MAX_MMIO_WINDOWS],
            mmio_window_count: 0,
            max_dma_bytes: 0,
            dma_region_ids: [0; DRIVER_HW_CAPABILITY_MAX_DMA_REGION_IDS],
            dma_region_id_count: 0,
            dma_confined: false,
            trust_tier: DRIVER_HW_CAPABILITY_TRUST_TIER,
        }
    }
}

const EMPTY_MMIO_WINDOW: MmioWindow = MmioWindow {
    bar_index: 0,
    offset_start: 0,
    offset_end: 0,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Decision {
    pub allowed: bool,
    pub reason: &'static str,
    pub dma_unconfined_warning: bool,
}

pub fn check_mmio(
    grant: &DriverHwCapability,
    bar_index: u8,
    offset: u32,
    is_write: bool,
) -> Decision {
    if let Err(reason) = validate_capability(grant) {
        return denied(reason);
    }
    if grant.mmio_window_count == 0 {
        return denied("mmio_window_not_granted");
    }

    let mut saw_bar = false;
    let mut idx = 0usize;
    while idx < grant.mmio_window_count {
        let window = grant.mmio_windows[idx];
        if window.bar_index == bar_index {
            saw_bar = true;
            if offset >= window.offset_start && offset <= window.offset_end {
                return Decision {
                    allowed: true,
                    reason: if is_write {
                        "mmio_write_allowed"
                    } else {
                        "mmio_read_allowed"
                    },
                    dma_unconfined_warning: false,
                };
            }
        }
        idx += 1;
    }

    if saw_bar {
        denied("mmio_offset_out_of_grant")
    } else {
        denied("mmio_bar_not_granted")
    }
}

pub fn check_dma(grant: &DriverHwCapability, target_len: u32) -> Decision {
    if let Err(reason) = validate_capability(grant) {
        return denied(reason);
    }
    if target_len == 0 {
        return denied("dma_len_zero");
    }
    if grant.max_dma_bytes == 0 {
        return denied("dma_not_granted");
    }
    if target_len > grant.max_dma_bytes {
        return denied("dma_len_exceeds_grant");
    }

    if grant.dma_confined {
        Decision {
            allowed: true,
            reason: "dma_allowed_iommu_confined",
            dma_unconfined_warning: false,
        }
    } else {
        Decision {
            allowed: true,
            reason: "dma_allowed_owner_trusted_not_iommu_confined",
            dma_unconfined_warning: true,
        }
    }
}

pub fn manifest_within_grant(
    requested: &DriverHwCapability,
    granted: &DriverHwCapability,
) -> Decision {
    if let Err(reason) = validate_capability(requested) {
        return denied(reason);
    }
    if let Err(reason) = validate_capability(granted) {
        return denied(reason);
    }
    if requested.device != granted.device {
        return denied("device_selector_not_granted");
    }
    if requested.trust_tier != DRIVER_HW_CAPABILITY_TRUST_TIER
        || granted.trust_tier != DRIVER_HW_CAPABILITY_TRUST_TIER
    {
        return denied("trust_tier_not_dev_key_not_owner_sealed");
    }

    let mut req_idx = 0usize;
    while req_idx < requested.mmio_window_count {
        if !window_within_grant(requested.mmio_windows[req_idx], granted) {
            return denied("mmio_window_exceeds_grant");
        }
        req_idx += 1;
    }

    if requested.max_dma_bytes > granted.max_dma_bytes {
        return denied("dma_budget_exceeds_grant");
    }
    if requested.dma_confined != granted.dma_confined {
        return denied("dma_confinement_label_mismatch");
    }
    if requested.max_dma_bytes > 0
        && requested.dma_region_id_count == 0
        && granted.dma_region_id_count > 0
    {
        return denied("dma_region_exceeds_grant");
    }

    let mut dma_idx = 0usize;
    while dma_idx < requested.dma_region_id_count {
        if !dma_region_granted(requested.dma_region_ids[dma_idx], granted) {
            return denied("dma_region_exceeds_grant");
        }
        dma_idx += 1;
    }

    Decision {
        allowed: true,
        reason: "manifest_within_owner_grant",
        dma_unconfined_warning: requested.max_dma_bytes > 0 && !requested.dma_confined,
    }
}

pub fn decode_driver_hw_capability_manifest(
    bytes: &[u8],
) -> Result<DriverHwCapability, &'static str> {
    if bytes.len() < 12 {
        return Err("manifest_header_truncated");
    }
    if &bytes[0..8] != DRIVER_HW_CAPABILITY_MANIFEST_MAGIC {
        return Err("bad_magic");
    }
    if read_u16(bytes, 8) != DRIVER_HW_CAPABILITY_MANIFEST_VERSION {
        return Err("version_mismatch");
    }
    let record_len = read_u16(bytes, 10) as usize;
    if record_len != DRIVER_HW_CAPABILITY_MANIFEST_LEN {
        return Err("manifest_len_mismatch");
    }
    if bytes.len() < record_len {
        return Err("manifest_truncated");
    }
    if bytes.len() != record_len {
        return Err("manifest_len_mismatch");
    }

    let selector_flags = bytes[16];
    if selector_flags & !SELECTOR_KNOWN_FLAGS != 0 {
        return Err("selector_flags_out_of_scope");
    }
    let bdf = if selector_flags & SELECTOR_HAS_BDF != 0 {
        let bdf = PciBdf {
            bus: bytes[17],
            device: bytes[18],
            function: bytes[19],
        };
        if !valid_bdf(bdf) {
            return Err("bdf_out_of_scope");
        }
        Some(bdf)
    } else {
        if bytes[17] != 0 || bytes[18] != 0 || bytes[19] != 0 {
            return Err("selector_padding_nonzero");
        }
        None
    };
    let subsystem = if selector_flags & SELECTOR_HAS_SUBSYSTEM != 0 {
        Some(PciSubsystemId {
            vendor_id: read_u16(bytes, 20),
            device_id: read_u16(bytes, 22),
        })
    } else {
        if read_u16(bytes, 20) != 0 || read_u16(bytes, 22) != 0 {
            return Err("selector_padding_nonzero");
        }
        None
    };

    let mmio_window_count = bytes[24] as usize;
    if mmio_window_count > DRIVER_HW_CAPABILITY_MAX_MMIO_WINDOWS {
        return Err("mmio_window_count_exceeds_max");
    }
    let dma_region_id_count = bytes[25] as usize;
    if dma_region_id_count > DRIVER_HW_CAPABILITY_MAX_DMA_REGION_IDS {
        return Err("dma_region_count_exceeds_max");
    }
    let dma_confined = match bytes[26] {
        0 => false,
        1 => true,
        _ => return Err("dma_confined_invalid"),
    };
    if bytes[27] != TRUST_TIER_DEV_KEY_NOT_OWNER_SEALED {
        return Err("trust_tier_out_of_scope");
    }

    let mut mmio_windows = [EMPTY_MMIO_WINDOW; DRIVER_HW_CAPABILITY_MAX_MMIO_WINDOWS];
    let mut idx = 0usize;
    while idx < DRIVER_HW_CAPABILITY_MAX_MMIO_WINDOWS {
        let offset = WINDOW_TABLE_OFFSET + idx * WINDOW_ENTRY_LEN;
        let entry = &bytes[offset..offset + WINDOW_ENTRY_LEN];
        if idx >= mmio_window_count {
            if !all_zero(entry) {
                return Err("unused_mmio_window_nonzero");
            }
        } else {
            if entry[1] != 0 || entry[2] != 0 || entry[3] != 0 {
                return Err("mmio_window_reserved_nonzero");
            }
            let window = MmioWindow {
                bar_index: entry[0],
                offset_start: read_u32(entry, 4),
                offset_end: read_u32(entry, 8),
            };
            if window.offset_end < window.offset_start {
                return Err("mmio_window_invalid");
            }
            mmio_windows[idx] = window;
        }
        idx += 1;
    }

    let mut dma_region_ids = [0; DRIVER_HW_CAPABILITY_MAX_DMA_REGION_IDS];
    let mut dma_idx = 0usize;
    while dma_idx < DRIVER_HW_CAPABILITY_MAX_DMA_REGION_IDS {
        let offset = DMA_REGION_TABLE_OFFSET + dma_idx * 4;
        let id = read_u32(bytes, offset);
        if dma_idx >= dma_region_id_count {
            if id != 0 {
                return Err("unused_dma_region_nonzero");
            }
        } else {
            dma_region_ids[dma_idx] = id;
        }
        dma_idx += 1;
    }

    let capability = DriverHwCapability {
        device: DriverDeviceSelector {
            vendor_id: read_u16(bytes, 12),
            device_id: read_u16(bytes, 14),
            bdf,
            subsystem,
        },
        mmio_windows,
        mmio_window_count,
        max_dma_bytes: read_u32(bytes, 28),
        dma_region_ids,
        dma_region_id_count,
        dma_confined,
        trust_tier: DRIVER_HW_CAPABILITY_TRUST_TIER,
    };
    validate_capability(&capability)?;
    Ok(capability)
}

fn validate_capability(capability: &DriverHwCapability) -> Result<(), &'static str> {
    if capability.trust_tier != DRIVER_HW_CAPABILITY_TRUST_TIER {
        return Err("trust_tier_not_dev_key_not_owner_sealed");
    }
    if let Some(bdf) = capability.device.bdf {
        if !valid_bdf(bdf) {
            return Err("bdf_out_of_scope");
        }
    }
    if capability.mmio_window_count > DRIVER_HW_CAPABILITY_MAX_MMIO_WINDOWS {
        return Err("mmio_window_count_exceeds_max");
    }
    if capability.dma_region_id_count > DRIVER_HW_CAPABILITY_MAX_DMA_REGION_IDS {
        return Err("dma_region_count_exceeds_max");
    }

    let mut idx = 0usize;
    while idx < capability.mmio_window_count {
        let window = capability.mmio_windows[idx];
        if window.offset_end < window.offset_start {
            return Err("mmio_window_invalid");
        }
        idx += 1;
    }

    let mut left = 0usize;
    while left < capability.dma_region_id_count {
        let mut right = left + 1;
        while right < capability.dma_region_id_count {
            if capability.dma_region_ids[left] == capability.dma_region_ids[right] {
                return Err("duplicate_dma_region");
            }
            right += 1;
        }
        left += 1;
    }
    Ok(())
}

fn window_within_grant(requested: MmioWindow, granted: &DriverHwCapability) -> bool {
    let mut grant_idx = 0usize;
    while grant_idx < granted.mmio_window_count {
        let window = granted.mmio_windows[grant_idx];
        if requested.bar_index == window.bar_index
            && requested.offset_start >= window.offset_start
            && requested.offset_end <= window.offset_end
        {
            return true;
        }
        grant_idx += 1;
    }
    false
}

fn dma_region_granted(requested_id: u32, granted: &DriverHwCapability) -> bool {
    if granted.dma_region_id_count == 0 {
        return true;
    }
    let mut idx = 0usize;
    while idx < granted.dma_region_id_count {
        if granted.dma_region_ids[idx] == requested_id {
            return true;
        }
        idx += 1;
    }
    false
}

const fn valid_bdf(bdf: PciBdf) -> bool {
    bdf.device < 32 && bdf.function < 8
}

const fn denied(reason: &'static str) -> Decision {
    Decision {
        allowed: false,
        reason,
        dma_unconfined_warning: false,
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    (bytes[offset] as u16) | ((bytes[offset + 1] as u16) << 8)
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    (bytes[offset] as u32)
        | ((bytes[offset + 1] as u32) << 8)
        | ((bytes[offset + 2] as u32) << 16)
        | ((bytes[offset + 3] as u32) << 24)
}

fn all_zero(bytes: &[u8]) -> bool {
    let mut idx = 0usize;
    while idx < bytes.len() {
        if bytes[idx] != 0 {
            return false;
        }
        idx += 1;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grant() -> DriverHwCapability {
        DriverHwCapability {
            device: DriverDeviceSelector {
                vendor_id: 0x11ab,
                device_id: 0x2b38,
                bdf: Some(PciBdf {
                    bus: 1,
                    device: 2,
                    function: 0,
                }),
                subsystem: Some(PciSubsystemId {
                    vendor_id: 0x1a56,
                    device_id: 0x0001,
                }),
            },
            mmio_windows: [
                MmioWindow {
                    bar_index: 2,
                    offset_start: 0x100,
                    offset_end: 0x1ff,
                },
                MmioWindow {
                    bar_index: 2,
                    offset_start: 0x300,
                    offset_end: 0x31f,
                },
                EMPTY_MMIO_WINDOW,
                EMPTY_MMIO_WINDOW,
            ],
            mmio_window_count: 2,
            max_dma_bytes: 4096,
            dma_region_ids: [7, 9, 0, 0],
            dma_region_id_count: 2,
            dma_confined: false,
            trust_tier: DRIVER_HW_CAPABILITY_TRUST_TIER,
        }
    }

    #[test]
    fn in_range_mmio_read_and_write_are_allowed() {
        assert_eq!(
            check_mmio(&grant(), 2, 0x100, false),
            Decision {
                allowed: true,
                reason: "mmio_read_allowed",
                dma_unconfined_warning: false,
            }
        );
        assert_eq!(
            check_mmio(&grant(), 2, 0x1ff, true),
            Decision {
                allowed: true,
                reason: "mmio_write_allowed",
                dma_unconfined_warning: false,
            }
        );
    }

    #[test]
    fn out_of_range_and_other_bar_mmio_are_denied() {
        assert_eq!(
            check_mmio(&grant(), 2, 0x200, false),
            Decision {
                allowed: false,
                reason: "mmio_offset_out_of_grant",
                dma_unconfined_warning: false,
            }
        );
        assert_eq!(
            check_mmio(&grant(), 0, 0x100, true),
            Decision {
                allowed: false,
                reason: "mmio_bar_not_granted",
                dma_unconfined_warning: false,
            }
        );
    }

    #[test]
    fn dma_budget_allows_within_and_denies_over_limit() {
        let allowed = check_dma(&grant(), 4096);
        assert!(allowed.allowed);
        assert_eq!(
            allowed.reason,
            "dma_allowed_owner_trusted_not_iommu_confined"
        );

        assert_eq!(
            check_dma(&grant(), 4097),
            Decision {
                allowed: false,
                reason: "dma_len_exceeds_grant",
                dma_unconfined_warning: false,
            }
        );
    }

    #[test]
    fn allowed_unconfined_dma_surfaces_warning() {
        let decision = check_dma(&grant(), 1);

        assert!(decision.allowed);
        assert!(decision.dma_unconfined_warning);
        assert_eq!(
            decision.reason,
            "dma_allowed_owner_trusted_not_iommu_confined"
        );
    }

    #[test]
    fn manifest_subset_allows_smaller_request_and_warns_for_unconfined_dma() {
        let mut requested = grant();
        requested.mmio_windows[0] = MmioWindow {
            bar_index: 2,
            offset_start: 0x120,
            offset_end: 0x140,
        };
        requested.mmio_window_count = 1;
        requested.max_dma_bytes = 1024;
        requested.dma_region_ids = [7, 0, 0, 0];
        requested.dma_region_id_count = 1;

        assert_eq!(
            manifest_within_grant(&requested, &grant()),
            Decision {
                allowed: true,
                reason: "manifest_within_owner_grant",
                dma_unconfined_warning: true,
            }
        );
    }

    #[test]
    fn requested_superset_of_grant_is_denied() {
        let mut requested = grant();
        requested.mmio_windows[0].offset_end = 0x200;
        assert_eq!(
            manifest_within_grant(&requested, &grant()).reason,
            "mmio_window_exceeds_grant"
        );

        let mut requested = grant();
        requested.max_dma_bytes = 4097;
        assert_eq!(
            manifest_within_grant(&requested, &grant()).reason,
            "dma_budget_exceeds_grant"
        );

        let mut requested = grant();
        requested.dma_region_ids = [8, 0, 0, 0];
        requested.dma_region_id_count = 1;
        assert_eq!(
            manifest_within_grant(&requested, &grant()).reason,
            "dma_region_exceeds_grant"
        );
    }

    #[test]
    fn malformed_manifest_fails_closed() {
        let manifest = valid_manifest();
        let mut bad_magic = manifest;
        bad_magic[0] = b'X';
        assert_eq!(
            decode_driver_hw_capability_manifest(&bad_magic),
            Err("bad_magic")
        );
        assert_eq!(
            decode_driver_hw_capability_manifest(&manifest[..20]),
            Err("manifest_truncated")
        );

        let mut bad_window = manifest;
        write_u32(&mut bad_window, WINDOW_TABLE_OFFSET + 4, 0x200);
        write_u32(&mut bad_window, WINDOW_TABLE_OFFSET + 8, 0x100);
        assert_eq!(
            decode_driver_hw_capability_manifest(&bad_window),
            Err("mmio_window_invalid")
        );
    }

    #[test]
    fn valid_manifest_decodes_and_subset_checks_against_grant() {
        let decoded = decode_driver_hw_capability_manifest(&valid_manifest()).unwrap();

        assert_eq!(decoded.device.vendor_id, 0x11ab);
        assert_eq!(decoded.device.device_id, 0x2b38);
        assert_eq!(decoded.mmio_window_count, 1);
        assert_eq!(decoded.max_dma_bytes, 1024);
        assert_eq!(decoded.dma_region_ids[0], 7);
        assert!(manifest_within_grant(&decoded, &grant()).allowed);
    }

    fn valid_manifest() -> [u8; DRIVER_HW_CAPABILITY_MANIFEST_LEN] {
        let mut out = [0u8; DRIVER_HW_CAPABILITY_MANIFEST_LEN];
        out[..8].copy_from_slice(DRIVER_HW_CAPABILITY_MANIFEST_MAGIC);
        write_u16(&mut out, 8, DRIVER_HW_CAPABILITY_MANIFEST_VERSION);
        write_u16(&mut out, 10, DRIVER_HW_CAPABILITY_MANIFEST_LEN as u16);
        write_u16(&mut out, 12, 0x11ab);
        write_u16(&mut out, 14, 0x2b38);
        out[16] = SELECTOR_HAS_BDF | SELECTOR_HAS_SUBSYSTEM;
        out[17] = 1;
        out[18] = 2;
        out[19] = 0;
        write_u16(&mut out, 20, 0x1a56);
        write_u16(&mut out, 22, 0x0001);
        out[24] = 1;
        out[25] = 1;
        out[26] = 0;
        out[27] = TRUST_TIER_DEV_KEY_NOT_OWNER_SEALED;
        write_u32(&mut out, 28, 1024);
        out[WINDOW_TABLE_OFFSET] = 2;
        write_u32(&mut out, WINDOW_TABLE_OFFSET + 4, 0x120);
        write_u32(&mut out, WINDOW_TABLE_OFFSET + 8, 0x140);
        write_u32(&mut out, DMA_REGION_TABLE_OFFSET, 7);
        out
    }

    fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset] = value as u8;
        bytes[offset + 1] = (value >> 8) as u8;
    }

    fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset] = value as u8;
        bytes[offset + 1] = (value >> 8) as u8;
        bytes[offset + 2] = (value >> 16) as u8;
        bytes[offset + 3] = (value >> 24) as u8;
    }
}
