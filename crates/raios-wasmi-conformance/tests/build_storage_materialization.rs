use std::{collections::BTreeMap, fs, path::PathBuf};

const BUILD_FS_CHUNK_SIZE: usize = 64 * 1024;

type Sha256 = [u8; 32];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Chunk {
    sha256: Sha256,
    len: u64,
}

#[derive(Clone, Debug)]
struct File {
    path: &'static str,
    len: u64,
    chunks: Vec<Chunk>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Handle {
    frame: usize,
    sha256: Sha256,
    len: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ResolvedChunk {
    sha256: Sha256,
    len: u64,
    handle: Handle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MaterializeDenied {
    ConflictingChunkLength {
        sha256: Sha256,
        resolved_len: u64,
        manifest_len: u64,
    },
    Resolve,
    Readback,
    HashMismatch,
}

struct RamStore {
    frames: Vec<Vec<u8>>,
    resolve_calls: BTreeMap<Sha256, usize>,
}

impl RamStore {
    fn resolve(&mut self, sha256: Sha256, len: u64) -> Result<Handle, MaterializeDenied> {
        let calls = self.resolve_calls.entry(sha256).or_default();
        *calls += 1;
        if *calls != 1 {
            return Err(MaterializeDenied::Resolve);
        }
        let (frame, _) = self
            .frames
            .iter()
            .enumerate()
            .find(|(_, bytes)| digest(bytes) == sha256 && bytes.len() as u64 == len)
            .ok_or(MaterializeDenied::Resolve)?;
        Ok(Handle { frame, sha256, len })
    }

    fn read(&self, handle: Handle) -> Result<Vec<u8>, MaterializeDenied> {
        let bytes = self
            .frames
            .get(handle.frame)
            .ok_or(MaterializeDenied::Readback)?;
        if bytes.len() as u64 != handle.len || digest(bytes) != handle.sha256 {
            return Err(MaterializeDenied::Readback);
        }
        Ok(bytes.clone())
    }
}

fn materialize_without_interning(
    files: &[File],
    store: &mut RamStore,
) -> Result<(), MaterializeDenied> {
    for file in files {
        for chunk in &file.chunks {
            let handle = store.resolve(chunk.sha256, chunk.len)?;
            let bytes = store.read(handle)?;
            if digest(&bytes) != chunk.sha256 {
                return Err(MaterializeDenied::HashMismatch);
            }
        }
    }
    Ok(())
}

fn materialize_with_interning(
    files: &[File],
    store: &mut RamStore,
) -> Result<(Vec<usize>, Vec<ResolvedChunk>), MaterializeDenied> {
    let mut logical_entries = Vec::new();
    let mut resolved_chunks: Vec<ResolvedChunk> = Vec::new();
    for file in files {
        for chunk in &file.chunks {
            let resolved_index = if let Some((index, resolved)) = resolved_chunks
                .iter()
                .enumerate()
                .find(|(_, resolved)| resolved.sha256 == chunk.sha256)
            {
                if resolved.len != chunk.len {
                    return Err(MaterializeDenied::ConflictingChunkLength {
                        sha256: chunk.sha256,
                        resolved_len: resolved.len,
                        manifest_len: chunk.len,
                    });
                }
                index
            } else {
                let handle = store.resolve(chunk.sha256, chunk.len)?;
                let bytes = store.read(handle)?;
                if digest(&bytes) != chunk.sha256 {
                    return Err(MaterializeDenied::HashMismatch);
                }
                resolved_chunks.push(ResolvedChunk {
                    sha256: chunk.sha256,
                    len: chunk.len,
                    handle,
                });
                resolved_chunks.len() - 1
            };
            logical_entries.push(resolved_index);
        }
    }
    Ok((logical_entries, resolved_chunks))
}

fn real_shape_fixture() -> (Vec<File>, Vec<Vec<u8>>, Sha256) {
    let full = vec![0x41; BUILD_FS_CHUNK_SIZE];
    let shared_tail = b"short-tail".to_vec();
    let full_chunk = Chunk {
        sha256: digest(&full),
        len: full.len() as u64,
    };
    let shared_chunk = Chunk {
        sha256: digest(&shared_tail),
        len: shared_tail.len() as u64,
    };
    (
        vec![
            File {
                path: "lib/large.rlib",
                len: (BUILD_FS_CHUNK_SIZE + shared_tail.len()) as u64,
                chunks: vec![full_chunk, shared_chunk],
            },
            File {
                path: "lib/small-a.rmeta",
                len: shared_tail.len() as u64,
                chunks: vec![shared_chunk],
            },
            File {
                path: "lib/small-b.rmeta",
                len: shared_tail.len() as u64,
                chunks: vec![shared_chunk],
            },
        ],
        vec![full, shared_tail],
        shared_chunk.sha256,
    )
}

fn kernel_materialization_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../seed-kernel/src/wasm_runtime/wasi_build_storage.rs");
    fs::read_to_string(path).expect("kernel materialization source must be readable")
}

// This host model deliberately mirrors the small, allocation-only interning
// boundary in wasi_build_storage.rs. The source assertions keep it tied to the
// production call site instead of allowing a host-only model to drift green.
#[test]
fn real_manifest_shape_uses_entry_lengths_and_interns_duplicate_chunks() {
    let (files, frames, shared_sha256) = real_shape_fixture();
    assert!(files[0].chunks[1].len < BUILD_FS_CHUNK_SIZE as u64);
    assert!(files[1].len < BUILD_FS_CHUNK_SIZE as u64);
    assert_eq!(files[1].len, files[1].chunks[0].len);
    assert_ne!(files[1].path, files[2].path);
    assert_eq!(files[1].chunks[0], files[2].chunks[0]);

    let mut pre_fix_store = RamStore {
        frames: frames.clone(),
        resolve_calls: BTreeMap::new(),
    };
    assert_eq!(
        materialize_without_interning(&files, &mut pre_fix_store),
        Err(MaterializeDenied::Resolve),
        "resolving every logical duplicate as a new CAS object reproduces the failure"
    );

    let mut store = RamStore {
        frames,
        resolve_calls: BTreeMap::new(),
    };
    let (logical_entries, resolved_chunks) =
        materialize_with_interning(&files, &mut store).expect("real manifest shape materializes");
    assert_eq!(logical_entries.len(), 4);
    assert_eq!(resolved_chunks.len(), 2);
    assert_eq!(logical_entries[1], logical_entries[2]);
    assert_eq!(logical_entries[2], logical_entries[3]);
    assert_eq!(store.resolve_calls.get(&shared_sha256), Some(&1));

    let source = kernel_materialization_source();
    assert!(
        source.contains("resolve_chunk(chunk.sha256, chunk.len)"),
        "the store resolver must receive each manifest entry's own length"
    );
    assert!(
        source.contains("resolved_chunks: &mut Vec<ResolvedChunkEntry>"),
        "production materialization must retain a unique resolved-chunk table"
    );
    assert!(
        source.contains("resolved_chunk_index"),
        "logical entries must point at the interned resolved-chunk table"
    );
}

#[test]
fn duplicate_sha_with_conflicting_length_is_typed_denial() {
    let bytes = b"same digest identity".to_vec();
    let sha256 = digest(&bytes);
    let files = vec![
        File {
            path: "lib/first",
            len: bytes.len() as u64,
            chunks: vec![Chunk {
                sha256,
                len: bytes.len() as u64,
            }],
        },
        File {
            path: "lib/conflict",
            len: bytes.len() as u64 + 1,
            chunks: vec![Chunk {
                sha256,
                len: bytes.len() as u64 + 1,
            }],
        },
    ];
    let mut store = RamStore {
        frames: vec![bytes.clone()],
        resolve_calls: BTreeMap::new(),
    };
    assert_eq!(
        materialize_with_interning(&files, &mut store),
        Err(MaterializeDenied::ConflictingChunkLength {
            sha256,
            resolved_len: bytes.len() as u64,
            manifest_len: bytes.len() as u64 + 1,
        })
    );
    assert_eq!(store.resolve_calls.get(&sha256), Some(&1));

    let source = kernel_materialization_source();
    assert!(
        source.contains("ConflictingChunkLength"),
        "production materialization must preserve the typed conflict boundary"
    );
}

// A deterministic 256-bit test digest is sufficient for the RAM-store double:
// this test exercises identity/length interning, while production keeps SHA-256.
fn digest(bytes: &[u8]) -> Sha256 {
    let mut state = [0u32; 8];
    for (index, byte) in bytes.iter().copied().enumerate() {
        let lane = index % state.len();
        state[lane] = state[lane]
            .rotate_left(5)
            .wrapping_add(u32::from(byte))
            .wrapping_add(index as u32);
    }
    let mut digest = [0u8; 32];
    for (index, lane) in state.into_iter().enumerate() {
        digest[index * 4..index * 4 + 4].copy_from_slice(&lane.to_le_bytes());
    }
    digest
}
