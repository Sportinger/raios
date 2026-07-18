//! Fail-closed authorization for the measured WASI build import family.
//!
//! A positive decision authorizes the complete typed declaration list or
//! nothing. Import implementations and their effects are outside this slice.

use crate::{
    parse_sha256_ref,
    wasi_preview1_import_abi::{
        wasi_import_declarations_sha256, WasiImportDeclaration, RUSTC_WASM_C6DCCF3E_IMPORTS,
    },
};

pub const RUSTC_WASM_C6DCCF3E_CANONICAL_IMPORTS_SHA256: &str =
    "4145184d6ae43a57e1f75e1cfc2b4a19c6dd27fb1a29dd7104e1df9817616e65";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasiBuildGrantHashField {
    CompilerArtifactSha256,
    JobManifestSha256,
    InventoryImportsSha256,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasiBuildGrantRejection {
    MalformedHash {
        field: WasiBuildGrantHashField,
    },
    ExtraImport {
        index: usize,
    },
    MissingImport {
        index: usize,
    },
    Reordered {
        index: usize,
        reference_index: usize,
    },
    SignatureMismatch {
        index: usize,
    },
    InventoryImportsHashMismatch,
}

impl WasiBuildGrantRejection {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::MalformedHash { .. } => "malformed_hash",
            Self::ExtraImport { .. } => "extra_import",
            Self::MissingImport { .. } => "missing_import",
            Self::Reordered { .. } => "reordered",
            Self::SignatureMismatch { .. } => "signature_mismatch",
            Self::InventoryImportsHashMismatch => "inventory_imports_hash_mismatch",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopedWasiBuildGrant<'a> {
    pub compiler_artifact_sha256: &'a str,
    pub job_manifest_sha256: &'a str,
    pub inventory_imports_sha256: &'a str,
    pub declared_imports: &'a [WasiImportDeclaration<'a>],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorizedWasiBuildGrant {
    pub compiler_artifact_sha256: [u8; 32],
    pub job_manifest_sha256: [u8; 32],
    pub inventory_imports_sha256: [u8; 32],
    pub authorized_import_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopedWasiBuildGrantDecision {
    Authorized(AuthorizedWasiBuildGrant),
    Denied(WasiBuildGrantRejection),
}

pub fn evaluate_scoped_wasi_build_grant(
    grant: &ScopedWasiBuildGrant<'_>,
) -> ScopedWasiBuildGrantDecision {
    let compiler_artifact_sha256 = match parse_exact_sha256(grant.compiler_artifact_sha256) {
        Some(value) => value,
        None => {
            return denied(WasiBuildGrantRejection::MalformedHash {
                field: WasiBuildGrantHashField::CompilerArtifactSha256,
            })
        }
    };
    let job_manifest_sha256 = match parse_exact_sha256(grant.job_manifest_sha256) {
        Some(value) => value,
        None => {
            return denied(WasiBuildGrantRejection::MalformedHash {
                field: WasiBuildGrantHashField::JobManifestSha256,
            })
        }
    };
    let inventory_imports_sha256 = match parse_exact_sha256(grant.inventory_imports_sha256) {
        Some(value) => value,
        None => {
            return denied(WasiBuildGrantRejection::MalformedHash {
                field: WasiBuildGrantHashField::InventoryImportsSha256,
            })
        }
    };

    if let Some(rejection) = declaration_mismatch(grant.declared_imports) {
        return denied(rejection);
    }

    if wasi_import_declarations_sha256(grant.declared_imports) != inventory_imports_sha256 {
        return denied(WasiBuildGrantRejection::InventoryImportsHashMismatch);
    }

    ScopedWasiBuildGrantDecision::Authorized(AuthorizedWasiBuildGrant {
        compiler_artifact_sha256,
        job_manifest_sha256,
        inventory_imports_sha256,
        authorized_import_count: grant.declared_imports.len(),
    })
}

fn parse_exact_sha256(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    parse_sha256_ref(value)
}

fn declaration_mismatch(declared: &[WasiImportDeclaration<'_>]) -> Option<WasiBuildGrantRejection> {
    let common_len = core::cmp::min(declared.len(), RUSTC_WASM_C6DCCF3E_IMPORTS.len());
    let mut index = 0usize;
    while index < common_len {
        let actual = declared[index];
        let expected = RUSTC_WASM_C6DCCF3E_IMPORTS[index];
        if same_identity(actual, expected) && actual.kind != expected.kind {
            return Some(WasiBuildGrantRejection::SignatureMismatch { index });
        }
        index += 1;
    }

    if declared.len() > RUSTC_WASM_C6DCCF3E_IMPORTS.len() {
        return Some(WasiBuildGrantRejection::ExtraImport {
            index: first_sequence_mismatch(declared),
        });
    }
    if declared.len() < RUSTC_WASM_C6DCCF3E_IMPORTS.len() {
        return Some(WasiBuildGrantRejection::MissingImport {
            index: first_sequence_mismatch(declared),
        });
    }

    index = 0;
    while index < declared.len() {
        if declared[index] != RUSTC_WASM_C6DCCF3E_IMPORTS[index] {
            if let Some(reference_index) = find_exact_reference(declared[index]) {
                return Some(WasiBuildGrantRejection::Reordered {
                    index,
                    reference_index,
                });
            }
            if find_identity_reference(declared[index]).is_some() {
                return Some(WasiBuildGrantRejection::SignatureMismatch { index });
            }
            return Some(WasiBuildGrantRejection::ExtraImport { index });
        }
        index += 1;
    }
    None
}

fn first_sequence_mismatch(declared: &[WasiImportDeclaration<'_>]) -> usize {
    let common_len = core::cmp::min(declared.len(), RUSTC_WASM_C6DCCF3E_IMPORTS.len());
    let mut index = 0usize;
    while index < common_len && declared[index] == RUSTC_WASM_C6DCCF3E_IMPORTS[index] {
        index += 1;
    }
    index
}

fn same_identity(left: WasiImportDeclaration<'_>, right: WasiImportDeclaration<'_>) -> bool {
    left.module == right.module && left.name == right.name
}

fn find_exact_reference(import: WasiImportDeclaration<'_>) -> Option<usize> {
    RUSTC_WASM_C6DCCF3E_IMPORTS
        .iter()
        .position(|reference| *reference == import)
}

fn find_identity_reference(import: WasiImportDeclaration<'_>) -> Option<usize> {
    RUSTC_WASM_C6DCCF3E_IMPORTS
        .iter()
        .position(|reference| same_identity(*reference, import))
}

fn denied(rejection: WasiBuildGrantRejection) -> ScopedWasiBuildGrantDecision {
    ScopedWasiBuildGrantDecision::Denied(rejection)
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use crate::{
        sha256_hex,
        wasi_preview1_import_abi::{WasiImportKind, WasiValueType},
    };

    use super::*;

    const COMPILER_SHA256: &str =
        "c6dccf3e5f01631b942a0a008b9f2f5312987e7d8590f8c61024cd00687a5791";
    const JOB_MANIFEST_SHA256: &str =
        "1111111111111111111111111111111111111111111111111111111111111111";

    fn measured_inventory_fixture() -> Vec<WasiImportDeclaration<'static>> {
        RUSTC_WASM_C6DCCF3E_IMPORTS.to_vec()
    }

    fn grant_for<'a>(imports: &'a [WasiImportDeclaration<'a>]) -> ScopedWasiBuildGrant<'a> {
        ScopedWasiBuildGrant {
            compiler_artifact_sha256: COMPILER_SHA256,
            job_manifest_sha256: JOB_MANIFEST_SHA256,
            inventory_imports_sha256: RUSTC_WASM_C6DCCF3E_CANONICAL_IMPORTS_SHA256,
            declared_imports: imports,
        }
    }

    fn rejection(decision: ScopedWasiBuildGrantDecision) -> WasiBuildGrantRejection {
        match decision {
            ScopedWasiBuildGrantDecision::Denied(rejection) => rejection,
            ScopedWasiBuildGrantDecision::Authorized(_) => panic!("expected denial"),
        }
    }

    #[test]
    fn measured_inventory_golden_hash_is_authorized() {
        let imports = measured_inventory_fixture();
        let hash = sha256_hex(&wasi_import_declarations_sha256(&imports));
        assert_eq!(
            hash.as_slice(),
            RUSTC_WASM_C6DCCF3E_CANONICAL_IMPORTS_SHA256.as_bytes()
        );

        assert_eq!(
            evaluate_scoped_wasi_build_grant(&grant_for(&imports)),
            ScopedWasiBuildGrantDecision::Authorized(AuthorizedWasiBuildGrant {
                compiler_artifact_sha256: parse_exact_sha256(COMPILER_SHA256).unwrap(),
                job_manifest_sha256: parse_exact_sha256(JOB_MANIFEST_SHA256).unwrap(),
                inventory_imports_sha256: parse_exact_sha256(
                    RUSTC_WASM_C6DCCF3E_CANONICAL_IMPORTS_SHA256
                )
                .unwrap(),
                authorized_import_count: 30,
            })
        );
    }

    #[test]
    fn extra_thirty_first_import_denies_the_entire_grant() {
        let mut imports = measured_inventory_fixture();
        imports.push(WasiImportDeclaration::func("env", "unexpected", &[], &[]));

        assert_eq!(
            rejection(evaluate_scoped_wasi_build_grant(&grant_for(&imports))),
            WasiBuildGrantRejection::ExtraImport { index: 30 }
        );
    }

    #[test]
    fn missing_import_denies_the_entire_grant() {
        let mut imports = measured_inventory_fixture();
        imports.remove(12);

        assert_eq!(
            rejection(evaluate_scoped_wasi_build_grant(&grant_for(&imports))),
            WasiBuildGrantRejection::MissingImport { index: 12 }
        );
    }

    #[test]
    fn reordered_imports_are_denied() {
        let mut imports = measured_inventory_fixture();
        imports.swap(0, 1);

        assert_eq!(
            rejection(evaluate_scoped_wasi_build_grant(&grant_for(&imports))),
            WasiBuildGrantRejection::Reordered {
                index: 0,
                reference_index: 1,
            }
        );
    }

    #[test]
    fn proc_exit_result_change_is_a_signature_mismatch() {
        let mut imports = measured_inventory_fixture();
        imports[27].kind = WasiImportKind::Func {
            params: &[WasiValueType::I32],
            results: &[WasiValueType::I32],
        };

        assert_eq!(
            rejection(evaluate_scoped_wasi_build_grant(&grant_for(&imports))),
            WasiBuildGrantRejection::SignatureMismatch { index: 27 }
        );
    }

    #[test]
    fn malformed_compiler_hash_is_denied_for_length_and_non_hex() {
        let imports = measured_inventory_fixture();
        let malformed = [
            &COMPILER_SHA256[..63],
            "g6dccf3e5f01631b942a0a008b9f2f5312987e7d8590f8c61024cd00687a5791",
        ];

        for compiler_artifact_sha256 in malformed {
            let mut grant = grant_for(&imports);
            grant.compiler_artifact_sha256 = compiler_artifact_sha256;
            let rejection = rejection(evaluate_scoped_wasi_build_grant(&grant));
            assert_eq!(
                rejection,
                WasiBuildGrantRejection::MalformedHash {
                    field: WasiBuildGrantHashField::CompilerArtifactSha256,
                }
            );
            assert_eq!(rejection.reason(), "malformed_hash");
        }
    }

    #[test]
    fn every_hash_binding_is_required_to_be_exact_hex() {
        let imports = measured_inventory_fixture();

        let mut malformed_job = grant_for(&imports);
        malformed_job.job_manifest_sha256 = &JOB_MANIFEST_SHA256[..63];
        assert_eq!(
            rejection(evaluate_scoped_wasi_build_grant(&malformed_job)),
            WasiBuildGrantRejection::MalformedHash {
                field: WasiBuildGrantHashField::JobManifestSha256,
            }
        );

        let mut malformed_inventory = grant_for(&imports);
        malformed_inventory.inventory_imports_sha256 =
            "z145184d6ae43a57e1f75e1cfc2b4a19c6dd27fb1a29dd7104e1df9817616e65";
        assert_eq!(
            rejection(evaluate_scoped_wasi_build_grant(&malformed_inventory)),
            WasiBuildGrantRejection::MalformedHash {
                field: WasiBuildGrantHashField::InventoryImportsSha256,
            }
        );
    }

    #[test]
    fn well_formed_but_wrong_inventory_hash_is_denied() {
        let imports = measured_inventory_fixture();
        let mut grant = grant_for(&imports);
        grant.inventory_imports_sha256 = JOB_MANIFEST_SHA256;

        assert_eq!(
            rejection(evaluate_scoped_wasi_build_grant(&grant)),
            WasiBuildGrantRejection::InventoryImportsHashMismatch
        );
    }
}
