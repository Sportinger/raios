//! M9C-1a: durable-memory resolver — latest-non-superseded-per-id with the R1
//! audit-immutability rule.
//!
//! Given the `(frame_seq, view)` pairs a [`crate::durable_record_frame::scan_reclog_payloads`]
//! walk + [`crate::memory_record::parse`] produced, this computes which memory
//! records are currently VISIBLE, applying three load-bearing rules:
//!
//! * LOW-1 — recency is ranked by the durable FRAME SEQ, never by the record's
//!   self-reported `view.sequence`. The latest version of an id is the one with
//!   the highest frame seq.
//! * R1 — a `supersedes` link that targets an audit/authority kind
//!   (capability_grant/denial, promotion_tx_ref, rollback_tx_ref, export_audit)
//!   is IGNORED: the audit target stays visible and the ignored link is recorded.
//!   The reclog audit trail must never be hideable by a later record. This holds
//!   BOTH ways an audit could be hidden: (a) a supersede link (above), and (b)
//!   id-shadowing — a later same-id NON-audit record can never displace an audit
//!   record from the visible set. The audit record stays the winner for its id and
//!   the attempted reuse is flagged in `audit_id_reused`. So audit-immutability is
//!   enforced by the resolver itself, not merely assumed from write-path id
//!   uniqueness.
//! * LOW-3 — record identity is the record's own `id` ONLY. The resolver never
//!   reads `source.record_id` for identity, so a spoofed source cannot re-home a
//!   record onto another id.
//!
//! This decodes nothing and grants nothing; it only reorders already-parsed views.

use alloc::vec::Vec;

use crate::memory_record::MemoryRecordView;

/// The outcome of resolving a set of parsed durable-memory views.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedMemory<'a> {
    /// Latest-per-id, not hidden by a supersede link, in ascending frame-seq order.
    pub visible: Vec<(u64, MemoryRecordView<'a>)>,
    /// Ids hidden because a surviving record superseded them (non-audit targets).
    pub superseded_hidden: Vec<&'a str>,
    /// `(superseding_id, target_id)` links ignored by R1 because the target is an
    /// audit/authority kind that may never be hidden.
    pub r1_ignored_supersedes: Vec<(&'a str, &'a str)>,
    /// Supersede targets that name an id not present among the resolved records.
    pub dangling_supersedes: Vec<&'a str>,
    /// Ids for which an audit-kind record and a non-audit record co-exist (id reuse).
    /// R1 audit-immutability keeps the audit record visible; the reuse is flagged
    /// here rather than silently letting the non-audit record shadow the audit.
    pub audit_id_reused: Vec<&'a str>,
}

