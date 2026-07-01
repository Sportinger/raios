use crate::agent_protocol_support::parse_sha256_ref;
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
    pub text: &'static str,
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

pub(crate) fn descriptor_source_hash(source: DescriptorSourceRecord) -> [u8; 32] {
    let digest = Sha256::digest(source.text.as_bytes());
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
