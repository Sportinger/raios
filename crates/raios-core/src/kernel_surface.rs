//! M11 starting baseline for the kernel internet-parsing surface.
//!
//! This module is pure measurement data. It grants no authority, performs no
//! live file-size checks, and makes no kernel/service decision.

use alloc::{vec, vec::Vec};

use crate::record::{sha256_of_json, Field, Value};

pub const M11_KERNEL_SURFACE_BASELINE_SCHEMA: &str = "raios.kernel_surface_baseline.v0";
pub const M11_KERNEL_SURFACE_BASELINE_ID: &str = "kernel_surface_baseline.m11_start.v0";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelSurfaceClass {
    PermanentCore,
    MixedCoreAndCandidate,
    ServiceCandidate,
    VendorServiceCandidate,
}

impl KernelSurfaceClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PermanentCore => "permanent_core",
            Self::MixedCoreAndCandidate => "mixed_core_and_candidate",
            Self::ServiceCandidate => "service_candidate",
            Self::VendorServiceCandidate => "vendor_service_candidate",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KernelSurfaceEntry<'a> {
    pub module: &'a str,
    pub loc: u32,
    pub class: KernelSurfaceClass,
    pub role: &'a str,
    pub source_ranges: &'a str,
}

pub const M11_KERNEL_SURFACE_BASELINE: &[KernelSurfaceEntry<'static>] = &[
    KernelSurfaceEntry {
        module: "seed-kernel/src/openai.rs",
        loc: 1624,
        class: KernelSurfaceClass::ServiceCandidate,
        role: "HTTP/JSON/provider glue",
        source_ranges: "1150-1284 TLS/HTTP; 1322-1597 parser",
    },
    KernelSurfaceEntry {
        module: "seed-kernel/src/openai_trust.rs",
        loc: 342,
        class: KernelSurfaceClass::MixedCoreAndCandidate,
        role: "TLS trust verifier/parser",
        source_ranges: "40-115 TLS trust parser",
    },
    KernelSurfaceEntry {
        module: "seed-kernel/src/tls_io.rs",
        loc: 105,
        class: KernelSurfaceClass::ServiceCandidate,
        role: "TCP/TLS bridge seam",
        source_ranges: "37-99 TCP/TLS bridge",
    },
    KernelSurfaceEntry {
        module: "seed-kernel/src/net.rs",
        loc: 756,
        class: KernelSurfaceClass::MixedCoreAndCandidate,
        role: "L2-L4 core + DNS parser candidate 579-735",
        source_ranges: "579-735 DNS parser; rest L2-L4 core",
    },
    KernelSurfaceEntry {
        module: "seed-kernel/src/wasm_runtime.rs",
        loc: 737,
        class: KernelSurfaceClass::PermanentCore,
        role: "Wasm isolation runtime",
        source_ranges: "281-360 runtime; 680-685 global imports",
    },
    KernelSurfaceEntry {
        module: "vendor/embedded-tls-0.17.0/src",
        loc: 6813,
        class: KernelSurfaceClass::VendorServiceCandidate,
        role: "vendored TLS dependency",
        source_ranges: "vendor source tree; 48 files",
    },
];

/// Returns file-level candidate-bearing LOC.
///
/// Mixed core/candidate modules are counted at full file size because this
/// baseline is deterministic measurement data, not a removable-portion claim.
pub fn service_candidate_loc_total() -> u32 {
    M11_KERNEL_SURFACE_BASELINE
        .iter()
        .filter(|entry| {
            matches!(
                entry.class,
                KernelSurfaceClass::MixedCoreAndCandidate
                    | KernelSurfaceClass::ServiceCandidate
                    | KernelSurfaceClass::VendorServiceCandidate
            )
        })
        .map(|entry| entry.loc)
        .sum()
}

pub fn permanent_core_loc_total() -> u32 {
    M11_KERNEL_SURFACE_BASELINE
        .iter()
        .filter(|entry| entry.class == KernelSurfaceClass::PermanentCore)
        .map(|entry| entry.loc)
        .sum()
}

pub fn grand_total_loc() -> u32 {
    M11_KERNEL_SURFACE_BASELINE
        .iter()
        .map(|entry| entry.loc)
        .sum()
}

