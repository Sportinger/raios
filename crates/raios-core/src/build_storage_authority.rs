//! Second-stage storage authority for one already-authorized WASI build job.
//!
//! Core binds content identities and resource ceilings only. Physical offsets
//! appear solely as untrusted claims to the pre-I/O commit evaluator; they are
//! never carried by [`BuildStorageAuthority`]. Both actual BuildFS manifests
//! are canonically revalidated and rehashed against the ticket pins. That
//! closes the EmptyManifest seam: a placeholder or unrelated manifest cannot
//! acquire the ticket's storage authority merely because the ticket exists.

use alloc::vec::Vec;

use crate::{
    authorized_build_job::AuthorizedBuildJob,
    build_guest_class::BuildGuestClassV1,
    buildfs_manifest::{BuildFsManifest, BuildFsManifestError, BUILD_FS_CHUNK_SIZE},
    scoped_wasi_artifact_egress::WasiArtifactEgressPlan,
    sha256_bytes,
};

const BUILD_MOUNT_BUDGET_V1_DOMAIN: &str = "raios.build_mount_budget.v1";
const BUILD_STORAGE_JOB_BINDING_V1_DOMAIN: &str = "raios.build_storage_job_binding.v1";
const BUILD_OUTPUT_COMMIT_CLAIMS_V1_DOMAIN: &str = "raios.build_output_commit_claims.v1";

const MAX_TOTAL_MOUNT_BYTES_PER_MOUNT: u64 = 128 * 1024 * 1024;
const MAX_FILE_COUNT_PER_MOUNT: u32 = 4_096;
const MAX_CHUNK_COUNT_PER_MOUNT: u32 = 4_096;
const MAX_PATH_LEN_BYTES: u32 = 4_096;
const MAX_OUTPUT_BYTES: u64 = 64 * 1024 * 1024;

/// Fixed-width marker for leases targeting the inert WASI build-output store.
pub const BUILD_OUTPUT_LEASE_TARGET_MARKER_V1: [u8; 16] = *b"RAIOS_BUILD_OUT1";

/// Sibling policy for the two read mounts and the output lease.
///
/// The 128-MiB per-mount ceiling leaves substantial room above the measured
/// 71.1-MiB sysroot. Its roughly 40 files and 1,200 64-KiB chunks fit within
/// the 4,096-entry ceilings without making malformed manifests unbounded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildMountBudgetV1 {
    pub max_total_mount_bytes_per_mount: u64,
    pub max_file_count_per_mount: u32,
    pub max_chunk_count_per_mount: u32,
    pub max_path_len_bytes: u32,
    pub max_output_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildMountBudgetValidationError {
    ZeroMaxTotalMountBytesPerMount,
    MaxTotalMountBytesPerMountExceedsPolicy { actual: u64, maximum: u64 },
    ZeroMaxFileCountPerMount,
    MaxFileCountPerMountExceedsPolicy { actual: u32, maximum: u32 },
    ZeroMaxChunkCountPerMount,
    MaxChunkCountPerMountExceedsPolicy { actual: u32, maximum: u32 },
    ZeroMaxPathLenBytes,
    MaxPathLenBytesExceedsPolicy { actual: u32, maximum: u32 },
    ZeroMaxOutputBytes,
    MaxOutputBytesExceedsPolicy { actual: u64, maximum: u64 },
}

impl BuildMountBudgetValidationError {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::ZeroMaxTotalMountBytesPerMount => "zero_max_total_mount_bytes_per_mount",
            Self::MaxTotalMountBytesPerMountExceedsPolicy { .. } => {
                "max_total_mount_bytes_per_mount_exceeds_policy"
            }
            Self::ZeroMaxFileCountPerMount => "zero_max_file_count_per_mount",
            Self::MaxFileCountPerMountExceedsPolicy { .. } => {
                "max_file_count_per_mount_exceeds_policy"
            }
            Self::ZeroMaxChunkCountPerMount => "zero_max_chunk_count_per_mount",
            Self::MaxChunkCountPerMountExceedsPolicy { .. } => {
                "max_chunk_count_per_mount_exceeds_policy"
            }
            Self::ZeroMaxPathLenBytes => "zero_max_path_len_bytes",
            Self::MaxPathLenBytesExceedsPolicy { .. } => "max_path_len_bytes_exceeds_policy",
            Self::ZeroMaxOutputBytes => "zero_max_output_bytes",
            Self::MaxOutputBytesExceedsPolicy { .. } => "max_output_bytes_exceeds_policy",
        }
    }
}

impl BuildMountBudgetV1 {
    pub const fn validate(&self) -> Result<(), BuildMountBudgetValidationError> {
        if self.max_total_mount_bytes_per_mount == 0 {
            return Err(BuildMountBudgetValidationError::ZeroMaxTotalMountBytesPerMount);
        }
        if self.max_total_mount_bytes_per_mount > MAX_TOTAL_MOUNT_BYTES_PER_MOUNT {
            return Err(
                BuildMountBudgetValidationError::MaxTotalMountBytesPerMountExceedsPolicy {
                    actual: self.max_total_mount_bytes_per_mount,
                    maximum: MAX_TOTAL_MOUNT_BYTES_PER_MOUNT,
                },
            );
        }
        if self.max_file_count_per_mount == 0 {
            return Err(BuildMountBudgetValidationError::ZeroMaxFileCountPerMount);
        }
        if self.max_file_count_per_mount > MAX_FILE_COUNT_PER_MOUNT {
            return Err(
                BuildMountBudgetValidationError::MaxFileCountPerMountExceedsPolicy {
                    actual: self.max_file_count_per_mount,
                    maximum: MAX_FILE_COUNT_PER_MOUNT,
                },
            );
        }
        if self.max_chunk_count_per_mount == 0 {
            return Err(BuildMountBudgetValidationError::ZeroMaxChunkCountPerMount);
        }
        if self.max_chunk_count_per_mount > MAX_CHUNK_COUNT_PER_MOUNT {
            return Err(
                BuildMountBudgetValidationError::MaxChunkCountPerMountExceedsPolicy {
                    actual: self.max_chunk_count_per_mount,
                    maximum: MAX_CHUNK_COUNT_PER_MOUNT,
                },
            );
        }
        if self.max_path_len_bytes == 0 {
            return Err(BuildMountBudgetValidationError::ZeroMaxPathLenBytes);
        }
        if self.max_path_len_bytes > MAX_PATH_LEN_BYTES {
            return Err(
                BuildMountBudgetValidationError::MaxPathLenBytesExceedsPolicy {
                    actual: self.max_path_len_bytes,
                    maximum: MAX_PATH_LEN_BYTES,
                },
            );
        }
        if self.max_output_bytes == 0 {
            return Err(BuildMountBudgetValidationError::ZeroMaxOutputBytes);
        }
        if self.max_output_bytes > MAX_OUTPUT_BYTES {
            return Err(
                BuildMountBudgetValidationError::MaxOutputBytesExceedsPolicy {
                    actual: self.max_output_bytes,
                    maximum: MAX_OUTPUT_BYTES,
                },
            );
        }
        Ok(())
    }
}

pub const RUSTC_BUILD_MOUNT_BUDGET_V1: BuildMountBudgetV1 = BuildMountBudgetV1 {
    max_total_mount_bytes_per_mount: MAX_TOTAL_MOUNT_BYTES_PER_MOUNT,
    max_file_count_per_mount: MAX_FILE_COUNT_PER_MOUNT,
    max_chunk_count_per_mount: MAX_CHUNK_COUNT_PER_MOUNT,
    max_path_len_bytes: MAX_PATH_LEN_BYTES,
    max_output_bytes: MAX_OUTPUT_BYTES,
};

pub fn build_mount_budget_canonical_bytes(budget: &BuildMountBudgetV1) -> Vec<u8> {
    let mut bytes = Vec::new();
    write_bytes(&mut bytes, BUILD_MOUNT_BUDGET_V1_DOMAIN.as_bytes());
    bytes.extend_from_slice(&budget.max_total_mount_bytes_per_mount.to_le_bytes());
    bytes.extend_from_slice(&budget.max_file_count_per_mount.to_le_bytes());
    bytes.extend_from_slice(&budget.max_chunk_count_per_mount.to_le_bytes());
    bytes.extend_from_slice(&budget.max_path_len_bytes.to_le_bytes());
    bytes.extend_from_slice(&budget.max_output_bytes.to_le_bytes());
    bytes
}

