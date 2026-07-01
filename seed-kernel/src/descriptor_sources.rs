use sha2::{Digest, Sha256};

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
    "canonicalization=raios.current_boot_load_descriptor.canonical.v0\n\
schema=raios.current_boot_load_descriptor.v0\n\
id=load_descriptor.current_boot.svc.demo.hello.v0\n\
source_kind=current_image_descriptor_source\n\
source_locator=current_image.descriptor_source.svc.demo.hello.v0\n\
service_id=svc.demo.hello\n\
artifact_id=builtin:svc.demo.hello\n\
artifact_kind=builtin_stage0_test_service\n\
scope=current_boot\n\
classification=local_only\n\
persistence=none\n\
accepts_external_artifact_bytes=false\n\
loads_external_artifact=false\n\
writes_persistent_state=false";

#[derive(Clone, Copy)]
pub(crate) struct CurrentImageDescriptorSource {
    pub schema: &'static str,
    pub id: &'static str,
    pub canonicalization: &'static str,
    pub locator: &'static str,
    pub kind: &'static str,
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

const HELLO_LOAD_DESCRIPTOR_SOURCE_RECORD: CurrentImageDescriptorSource =
    CurrentImageDescriptorSource {
        schema: HELLO_LOAD_DESCRIPTOR_SCHEMA,
        id: HELLO_LOAD_DESCRIPTOR_ID,
        canonicalization: HELLO_LOAD_DESCRIPTOR_CANONICALIZATION,
        locator: HELLO_LOAD_DESCRIPTOR_SOURCE_LOCATOR,
        kind: HELLO_LOAD_DESCRIPTOR_SOURCE_KIND,
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

pub(crate) fn lookup_current_image_descriptor_source(
    descriptor_id: &str,
) -> Option<CurrentImageDescriptorSource> {
    // ponytail: one current-image record; add a table when a second descriptor exists.
    if descriptor_id.eq_ignore_ascii_case(HELLO_LOAD_DESCRIPTOR_ID) {
        Some(HELLO_LOAD_DESCRIPTOR_SOURCE_RECORD)
    } else {
        None
    }
}

pub(crate) fn validate_current_image_descriptor_source(
    source: CurrentImageDescriptorSource,
) -> bool {
    source.schema == HELLO_LOAD_DESCRIPTOR_SCHEMA
        && source.id == HELLO_LOAD_DESCRIPTOR_ID
        && source.canonicalization == HELLO_LOAD_DESCRIPTOR_CANONICALIZATION
        && source.locator == HELLO_LOAD_DESCRIPTOR_SOURCE_LOCATOR
        && source.kind == HELLO_LOAD_DESCRIPTOR_SOURCE_KIND
        && source.service_id == HELLO_SERVICE_ID
        && source.artifact_id == HELLO_ARTIFACT_ID
        && source.artifact_kind == "builtin_stage0_test_service"
        && source.scope == "current_boot"
        && source.classification == "local_only"
        && source.persistence == "none"
        && !source.accepts_external_artifact_bytes
        && !source.loads_external_artifact
        && !source.writes_persistent_state
        && source.text == HELLO_LOAD_DESCRIPTOR_SOURCE
}

pub(crate) fn descriptor_source_hash(source: CurrentImageDescriptorSource) -> [u8; 32] {
    let digest = Sha256::digest(source.text.as_bytes());
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

pub(crate) fn hello_load_descriptor_source_hash() -> [u8; 32] {
    descriptor_source_hash(HELLO_LOAD_DESCRIPTOR_SOURCE_RECORD)
}
