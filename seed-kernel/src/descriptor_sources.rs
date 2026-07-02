use crate::agent_protocol_support::parse_sha256_ref;
use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use sha2::{Digest, Sha256};

include!(concat!(
    env!("OUT_DIR"),
    "/hello_host_bound_descriptor_source.rs"
));

pub(crate) const HELLO_SERVICE_ID: &str = "svc.demo.hello";
pub(crate) const HELLO_ARTIFACT_ID: &str = "builtin:svc.demo.hello";
pub(crate) const HELLO_LOAD_DESCRIPTOR_SCHEMA: &str = "raios.current_boot_load_descriptor.v0";
pub(crate) const HELLO_LOAD_DESCRIPTOR_ID: &str = "load_descriptor.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_LOAD_DESCRIPTOR_CANONICALIZATION: &str =
    "raios.current_boot_load_descriptor.canonical.v0";
pub(crate) const HELLO_LOAD_DESCRIPTOR_SOURCE_LOCATOR: &str =
    "current_image.descriptor_source.svc.demo.hello.v0";
pub(crate) const HELLO_LOAD_DESCRIPTOR_SOURCE_KIND: &str = "current_image_descriptor_source";
pub(crate) const HELLO_LOAD_DESCRIPTOR_SOURCE: &str =
    include_str!("../descriptors/svc.demo.hello.current_image.desc");
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_SCHEMA: &str =
    "raios.builtin_artifact_identity.v0";
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_ID: &str =
    "builtin_artifact_identity.svc.demo.hello.v0";
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_CANONICALIZATION: &str =
    "raios.builtin_artifact_identity.canonical.v0";
pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_SOURCE: &str =
    include_str!("../descriptors/svc.demo.hello.builtin_artifact_identity.desc");
pub(crate) const HELLO_DESCRIPTOR_SOURCE_TRUST_SELFTEST_ID: &str =
    "descriptor_source_trust_selftest.current_image.svc.demo.hello.v0";
pub(crate) const HELLO_DESCRIPTOR_SOURCE_TRUST_SELFTEST_CASES: usize = 5;
pub(crate) const HELLO_ARTIFACT_REFERENCE_TRUST_SELFTEST_ID: &str =
    "artifact_reference_trust_selftest.builtin.svc.demo.hello.v0";
pub(crate) const HELLO_ARTIFACT_REFERENCE_TRUST_SELFTEST_CASES: usize = 5;

#[derive(Clone, Copy)]
pub(crate) struct DescriptorSourceEnvelope {
    pub schema: &'static str,
    pub id: &'static str,
    pub algorithm: &'static str,
    pub verification_phase: &'static str,
    pub trust_scope: &'static str,
    pub payload_source_locator: &'static str,
    pub payload_source_kind: &'static str,
    pub payload_hash: [u8; 32],
    pub envelope_hash: [u8; 32],
    pub public_key_hash: [u8; 32],
    pub signature_hash: [u8; 32],
    pub public_key_sec1: &'static [u8],
    pub signature_der: &'static [u8],
    pub authorizes_external_artifact_load: bool,
    pub authorizes_persistent_install: bool,
    pub text: &'static str,
}

pub(crate) const HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_ENVELOPE: DescriptorSourceEnvelope =
    DescriptorSourceEnvelope {
        schema: HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_ENVELOPE_SCHEMA,
        id: HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_ENVELOPE_ID,
        algorithm: HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_SIGNATURE_ALGORITHM,
        verification_phase: HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_ENVELOPE_VERIFICATION_PHASE,
        trust_scope: HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_ENVELOPE_TRUST_SCOPE,
        payload_source_locator: HELLO_LOAD_DESCRIPTOR_SOURCE_LOCATOR,
        payload_source_kind: HELLO_LOAD_DESCRIPTOR_SOURCE_KIND,
        payload_hash: HELLO_HOST_BOUND_CURRENT_IMAGE_SOURCE_HASH,
        envelope_hash: HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_ENVELOPE_HASH,
        public_key_hash: HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_PUBLIC_KEY_HASH,
        signature_hash: HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_SIGNATURE_HASH,
        public_key_sec1: HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_PUBLIC_KEY_SEC1,
        signature_der: HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_SIGNATURE_DER,
        authorizes_external_artifact_load: false,
        authorizes_persistent_install: false,
        text: HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_ENVELOPE_TEXT,
    };

#[derive(Clone, Copy)]
pub(crate) struct ArtifactIdentityEnvelope {
    pub schema: &'static str,
    pub id: &'static str,
    pub algorithm: &'static str,
    pub verification_phase: &'static str,
    pub trust_scope: &'static str,
    pub payload_identity_id: &'static str,
    pub payload_artifact_id: &'static str,
    pub payload_hash: [u8; 32],
    pub envelope_hash: [u8; 32],
    pub public_key_hash: [u8; 32],
    pub signature_hash: [u8; 32],
    pub public_key_sec1: &'static [u8],
    pub signature_der: &'static [u8],
    pub authorizes_external_artifact_load: bool,
    pub authorizes_persistent_install: bool,
    pub authorizes_rollback_install: bool,
    pub text: &'static str,
}