pub fn build_mount_budget_sha256(budget: &BuildMountBudgetV1) -> [u8; 32] {
    sha256_bytes(&build_mount_budget_canonical_bytes(budget))
}

/// Kernel-minted, logical output destination. It contains no physical offset.
#[derive(Debug, Eq, PartialEq)]
pub struct OutputLeaseDescriptorV1 {
    lease_id: u64,
    store_instance_id: u64,
    store_generation: u64,
    max_bytes: u64,
    target_marker: [u8; 16],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputLeaseValidationError {
    ZeroLeaseId,
    ZeroStoreInstanceId,
    ZeroStoreGeneration,
    ZeroMaxBytes,
    MaxBytesExceedsBudget { actual: u64, maximum: u64 },
    MaxBytesExceedsGuestOutQuota { actual: u64, maximum: u64 },
    TargetMarkerMismatch,
}

impl OutputLeaseValidationError {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::ZeroLeaseId => "zero_output_lease_id",
            Self::ZeroStoreInstanceId => "zero_output_store_instance_id",
            Self::ZeroStoreGeneration => "zero_output_store_generation",
            Self::ZeroMaxBytes => "zero_output_lease_max_bytes",
            Self::MaxBytesExceedsBudget { .. } => "output_lease_max_bytes_exceeds_budget",
            Self::MaxBytesExceedsGuestOutQuota { .. } => {
                "output_lease_max_bytes_exceeds_guest_out_quota"
            }
            Self::TargetMarkerMismatch => "output_lease_target_marker_mismatch",
        }
    }
}

impl OutputLeaseDescriptorV1 {
    /// Records values minted by the kernel lease allocator. This descriptor
    /// grants nothing until [`authorize_build_storage`] validates and binds it.
    pub const fn kernel_minted(
        lease_id: u64,
        store_instance_id: u64,
        store_generation: u64,
        max_bytes: u64,
        target_marker: [u8; 16],
    ) -> Self {
        Self {
            lease_id,
            store_instance_id,
            store_generation,
            max_bytes,
            target_marker,
        }
    }

    pub const fn validate(
        &self,
        budget: &BuildMountBudgetV1,
        guest_class: &BuildGuestClassV1,
    ) -> Result<(), OutputLeaseValidationError> {
        if self.lease_id == 0 {
            return Err(OutputLeaseValidationError::ZeroLeaseId);
        }
        if self.store_instance_id == 0 {
            return Err(OutputLeaseValidationError::ZeroStoreInstanceId);
        }
        if self.store_generation == 0 {
            return Err(OutputLeaseValidationError::ZeroStoreGeneration);
        }
        if self.max_bytes == 0 {
            return Err(OutputLeaseValidationError::ZeroMaxBytes);
        }
        if self.max_bytes > budget.max_output_bytes {
            return Err(OutputLeaseValidationError::MaxBytesExceedsBudget {
                actual: self.max_bytes,
                maximum: budget.max_output_bytes,
            });
        }
        if self.max_bytes > guest_class.out_max_bytes {
            return Err(OutputLeaseValidationError::MaxBytesExceedsGuestOutQuota {
                actual: self.max_bytes,
                maximum: guest_class.out_max_bytes,
            });
        }
        let mut marker_index = 0usize;
        while marker_index < BUILD_OUTPUT_LEASE_TARGET_MARKER_V1.len() {
            if self.target_marker[marker_index] != BUILD_OUTPUT_LEASE_TARGET_MARKER_V1[marker_index]
            {
                return Err(OutputLeaseValidationError::TargetMarkerMismatch);
            }
            marker_index += 1;
        }
        Ok(())
    }

    pub const fn lease_id(&self) -> u64 {
        self.lease_id
    }

    pub const fn store_instance_id(&self) -> u64 {
        self.store_instance_id
    }

    pub const fn store_generation(&self) -> u64 {
        self.store_generation
    }

    pub const fn max_bytes(&self) -> u64 {
        self.max_bytes
    }

    pub const fn target_marker(&self) -> [u8; 16] {
        self.target_marker
    }
}

/// Per-instantiation nonce whose ownership can cross a run boundary once.
///
/// The kernel is responsible for freshness. Core rejects the zero sentinel
/// and makes extraction consume the non-`Copy`, non-`Clone` value.
///
/// ```compile_fail
/// use raios_core::build_storage_authority::BuildRunNonce;
///
/// let nonce = BuildRunNonce::kernel_minted(7).unwrap();
/// let _first_use = nonce.into_value();
/// let _reuse = nonce.into_value();
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct BuildRunNonce {
    value: u64,
}

