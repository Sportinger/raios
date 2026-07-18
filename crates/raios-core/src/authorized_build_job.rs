//! Opaque, fail-closed authorization ticket for one WASI build job.
//!
//! The kernel receives only `AuthorizedBuildJob`, never unchecked request
//! parts. Authorization deliberately repeats the observed-import comparison
//! after the scoped grant check so the integration boundary has two
//! independent exact-list gates.

use crate::{
    build_guest_class::{
        build_guest_class_sha256, BuildGuestClassV1, BuildGuestClassValidationError,
    },
    scoped_wasi_build_grant::{
        evaluate_scoped_wasi_build_grant, AuthorizedWasiBuildGrant, ScopedWasiBuildGrant,
        ScopedWasiBuildGrantDecision, WasiBuildGrantRejection,
    },
    wasi_preview1_import_abi::{WasiImportDeclaration, RUSTC_WASM_C6DCCF3E_IMPORTS},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorizedBuildJobRequest<'a> {
    pub wasi_grant: ScopedWasiBuildGrant<'a>,
    pub observed_imports: &'a [WasiImportDeclaration<'a>],
    pub guest_class: BuildGuestClassV1,
    pub sysroot_mount_manifest_sha256: [u8; 32],
    pub src_mount_manifest_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildJobMountHashField {
    SysrootMountManifestSha256,
    SrcMountManifestSha256,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildJobDenied {
    GrantDenied {
        rejection: WasiBuildGrantRejection,
    },
    ImportsMismatch {
        first_mismatch: usize,
        expected_count: usize,
        actual_count: usize,
    },
    GuestClassInvalid {
        rejection: BuildGuestClassValidationError,
    },
    MalformedMountHash {
        field: BuildJobMountHashField,
    },
}

impl BuildJobDenied {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::GrantDenied { .. } => "grant_denied",
            Self::ImportsMismatch { .. } => "imports_mismatch",
            Self::GuestClassInvalid { .. } => "guest_class_invalid",
            Self::MalformedMountHash { .. } => "malformed_mount_hash",
        }
    }
}

/// Fully checked, immutable build-job ticket.
///
/// Its fields are private and it has no unchecked constructor or setters.
/// Existing tickets may be copied, but new authority can only originate from
/// [`AuthorizedBuildJob::authorize`].
///
/// ```compile_fail
/// use raios_core::authorized_build_job::AuthorizedBuildJob;
///
/// let forged = AuthorizedBuildJob {};
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorizedBuildJob {
    guest_class: BuildGuestClassV1,
    guest_class_sha256: [u8; 32],
    grant: AuthorizedWasiBuildGrant,
    sysroot_mount_manifest_sha256: [u8; 32],
    src_mount_manifest_sha256: [u8; 32],
}

impl AuthorizedBuildJob {
    pub fn authorize(request: AuthorizedBuildJobRequest<'_>) -> Result<Self, BuildJobDenied> {
        let grant = match evaluate_scoped_wasi_build_grant(&request.wasi_grant) {
            ScopedWasiBuildGrantDecision::Authorized(grant) => grant,
            ScopedWasiBuildGrantDecision::Denied(rejection) => {
                return Err(BuildJobDenied::GrantDenied { rejection });
            }
        };

        if request.observed_imports != RUSTC_WASM_C6DCCF3E_IMPORTS {
            return Err(BuildJobDenied::ImportsMismatch {
                first_mismatch: first_import_mismatch(request.observed_imports),
                expected_count: RUSTC_WASM_C6DCCF3E_IMPORTS.len(),
                actual_count: request.observed_imports.len(),
            });
        }

        request
            .guest_class
            .validate()
            .map_err(|rejection| BuildJobDenied::GuestClassInvalid { rejection })?;

        // All-zero is the reserved missing/uninitialized digest sentinel. The
        // request fields are fixed-width, so no other malformed length exists.
        if request.sysroot_mount_manifest_sha256 == [0; 32] {
            return Err(BuildJobDenied::MalformedMountHash {
                field: BuildJobMountHashField::SysrootMountManifestSha256,
            });
        }
        if request.src_mount_manifest_sha256 == [0; 32] {
            return Err(BuildJobDenied::MalformedMountHash {
                field: BuildJobMountHashField::SrcMountManifestSha256,
            });
        }

        Ok(Self {
            guest_class: request.guest_class,
            guest_class_sha256: build_guest_class_sha256(&request.guest_class),
            grant,
            sysroot_mount_manifest_sha256: request.sysroot_mount_manifest_sha256,
            src_mount_manifest_sha256: request.src_mount_manifest_sha256,
        })
    }

