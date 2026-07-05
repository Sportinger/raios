use super::*;

pub(crate) const LOAD_DESCRIPTOR: LoadDescriptor = LoadDescriptor {
    schema: LOAD_DESCRIPTOR_SCHEMA,
    id: LOAD_DESCRIPTOR_ID,
    canonicalization: LOAD_DESCRIPTOR_CANONICALIZATION,
    source_locator: LOAD_DESCRIPTOR_SOURCE_LOCATOR,
    source_kind: LOAD_DESCRIPTOR_SOURCE_KIND,
    binds_source_locator: None,
    binds_source_kind: None,
    binds_source_hash: None,
    source_text: descriptor_sources::HELLO_LOAD_DESCRIPTOR_SOURCE,
    source_envelope: Some(descriptor_sources::HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_ENVELOPE),
    artifact_identity: descriptor_sources::hello_builtin_artifact_identity(),
    service_id: SERVICE_ID,
    artifact_id: ARTIFACT_ID,
    artifact_kind: "builtin_stage0_test_service",
    scope: "current_boot",
    classification: "local_only",
    persistence: "none",
};

pub(crate) fn load_descriptor_source_hash() -> [u8; 32] {
    descriptor_sources::hello_load_descriptor_source_hash()
}

pub(crate) fn descriptor_source_hash(descriptor: LoadDescriptor) -> [u8; 32] {
    if let Some(hash) =
        descriptor_sources::descriptor_source_hash_for_locator(descriptor.source_locator)
    {
        hash
    } else {
        load_descriptor_source_hash()
    }
}

pub(crate) fn descriptor_source_signature_verified(descriptor: LoadDescriptor) -> bool {
    descriptor_sources::verify_descriptor_source_envelope_parts(
        descriptor.source_envelope,
        descriptor.source_locator,
        descriptor.source_kind,
        descriptor.source_text,
    )
}

pub(crate) fn artifact_identity_hash(descriptor: LoadDescriptor) -> [u8; 32] {
    descriptor_sources::artifact_identity_hash(descriptor.artifact_identity)
}

pub(crate) fn artifact_content_binding_hash(descriptor: LoadDescriptor) -> [u8; 32] {
    descriptor_sources::artifact_content_binding_hash(descriptor.artifact_identity)
}

pub(crate) fn artifact_reference_hash(descriptor: LoadDescriptor) -> [u8; 32] {
    descriptor_sources::artifact_reference_hash(descriptor.artifact_identity)
}

pub(crate) fn artifact_reference_bytes_hash(descriptor: LoadDescriptor) -> [u8; 32] {
    descriptor_sources::artifact_reference_bytes_hash(descriptor.artifact_identity)
}

pub(crate) fn service_version(descriptor: LoadDescriptor) -> &'static str {
    if descriptor.artifact_identity.id == descriptor_sources::HELLO_BUILTIN_ARTIFACT_IDENTITY_V2_ID
    {
        "v2"
    } else {
        "v1"
    }
}

pub(crate) fn artifact_load_plan_preflight_hash(descriptor: LoadDescriptor) -> [u8; 32] {
    artifact_load_plan_preflight_record(descriptor).preflight_hash
}

pub(crate) fn service_slot_activation_hash(descriptor: LoadDescriptor) -> [u8; 32] {
    service_slot_activation_record(descriptor, SERVICE_SLOT_ACTIVATION_ACTIVE_STATUS, true)
        .activation_hash
}