impl BuildRunNonce {
    pub const fn kernel_minted(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self { value })
        }
    }

    pub const fn into_value(self) -> u64 {
        self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildStorageMount {
    Sysroot,
    Src,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildMountMeasureField {
    TotalBytes,
    FileCount,
    ChunkCount,
    PathLength,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildStorageDenied {
    SysrootManifestInvalid {
        rejection: BuildFsManifestError,
    },
    SrcManifestInvalid {
        rejection: BuildFsManifestError,
    },
    SysrootManifestHashMismatch {
        expected: [u8; 32],
        actual: [u8; 32],
    },
    SrcManifestHashMismatch {
        expected: [u8; 32],
        actual: [u8; 32],
    },
    BudgetInvalid {
        rejection: BuildMountBudgetValidationError,
    },
    ManifestMeasureOverflow {
        mount: BuildStorageMount,
        field: BuildMountMeasureField,
    },
    ManifestBudgetExceeded {
        mount: BuildStorageMount,
        field: BuildMountMeasureField,
        actual: u64,
        maximum: u64,
    },
    LeaseInvalid {
        rejection: OutputLeaseValidationError,
    },
}

impl BuildStorageDenied {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::SysrootManifestInvalid { .. } => "sysroot_manifest_invalid",
            Self::SrcManifestInvalid { .. } => "src_manifest_invalid",
            Self::SysrootManifestHashMismatch { .. } => "sysroot_manifest_hash_mismatch",
            Self::SrcManifestHashMismatch { .. } => "src_manifest_hash_mismatch",
            Self::BudgetInvalid { rejection } => rejection.reason(),
            Self::ManifestMeasureOverflow {
                mount: BuildStorageMount::Sysroot,
                field: BuildMountMeasureField::TotalBytes,
            } => "sysroot_manifest_total_bytes_overflow",
            Self::ManifestMeasureOverflow {
                mount: BuildStorageMount::Sysroot,
                field: BuildMountMeasureField::FileCount,
            } => "sysroot_manifest_file_count_overflow",
            Self::ManifestMeasureOverflow {
                mount: BuildStorageMount::Sysroot,
                field: BuildMountMeasureField::ChunkCount,
            } => "sysroot_manifest_chunk_count_overflow",
            Self::ManifestMeasureOverflow {
                mount: BuildStorageMount::Sysroot,
                field: BuildMountMeasureField::PathLength,
            } => "sysroot_manifest_path_length_overflow",
            Self::ManifestMeasureOverflow {
                mount: BuildStorageMount::Src,
                field: BuildMountMeasureField::TotalBytes,
            } => "src_manifest_total_bytes_overflow",
            Self::ManifestMeasureOverflow {
                mount: BuildStorageMount::Src,
                field: BuildMountMeasureField::FileCount,
            } => "src_manifest_file_count_overflow",
            Self::ManifestMeasureOverflow {
                mount: BuildStorageMount::Src,
                field: BuildMountMeasureField::ChunkCount,
            } => "src_manifest_chunk_count_overflow",
            Self::ManifestMeasureOverflow {
                mount: BuildStorageMount::Src,
                field: BuildMountMeasureField::PathLength,
            } => "src_manifest_path_length_overflow",
            Self::ManifestBudgetExceeded {
                mount: BuildStorageMount::Sysroot,
                field: BuildMountMeasureField::TotalBytes,
                ..
            } => "sysroot_manifest_total_bytes_exceeds_budget",
            Self::ManifestBudgetExceeded {
                mount: BuildStorageMount::Sysroot,
                field: BuildMountMeasureField::FileCount,
                ..
            } => "sysroot_manifest_file_count_exceeds_budget",
            Self::ManifestBudgetExceeded {
                mount: BuildStorageMount::Sysroot,
                field: BuildMountMeasureField::ChunkCount,
                ..
            } => "sysroot_manifest_chunk_count_exceeds_budget",
            Self::ManifestBudgetExceeded {
                mount: BuildStorageMount::Sysroot,
                field: BuildMountMeasureField::PathLength,
                ..
            } => "sysroot_manifest_path_length_exceeds_budget",
            Self::ManifestBudgetExceeded {
                mount: BuildStorageMount::Src,
                field: BuildMountMeasureField::TotalBytes,
                ..
            } => "src_manifest_total_bytes_exceeds_budget",
            Self::ManifestBudgetExceeded {
                mount: BuildStorageMount::Src,
                field: BuildMountMeasureField::FileCount,
                ..
            } => "src_manifest_file_count_exceeds_budget",
            Self::ManifestBudgetExceeded {
                mount: BuildStorageMount::Src,
                field: BuildMountMeasureField::ChunkCount,
                ..
            } => "src_manifest_chunk_count_exceeds_budget",
            Self::ManifestBudgetExceeded {
                mount: BuildStorageMount::Src,
                field: BuildMountMeasureField::PathLength,
                ..
            } => "src_manifest_path_length_exceeds_budget",
            Self::LeaseInvalid { rejection } => rejection.reason(),
        }
    }
}

/// Fully validated logical storage authority for one build job.
///
/// No unchecked constructor exists, and this type is deliberately neither
/// `Copy` nor `Clone`.
///
/// ```compile_fail
/// use raios_core::build_storage_authority::BuildStorageAuthority;
///
/// let forged = BuildStorageAuthority {};
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct BuildStorageAuthority {
    guest_class_sha256: [u8; 32],
    compiler_artifact_sha256: [u8; 32],
    job_manifest_sha256: [u8; 32],
    sysroot_mount_manifest_sha256: [u8; 32],
    src_mount_manifest_sha256: [u8; 32],
    budget: BuildMountBudgetV1,
    budget_sha256: [u8; 32],
    output_lease: OutputLeaseDescriptorV1,
    job_binding_sha256: [u8; 32],
}

impl BuildStorageAuthority {
    pub const fn guest_class_sha256(&self) -> [u8; 32] {
        self.guest_class_sha256
    }

    pub const fn compiler_artifact_sha256(&self) -> [u8; 32] {
        self.compiler_artifact_sha256
    }

    pub const fn job_manifest_sha256(&self) -> [u8; 32] {
        self.job_manifest_sha256
    }

    pub const fn sysroot_mount_manifest_sha256(&self) -> [u8; 32] {
        self.sysroot_mount_manifest_sha256
    }

    pub const fn src_mount_manifest_sha256(&self) -> [u8; 32] {
        self.src_mount_manifest_sha256
    }

    pub const fn budget(&self) -> &BuildMountBudgetV1 {
        &self.budget
    }

    pub const fn budget_sha256(&self) -> [u8; 32] {
        self.budget_sha256
    }

    pub const fn output_lease(&self) -> &OutputLeaseDescriptorV1 {
        &self.output_lease
    }

    pub const fn job_binding_sha256(&self) -> [u8; 32] {
        self.job_binding_sha256
    }
}

pub fn authorize_build_storage(
    job: &AuthorizedBuildJob,
    sysroot_manifest: &BuildFsManifest,
    src_manifest: &BuildFsManifest,
    budget: BuildMountBudgetV1,
    output_lease: OutputLeaseDescriptorV1,
) -> Result<BuildStorageAuthority, BuildStorageDenied> {
    sysroot_manifest
        .validate()
        .map_err(|rejection| BuildStorageDenied::SysrootManifestInvalid { rejection })?;
    src_manifest
        .validate()
        .map_err(|rejection| BuildStorageDenied::SrcManifestInvalid { rejection })?;

    let sysroot_mount_manifest_sha256 = sysroot_manifest
        .sha256()
        .map_err(|rejection| BuildStorageDenied::SysrootManifestInvalid { rejection })?;
    let src_mount_manifest_sha256 = src_manifest
        .sha256()
        .map_err(|rejection| BuildStorageDenied::SrcManifestInvalid { rejection })?;

    if sysroot_mount_manifest_sha256 != job.sysroot_mount_manifest_sha256() {
        return Err(BuildStorageDenied::SysrootManifestHashMismatch {
            expected: job.sysroot_mount_manifest_sha256(),
            actual: sysroot_mount_manifest_sha256,
        });
    }
    if src_mount_manifest_sha256 != job.src_mount_manifest_sha256() {
        return Err(BuildStorageDenied::SrcManifestHashMismatch {
            expected: job.src_mount_manifest_sha256(),
            actual: src_mount_manifest_sha256,
        });
    }

    budget
        .validate()
        .map_err(|rejection| BuildStorageDenied::BudgetInvalid { rejection })?;
    validate_manifest_budget(sysroot_manifest, &budget, BuildStorageMount::Sysroot)?;
    validate_manifest_budget(src_manifest, &budget, BuildStorageMount::Src)?;

    output_lease
        .validate(&budget, job.guest_class())
        .map_err(|rejection| BuildStorageDenied::LeaseInvalid { rejection })?;

    let budget_sha256 = build_mount_budget_sha256(&budget);
    let job_binding_sha256 = build_storage_job_binding_sha256(
        job.guest_class_sha256(),
        job.compiler_artifact_sha256(),
        job.job_manifest_sha256(),
        sysroot_mount_manifest_sha256,
        src_mount_manifest_sha256,
        budget_sha256,
        &output_lease,
    );

    Ok(BuildStorageAuthority {
        guest_class_sha256: job.guest_class_sha256(),
        compiler_artifact_sha256: job.compiler_artifact_sha256(),
        job_manifest_sha256: job.job_manifest_sha256(),
        sysroot_mount_manifest_sha256,
        src_mount_manifest_sha256,
        budget,
        budget_sha256,
        output_lease,
        job_binding_sha256,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopedBuildOutputCommitInput {
    pub job_binding_sha256: [u8; 32],
    pub lease_id: u64,
    pub store_instance_id: u64,
    pub store_generation: u64,
    pub lease_max_bytes: u64,
    pub lease_target_marker: [u8; 16],
    /// Absolute byte offset selected by ARTSTOR under its allocator lock.
    pub span_offset: u64,
    pub span_len: u64,
    /// Absolute byte bounds of the live ARTSTOR region.
    pub artstor_region_offset: u64,
    pub artstor_region_len: u64,
    pub alignment: u64,
    pub bundle_len: u64,
    pub output_manifest_sha256: [u8; 32],
    /// Number of 64-KiB storage chunks covering the complete output bundle.
    pub chunk_count: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildOutputCommitDenied {
    EgressJobBindingMismatch,
    ClaimedJobBindingMismatch,
    EgressLeaseIdMismatch,
    ClaimedLeaseIdMismatch,
    ClaimedStoreInstanceIdMismatch,
    ClaimedStoreGenerationMismatch,
    ClaimedLeaseMaxBytesMismatch,
    ClaimedLeaseTargetMarkerMismatch,
    ArtstorRegionBoundsOverflow,
    OutputSpanOverflow,
    OutputSpanOutOfArtstor,
    EmptyBundle,
    BundleLengthExceedsLease,
    OutputSpanLengthExceedsLease,
    BundleLengthEgressMismatch,
    BundleDoesNotFitOutputSpan,
    BundleManifestSha256Mismatch,
    BundleChunkCountMismatch,
    InvalidAlignment,
    OutputSpanOffsetMisaligned,
    OutputSpanLengthMisaligned,
}

impl BuildOutputCommitDenied {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::EgressJobBindingMismatch => "egress_job_binding_sha256_mismatch",
            Self::ClaimedJobBindingMismatch => "claimed_job_binding_sha256_mismatch",
            Self::EgressLeaseIdMismatch => "egress_lease_id_mismatch",
            Self::ClaimedLeaseIdMismatch => "claimed_lease_id_mismatch",
            Self::ClaimedStoreInstanceIdMismatch => "claimed_store_instance_id_mismatch",
            Self::ClaimedStoreGenerationMismatch => "claimed_store_generation_mismatch",
            Self::ClaimedLeaseMaxBytesMismatch => "claimed_lease_max_bytes_mismatch",
            Self::ClaimedLeaseTargetMarkerMismatch => "claimed_lease_target_marker_mismatch",
            Self::ArtstorRegionBoundsOverflow => "artstor_region_bounds_overflow",
            Self::OutputSpanOverflow => "output_span_overflow",
            Self::OutputSpanOutOfArtstor => "output_span_out_of_artstor",
            Self::EmptyBundle => "empty_output_bundle",
            Self::BundleLengthExceedsLease => "bundle_length_exceeds_lease",
            Self::OutputSpanLengthExceedsLease => "output_span_length_exceeds_lease",
            Self::BundleLengthEgressMismatch => "bundle_length_egress_mismatch",
            Self::BundleDoesNotFitOutputSpan => "bundle_does_not_fit_output_span",
            Self::BundleManifestSha256Mismatch => "bundle_manifest_sha256_mismatch",
            Self::BundleChunkCountMismatch => "bundle_chunk_count_mismatch",
            Self::InvalidAlignment => "invalid_output_alignment",
            Self::OutputSpanOffsetMisaligned => "output_span_offset_misaligned",
            Self::OutputSpanLengthMisaligned => "output_span_length_misaligned",
        }
    }
}

/// Opaque permission for ARTSTOR to begin the checked write/readback/record
/// transaction. Constructible only by [`evaluate_scoped_build_output_commit`].
///
/// ```compile_fail
/// use raios_core::build_storage_authority::AuthorizedBuildOutputCommit;
///
/// let forged = AuthorizedBuildOutputCommit {};
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct AuthorizedBuildOutputCommit {
    job_binding_sha256: [u8; 32],
    lease_id: u64,
    commit_claims_sha256: [u8; 32],
    bundle_len: u64,
    output_manifest_sha256: [u8; 32],
    chunk_count: u64,
}

impl AuthorizedBuildOutputCommit {
    pub const fn job_binding_sha256(&self) -> [u8; 32] {
        self.job_binding_sha256
    }

    pub const fn lease_id(&self) -> u64 {
        self.lease_id
    }

    /// Domain-separated digest of every validated pre-I/O claim, including
    /// the store-local reservation. No physical address escapes this digest.
    pub const fn commit_claims_sha256(&self) -> [u8; 32] {
        self.commit_claims_sha256
    }

    pub const fn bundle_len(&self) -> u64 {
        self.bundle_len
    }

    pub const fn output_manifest_sha256(&self) -> [u8; 32] {
        self.output_manifest_sha256
    }

    pub const fn chunk_count(&self) -> u64 {
        self.chunk_count
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ScopedBuildOutputCommitDecision {
    Authorized(AuthorizedBuildOutputCommit),
    Denied(BuildOutputCommitDenied),
}

pub fn evaluate_scoped_build_output_commit(
    input: &ScopedBuildOutputCommitInput,
    authority: &BuildStorageAuthority,
    egress: &WasiArtifactEgressPlan,
) -> ScopedBuildOutputCommitDecision {
    if egress.job_binding_sha256() != authority.job_binding_sha256() {
        return commit_denied(BuildOutputCommitDenied::EgressJobBindingMismatch);
    }
    if input.job_binding_sha256 != authority.job_binding_sha256() {
        return commit_denied(BuildOutputCommitDenied::ClaimedJobBindingMismatch);
    }

    let lease = authority.output_lease();
    if egress.output_lease_id() != lease.lease_id() {
        return commit_denied(BuildOutputCommitDenied::EgressLeaseIdMismatch);
    }
    if input.lease_id != lease.lease_id() {
        return commit_denied(BuildOutputCommitDenied::ClaimedLeaseIdMismatch);
    }
    if input.store_instance_id != lease.store_instance_id() {
        return commit_denied(BuildOutputCommitDenied::ClaimedStoreInstanceIdMismatch);
    }
    if input.store_generation != lease.store_generation() {
        return commit_denied(BuildOutputCommitDenied::ClaimedStoreGenerationMismatch);
    }
    if input.lease_max_bytes != lease.max_bytes() {
        return commit_denied(BuildOutputCommitDenied::ClaimedLeaseMaxBytesMismatch);
    }
    if input.lease_target_marker != lease.target_marker() {
        return commit_denied(BuildOutputCommitDenied::ClaimedLeaseTargetMarkerMismatch);
    }

    if input
        .artstor_region_offset
        .checked_add(input.artstor_region_len)
        .is_none()
    {
        return commit_denied(BuildOutputCommitDenied::ArtstorRegionBoundsOverflow);
    }
    if input.span_offset.checked_add(input.span_len).is_none() {
        return commit_denied(BuildOutputCommitDenied::OutputSpanOverflow);
    }
    if !span_inside_region(
        input.artstor_region_offset,
        input.artstor_region_len,
        input.span_offset,
        input.span_len,
    ) {
        return commit_denied(BuildOutputCommitDenied::OutputSpanOutOfArtstor);
    }

    if input.bundle_len == 0 {
        return commit_denied(BuildOutputCommitDenied::EmptyBundle);
    }
    if input.bundle_len > lease.max_bytes() {
        return commit_denied(BuildOutputCommitDenied::BundleLengthExceedsLease);
    }
    if input.span_len > lease.max_bytes() {
        return commit_denied(BuildOutputCommitDenied::OutputSpanLengthExceedsLease);
    }
    if input.bundle_len != egress.logical_content_size() {
        return commit_denied(BuildOutputCommitDenied::BundleLengthEgressMismatch);
    }
    if input.bundle_len > input.span_len {
        return commit_denied(BuildOutputCommitDenied::BundleDoesNotFitOutputSpan);
    }

    if input.output_manifest_sha256 != egress.output_manifest_sha256() {
        return commit_denied(BuildOutputCommitDenied::BundleManifestSha256Mismatch);
    }
    let expected_chunk_count = ((input.bundle_len - 1) / BUILD_FS_CHUNK_SIZE) + 1;
    if input.chunk_count != expected_chunk_count {
        return commit_denied(BuildOutputCommitDenied::BundleChunkCountMismatch);
    }

    if input.alignment == 0 || !input.alignment.is_power_of_two() {
        return commit_denied(BuildOutputCommitDenied::InvalidAlignment);
    }
    if input.span_offset % input.alignment != 0 {
        return commit_denied(BuildOutputCommitDenied::OutputSpanOffsetMisaligned);
    }
    if input.span_len % input.alignment != 0 {
        return commit_denied(BuildOutputCommitDenied::OutputSpanLengthMisaligned);
    }

    ScopedBuildOutputCommitDecision::Authorized(AuthorizedBuildOutputCommit {
        job_binding_sha256: authority.job_binding_sha256(),
        lease_id: lease.lease_id(),
        commit_claims_sha256: build_output_commit_claims_sha256(input),
        bundle_len: input.bundle_len,
        output_manifest_sha256: input.output_manifest_sha256,
        chunk_count: input.chunk_count,
    })
}

fn validate_manifest_budget(
    manifest: &BuildFsManifest,
    budget: &BuildMountBudgetV1,
    mount: BuildStorageMount,
) -> Result<(), BuildStorageDenied> {
    let file_count = u64::try_from(manifest.files.len()).map_err(|_| {
        BuildStorageDenied::ManifestMeasureOverflow {
            mount,
            field: BuildMountMeasureField::FileCount,
        }
    })?;
    if file_count > u64::from(budget.max_file_count_per_mount) {
        return Err(BuildStorageDenied::ManifestBudgetExceeded {
            mount,
            field: BuildMountMeasureField::FileCount,
            actual: file_count,
            maximum: u64::from(budget.max_file_count_per_mount),
        });
    }

    let mut total_bytes = 0u64;
    let mut chunk_count = 0u64;
    let mut longest_path = 0u64;
    for directory in &manifest.directories {
        let path_len = u64::try_from(directory.path.len()).map_err(|_| {
            BuildStorageDenied::ManifestMeasureOverflow {
                mount,
                field: BuildMountMeasureField::PathLength,
            }
        })?;
        longest_path = core::cmp::max(longest_path, path_len);
    }
    for file in &manifest.files {
        total_bytes = total_bytes.checked_add(file.len).ok_or(
            BuildStorageDenied::ManifestMeasureOverflow {
                mount,
                field: BuildMountMeasureField::TotalBytes,
            },
        )?;
        let file_chunks = u64::try_from(file.chunks.len()).map_err(|_| {
            BuildStorageDenied::ManifestMeasureOverflow {
                mount,
                field: BuildMountMeasureField::ChunkCount,
            }
        })?;
        chunk_count = chunk_count.checked_add(file_chunks).ok_or(
            BuildStorageDenied::ManifestMeasureOverflow {
                mount,
                field: BuildMountMeasureField::ChunkCount,
            },
        )?;
        let path_len = u64::try_from(file.path.len()).map_err(|_| {
            BuildStorageDenied::ManifestMeasureOverflow {
                mount,
                field: BuildMountMeasureField::PathLength,
            }
        })?;
        longest_path = core::cmp::max(longest_path, path_len);
    }

    let checks = [
        (
            BuildMountMeasureField::TotalBytes,
            total_bytes,
            budget.max_total_mount_bytes_per_mount,
        ),
        (
            BuildMountMeasureField::ChunkCount,
            chunk_count,
            u64::from(budget.max_chunk_count_per_mount),
        ),
        (
            BuildMountMeasureField::PathLength,
            longest_path,
            u64::from(budget.max_path_len_bytes),
        ),
    ];
    for (field, actual, maximum) in checks {
        if actual > maximum {
            return Err(BuildStorageDenied::ManifestBudgetExceeded {
                mount,
                field,
                actual,
                maximum,
            });
        }
    }
    Ok(())
}

fn build_storage_job_binding_sha256(
    guest_class_sha256: [u8; 32],
    compiler_artifact_sha256: [u8; 32],
    job_manifest_sha256: [u8; 32],
    sysroot_mount_manifest_sha256: [u8; 32],
    src_mount_manifest_sha256: [u8; 32],
    budget_sha256: [u8; 32],
    output_lease: &OutputLeaseDescriptorV1,
) -> [u8; 32] {
    let mut bytes = Vec::new();
    write_bytes(&mut bytes, BUILD_STORAGE_JOB_BINDING_V1_DOMAIN.as_bytes());
    bytes.extend_from_slice(&guest_class_sha256);
    bytes.extend_from_slice(&compiler_artifact_sha256);
    bytes.extend_from_slice(&job_manifest_sha256);
    bytes.extend_from_slice(&sysroot_mount_manifest_sha256);
    bytes.extend_from_slice(&src_mount_manifest_sha256);
    bytes.extend_from_slice(&budget_sha256);
    bytes.extend_from_slice(&output_lease.lease_id.to_le_bytes());
    bytes.extend_from_slice(&output_lease.store_instance_id.to_le_bytes());
    bytes.extend_from_slice(&output_lease.store_generation.to_le_bytes());
    bytes.extend_from_slice(&output_lease.max_bytes.to_le_bytes());
    bytes.extend_from_slice(&output_lease.target_marker);
    sha256_bytes(&bytes)
}

fn build_output_commit_claims_sha256(input: &ScopedBuildOutputCommitInput) -> [u8; 32] {
    let mut bytes = Vec::new();
    write_bytes(&mut bytes, BUILD_OUTPUT_COMMIT_CLAIMS_V1_DOMAIN.as_bytes());
    bytes.extend_from_slice(&input.job_binding_sha256);
    bytes.extend_from_slice(&input.lease_id.to_le_bytes());
    bytes.extend_from_slice(&input.store_instance_id.to_le_bytes());
    bytes.extend_from_slice(&input.store_generation.to_le_bytes());
    bytes.extend_from_slice(&input.lease_max_bytes.to_le_bytes());
    bytes.extend_from_slice(&input.lease_target_marker);
    bytes.extend_from_slice(&input.span_offset.to_le_bytes());
    bytes.extend_from_slice(&input.span_len.to_le_bytes());
    bytes.extend_from_slice(&input.artstor_region_offset.to_le_bytes());
    bytes.extend_from_slice(&input.artstor_region_len.to_le_bytes());
    bytes.extend_from_slice(&input.alignment.to_le_bytes());
    bytes.extend_from_slice(&input.bundle_len.to_le_bytes());
    bytes.extend_from_slice(&input.output_manifest_sha256);
    bytes.extend_from_slice(&input.chunk_count.to_le_bytes());
    sha256_bytes(&bytes)
}

fn span_inside_region(
    region_offset: u64,
    region_len: u64,
    span_offset: u64,
    span_len: u64,
) -> bool {
    let Some(region_end) = region_offset.checked_add(region_len) else {
        return false;
    };
    let Some(span_end) = span_offset.checked_add(span_len) else {
        return false;
    };
    span_offset >= region_offset && span_end <= region_end
}

fn write_bytes(output: &mut Vec<u8>, value: &[u8]) {
    output.extend_from_slice(&(value.len() as u64).to_le_bytes());
    output.extend_from_slice(value);
}

fn commit_denied(rejection: BuildOutputCommitDenied) -> ScopedBuildOutputCommitDecision {
    ScopedBuildOutputCommitDecision::Denied(rejection)
}

#[cfg(test)]
mod tests {
    use alloc::{string::ToString, vec, vec::Vec};

    use super::*;
    use crate::{
        authorized_build_job::{AuthorizedBuildJob, AuthorizedBuildJobRequest},
        build_guest_class::RUSTC_BUILD_GUEST_CLASS_V1,
        buildfs_manifest::{BuildFsChunk, BuildFsDirectory, BuildFsFile},
        scoped_wasi_artifact_egress::{
            evaluate_scoped_wasi_artifact_egress, ScopedWasiArtifactEgress,
            ScopedWasiArtifactEgressDecision,
        },
        scoped_wasi_build_grant::{
            ScopedWasiBuildGrant, RUSTC_WASM_C6DCCF3E_CANONICAL_IMPORTS_SHA256,
        },
        sha256_hex,
        wasi_build_output::FrozenOutput,
        wasi_preview1_import_abi::RUSTC_WASM_C6DCCF3E_IMPORTS,
    };

    const COMPILER_SHA256: &str =
        "c6dccf3e5f01631b942a0a008b9f2f5312987e7d8590f8c61024cd00687a5791";
    const JOB_MANIFEST_SHA256: &str =
        "1111111111111111111111111111111111111111111111111111111111111111";

    fn chunk(len: u64, tag: u8) -> BuildFsChunk {
        BuildFsChunk {
            len,
            sha256: [tag; 32],
        }
    }

    fn manifest(root: &str, file_len: u64, tag: u8) -> BuildFsManifest {
        BuildFsManifest::new(
            vec![BuildFsDirectory {
                path: root.to_string(),
            }],
            vec![BuildFsFile {
                path: alloc::format!("{root}/input"),
                len: file_len,
                sha256: [tag; 32],
                chunks: if file_len == 0 {
                    vec![]
                } else {
                    vec![chunk(file_len, tag)]
                },
            }],
        )
        .unwrap()
    }

    fn job_for(
        sysroot_manifest_sha256: [u8; 32],
        src_manifest_sha256: [u8; 32],
    ) -> AuthorizedBuildJob {
        AuthorizedBuildJob::authorize(AuthorizedBuildJobRequest {
            wasi_grant: ScopedWasiBuildGrant {
                compiler_artifact_sha256: COMPILER_SHA256,
                job_manifest_sha256: JOB_MANIFEST_SHA256,
                inventory_imports_sha256: RUSTC_WASM_C6DCCF3E_CANONICAL_IMPORTS_SHA256,
                declared_imports: RUSTC_WASM_C6DCCF3E_IMPORTS,
            },
            observed_imports: RUSTC_WASM_C6DCCF3E_IMPORTS,
            guest_class: RUSTC_BUILD_GUEST_CLASS_V1,
            sysroot_mount_manifest_sha256: sysroot_manifest_sha256,
            src_mount_manifest_sha256: src_manifest_sha256,
        })
        .unwrap()
    }

    fn lease(
        lease_id: u64,
        store_instance_id: u64,
        store_generation: u64,
        max_bytes: u64,
        target_marker: [u8; 16],
    ) -> OutputLeaseDescriptorV1 {
        OutputLeaseDescriptorV1::kernel_minted(
            lease_id,
            store_instance_id,
            store_generation,
            max_bytes,
            target_marker,
        )
    }

    fn authorize_fixture(
        budget: BuildMountBudgetV1,
        output_lease: OutputLeaseDescriptorV1,
    ) -> BuildStorageAuthority {
        authorize_fixture_result(budget, output_lease).unwrap()
    }

    fn authorize_fixture_result(
        budget: BuildMountBudgetV1,
        output_lease: OutputLeaseDescriptorV1,
    ) -> Result<BuildStorageAuthority, BuildStorageDenied> {
        let sysroot = manifest("sysroot", 32, 1);
        let src = manifest("src", 16, 2);
        let job = job_for(sysroot.sha256().unwrap(), src.sha256().unwrap());
        authorize_build_storage(&job, &sysroot, &src, budget, output_lease)
    }

    fn default_authority() -> BuildStorageAuthority {
        authorize_fixture(
            RUSTC_BUILD_MOUNT_BUDGET_V1,
            lease(
                7,
                11,
                13,
                RUSTC_BUILD_MOUNT_BUDGET_V1.max_output_bytes,
                BUILD_OUTPUT_LEASE_TARGET_MARKER_V1,
            ),
        )
    }

    fn egress_for(
        authority: &BuildStorageAuthority,
        manifest_bytes: &[u8],
        logical_content_size: u64,
    ) -> WasiArtifactEgressPlan {
        let input = ScopedWasiArtifactEgress {
            run_one: FrozenOutput::from_manifest_bytes(manifest_bytes),
            run_two: FrozenOutput::from_manifest_bytes(manifest_bytes),
            run_one_exit_status: 0,
            run_two_exit_status: 0,
            run_one_logical_content_size: logical_content_size,
            run_two_logical_content_size: logical_content_size,
        };
        match evaluate_scoped_wasi_artifact_egress(&input, authority) {
            ScopedWasiArtifactEgressDecision::Planned(plan) => plan,
            ScopedWasiArtifactEgressDecision::Denied(rejection) => {
                panic!("fixture egress denied: {rejection:?}")
            }
        }
    }

    fn valid_commit_input(
        authority: &BuildStorageAuthority,
        egress: &WasiArtifactEgressPlan,
    ) -> ScopedBuildOutputCommitInput {
        let lease = authority.output_lease();
        ScopedBuildOutputCommitInput {
            job_binding_sha256: authority.job_binding_sha256(),
            lease_id: lease.lease_id(),
            store_instance_id: lease.store_instance_id(),
            store_generation: lease.store_generation(),
            lease_max_bytes: lease.max_bytes(),
            lease_target_marker: lease.target_marker(),
            span_offset: 4_096,
            span_len: BUILD_FS_CHUNK_SIZE,
            artstor_region_offset: 0,
            artstor_region_len: 128 * 1024 * 1024,
            alignment: 512,
            bundle_len: BUILD_FS_CHUNK_SIZE,
            output_manifest_sha256: egress.output_manifest_sha256(),
            chunk_count: 1,
        }
    }

    #[test]
    fn rustc_budget_is_valid_canonical_and_pinned() {
        assert_eq!(RUSTC_BUILD_MOUNT_BUDGET_V1.validate(), Ok(()));
        assert!(72 * 1024 * 1024 <= RUSTC_BUILD_MOUNT_BUDGET_V1.max_total_mount_bytes_per_mount);
        assert!(40 <= RUSTC_BUILD_MOUNT_BUDGET_V1.max_file_count_per_mount);
        assert!(1_200 <= RUSTC_BUILD_MOUNT_BUDGET_V1.max_chunk_count_per_mount);
        assert_eq!(
            sha256_hex(&build_mount_budget_sha256(&RUSTC_BUILD_MOUNT_BUDGET_V1)),
            *b"9db223b501f750baba30510b99d3d08ffd2717188bc093dfba0722274e383537"
        );
    }

    #[test]
    fn budget_validate_has_typed_zero_and_upper_bound_rejections() {
        let mut cases = Vec::new();

        let mut budget = RUSTC_BUILD_MOUNT_BUDGET_V1;
        budget.max_total_mount_bytes_per_mount = 0;
        cases.push((
            budget,
            BuildMountBudgetValidationError::ZeroMaxTotalMountBytesPerMount,
        ));
        budget = RUSTC_BUILD_MOUNT_BUDGET_V1;
        budget.max_total_mount_bytes_per_mount += 1;
        cases.push((
            budget,
            BuildMountBudgetValidationError::MaxTotalMountBytesPerMountExceedsPolicy {
                actual: MAX_TOTAL_MOUNT_BYTES_PER_MOUNT + 1,
                maximum: MAX_TOTAL_MOUNT_BYTES_PER_MOUNT,
            },
        ));
        budget = RUSTC_BUILD_MOUNT_BUDGET_V1;
        budget.max_file_count_per_mount = 0;
        cases.push((
            budget,
            BuildMountBudgetValidationError::ZeroMaxFileCountPerMount,
        ));
        budget = RUSTC_BUILD_MOUNT_BUDGET_V1;
        budget.max_file_count_per_mount += 1;
        cases.push((
            budget,
            BuildMountBudgetValidationError::MaxFileCountPerMountExceedsPolicy {
                actual: MAX_FILE_COUNT_PER_MOUNT + 1,
                maximum: MAX_FILE_COUNT_PER_MOUNT,
            },
        ));
        budget = RUSTC_BUILD_MOUNT_BUDGET_V1;
        budget.max_chunk_count_per_mount = 0;
        cases.push((
            budget,
            BuildMountBudgetValidationError::ZeroMaxChunkCountPerMount,
        ));
        budget = RUSTC_BUILD_MOUNT_BUDGET_V1;
        budget.max_chunk_count_per_mount += 1;
        cases.push((
            budget,
            BuildMountBudgetValidationError::MaxChunkCountPerMountExceedsPolicy {
                actual: MAX_CHUNK_COUNT_PER_MOUNT + 1,
                maximum: MAX_CHUNK_COUNT_PER_MOUNT,
            },
        ));
        budget = RUSTC_BUILD_MOUNT_BUDGET_V1;
        budget.max_path_len_bytes = 0;
        cases.push((budget, BuildMountBudgetValidationError::ZeroMaxPathLenBytes));
        budget = RUSTC_BUILD_MOUNT_BUDGET_V1;
        budget.max_path_len_bytes += 1;
        cases.push((
            budget,
            BuildMountBudgetValidationError::MaxPathLenBytesExceedsPolicy {
                actual: MAX_PATH_LEN_BYTES + 1,
                maximum: MAX_PATH_LEN_BYTES,
            },
        ));
        budget = RUSTC_BUILD_MOUNT_BUDGET_V1;
        budget.max_output_bytes = 0;
        cases.push((budget, BuildMountBudgetValidationError::ZeroMaxOutputBytes));
        budget = RUSTC_BUILD_MOUNT_BUDGET_V1;
        budget.max_output_bytes += 1;
        cases.push((
            budget,
            BuildMountBudgetValidationError::MaxOutputBytesExceedsPolicy {
                actual: MAX_OUTPUT_BYTES + 1,
                maximum: MAX_OUTPUT_BYTES,
            },
        ));

        let mut reasons = Vec::new();
        for (candidate, expected) in cases {
            assert_eq!(candidate.validate(), Err(expected));
            assert!(!reasons.contains(&expected.reason()));
            reasons.push(expected.reason());
        }
    }

    #[test]
    fn exact_inputs_authorize_opaque_storage_and_bind_every_stage() {
        let authority = default_authority();
        assert_eq!(
            authority.budget_sha256(),
            build_mount_budget_sha256(&RUSTC_BUILD_MOUNT_BUDGET_V1)
        );
        assert_ne!(authority.job_binding_sha256(), [0; 32]);
        assert_eq!(authority.output_lease().lease_id(), 7);
        assert_eq!(authority.output_lease().store_instance_id(), 11);
        assert_eq!(authority.output_lease().store_generation(), 13);
        assert_eq!(
            authority.output_lease().target_marker(),
            BUILD_OUTPUT_LEASE_TARGET_MARKER_V1
        );
    }

    #[test]
    fn canonical_manifest_hash_must_match_each_ticket_pin() {
        let sysroot = manifest("sysroot", 32, 1);
        let src = manifest("src", 16, 2);
        let wrong_sysroot_hash = sha256_bytes(b"unrelated manifest");
        let job = job_for(wrong_sysroot_hash, src.sha256().unwrap());
        let denial = authorize_build_storage(
            &job,
            &sysroot,
            &src,
            RUSTC_BUILD_MOUNT_BUDGET_V1,
            lease(
                7,
                11,
                13,
                MAX_OUTPUT_BYTES,
                BUILD_OUTPUT_LEASE_TARGET_MARKER_V1,
            ),
        )
        .unwrap_err();
        assert_eq!(denial.reason(), "sysroot_manifest_hash_mismatch");
        assert_eq!(
            denial,
            BuildStorageDenied::SysrootManifestHashMismatch {
                expected: wrong_sysroot_hash,
                actual: sysroot.sha256().unwrap(),
            }
        );
    }

    #[test]
    fn malformed_manifest_is_denied_before_hash_or_budget_trust() {
        let sysroot = BuildFsManifest {
            directories: vec![
                BuildFsDirectory {
                    path: "z".to_string(),
                },
                BuildFsDirectory {
                    path: "a".to_string(),
                },
            ],
            files: vec![],
        };
        let src = manifest("src", 16, 2);
        let job = job_for(sha256_bytes(b"pin"), src.sha256().unwrap());
        let denial = authorize_build_storage(
            &job,
            &sysroot,
            &src,
            RUSTC_BUILD_MOUNT_BUDGET_V1,
            lease(
                7,
                11,
                13,
                MAX_OUTPUT_BYTES,
                BUILD_OUTPUT_LEASE_TARGET_MARKER_V1,
            ),
        )
        .unwrap_err();
        assert_eq!(
            denial,
            BuildStorageDenied::SysrootManifestInvalid {
                rejection: BuildFsManifestError::NonCanonicalOrder,
            }
        );
    }

    #[test]
    fn manifest_over_budget_is_a_mount_and_measure_specific_denial() {
        let sysroot = manifest("sysroot", 32, 1);
        let src = manifest("src", 16, 2);
        let job = job_for(sysroot.sha256().unwrap(), src.sha256().unwrap());
        let mut budget = RUSTC_BUILD_MOUNT_BUDGET_V1;
        budget.max_total_mount_bytes_per_mount = 15;
        let denial = authorize_build_storage(
            &job,
            &sysroot,
            &src,
            budget,
            lease(
                7,
                11,
                13,
                MAX_OUTPUT_BYTES,
                BUILD_OUTPUT_LEASE_TARGET_MARKER_V1,
            ),
        )
        .unwrap_err();
        assert_eq!(
            denial.reason(),
            "sysroot_manifest_total_bytes_exceeds_budget"
        );
        assert_eq!(
            denial,
            BuildStorageDenied::ManifestBudgetExceeded {
                mount: BuildStorageMount::Sysroot,
                field: BuildMountMeasureField::TotalBytes,
                actual: 32,
                maximum: 15,
            }
        );
    }

    #[test]
    fn zero_and_malformed_lease_fields_are_independently_typed() {
        let cases = [
            (
                lease(
                    0,
                    11,
                    13,
                    MAX_OUTPUT_BYTES,
                    BUILD_OUTPUT_LEASE_TARGET_MARKER_V1,
                ),
                OutputLeaseValidationError::ZeroLeaseId,
            ),
            (
                lease(
                    7,
                    0,
                    13,
                    MAX_OUTPUT_BYTES,
                    BUILD_OUTPUT_LEASE_TARGET_MARKER_V1,
                ),
                OutputLeaseValidationError::ZeroStoreInstanceId,
            ),
            (
                lease(
                    7,
                    11,
                    0,
                    MAX_OUTPUT_BYTES,
                    BUILD_OUTPUT_LEASE_TARGET_MARKER_V1,
                ),
                OutputLeaseValidationError::ZeroStoreGeneration,
            ),
            (
                lease(7, 11, 13, 0, BUILD_OUTPUT_LEASE_TARGET_MARKER_V1),
                OutputLeaseValidationError::ZeroMaxBytes,
            ),
            (
                lease(7, 11, 13, MAX_OUTPUT_BYTES, [0; 16]),
                OutputLeaseValidationError::TargetMarkerMismatch,
            ),
        ];
        for (candidate, expected) in cases {
            let denial = authorize_fixture_result(RUSTC_BUILD_MOUNT_BUDGET_V1, candidate)
                .expect_err("invalid lease must not authorize");
            assert_eq!(
                denial,
                BuildStorageDenied::LeaseInvalid {
                    rejection: expected,
                }
            );
            assert_eq!(denial.reason(), expected.reason());
        }
    }

    #[test]
    fn output_lease_max_bytes_fits_both_budget_and_guest_out_quota() {
        let mut smaller_budget = RUSTC_BUILD_MOUNT_BUDGET_V1;
        smaller_budget.max_output_bytes = 1_024;
        let over_budget = lease(7, 11, 13, 1_025, BUILD_OUTPUT_LEASE_TARGET_MARKER_V1);
        assert_eq!(
            over_budget.validate(&smaller_budget, &RUSTC_BUILD_GUEST_CLASS_V1),
            Err(OutputLeaseValidationError::MaxBytesExceedsBudget {
                actual: 1_025,
                maximum: 1_024,
            })
        );

        let mut smaller_class = RUSTC_BUILD_GUEST_CLASS_V1;
        smaller_class.out_max_bytes = 1_024;
        smaller_class.max_file_bytes = 1_024;
        assert_eq!(smaller_class.validate(), Ok(()));
        let over_class = lease(7, 11, 13, 1_025, BUILD_OUTPUT_LEASE_TARGET_MARKER_V1);
        assert_eq!(
            over_class.validate(&RUSTC_BUILD_MOUNT_BUDGET_V1, &smaller_class),
            Err(OutputLeaseValidationError::MaxBytesExceedsGuestOutQuota {
                actual: 1_025,
                maximum: 1_024,
            })
        );
    }

    #[test]
    fn build_run_nonce_rejects_zero_and_consumes_on_extraction() {
        assert!(BuildRunNonce::kernel_minted(0).is_none());
        let nonce = BuildRunNonce::kernel_minted(9).unwrap();
        assert_eq!(nonce.into_value(), 9);
    }

    #[derive(Clone, Copy)]
    enum CommitMutation {
        ClaimedBinding,
        LeaseId,
        StoreInstance,
        StoreGeneration,
        LeaseMax,
        LeaseTarget,
        RegionOverflow,
        SpanOverflow,
        SpanOut,
        EmptyBundle,
        BundleOverLease,
        SpanOverLease,
        BundleLength,
        BundleDoesNotFit,
        Manifest,
        ChunkCount,
        InvalidAlignment,
        OffsetMisaligned,
        LengthMisaligned,
    }

    fn mutate(input: &mut ScopedBuildOutputCommitInput, mutation: CommitMutation) {
        match mutation {
            CommitMutation::ClaimedBinding => input.job_binding_sha256[0] ^= 1,
            CommitMutation::LeaseId => input.lease_id += 1,
            CommitMutation::StoreInstance => input.store_instance_id += 1,
            CommitMutation::StoreGeneration => input.store_generation += 1,
            CommitMutation::LeaseMax => input.lease_max_bytes -= 1,
            CommitMutation::LeaseTarget => input.lease_target_marker[0] ^= 1,
            CommitMutation::RegionOverflow => {
                input.artstor_region_offset = u64::MAX - 1;
            }
            CommitMutation::SpanOverflow => input.span_offset = u64::MAX - 511,
            CommitMutation::SpanOut => input.artstor_region_len = 32 * 1024,
            CommitMutation::EmptyBundle => input.bundle_len = 0,
            CommitMutation::BundleOverLease => input.bundle_len = input.lease_max_bytes + 1,
            CommitMutation::SpanOverLease => input.span_len = input.lease_max_bytes + 512,
            CommitMutation::BundleLength => input.bundle_len -= 1,
            CommitMutation::BundleDoesNotFit => input.span_len /= 2,
            CommitMutation::Manifest => input.output_manifest_sha256[0] ^= 1,
            CommitMutation::ChunkCount => input.chunk_count += 1,
            CommitMutation::InvalidAlignment => input.alignment = 3,
            CommitMutation::OffsetMisaligned => input.span_offset += 1,
            CommitMutation::LengthMisaligned => input.span_len += 1,
        }
    }

    #[test]
    fn pre_io_commit_truth_table_names_each_first_failed_pin() {
        let authority = default_authority();
        let egress = egress_for(&authority, b"output manifest", BUILD_FS_CHUNK_SIZE);
        let valid = valid_commit_input(&authority, &egress);
        let decision = evaluate_scoped_build_output_commit(&valid, &authority, &egress);
        let ScopedBuildOutputCommitDecision::Authorized(commit) = decision else {
            panic!("valid pre-I/O claims must authorize")
        };
        assert_eq!(commit.job_binding_sha256(), authority.job_binding_sha256());
        assert_eq!(commit.lease_id(), 7);
        assert_ne!(commit.commit_claims_sha256(), [0; 32]);
        assert_eq!(commit.bundle_len(), BUILD_FS_CHUNK_SIZE);
        assert_eq!(commit.chunk_count(), 1);

        let mut relocated = valid;
        relocated.span_offset += 512;
        let ScopedBuildOutputCommitDecision::Authorized(relocated_commit) =
            evaluate_scoped_build_output_commit(&relocated, &authority, &egress)
        else {
            panic!("a second in-bounds aligned reservation must authorize")
        };
        assert_ne!(
            commit.commit_claims_sha256(),
            relocated_commit.commit_claims_sha256()
        );

        let cases = [
            (
                CommitMutation::ClaimedBinding,
                BuildOutputCommitDenied::ClaimedJobBindingMismatch,
            ),
            (
                CommitMutation::LeaseId,
                BuildOutputCommitDenied::ClaimedLeaseIdMismatch,
            ),
            (
                CommitMutation::StoreInstance,
                BuildOutputCommitDenied::ClaimedStoreInstanceIdMismatch,
            ),
            (
                CommitMutation::StoreGeneration,
                BuildOutputCommitDenied::ClaimedStoreGenerationMismatch,
            ),
            (
                CommitMutation::LeaseMax,
                BuildOutputCommitDenied::ClaimedLeaseMaxBytesMismatch,
            ),
            (
                CommitMutation::LeaseTarget,
                BuildOutputCommitDenied::ClaimedLeaseTargetMarkerMismatch,
            ),
            (
                CommitMutation::RegionOverflow,
                BuildOutputCommitDenied::ArtstorRegionBoundsOverflow,
            ),
            (
                CommitMutation::SpanOverflow,
                BuildOutputCommitDenied::OutputSpanOverflow,
            ),
            (
                CommitMutation::SpanOut,
                BuildOutputCommitDenied::OutputSpanOutOfArtstor,
            ),
            (
                CommitMutation::EmptyBundle,
                BuildOutputCommitDenied::EmptyBundle,
            ),
            (
                CommitMutation::BundleOverLease,
                BuildOutputCommitDenied::BundleLengthExceedsLease,
            ),
            (
                CommitMutation::SpanOverLease,
                BuildOutputCommitDenied::OutputSpanLengthExceedsLease,
            ),
            (
                CommitMutation::BundleLength,
                BuildOutputCommitDenied::BundleLengthEgressMismatch,
            ),
            (
                CommitMutation::BundleDoesNotFit,
                BuildOutputCommitDenied::BundleDoesNotFitOutputSpan,
            ),
            (
                CommitMutation::Manifest,
                BuildOutputCommitDenied::BundleManifestSha256Mismatch,
            ),
            (
                CommitMutation::ChunkCount,
                BuildOutputCommitDenied::BundleChunkCountMismatch,
            ),
            (
                CommitMutation::InvalidAlignment,
                BuildOutputCommitDenied::InvalidAlignment,
            ),
            (
                CommitMutation::OffsetMisaligned,
                BuildOutputCommitDenied::OutputSpanOffsetMisaligned,
            ),
            (
                CommitMutation::LengthMisaligned,
                BuildOutputCommitDenied::OutputSpanLengthMisaligned,
            ),
        ];

        let mut reasons: Vec<&str> = Vec::new();
        for (mutation, expected) in cases {
            let mut input = valid;
            mutate(&mut input, mutation);
            let decision = evaluate_scoped_build_output_commit(&input, &authority, &egress);
            assert_eq!(
                decision,
                ScopedBuildOutputCommitDecision::Denied(expected),
                "wrong denial for {}",
                expected.reason()
            );
            assert!(!reasons.contains(&expected.reason()));
            reasons.push(expected.reason());
        }
    }

    #[test]
    fn budget_hash_drift_changes_binding_and_rejects_the_old_egress_result() {
        let first = default_authority();
        let first_egress = egress_for(&first, b"output manifest", BUILD_FS_CHUNK_SIZE);

        let mut smaller_budget = RUSTC_BUILD_MOUNT_BUDGET_V1;
        smaller_budget.max_path_len_bytes -= 1;
        let second = authorize_fixture(
            smaller_budget,
            lease(
                7,
                11,
                13,
                smaller_budget.max_output_bytes,
                BUILD_OUTPUT_LEASE_TARGET_MARKER_V1,
            ),
        );
        assert_ne!(first.budget_sha256(), second.budget_sha256());
        assert_ne!(first.job_binding_sha256(), second.job_binding_sha256());

        let input = valid_commit_input(&second, &first_egress);
        assert_eq!(
            evaluate_scoped_build_output_commit(&input, &second, &first_egress),
            ScopedBuildOutputCommitDecision::Denied(
                BuildOutputCommitDenied::EgressJobBindingMismatch
            )
        );
    }

    #[test]
    fn standalone_span_predicate_matches_blob_edge_cases() {
        const SECTOR_SIZE: u64 = 512;

        fn blob_reference(start_lba: u64, lba_count: u64, offset: u64, len: u64) -> bool {
            let Some(end_lba) = start_lba.checked_add(lba_count) else {
                return false;
            };
            let sector_offset = offset / SECTOR_SIZE;
            let sector_count = len / SECTOR_SIZE;
            let mut index = 0;
            while index < sector_count {
                let Some(lba) = start_lba
                    .checked_add(sector_offset)
                    .and_then(|base| base.checked_add(index))
                else {
                    return false;
                };
                if lba < start_lba || lba >= end_lba {
                    return false;
                }
                index += 1;
            }
            true
        }

        let cases = [
            (8_192, 8, 1_024, 512),
            (8_192, 2, 1_024, 512),
            (u64::MAX - 1, 2, 0, 512),
            (8_192, 8, u64::MAX - 511, 512),
            (8_192, 8, 0, 4_096),
        ];
        for (start_lba, lba_count, offset, len) in cases {
            let expected = blob_reference(start_lba, lba_count, offset, len);
            let actual = start_lba
                .checked_mul(SECTOR_SIZE)
                .and_then(|region_offset| {
                    lba_count.checked_mul(SECTOR_SIZE).map(|region_len| {
                        span_inside_region(
                            region_offset,
                            region_len,
                            region_offset.saturating_add(offset),
                            len,
                        )
                    })
                })
                .unwrap_or(false);
            assert_eq!(
                actual, expected,
                "edge case {start_lba}/{lba_count}/{offset}/{len}"
            );
        }
    }
}
