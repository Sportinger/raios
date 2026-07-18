//! Fail-closed double-build gate for inert WASI build artifact egress.
//!
//! This gate creates a job- and lease-bound typed plan. It has no persistence
//! callback and no artifact-store access; those effects require the separate
//! pre-I/O commit authority.

use crate::{build_storage_authority::BuildStorageAuthority, wasi_build_output::FrozenOutput};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopedWasiArtifactEgress {
    pub run_one: FrozenOutput,
    pub run_two: FrozenOutput,
    pub run_one_exit_status: i32,
    pub run_two_exit_status: i32,
    pub run_one_logical_content_size: u64,
    pub run_two_logical_content_size: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasiArtifactEgressRejection {
    BuildFailed {
        run_one_exit_status: i32,
        run_two_exit_status: i32,
    },
    BuildOutputsNotReproducible,
    BuildOutputSizesNotReproducible {
        run_one_logical_content_size: u64,
        run_two_logical_content_size: u64,
    },
}

impl WasiArtifactEgressRejection {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::BuildFailed { .. } => "build_failed",
            Self::BuildOutputsNotReproducible => "build_outputs_not_reproducible",
            Self::BuildOutputSizesNotReproducible { .. } => "build_output_sizes_not_reproducible",
        }
    }
}

/// Inert, destination-bound result of the double-build gate.
///
/// Fields are private and no unchecked constructor exists. Loading remains a
/// structural constant rather than caller-controlled state.
///
/// ```compile_fail
/// use raios_core::scoped_wasi_artifact_egress::WasiArtifactEgressPlan;
///
/// let forged = WasiArtifactEgressPlan {};
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct WasiArtifactEgressPlan {
    job_binding_sha256: [u8; 32],
    output_lease_id: u64,
    output_manifest_sha256: [u8; 32],
    logical_content_size: u64,
}

impl WasiArtifactEgressPlan {
    pub const fn job_binding_sha256(&self) -> [u8; 32] {
        self.job_binding_sha256
    }

    pub const fn output_lease_id(&self) -> u64 {
        self.output_lease_id
    }

    pub const fn output_manifest_sha256(&self) -> [u8; 32] {
        self.output_manifest_sha256
    }

    pub const fn logical_content_size(&self) -> u64 {
        self.logical_content_size
    }