pub(crate) const HELLO_BUILTIN_ARTIFACT_IDENTITY_ENVELOPE: ArtifactIdentityEnvelope =
    ArtifactIdentityEnvelope {
        schema: HELLO_BUILTIN_ARTIFACT_IDENTITY_ENVELOPE_SCHEMA,
        id: HELLO_BUILTIN_ARTIFACT_IDENTITY_ENVELOPE_ID,
        algorithm: HELLO_BUILTIN_ARTIFACT_IDENTITY_SIGNATURE_ALGORITHM,
        verification_phase: HELLO_BUILTIN_ARTIFACT_IDENTITY_ENVELOPE_VERIFICATION_PHASE,
        trust_scope: HELLO_BUILTIN_ARTIFACT_IDENTITY_ENVELOPE_TRUST_SCOPE,
        payload_identity_id: HELLO_BUILTIN_ARTIFACT_IDENTITY_ID,
        payload_artifact_id: HELLO_ARTIFACT_ID,
        payload_hash: HELLO_BUILTIN_ARTIFACT_IDENTITY_HASH,
        envelope_hash: HELLO_BUILTIN_ARTIFACT_IDENTITY_ENVELOPE_HASH,
        public_key_hash: HELLO_BUILTIN_ARTIFACT_IDENTITY_PUBLIC_KEY_HASH,
        signature_hash: HELLO_BUILTIN_ARTIFACT_IDENTITY_SIGNATURE_HASH,
        public_key_sec1: HELLO_BUILTIN_ARTIFACT_IDENTITY_PUBLIC_KEY_SEC1,
        signature_der: HELLO_BUILTIN_ARTIFACT_IDENTITY_SIGNATURE_DER,
        authorizes_external_artifact_load: false,
        authorizes_persistent_install: false,
        authorizes_rollback_install: false,
        text: HELLO_BUILTIN_ARTIFACT_IDENTITY_ENVELOPE_TEXT,
    };