    pub const fn guest_class(&self) -> &BuildGuestClassV1 {
        &self.guest_class
    }

    pub const fn guest_class_sha256(&self) -> [u8; 32] {
        self.guest_class_sha256
    }

    pub const fn compiler_artifact_sha256(&self) -> [u8; 32] {
        self.grant.compiler_artifact_sha256
    }

    pub const fn job_manifest_sha256(&self) -> [u8; 32] {
        self.grant.job_manifest_sha256
    }

    pub const fn inventory_imports_sha256(&self) -> [u8; 32] {
        self.grant.inventory_imports_sha256
    }

    pub const fn authorized_import_count(&self) -> usize {
        self.grant.authorized_import_count
    }

    pub const fn sysroot_mount_manifest_sha256(&self) -> [u8; 32] {
        self.sysroot_mount_manifest_sha256
    }

    pub const fn src_mount_manifest_sha256(&self) -> [u8; 32] {
        self.src_mount_manifest_sha256
    }
}

fn first_import_mismatch(observed: &[WasiImportDeclaration<'_>]) -> usize {
    let common_len = core::cmp::min(observed.len(), RUSTC_WASM_C6DCCF3E_IMPORTS.len());
    let mut index = 0usize;
    while index < common_len && observed[index] == RUSTC_WASM_C6DCCF3E_IMPORTS[index] {
        index += 1;
    }
    index
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;
    use crate::{
        build_guest_class::RUSTC_BUILD_GUEST_CLASS_V1,
        parse_sha256_ref,
        scoped_wasi_build_grant::{
            WasiBuildGrantHashField, RUSTC_WASM_C6DCCF3E_CANONICAL_IMPORTS_SHA256,
        },
        sha256_bytes,
    };

    const COMPILER_SHA256: &str =
        "c6dccf3e5f01631b942a0a008b9f2f5312987e7d8590f8c61024cd00687a5791";
    const JOB_MANIFEST_SHA256: &str =
        "1111111111111111111111111111111111111111111111111111111111111111";

    fn request<'a>(
        declared_imports: &'a [WasiImportDeclaration<'a>],
        observed_imports: &'a [WasiImportDeclaration<'a>],
    ) -> AuthorizedBuildJobRequest<'a> {
        AuthorizedBuildJobRequest {
            wasi_grant: ScopedWasiBuildGrant {
                compiler_artifact_sha256: COMPILER_SHA256,
                job_manifest_sha256: JOB_MANIFEST_SHA256,
                inventory_imports_sha256: RUSTC_WASM_C6DCCF3E_CANONICAL_IMPORTS_SHA256,
                declared_imports,
            },
            observed_imports,
            guest_class: RUSTC_BUILD_GUEST_CLASS_V1,
            sysroot_mount_manifest_sha256: sha256_bytes(b"sysroot BuildFS manifest"),
            src_mount_manifest_sha256: sha256_bytes(b"src BuildFS manifest"),
        }
    }

    #[test]
    fn exact_request_authorizes_one_opaque_ticket_with_getters_only() {
        let request = request(RUSTC_WASM_C6DCCF3E_IMPORTS, RUSTC_WASM_C6DCCF3E_IMPORTS);
        let job = AuthorizedBuildJob::authorize(request).unwrap();

        assert_eq!(job.guest_class(), &RUSTC_BUILD_GUEST_CLASS_V1);
        assert_eq!(
            job.guest_class_sha256(),
            build_guest_class_sha256(&RUSTC_BUILD_GUEST_CLASS_V1)
        );
        assert_eq!(
            job.compiler_artifact_sha256(),
            parse_sha256_ref(COMPILER_SHA256).unwrap()
        );
        assert_eq!(
            job.job_manifest_sha256(),
            parse_sha256_ref(JOB_MANIFEST_SHA256).unwrap()
        );
        assert_eq!(
            job.inventory_imports_sha256(),
            parse_sha256_ref(RUSTC_WASM_C6DCCF3E_CANONICAL_IMPORTS_SHA256).unwrap()
        );
        assert_eq!(job.authorized_import_count(), 30);
        assert_eq!(
            job.sysroot_mount_manifest_sha256(),
            sha256_bytes(b"sysroot BuildFS manifest")
        );
        assert_eq!(
            job.src_mount_manifest_sha256(),
            sha256_bytes(b"src BuildFS manifest")
        );
    }

    #[test]
    fn extra_observed_import_is_an_independent_imports_mismatch() {
        let mut observed: Vec<_> = RUSTC_WASM_C6DCCF3E_IMPORTS.to_vec();
        observed.push(RUSTC_WASM_C6DCCF3E_IMPORTS[0]);

        let denial = AuthorizedBuildJob::authorize(request(RUSTC_WASM_C6DCCF3E_IMPORTS, &observed))
            .unwrap_err();
        assert_eq!(
            denial,
            BuildJobDenied::ImportsMismatch {
                first_mismatch: 30,
                expected_count: 30,
                actual_count: 31,
            }
        );
        assert_eq!(denial.reason(), "imports_mismatch");
    }

    #[test]
    fn invalid_guest_class_is_denied_before_ticket_construction() {
        let mut request = request(RUSTC_WASM_C6DCCF3E_IMPORTS, RUSTC_WASM_C6DCCF3E_IMPORTS);
        request.guest_class.fuel_quantum = request.guest_class.max_total_fuel + 1;

        let denial = AuthorizedBuildJob::authorize(request).unwrap_err();
        assert_eq!(
            denial,
            BuildJobDenied::GuestClassInvalid {
                rejection: BuildGuestClassValidationError::FuelQuantumExceedsMaxTotalFuel {
                    fuel_quantum: RUSTC_BUILD_GUEST_CLASS_V1.max_total_fuel + 1,
                    max_total_fuel: RUSTC_BUILD_GUEST_CLASS_V1.max_total_fuel,
                },
            }
        );
        assert_eq!(denial.reason(), "guest_class_invalid");
    }

    #[test]
    fn missing_mount_digest_sentinel_is_typed_per_mount() {
        for field in [
            BuildJobMountHashField::SysrootMountManifestSha256,
            BuildJobMountHashField::SrcMountManifestSha256,
        ] {
            let mut request = request(RUSTC_WASM_C6DCCF3E_IMPORTS, RUSTC_WASM_C6DCCF3E_IMPORTS);
            match field {
                BuildJobMountHashField::SysrootMountManifestSha256 => {
                    request.sysroot_mount_manifest_sha256 = [0; 32]
                }
                BuildJobMountHashField::SrcMountManifestSha256 => {
                    request.src_mount_manifest_sha256 = [0; 32]
                }
            }

            let denial = AuthorizedBuildJob::authorize(request).unwrap_err();
            assert_eq!(denial, BuildJobDenied::MalformedMountHash { field });
            assert_eq!(denial.reason(), "malformed_mount_hash");
        }
    }

    #[test]
    fn scoped_grant_denial_is_preserved_and_no_ticket_is_created() {
        let mut request = request(RUSTC_WASM_C6DCCF3E_IMPORTS, RUSTC_WASM_C6DCCF3E_IMPORTS);
        request.wasi_grant.compiler_artifact_sha256 = "short";

        let denial = AuthorizedBuildJob::authorize(request).unwrap_err();
        assert_eq!(
            denial,
            BuildJobDenied::GrantDenied {
                rejection: WasiBuildGrantRejection::MalformedHash {
                    field: WasiBuildGrantHashField::CompilerArtifactSha256,
                },
            }
        );
        assert_eq!(denial.reason(), "grant_denied");
    }
}
