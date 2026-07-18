use raios_core::buildfs_manifest::{
    BuildFsChunk, BuildFsDirectory, BuildFsFile, BuildFsManifest, BUILD_FS_CHUNK_SIZE,
};
use raios_core::{sha256_bytes, sha256_hex};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::fs::{self, File, Metadata};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackError {
    message: String,
}

impl PackError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PackError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for PackError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackResult {
    manifest: BuildFsManifest,
    manifest_sha256: [u8; 32],
}

impl PackResult {
    pub fn manifest(&self) -> &BuildFsManifest {
        &self.manifest
    }

    pub fn manifest_sha256(&self) -> [u8; 32] {
        self.manifest_sha256
    }
}

/// Packs `input_directory` into a fresh BuildFS-v1 output directory.
///
/// The output directory may be absent or empty. Every failure during packing
/// removes it so callers never observe a partial package.
pub fn pack_directory(
    input_directory: &Path,
    output_directory: &Path,
    expected_manifest_sha256: Option<[u8; 32]>,
) -> Result<PackResult, PackError> {
    let input_metadata = fs::symlink_metadata(input_directory).map_err(|error| {
        PackError::new(format!(
            "cannot inspect input directory {}: {error}",
            input_directory.display()
        ))
    })?;
    reject_link_or_reparse(input_directory, &input_metadata)?;
    if !input_metadata.is_dir() {
        return Err(PackError::new(format!(
            "input {} is not a directory",
            input_directory.display()
        )));
    }

    let canonical_input = fs::canonicalize(input_directory).map_err(|error| {
        PackError::new(format!(
            "cannot resolve input directory {}: {error}",
            input_directory.display()
        ))
    })?;
    let canonical_output = resolve_path_even_if_missing(output_directory)?;
    if canonical_output == canonical_input || canonical_output.starts_with(&canonical_input) {
        return Err(PackError::new(
            "output directory must not be the input directory or lie inside it",
        ));
    }

    ensure_output_absent_or_empty(output_directory)?;
    fs::create_dir_all(output_directory).map_err(|error| {
        PackError::new(format!(
            "cannot create output directory {}: {error}",
            output_directory.display()
        ))
    })?;

    match pack_created_output(input_directory, output_directory, expected_manifest_sha256) {
        Ok(result) => Ok(result),
        Err(error) => match fs::remove_dir_all(output_directory) {
            Ok(()) => Err(error),
            Err(cleanup_error) => Err(PackError::new(format!(
                "{error}; additionally failed to remove output {}: {cleanup_error}",
                output_directory.display()
            ))),
        },
    }
}

fn pack_created_output(
    input_directory: &Path,
    output_directory: &Path,
    expected_manifest_sha256: Option<[u8; 32]>,
) -> Result<PackResult, PackError> {
    let chunks_directory = output_directory.join("chunks");
    fs::create_dir(&chunks_directory).map_err(|error| {
        PackError::new(format!(
            "cannot create chunk directory {}: {error}",
            chunks_directory.display()
        ))
    })?;

    let mut builder = TreeBuilder {
        directories: Vec::new(),
        files: Vec::new(),
        written_chunks: HashSet::new(),
        chunks_directory: &chunks_directory,
    };
    builder.walk_directory(input_directory, "")?;

    let manifest = BuildFsManifest::new(builder.directories, builder.files).map_err(|error| {
        PackError::new(format!(
            "BuildFS manifest rejected by raios-core: {error:?}"
        ))
    })?;
    let manifest_bytes = manifest.canonical_bytes().map_err(|error| {
        PackError::new(format!(
            "BuildFS encoding rejected by raios-core: {error:?}"
        ))
    })?;
    let manifest_sha256 = manifest.sha256().map_err(|error| {
        PackError::new(format!("BuildFS hashing rejected by raios-core: {error:?}"))
    })?;

    if let Some(expected) = expected_manifest_sha256 {
        if manifest_sha256 != expected {
            return Err(PackError::new(format!(
                "manifest SHA-256 mismatch: expected {}, got {}",
                digest_hex_string(&expected),
                digest_hex_string(&manifest_sha256)
            )));
        }
    }

    fs::write(output_directory.join("manifest.bin"), manifest_bytes)
        .map_err(|error| PackError::new(format!("cannot write manifest.bin: {error}")))?;
    fs::write(
        output_directory.join("manifest.sha256"),
        sha256_hex(&manifest_sha256),
    )
    .map_err(|error| PackError::new(format!("cannot write manifest.sha256: {error}")))?;

    Ok(PackResult {
        manifest,
        manifest_sha256,
    })
}

struct TreeBuilder<'a> {
    directories: Vec<BuildFsDirectory>,
    files: Vec<BuildFsFile>,
    written_chunks: HashSet<[u8; 32]>,
    chunks_directory: &'a Path,
}