#[derive(Clone, Copy)]
pub(crate) struct DescriptorSourceRecord {
    pub schema: &'static str,
    pub id: &'static str,
    pub canonicalization: &'static str,
    pub locator: &'static str,
    pub kind: &'static str,
    pub binds_source_locator: Option<&'static str>,
    pub binds_source_kind: Option<&'static str>,
    pub binds_source_hash: Option<[u8; 32]>,
    pub service_id: &'static str,
    pub artifact_id: &'static str,
    pub artifact_kind: &'static str,
    pub scope: &'static str,
    pub classification: &'static str,
    pub persistence: &'static str,
    pub accepts_external_artifact_bytes: bool,
    pub loads_external_artifact: bool,
    pub writes_persistent_state: bool,
    pub signed_envelope: Option<DescriptorSourceEnvelope>,
    pub text: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct ArtifactIdentityRecord {
    pub schema: &'static str,
    pub id: &'static str,
    pub canonicalization: &'static str,
    pub service_id: &'static str,
    pub artifact_id: &'static str,
    pub artifact_kind: &'static str,
    pub load_descriptor_id: &'static str,
    pub artifact_content_binding_schema: &'static str,
    pub artifact_content_binding_id: &'static str,
    pub artifact_content_kind: &'static str,
    pub artifact_content_source_locator: &'static str,
    pub artifact_content_source_hash: [u8; 32],
    pub artifact_content_binding_hash: [u8; 32],
    pub artifact_content_accepts_external_artifact_bytes: bool,
    pub artifact_content_loads_external_artifact: bool,
    pub artifact_content_maps_executable_pages: bool,
    pub artifact_content_writes_persistent_state: bool,
    pub artifact_reference_schema: &'static str,
    pub artifact_reference_id: &'static str,
    pub artifact_reference_kind: &'static str,
    pub artifact_reference_locator: &'static str,
    pub artifact_reference_hash: [u8; 32],
    pub artifact_reference_bytes_hash: [u8; 32],
    pub artifact_reference_content_binding_hash: [u8; 32],
    pub artifact_reference_accepts_external_artifact_bytes: bool,
    pub artifact_reference_loads_artifact_as_code: bool,
    pub artifact_reference_maps_executable_pages: bool,
    pub artifact_reference_writes_persistent_state: bool,
    pub scope: &'static str,
    pub classification: &'static str,
    pub persistence: &'static str,
    pub accepts_external_artifact_bytes: bool,
    pub loads_external_artifact: bool,
    pub maps_executable_pages: bool,
    pub writes_persistent_state: bool,
    pub authorizes_external_artifact_load: bool,
    pub authorizes_persistent_install: bool,
    pub authorizes_rollback_install: bool,
    pub signed_envelope: ArtifactIdentityEnvelope,
    pub text: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct DescriptorSourceTrustSelftestCase {
    pub name: &'static str,
    pub expected_accept: bool,
    pub actual_accept: bool,
    pub passed: bool,
    pub reason: &'static str,
}

const HELLO_LOAD_DESCRIPTOR_SOURCE_RECORD: DescriptorSourceRecord = DescriptorSourceRecord {
    schema: HELLO_LOAD_DESCRIPTOR_SCHEMA,
    id: HELLO_LOAD_DESCRIPTOR_ID,
    canonicalization: HELLO_LOAD_DESCRIPTOR_CANONICALIZATION,
    locator: HELLO_LOAD_DESCRIPTOR_SOURCE_LOCATOR,
    kind: HELLO_LOAD_DESCRIPTOR_SOURCE_KIND,
    binds_source_locator: None,
    binds_source_kind: None,
    binds_source_hash: None,
    service_id: HELLO_SERVICE_ID,
    artifact_id: HELLO_ARTIFACT_ID,
    artifact_kind: "builtin_stage0_test_service",
    scope: "current_boot",
    classification: "local_only",
    persistence: "none",
    accepts_external_artifact_bytes: false,
    loads_external_artifact: false,
    writes_persistent_state: false,
    signed_envelope: Some(HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_ENVELOPE),
    text: HELLO_LOAD_DESCRIPTOR_SOURCE,
};

const HELLO_HOST_BOUND_DESCRIPTOR_SOURCE_RECORD: DescriptorSourceRecord = DescriptorSourceRecord {
    schema: HELLO_LOAD_DESCRIPTOR_SCHEMA,
    id: HELLO_LOAD_DESCRIPTOR_ID,
    canonicalization: HELLO_LOAD_DESCRIPTOR_CANONICALIZATION,
    locator: HELLO_HOST_BOUND_DESCRIPTOR_SOURCE_LOCATOR,
    kind: HELLO_HOST_BOUND_DESCRIPTOR_SOURCE_KIND,
    binds_source_locator: Some(HELLO_LOAD_DESCRIPTOR_SOURCE_LOCATOR),
    binds_source_kind: Some(HELLO_LOAD_DESCRIPTOR_SOURCE_KIND),
    binds_source_hash: Some(HELLO_HOST_BOUND_CURRENT_IMAGE_SOURCE_HASH),
    service_id: HELLO_SERVICE_ID,
    artifact_id: HELLO_ARTIFACT_ID,
    artifact_kind: "builtin_stage0_test_service",
    scope: "current_boot",
    classification: "local_only",
    persistence: "none",
    accepts_external_artifact_bytes: false,
    loads_external_artifact: false,
    writes_persistent_state: false,
    signed_envelope: None,
    text: HELLO_HOST_BOUND_DESCRIPTOR_SOURCE,
};

const HELLO_BUILTIN_ARTIFACT_IDENTITY_RECORD: ArtifactIdentityRecord = ArtifactIdentityRecord {
    schema: HELLO_BUILTIN_ARTIFACT_IDENTITY_SCHEMA,
    id: HELLO_BUILTIN_ARTIFACT_IDENTITY_ID,
    canonicalization: HELLO_BUILTIN_ARTIFACT_IDENTITY_CANONICALIZATION,
    service_id: HELLO_SERVICE_ID,
    artifact_id: HELLO_ARTIFACT_ID,
    artifact_kind: "builtin_stage0_test_service",
    load_descriptor_id: HELLO_LOAD_DESCRIPTOR_ID,
    artifact_content_binding_schema: HELLO_BUILTIN_ARTIFACT_CONTENT_BINDING_SCHEMA,
    artifact_content_binding_id: HELLO_BUILTIN_ARTIFACT_CONTENT_BINDING_ID,
    artifact_content_kind: HELLO_BUILTIN_ARTIFACT_CONTENT_KIND,
    artifact_content_source_locator: HELLO_BUILTIN_ARTIFACT_CONTENT_SOURCE_LOCATOR,
    artifact_content_source_hash: HELLO_BUILTIN_ARTIFACT_CONTENT_SOURCE_HASH,
    artifact_content_binding_hash: HELLO_BUILTIN_ARTIFACT_CONTENT_BINDING_HASH,
    artifact_content_accepts_external_artifact_bytes: false,
    artifact_content_loads_external_artifact: false,
    artifact_content_maps_executable_pages: false,
    artifact_content_writes_persistent_state: false,
    artifact_reference_schema: HELLO_BUILTIN_ARTIFACT_REFERENCE_SCHEMA,
    artifact_reference_id: HELLO_BUILTIN_ARTIFACT_REFERENCE_ID,
    artifact_reference_kind: HELLO_BUILTIN_ARTIFACT_REFERENCE_KIND,
    artifact_reference_locator: HELLO_BUILTIN_ARTIFACT_REFERENCE_LOCATOR,
    artifact_reference_hash: HELLO_BUILTIN_ARTIFACT_REFERENCE_HASH,
    artifact_reference_bytes_hash: HELLO_BUILTIN_ARTIFACT_BYTES_HASH,
    artifact_reference_content_binding_hash: HELLO_BUILTIN_ARTIFACT_CONTENT_BINDING_HASH,
    artifact_reference_accepts_external_artifact_bytes: false,
    artifact_reference_loads_artifact_as_code: false,
    artifact_reference_maps_executable_pages: false,
    artifact_reference_writes_persistent_state: false,
    scope: "current_boot",
    classification: "local_only",
    persistence: "none",
    accepts_external_artifact_bytes: false,
    loads_external_artifact: false,
    maps_executable_pages: false,
    writes_persistent_state: false,
    authorizes_external_artifact_load: false,
    authorizes_persistent_install: false,
    authorizes_rollback_install: false,
    signed_envelope: HELLO_BUILTIN_ARTIFACT_IDENTITY_ENVELOPE,
    text: HELLO_BUILTIN_ARTIFACT_IDENTITY_SOURCE,
};

pub(crate) fn lookup_current_image_descriptor_source(
    descriptor_id: &str,
) -> Option<DescriptorSourceRecord> {
    if descriptor_id.eq_ignore_ascii_case(HELLO_LOAD_DESCRIPTOR_ID) {
        Some(HELLO_LOAD_DESCRIPTOR_SOURCE_RECORD)
    } else {
        None
    }
}

pub(crate) fn lookup_host_bound_descriptor_source(
    descriptor_id: &str,
) -> Option<DescriptorSourceRecord> {
    if descriptor_id.eq_ignore_ascii_case(HELLO_LOAD_DESCRIPTOR_ID) {
        Some(HELLO_HOST_BOUND_DESCRIPTOR_SOURCE_RECORD)
    } else {
        None
    }
}

pub(crate) const fn hello_builtin_artifact_identity() -> ArtifactIdentityRecord {
    HELLO_BUILTIN_ARTIFACT_IDENTITY_RECORD
}

pub(crate) fn validate_builtin_hello_artifact_identity(identity: ArtifactIdentityRecord) -> bool {
    key_value_text_is_canonical(identity.text)
        && identity.schema == HELLO_BUILTIN_ARTIFACT_IDENTITY_SCHEMA
        && identity.id == HELLO_BUILTIN_ARTIFACT_IDENTITY_ID
        && identity.canonicalization == HELLO_BUILTIN_ARTIFACT_IDENTITY_CANONICALIZATION
        && identity.service_id == HELLO_SERVICE_ID
        && identity.artifact_id == HELLO_ARTIFACT_ID
        && identity.artifact_kind == "builtin_stage0_test_service"
        && identity.load_descriptor_id == HELLO_LOAD_DESCRIPTOR_ID
        && identity.artifact_content_binding_schema == HELLO_BUILTIN_ARTIFACT_CONTENT_BINDING_SCHEMA
        && identity.artifact_content_binding_id == HELLO_BUILTIN_ARTIFACT_CONTENT_BINDING_ID
        && identity.artifact_content_kind == HELLO_BUILTIN_ARTIFACT_CONTENT_KIND
        && identity.artifact_content_source_locator == HELLO_BUILTIN_ARTIFACT_CONTENT_SOURCE_LOCATOR
        && identity.artifact_content_source_hash == HELLO_BUILTIN_ARTIFACT_CONTENT_SOURCE_HASH
        && identity.artifact_content_binding_hash == HELLO_BUILTIN_ARTIFACT_CONTENT_BINDING_HASH
        && identity.artifact_content_binding_hash
            == sha256_bytes(HELLO_BUILTIN_ARTIFACT_CONTENT_BINDING_TEXT.as_bytes())
        && !identity.artifact_content_accepts_external_artifact_bytes
        && !identity.artifact_content_loads_external_artifact
        && !identity.artifact_content_maps_executable_pages
        && !identity.artifact_content_writes_persistent_state
        && identity.artifact_reference_schema == HELLO_BUILTIN_ARTIFACT_REFERENCE_SCHEMA
        && identity.artifact_reference_id == HELLO_BUILTIN_ARTIFACT_REFERENCE_ID
        && identity.artifact_reference_kind == HELLO_BUILTIN_ARTIFACT_REFERENCE_KIND
        && identity.artifact_reference_locator == HELLO_BUILTIN_ARTIFACT_REFERENCE_LOCATOR
        && identity.artifact_reference_hash == HELLO_BUILTIN_ARTIFACT_REFERENCE_HASH
        && identity.artifact_reference_hash
            == sha256_bytes(HELLO_BUILTIN_ARTIFACT_REFERENCE_TEXT.as_bytes())
        && identity.artifact_reference_bytes_hash == HELLO_BUILTIN_ARTIFACT_BYTES_HASH
        && identity.artifact_reference_content_binding_hash
            == identity.artifact_content_binding_hash
        && !identity.artifact_reference_accepts_external_artifact_bytes
        && !identity.artifact_reference_loads_artifact_as_code
        && !identity.artifact_reference_maps_executable_pages
        && !identity.artifact_reference_writes_persistent_state
        && identity.scope == "current_boot"
        && identity.classification == "local_only"
        && identity.persistence == "none"
        && !identity.accepts_external_artifact_bytes
        && !identity.loads_external_artifact
        && !identity.maps_executable_pages
        && !identity.writes_persistent_state
        && !identity.authorizes_external_artifact_load
        && !identity.authorizes_persistent_install
        && !identity.authorizes_rollback_install
        && text_field_eq(identity.text, "canonicalization", identity.canonicalization)
        && text_field_eq(identity.text, "schema", identity.schema)
        && text_field_eq(identity.text, "id", identity.id)
        && text_field_eq(identity.text, "service_id", identity.service_id)
        && text_field_eq(identity.text, "artifact_id", identity.artifact_id)
        && text_field_eq(identity.text, "artifact_kind", identity.artifact_kind)
        && text_field_eq(
            identity.text,
            "load_descriptor_id",
            identity.load_descriptor_id,
        )
        && text_field_eq(
            identity.text,
            "artifact_content_binding_schema",
            identity.artifact_content_binding_schema,
        )
        && text_field_eq(
            identity.text,
            "artifact_content_binding_id",
            identity.artifact_content_binding_id,
        )
        && text_field_eq(
            identity.text,
            "artifact_content_kind",
            identity.artifact_content_kind,
        )
        && text_field_eq(
            identity.text,
            "artifact_content_source_locator",
            identity.artifact_content_source_locator,
        )
        && text_sha256_field(identity.text, "artifact_content_source_sha256")
            == Some(identity.artifact_content_source_hash)
        && text_sha256_field(identity.text, "artifact_content_binding_sha256")
            == Some(identity.artifact_content_binding_hash)
        && text_field_eq(
            identity.text,
            "artifact_content_accepts_external_artifact_bytes",
            "false",
        )
        && text_field_eq(
            identity.text,
            "artifact_content_loads_external_artifact",
            "false",
        )
        && text_field_eq(
            identity.text,
            "artifact_content_maps_executable_pages",
            "false",
        )
        && text_field_eq(
            identity.text,
            "artifact_content_writes_persistent_state",
            "false",
        )
        && text_field_eq(
            identity.text,
            "artifact_reference_schema",
            identity.artifact_reference_schema,
        )
        && text_field_eq(
            identity.text,
            "artifact_reference_id",
            identity.artifact_reference_id,
        )
        && text_field_eq(
            identity.text,
            "artifact_reference_kind",
            identity.artifact_reference_kind,
        )
        && text_field_eq(
            identity.text,
            "artifact_reference_locator",
            identity.artifact_reference_locator,
        )
        && text_sha256_field(identity.text, "artifact_reference_sha256")
            == Some(identity.artifact_reference_hash)
        && text_sha256_field(identity.text, "artifact_reference_bytes_sha256")
            == Some(identity.artifact_reference_bytes_hash)
        && text_sha256_field(identity.text, "artifact_reference_content_binding_sha256")
            == Some(identity.artifact_reference_content_binding_hash)
        && text_field_eq(
            identity.text,
            "artifact_reference_accepts_external_artifact_bytes",
            "false",
        )
        && text_field_eq(
            identity.text,
            "artifact_reference_loads_artifact_as_code",
            "false",
        )
        && text_field_eq(
            identity.text,
            "artifact_reference_maps_executable_pages",
            "false",
        )
        && text_field_eq(
            identity.text,
            "artifact_reference_writes_persistent_state",
            "false",
        )
        && text_field_eq(identity.text, "scope", identity.scope)
        && text_field_eq(identity.text, "classification", identity.classification)
        && text_field_eq(identity.text, "persistence", identity.persistence)
        && text_field_eq(identity.text, "accepts_external_artifact_bytes", "false")
        && text_field_eq(identity.text, "loads_external_artifact", "false")
        && text_field_eq(identity.text, "maps_executable_pages", "false")
        && text_field_eq(identity.text, "writes_persistent_state", "false")
        && text_field_eq(identity.text, "authorizes_external_artifact_load", "false")
        && text_field_eq(identity.text, "authorizes_persistent_install", "false")
        && text_field_eq(identity.text, "authorizes_rollback_install", "false")
        && validate_artifact_identity_envelope(identity)
}

pub(crate) fn validate_current_image_descriptor_source(source: DescriptorSourceRecord) -> bool {
    validate_common_descriptor_source(source)
        && source.locator == HELLO_LOAD_DESCRIPTOR_SOURCE_LOCATOR
        && source.kind == HELLO_LOAD_DESCRIPTOR_SOURCE_KIND
        && source_field_eq(
            source,
            "source_locator",
            HELLO_LOAD_DESCRIPTOR_SOURCE_LOCATOR,
        )
        && source_field_eq(source, "source_kind", HELLO_LOAD_DESCRIPTOR_SOURCE_KIND)
        && source.binds_source_locator.is_none()
        && source.binds_source_kind.is_none()
        && source.binds_source_hash.is_none()
        && source_field(source, "binds_source_locator").is_none()
        && source_field(source, "binds_source_kind").is_none()
        && source_field(source, "binds_source_hash").is_none()
        && validate_descriptor_source_envelope(source)
}

pub(crate) fn validate_host_bound_descriptor_source(source: DescriptorSourceRecord) -> bool {
    let current_image_hash = descriptor_source_hash(HELLO_LOAD_DESCRIPTOR_SOURCE_RECORD);
    validate_common_descriptor_source(source)
        && source.locator == HELLO_HOST_BOUND_DESCRIPTOR_SOURCE_LOCATOR
        && source.kind == HELLO_HOST_BOUND_DESCRIPTOR_SOURCE_KIND
        && source_field_eq(
            source,
            "source_locator",
            HELLO_HOST_BOUND_DESCRIPTOR_SOURCE_LOCATOR,
        )
        && source_field_eq(
            source,
            "source_kind",
            HELLO_HOST_BOUND_DESCRIPTOR_SOURCE_KIND,
        )
        && source.binds_source_locator == Some(HELLO_LOAD_DESCRIPTOR_SOURCE_LOCATOR)
        && source.binds_source_kind == Some(HELLO_LOAD_DESCRIPTOR_SOURCE_KIND)
        && source.binds_source_hash == Some(current_image_hash)
        && source_field_eq(
            source,
            "binds_source_locator",
            HELLO_LOAD_DESCRIPTOR_SOURCE_LOCATOR,
        )
        && source_field_eq(
            source,
            "binds_source_kind",
            HELLO_LOAD_DESCRIPTOR_SOURCE_KIND,
        )
        && source_sha256_field(source, "binds_source_hash") == Some(current_image_hash)
}

pub(crate) fn validate_descriptor_source(source: DescriptorSourceRecord) -> bool {
    validate_current_image_descriptor_source(source)
        || validate_host_bound_descriptor_source(source)
}

pub(crate) fn verify_descriptor_source_envelope_parts(
    envelope: Option<DescriptorSourceEnvelope>,
    locator: &str,
    kind: &str,
    text: &str,
) -> bool {
    let Some(envelope) = envelope else {
        return false;
    };
    validate_descriptor_source_envelope_parts(envelope, locator, kind, text)
}

pub(crate) fn verify_artifact_identity_envelope_parts(
    envelope: ArtifactIdentityEnvelope,
    identity_id: &str,
    artifact_id: &str,
    text: &str,
) -> bool {
    validate_artifact_identity_envelope_parts(envelope, identity_id, artifact_id, text)
}

pub(crate) fn hello_descriptor_source_trust_selftest_hash() -> [u8; 32] {
    sha256_bytes(
        b"schema=raios.descriptor_source_trust_selftest.v0\n\
id=descriptor_source_trust_selftest.current_image.svc.demo.hello.v0\n\
cases=valid_current_image,tampered_payload,tampered_locator_kind,tampered_public_key_hash,tampered_signature",
    )
}

pub(crate) fn hello_descriptor_source_trust_selftest_cases(
) -> [DescriptorSourceTrustSelftestCase; HELLO_DESCRIPTOR_SOURCE_TRUST_SELFTEST_CASES] {
    let source = HELLO_LOAD_DESCRIPTOR_SOURCE_RECORD;
    let envelope = HELLO_CURRENT_IMAGE_DESCRIPTOR_SOURCE_ENVELOPE;
    let mut bad_key_hash = envelope;
    bad_key_hash.public_key_hash = [0u8; 32];
    let bad_signature = b"not-a-valid-der-signature";
    let mut bad_signature_envelope = envelope;
    bad_signature_envelope.signature_der = bad_signature;
    bad_signature_envelope.signature_hash = sha256_bytes(bad_signature);

    [
        trust_case(
            "valid_current_image_envelope",
            true,
            validate_current_image_descriptor_source(source),
            "accepted_verified_current_image_descriptor_source",
        ),
        trust_case(
            "tampered_payload_denied",
            false,
            verify_descriptor_source_envelope_parts(
                Some(envelope),
                HELLO_LOAD_DESCRIPTOR_SOURCE_LOCATOR,
                HELLO_LOAD_DESCRIPTOR_SOURCE_KIND,
                "tampered_descriptor_source_payload",
            ),
            "payload_hash_or_signature_must_match_source_text",
        ),
        trust_case(
            "tampered_locator_kind_denied",
            false,
            verify_descriptor_source_envelope_parts(
                Some(envelope),
                "tampered.descriptor_source.svc.demo.hello.v0",
                "tampered_descriptor_source",
                HELLO_LOAD_DESCRIPTOR_SOURCE,
            ),
            "envelope_must_bind_source_locator_and_kind",
        ),
        trust_case(
            "tampered_public_key_hash_denied",
            false,
            verify_descriptor_source_envelope_parts(
                Some(bad_key_hash),
                HELLO_LOAD_DESCRIPTOR_SOURCE_LOCATOR,
                HELLO_LOAD_DESCRIPTOR_SOURCE_KIND,
                HELLO_LOAD_DESCRIPTOR_SOURCE,
            ),
            "public_key_hash_must_match_envelope_key",
        ),
        trust_case(
            "tampered_signature_denied",
            false,
            verify_descriptor_source_envelope_parts(
                Some(bad_signature_envelope),
                HELLO_LOAD_DESCRIPTOR_SOURCE_LOCATOR,
                HELLO_LOAD_DESCRIPTOR_SOURCE_KIND,
                HELLO_LOAD_DESCRIPTOR_SOURCE,
            ),
            "signature_must_verify_payload",
        ),
    ]
}

pub(crate) fn hello_artifact_reference_trust_selftest_hash() -> [u8; 32] {
    sha256_bytes(
        b"schema=raios.builtin_artifact_reference_trust_selftest.v0\n\
id=artifact_reference_trust_selftest.builtin.svc.demo.hello.v0\n\
cases=valid_reference,tampered_artifact_bytes_hash,tampered_content_binding_hash,tampered_reference_hash,tampered_trust_payload_hash",
    )
}

pub(crate) fn hello_artifact_reference_trust_selftest_cases(
) -> [DescriptorSourceTrustSelftestCase; HELLO_ARTIFACT_REFERENCE_TRUST_SELFTEST_CASES] {
    let identity = HELLO_BUILTIN_ARTIFACT_IDENTITY_RECORD;
    let mut bad_bytes = identity;
    bad_bytes.artifact_reference_bytes_hash = [0x11; 32];
    let mut bad_content_binding = identity;
    bad_content_binding.artifact_reference_content_binding_hash = [0x22; 32];
    let mut bad_reference_hash = identity;
    bad_reference_hash.artifact_reference_hash = [0x33; 32];
    let mut bad_trust_linkage = identity;
    bad_trust_linkage.signed_envelope.payload_hash = [0x44; 32];

    [
        trust_case(
            "valid_builtin_artifact_reference",
            true,
            validate_builtin_hello_artifact_identity(identity),
            "accepted_verified_builtin_artifact_reference",
        ),
        trust_case(
            "tampered_artifact_bytes_hash_denied",
            false,
            validate_builtin_hello_artifact_identity(bad_bytes),
            "artifact_bytes_hash_must_match_signed_reference",
        ),
        trust_case(
            "tampered_content_binding_hash_denied",
            false,
            validate_builtin_hello_artifact_identity(bad_content_binding),
            "artifact_reference_must_bind_content_binding_hash",
        ),
        trust_case(
            "tampered_reference_hash_denied",
            false,
            validate_builtin_hello_artifact_identity(bad_reference_hash),
            "artifact_reference_hash_must_match_reference_text",
        ),
        trust_case(
            "tampered_trust_payload_hash_denied",
            false,
            validate_builtin_hello_artifact_identity(bad_trust_linkage),
            "artifact_identity_trust_envelope_must_bind_payload_hash",
        ),
    ]
}

fn validate_common_descriptor_source(source: DescriptorSourceRecord) -> bool {
    source_text_is_canonical_key_value(source)
        && source.schema == HELLO_LOAD_DESCRIPTOR_SCHEMA
        && source.id == HELLO_LOAD_DESCRIPTOR_ID
        && source.canonicalization == HELLO_LOAD_DESCRIPTOR_CANONICALIZATION
        && source.service_id == HELLO_SERVICE_ID
        && source.artifact_id == HELLO_ARTIFACT_ID
        && source.artifact_kind == "builtin_stage0_test_service"
        && source.scope == "current_boot"
        && source.classification == "local_only"
        && source.persistence == "none"
        && !source.accepts_external_artifact_bytes
        && !source.loads_external_artifact
        && !source.writes_persistent_state
        && source_field_eq(source, "canonicalization", source.canonicalization)
        && source_field_eq(source, "schema", source.schema)
        && source_field_eq(source, "id", source.id)
        && source_field_eq(source, "service_id", source.service_id)
        && source_field_eq(source, "artifact_id", source.artifact_id)
        && source_field_eq(source, "artifact_kind", source.artifact_kind)
        && source_field_eq(source, "scope", source.scope)
        && source_field_eq(source, "classification", source.classification)
        && source_field_eq(source, "persistence", source.persistence)
        && source_field_eq(source, "accepts_external_artifact_bytes", "false")
        && source_field_eq(source, "loads_external_artifact", "false")
        && source_field_eq(source, "writes_persistent_state", "false")
}

fn source_text_is_canonical_key_value(source: DescriptorSourceRecord) -> bool {
    key_value_text_is_canonical(source.text)
}

fn key_value_text_is_canonical(text: &str) -> bool {
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some(eq) = line.find('=') else {
            return false;
        };
        let key = line[..eq].trim();
        let value = line[eq + 1..].trim();
        if key.is_empty()
            || value.is_empty()
            || !key
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return false;
        }
    }
    true
}

