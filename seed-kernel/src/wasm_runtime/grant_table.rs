//! In-memory revocable host-import grants for one live Wasm domain.
//!
//! The hot path remains allocation-free. Slice 3 additionally installs the
//! validated durable boot fold into a separate kernel-owned snapshot.

use alloc::{string::String, vec::Vec};
use spin::Mutex;

const GRANT_SLOT_CAPACITY: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum HostImportId {
    EnvLog,
    EnvCounterGet,
}

#[derive(Clone)]
struct BootGrantSlot {
    service_id: String,
    domain_instance: u64,
    binding_sha256: [u8; 32],
    surface: HostImportId,
    generation: u64,
    revoked: bool,
}

struct BootProjection {
    valid: bool,
    sha256: [u8; 32],
    event_count: u64,
    slots: Vec<BootGrantSlot>,
}

impl BootProjection {
    const fn empty() -> Self {
        Self {
            valid: false,
            sha256: [0; 32],
            event_count: 0,
            slots: Vec::new(),
        }
    }
}

static BOOT_PROJECTION: Mutex<BootProjection> = Mutex::new(BootProjection::empty());

pub(super) fn install_boot_projection(
    projection: crate::agent_protocol::durable_store::DurableWasmGrantProjection,
) {
    let mut boot = BOOT_PROJECTION.lock();
    boot.valid = projection.valid;
    boot.sha256 = projection.sha256;
    boot.event_count = projection.event_count;
    boot.slots.clear();
    if !projection.valid {
        return;
    }
    for slot in projection.slots {
        let surface = match slot.host_import_id {
            raios_core::wasm_import_grant_event::HostImportId::EnvLog => HostImportId::EnvLog,
            raios_core::wasm_import_grant_event::HostImportId::EnvCounterGet => {
                HostImportId::EnvCounterGet
            }
        };
        boot.slots.push(BootGrantSlot {
            service_id: slot.service_id,
            domain_instance: slot.domain_instance,
            binding_sha256: slot.binding_sha256,
            surface,
            generation: slot.generation,
            revoked: slot.revoked,
        });
    }
}

pub(super) fn boot_projection_evidence() -> (bool, [u8; 32], u64) {
    let boot = BOOT_PROJECTION.lock();
    (boot.valid, boot.sha256, boot.event_count)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum DurableImportState {
    LegacyDefault,
    Live,
    Revoked,
    DeniedInvalidProjection,
}

/// Resolves one exact durable domain/surface binding. Invalid boot history is
/// denied; valid history with no event preserves the explicit Slice-2 default.
pub(super) fn durable_import_state(
    service_id: &str,
    domain_instance: u64,
    binding_sha256: [u8; 32],
    surface: HostImportId,
) -> DurableImportState {
    let boot = BOOT_PROJECTION.lock();
    if !boot.valid {
        return DurableImportState::DeniedInvalidProjection;
    }
    let newest = boot
        .slots
        .iter()
        .filter(|slot| {
            slot.service_id == service_id
                && slot.domain_instance == domain_instance
                && slot.binding_sha256 == binding_sha256
                && slot.surface == surface
        })
        .max_by_key(|slot| slot.generation);
    match newest {
        None => DurableImportState::LegacyDefault,
        Some(slot) if slot.revoked => DurableImportState::Revoked,
        Some(_) => DurableImportState::Live,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GrantState {
    Live,
    Revoked,
}

#[derive(Clone, Copy)]
struct GrantSlot {
    instance_generation: u64,
    surface: HostImportId,
    state: GrantState,
}

pub(super) struct GrantTable {
    slots: [Option<GrantSlot>; GRANT_SLOT_CAPACITY],
}

impl GrantTable {
    pub(super) const fn new() -> Self {
        Self {
            slots: [None; GRANT_SLOT_CAPACITY],
        }
    }

    /// Installs a new live slot. A live duplicate is idempotent, but a
    /// revoked `(generation, surface)` can never be reused or revived.
    pub(super) fn grant(&mut self, instance_generation: u64, surface: HostImportId) -> bool {
        let mut empty = None;
        let mut idx = 0usize;
        while idx < self.slots.len() {
            match self.slots[idx] {
                Some(slot)
                    if slot.instance_generation == instance_generation
                        && slot.surface == surface =>
                {
                    return slot.state == GrantState::Live;
                }
                None if empty.is_none() => empty = Some(idx),
                _ => {}
            }
            idx += 1;
        }

        let Some(idx) = empty else {
            return false;
        };
        self.slots[idx] = Some(GrantSlot {
            instance_generation,
            surface,
            state: GrantState::Live,
        });
        true
    }

    pub(super) fn revoke(&mut self, instance_generation: u64, surface: HostImportId) -> bool {
        let mut idx = 0usize;
        while idx < self.slots.len() {
            let Some(slot) = self.slots[idx].as_mut() else {
                idx += 1;
                continue;
            };
            if slot.instance_generation == instance_generation && slot.surface == surface {
                slot.state = GrantState::Revoked;
                return true;
            }
            idx += 1;
        }
        false
    }

    /// Fixed-capacity scan: bounded and allocation-free on the host-call path.
    pub(super) fn is_live(&self, instance_generation: u64, surface: HostImportId) -> bool {
        let mut idx = 0usize;
        while idx < self.slots.len() {
            if let Some(slot) = self.slots[idx] {
                if slot.instance_generation == instance_generation && slot.surface == surface {
                    return slot.state == GrantState::Live;
                }
            }
            idx += 1;
        }
        false
    }
}
