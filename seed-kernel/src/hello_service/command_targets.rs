use super::*;

fn load_target(descriptor: LoadDescriptor) -> current_boot_service::LoadTarget<'static> {
    current_boot_service::LoadTarget {
        service_id: descriptor.service_id,
        artifact_id: descriptor.artifact_id,
        descriptor_id: descriptor.id,
    }
}

pub(crate) fn load_request(method: &str) -> Option<LoadRequest> {
    load_request_for_head(method, "module.load_ephemeral")
        .or_else(|| load_request_for_head(method, "service.load_ephemeral"))
}

pub(crate) fn hot_swap_request(method: &str) -> Option<LoadRequest> {
    load_request_for_head(method, "service.hot_swap")
}

pub(crate) fn load_request_for_head(method: &str, head: &'static str) -> Option<LoadRequest> {
    let target = current_boot_service::method_target(method, head)?;
    let descriptor = verified_load_descriptor_for_target(target, head == "service.hot_swap")?;
    Some(LoadRequest {
        source_method: head,
        descriptor,
    })
}

pub(crate) fn verified_load_descriptor_for_target(
    target: &str,
    allow_replacement: bool,
) -> Option<LoadDescriptor> {
    let source = descriptor_source_for_target(target)?;
    if !descriptor_sources::validate_descriptor_source(source) {
        return None;
    }
    let replacement_target = replacement_target_matches(target);
    let artifact_identity = if allow_replacement && replacement_target {
        descriptor_sources::hello_builtin_artifact_identity_v2()
    } else {
        descriptor_sources::hello_builtin_artifact_identity()
    };
    let descriptor = load_descriptor_from_source(source, artifact_identity);
    if !descriptor_sources::validate_builtin_hello_artifact_identity(descriptor.artifact_identity) {
        return None;
    }
    if descriptor_target_matches(target, descriptor)
        || descriptor_source_target_matches(target, source)
        || (allow_replacement && replacement_target)
    {
        Some(descriptor)
    } else {
        None
    }
}

pub(crate) fn descriptor_source_for_target(
    target: &str,
) -> Option<descriptor_sources::DescriptorSourceRecord> {
    if current_boot_service::descriptor_source_target_matches_locator(
        target,
        descriptor_sources::HELLO_HOST_BOUND_DESCRIPTOR_SOURCE_LOCATOR,
    ) || target.eq_ignore_ascii_case(HELLO_SERVICE_DESCRIPTOR.host_bound_alias)
    {
        descriptor_sources::lookup_host_bound_descriptor_source(LOAD_DESCRIPTOR_ID)
    } else {
        descriptor_sources::lookup_current_image_descriptor_source(LOAD_DESCRIPTOR_ID)
    }
}

pub(crate) fn load_descriptor_from_source(
    source: descriptor_sources::DescriptorSourceRecord,
    artifact_identity: descriptor_sources::ArtifactIdentityRecord,
) -> LoadDescriptor {
    LoadDescriptor {
        schema: source.schema,
        id: source.id,
        canonicalization: source.canonicalization,
        source_locator: source.locator,
        source_kind: source.kind,
        binds_source_locator: source.binds_source_locator,
        binds_source_kind: source.binds_source_kind,
        binds_source_hash: source.binds_source_hash,
        source_text: source.text,
        source_envelope: source.signed_envelope,
        artifact_identity,
        service_id: source.service_id,
        artifact_id: source.artifact_id,
        artifact_kind: source.artifact_kind,
        scope: source.scope,
        classification: source.classification,
        persistence: source.persistence,
    }
}

pub(crate) fn descriptor_source_target_matches(
    target: &str,
    source: descriptor_sources::DescriptorSourceRecord,
) -> bool {
    current_boot_service::descriptor_source_target_matches(
        target,
        source.locator,
        descriptor_sources::HELLO_HOST_BOUND_DESCRIPTOR_SOURCE_LOCATOR,
        HELLO_SERVICE_DESCRIPTOR,
    )
}

pub(crate) fn replacement_target_matches(target: &str) -> bool {
    current_boot_service::replacement_target_matches(target, HELLO_SERVICE_DESCRIPTOR)
}

pub(crate) fn reset_state_hot_swap_target(method: &str) -> bool {
    current_boot_service::reset_state_hot_swap_target(method, HELLO_SERVICE_DESCRIPTOR)
}

pub(crate) fn target_arg_matches(method: &str, head: &str) -> bool {
    current_boot_service::target_arg_matches(
        method,
        head,
        load_target(LOAD_DESCRIPTOR),
        HELLO_SERVICE_DESCRIPTOR,
    )
}

pub(crate) fn descriptor_target_matches(target: &str, descriptor: LoadDescriptor) -> bool {
    current_boot_service::descriptor_target_matches(
        target,
        load_target(descriptor),
        HELLO_SERVICE_DESCRIPTOR,
    )
}

pub(crate) fn health_state(snapshot: Snapshot) -> &'static str {
    current_boot_service::health_state(snapshot.loaded, snapshot.running, HELLO_SERVICE_DESCRIPTOR)
}

pub(crate) fn service_slot_activation_status(snapshot: Snapshot) -> &'static str {
    current_boot_service::service_slot_activation_status(
        snapshot.loaded,
        snapshot.running,
        snapshot.last_action,
        snapshot.last_reason,
        HELLO_SERVICE_DESCRIPTOR,
    )
}

pub(crate) fn service_slot_activation_active(snapshot: Snapshot) -> bool {
    current_boot_service::service_slot_activation_active(snapshot.loaded)
}