fn source_field(source: DescriptorSourceRecord, key: &str) -> Option<&'static str> {
    text_field(source.text, key)
}

fn text_field(text: &'static str, key: &str) -> Option<&'static str> {
    let mut found = None;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let eq = line.find('=')?;
        let candidate = line[..eq].trim();
        if candidate == key {
            if found.is_some() {
                return None;
            }
            found = Some(line[eq + 1..].trim());
        }
    }
    found
}

fn source_field_eq(source: DescriptorSourceRecord, key: &str, expected: &str) -> bool {
    source_field(source, key) == Some(expected)
}

fn text_field_eq(text: &'static str, key: &str, expected: &str) -> bool {
    text_field(text, key) == Some(expected)
}

fn text_sha256_field(text: &'static str, key: &str) -> Option<[u8; 32]> {
    text_field(text, key).and_then(parse_sha256_ref)
}

fn source_sha256_field(source: DescriptorSourceRecord, key: &str) -> Option<[u8; 32]> {
    source_field(source, key).and_then(parse_sha256_ref)
}

fn trust_case(
    name: &'static str,
    expected_accept: bool,
    actual_accept: bool,
    reason: &'static str,
) -> DescriptorSourceTrustSelftestCase {
    DescriptorSourceTrustSelftestCase {
        name,
        expected_accept,
        actual_accept,
        passed: expected_accept == actual_accept,
        reason,
    }
}