/// Resolves the visible durable-memory set from `(frame_seq, view)` pairs.
///
/// See the module docs for LOW-1 / R1 / LOW-3. Ranking uses the tuple's frame
/// seq (`tuple.0`); on an (invalid) frame-seq tie the later slice position wins.
pub fn resolve_durable_memory<'a>(views: &'a [(u64, MemoryRecordView<'a>)]) -> ResolvedMemory<'a> {
    // Step 1 (LOW-1 + LOW-3 + audit-immutability): the winning (latest) slice index
    // per record id, keyed on `view.id` ONLY, ranked by frame seq (`tuple.0`) — EXCEPT
    // an audit-kind record for an id can never be displaced from `visible` by a
    // non-audit record of the same id (id-shadowing is the second way an audit could
    // be hidden; the reuse is flagged in `audit_id_reused`).
    let mut winners: Vec<usize> = Vec::new();
    let mut audit_id_reused: Vec<&'a str> = Vec::new();
    for (index, (seq, view)) in views.iter().enumerate() {
        let mut existing = None;
        for (slot, &winner_index) in winners.iter().enumerate() {
            if views[winner_index].1.id == view.id {
                existing = Some(slot);
                break;
            }
        }
        match existing {
            Some(slot) => {
                let winner_is_audit = views[winners[slot]].1.kind.is_audit();
                let new_is_audit = view.kind.is_audit();
                match (winner_is_audit, new_is_audit) {
                    // Audit-immutability: a non-audit record may NEVER displace an
                    // audit record of the same id. Keep the audit; flag the reuse.
                    (true, false) => {
                        if !audit_id_reused.contains(&view.id) {
                            audit_id_reused.push(view.id);
                        }
                    }
                    // An audit record for this id always wins over a prior non-audit
                    // same-id record (the audit is immutable and authoritative).
                    (false, true) => {
                        if !audit_id_reused.contains(&view.id) {
                            audit_id_reused.push(view.id);
                        }
                        winners[slot] = index;
                    }
                    // Same audit-ness: higher frame seq wins; equal seq -> later wins.
                    _ => {
                        if *seq >= views[winners[slot]].0 {
                            winners[slot] = index;
                        }
                    }
                }
            }
            None => winners.push(index),
        }
    }

    // Deterministic output order: ascending frame seq, then slice index.
    winners.sort_by(|&a, &b| views[a].0.cmp(&views[b].0).then(a.cmp(&b)));

    // Step 2 (R1): resolve each surviving record's supersede links against the
    // winning view of each target id.
    let mut superseded_hidden: Vec<&'a str> = Vec::new();
    let mut r1_ignored_supersedes: Vec<(&'a str, &'a str)> = Vec::new();
    let mut dangling_supersedes: Vec<&'a str> = Vec::new();

    for &winner_index in &winners {
        let view = &views[winner_index].1;
        for &target_id in &view.supersedes {
            // Belt-and-suspenders: `parse` already rejects a self-supersede, but the
            // resolver must not hide a record via its own id if a non-parse producer
            // ever slips one through.
            if target_id == view.id {
                continue;
            }
            let target_kind = winners
                .iter()
                .map(|&idx| &views[idx].1)
                .find(|candidate| candidate.id == target_id)
                .map(|candidate| candidate.kind);

            match target_kind {
                // Target id is absent from the resolved set.
                None => {
                    if !dangling_supersedes.contains(&target_id) {
                        dangling_supersedes.push(target_id);
                    }
                }
                // R1: audit/authority targets are immutable — ignore the link.
                Some(kind) if kind.is_audit() => {
                    r1_ignored_supersedes.push((view.id, target_id));
                }
                // Ordinary supersede: hide the (non-audit) target.
                Some(_) => {
                    if !superseded_hidden.contains(&target_id) {
                        superseded_hidden.push(target_id);
                    }
                }
            }
        }
    }

    // Step 3: visible = winners whose id is not hidden, in frame-seq order.
    let mut visible: Vec<(u64, MemoryRecordView<'a>)> = Vec::new();
    for &winner_index in &winners {
        let (seq, view) = (views[winner_index].0, &views[winner_index].1);
        if !superseded_hidden.contains(&view.id) {
            visible.push((seq, view.clone()));
        }
    }

    ResolvedMemory {
        visible,
        superseded_hidden,
        r1_ignored_supersedes,
        dangling_supersedes,
        audit_id_reused,
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_durable_memory, ResolvedMemory};
    use crate::memory_record::{Classification, MemoryKind, MemoryRecordView, MemorySource};
    use std::vec::Vec;

    /// Builds a view with a given id/kind/supersedes; other fields are inert.
    fn view(
        id: &'static str,
        kind: MemoryKind,
        supersedes: Vec<&'static str>,
    ) -> MemoryRecordView<'static> {
        MemoryRecordView {
            id,
            kind,
            entity: "e",
            predicate: "p",
            classification: Classification::LocalOnly,
            authority: "a",
            boot_id: "boot:1",
            sequence: 0,
            source: MemorySource::new("m", "r"),
            evidence: Vec::new(),
            tags: Vec::new(),
            supersedes,
            created_at_ticks: 0,
        }
    }

    fn visible_ids<'a>(resolved: &ResolvedMemory<'a>) -> Vec<&'a str> {
        resolved.visible.iter().map(|(_, v)| v.id).collect()
    }

    #[test]
    fn latest_per_id_is_ranked_by_frame_seq_not_view_sequence() {
        // Two versions of "mem.a": the OLDER frame (seq 1) carries a HIGHER
        // self-reported view.sequence; the NEWER frame (seq 2) carries a lower
        // one. LOW-1: the frame seq decides, so the seq-2 version is visible.
        let mut old = view("mem.a", MemoryKind::Decision, vec![]);
        old.sequence = 999;
        old.predicate = "old";
        let mut new = view("mem.a", MemoryKind::Decision, vec![]);
        new.sequence = 0;
        new.predicate = "new";

        let views = vec![(1u64, old), (2u64, new)];
        let resolved = resolve_durable_memory(&views);

        assert_eq!(resolved.visible.len(), 1);
        assert_eq!(resolved.visible[0].0, 2);
        assert_eq!(resolved.visible[0].1.predicate, "new");
    }

    #[test]
    fn decision_superseding_non_audit_hides_the_target() {
        let target = view("mem.problem.old", MemoryKind::Problem, vec![]);
        let superseder = view(
            "mem.problem.new",
            MemoryKind::Problem,
            vec!["mem.problem.old"],
        );

        let views = vec![(1u64, target), (2u64, superseder)];
        let resolved = resolve_durable_memory(&views);

        assert_eq!(visible_ids(&resolved), vec!["mem.problem.new"]);
        assert_eq!(resolved.superseded_hidden, vec!["mem.problem.old"]);
        assert!(resolved.r1_ignored_supersedes.is_empty());
        assert!(resolved.dangling_supersedes.is_empty());
    }

    #[test]
    fn r1_ignores_supersede_of_each_audit_kind_and_keeps_target_visible() {
        for audit_kind in [
            MemoryKind::CapabilityGrant,
            MemoryKind::CapabilityDenial,
            MemoryKind::PromotionTxRef,
            MemoryKind::RollbackTxRef,
            MemoryKind::ExportAudit,
        ] {
            assert!(audit_kind.is_audit());
            let target = view("mem.audit", audit_kind, vec![]);
            let superseder = view("mem.decision", MemoryKind::Decision, vec!["mem.audit"]);

            let views = vec![(1u64, target), (2u64, superseder)];
            let resolved = resolve_durable_memory(&views);

            // Target stays visible; both records are visible.
            let mut ids = visible_ids(&resolved);
            ids.sort();
            assert_eq!(ids, vec!["mem.audit", "mem.decision"]);
            assert!(resolved.superseded_hidden.is_empty());
            assert_eq!(
                resolved.r1_ignored_supersedes,
                vec![("mem.decision", "mem.audit")]
            );
            assert!(resolved.dangling_supersedes.is_empty());
        }
    }

    #[test]
    fn dangling_supersede_target_is_recorded() {
        let superseder = view("mem.decision", MemoryKind::Decision, vec!["mem.ghost"]);
        let views = vec![(1u64, superseder)];
        let resolved = resolve_durable_memory(&views);

        assert_eq!(visible_ids(&resolved), vec!["mem.decision"]);
        assert!(resolved.superseded_hidden.is_empty());
        assert!(resolved.r1_ignored_supersedes.is_empty());
        assert_eq!(resolved.dangling_supersedes, vec!["mem.ghost"]);
    }

    #[test]
    fn low3_identity_ignores_spoofed_source_record_id() {
        // viewX (id "mem.real") carries a source.record_id spoofed to "mem.decoy";
        // viewY genuinely has id "mem.decoy". A superseder targets "mem.decoy" and
        // must hide viewY (by id), never viewX — identity is the record id only.
        let mut real = view("mem.real", MemoryKind::Decision, vec![]);
        real.source = MemorySource::new("m", "mem.decoy");
        let decoy = view("mem.decoy", MemoryKind::Problem, vec![]);
        let superseder = view("mem.super", MemoryKind::Decision, vec!["mem.decoy"]);

        let views = vec![(1u64, real), (2u64, decoy), (3u64, superseder)];
        let resolved = resolve_durable_memory(&views);

        let mut ids = visible_ids(&resolved);
        ids.sort();
        assert_eq!(ids, vec!["mem.real", "mem.super"]);
        assert_eq!(resolved.superseded_hidden, vec!["mem.decoy"]);
    }

    #[test]
    fn audit_record_is_never_shadowed_by_a_later_same_id_non_audit_record() {
        // A capability_denial (audit) at seq 1, then a decision REUSING its id at a
        // HIGHER seq 2. Latest-per-id would pick the decision — but audit-immutability
        // keeps the audit visible and flags the reuse (id-shadowing is the 2nd way an
        // audit could be hidden, alongside supersede links).
        let audit = view("mem.x", MemoryKind::CapabilityDenial, vec![]);
        let shadow = view("mem.x", MemoryKind::Decision, vec![]);
        let views = vec![(1u64, audit), (2u64, shadow)];
        let resolved = resolve_durable_memory(&views);

        assert_eq!(resolved.visible.len(), 1);
        assert_eq!(resolved.visible[0].1.kind, MemoryKind::CapabilityDenial);
        assert_eq!(resolved.audit_id_reused, vec!["mem.x"]);

        // Reverse order (non-audit first, audit at higher seq): the audit still wins.
        let shadow0 = view("mem.x", MemoryKind::Decision, vec![]);
        let audit2 = view("mem.x", MemoryKind::CapabilityDenial, vec![]);
        let views2 = vec![(1u64, shadow0), (2u64, audit2)];
        let resolved2 = resolve_durable_memory(&views2);
        assert_eq!(resolved2.visible.len(), 1);
        assert_eq!(resolved2.visible[0].1.kind, MemoryKind::CapabilityDenial);
        assert_eq!(resolved2.audit_id_reused, vec!["mem.x"]);
    }

    #[test]
    fn visible_is_ordered_by_frame_seq() {
        let views = vec![
            (3u64, view("mem.c", MemoryKind::Decision, vec![])),
            (1u64, view("mem.a", MemoryKind::Decision, vec![])),
            (2u64, view("mem.b", MemoryKind::Decision, vec![])),
        ];
        let resolved = resolve_durable_memory(&views);
        assert_eq!(visible_ids(&resolved), vec!["mem.a", "mem.b", "mem.c"]);
    }
}
