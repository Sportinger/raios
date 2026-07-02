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
pub(crate) const HELLO_DESCRIPTOR_SOURCE_TRUST_SELFTEST_ID: &str =
    "descriptor_source_trust_selftest.current_image.svc.demo.hello.v0";
pub(crate) const HELLO_DESCRIPTOR_SOURCE_TRUST_SELFTEST_CASES: usize = 5;

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
    for line in source.text.lines() {
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
    let mut found = None;
    for line in source.text.lines() {
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