fn validate_descriptor_source_envelope(source: DescriptorSourceRecord) -> bool {
    verify_descriptor_source_envelope_parts(
        source.signed_envelope,
        source.locator,
        source.kind,
        source.text,
    )
}

fn validate_artifact_identity_envelope(identity: ArtifactIdentityRecord) -> bool {
    verify_artifact_identity_envelope_parts(
        identity.signed_envelope,
        identity.id,
        identity.artifact_id,
        identity.text,
    )
}

fn validate_descriptor_source_envelope_parts(
    envelope: DescriptorSourceEnvelope,
    locator: &str,
    kind: &str,
    text: &str,
) -> bool {
    envelope.schema == "raios.descriptor_source_signature_envelope.v0"
        && envelope.id == "descriptor_source_signature.current_image.svc.demo.hello.v0"
        && envelope.algorithm == "ecdsa_p256_sha256_asn1_der"
        && envelope.verification_phase == "runtime_before_descriptor_selection"
        && envelope.trust_scope == "current_boot_repo_descriptor_source_candidate"
        && envelope.payload_source_locator == locator
        && envelope.payload_source_kind == kind
        && envelope.payload_hash == sha256_bytes(text.as_bytes())
        && envelope.envelope_hash == sha256_bytes(envelope.text.as_bytes())
        && envelope.public_key_hash == sha256_bytes(envelope.public_key_sec1)
        && envelope.signature_hash == sha256_bytes(envelope.signature_der)
        && !envelope.authorizes_external_artifact_load
        && !envelope.authorizes_persistent_install
        && verify_p256_signature(
            envelope.public_key_sec1,
            envelope.signature_der,
            text.as_bytes(),
        )
}

