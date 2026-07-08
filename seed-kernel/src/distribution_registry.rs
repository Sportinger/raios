use raios_core::{
    distribution_provenance::PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SHA256,
    distribution_registry::{
        DistributionRegistryEntry, DistributionRegistryEntryInput, RegistrySelectionReason,
    },
};

use crate::wasm_runtime;

pub(crate) const BUILTIN_ECHO_REGISTRY_ENTRY_ID: &str = "builtin.svc.demo.echo";

pub(crate) const BUILTIN_ECHO_PROVENANCE_SIGNATURE_DER: &[u8] = &[
    0x30, 0x44, 0x02, 0x20, 0x1f, 0xd9, 0xaa, 0x3e, 0x26, 0x57, 0x9a, 0xb9, 0x85, 0x2a, 0x1e, 0xa6,
    0x1a, 0x7f, 0xe2, 0x3f, 0x79, 0xc3, 0x9b, 0xad, 0xd1, 0x3e, 0x2c, 0x74, 0xdb, 0xdf, 0x9d, 0x95,
    0x7a, 0x25, 0x44, 0x9b, 0x02, 0x20, 0x4f, 0x78, 0x31, 0x91, 0x89, 0x4c, 0xfb, 0x60, 0x9d, 0x35,
    0xc5, 0xba, 0xbc, 0x9f, 0xb3, 0x20, 0x8e, 0x77, 0xd9, 0xc7, 0x12, 0xd2, 0x64, 0x5a, 0x63, 0x31,
    0x20, 0xdb, 0x6d, 0xcd, 0xd8, 0x9b,
];

pub(crate) fn builtin_echo_entry(
) -> Result<DistributionRegistryEntry<'static>, RegistrySelectionReason> {
    DistributionRegistryEntry::new(DistributionRegistryEntryInput {
        entry_id: BUILTIN_ECHO_REGISTRY_ENTRY_ID,
        artifact_bytes: wasm_runtime::ECHO_WASM_ARTIFACT_BYTES,
        claimed_artifact_sha256: wasm_runtime::ECHO_WASM_ARTIFACT_BYTES_HASH,
        provenance_signature_der: Some(BUILTIN_ECHO_PROVENANCE_SIGNATURE_DER),
        publisher_key_sha256: PLACEHOLDER_DISTRIBUTION_PUBLISHER_PUBLIC_KEY_SHA256,
        classification: "local_only",
    })
}
