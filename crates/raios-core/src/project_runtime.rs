use crate::{
    project_build::{validate_build_receipt, ProjectBuildReceipt},
    scoped_wasm_import_grant::{
        authorized_import_list_sha256, evaluate_observed_wasm_import_grant, WasmImportGrantInput,
    },
    sha256_bytes,
};

pub const WORKSPACE_SERVICE_ID: &str = "svc.workspace.current_boot";
pub const WORKSPACE_ENTRYPOINT: &str = "raios_service_main";
pub const WORKSPACE_FUEL_BUDGET: u64 = 250_000;
pub const WORKSPACE_MEMORY_LIMIT_BYTES: usize = 4 * 1024 * 1024;
pub const WORKSPACE_INSTANCE_LIMIT: usize = 1;
pub const WORKSPACE_MEMORY_COUNT_LIMIT: usize = 1;
pub const WORKSPACE_TABLE_LIMIT: usize = 0;
pub const WORKSPACE_TRUST_TIER: &str =
    "owner_workstation_build_plus_physical_current_boot_not_owner_sealed";

const CHALLENGE_DOMAIN: &[u8] = b"raios.workspace_run_challenge.v1";
const APPROVAL_DOMAIN: &[u8] = b"raios.workspace_physical_approval.v1";
pub const WORKSPACE_PHYSICAL_APPROVAL_SOURCE: &str = "genesis_pointer";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkspaceRunBinding {
    pub project_id: [u8; 16],
    pub project_revision_sha256: [u8; 32],
    pub source_tree_sha256: [u8; 32],
    pub cargo_lock_sha256: [u8; 32],
    pub input_manifest_sha256: [u8; 32],
    pub receipt_sha256: [u8; 32],
    pub candidate_sha256: [u8; 32],
    pub candidate_byte_len: usize,
    pub import_list_sha256: [u8; 32],
    pub import_count: usize,
    pub fuel_budget: u64,
    pub memory_limit_bytes: usize,
    pub instance_limit: usize,
    pub memory_count_limit: usize,
    pub table_limit: usize,
    pub approval_challenge_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkspacePhysicalApproval {
    pub challenge_sha256: [u8; 32],
    pub source: &'static str,
    pub approval_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceRunDenied {
    ReceiptNotReproducible,
    CandidateMismatch,
    ImportListNotObserved,
    ImportsNotEmpty,
    ImportGrantDenied,
    ReceiptInvalid,
    ReceiptAuthorityInvalid,
}

impl WorkspaceRunDenied {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::ReceiptNotReproducible => "workspace_receipt_not_reproducible",
            Self::CandidateMismatch => "workspace_candidate_receipt_mismatch",
            Self::ImportListNotObserved => "workspace_import_list_not_observed",
            Self::ImportsNotEmpty => "workspace_import_surface_not_empty",
            Self::ImportGrantDenied => "workspace_import_grant_denied",
            Self::ReceiptInvalid => "workspace_receipt_invalid",
            Self::ReceiptAuthorityInvalid => "workspace_receipt_authority_invalid",
        }
    }
}

pub fn workspace_import_list_sha256() -> [u8; 32] {
    authorized_import_list_sha256(WORKSPACE_SERVICE_ID, &[])
}

pub fn build_workspace_pointer_approval(
    binding: &WorkspaceRunBinding,
) -> WorkspacePhysicalApproval {
    build_workspace_pointer_approval_from_challenge(binding.approval_challenge_sha256)
}

pub fn build_workspace_pointer_approval_from_challenge(
    approval_challenge_sha256: [u8; 32],
) -> WorkspacePhysicalApproval {
    let mut bytes = alloc::vec::Vec::from(APPROVAL_DOMAIN);
    bytes.extend_from_slice(&approval_challenge_sha256);
    bytes.extend_from_slice(WORKSPACE_PHYSICAL_APPROVAL_SOURCE.as_bytes());
    WorkspacePhysicalApproval {
        challenge_sha256: approval_challenge_sha256,
        source: WORKSPACE_PHYSICAL_APPROVAL_SOURCE,
        approval_sha256: sha256_bytes(&bytes),
    }
}

