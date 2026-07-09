use sha2::{Digest, Sha256};
use spin::Mutex;

use crate::{entropy, iommu_vtd, serial};

pub const RAM_CANDIDATE_ID: &str = "owner_key.ram_candidate.current_boot";
pub const RAM_CANDIDATE_HANDLE: &str = "owner_key.handle.current_boot.ram0";
pub const RAM_CANDIDATE_ALGORITHM: &str = "ram_32_byte_entropy_seed_sha256_fingerprint";
pub const RAM_CANDIDATE_FINGERPRINT_DOMAIN: &[u8] = b"raios.owner_key.ram_candidate.v0";
pub const HARDWARE_BINDING_PROBE_SOURCE: &str = "acpi_tpm2_table";
const TPM2_ACPI_SIGNATURE: &[u8; 4] = b"TPM2";
const TPM2_ACPI_ABSENT_REASON: &str = "TPM2 ACPI table missing";
const TPM2_ACPI_PRESENT_STATUS: &str = "tpm2_acpi_present_seal_not_verified";
const TPM2_ACPI_ABSENT_STATUS: &str = "tpm2_acpi_absent";
const ACPI_ROOT_INVALID_STATUS: &str = "acpi_root_invalid";
const ACPI_RSDP_MISSING_STATUS: &str = "acpi_rsdp_missing";
const TPM2_ACPI_HEADER_LEN: usize = 36;
const TPM2_PLATFORM_CLASS_OFFSET: usize = TPM2_ACPI_HEADER_LEN;
const TPM2_CONTROL_AREA_OFFSET: usize = TPM2_ACPI_HEADER_LEN + 4;
const TPM2_START_METHOD_OFFSET: usize = TPM2_ACPI_HEADER_LEN + 12;
const TPM2_COMMON_INTERFACE_LEN: usize = TPM2_ACPI_HEADER_LEN + 16;
const TPM2_INTERFACE_KIND_NONE: &str = "none";
const TPM2_INTERFACE_KIND_NOT_SET: &str = "not_set";
const TPM2_INTERFACE_KIND_ACPI_START_METHOD: &str = "acpi_start_method";
const TPM2_INTERFACE_KIND_TIS_MMIO_CANCEL: &str = "tis_mmio_cancel";
const TPM2_INTERFACE_KIND_CRB: &str = "crb";
const TPM2_INTERFACE_KIND_CRB_ACPI_START: &str = "crb_acpi_start";
const TPM2_INTERFACE_KIND_CRB_ARM_SMC_HVC: &str = "crb_arm_smc_hvc";
const TPM2_INTERFACE_KIND_FIFO_I2C: &str = "fifo_i2c";
const TPM2_INTERFACE_KIND_CRB_AMD_MAILBOX: &str = "crb_amd_mailbox";
const TPM2_INTERFACE_KIND_FUTURE_MMIO: &str = "future_mmio_reserved";
const TPM2_INTERFACE_KIND_CRB_ARM_FFA: &str = "crb_arm_ffa";
const TPM2_INTERFACE_KIND_VENDOR_RESERVED: &str = "vendor_or_legacy_reserved";
const TPM2_INTERFACE_KIND_FUTURE_RESERVED: &str = "future_reserved";
const TPM2_INTERFACE_STATUS_NOT_PROBED: &str = "not_probed";
const TPM2_INTERFACE_STATUS_TABLE_TOO_SHORT: &str = "tpm2_acpi_table_too_short";
const TPM2_INTERFACE_STATUS_CRB_DETAILS_PARSED: &str =
    "tpm2_crb_interface_details_parsed_status_read_not_attempted";
const TPM2_INTERFACE_STATUS_TIS_DETAILS_PARSED: &str =
    "tpm2_tis_interface_details_parsed_status_read_not_attempted";
const TPM2_INTERFACE_STATUS_TIS_FIXED_BASE_NOT_MAPPED: &str = "tpm2_tis_fixed_base_not_mapped";
const TPM2_INTERFACE_STATUS_CONTROL_AREA_MISSING: &str = "tpm2_control_area_missing";
const TPM2_INTERFACE_STATUS_START_METHOD_UNSUPPORTED: &str = "tpm2_start_method_not_supported_yet";
const TPM2_INTERFACE_REASON_TABLE_TOO_SHORT: &str =
    "TPM2 ACPI table shorter than common interface fields";
