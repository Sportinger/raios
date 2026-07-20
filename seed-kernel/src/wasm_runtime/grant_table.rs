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
    verified_domain: Option<VerifiedDomain>,
}

#[derive(Clone)]
struct VerifiedDomain {
    service_id: String,
    domain_instance: u64,
    binding_sha256: [u8; 32],
}

impl VerifiedDomain {
    fn new(service_id: &str, domain_instance: u64, binding_sha256: [u8; 32]) -> Self {
        Self {
            service_id: String::from(service_id),
            domain_instance,
            binding_sha256,
        }
    }
}

impl BootProjection {
    const fn empty() -> Self {
        Self {
            valid: false,
            sha256: [0; 32],
            event_count: 0,
            slots: Vec::new(),
            verified_domain: None,
        }
    }
}

static BOOT_PROJECTION: Mutex<BootProjection> = Mutex::new(BootProjection::empty());

pub(super) fn install_boot_projection(
    projection: crate::agent_protocol::durable_store::DurableWasmGrantProjection,
) {
    install_projection(&mut BOOT_PROJECTION.lock(), projection);
}

pub(super) fn install_post_commit_projection(
    projection: crate::agent_protocol::durable_store::DurableWasmGrantProjection,
    committed_sha256: [u8; 32],
) -> bool {
    transition_post_commit_projection(&mut BOOT_PROJECTION.lock(), projection, committed_sha256)
}

pub(super) fn deny_post_commit_projection() {
    deny_projection(&mut BOOT_PROJECTION.lock());
}

fn deny_projection(boot: &mut BootProjection) {
    boot.valid = false;
    boot.sha256 = [0; 32];
    boot.event_count = 0;
    boot.slots.clear();
    boot.verified_domain = None;
}

fn transition_post_commit_projection(
    boot: &mut BootProjection,
    projection: crate::agent_protocol::durable_store::DurableWasmGrantProjection,
    committed_sha256: [u8; 32],
) -> bool {
    if !projection.valid || projection.sha256 != committed_sha256 {
        deny_projection(boot);
        return false;
    }
    install_projection(boot, projection);
    true
}

