//! Fail-closed double-build gate for inert WASI build artifact egress.
//!
//! This slice only creates a typed plan. It has no persistence callback and no
//! artifact-store access; those effects belong to a later scoped adapter.

use crate::wasi_build_output::FrozenOutput;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopedWasiArtifactEgress {
    pub run_one: FrozenOutput,
    pub run_two: FrozenOutput,
    pub run_one_exit_status: i32,
    pub run_two_exit_status: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasiArtifactEgressRejection {
    BuildFailed {
        run_one_exit_status: i32,
        run_two_exit_status: i32,
    },
    BuildOutputsNotReproducible,
}

impl WasiArtifactEgressRejection {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::BuildFailed { .. } => "build_failed",
            Self::BuildOutputsNotReproducible => "build_outputs_not_reproducible",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasiArtifactEgressPlan {
    pub output_manifest_sha256: [u8; 32],
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
    input: &ScopedWasiArtifactEgress,
) -> ScopedWasiArtifactEgressDecision {
    if input.run_one_exit_status != 0 || input.run_two_exit_status != 0 {
        return denied(WasiArtifactEgressRejection::BuildFailed {
            run_one_exit_status: input.run_one_exit_status,
            run_two_exit_status: input.run_two_exit_status,
        });
    }

    if input.run_one.digest() != input.run_two.digest() {
        return denied(WasiArtifactEgressRejection::BuildOutputsNotReproducible);
    }

    ScopedWasiArtifactEgressDecision::Planned(WasiArtifactEgressPlan {
        output_manifest_sha256: input.run_one.digest(),
        authorizes_load: false,
    })
}

fn denied(rejection: WasiArtifactEgressRejection) -> ScopedWasiArtifactEgressDecision {
    ScopedWasiArtifactEgressDecision::Denied(rejection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sha256_bytes;

    fn input(
        first_manifest: &[u8],
        second_manifest: &[u8],
        first_exit: i32,
        second_exit: i32,
    ) -> ScopedWasiArtifactEgress {
        ScopedWasiArtifactEgress {
            run_one: FrozenOutput::from_manifest_bytes(first_manifest),
            run_two: FrozenOutput::from_manifest_bytes(second_manifest),
            run_one_exit_status: first_exit,
            run_two_exit_status: second_exit,
        }
    }

    fn rejection(decision: ScopedWasiArtifactEgressDecision) -> WasiArtifactEgressRejection {
        match decision {
            ScopedWasiArtifactEgressDecision::Denied(rejection) => rejection,
            ScopedWasiArtifactEgressDecision::Planned(_) => panic!("expected denial, got plan"),
        }
    }

    #[test]
    fn byte_identical_successful_runs_create_an_inert_plan() {
        let decision = evaluate_scoped_wasi_artifact_egress(&input(
            b"canonical manifest",
            b"canonical manifest",
            0,
            0,
        ));
        let ScopedWasiArtifactEgressDecision::Planned(plan) = decision else {
            panic!("equal successful runs must plan egress")
        };
        assert_eq!(
            plan.output_manifest_sha256,
            sha256_bytes(b"canonical manifest")
        );
        assert!(!plan.authorizes_load);
    }

    #[test]
    fn same_claimed_hash_cannot_forge_different_manifest_bytes() {
        let claimed_hash = [0xabu8; 32];
        let run_one_claim = claimed_hash;
        let run_two_claim = claimed_hash;
        assert_eq!(run_one_claim, run_two_claim);

        // The gate never accepts either claim: both FrozenOutput values must
        // recompute their private digest from the actual manifest bytes.
        assert_eq!(
            rejection(evaluate_scoped_wasi_artifact_egress(&input(
                b"output-a",
                b"output-b",
                0,
                0,
            ))),
            WasiArtifactEgressRejection::BuildOutputsNotReproducible
        );
    }

    #[test]
    fn one_changed_output_byte_denies_reproducibility_without_a_plan() {
        let rejection = rejection(evaluate_scoped_wasi_artifact_egress(&input(
            b"manifest-a",
            b"manifest-b",
            0,
            0,
        )));
        assert_eq!(
            rejection,
            WasiArtifactEgressRejection::BuildOutputsNotReproducible
        );
        assert_eq!(rejection.reason(), "build_outputs_not_reproducible");
    }

    #[test]
    fn nonzero_exit_in_either_run_denies_the_entire_plan() {
        for exits in [(1, 0), (0, -9), (2, 3)] {
            let rejection = rejection(evaluate_scoped_wasi_artifact_egress(&input(
                b"manifest",
                b"manifest",
                exits.0,
                exits.1,
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
}