fn validate_artifact_identity_envelope_parts(
    envelope: ArtifactIdentityEnvelope,
    identity_id: &str,
    artifact_id: &str,
    text: &str,
) -> bool {
    envelope.schema == "raios.builtin_artifact_identity_signature_envelope.v0"
        && envelope.id == "artifact_identity_signature.builtin.svc.demo.hello.v0"
        && envelope.algorithm == "ecdsa_p256_sha256_asn1_der"
        && envelope.verification_phase == "runtime_before_builtin_artifact_selection"
        && envelope.trust_scope == "current_boot_builtin_artifact_identity_candidate"
        && envelope.payload_identity_id == identity_id
        && envelope.payload_artifact_id == artifact_id
        && envelope.payload_hash == sha256_bytes(text.as_bytes())
        && envelope.envelope_hash == sha256_bytes(envelope.text.as_bytes())
        && envelope.public_key_hash == sha256_bytes(envelope.public_key_sec1)
        && envelope.signature_hash == sha256_bytes(envelope.signature_der)
        && !envelope.authorizes_external_artifact_load
        && !envelope.authorizes_persistent_install
        && !envelope.authorizes_rollback_install
        && verify_p256_signature(
            envelope.public_key_sec1,
            envelope.signature_der,
            text.as_bytes(),
        )
}

