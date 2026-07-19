//! In-memory revocable host-import grants for one live Wasm domain.
//!
//! Slice 2 deliberately keeps this projection in RAM. Durable grant/revoke
//! records and boot-time re-folding belong to ADR 0023 Slice 3.

const GRANT_SLOT_CAPACITY: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum HostImportId {
    EnvLog,
    EnvCounterGet,
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
