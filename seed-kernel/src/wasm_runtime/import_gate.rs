//! ADR 0023 Slice 1 pass-through chokepoint for Wasm host-import registration.
//!
//! The per-domain grant slot table (the fold of the append-only grant/revoke
//! chain) and revocation enforcement land in Slice 2 and later. For now,
//! [`ImportGate::link`] transparently forwards registration to wasmi and only
//! records the surfaces that it successfully linked.

use alloc::vec::Vec;
use wasmi::{IntoFunc, Linker};

use super::envelope::EnvelopeState;

pub(super) struct ImportGate<'a> {
    linker: &'a mut Linker<EnvelopeState>,
    linked: Vec<(&'static str, &'static str)>,
}

impl<'a> ImportGate<'a> {
    pub(super) fn new(linker: &'a mut Linker<EnvelopeState>) -> Self {
        Self {
            linker,
            linked: Vec::new(),
        }
    }

    pub(super) fn link<Params, Results>(
        &mut self,
        module: &'static str,
        name: &'static str,
        func: impl IntoFunc<EnvelopeState, Params, Results>,
    ) -> Result<(), &'static str> {
        self.linker
            .func_wrap(module, name, func)
            .map_err(|_| "host_import_link_failed")?;
        self.linked.push((module, name));
        Ok(())
    }

    pub(super) fn linked_count(&self) -> u64 {
        self.linked.len() as u64
    }
}
