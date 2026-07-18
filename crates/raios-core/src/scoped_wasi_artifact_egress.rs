//! Fail-closed double-build gate for inert WASI build artifact egress.
//!
//! This slice only creates a typed plan. It has no persistence callback and no
//! artifact-store access; those effects belong to a later scoped adapter.

use alloc::vec::Vec;

use crate::wasi_build_output::{
    parse_output_manifest_sha256, WasiBuildOutputChunk, WasiBuildOutputHashField,
    WasiBuildOutputRun,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopedWasiArtifactEgress<'a> {
    pub run_one: WasiBuildOutputRun<'a>,
    pub run_two: WasiBuildOutputRun<'a>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasiArtifactEgressRejection {
    MalformedHash {
        field: WasiBuildOutputHashField,
    },
    BuildFailed {
        run_one_exit_status: i32,
        run_two_exit_status: i32,
    },
    BuildOutputsNotReproducible,
}

impl WasiArtifactEgressRejection {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::MalformedHash { .. } => "malformed_hash",
            Self::BuildFailed { .. } => "build_failed",
            Self::BuildOutputsNotReproducible => "build_outputs_not_reproducible",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasiArtifactEgressPlan {
    pub output_manifest_sha256: [u8; 32],
    pub chunks: Vec<WasiBuildOutputChunk>,
    /// Equality authorizes inert artifact egress only. It never authorizes the
    /// artifact to be loaded, installed, promoted, or executed.
    pub authorizes_load: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScopedWasiArtifactEgressDecision {
    Planned(WasiArtifactEgressPlan),
    Denied(WasiArtifactEgressRejection),
}

pub fn evaluate_scoped_wasi_artifact_egress(
    input: &ScopedWasiArtifactEgress<'_>,
) -> ScopedWasiArtifactEgressDecision {
    let run_one_hash = match parse_output_manifest_sha256(input.run_one.output_manifest_sha256) {
        Some(hash) => hash,
        None => {
            return denied(WasiArtifactEgressRejection::MalformedHash {
                field: WasiBuildOutputHashField::RunOneOutputManifestSha256,
            })
        }
    };
    let run_two_hash = match parse_output_manifest_sha256(input.run_two.output_manifest_sha256) {
        Some(hash) => hash,
        None => {
            return denied(WasiArtifactEgressRejection::MalformedHash {
                field: WasiBuildOutputHashField::RunTwoOutputManifestSha256,
            })
        }
    };

    if input.run_one.exit_status != 0 || input.run_two.exit_status != 0 {
        return denied(WasiArtifactEgressRejection::BuildFailed {
            run_one_exit_status: input.run_one.exit_status,
            run_two_exit_status: input.run_two.exit_status,
        });
    }

    if run_one_hash != run_two_hash || input.run_one.chunks != input.run_two.chunks {
        return denied(WasiArtifactEgressRejection::BuildOutputsNotReproducible);
    }

    ScopedWasiArtifactEgressDecision::Planned(WasiArtifactEgressPlan {
        output_manifest_sha256: run_one_hash,
        chunks: input.run_one.chunks.to_vec(),
        authorizes_load: false,
    })
}

fn denied(rejection: WasiArtifactEgressRejection) -> ScopedWasiArtifactEgressDecision {
    ScopedWasiArtifactEgressDecision::Denied(rejection)
}

#[cfg(test)]
mod tests {
    use alloc::{format, string::String};

    use crate::{sha256_bytes, sha256_hex};

    use super::*;

    fn hex(bytes: &[u8]) -> String {
        String::from_utf8(sha256_hex(&sha256_bytes(bytes)).to_vec()).unwrap()
    }

    fn chunk(bytes: &[u8]) -> WasiBuildOutputChunk {
        WasiBuildOutputChunk {
            len: bytes.len() as u64,
            sha256: sha256_bytes(bytes),
        }
    }

    fn input<'a>(
        first_hash: &'a str,
        second_hash: &'a str,
        first_exit: i32,
        second_exit: i32,
        first_chunks: &'a [WasiBuildOutputChunk],
        second_chunks: &'a [WasiBuildOutputChunk],
    ) -> ScopedWasiArtifactEgress<'a> {
        ScopedWasiArtifactEgress {
            run_one: WasiBuildOutputRun {
                exit_status: first_exit,
                output_manifest_sha256: first_hash,
                chunks: first_chunks,
            },
            run_two: WasiBuildOutputRun {
                exit_status: second_exit,
                output_manifest_sha256: second_hash,
                chunks: second_chunks,
            },
        }
    }

