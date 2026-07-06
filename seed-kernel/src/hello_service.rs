use raios_core::sha256_bytes;
use spin::Mutex;

use sha2::{Digest, Sha256};

use crate::{
    agent_protocol_module_write_boundary_append_contract::{
        self as rollback_append_contract, evaluate_module_audit_rollback_append_contract_candidate,
        module_audit_rollback_append_contract_snapshot_from_storage_and_engine,
    },
    agent_protocol_module_write_boundary_append_engine::{
        evaluate_module_audit_rollback_append_engine_candidate,
        module_audit_rollback_append_engine_snapshot,
    },
    agent_protocol_module_write_boundary_storage_layout::{
        self as rollback_storage_layout, evaluate_module_audit_rollback_storage_layout_candidate,
        module_audit_rollback_storage_layout_snapshot,
    },
    agent_protocol_support::{
        begin_response, end_response, json_sha256, json_str, method_eq, raw, raw_bool, raw_fmt,
        raw_line,
    },
    ahci, current_boot_service, descriptor_sources, event_log, pci,
};

mod command_targets;
mod constants;
mod descriptor_identity;
mod emitters;
mod hash_support;
mod lifecycle_binding;
mod preflight;
mod records;
mod rollback_authority_gates;
mod rollback_bindings;
mod rollback_hashes_a;
mod rollback_hashes_b;
mod rollback_writer_bindings;
mod rollback_writer_gate;
mod runtime;
mod state_machine;
mod state_records;
mod storage_authority_gate;
mod storage_gate_hash;

pub(crate) use command_targets::*;
pub(crate) use constants::*;
pub(crate) use descriptor_identity::*;
pub(crate) use emitters::*;
pub(crate) use hash_support::*;
pub(crate) use lifecycle_binding::*;
pub(crate) use preflight::*;
pub(crate) use records::*;
pub(crate) use rollback_authority_gates::*;
pub(crate) use rollback_writer_bindings::*;
pub(crate) use rollback_writer_gate::*;
pub(crate) use runtime::*;
pub(crate) use state_machine::*;
pub(crate) use state_records::*;
pub(crate) use storage_authority_gate::*;