fn verify_p256_signature(public_key_sec1: &[u8], signature_der: &[u8], payload: &[u8]) -> bool {
    let Ok(verifying_key) = VerifyingKey::from_sec1_bytes(public_key_sec1) else {
        return false;
    };
    let Ok(signature) = Signature::from_der(signature_der) else {
        return false;
    };
    verifying_key.verify(payload, &signature).is_ok()
}

pub(crate) fn descriptor_source_hash(source: DescriptorSourceRecord) -> [u8; 32] {
    sha256_bytes(source.text.as_bytes())
}

pub(crate) fn artifact_identity_hash(identity: ArtifactIdentityRecord) -> [u8; 32] {
    sha256_bytes(identity.text.as_bytes())
}

pub(crate) fn artifact_content_binding_hash(identity: ArtifactIdentityRecord) -> [u8; 32] {
    identity.artifact_content_binding_hash
}

pub(crate) fn artifact_reference_hash(identity: ArtifactIdentityRecord) -> [u8; 32] {
    identity.artifact_reference_hash
}

pub(crate) fn artifact_reference_bytes_hash(identity: ArtifactIdentityRecord) -> [u8; 32] {
    identity.artifact_reference_bytes_hash
}

fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(bytes);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

pub(crate) fn descriptor_source_hash_for_locator(locator: &str) -> Option<[u8; 32]> {
    if locator.eq_ignore_ascii_case(HELLO_LOAD_DESCRIPTOR_SOURCE_LOCATOR) {
        Some(descriptor_source_hash(HELLO_LOAD_DESCRIPTOR_SOURCE_RECORD))
    } else if locator.eq_ignore_ascii_case(HELLO_HOST_BOUND_DESCRIPTOR_SOURCE_LOCATOR) {
        Some(descriptor_source_hash(
            HELLO_HOST_BOUND_DESCRIPTOR_SOURCE_RECORD,
        ))
    } else {
        None
    }
}

pub(crate) fn hello_load_descriptor_source_hash() -> [u8; 32] {
    descriptor_source_hash(HELLO_LOAD_DESCRIPTOR_SOURCE_RECORD)
}