pub fn baseline_record_value() -> Value<'static> {
    let mut entries = Vec::new();
    for entry in M11_KERNEL_SURFACE_BASELINE {
        entries.push(entry_record_value(entry));
    }

    Value::Object(vec![
        Field::new("schema", Value::Str(M11_KERNEL_SURFACE_BASELINE_SCHEMA)),
        Field::new("id", Value::Str(M11_KERNEL_SURFACE_BASELINE_ID)),
        Field::new("entries", Value::Array(entries)),
    ])
}

pub fn baseline_record_sha256() -> [u8; 32] {
    sha256_of_json(&baseline_record_value())
}

fn entry_record_value<'a>(entry: &KernelSurfaceEntry<'a>) -> Value<'a> {
    Value::Object(vec![
        Field::new("module", Value::Str(entry.module)),
        Field::new("loc", Value::U64(entry.loc as u64)),
        Field::new("class", Value::Str(entry.class.as_str())),
        Field::new("role", Value::Str(entry.role)),
        Field::new("source_ranges", Value::Str(entry.source_ranges)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_lists_expected_modules_and_classes() {
        let expected = [
            (
                "seed-kernel/src/openai.rs",
                1624,
                KernelSurfaceClass::ServiceCandidate,
            ),
            (
                "seed-kernel/src/openai_trust.rs",
                342,
                KernelSurfaceClass::MixedCoreAndCandidate,
            ),
            (
                "seed-kernel/src/tls_io.rs",
                105,
                KernelSurfaceClass::ServiceCandidate,
            ),
            (
                "seed-kernel/src/net.rs",
                756,
                KernelSurfaceClass::MixedCoreAndCandidate,
            ),
            (
                "seed-kernel/src/wasm_runtime.rs",
                737,
                KernelSurfaceClass::PermanentCore,
            ),
            (
                "vendor/embedded-tls-0.17.0/src",
                6813,
                KernelSurfaceClass::VendorServiceCandidate,
            ),
        ];

        assert_eq!(M11_KERNEL_SURFACE_BASELINE.len(), expected.len());
        for (entry, (module, loc, class)) in M11_KERNEL_SURFACE_BASELINE.iter().zip(expected) {
            assert_eq!(entry.module, module);
            assert_eq!(entry.loc, loc);
            assert_eq!(entry.class, class);
        }
    }

    #[test]
    fn grand_total_matches_recorded_entry_sum() {
        let recorded_sum: u32 = M11_KERNEL_SURFACE_BASELINE
            .iter()
            .map(|entry| entry.loc)
            .sum();

        assert_eq!(grand_total_loc(), recorded_sum);
        assert_eq!(grand_total_loc(), 10_377);
    }

    #[test]
    fn candidate_bearing_accounting_is_file_level_and_deterministic() {
        assert_eq!(service_candidate_loc_total(), 1624 + 342 + 105 + 756 + 6813);
        assert_eq!(permanent_core_loc_total(), 737);
        assert_eq!(
            service_candidate_loc_total() + permanent_core_loc_total(),
            grand_total_loc()
        );
    }

    #[test]
    fn baseline_record_sha256_is_deterministic_and_nonzero() {
        let first = baseline_record_sha256();
        let second = baseline_record_sha256();

        assert_eq!(first, second);
        assert_ne!(first, [0u8; 32]);
    }

    #[test]
    fn kernel_surface_is_pure_measurement_grants_nothing() {
        assert_eq!(M11_KERNEL_SURFACE_BASELINE.len(), 6);
        assert_eq!(grand_total_loc(), 10_377);

        let record = baseline_record_value();
        assert!(field_keys_avoid_authority_language(&record));
    }

    fn field_keys_avoid_authority_language(value: &Value<'_>) -> bool {
        match value {
            Value::Object(fields) | Value::InlineObject(fields) => fields.iter().all(|field| {
                !mentions_authority(field.key) && field_keys_avoid_authority_language(&field.value)
            }),
            Value::Array(values) | Value::InlineArray(values) => {
                values.iter().all(field_keys_avoid_authority_language)
            }
            _ => true,
        }
    }

    fn mentions_authority(key: &str) -> bool {
        key.contains("authoriz")
            || key.contains("capability")
            || key.contains("grant")
            || key.contains("transmission")
            || key.contains("write")
    }
}