const TPM2_INTERFACE_REASON_CRB_PARSED: &str =
    "TPM2 CRB control area discovered; MMIO status read awaits read-only register slice";
const TPM2_INTERFACE_REASON_TIS_PARSED: &str =
    "TPM2 TIS/FIFO base discovered; MMIO status read awaits read-only register slice";
const TPM2_INTERFACE_REASON_TIS_FIXED_BASE: &str =
    "TPM2 TIS/FIFO fixed base not mapped by this slice";
const TPM2_INTERFACE_REASON_CONTROL_AREA_MISSING: &str =
    "TPM2 start method requires a nonzero control area for this probe";
const TPM2_INTERFACE_REASON_START_METHOD_UNSUPPORTED: &str =
    "TPM2 start method parsed but not supported by this read-only probe";
const SECRET_LEN: usize = 32;

static OWNER_KEY_STATE: Mutex<OwnerKeyState> = Mutex::new(OwnerKeyState::new());

#[derive(Clone, Copy)]
pub struct OwnerKeySnapshot {
    pub generated: bool,
    pub handle: Option<&'static str>,
    pub fingerprint: Option<[u8; 32]>,
    pub secret_len: usize,
    pub hardware_binding: HardwareBindingSnapshot,
}

#[derive(Clone, Copy)]
pub struct HardwareBindingSnapshot {
    pub probe_performed: bool,
    pub acpi_rsdp_present: bool,
    pub acpi_root_table_valid: bool,
    pub tpm2_acpi_table_present: bool,
    pub tpm2_acpi_table_phys: u64,
    pub tpm2_acpi_table_length: u32,
    pub tpm2_acpi_table_revision: u8,
    pub tpm2_table_details_valid: bool,
    pub tpm2_platform_class: u16,
    pub tpm2_control_area: u64,
    pub tpm2_start_method: u32,
    pub tpm2_interface_kind: &'static str,
    pub tpm2_interface_status_probe_performed: bool,
    pub tpm2_interface_status: &'static str,
    pub tpm2_interface_status_reason: &'static str,
    pub status: &'static str,
    pub reason: &'static str,
}

struct OwnerKeyState {
    generated: bool,
    secret: [u8; SECRET_LEN],
    fingerprint: [u8; 32],
    hardware_binding: HardwareBindingSnapshot,
}

impl OwnerKeyState {
    const fn new() -> Self {
        Self {
            generated: false,
            secret: [0; SECRET_LEN],
            fingerprint: [0; 32],
            hardware_binding: HardwareBindingSnapshot::not_probed(),
        }
    }

    fn snapshot(&self) -> OwnerKeySnapshot {
        OwnerKeySnapshot {
            generated: self.generated,
            handle: if self.generated {
                Some(RAM_CANDIDATE_HANDLE)
            } else {
                None
            },
            fingerprint: if self.generated {
                Some(self.fingerprint)
            } else {
                None
            },
            secret_len: SECRET_LEN,
            hardware_binding: self.hardware_binding,
        }
    }
}

impl HardwareBindingSnapshot {
    const fn not_probed() -> Self {
        Self {
            probe_performed: false,
            acpi_rsdp_present: false,
            acpi_root_table_valid: false,
            tpm2_acpi_table_present: false,
            tpm2_acpi_table_phys: 0,
            tpm2_acpi_table_length: 0,
            tpm2_acpi_table_revision: 0,
            tpm2_table_details_valid: false,
            tpm2_platform_class: 0,
            tpm2_control_area: 0,
            tpm2_start_method: 0,
            tpm2_interface_kind: TPM2_INTERFACE_KIND_NONE,
            tpm2_interface_status_probe_performed: false,
            tpm2_interface_status: TPM2_INTERFACE_STATUS_NOT_PROBED,
            tpm2_interface_status_reason: "not_probed",
            status: "not_probed",
            reason: "not_probed",
        }
    }
}

