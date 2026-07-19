use alloc::{boxed::Box, vec, vec::Vec};

use raios_core::{
    build_storage_authority::{BuildRunNonce, BuildStorageAuthority},
    buildfs_manifest::{BuildFsManifest, BuildFsManifestError, BUILD_FS_CHUNK_SIZE},
    sha256_bytes,
};
use raios_wasi_preview1::{
    BuildFs, BuildFsManifestView, ChunkRead, ChunkReadError, ChunkReadRequest, Errno, MountId,
    NodeRef, NormalizedPath, SOURCE_MOUNT, SYSROOT_MOUNT,
};

/// Store-private physical handle. It never crosses into a preview1 request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BuildChunkHandle {
    pub(crate) store_generation: u64,
    pub(crate) frame_offset: u64,
    pub(crate) frame_len: u64,
    pub(crate) payload_len: u64,
    pub(crate) payload_sha256: [u8; 32],
    pub(crate) frame_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BuildChunkStoreError {
    Missing,
    Bounds,
    MalformedFrame,
    Io,
}

/// Kernel-side resolver/readback seam. This is deliberately not `ChunkRead`:
/// only the table-constrained adapter below is visible to the WASI host state.
pub(crate) trait BuildChunkStore {
    fn store_instance_id(&self) -> u64;
    fn store_generation(&self) -> u64;

    fn resolve_chunk(
        &mut self,
        sha256: [u8; 32],
        len: u64,
    ) -> Result<BuildChunkHandle, BuildChunkStoreError>;

