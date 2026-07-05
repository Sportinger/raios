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
        begin_response, emit_inline_string_array, end_response, json_event_id_option, json_opt_str,
        json_sha256, json_sha256_option, json_str, method_eq, method_head_eq, raw, raw_bool,
        raw_fmt, raw_line,
    },
    ahci, descriptor_sources, event_log, pci,
};

mod command_targets;
mod constants;
mod descriptor_identity;
mod emitters;
mod hash_support;
mod lifecycle_binding;
mod preflight;
mod records;
mod rollback_bindings;
mod rollback_hashes_a;
mod rollback_hashes_b;
mod runtime;
mod state_machine;
mod state_records;
mod storage_gate_hash;

pub(crate) use command_targets::*;
pub(crate) use constants::*;
pub(crate) use descriptor_identity::*;
pub(crate) use emitters::*;
pub(crate) use hash_support::*;
pub(crate) use lifecycle_binding::*;
pub(crate) use preflight::*;
pub(crate) use records::*;
pub(crate) use rollback_bindings::*;
pub(crate) use rollback_hashes_a::*;
pub(crate) use rollback_hashes_b::*;
pub(crate) use runtime::*;
pub(crate) use state_machine::*;
pub(crate) use state_records::*;
pub(crate) use storage_gate_hash::*;
