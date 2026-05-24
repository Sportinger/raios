use crate::agent_protocol_support::method_head_eq;

pub(crate) use crate::agent_protocol_module_write_boundary_append_contract::{
    emit_module_audit_rollback_append_contract, emit_module_audit_rollback_append_contract_selftest,
};
pub(crate) use crate::agent_protocol_module_write_boundary_append_engine::{
    emit_module_audit_rollback_append_engine, emit_module_audit_rollback_append_engine_selftest,
};
pub(crate) use crate::agent_protocol_module_write_boundary_append_intent::{
    emit_module_audit_rollback_append_intent, emit_module_audit_rollback_append_intent_selftest,
};
pub(crate) use crate::agent_protocol_module_write_boundary_append_payload_hash::{
    emit_module_audit_rollback_append_payload_hash,
    emit_module_audit_rollback_append_payload_hash_selftest,
};
pub(crate) use crate::agent_protocol_module_write_boundary_availability::{
    emit_module_audit_rollback_availability, emit_module_audit_rollback_availability_selftest,
};
pub(crate) use crate::agent_protocol_module_write_boundary_boundary::{
    emit_module_audit_rollback_write_boundary, emit_module_audit_rollback_write_boundary_selftest,
};
pub(crate) use crate::agent_protocol_module_write_boundary_storage_layout::{
    emit_module_audit_rollback_storage_layout, emit_module_audit_rollback_storage_layout_selftest,
};
pub(crate) use crate::agent_protocol_module_write_boundary_write_policy::{
    emit_module_audit_rollback_write_policy, emit_module_audit_rollback_write_policy_selftest,
};

pub(crate) fn module_audit_rollback_availability_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_availability")
        || method_head_eq(method, "module.audit_rollback_store_availability")
}

pub(crate) fn module_audit_rollback_availability_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_availability_selftest")
        || method_head_eq(method, "module.audit_rollback_store_availability_selftest")
}

pub(crate) fn module_audit_rollback_write_policy_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_write_policy")
        || method_head_eq(method, "module.audit_rollback_policy")
}

pub(crate) fn module_audit_rollback_write_policy_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_write_policy_selftest")
        || method_head_eq(method, "module.audit_rollback_policy_selftest")
}

pub(crate) fn module_audit_rollback_storage_layout_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_storage_layout")
        || method_head_eq(method, "module.audit_rollback_persistence_layout")
}

pub(crate) fn module_audit_rollback_storage_layout_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_storage_layout_selftest")
        || method_head_eq(method, "module.audit_rollback_persistence_layout_selftest")
}

pub(crate) fn module_audit_rollback_append_engine_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_append_engine")
        || method_head_eq(method, "module.audit_rollback_append_engine_readiness")
}

pub(crate) fn module_audit_rollback_append_engine_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_append_engine_selftest")
        || method_head_eq(
            method,
            "module.audit_rollback_append_engine_readiness_selftest",
        )
}

pub(crate) fn module_audit_rollback_append_contract_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_append_contract")
        || method_head_eq(method, "module.audit_rollback_storage_contract")
}

pub(crate) fn module_audit_rollback_append_contract_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_append_contract_selftest")
        || method_head_eq(method, "module.audit_rollback_storage_contract_selftest")
}

pub(crate) fn module_audit_rollback_append_payload_hash_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_append_payload_hash")
        || method_head_eq(method, "module.audit_rollback_append_payload")
}

pub(crate) fn module_audit_rollback_append_payload_hash_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_append_payload_hash_selftest")
        || method_head_eq(method, "module.audit_rollback_append_payload_selftest")
}

pub(crate) fn module_audit_rollback_append_intent_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_append_intent")
        || method_head_eq(method, "module.audit_rollback_append_request")
}

pub(crate) fn module_audit_rollback_append_intent_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_append_intent_selftest")
        || method_head_eq(method, "module.audit_rollback_append_request_selftest")
}

pub(crate) fn module_audit_rollback_write_boundary_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_write_boundary")
        || method_head_eq(method, "module.audit_rollback_write_gate")
}

pub(crate) fn module_audit_rollback_write_boundary_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_write_boundary_selftest")
        || method_head_eq(method, "module.audit_rollback_write_gate_selftest")
}