pub fn build_workspace_run_binding(
    receipt: &ProjectBuildReceipt,
    candidate_sha256: [u8; 32],
    candidate_byte_len: usize,
    import_list_observed: bool,
    import_count: usize,
) -> Result<WorkspaceRunBinding, WorkspaceRunDenied> {
    validate_build_receipt(receipt).map_err(|_| WorkspaceRunDenied::ReceiptInvalid)?;
    if !receipt.reproducible || !receipt.host_build_attested {
        return Err(WorkspaceRunDenied::ReceiptNotReproducible);
    }
    if receipt.authorizes_promotion
        || receipt.authorizes_install
        || receipt.authorizes_load
        || receipt.authorizes_execution
    {
        return Err(WorkspaceRunDenied::ReceiptAuthorityInvalid);
    }
    if receipt.candidate_sha256 != candidate_sha256
        || receipt.candidate_byte_len != candidate_byte_len
        || !receipt.candidate_wasm_valid
    {
        return Err(WorkspaceRunDenied::CandidateMismatch);
    }
    if !import_list_observed {
        return Err(WorkspaceRunDenied::ImportListNotObserved);
    }
    if import_count != 0 {
        return Err(WorkspaceRunDenied::ImportsNotEmpty);
    }
    let grant = evaluate_observed_wasm_import_grant(
        &WasmImportGrantInput {
            service_id: Some(WORKSPACE_SERVICE_ID),
            artifact_sha256_present: true,
            requested_imports: &[],
            policy_allows_beyond_env: false,
        },
        true,
    );
    if !grant.performed || grant.authorized_import_count != 0 {
        return Err(WorkspaceRunDenied::ImportGrantDenied);
    }
    let mut binding = WorkspaceRunBinding {
        project_id: receipt.snapshot.project_id.bytes(),
        project_revision_sha256: receipt.snapshot.project_revision_sha256,
        source_tree_sha256: receipt.snapshot.source_tree_sha256,
        cargo_lock_sha256: receipt.snapshot.cargo_lock_sha256,
        input_manifest_sha256: receipt.snapshot.input_manifest_sha256,
        receipt_sha256: receipt.receipt_sha256,
        candidate_sha256,
        candidate_byte_len,
        import_list_sha256: workspace_import_list_sha256(),
        import_count,
        fuel_budget: WORKSPACE_FUEL_BUDGET,
        memory_limit_bytes: WORKSPACE_MEMORY_LIMIT_BYTES,
        instance_limit: WORKSPACE_INSTANCE_LIMIT,
        memory_count_limit: WORKSPACE_MEMORY_COUNT_LIMIT,
        table_limit: WORKSPACE_TABLE_LIMIT,
        approval_challenge_sha256: [0; 32],
    };
    binding.approval_challenge_sha256 = challenge_hash(&binding);
    Ok(binding)
}

fn challenge_hash(binding: &WorkspaceRunBinding) -> [u8; 32] {
    let mut bytes = alloc::vec::Vec::from(CHALLENGE_DOMAIN);
    bytes.extend_from_slice(&binding.project_id);
    bytes.extend_from_slice(&binding.project_revision_sha256);
    bytes.extend_from_slice(&binding.source_tree_sha256);
    bytes.extend_from_slice(&binding.cargo_lock_sha256);
    bytes.extend_from_slice(&binding.input_manifest_sha256);
    bytes.extend_from_slice(&binding.receipt_sha256);
    bytes.extend_from_slice(&binding.candidate_sha256);
    bytes.extend_from_slice(&(binding.candidate_byte_len as u64).to_le_bytes());
    bytes.extend_from_slice(&binding.import_list_sha256);
    bytes.extend_from_slice(&(binding.import_count as u64).to_le_bytes());
    bytes.extend_from_slice(WORKSPACE_SERVICE_ID.as_bytes());
    bytes.extend_from_slice(WORKSPACE_ENTRYPOINT.as_bytes());
    bytes.extend_from_slice(&binding.fuel_budget.to_le_bytes());
    bytes.extend_from_slice(&(binding.memory_limit_bytes as u64).to_le_bytes());
    bytes.extend_from_slice(&(binding.instance_limit as u64).to_le_bytes());
    bytes.extend_from_slice(&(binding.memory_count_limit as u64).to_le_bytes());
    bytes.extend_from_slice(&(binding.table_limit as u64).to_le_bytes());
    sha256_bytes(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        project_build::{
            build_receipt, build_snapshot, pinned_toolchain_manifest_sha256, BuildRun,
            BuildSourceFile,
        },
        project_workspace::ProjectId,
    };
    use alloc::{string::String, vec};

    fn receipt() -> ProjectBuildReceipt {
        let lock = sha256_bytes(b"lock");
        let snapshot = build_snapshot(
            ProjectId::new([1; 16]),
            [2; 32],
            [3; 32],
            lock,
            vec![
                BuildSourceFile {
                    path: String::from("Cargo.lock"),
                    byte_len: 4,
                    blob_sha256: lock,
                },
                BuildSourceFile {
                    path: String::from("Cargo.toml"),
                    byte_len: 5,
                    blob_sha256: [4; 32],
                },
            ],
            vec![],
        )
        .unwrap();
        let run = BuildRun {
            exit_code: 0,
            stdout_sha256: [5; 32],
            stderr_sha256: [6; 32],
            output_byte_len: 8,
            output_sha256: [7; 32],
        };
        build_receipt(
            snapshot,
            pinned_toolchain_manifest_sha256(),
            run,
            run,
            [7; 32],
            8,
            true,
        )
        .unwrap()
    }

    #[test]
    fn observed_empty_import_surface_builds_non_authorizing_binding() {
        let receipt = receipt();
        let binding = build_workspace_run_binding(&receipt, [7; 32], 8, true, 0).unwrap();
        assert_eq!(binding.import_count, 0);
        assert_eq!(binding.import_list_sha256, workspace_import_list_sha256());
        assert_ne!(binding.approval_challenge_sha256, [0; 32]);
        let approval = build_workspace_pointer_approval(&binding);
        assert_eq!(approval.challenge_sha256, binding.approval_challenge_sha256);
        assert_ne!(approval.approval_sha256, [0; 32]);
    }

    #[test]
    fn absent_or_nonempty_import_surface_is_denied() {
        let receipt = receipt();
        assert_eq!(
            build_workspace_run_binding(&receipt, [7; 32], 8, false, 0),
            Err(WorkspaceRunDenied::ImportListNotObserved)
        );
        assert_eq!(
            build_workspace_run_binding(&receipt, [7; 32], 8, true, 1),
            Err(WorkspaceRunDenied::ImportsNotEmpty)
        );
    }
}