    fn rejection(decision: ScopedWasiArtifactEgressDecision) -> WasiArtifactEgressRejection {
        match decision {
            ScopedWasiArtifactEgressDecision::Denied(rejection) => rejection,
            ScopedWasiArtifactEgressDecision::Planned(_) => panic!("expected denial, got plan"),
        }
    }

    #[test]
    fn equal_successful_runs_create_one_complete_inert_plan() {
        let hash = hex(b"manifest");
        let chunks = [chunk(b"one"), chunk(b"two")];
        let decision =
            evaluate_scoped_wasi_artifact_egress(&input(&hash, &hash, 0, 0, &chunks, &chunks));
        let ScopedWasiArtifactEgressDecision::Planned(plan) = decision else {
            panic!("equal successful runs must plan egress")
        };
        assert_eq!(plan.output_manifest_sha256, sha256_bytes(b"manifest"));
        assert_eq!(plan.chunks, chunks);
        assert!(!plan.authorizes_load);
    }

    #[test]
    fn one_changed_output_byte_denies_reproducibility_without_a_plan() {
        let first = hex(b"output-a");
        let second = hex(b"output-b");
        let chunks = [chunk(b"payload")];
        let rejection = rejection(evaluate_scoped_wasi_artifact_egress(&input(
            &first, &second, 0, 0, &chunks, &chunks,
        )));
        assert_eq!(
            rejection,
            WasiArtifactEgressRejection::BuildOutputsNotReproducible
        );
        assert_eq!(rejection.reason(), "build_outputs_not_reproducible");
    }

    #[test]
    fn nonzero_exit_in_either_run_denies_the_entire_plan() {
        let hash = hex(b"manifest");
        let chunks = [chunk(b"payload")];
        for exits in [(1, 0), (0, -9), (2, 3)] {
            let rejection = rejection(evaluate_scoped_wasi_artifact_egress(&input(
                &hash, &hash, exits.0, exits.1, &chunks, &chunks,
            )));
            assert_eq!(rejection.reason(), "build_failed");
            assert_eq!(
                rejection,
                WasiArtifactEgressRejection::BuildFailed {
                    run_one_exit_status: exits.0,
                    run_two_exit_status: exits.1,
                }
            );
        }
    }

    #[test]
    fn malformed_hash_is_typed_per_run_and_never_yields_a_partial_plan() {
        let valid = hex(b"manifest");
        let chunks = [chunk(b"payload")];
        for (first, second, field) in [
            (
                "short",
                valid.as_str(),
                WasiBuildOutputHashField::RunOneOutputManifestSha256,
            ),
            (
                valid.as_str(),
                "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
                WasiBuildOutputHashField::RunTwoOutputManifestSha256,
            ),
        ] {
            let rejection = rejection(evaluate_scoped_wasi_artifact_egress(&input(
                first, second, 0, 0, &chunks, &chunks,
            )));
            assert_eq!(
                rejection,
                WasiArtifactEgressRejection::MalformedHash { field }
            );
            assert_eq!(rejection.reason(), "malformed_hash");
        }
        assert!(format!(
            "{:?}",
            ScopedWasiArtifactEgressDecision::Denied(
                WasiArtifactEgressRejection::BuildOutputsNotReproducible
            )
        )
        .contains("Denied"));
    }

    #[test]
    fn same_claimed_hash_with_different_chunk_list_fails_closed() {
        let hash = hex(b"manifest");
        let first = [chunk(b"one")];
        let second = [chunk(b"two")];
        assert_eq!(
            rejection(evaluate_scoped_wasi_artifact_egress(&input(
                &hash, &hash, 0, 0, &first, &second,
            ))),
            WasiArtifactEgressRejection::BuildOutputsNotReproducible
        );
    }
}