impl TreeBuilder<'_> {
    fn walk_directory(
        &mut self,
        filesystem_path: &Path,
        manifest_path: &str,
    ) -> Result<(), PackError> {
        let entries = fs::read_dir(filesystem_path).map_err(|error| {
            PackError::new(format!(
                "cannot read directory {}: {error}",
                filesystem_path.display()
            ))
        })?;
        let mut entries = entries
            .map(|entry| {
                let entry = entry.map_err(|error| {
                    PackError::new(format!(
                        "cannot read an entry in {}: {error}",
                        filesystem_path.display()
                    ))
                })?;
                let name = entry.file_name().into_string().map_err(|name| {
                    PackError::new(format!(
                        "non-UTF-8 entry name in {}: {name:?}",
                        filesystem_path.display()
                    ))
                })?;
                Ok((name, entry))
            })
            .collect::<Result<Vec<_>, PackError>>()?;
        entries.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));

        for (name, entry) in entries {
            let child_manifest_path = if manifest_path.is_empty() {
                name
            } else {
                format!("{manifest_path}/{name}")
            };
            let child_path = entry.path();
            let metadata = fs::symlink_metadata(&child_path).map_err(|error| {
                PackError::new(format!("cannot inspect {}: {error}", child_path.display()))
            })?;
            reject_link_or_reparse(&child_path, &metadata)?;

            if metadata.is_dir() {
                self.directories.push(BuildFsDirectory {
                    path: child_manifest_path.clone(),
                });
                self.walk_directory(&child_path, &child_manifest_path)?;
            } else if metadata.is_file() {
                let packed_file = self.pack_file(&child_path, child_manifest_path)?;
                self.files.push(packed_file);
            } else {
                return Err(PackError::new(format!(
                    "unsupported filesystem entry type at {}",
                    child_path.display()
                )));
            }
        }
        Ok(())
    }

    fn pack_file(&mut self, path: &Path, manifest_path: String) -> Result<BuildFsFile, PackError> {
        let mut file = File::open(path)
            .map_err(|error| PackError::new(format!("cannot open {}: {error}", path.display())))?;
        let opened_metadata = file.metadata().map_err(|error| {
            PackError::new(format!(
                "cannot inspect opened file {}: {error}",
                path.display()
            ))
        })?;
        reject_link_or_reparse(path, &opened_metadata)?;
        if !opened_metadata.is_file() {
            return Err(PackError::new(format!(
                "entry changed type while packing {}",
                path.display()
            )));
        }

        let chunk_size = usize::try_from(BUILD_FS_CHUNK_SIZE)
            .map_err(|_| PackError::new("BuildFS chunk size does not fit this host"))?;
        let mut buffer = vec![0u8; chunk_size];
        let mut chunks = Vec::new();
        let mut file_hasher = Sha256::new();
        let mut file_len = 0u64;

        loop {
            let read = read_full_chunk(&mut file, &mut buffer).map_err(|error| {
                PackError::new(format!("cannot read {}: {error}", path.display()))
            })?;
            if read == 0 {
                break;
            }
            let bytes = &buffer[..read];
            file_hasher.update(bytes);
            file_len = file_len
                .checked_add(read as u64)
                .ok_or_else(|| PackError::new(format!("file is too large: {}", path.display())))?;
            let digest = sha256_bytes(bytes);
            if self.written_chunks.insert(digest) {
                fs::write(
                    self.chunks_directory.join(digest_hex_string(&digest)),
                    bytes,
                )
                .map_err(|error| {
                    PackError::new(format!(
                        "cannot write chunk for {}: {error}",
                        path.display()
                    ))
                })?;
            }
            chunks.push(BuildFsChunk {
                len: read as u64,
                sha256: digest,
            });
        }

        let digest = file_hasher.finalize();
        let mut file_sha256 = [0u8; 32];
        file_sha256.copy_from_slice(&digest);
        Ok(BuildFsFile {
            path: manifest_path,
            len: file_len,
            sha256: file_sha256,
            chunks,
        })
    }
}

fn read_full_chunk(reader: &mut File, buffer: &mut [u8]) -> io::Result<usize> {
    let mut filled = 0usize;
    while filled < buffer.len() {
        match reader.read(&mut buffer[filled..])? {
            0 => break,
            read => filled += read,
        }
    }
    Ok(filled)
}

fn ensure_output_absent_or_empty(output: &Path) -> Result<(), PackError> {
    match fs::symlink_metadata(output) {
        Ok(metadata) => {
            reject_link_or_reparse(output, &metadata)?;
            if !metadata.is_dir() {
                return Err(PackError::new(format!(
                    "output {} exists and is not a directory",
                    output.display()
                )));
            }
            let mut entries = fs::read_dir(output).map_err(|error| {
                PackError::new(format!(
                    "cannot inspect output {}: {error}",
                    output.display()
                ))
            })?;
            if entries
                .next()
                .transpose()
                .map_err(|error| {
                    PackError::new(format!(
                        "cannot inspect output {}: {error}",
                        output.display()
                    ))
                })?
                .is_some()
            {
                return Err(PackError::new(format!(
                    "output directory {} is not empty",
                    output.display()
                )));
            }
            Ok(())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(PackError::new(format!(
            "cannot inspect output {}: {error}",
            output.display()
        ))),
    }
}

fn resolve_path_even_if_missing(path: &Path) -> Result<PathBuf, PackError> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .map_err(|error| PackError::new(format!("cannot resolve current directory: {error}")))?
            .join(path)
    };
    let mut missing = Vec::new();
    let mut existing = absolute.as_path();
    while !existing.exists() {
        let name = existing.file_name().ok_or_else(|| {
            PackError::new(format!("cannot resolve output path {}", path.display()))
        })?;
        missing.push(name.to_owned());
        existing = existing.parent().ok_or_else(|| {
            PackError::new(format!("cannot resolve output path {}", path.display()))
        })?;
    }
    let mut resolved = fs::canonicalize(existing).map_err(|error| {
        PackError::new(format!(
            "cannot resolve output path {}: {error}",
            path.display()
        ))
    })?;
    for component in missing.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn reject_link_or_reparse(path: &Path, metadata: &Metadata) -> Result<(), PackError> {
    if metadata.file_type().is_symlink() || is_windows_reparse_point(metadata) {
        return Err(PackError::new(format!(
            "symbolic links and junctions are unsupported: {}",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(windows)]
fn is_windows_reparse_point(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_windows_reparse_point(_metadata: &Metadata) -> bool {
    false
}

fn digest_hex_string(digest: &[u8; 32]) -> String {
    String::from_utf8(sha256_hex(digest).to_vec()).expect("SHA-256 hex is ASCII")
}