    fn read_resolved_chunk(
        &mut self,
        handle: BuildChunkHandle,
        destination: &mut [u8],
    ) -> Result<(), BuildChunkStoreError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BuildStorageMaterializationError {
    SysrootManifestInvalid(BuildFsManifestError),
    SrcManifestInvalid(BuildFsManifestError),
    SysrootManifestHashMismatch,
    SrcManifestHashMismatch,
    StoreInstanceMismatch,
    StoreGenerationMismatch,
    Projection(Errno),
    ChunkLengthOverflow,
    ChunkResolve {
        mount_id: MountId,
        file: NodeRef,
        chunk_index: u64,
        error: BuildChunkStoreError,
    },
    ConflictingChunkLength {
        mount_id: MountId,
        file: NodeRef,
        chunk_index: u64,
        chunk_sha256: [u8; 32],
        resolved_len: u64,
        manifest_len: u64,
    },
    ResolvedHandleMismatch {
        mount_id: MountId,
        file: NodeRef,
        chunk_index: u64,
    },
    ChunkReadback {
        mount_id: MountId,
        file: NodeRef,
        chunk_index: u64,
        error: BuildChunkStoreError,
    },
    ChunkHashMismatch {
        mount_id: MountId,
        file: NodeRef,
        chunk_index: u64,
    },
}

impl BuildStorageMaterializationError {
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::SysrootManifestInvalid(_) => "sysroot_manifest_invalid",
            Self::SrcManifestInvalid(_) => "src_manifest_invalid",
            Self::SysrootManifestHashMismatch => "sysroot_manifest_hash_mismatch",
            Self::SrcManifestHashMismatch => "src_manifest_hash_mismatch",
            Self::StoreInstanceMismatch => "store_instance_mismatch",
            Self::StoreGenerationMismatch => "store_generation_mismatch",
            Self::Projection(_) => "manifest_projection_failed",
            Self::ChunkLengthOverflow => "chunk_length_overflow",
            Self::ChunkResolve { .. } => "chunk_resolution_failed",
            Self::ConflictingChunkLength { .. } => "conflicting_chunk_length",
            Self::ResolvedHandleMismatch { .. } => "resolved_chunk_handle_mismatch",
            Self::ChunkReadback { .. } => "chunk_readback_failed",
            Self::ChunkHashMismatch { .. } => "chunk_hash_mismatch",
        }
    }

    /// Diagnostic-only: the inner chunk-store error variant for the resolve/
    /// readback cases, so a serial fail token can distinguish a scan miss
    /// from a malformed/io frame without leaking store topology. "none" for
    /// errors that carry no inner chunk-store error.
    pub(crate) const fn chunk_store_detail(self) -> &'static str {
        match self {
            Self::ChunkResolve { error, .. } | Self::ChunkReadback { error, .. } => match error {
                BuildChunkStoreError::Missing => "missing",
                BuildChunkStoreError::Bounds => "bounds",
                BuildChunkStoreError::MalformedFrame => "malformed",
                BuildChunkStoreError::Io => "io",
            },
            _ => "none",
        }
    }

    /// Diagnostic-only: the manifest chunk index the failure is anchored to,
    /// or `u64::MAX` when the error is not chunk-anchored.
    pub(crate) const fn failing_chunk_index(self) -> u64 {
        match self {
            Self::ChunkResolve { chunk_index, .. }
            | Self::ConflictingChunkLength { chunk_index, .. }
            | Self::ResolvedHandleMismatch { chunk_index, .. }
            | Self::ChunkReadback { chunk_index, .. }
            | Self::ChunkHashMismatch { chunk_index, .. } => chunk_index,
            _ => u64::MAX,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GrantedChunkReadDenied {
    AbsentEntry,
    WrongRange,
    StoreRead,
    HashMismatch,
}

impl GrantedChunkReadDenied {
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::AbsentEntry => "absent_entry",
            Self::WrongRange => "wrong_range",
            Self::StoreRead => "store_read_failed",
            Self::HashMismatch => "hash_mismatch",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MaterializedChunkEntry {
    mount_id: MountId,
    file: NodeRef,
    file_sha256: [u8; 32],
    chunk_index: u64,
    chunk_sha256: [u8; 32],
    chunk_len: u64,
    resolved_chunk_index: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ResolvedChunkEntry {
    chunk_sha256: [u8; 32],
    chunk_len: u64,
    handle: BuildChunkHandle,
}

/// The sole `ChunkRead` implementation injected into a WASI build instance.
/// It owns a per-run table and has no ambient digest lookup fallback.
pub(crate) struct GrantedChunkReader {
    job_binding_sha256: [u8; 32],
    run_nonce: u64,
    store_generation: u64,
    entries: Vec<MaterializedChunkEntry>,
    resolved_chunks: Vec<ResolvedChunkEntry>,
    store: Box<dyn BuildChunkStore>,
    last_denial: Option<GrantedChunkReadDenied>,
}

impl GrantedChunkReader {
    pub(crate) const fn job_binding_sha256(&self) -> [u8; 32] {
        self.job_binding_sha256
    }

    pub(crate) const fn run_nonce(&self) -> u64 {
        self.run_nonce
    }

    pub(crate) const fn store_generation(&self) -> u64 {
        self.store_generation
    }

    pub(crate) fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub(crate) const fn last_denial(&self) -> Option<GrantedChunkReadDenied> {
        self.last_denial
    }

    fn deny(&mut self, denial: GrantedChunkReadDenied) -> Result<(), ChunkReadError> {
        self.last_denial = Some(denial);
        Err(match denial {
            GrantedChunkReadDenied::AbsentEntry | GrantedChunkReadDenied::WrongRange => {
                ChunkReadError::NotCapable
            }
            GrantedChunkReadDenied::StoreRead | GrantedChunkReadDenied::HashMismatch => {
                ChunkReadError::Io
            }
        })
    }
}

impl ChunkRead for GrantedChunkReader {
    fn read_chunk(
        &mut self,
        request: ChunkReadRequest,
        destination: &mut [u8],
    ) -> Result<(), ChunkReadError> {
        self.last_denial = None;
        let Some(entry) = self.entries.iter().copied().find(|entry| {
            entry.mount_id == request.mount_id
                && entry.file == request.file
                && entry.file_sha256 == request.file_sha256
                && entry.chunk_index == request.chunk_index
                && entry.chunk_sha256 == request.chunk_sha256
        }) else {
            return self.deny(GrantedChunkReadDenied::AbsentEntry);
        };
        let Some(resolved) = self
            .resolved_chunks
            .get(entry.resolved_chunk_index)
            .copied()
        else {
            return self.deny(GrantedChunkReadDenied::StoreRead);
        };
        let destination_len = match u64::try_from(destination.len()) {
            Ok(len) => len,
            Err(_) => return self.deny(GrantedChunkReadDenied::WrongRange),
        };
        if request.range_offset != 0
            || request.range_len != entry.chunk_len
            || destination_len != entry.chunk_len
        {
            return self.deny(GrantedChunkReadDenied::WrongRange);
        }
        if resolved.chunk_sha256 != entry.chunk_sha256
            || resolved.chunk_len != entry.chunk_len
            || resolved.handle.store_generation != self.store_generation
            || resolved.handle.payload_len != entry.chunk_len
            || resolved.handle.payload_sha256 != entry.chunk_sha256
        {
            return self.deny(GrantedChunkReadDenied::StoreRead);
        }

        // Never let a failed read or verification partially alter guest-bound
        // output. The store fills a private scratch buffer first.
        let mut scratch = vec![0; destination.len()];
        if self
            .store
            .read_resolved_chunk(resolved.handle, &mut scratch)
            .is_err()
        {
            return self.deny(GrantedChunkReadDenied::StoreRead);
        }
        if sha256_bytes(&scratch) != entry.chunk_sha256 {
            return self.deny(GrantedChunkReadDenied::HashMismatch);
        }
        destination.copy_from_slice(&scratch);
        Ok(())
    }
}

/// Consumes the nonce and resolves/verifies every chunk before the guest can
/// be instantiated. Reusing the same nonce value requires minting a new core
/// value; this owned instance itself cannot be materialized twice.
pub(crate) fn materialize_build_storage(
    authority: &BuildStorageAuthority,
    sysroot_manifest: &BuildFsManifest,
    src_manifest: &BuildFsManifest,
    nonce: BuildRunNonce,
    mut store: Box<dyn BuildChunkStore>,
) -> Result<GrantedChunkReader, BuildStorageMaterializationError> {
    let run_nonce = nonce.into_value();
    sysroot_manifest
        .validate()
        .map_err(BuildStorageMaterializationError::SysrootManifestInvalid)?;
    src_manifest
        .validate()
        .map_err(BuildStorageMaterializationError::SrcManifestInvalid)?;
    if sysroot_manifest
        .sha256()
        .map_err(BuildStorageMaterializationError::SysrootManifestInvalid)?
        != authority.sysroot_mount_manifest_sha256()
    {
        return Err(BuildStorageMaterializationError::SysrootManifestHashMismatch);
    }
    if src_manifest
        .sha256()
        .map_err(BuildStorageMaterializationError::SrcManifestInvalid)?
        != authority.src_mount_manifest_sha256()
    {
        return Err(BuildStorageMaterializationError::SrcManifestHashMismatch);
    }
    let lease = authority.output_lease();
    if store.store_instance_id() != lease.store_instance_id() {
        return Err(BuildStorageMaterializationError::StoreInstanceMismatch);
    }
    if store.store_generation() != lease.store_generation() {
        return Err(BuildStorageMaterializationError::StoreGenerationMismatch);
    }

    let mut entries = Vec::new();
    let mut resolved_chunks = Vec::new();
    materialize_manifest(
        sysroot_manifest,
        SYSROOT_MOUNT,
        store.store_generation(),
        store.as_mut(),
        &mut entries,
        &mut resolved_chunks,
    )?;
    materialize_manifest(
        src_manifest,
        SOURCE_MOUNT,
        store.store_generation(),
        store.as_mut(),
        &mut entries,
        &mut resolved_chunks,
    )?;
    Ok(GrantedChunkReader {
        job_binding_sha256: authority.job_binding_sha256(),
        run_nonce,
        store_generation: store.store_generation(),
        entries,
        resolved_chunks,
        store,
        last_denial: None,
    })
}

fn materialize_manifest(
    manifest: &BuildFsManifest,
    mount_id: MountId,
    store_generation: u64,
    store: &mut dyn BuildChunkStore,
    entries: &mut Vec<MaterializedChunkEntry>,
    resolved_chunks: &mut Vec<ResolvedChunkEntry>,
) -> Result<(), BuildStorageMaterializationError> {
    let buildfs = project_core_manifest(manifest)?;
    for file in &manifest.files {
        let path = NormalizedPath::root()
            .resolve(file.path.as_bytes())
            .map_err(BuildStorageMaterializationError::Projection)?;
        let file_node = buildfs
            .node_for_path(&path)
            .map_err(BuildStorageMaterializationError::Projection)?;
        for (chunk_index, chunk) in file.chunks.iter().enumerate() {
            let chunk_index = u64::try_from(chunk_index)
                .map_err(|_| BuildStorageMaterializationError::ChunkLengthOverflow)?;
            let resolved_chunk_index = if let Some((index, resolved)) = resolved_chunks
                .iter()
                .copied()
                .enumerate()
                .find(|(_, resolved)| resolved.chunk_sha256 == chunk.sha256)
            {
                if resolved.chunk_len != chunk.len {
                    return Err(BuildStorageMaterializationError::ConflictingChunkLength {
                        mount_id,
                        file: file_node,
                        chunk_index,
                        chunk_sha256: chunk.sha256,
                        resolved_len: resolved.chunk_len,
                        manifest_len: chunk.len,
                    });
                }
                index
            } else {
                let handle = store
                    .resolve_chunk(chunk.sha256, chunk.len)
                    .map_err(|error| BuildStorageMaterializationError::ChunkResolve {
                        mount_id,
                        file: file_node,
                        chunk_index,
                        error,
                    })?;
                if handle.store_generation != store_generation
                    || handle.payload_len != chunk.len
                    || handle.payload_sha256 != chunk.sha256
                {
                    return Err(BuildStorageMaterializationError::ResolvedHandleMismatch {
                        mount_id,
                        file: file_node,
                        chunk_index,
                    });
                }
                let chunk_len = usize::try_from(chunk.len)
                    .map_err(|_| BuildStorageMaterializationError::ChunkLengthOverflow)?;
                let mut bytes = vec![0; chunk_len];
                store
                    .read_resolved_chunk(handle, &mut bytes)
                    .map_err(|error| BuildStorageMaterializationError::ChunkReadback {
                        mount_id,
                        file: file_node,
                        chunk_index,
                        error,
                    })?;
                if sha256_bytes(&bytes) != chunk.sha256 {
                    return Err(BuildStorageMaterializationError::ChunkHashMismatch {
                        mount_id,
                        file: file_node,
                        chunk_index,
                    });
                }
                resolved_chunks.push(ResolvedChunkEntry {
                    chunk_sha256: chunk.sha256,
                    chunk_len: chunk.len,
                    handle,
                });
                resolved_chunks.len() - 1
            };
            entries.push(MaterializedChunkEntry {
                mount_id,
                file: file_node,
                file_sha256: file.sha256,
                chunk_index,
                chunk_sha256: chunk.sha256,
                chunk_len: chunk.len,
                resolved_chunk_index,
            });
        }
    }
    Ok(())
}

pub(crate) fn project_core_manifest(
    manifest: &BuildFsManifest,
) -> Result<BuildFs, BuildStorageMaterializationError> {
    BuildFs::project(&CoreManifestView(manifest))
        .map_err(BuildStorageMaterializationError::Projection)
}

struct CoreManifestView<'a>(&'a BuildFsManifest);

impl BuildFsManifestView for CoreManifestView<'_> {
    fn chunk_size(&self) -> u64 {
        BUILD_FS_CHUNK_SIZE
    }

    fn directory_count(&self) -> usize {
        self.0.directories.len()
    }

    fn directory_path(&self, index: usize) -> Option<&str> {
        self.0
            .directories
            .get(index)
            .map(|entry| entry.path.as_str())
    }

    fn file_count(&self) -> usize {
        self.0.files.len()
    }

    fn file_path(&self, file: usize) -> Option<&str> {
        self.0.files.get(file).map(|entry| entry.path.as_str())
    }

    fn file_len(&self, file: usize) -> Option<u64> {
        self.0.files.get(file).map(|entry| entry.len)
    }

    fn file_sha256(&self, file: usize) -> Option<[u8; 32]> {
        self.0.files.get(file).map(|entry| entry.sha256)
    }

    fn chunk_count(&self, file: usize) -> Option<usize> {
        self.0.files.get(file).map(|entry| entry.chunks.len())
    }

    fn chunk_len(&self, file: usize, chunk: usize) -> Option<u64> {
        self.0
            .files
            .get(file)?
            .chunks
            .get(chunk)
            .map(|entry| entry.len)
    }

    fn chunk_sha256(&self, file: usize, chunk: usize) -> Option<[u8; 32]> {
        self.0
            .files
            .get(file)?
            .chunks
            .get(chunk)
            .map(|entry| entry.sha256)
    }
}

/// Empty-manifest runs still receive the same granted reader shape. Any
/// unexpected read is denied by the empty table before these methods run.
pub(crate) struct UnbackedChunkStore {
    store_instance_id: u64,
    store_generation: u64,
}

impl UnbackedChunkStore {
    pub(crate) const fn new(store_instance_id: u64, store_generation: u64) -> Self {
        Self {
            store_instance_id,
            store_generation,
        }
    }
}

impl BuildChunkStore for UnbackedChunkStore {
    fn store_instance_id(&self) -> u64 {
        self.store_instance_id
    }

    fn store_generation(&self) -> u64 {
        self.store_generation
    }

    fn resolve_chunk(
        &mut self,
        _sha256: [u8; 32],
        _len: u64,
    ) -> Result<BuildChunkHandle, BuildChunkStoreError> {
        Err(BuildChunkStoreError::Missing)
    }

    fn read_resolved_chunk(
        &mut self,
        _handle: BuildChunkHandle,
        _destination: &mut [u8],
    ) -> Result<(), BuildChunkStoreError> {
        Err(BuildChunkStoreError::Io)
    }
}

impl BuildChunkStore for crate::agent_protocol::artifact_store::BuildChunkReadSession {
    fn store_instance_id(&self) -> u64 {
        crate::agent_protocol::artifact_store::BuildChunkReadSession::store_instance_id(self)
    }

    fn store_generation(&self) -> u64 {
        crate::agent_protocol::artifact_store::BuildChunkReadSession::store_generation(self)
    }

    fn resolve_chunk(
        &mut self,
        sha256: [u8; 32],
        len: u64,
    ) -> Result<BuildChunkHandle, BuildChunkStoreError> {
        let frame = self
            .resolve_chunk_frame(sha256, len)
            .map_err(map_artstor_error)?;
        Ok(BuildChunkHandle {
            store_generation: frame.store_generation,
            frame_offset: frame.frame_offset,
            frame_len: frame.frame_len,
            payload_len: frame.payload_len,
            payload_sha256: frame.payload_sha256,
            frame_sha256: frame.frame_sha256,
        })
    }

    fn read_resolved_chunk(
        &mut self,
        handle: BuildChunkHandle,
        destination: &mut [u8],
    ) -> Result<(), BuildChunkStoreError> {
        self.read_chunk_frame(
            crate::agent_protocol::artifact_store::BuildFsChunkFrame {
                store_generation: handle.store_generation,
                frame_offset: handle.frame_offset,
                frame_len: handle.frame_len,
                payload_len: handle.payload_len,
                payload_sha256: handle.payload_sha256,
                frame_sha256: handle.frame_sha256,
            },
            destination,
        )
        .map_err(map_artstor_error)
    }
}

fn map_artstor_error(
    error: crate::agent_protocol::artifact_store::BuildFsChunkFrameError,
) -> BuildChunkStoreError {
    match error {
        crate::agent_protocol::artifact_store::BuildFsChunkFrameError::Missing => {
            BuildChunkStoreError::Missing
        }
        crate::agent_protocol::artifact_store::BuildFsChunkFrameError::Bounds
        | crate::agent_protocol::artifact_store::BuildFsChunkFrameError::LengthMismatch => {
            BuildChunkStoreError::Bounds
        }
        crate::agent_protocol::artifact_store::BuildFsChunkFrameError::Malformed => {
            BuildChunkStoreError::MalformedFrame
        }
        crate::agent_protocol::artifact_store::BuildFsChunkFrameError::Io => {
            BuildChunkStoreError::Io
        }
    }
}
