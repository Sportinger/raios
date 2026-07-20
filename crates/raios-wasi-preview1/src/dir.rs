use alloc::vec::Vec;

use crate::{DirectorySnapshot, Errno, FileType, NodeRef};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dirent {
    pub next_cookie: u64,
    pub inode: NodeRef,
    pub file_type: FileType,
    pub name: Vec<u8>,
}

pub(crate) fn read_directory(
    snapshot: &DirectorySnapshot,
    cookie: u64,
    max_entries: usize,
) -> Result<Vec<Dirent>, Errno> {
    let entries = snapshot.from_cookie(cookie)?;
    let mut output = Vec::new();
    for (relative, entry) in entries.iter().take(max_entries).enumerate() {
        let current = cookie
            .checked_add(u64::try_from(relative).map_err(|_| Errno::Inval)?)
            .ok_or(Errno::Inval)?;
        output.push(Dirent {
            next_cookie: DirectorySnapshot::next_cookie(current)?,
            inode: entry.node(),
            file_type: entry.file_type(),
            name: entry.name().to_vec(),
        });
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::DirectoryEntry;

    #[test]
    fn cookies_resume_the_same_immutable_byte_sorted_snapshot() {
        let snapshot = DirectorySnapshot::new(vec![
            DirectoryEntry::new(b"z", NodeRef(3), FileType::RegularFile).unwrap(),
            DirectoryEntry::new(b"a", NodeRef(1), FileType::Directory).unwrap(),
            DirectoryEntry::new(b"m", NodeRef(2), FileType::RegularFile).unwrap(),
        ])
        .unwrap();
        let first = read_directory(&snapshot, 0, 2).unwrap();
        assert_eq!(
            first
                .iter()
                .map(|entry| entry.name.as_slice())
                .collect::<Vec<_>>(),
            vec![&b"a"[..], &b"m"[..]]
        );
        let rest = read_directory(&snapshot, first[1].next_cookie, 2).unwrap();
        assert_eq!(rest[0].name, b"z");
        assert_eq!(rest[0].next_cookie, 3);
    }
}
