use crate::event_log;

#[derive(Clone, Copy)]
pub(crate) struct ServiceDescriptor {
    pub(crate) service_id: &'static str,
    pub(crate) artifact_id: &'static str,
    pub(crate) artifact_kind: &'static str,
    pub(crate) scope: &'static str,
    pub(crate) classification: &'static str,
    pub(crate) persistence: &'static str,
    pub(crate) service_capability: &'static str,
    pub(crate) health_capability: &'static str,
    pub(crate) rollback_preview_capability: &'static str,
    pub(crate) rollback_apply_capability: &'static str,
    pub(crate) rollback_materialize_capability: &'static str,
    pub(crate) rollback_inspect_capability: &'static str,
    pub(crate) primary_alias: &'static str,
    pub(crate) host_bound_alias: &'static str,
    pub(crate) replacement_service_id: &'static str,
    pub(crate) replacement_alias: &'static str,
    pub(crate) replacement_artifact_identity_id: &'static str,
    pub(crate) reset_state_service_id: &'static str,
    pub(crate) reset_state_alias: &'static str,
    pub(crate) artifact_load_plan_preflight_id: &'static str,
    pub(crate) artifact_load_plan_preflight_status: &'static str,
    pub(crate) service_slot_intent_id: &'static str,
    pub(crate) ram_only_service_slot_id: &'static str,
    pub(crate) service_slot_activation_id: &'static str,
    pub(crate) service_slot_activation_active_status: &'static str,
    pub(crate) service_slot_activation_stopped_status: &'static str,
    pub(crate) service_slot_activation_cleared_status: &'static str,
    pub(crate) service_slot_activation_missing_status: &'static str,
    pub(crate) inventory_kind: &'static str,
    pub(crate) inventory_replaceable: bool,
    pub(crate) inventory_core_owned: bool,
    pub(crate) inventory_health_running: &'static str,
    pub(crate) inventory_health_stopped: &'static str,
    pub(crate) inventory_health_missing: &'static str,
    pub(crate) event_lifecycle_kind: &'static str,
    pub(crate) event_health_kind: &'static str,
    pub(crate) event_rollback_preview_kind: &'static str,
    pub(crate) event_rollback_apply_kind: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct ServiceState {
    pub(crate) loaded: bool,
    pub(crate) running: bool,
    pub(crate) generation: u64,
    pub(crate) state_counter: u64,
    pub(crate) last_action: &'static str,
    pub(crate) last_reason: &'static str,
    pub(crate) last_inventory_change: &'static str,
    pub(crate) last_event_id: Option<event_log::EventId>,
    pub(crate) load_event_id: Option<event_log::EventId>,
    pub(crate) start_event_id: Option<event_log::EventId>,
    pub(crate) hot_swap_event_id: Option<event_log::EventId>,
    pub(crate) stop_event_id: Option<event_log::EventId>,
    pub(crate) drop_event_id: Option<event_log::EventId>,
}

impl ServiceState {
    pub(crate) const fn new() -> Self {
        Self {
            loaded: false,
            running: false,
            generation: 0,
            state_counter: 0,
            last_action: "none",
            last_reason: "not_loaded",
            last_inventory_change: "none",
            last_event_id: None,
            load_event_id: None,
            start_event_id: None,
            hot_swap_event_id: None,
            stop_event_id: None,
            drop_event_id: None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct LoadTarget<'a> {
    pub(crate) service_id: &'a str,
    pub(crate) artifact_id: &'a str,
    pub(crate) descriptor_id: &'a str,
}

pub(crate) fn method_target<'a>(method: &'a str, head: &str) -> Option<&'a str> {
    let method = method.trim();
    if !raios_core::method_head_eq(method, head) {
        return None;
    }
    Some(method[head.len()..].trim())
}

pub(crate) fn target_arg_matches(
    method: &str,
    head: &str,
    target: LoadTarget<'_>,
    descriptor: ServiceDescriptor,
) -> bool {
    let Some(arg) = method_target(method, head) else {
        return false;
    };
    descriptor_target_matches(arg, target, descriptor)
}

pub(crate) fn descriptor_target_matches(
    target: &str,
    load_target: LoadTarget<'_>,
    descriptor: ServiceDescriptor,
) -> bool {
    target.eq_ignore_ascii_case(load_target.service_id)
        || target.eq_ignore_ascii_case(descriptor.primary_alias)
        || target.eq_ignore_ascii_case(load_target.artifact_id)
        || target.eq_ignore_ascii_case(load_target.descriptor_id)
}

pub(crate) fn descriptor_source_target_matches(
    target: &str,
    source_locator: &str,
    host_bound_locator: &str,
    descriptor: ServiceDescriptor,
) -> bool {
    descriptor_source_target_matches_locator(target, source_locator)
        || (source_locator == host_bound_locator
            && target.eq_ignore_ascii_case(descriptor.host_bound_alias))
}

pub(crate) fn descriptor_source_target_matches_locator(target: &str, locator: &str) -> bool {
    target.eq_ignore_ascii_case(locator)
}

pub(crate) fn replacement_target_matches(target: &str, descriptor: ServiceDescriptor) -> bool {
    target.eq_ignore_ascii_case(descriptor.replacement_service_id)
        || target.eq_ignore_ascii_case(descriptor.replacement_alias)
        || target.eq_ignore_ascii_case(descriptor.replacement_artifact_identity_id)
}

pub(crate) fn reset_state_hot_swap_target(method: &str, descriptor: ServiceDescriptor) -> bool {
    let Some(target) = method_target(method, "service.hot_swap") else {
        return false;
    };
    target.eq_ignore_ascii_case(descriptor.reset_state_service_id)
        || target.eq_ignore_ascii_case(descriptor.reset_state_alias)
}

pub(crate) fn health_state(
    loaded: bool,
    running: bool,
    descriptor: ServiceDescriptor,
) -> &'static str {
    if running {
        descriptor.inventory_health_running
    } else if loaded {
        descriptor.inventory_health_stopped
    } else {
        descriptor.inventory_health_missing
    }
}

pub(crate) fn service_slot_activation_status(
    loaded: bool,
    running: bool,
    last_action: &'static str,
    last_reason: &'static str,
    descriptor: ServiceDescriptor,
) -> &'static str {
    if running {
        descriptor.service_slot_activation_active_status
    } else if loaded {
        descriptor.service_slot_activation_stopped_status
    } else if last_action == "drop" && last_reason == "dropped" {
        descriptor.service_slot_activation_cleared_status
    } else {
        descriptor.service_slot_activation_missing_status
    }
}

pub(crate) fn service_slot_activation_active(loaded: bool) -> bool {
    loaded
}
