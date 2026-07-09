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
    pub tpm2_acpi_table_length: u32,
    pub tpm2_acpi_table_revision: u8,
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
            tpm2_acpi_table_length: 0,
            tpm2_acpi_table_revision: 0,
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
    HardwareBindingSnapshot {
        probe_performed: probe.probe_performed,
        acpi_rsdp_present: probe.rsdp_present,
        acpi_root_table_valid: probe.root_table_valid,
        tpm2_acpi_table_present: probe.table_present,
        tpm2_acpi_table_length: probe.table_length,
        tpm2_acpi_table_revision: probe.table_revision,
        status,
        reason: probe.reason,
    }
}