pub fn ensure_hardware_binding_probe() -> HardwareBindingSnapshot {
    {
        let state = OWNER_KEY_STATE.lock();
        if state.hardware_binding.probe_performed {
            return state.hardware_binding;
        }
    }

    let probe = iommu_vtd::probe_acpi_table(TPM2_ACPI_SIGNATURE, TPM2_ACPI_ABSENT_REASON);
    let snapshot = hardware_binding_from_acpi_probe(probe);
    let mut state = OWNER_KEY_STATE.lock();
    if !state.hardware_binding.probe_performed {
        state.hardware_binding = snapshot;
        serial::write_fmt(format_args!(
            "owner-key: hardware binding probe status={} reason={}\r\n",
            snapshot.status, snapshot.reason
        ));
    }
    state.hardware_binding
}

pub fn ensure_current_boot_candidate() -> OwnerKeySnapshot {
    {
        let state = OWNER_KEY_STATE.lock();
        if state.generated {
            return state.snapshot();
        }
    }

    if !entropy::is_ready() {
        return snapshot();
    }

    let mut secret = [0u8; SECRET_LEN];
    entropy::take(&mut secret);
    let fingerprint = fingerprint_secret(&secret);

    let mut state = OWNER_KEY_STATE.lock();
    if !state.generated {
        state.secret.copy_from_slice(&secret);
        state.fingerprint = fingerprint;
        state.generated = true;
        serial::write_line("owner-key: RAM current_boot candidate generated");
    }
    for byte in secret.iter_mut() {
        *byte = 0;
    }
    state.snapshot()
}

pub fn snapshot() -> OwnerKeySnapshot {
    OWNER_KEY_STATE.lock().snapshot()
}

fn fingerprint_secret(secret: &[u8; SECRET_LEN]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(RAM_CANDIDATE_FINGERPRINT_DOMAIN);
    hash.update(secret);
    let digest = hash.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

fn hardware_binding_from_acpi_probe(probe: iommu_vtd::AcpiTableProbe) -> HardwareBindingSnapshot {
    let status = if probe.table_present {
        TPM2_ACPI_PRESENT_STATUS
    } else if probe.root_table_valid {
        TPM2_ACPI_ABSENT_STATUS
    } else if probe.rsdp_present {
        ACPI_ROOT_INVALID_STATUS
    } else {
        ACPI_RSDP_MISSING_STATUS
    };
    let details = tpm2_acpi_details_from_probe(probe);
    HardwareBindingSnapshot {
        probe_performed: probe.probe_performed,
        acpi_rsdp_present: probe.rsdp_present,
        acpi_root_table_valid: probe.root_table_valid,
        tpm2_acpi_table_present: probe.table_present,
        tpm2_acpi_table_phys: probe.table_phys,
        tpm2_acpi_table_length: probe.table_length,
        tpm2_acpi_table_revision: probe.table_revision,
        tpm2_table_details_valid: details.valid,
        tpm2_platform_class: details.platform_class,
        tpm2_control_area: details.control_area,
        tpm2_start_method: details.start_method,
        tpm2_interface_kind: details.interface_kind,
        tpm2_interface_status_probe_performed: details.status_probe_performed,
        tpm2_interface_status: details.status,
        tpm2_interface_status_reason: details.reason,
        status,
        reason: probe.reason,
    }
}

#[derive(Clone, Copy)]
struct Tpm2AcpiDetails {
    valid: bool,
    platform_class: u16,
    control_area: u64,
    start_method: u32,
    interface_kind: &'static str,
    status_probe_performed: bool,
    status: &'static str,
    reason: &'static str,
}

fn tpm2_acpi_details_from_probe(probe: iommu_vtd::AcpiTableProbe) -> Tpm2AcpiDetails {
    if !probe.table_present {
        return Tpm2AcpiDetails {
            valid: false,
            platform_class: 0,
            control_area: 0,
            start_method: 0,
            interface_kind: TPM2_INTERFACE_KIND_NONE,
            status_probe_performed: probe.probe_performed,
            status: TPM2_ACPI_ABSENT_STATUS,
            reason: probe.reason,
        };
    }
    let Some(table) = probe.table_data else {
        return tpm2_details_too_short();
    };
    if table.len() < TPM2_COMMON_INTERFACE_LEN {
        return tpm2_details_too_short();
    }

    let platform_class = read_le_u16(table, TPM2_PLATFORM_CLASS_OFFSET).unwrap_or(0);
    let control_area = read_le_u64(table, TPM2_CONTROL_AREA_OFFSET).unwrap_or(0);
    let start_method = read_le_u32(table, TPM2_START_METHOD_OFFSET).unwrap_or(0);
    let interface_kind = tpm2_start_method_kind(start_method);
    let (status, reason) = tpm2_interface_status(start_method, control_area);

    Tpm2AcpiDetails {
        valid: true,
        platform_class,
        control_area,
        start_method,
        interface_kind,
        status_probe_performed: true,
        status,
        reason,
    }
}

fn tpm2_details_too_short() -> Tpm2AcpiDetails {
    Tpm2AcpiDetails {
        valid: false,
        platform_class: 0,
        control_area: 0,
        start_method: 0,
        interface_kind: TPM2_INTERFACE_KIND_NONE,
        status_probe_performed: true,
        status: TPM2_INTERFACE_STATUS_TABLE_TOO_SHORT,
        reason: TPM2_INTERFACE_REASON_TABLE_TOO_SHORT,
    }
}

fn tpm2_start_method_kind(start_method: u32) -> &'static str {
    match start_method {
        0 => TPM2_INTERFACE_KIND_NOT_SET,
        1 | 3..=5 | 9..=10 => TPM2_INTERFACE_KIND_VENDOR_RESERVED,
        2 => TPM2_INTERFACE_KIND_ACPI_START_METHOD,
        6 => TPM2_INTERFACE_KIND_TIS_MMIO_CANCEL,
        7 => TPM2_INTERFACE_KIND_CRB,
        8 => TPM2_INTERFACE_KIND_CRB_ACPI_START,
        11 => TPM2_INTERFACE_KIND_CRB_ARM_SMC_HVC,
        12 => TPM2_INTERFACE_KIND_FIFO_I2C,
        13 => TPM2_INTERFACE_KIND_CRB_AMD_MAILBOX,
        14 => TPM2_INTERFACE_KIND_FUTURE_MMIO,
        15 => TPM2_INTERFACE_KIND_CRB_ARM_FFA,
        _ => TPM2_INTERFACE_KIND_FUTURE_RESERVED,
    }
}