    /// Inert egress never authorizes loading, installation, promotion, or
    /// execution. There is deliberately no stored boolean a caller can flip.
    pub const fn authorizes_load(&self) -> bool {
        false
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ScopedWasiArtifactEgressDecision {
    Planned(WasiArtifactEgressPlan),
    Denied(WasiArtifactEgressRejection),
}

pub fn evaluate_scoped_wasi_artifact_egress(
    input: &ScopedWasiArtifactEgress,
    authority: &BuildStorageAuthority,
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
    if input.run_one_logical_content_size != input.run_two_logical_content_size {
        return denied(
            WasiArtifactEgressRejection::BuildOutputSizesNotReproducible {
                run_one_logical_content_size: input.run_one_logical_content_size,
                run_two_logical_content_size: input.run_two_logical_content_size,
            },
        );
    }

    ScopedWasiArtifactEgressDecision::Planned(WasiArtifactEgressPlan {
        job_binding_sha256: authority.job_binding_sha256(),
        output_lease_id: authority.output_lease().lease_id(),
        output_manifest_sha256: input.run_one.digest(),
        logical_content_size: input.run_one_logical_content_size,
    })
}

fn denied(rejection: WasiArtifactEgressRejection) -> ScopedWasiArtifactEgressDecision {
    ScopedWasiArtifactEgressDecision::Denied(rejection)
}

#[cfg(test)]
mod tests {
    use alloc::{string::ToString, vec};

    use super::*;
    use crate::{
        authorized_build_job::{AuthorizedBuildJob, AuthorizedBuildJobRequest},
        build_guest_class::RUSTC_BUILD_GUEST_CLASS_V1,
        build_storage_authority::{
            authorize_build_storage, BuildStorageAuthority, OutputLeaseDescriptorV1,
            BUILD_OUTPUT_LEASE_TARGET_MARKER_V1, RUSTC_BUILD_MOUNT_BUDGET_V1,
        },
        buildfs_manifest::{BuildFsChunk, BuildFsDirectory, BuildFsFile, BuildFsManifest},
        scoped_wasi_build_grant::{
            ScopedWasiBuildGrant, RUSTC_WASM_C6DCCF3E_CANONICAL_IMPORTS_SHA256,
        },
        sha256_bytes,
        wasi_preview1_import_abi::RUSTC_WASM_C6DCCF3E_IMPORTS,
    };

    const COMPILER_SHA256: &str =
        "c6dccf3e5f01631b942a0a008b9f2f5312987e7d8590f8c61024cd00687a5791";
    const JOB_MANIFEST_SHA256: &str =
        "1111111111111111111111111111111111111111111111111111111111111111";

    fn manifest(root: &str, tag: u8) -> BuildFsManifest {
        BuildFsManifest::new(
            vec![BuildFsDirectory {
                path: root.to_string(),
            }],
            vec![BuildFsFile {
                path: alloc::format!("{root}/input"),
                len: 1,
                sha256: [tag; 32],
                chunks: vec![BuildFsChunk {
                    len: 1,
                    sha256: [tag; 32],
                }],
            }],
        )
        .unwrap()
    }

    fn authority() -> BuildStorageAuthority {
        let sysroot = manifest("sysroot", 1);
        let src = manifest("src", 2);
        let job = AuthorizedBuildJob::authorize(AuthorizedBuildJobRequest {
            wasi_grant: ScopedWasiBuildGrant {
                compiler_artifact_sha256: COMPILER_SHA256,
                job_manifest_sha256: JOB_MANIFEST_SHA256,
                inventory_imports_sha256: RUSTC_WASM_C6DCCF3E_CANONICAL_IMPORTS_SHA256,
                declared_imports: RUSTC_WASM_C6DCCF3E_IMPORTS,
            },
            observed_imports: RUSTC_WASM_C6DCCF3E_IMPORTS,
            guest_class: RUSTC_BUILD_GUEST_CLASS_V1,
            sysroot_mount_manifest_sha256: sysroot.sha256().unwrap(),
            src_mount_manifest_sha256: src.sha256().unwrap(),
        })
        .unwrap();
        authorize_build_storage(
            &job,
            &sysroot,
            &src,
            RUSTC_BUILD_MOUNT_BUDGET_V1,
            OutputLeaseDescriptorV1::kernel_minted(
                7,
                11,
                13,
                RUSTC_BUILD_MOUNT_BUDGET_V1.max_output_bytes,
                BUILD_OUTPUT_LEASE_TARGET_MARKER_V1,
            ),
        )
        .unwrap()
    }

    fn input(
        first_manifest: &[u8],
        second_manifest: &[u8],
        first_exit: i32,
        second_exit: i32,
        first_size: u64,
        second_size: u64,
    ) -> ScopedWasiArtifactEgress {
        ScopedWasiArtifactEgress {
            run_one: FrozenOutput::from_manifest_bytes(first_manifest),
            run_two: FrozenOutput::from_manifest_bytes(second_manifest),
            run_one_exit_status: first_exit,
            run_two_exit_status: second_exit,
            run_one_logical_content_size: first_size,
            run_two_logical_content_size: second_size,
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
        let authority = authority();
        let decision = evaluate_scoped_wasi_artifact_egress(
            &input(
                b"canonical manifest",
                b"canonical manifest",
                0,
                0,
                4_096,
                4_096,
            ),
            &authority,
        );
        let ScopedWasiArtifactEgressDecision::Planned(plan) = decision else {
            panic!("equal successful runs must plan egress")
        };
        assert_eq!(
            plan.output_manifest_sha256(),
            sha256_bytes(b"canonical manifest")
        );
        assert_eq!(plan.job_binding_sha256(), authority.job_binding_sha256());
        assert_eq!(plan.output_lease_id(), authority.output_lease().lease_id());
        assert_eq!(plan.logical_content_size(), 4_096);
        assert!(!plan.authorizes_load());
    }

    #[test]
    fn same_claimed_hash_cannot_forge_different_manifest_bytes() {
        let authority = authority();
        let claimed_hash = [0xabu8; 32];
        let run_one_claim = claimed_hash;
        let run_two_claim = claimed_hash;
        assert_eq!(run_one_claim, run_two_claim);

        // The gate never accepts either claim: both FrozenOutput values must
        // recompute their private digest from the actual manifest bytes.
        assert_eq!(
            rejection(evaluate_scoped_wasi_artifact_egress(
                &input(b"output-a", b"output-b", 0, 0, 8, 8,),
                &authority
            )),
            WasiArtifactEgressRejection::BuildOutputsNotReproducible
        );
    }

    #[test]
    fn one_changed_output_byte_denies_reproducibility_without_a_plan() {
        let authority = authority();
        let rejection = rejection(evaluate_scoped_wasi_artifact_egress(
            &input(b"manifest-a", b"manifest-b", 0, 0, 10, 10),
            &authority,
        ));
        assert_eq!(
            rejection,
            WasiArtifactEgressRejection::BuildOutputsNotReproducible
        );
        assert_eq!(rejection.reason(), "build_outputs_not_reproducible");
    }

    #[test]
    fn nonzero_exit_in_either_run_denies_the_entire_plan() {
        let authority = authority();
        for exits in [(1, 0), (0, -9), (2, 3)] {
            let rejection = rejection(evaluate_scoped_wasi_artifact_egress(
                &input(b"manifest", b"manifest", exits.0, exits.1, 8, 8),
                &authority,
            ));
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
    fn differing_logical_content_sizes_deny_destination_bound_egress() {
        let authority = authority();
        let rejection = rejection(evaluate_scoped_wasi_artifact_egress(
            &input(b"manifest", b"manifest", 0, 0, 8, 9),
            &authority,
        ));
        assert_eq!(
            rejection,
            WasiArtifactEgressRejection::BuildOutputSizesNotReproducible {
                run_one_logical_content_size: 8,
                run_two_logical_content_size: 9,
            }
        );
        assert_eq!(rejection.reason(), "build_output_sizes_not_reproducible");
    }
}
