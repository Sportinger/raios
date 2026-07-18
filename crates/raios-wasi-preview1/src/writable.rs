use alloc::{vec, vec::Vec};

use crate::output_manifest::{freeze_output, OutputManifest};
use crate::ramfs::{RamFs, RamQuotas};
use crate::{Errno, Fd, FileType};

const TMP_NAMESPACE: &[u8] = b".raios-tmp-mount";
const ROOT_NAMESPACE: &[u8] = b".raios-root-scratch";
const RESERVED: [&[u8]; 4] = [b"sysroot", b"src", b"out", b"tmp"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WritableError {
    Errno(Errno),
    ReservedRootName,
}

impl WritableError {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::Errno(_) => "wasi_errno",
            Self::ReservedRootName => "reserved_root_name",
        }
    }
}

impl From<Errno> for WritableError {
    fn from(value: Errno) -> Self {
        Self::Errno(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositeDirectoryEntry {
    pub name: Vec<u8>,
    pub node_id: u64,
    pub file_type: FileType,
}

/// Composite build root: immutable reserved mounts plus two volatile arenas.
/// `/tmp` and arbitrary root children share `scratch`; `/out` is isolated.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WritableRoot {
    scratch: RamFs,
    output: RamFs,
}

impl WritableRoot {
    pub fn new(
        scratch_quotas: RamQuotas,
        output_quotas: RamQuotas,
        fd_limit_per_arena: u32,
    ) -> Result<Self, Errno> {
        let mut scratch = RamFs::new(scratch_quotas, fd_limit_per_arena)?;
        scratch.create_namespace(TMP_NAMESPACE)?;
        scratch.create_namespace(ROOT_NAMESPACE)?;
        Ok(Self {
            scratch,
            output: RamFs::new(output_quotas, fd_limit_per_arena)?,
        })
    }

    pub fn scratch(&self) -> &RamFs {
        &self.scratch
    }

    pub fn output(&self) -> &RamFs {
        &self.output
    }

    pub fn create_file(&mut self, path: &[u8]) -> Result<(), WritableError> {
        self.reject_reserved_root(path)?;
        let route = route(path)?;
        let (arena, path) = self.mutable_route(route)?;
        arena.path_create_file(Fd::ROOT_PREOPEN, &path)?;
        Ok(())
    }

    pub fn create_directory(&mut self, path: &[u8]) -> Result<(), WritableError> {
        self.reject_reserved_root(path)?;
        let route = route(path)?;
        let (arena, path) = self.mutable_route(route)?;
        arena.path_create_directory(Fd::ROOT_PREOPEN, &path)?;
        Ok(())
    }

    pub fn write(&mut self, path: &[u8], offset: u64, bytes: &[u8]) -> Result<(), WritableError> {
        let route = route(path)?;
        let (arena, path) = self.mutable_route(route)?;
        arena.write_path(&path, offset, bytes)?;
        Ok(())
    }

    pub fn read(&self, path: &[u8]) -> Result<Vec<u8>, WritableError> {
        match route(path)? {
            Route::Scratch(path) => Ok(self.scratch.read_path(&path)?),
            Route::Output(path) => Ok(self.output.read_path(&path)?),
            Route::ReadOnly | Route::Root => Err(WritableError::Errno(Errno::Rofs)),
        }
    }

    pub fn rename(&mut self, source: &[u8], target: &[u8]) -> Result<(), WritableError> {
        self.reject_reserved_root(source)?;
        self.reject_reserved_root(target)?;
        match (route(source)?, route(target)?) {
            (Route::Scratch(source), Route::Scratch(target)) => {
                self.scratch
                    .path_rename(Fd::ROOT_PREOPEN, &source, Fd::ROOT_PREOPEN, &target)?;
                Ok(())
            }
            (Route::Output(source), Route::Output(target)) => {
                self.output
                    .path_rename(Fd::ROOT_PREOPEN, &source, Fd::ROOT_PREOPEN, &target)?;
                Ok(())
            }
            (Route::ReadOnly, _) | (_, Route::ReadOnly) => Err(WritableError::Errno(Errno::Rofs)),
            (Route::Root, _) | (_, Route::Root) => Err(WritableError::Errno(Errno::Perm)),
            _ => Err(WritableError::Errno(Errno::Xdev)),
        }
    }

    pub fn unlink_file(&mut self, path: &[u8]) -> Result<(), WritableError> {
        self.reject_reserved_root(path)?;
        let route = route(path)?;
        let (arena, path) = self.mutable_route(route)?;
        arena.path_unlink_file(Fd::ROOT_PREOPEN, &path)?;
        Ok(())
    }

    pub fn remove_directory(&mut self, path: &[u8]) -> Result<(), WritableError> {
        self.reject_reserved_root(path)?;
        let route = route(path)?;
        let (arena, path) = self.mutable_route(route)?;
        arena.path_remove_directory(Fd::ROOT_PREOPEN, &path)?;
        Ok(())
    }

    pub fn readdir(&self, path: &[u8]) -> Result<Vec<CompositeDirectoryEntry>, WritableError> {
        match route(path)? {
            Route::Root => self.root_entries(),
            Route::Scratch(path) => entries_from_snapshot(&self.scratch, &path, 1u64 << 63),
            Route::Output(path) => entries_from_snapshot(&self.output, &path, 1u64 << 62),
            Route::ReadOnly => Err(WritableError::Errno(Errno::Rofs)),
        }
    }

    pub fn freeze_output(&self) -> Result<OutputManifest, Errno> {
        freeze_output(&self.output)
    }

    fn mutable_route(&mut self, route: Route) -> Result<(&mut RamFs, Vec<u8>), WritableError> {
        match route {
            Route::Scratch(path) => Ok((&mut self.scratch, path)),
            Route::Output(path) => Ok((&mut self.output, path)),
            Route::ReadOnly => Err(WritableError::Errno(Errno::Rofs)),
            Route::Root => Err(WritableError::Errno(Errno::Perm)),
        }
    }

    fn reject_reserved_root(&self, path: &[u8]) -> Result<(), WritableError> {
        let components = absolute_components(path)?;
        if components.len() == 1 && RESERVED.contains(&components[0]) {
            Err(WritableError::ReservedRootName)
        } else {
            Ok(())
        }
    }

    fn root_entries(&self) -> Result<Vec<CompositeDirectoryEntry>, WritableError> {
        let mut entries = vec![
            CompositeDirectoryEntry {
                name: b"sysroot".to_vec(),
                node_id: 0,
                file_type: FileType::Directory,
            },
            CompositeDirectoryEntry {
                name: b"src".to_vec(),
                node_id: 1,
                file_type: FileType::Directory,
            },
            CompositeDirectoryEntry {
                name: b"out".to_vec(),
                node_id: 2,
                file_type: FileType::Directory,
            },
            CompositeDirectoryEntry {
                name: b"tmp".to_vec(),
                node_id: 3,
                file_type: FileType::Directory,
            },
        ];
        let snapshot = self.scratch.directory_snapshot_for_path(ROOT_NAMESPACE)?;
        for entry in snapshot.entries() {
            entries.push(CompositeDirectoryEntry {
                name: entry.name().to_vec(),
                node_id: 4 + entry.node().0,
                file_type: entry.file_type(),
            });
        }
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(entries)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Route {
    Root,
    Scratch(Vec<u8>),
    Output(Vec<u8>),
    ReadOnly,
}

fn route(path: &[u8]) -> Result<Route, WritableError> {
    let components = absolute_components(path)?;
    let Some(first) = components.first() else {
        return Ok(Route::Root);
    };
    match *first {
        b"sysroot" | b"src" => Ok(Route::ReadOnly),
        b"out" => Ok(Route::Output(join_components(&components[1..]))),
        b"tmp" => Ok(Route::Scratch(prefixed(TMP_NAMESPACE, &components[1..]))),
        _ => Ok(Route::Scratch(prefixed(ROOT_NAMESPACE, &components))),
    }
}

fn absolute_components(path: &[u8]) -> Result<Vec<&[u8]>, WritableError> {
    if path.first() != Some(&b'/') || path.contains(&0) {
        return Err(WritableError::Errno(Errno::Inval));
    }
    if path == b"/" {
        return Ok(Vec::new());
    }
    let mut components = Vec::new();
    for component in path[1..].split(|byte| *byte == b'/') {
        if component.is_empty() || component == b"." || component == b".." {
            return Err(WritableError::Errno(Errno::Inval));
        }
        components.push(component);
    }
    Ok(components)
}

fn prefixed(prefix: &[u8], components: &[&[u8]]) -> Vec<u8> {
    let mut path = prefix.to_vec();
    for component in components {
        path.push(b'/');
        path.extend_from_slice(component);
    }
    path
}

fn join_components(components: &[&[u8]]) -> Vec<u8> {
    let mut path = Vec::new();
    for (index, component) in components.iter().enumerate() {
        if index != 0 {
            path.push(b'/');
        }
        path.extend_from_slice(component);
    }
    path
}

fn entries_from_snapshot(
    arena: &RamFs,
    path: &[u8],
    tag: u64,
) -> Result<Vec<CompositeDirectoryEntry>, WritableError> {
    let snapshot = arena.directory_snapshot_for_path(path)?;
    Ok(snapshot
        .entries()
        .iter()
        .map(|entry| CompositeDirectoryEntry {
            name: entry.name().to_vec(),
            node_id: tag | entry.node().0,
            file_type: entry.file_type(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fs() -> WritableRoot {
        let quotas = RamQuotas::new(200_000, 20, 20, 200_000);
        WritableRoot::new(quotas, quotas, 16).unwrap()
    }

    fn lifecycle(mut root: WritableRoot) -> WritableRoot {
        for base in [b"/tmp/work".as_slice(), b"/out/work", b"/rustc-temp"] {
            root.create_directory(base).unwrap();
            let mut file = base.to_vec();
            file.extend_from_slice(b"/a");
            root.create_file(&file).unwrap();
            root.write(&file, 0, b"payload").unwrap();
            let mut renamed = base.to_vec();
            renamed.extend_from_slice(b"/b");
            root.rename(&file, &renamed).unwrap();
            assert_eq!(root.read(&renamed).unwrap(), b"payload");
            root.unlink_file(&renamed).unwrap();
            root.remove_directory(base).unwrap();
        }
        root
    }

    #[test]
    fn tmp_out_and_root_scratch_lifecycles_are_deterministic() {
        let first = lifecycle(fs());
        let second = lifecycle(fs());
        assert_eq!(first, second);
        assert_eq!(
            first.freeze_output().unwrap(),
            second.freeze_output().unwrap()
        );
        assert_eq!(first.readdir(b"/").unwrap(), second.readdir(b"/").unwrap());
    }

    #[test]
    fn reserved_mount_names_cannot_be_created_renamed_or_shadowed() {
        for reserved in [b"/sysroot".as_slice(), b"/src", b"/out", b"/tmp"] {
            let mut root = fs();
            assert_eq!(
                root.create_file(reserved),
                Err(WritableError::ReservedRootName)
            );
            assert_eq!(
                root.create_directory(reserved),
                Err(WritableError::ReservedRootName)
            );
            root.create_file(b"/scratch").unwrap();
            assert_eq!(
                root.rename(b"/scratch", reserved),
                Err(WritableError::ReservedRootName)
            );
            assert_eq!(
                root.rename(reserved, b"/shadow"),
                Err(WritableError::ReservedRootName)
            );
            let names: Vec<Vec<u8>> = root
                .readdir(b"/")
                .unwrap()
                .into_iter()
                .map(|e| e.name)
                .collect();
            assert_eq!(
                names
                    .iter()
                    .filter(|name| name.as_slice() == &reserved[1..])
                    .count(),
                1
            );
        }
    }

    #[test]
    fn rename_crossing_output_arena_fails_xdev_without_mutation() {
        let mut root = fs();
        root.create_file(b"/tmp/a").unwrap();
        root.write(b"/tmp/a", 0, b"kept").unwrap();
        let before = root.clone();
        assert_eq!(
            root.rename(b"/tmp/a", b"/out/a"),
            Err(WritableError::Errno(Errno::Xdev))
        );
        assert_eq!(root, before);
    }

    #[test]
    fn root_listing_merges_mounts_and_scratch_in_byte_order() {
        let mut root = fs();
        root.create_directory(b"/zeta").unwrap();
        root.create_file(b"/alpha").unwrap();
        let names: Vec<Vec<u8>> = root
            .readdir(b"/")
            .unwrap()
            .into_iter()
            .map(|e| e.name)
            .collect();
        assert_eq!(
            names,
            [
                b"alpha".as_slice(),
                b"out".as_slice(),
                b"src".as_slice(),
                b"sysroot".as_slice(),
                b"tmp".as_slice(),
                b"zeta".as_slice(),
            ]
            .iter()
            .map(|name| name.to_vec())
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn freeze_can_only_observe_output_arena() {
        let mut root = fs();
        root.create_file(b"/tmp/secret").unwrap();
        root.write(b"/tmp/secret", 0, b"tmp").unwrap();
        root.create_file(b"/root-secret").unwrap();
        root.write(b"/root-secret", 0, b"root").unwrap();
        root.create_file(b"/out/public").unwrap();
        root.write(b"/out/public", 0, b"out").unwrap();
        let manifest = root.freeze_output().unwrap();
        assert_eq!(manifest.files.len(), 1);
        assert_eq!(manifest.files[0].path, "public");
        assert!(!manifest
            .canonical_bytes()
            .windows(6)
            .any(|w| w == b"secret"));
    }
}