fn tpm2_interface_status(start_method: u32, control_area: u64) -> (&'static str, &'static str) {
    match start_method {
        7 | 8 => {
            if control_area == 0 {
                (
                    TPM2_INTERFACE_STATUS_CONTROL_AREA_MISSING,
                    TPM2_INTERFACE_REASON_CONTROL_AREA_MISSING,
                )
            } else {
                (
                    TPM2_INTERFACE_STATUS_CRB_DETAILS_PARSED,
                    TPM2_INTERFACE_REASON_CRB_PARSED,
                )
            }
        }
        6 => {
            if control_area == 0 {
                (
                    TPM2_INTERFACE_STATUS_TIS_FIXED_BASE_NOT_MAPPED,
                    TPM2_INTERFACE_REASON_TIS_FIXED_BASE,
                )
            } else {
                (
                    TPM2_INTERFACE_STATUS_TIS_DETAILS_PARSED,
                    TPM2_INTERFACE_REASON_TIS_PARSED,
                )
            }
        }
        _ => (
            TPM2_INTERFACE_STATUS_START_METHOD_UNSUPPORTED,
            TPM2_INTERFACE_REASON_START_METHOD_UNSUPPORTED,
        ),
    }
}

fn read_le_u16(data: &[u8], offset: usize) -> Option<u16> {
    read_le(data, offset, 2).map(|value| value as u16)
}

fn read_le_u32(data: &[u8], offset: usize) -> Option<u32> {
    read_le(data, offset, 4).map(|value| value as u32)
}

fn read_le_u64(data: &[u8], offset: usize) -> Option<u64> {
    read_le(data, offset, 8)
}

fn read_le(data: &[u8], offset: usize, width: usize) -> Option<u64> {
    let mut value = 0u64;
    let mut idx = 0usize;
    while idx < width {
        let byte = *data.get(offset.checked_add(idx)?)?;
        value |= (byte as u64) << ((idx * 8) as u32);
        idx += 1;
    }
    Some(value)
}
