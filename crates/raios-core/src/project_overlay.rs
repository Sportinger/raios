//! In-memory project edits bound to one exact immutable workspace revision.

extern crate alloc;

use alloc::{string::String, vec::Vec};

use crate::{
    project_workspace::{
        build_agent_answer, build_agent_edit, build_local_import, BuiltRevision, Classification,
        ProjectId, ProjectRevision, RevisionAction, SourceFile, TreeEntry, WorkspaceError,
    },
    sha256_bytes,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiffKind {
    Add,
    Replace,
    Delete,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffEntry {
    pub kind: DiffKind,
    pub path: String,
    pub old: Option<TreeEntry>,
    pub new: Option<TreeEntry>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlayError {
    WrongProject,
    WrongBaseRevision,
    BaseFilesMismatch,
    DuplicateOperation,
    NoOp,
    InvalidDelete,
    Workspace(WorkspaceError),
}

impl From<WorkspaceError> for OverlayError {
    fn from(value: WorkspaceError) -> Self {
        Self::Workspace(value)
    }
}

#[derive(Clone)]
struct OwnedFile {
    path: String,
    classification: Classification,
    media_type: String,
    bytes: Vec<u8>,
}

impl OwnedFile {
    fn from_source(file: SourceFile<'_>) -> Self {
        Self {
            path: String::from(file.path),
            classification: file.classification,
            media_type: String::from(file.media_type),
            bytes: file.bytes.to_vec(),
        }
    }

    fn source(&self) -> SourceFile<'_> {
        SourceFile {
            path: &self.path,
            classification: self.classification,
            media_type: &self.media_type,
            bytes: &self.bytes,
        }
    }

    fn entry(&self) -> TreeEntry {
        TreeEntry {
            path: self.path.clone(),
            classification: self.classification,
            media_type: self.media_type.clone(),
            byte_len: self.bytes.len(),
            blob_sha256: sha256_bytes(&self.bytes),
        }
    }

    fn same_source(&self, file: SourceFile<'_>) -> bool {
        self.path == file.path
            && self.classification == file.classification
            && self.media_type == file.media_type
            && self.bytes == file.bytes
    }
}

pub struct ProjectOverlay {
    base: ProjectRevision,
    base_files: Vec<OwnedFile>,
    files: Vec<OwnedFile>,
    touched_paths: Vec<String>,
}

impl ProjectOverlay {
    pub fn new(
        project_id: ProjectId,
        base_revision_sha256: [u8; 32],
        base: &ProjectRevision,
        verified_files: &[SourceFile<'_>],
    ) -> Result<Self, OverlayError> {
        check_binding(project_id, base_revision_sha256, base)?;
        let verified = rebuild_base(base, verified_files)?;
        if verified.revision != *base {
            return Err(OverlayError::BaseFilesMismatch);
        }
        let base_files: Vec<_> = verified_files
            .iter()
            .copied()
            .map(OwnedFile::from_source)
            .collect();
        Ok(Self {
            base: base.clone(),
            files: base_files.clone(),
            base_files,
            touched_paths: Vec::new(),
        })
    }

    pub fn replace_or_add(
        &mut self,
        project_id: ProjectId,
        base_revision_sha256: [u8; 32],
        file: SourceFile<'_>,
    ) -> Result<(), OverlayError> {
        self.check_binding(project_id, base_revision_sha256)?;
        self.check_untouched(file.path)?;

        let mut candidate = self.files.clone();
        if let Some(index) = candidate.iter().position(|value| value.path == file.path) {
            if candidate[index].same_source(file) {
                return Err(OverlayError::NoOp);
            }
            candidate[index] = OwnedFile::from_source(file);
        } else {
            candidate.push(OwnedFile::from_source(file));
        }
        validate_files(&self.base, &candidate)?;
        self.files = candidate;
        self.touched_paths.push(String::from(file.path));
        Ok(())
    }

    pub fn delete(
        &mut self,
        project_id: ProjectId,
        base_revision_sha256: [u8; 32],
        path: &str,
    ) -> Result<(), OverlayError> {
        self.check_binding(project_id, base_revision_sha256)?;
        self.check_untouched(path)?;
        let Some(index) = self.files.iter().position(|file| file.path == path) else {
            return Err(OverlayError::InvalidDelete);
        };
        let mut candidate = self.files.clone();
        candidate.remove(index);
        validate_files(&self.base, &candidate)?;
        self.files = candidate;
        self.touched_paths.push(String::from(path));
        Ok(())
    }

    pub fn discard(&mut self) {
        self.files = self.base_files.clone();
        self.touched_paths.clear();
    }

    pub fn diff(&self) -> Vec<DiffEntry> {
        let mut diff = Vec::new();
        for old in &self.base_files {
            match self.files.iter().find(|new| new.path == old.path) {
                Some(new) if old.same_source(new.source()) => {}
                Some(new) => diff.push(DiffEntry {
                    kind: DiffKind::Replace,
                    path: old.path.clone(),
                    old: Some(old.entry()),
                    new: Some(new.entry()),
                }),
                None => diff.push(DiffEntry {
                    kind: DiffKind::Delete,
                    path: old.path.clone(),
                    old: Some(old.entry()),
                    new: None,
                }),
            }
        }
        for new in &self.files {
            if !self.base_files.iter().any(|old| old.path == new.path) {
                diff.push(DiffEntry {
                    kind: DiffKind::Add,
                    path: new.path.clone(),
                    old: None,
                    new: Some(new.entry()),
                });
            }
        }
        diff.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
        diff
    }

    pub fn build_revision(&self) -> Result<BuiltRevision, OverlayError> {
        if self.diff().is_empty() {
            return Err(OverlayError::NoOp);
        }
        let sources: Vec<_> = self.files.iter().map(OwnedFile::source).collect();
        Ok(build_agent_edit(
            self.base.project_id,
            self.base.revision_sha256,
            &sources,
        )?)
    }

    fn check_binding(
        &self,
        project_id: ProjectId,
        base_revision_sha256: [u8; 32],
    ) -> Result<(), OverlayError> {
        check_binding(project_id, base_revision_sha256, &self.base)
    }

    fn check_untouched(&self, path: &str) -> Result<(), OverlayError> {
        if self
            .touched_paths
            .iter()
            .any(|touched| paths_alias(touched, path))
        {
            Err(OverlayError::DuplicateOperation)
        } else {
            Ok(())
        }
    }
}

fn check_binding(
    project_id: ProjectId,
    base_revision_sha256: [u8; 32],
    base: &ProjectRevision,
) -> Result<(), OverlayError> {
    if project_id != base.project_id {
        return Err(OverlayError::WrongProject);
    }
    if base_revision_sha256 != base.revision_sha256 {
        return Err(OverlayError::WrongBaseRevision);
    }
    Ok(())
}

fn validate_files(base: &ProjectRevision, files: &[OwnedFile]) -> Result<(), OverlayError> {
    let sources: Vec<_> = files.iter().map(OwnedFile::source).collect();
    build_agent_edit(base.project_id, base.revision_sha256, &sources)?;
    Ok(())
}

fn rebuild_base(
    base: &ProjectRevision,
    files: &[SourceFile<'_>],
) -> Result<BuiltRevision, OverlayError> {
    Ok(match base.action {
        RevisionAction::OwnerLocalImport => {
            build_local_import(base.project_id, base.parent_revision_sha256, files)?
        }
        RevisionAction::AgentOverlayCommit => build_agent_edit(
            base.project_id,
            base.parent_revision_sha256
                .ok_or(OverlayError::BaseFilesMismatch)?,
            files,
        )?,
        RevisionAction::AgentAnswer => {
            build_agent_answer(base.project_id, base.parent_revision_sha256, files)?
        }
    })
}

fn paths_alias(left: &str, right: &str) -> bool {
    left.len() == right.len()
        && left
            .bytes()
            .zip(right.bytes())
            .all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_workspace::{MAX_FILE_BYTES, MAX_TOTAL_SOURCE_BYTES};

    const PROJECT: ProjectId = ProjectId::new([9; 16]);

    fn base_files<'a>() -> [SourceFile<'a>; 3] {
        [
            SourceFile {
                path: "a.txt",
                classification: Classification::LocalOnly,
                media_type: "text/plain",
                bytes: b"old a",
            },
            SourceFile {
                path: "b.txt",
                classification: Classification::Public,
                media_type: "text/plain",
                bytes: b"old b",
            },
            SourceFile {
                path: "c.txt",
                classification: Classification::LocalOnly,
                media_type: "text/plain",
                bytes: b"old c",
            },
        ]
    }

    fn setup() -> (ProjectRevision, ProjectOverlay) {
        let files = base_files();
        let base = build_local_import(PROJECT, None, &files).unwrap().revision;
        let overlay = ProjectOverlay::new(PROJECT, base.revision_sha256, &base, &files).unwrap();
        (base, overlay)
    }

    fn source<'a>(path: &'a str, bytes: &'a [u8]) -> SourceFile<'a> {
        SourceFile {
            path,
            classification: Classification::LocalOnly,
            media_type: "text/plain",
            bytes,
        }
    }

    #[test]
    fn add_replace_delete_produce_sorted_hash_bound_diff() {
        let (base, mut overlay) = setup();
        overlay
            .replace_or_add(PROJECT, base.revision_sha256, source("z.txt", b"new z"))
            .unwrap();
        overlay
            .delete(PROJECT, base.revision_sha256, "b.txt")
            .unwrap();
        overlay
            .replace_or_add(PROJECT, base.revision_sha256, source("a.txt", b"new a"))
            .unwrap();

        let diff = overlay.diff();
        assert_eq!(
            diff.iter()
                .map(|entry| entry.path.as_str())
                .collect::<Vec<_>>(),
            ["a.txt", "b.txt", "z.txt"]
        );
        assert_eq!(diff[0].kind, DiffKind::Replace);
        assert_eq!(diff[1].kind, DiffKind::Delete);
        assert_eq!(diff[2].kind, DiffKind::Add);
        assert_eq!(
            diff[0].old.as_ref().unwrap().blob_sha256,
            sha256_bytes(b"old a")
        );
        assert_eq!(
            diff[0].new.as_ref().unwrap().blob_sha256,
            sha256_bytes(b"new a")
        );
        assert_eq!(
            diff[1].old.as_ref().unwrap().classification,
            Classification::Public
        );
        assert_eq!(diff[2].new.as_ref().unwrap().byte_len, 5);
    }

    #[test]
    fn discard_restores_overlay_without_mutating_base() {
        let (base, mut overlay) = setup();
        let unchanged = base.clone();
        overlay
            .replace_or_add(PROJECT, base.revision_sha256, source("a.txt", b"changed"))
            .unwrap();
        overlay.discard();

        assert!(overlay.diff().is_empty());
        assert_eq!(base, unchanged);
        assert!(matches!(overlay.build_revision(), Err(OverlayError::NoOp)));
    }

    #[test]
    fn no_op_duplicate_invalid_delete_and_wrong_bindings_are_denied() {
        let (base, mut overlay) = setup();
        assert_eq!(
            overlay.replace_or_add(PROJECT, base.revision_sha256, source("a.txt", b"old a")),
            Err(OverlayError::NoOp)
        );
        assert_eq!(
            overlay.delete(PROJECT, base.revision_sha256, "missing.txt"),
            Err(OverlayError::InvalidDelete)
        );
        overlay
            .replace_or_add(PROJECT, base.revision_sha256, source("a.txt", b"new"))
            .unwrap();
        assert_eq!(
            overlay.delete(PROJECT, base.revision_sha256, "A.TXT"),
            Err(OverlayError::DuplicateOperation)
        );
        assert_eq!(
            overlay.delete(ProjectId::new([1; 16]), base.revision_sha256, "b.txt"),
            Err(OverlayError::WrongProject)
        );
        assert_eq!(
            overlay.delete(PROJECT, [0; 32], "b.txt"),
            Err(OverlayError::WrongBaseRevision)
        );
    }

    #[test]
    fn next_revision_binds_parent_and_builder_enforces_hashes_and_limits() {
        let (base, mut overlay) = setup();
        overlay
            .replace_or_add(PROJECT, base.revision_sha256, source("a.txt", b"next"))
            .unwrap();
        let built = overlay.build_revision().unwrap();
        assert_eq!(
            built.revision.parent_revision_sha256,
            Some(base.revision_sha256)
        );
        assert_eq!(built.revision.action, RevisionAction::AgentOverlayCommit);
        assert_ne!(built.revision.tree_sha256, base.tree_sha256);

        let oversized = vec![0; MAX_FILE_BYTES + 1];
        let (_, mut overlay) = setup();
        assert_eq!(
            overlay.replace_or_add(
                PROJECT,
                base.revision_sha256,
                source("large.bin", &oversized),
            ),
            Err(OverlayError::Workspace(WorkspaceError::FileTooLarge))
        );

        let base_files = base_files();
        let mut substituted = base_files;
        substituted[0].bytes = b"substituted";
        assert!(matches!(
            ProjectOverlay::new(PROJECT, base.revision_sha256, &base, &substituted),
            Err(OverlayError::BaseFilesMismatch)
        ));

        let too_large_a = vec![0; MAX_TOTAL_SOURCE_BYTES / 2 + 1];
        let too_large_b = vec![0; MAX_TOTAL_SOURCE_BYTES / 2 + 1];
        let files = [
            source("one.bin", &too_large_a),
            source("two.bin", &too_large_b),
        ];
        assert_eq!(
            build_local_import(PROJECT, None, &files),
            Err(WorkspaceError::TotalTooLarge)
        );
    }

    #[test]
    fn overlay_chains_from_verified_agent_revision() {
        let (base, mut first) = setup();
        first
            .replace_or_add(PROJECT, base.revision_sha256, source("a.txt", b"first"))
            .unwrap();
        let child = first.build_revision().unwrap();
        let child_files = [
            source("a.txt", b"first"),
            SourceFile {
                path: "b.txt",
                classification: Classification::Public,
                media_type: "text/plain",
                bytes: b"old b",
            },
            source("c.txt", b"old c"),
        ];
        let mut second = ProjectOverlay::new(
            PROJECT,
            child.revision.revision_sha256,
            &child.revision,
            &child_files,
        )
        .unwrap();
        second
            .replace_or_add(
                PROJECT,
                child.revision.revision_sha256,
                source("c.txt", b"second"),
            )
            .unwrap();
        let grandchild = second.build_revision().unwrap();

        assert_eq!(
            grandchild.revision.action,
            RevisionAction::AgentOverlayCommit
        );
        assert_eq!(
            grandchild.revision.parent_revision_sha256,
            Some(child.revision.revision_sha256)
        );
    }
}
