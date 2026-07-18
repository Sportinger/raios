use alloc::{vec, vec::Vec};
use core::cmp::Ordering;

use crate::{Errno, FileType, NodeRef};

/// A lexical path anchored at, and unable to escape, one capability root.
///
/// Components are opaque bytes. Resolution never consults or follows symlinks.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NormalizedPath {
    components: Vec<Vec<u8>>,
}

impl NormalizedPath {
    pub const fn root() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    /// Resolves a preview1 relative path against this capability-relative base.
    pub fn resolve(&self, path: &[u8]) -> Result<Self, Errno> {
        if path.first() == Some(&b'/') {
            return Err(Errno::Notcapable);
        }
        let mut components = self.components.clone();
        for component in path.split(|byte| *byte == b'/') {
            match component {
                b"" | b"." => {}
                b".." => {
                    components.pop().ok_or(Errno::Notcapable)?;
                }
                value if value.contains(&0) => return Err(Errno::Inval),
                value => components.push(value.to_vec()),
            }
        }
        Ok(Self { components })
    }

    pub fn components(&self) -> impl ExactSizeIterator<Item = &[u8]> {
        self.components.iter().map(Vec::as_slice)
    }

    /// Returns the canonical guest-visible absolute spelling.
    pub fn to_absolute_bytes(&self) -> Vec<u8> {
        if self.components.is_empty() {
            return vec![b'/'];
        }
        let mut output = Vec::new();
        output.push(b'/');
        for (index, component) in self.components.iter().enumerate() {
            if index != 0 {
                output.push(b'/');
            }
            output.extend_from_slice(component);
        }
        output
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectoryEntry {
    name: Vec<u8>,
    node: NodeRef,
    file_type: FileType,
}

impl DirectoryEntry {
    pub fn new(name: &[u8], node: NodeRef, file_type: FileType) -> Result<Self, Errno> {
        if name.is_empty()
            || name == b"."
            || name == b".."
            || name.contains(&b'/')
            || name.contains(&0)
        {
            return Err(Errno::Inval);
        }
        Ok(Self {
            name: name.to_vec(),
            node,
            file_type,
        })
    }

    pub fn name(&self) -> &[u8] {
        &self.name
    }

    pub const fn node(&self) -> NodeRef {
        self.node
    }

    pub const fn file_type(&self) -> FileType {
        self.file_type
    }
}

/// A canonical, immutable directory listing. Cookies are indices into `entries`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DirectorySnapshot {
    entries: Vec<DirectoryEntry>,
}

impl DirectorySnapshot {
    pub fn new(mut entries: Vec<DirectoryEntry>) -> Result<Self, Errno> {
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        if entries
            .windows(2)
            .any(|pair| pair[0].name.cmp(&pair[1].name) == Ordering::Equal)
        {
            return Err(Errno::Exist);
        }
        Ok(Self { entries })
    }

    pub fn entries(&self) -> &[DirectoryEntry] {
        &self.entries
    }

    /// Returns the listing suffix beginning at `cookie`; `len` is a valid EOF cookie.
    pub fn from_cookie(&self, cookie: u64) -> Result<&[DirectoryEntry], Errno> {
        let index = usize::try_from(cookie).map_err(|_| Errno::Inval)?;
        self.entries.get(index..).ok_or(Errno::Inval)
    }

    pub const fn next_cookie(cookie: u64) -> Result<u64, Errno> {
        match cookie.checked_add(1) {
            Some(next) => Ok(next),
            None => Err(Errno::Inval),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

    use super::{DirectoryEntry, DirectorySnapshot, NormalizedPath};
    use crate::{Errno, FileType, NodeRef};

    fn absolute(path: &NormalizedPath) -> Vec<u8> {
        path.to_absolute_bytes()
    }

    #[test]
    fn path_normalization_is_lexical_and_base_relative() {
        let root = NormalizedPath::root();
        assert_eq!(root.resolve(b"a//b/./c"), root.resolve(b"a/b/c"));
        assert_eq!(root.resolve(b"a/b/../c"), root.resolve(b"a/c"));

        let base = root.resolve(b"base/dir").unwrap();
        assert_eq!(absolute(&base.resolve(b"../file").unwrap()), b"/base/file");
    }

    #[test]
    fn capability_escape_and_absolute_preview1_paths_fail_closed() {
        let root = NormalizedPath::root();
        let base = root.resolve(b"base").unwrap();
        assert_eq!(root.resolve(b"../escape"), Err(Errno::Notcapable));
        assert_eq!(root.resolve(b"a/../../b"), Err(Errno::Notcapable));
        assert_eq!(root.resolve(b"/absolute"), Err(Errno::Notcapable));
        assert_eq!(base.resolve(b"/absolute"), Err(Errno::Notcapable));
        assert_eq!(absolute(&root.resolve(b"relative").unwrap()), b"/relative");
        assert_eq!(
            absolute(&base.resolve(b"relative").unwrap()),
            b"/base/relative"
        );
    }

    #[test]
    fn directory_snapshot_sorts_bytes_and_uses_stable_index_cookies() {
        let entry = |name: &[u8], node| {
            DirectoryEntry::new(name, NodeRef(node), FileType::RegularFile).unwrap()
        };
        let first = DirectorySnapshot::new(vec![
            entry(b"zeta", 3),
            entry(b"alpha", 1),
            entry(b"beta", 2),
        ])
        .unwrap();
        let second = DirectorySnapshot::new(vec![
            entry(b"beta", 2),
            entry(b"zeta", 3),
            entry(b"alpha", 1),
        ])
        .unwrap();
        assert_eq!(first, second);
        assert_eq!(first.from_cookie(0).unwrap()[0].name, b"alpha");
        assert_eq!(first.from_cookie(1).unwrap()[0].name, b"beta");
        assert!(first.from_cookie(3).unwrap().is_empty());
    }

    #[test]
    fn cookie_overflow_is_typed_and_never_panics() {
        assert_eq!(DirectorySnapshot::next_cookie(u64::MAX), Err(Errno::Inval));
        let empty = DirectorySnapshot::default();
        assert_eq!(empty.from_cookie(u64::MAX), Err(Errno::Inval));
    }
}