fn install_projection(
    boot: &mut BootProjection,
    projection: crate::agent_protocol::durable_store::DurableWasmGrantProjection,
) {
    boot.verified_domain = None;
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

pub(super) fn post_commit_projection_transition_selftest() -> bool {
    use crate::agent_protocol::durable_store::{DurableWasmGrantProjection, DurableWasmGrantSlot};
    use alloc::vec;
    use raios_core::wasm_import_grant_event::{GrantScope, HostImportId as DurableHostImportId};

    const SERVICE: &str = "svc.dev.granted_candidate";
    let binding = raios_core::sha256_bytes(SERVICE.as_bytes());
    let slot = |surface, generation| DurableWasmGrantSlot {
        grant_id: String::from("selftest-grant"),
        grant_sha256: [generation as u8; 32],
        revoke_id: None,
        service_id: String::from(SERVICE),
        domain_instance: 1,
        binding_sha256: binding,
        host_import_id: surface,
        scope: GrantScope::HostCall,
        generation,
        grant_epoch: generation,
        revoked: false,
    };
    let projection = |sha256, slots| DurableWasmGrantProjection {
        valid: true,
        reason: "selftest_valid",
        sha256,
        event_count: 1,
        next_epoch: 2,
        slots,
    };
    let mut boot = BootProjection::empty();
    install_projection(
        &mut boot,
        projection([1; 32], vec![slot(DurableHostImportId::EnvCounterGet, 1)]),
    );
    boot.verified_domain = Some(VerifiedDomain::new(SERVICE, 1, binding));
    let source_live = domain_is_verified_in(&boot, SERVICE, 1, binding)
        && durable_import_state_in(&boot, SERVICE, 1, binding, HostImportId::EnvCounterGet)
            == DurableImportState::Live;
    let mismatch_denied = !transition_post_commit_projection(
        &mut boot,
        projection([2; 32], vec![slot(DurableHostImportId::EnvLog, 2)]),
        [3; 32],
    );
    let denied_empty = !boot.valid
        && boot.sha256 == [0; 32]
        && boot.event_count == 0
        && boot.slots.is_empty()
        && !domain_is_verified_in(&boot, SERVICE, 1, binding)
        && durable_import_state_in(&boot, SERVICE, 1, binding, HostImportId::EnvCounterGet)
            == DurableImportState::DeniedInvalidProjection;
    let target_installed = transition_post_commit_projection(
        &mut boot,
        projection([4; 32], vec![slot(DurableHostImportId::EnvLog, 2)]),
        [4; 32],
    );
    let target_exact = target_installed
        && !domain_is_verified_in(&boot, SERVICE, 1, binding)
        && durable_import_state_in(&boot, SERVICE, 1, binding, HostImportId::EnvLog)
            == DurableImportState::DeniedMissingDomain
        && boot.slots.len() == 1;

    source_live && mismatch_denied && denied_empty && target_exact
}

pub(super) fn boot_projection_evidence() -> (bool, [u8; 32], u64) {
    let boot = BOOT_PROJECTION.lock();
    (boot.valid, boot.sha256, boot.event_count)
}

fn domain_is_verified_in(
    boot: &BootProjection,
    service_id: &str,
    domain_instance: u64,
    binding_sha256: [u8; 32],
) -> bool {
    boot.valid
        && boot.verified_domain.as_ref().is_some_and(|domain| {
            domain.service_id == service_id
                && domain.domain_instance == domain_instance
                && domain.binding_sha256 == binding_sha256
        })
}

pub(super) fn register_verified_durable_domain(
    service_id: &str,
    domain_instance: u64,
    binding_sha256: [u8; 32],
) -> bool {
    const DURABLE_SERVICE_ID: &str = "svc.dev.granted_candidate";
    const DURABLE_DOMAIN_INSTANCE: u64 = 1;

    if service_id != DURABLE_SERVICE_ID
        || domain_instance != DURABLE_DOMAIN_INSTANCE
        || binding_sha256 != raios_core::sha256_bytes(DURABLE_SERVICE_ID.as_bytes())
    {
        return false;
    }
    let mut boot = BOOT_PROJECTION.lock();
    if !boot.valid {
        return false;
    }
    boot.verified_domain = Some(VerifiedDomain::new(
        service_id,
        domain_instance,
        binding_sha256,
    ));
    true
}

pub(super) fn clear_verified_durable_domain(
    service_id: &str,
    domain_instance: u64,
    binding_sha256: [u8; 32],
) -> bool {
    clear_verified_domain_in(
        &mut BOOT_PROJECTION.lock(),
        service_id,
        domain_instance,
        binding_sha256,
    )
}

fn clear_verified_domain_in(
    boot: &mut BootProjection,
    service_id: &str,
    domain_instance: u64,
    binding_sha256: [u8; 32],
) -> bool {
    if !boot.verified_domain.as_ref().is_some_and(|domain| {
        domain.service_id == service_id
            && domain.domain_instance == domain_instance
            && domain.binding_sha256 == binding_sha256
    }) {
        return false;
    }
    boot.verified_domain = None;
    true
}

pub(super) fn verified_domain_finalization_selftest(
    start_trap_succeeded: bool,
    nonzero_succeeded: bool,
    zero_succeeded: bool,
) -> bool {
    let binding = raios_core::sha256_bytes(b"svc.dev.granted_candidate");
    let mut boot = BootProjection::empty();
    boot.valid = true;
    let identity = || VerifiedDomain::new("svc.dev.granted_candidate", 1, binding);
    boot.verified_domain = Some(identity());
    let trap_cleared = !start_trap_succeeded
        && clear_verified_domain_in(&mut boot, "svc.dev.granted_candidate", 1, binding);
    boot.verified_domain = Some(identity());
    let nonzero_cleared = !nonzero_succeeded
        && clear_verified_domain_in(&mut boot, "svc.dev.granted_candidate", 1, binding);
    boot.verified_domain = Some(identity());
    trap_cleared
        && nonzero_cleared
        && zero_succeeded
        && domain_is_verified_in(&boot, "svc.dev.granted_candidate", 1, binding)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum DurableImportState {
    Live,
    Revoked,
    DeniedMissingDomain,
    DeniedMissingSurface,
    DeniedInvalidProjection,
}

pub(super) fn durable_import_state(
    service_id: &str,
    domain_instance: u64,
    binding_sha256: [u8; 32],
    surface: HostImportId,
) -> DurableImportState {
    let boot = BOOT_PROJECTION.lock();
    durable_import_state_in(&boot, service_id, domain_instance, binding_sha256, surface)
}

fn durable_import_state_in(
    boot: &BootProjection,
    service_id: &str,
    domain_instance: u64,
    binding_sha256: [u8; 32],
    surface: HostImportId,
) -> DurableImportState {
    if !boot.valid {
        return DurableImportState::DeniedInvalidProjection;
    }
    if !boot.verified_domain.as_ref().is_some_and(|domain| {
        domain.service_id == service_id
            && domain.domain_instance == domain_instance
            && domain.binding_sha256 == binding_sha256
    }) {
        return DurableImportState::DeniedMissingDomain;
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
        None => DurableImportState::DeniedMissingSurface,
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
