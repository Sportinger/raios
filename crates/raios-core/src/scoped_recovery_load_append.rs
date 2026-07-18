//! Scoped authorization for the `raios.recovery_load.v0` durable audit append
//! (M8D-2 `recovery.load_artifact_by_hash` — the recovery-lifeline authority flip).
//!
//! This is a dedicated CLONE of `scoped_repromotion_append` with its OWN pinned
//! identity (`recovery.load_artifact_by_hash` / `append.recovery_load.seed_data` /
//! `raios.recovery_load.v0` / `RAIOS_DATA_RECLOG`). It authorizes exactly ONE thing:
//! appending a recovery-load audit record through the shared reclog gauntlet
//! (scan -> plan -> write_readback -> reparse -> evaluate -> rescan) after the
//! UNMODIFIED M6 gate has genuinely re-instated the RAM-only current-boot service.
//! It NEVER shares the repromotion or recovery-action write boundary. `authorizes_load`
//! and `cross_reboot_proven` are permitted ONLY when the decision status is the
//! reinstated success status; `owner_sealed`, `persistence_claimed`, and any non
//! dev-tier trust tier are denied on ALL statuses; `promotion_authority_is_placeholder`
//! is required. Every unmet pin denies with a distinct, pairwise-unique reason.

pub const SCOPED_RECOVERY_LOAD_APPEND_DECISION_SCHEMA: &str =
    "raios.scoped_recovery_load_append_authorization_decision.v0";
pub const SCOPED_RECOVERY_LOAD_APPEND_DECISION_ID: &str =
    "scoped_recovery_load_append_authorization.current_boot.seed_data.reclog.v0";
pub const SCOPED_RECOVERY_LOAD_APPEND_DECISION_MARKER: &str =
    "RAIOS_RECOVERY_LOAD_APPEND_SCOPE_DECISION";

pub const EXPECTED_METHOD: &str = "recovery.load_artifact_by_hash";
pub const EXPECTED_TARGET_ID: &str = "append.recovery_load.seed_data";
pub const EXPECTED_RECORD_SCHEMA: &str = "raios.recovery_load.v0";
pub const EXPECTED_REGION_MARKER: &str = "RAIOS_DATA_RECLOG";
pub const EXPECTED_TRUST_TIER: &str = "dev_key_not_owner_sealed";

/// The recovery-load decision statuses this evaluator recognises. `reinstated` is the
/// success status: the M6 gate re-verified, re-loaded, AND re-started the persisted
/// artifact as a RAM-only current-boot service, so — and ONLY so — `authorizes_load`
/// and `cross_reboot_proven` may be true. `reverified` records a re-verified-but-not
/// re-instated chain (never `authorizes_load`), and `reinstate_denied` records a denial.
pub const DECISION_STATUS_REVERIFIED: &str = "reverified";
pub const DECISION_STATUS_REINSTATED: &str = "reinstated";
pub const DECISION_STATUS_REINSTATE_DENIED: &str = "reinstate_denied";
const SECTOR_SIZE: u64 = 512;

#[derive(Clone, Copy)]
pub struct ScopedRecoveryLoadAppendInput<'a> {
    pub method: Option<&'a str>,
    pub target_id: Option<&'a str>,
    pub record_schema: Option<&'a str>,
    pub region_marker: Option<&'a str>,
    pub frame_len: Option<u64>,
    pub write_offset: Option<u64>,
    pub reclog_byte_count: Option<u64>,
    pub absolute_start_lba: Option<u64>,
    pub reclog_lba_count: Option<u64>,
    pub seq: Option<u64>,
    pub tail_seq: Option<u64>,
    pub count: Option<u64>,
    pub prev_frame_sha256: Option<[u8; 32]>,
    pub tail_frame_sha256: Option<[u8; 32]>,
    pub payload_sha256: Option<[u8; 32]>,
    pub planned_payload_sha256: Option<[u8; 32]>,
    pub planned_frame_sha256: Option<[u8; 32]>,
    pub readback_frame_sha256: Option<[u8; 32]>,
    pub write_attempted: bool,
    pub write_completed: bool,
    pub readback_completed: bool,
    pub readback_matches_planned: bool,
    pub reparse_valid: bool,
    pub span_in_bounds: bool,
    pub decision_status: Option<&'a str>,
    pub would_reinstate: bool,
    pub authorizes_load: bool,
    pub trust_tier: Option<&'a str>,
    pub owner_sealed: bool,
    pub promotion_authority_is_placeholder: bool,
    pub cross_reboot_proven: bool,
    pub persistence_claimed: bool,
}

impl<'a> ScopedRecoveryLoadAppendInput<'a> {
    pub const fn empty() -> Self {
        Self {
            method: None,
            target_id: None,
            record_schema: None,
            region_marker: None,
            frame_len: None,
            write_offset: None,
            reclog_byte_count: None,
            absolute_start_lba: None,
            reclog_lba_count: None,
            seq: None,
            tail_seq: None,
            count: None,
            prev_frame_sha256: None,
            tail_frame_sha256: None,
            payload_sha256: None,
            planned_payload_sha256: None,
            planned_frame_sha256: None,
            readback_frame_sha256: None,
            write_attempted: false,
            write_completed: false,
            readback_completed: false,
            readback_matches_planned: false,
            reparse_valid: false,
            span_in_bounds: false,
            decision_status: None,
            would_reinstate: false,
            authorizes_load: false,
            trust_tier: None,
            owner_sealed: false,
            promotion_authority_is_placeholder: false,
            cross_reboot_proven: false,
            persistence_claimed: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopedRecoveryLoadAppendDecision {
    pub performed: bool,
    pub status: &'static str,
    pub reason: &'static str,
}

pub fn evaluate_scoped_recovery_load_append(
    input: &ScopedRecoveryLoadAppendInput<'_>,
) -> ScopedRecoveryLoadAppendDecision {
    if let Err(decision) = require_str(
        input.method,
        EXPECTED_METHOD,
        "missing_method",
        "method_out_of_scope",
    ) {
        return decision;
    }
    if let Err(decision) = require_str(
        input.target_id,
        EXPECTED_TARGET_ID,
        "missing_target_id",
        "target_out_of_scope",
    ) {
        return decision;
    }
    if let Err(decision) = require_str(
        input.record_schema,
        EXPECTED_RECORD_SCHEMA,
        "missing_record_schema",
        "record_schema_mismatch",
    ) {
        return decision;
    }
    if let Err(decision) = require_str(
        input.region_marker,
        EXPECTED_REGION_MARKER,
        "missing_region_marker",
        "region_marker_mismatch",
    ) {
        return decision;
    }

    let frame_len = match input.frame_len {
        Some(value) if value >= SECTOR_SIZE && value % SECTOR_SIZE == 0 => value,
        Some(_) => return denied("frame_len_out_of_scope"),
        None => return denied("missing_frame_len"),
    };
    let write_offset = match input.write_offset {
        Some(value) if value % SECTOR_SIZE == 0 => value,
        Some(_) => return denied("write_offset_misaligned"),
        None => return denied("missing_write_offset"),
    };
    let reclog_byte_count = match input.reclog_byte_count {
        Some(value) => value,
        None => return denied("missing_reclog_byte_count"),
    };
    if write_offset
        .checked_add(frame_len)
        .map(|end| end > reclog_byte_count)
        .unwrap_or(true)
    {
        return denied("durable_store_full");
    }

    let absolute_start_lba = match input.absolute_start_lba {
        Some(value) => value,
        None => return denied("missing_absolute_start_lba"),
    };
    let reclog_lba_count = match input.reclog_lba_count {
        Some(value) => value,
        None => return denied("missing_reclog_lba_count"),
    };
    if !write_span_inside_reclog(
        absolute_start_lba,
        reclog_lba_count,
        write_offset,
        frame_len,
    ) {
        return denied("write_span_out_of_reclog");
    }

    let count = match input.count {
        Some(value) => value,
        None => return denied("missing_count"),
    };
    let seq = match input.seq {
        Some(value) => value,
        None => return denied("missing_seq"),
    };
    let expected_seq = if count == 0 {
        1
    } else {
        match input.tail_seq {
            Some(value) => match value.checked_add(1) {
                Some(next) => next,
                None => return denied("bad_seq"),
            },
            None => return denied("missing_tail_seq"),
        }
    };
    if seq != expected_seq {
        return denied("bad_seq");
    }

    let prev_frame_sha256 = match input.prev_frame_sha256 {
        Some(value) => value,
        None => return denied("missing_prev_frame_sha256"),
    };
    let expected_prev = if count == 0 {
        [0u8; 32]
    } else {
        match input.tail_frame_sha256 {
            Some(value) => value,
            None => return denied("missing_tail_frame_sha256"),
        }
    };
    if prev_frame_sha256 != expected_prev {
        return denied("bad_prev");
    }

    match (input.payload_sha256, input.planned_payload_sha256) {
        (Some(actual), Some(expected)) if actual == expected => {}
        _ => return denied("bad_payload_sha256"),
    }
    match (input.planned_frame_sha256, input.readback_frame_sha256) {
        (Some(planned), Some(readback)) if planned == readback => {}
        _ => return denied("frame_sha256_mismatch"),
    }

    if !input.write_attempted {
        return denied("write_not_attempted");
    }
    if !input.write_completed {
        return denied("write_not_completed");
    }
    if !input.readback_completed {
        return denied("readback_not_completed");
    }
    if !input.readback_matches_planned {
        return denied("readback_hash_mismatch");
    }
    if !input.reparse_valid {
        return denied("reparse_not_valid");
    }
    if !input.span_in_bounds {
        return denied("span_not_in_bounds");
    }

    let decision_status = input.decision_status;
    match decision_status {
        Some(DECISION_STATUS_REVERIFIED) if input.would_reinstate => {}
        Some(DECISION_STATUS_REVERIFIED) => {
            return denied("reverified_decision_missing_would_reinstate")
        }
        Some(DECISION_STATUS_REINSTATED) if input.would_reinstate => {}
        Some(DECISION_STATUS_REINSTATED) => {
            return denied("reinstated_decision_missing_would_reinstate")
        }
        Some(DECISION_STATUS_REINSTATE_DENIED) if !input.would_reinstate => {}
        Some(DECISION_STATUS_REINSTATE_DENIED) => {
            return denied("denial_decision_cannot_would_reinstate")
        }
        Some(_) => return denied("decision_status_out_of_scope"),
        None => return denied("missing_decision_status"),
    }
    if input.authorizes_load && decision_status != Some(DECISION_STATUS_REINSTATED) {
        return denied("authorizes_load_not_allowed");
    }
    if input.trust_tier != Some(EXPECTED_TRUST_TIER) {
        return denied("trust_tier_not_dev_key_not_owner_sealed");
    }
    if input.owner_sealed {
        return denied("owner_sealed_not_allowed");
    }
    if !input.promotion_authority_is_placeholder {
        return denied("promotion_authority_not_placeholder");
    }
    if input.cross_reboot_proven && decision_status != Some(DECISION_STATUS_REINSTATED) {
        return denied("cross_reboot_proven_not_allowed");
    }
    if input.persistence_claimed {
        return denied("persistence_claimed_not_allowed");
    }

    ScopedRecoveryLoadAppendDecision {
        performed: true,
        status: "appended",
        reason: "authorized_recovery_load_audit_append_readback_reparse_verified",
    }
}

fn require_str(
    actual: Option<&str>,
    expected: &str,
    missing: &'static str,
    mismatch: &'static str,
) -> Result<(), ScopedRecoveryLoadAppendDecision> {
    match actual {
        Some(actual) if actual == expected => Ok(()),
        Some(_) => Err(denied(mismatch)),
        None => Err(denied(missing)),
    }
}

fn write_span_inside_reclog(
    absolute_start_lba: u64,
    reclog_lba_count: u64,
    write_offset: u64,
    frame_len: u64,
) -> bool {
    let Some(reclog_end_lba) = absolute_start_lba.checked_add(reclog_lba_count) else {
        return false;
    };
    let sector_offset = write_offset / SECTOR_SIZE;
    let sector_count = frame_len / SECTOR_SIZE;
    let mut idx = 0u64;
    while idx < sector_count {
        let Some(lba) = absolute_start_lba
            .checked_add(sector_offset)
            .and_then(|base| base.checked_add(idx))
        else {
            return false;
        };
        if lba < absolute_start_lba || lba >= reclog_end_lba {
            return false;
        }
        idx += 1;
    }
    true
}

fn denied(reason: &'static str) -> ScopedRecoveryLoadAppendDecision {
    ScopedRecoveryLoadAppendDecision {
        performed: false,
        status: "denied",
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(value: u8) -> [u8; 32] {
        [value; 32]
    }

    fn valid_input(status: &'static str) -> ScopedRecoveryLoadAppendInput<'static> {
        ScopedRecoveryLoadAppendInput {
            method: Some(EXPECTED_METHOD),
            target_id: Some(EXPECTED_TARGET_ID),
            record_schema: Some(EXPECTED_RECORD_SCHEMA),
            region_marker: Some(EXPECTED_REGION_MARKER),
            frame_len: Some(512),
            write_offset: Some(1024),
            reclog_byte_count: Some(4096),
            absolute_start_lba: Some(100),
            reclog_lba_count: Some(8),
            seq: Some(3),
            tail_seq: Some(2),
            count: Some(2),
            prev_frame_sha256: Some(h(2)),
            tail_frame_sha256: Some(h(2)),
            payload_sha256: Some(h(3)),
            planned_payload_sha256: Some(h(3)),
            planned_frame_sha256: Some(h(4)),
            readback_frame_sha256: Some(h(4)),
            write_attempted: true,
            write_completed: true,
            readback_completed: true,
            readback_matches_planned: true,
            reparse_valid: true,
            span_in_bounds: true,
            decision_status: Some(status),
            would_reinstate: matches!(status, "reverified" | "reinstated"),
            authorizes_load: status == "reinstated",
            trust_tier: Some(EXPECTED_TRUST_TIER),
            owner_sealed: false,
            promotion_authority_is_placeholder: true,
            cross_reboot_proven: status == "reinstated",
            persistence_claimed: false,
        }
    }

    #[test]
    fn authorizes_reverified_and_denied_recovery_load_audit_records() {
        for status in ["reverified", "reinstate_denied"] {
            assert_eq!(
                evaluate_scoped_recovery_load_append(&valid_input(status)),
                ScopedRecoveryLoadAppendDecision {
                    performed: true,
                    status: "appended",
                    reason: "authorized_recovery_load_audit_append_readback_reparse_verified"
                }
            );
        }
    }

    #[test]
    fn authorizes_reinstated_load_audit_record() {
        assert_eq!(
            evaluate_scoped_recovery_load_append(&valid_input("reinstated")),
            ScopedRecoveryLoadAppendDecision {
                performed: true,
                status: "appended",
                reason: "authorized_recovery_load_audit_append_readback_reparse_verified"
            }
        );
    }

    #[test]
    fn reinstated_status_keeps_dev_tier_pins() {
        let cases: [(
            fn(&mut ScopedRecoveryLoadAppendInput<'static>),
            &'static str,
        ); 4] = [
            (
                |input: &mut ScopedRecoveryLoadAppendInput<'static>| input.owner_sealed = true,
                "owner_sealed_not_allowed",
            ),
            (
                |input: &mut ScopedRecoveryLoadAppendInput<'static>| {
                    input.persistence_claimed = true
                },
                "persistence_claimed_not_allowed",
            ),
            (
                |input: &mut ScopedRecoveryLoadAppendInput<'static>| {
                    input.trust_tier = Some("owner_sealed")
                },
                "trust_tier_not_dev_key_not_owner_sealed",
            ),
            (
                |input: &mut ScopedRecoveryLoadAppendInput<'static>| input.would_reinstate = false,
                "reinstated_decision_missing_would_reinstate",
            ),
        ];

        let mut idx = 0usize;
        while idx < cases.len() {
            let mut input = valid_input("reinstated");
            cases[idx].0(&mut input);
            let decision = evaluate_scoped_recovery_load_append(&input);
            assert_eq!(decision.status, "denied");
            assert_eq!(decision.reason, cases[idx].1);
            idx += 1;
        }
    }

    #[test]
    fn reverified_still_cannot_claim_load_or_cross_reboot() {
        let mut input = valid_input("reverified");
        input.authorizes_load = true;
        assert_eq!(
            evaluate_scoped_recovery_load_append(&input).reason,
            "authorizes_load_not_allowed"
        );

        let mut input = valid_input("reverified");
        input.cross_reboot_proven = true;
        assert_eq!(
            evaluate_scoped_recovery_load_append(&input).reason,
            "cross_reboot_proven_not_allowed"
        );
    }

    #[test]
    fn empty_log_chain_expects_seq_one_and_zero_prev() {
        let mut input = valid_input("reverified");
        input.count = Some(0);
        input.tail_seq = None;
        input.seq = Some(1);
        input.tail_frame_sha256 = None;
        input.prev_frame_sha256 = Some([0u8; 32]);

        assert!(evaluate_scoped_recovery_load_append(&input).performed);
    }

    #[derive(Clone, Copy)]
    enum Mutation {
        MissingMethod,
        WrongMethod,
        MissingTarget,
        WrongTarget,
        MissingSchema,
        WrongSchema,
        MissingMarker,
        WrongMarker,
        MissingFrameLen,
        BadFrameLen,
        MissingWriteOffset,
        MisalignedWriteOffset,
        MissingByteCount,
        StoreFull,
        MissingStartLba,
        MissingLbaCount,
        SpanOut,
        MissingCount,
        MissingSeq,
        MissingTailSeq,
        BadSeq,
        MissingPrev,
        MissingTail,
        BadPrev,
        BadPayloadHash,
        FrameHashMismatch,
        WriteNotAttempted,
        WriteNotCompleted,
        ReadbackNotCompleted,
        ReadbackHashMismatch,
        ReparseNotValid,
        SpanFlagFalse,
        MissingDecisionStatus,
        WrongDecisionStatus,
        ReverifiedWithoutWould,
        DenialWithWould,
        AuthorizesLoad,
        WrongTrustTier,
        OwnerSealed,
        PromotionAuthorityNotPlaceholder,
        CrossRebootProven,
        PersistenceClaimed,
    }

    fn apply(input: &mut ScopedRecoveryLoadAppendInput<'static>, mutation: Mutation) {
        match mutation {
            Mutation::MissingMethod => input.method = None,
            Mutation::WrongMethod => input.method = Some("repromotion.run"),
            Mutation::MissingTarget => input.target_id = None,
            Mutation::WrongTarget => input.target_id = Some("append.repromotion_audit.seed_data"),
            Mutation::MissingSchema => input.record_schema = None,
            Mutation::WrongSchema => input.record_schema = Some("raios.repromotion.v0"),
            Mutation::MissingMarker => input.region_marker = None,
            Mutation::WrongMarker => input.region_marker = Some("RAIOS_AUDITRB_V0"),
            Mutation::MissingFrameLen => input.frame_len = None,
            Mutation::BadFrameLen => input.frame_len = Some(513),
            Mutation::MissingWriteOffset => input.write_offset = None,
            Mutation::MisalignedWriteOffset => input.write_offset = Some(1),
            Mutation::MissingByteCount => input.reclog_byte_count = None,
            Mutation::StoreFull => input.reclog_byte_count = Some(1200),
            Mutation::MissingStartLba => input.absolute_start_lba = None,
            Mutation::MissingLbaCount => input.reclog_lba_count = None,
            Mutation::SpanOut => input.reclog_lba_count = Some(2),
            Mutation::MissingCount => input.count = None,
            Mutation::MissingSeq => input.seq = None,
            Mutation::MissingTailSeq => input.tail_seq = None,
            Mutation::BadSeq => input.seq = Some(4),
            Mutation::MissingPrev => input.prev_frame_sha256 = None,
            Mutation::MissingTail => input.tail_frame_sha256 = None,
            Mutation::BadPrev => input.prev_frame_sha256 = Some(h(9)),
            Mutation::BadPayloadHash => input.payload_sha256 = Some(h(9)),
            Mutation::FrameHashMismatch => input.readback_frame_sha256 = Some(h(9)),
            Mutation::WriteNotAttempted => input.write_attempted = false,
            Mutation::WriteNotCompleted => input.write_completed = false,
            Mutation::ReadbackNotCompleted => input.readback_completed = false,
            Mutation::ReadbackHashMismatch => input.readback_matches_planned = false,
            Mutation::ReparseNotValid => input.reparse_valid = false,
            Mutation::SpanFlagFalse => input.span_in_bounds = false,
            Mutation::MissingDecisionStatus => input.decision_status = None,
            Mutation::WrongDecisionStatus => input.decision_status = Some("skipped"),
            Mutation::ReverifiedWithoutWould => input.would_reinstate = false,
            Mutation::DenialWithWould => {
                input.decision_status = Some("reinstate_denied");
                input.would_reinstate = true;
            }
            Mutation::AuthorizesLoad => input.authorizes_load = true,
            Mutation::WrongTrustTier => input.trust_tier = Some("owner_sealed"),
            Mutation::OwnerSealed => input.owner_sealed = true,
            Mutation::PromotionAuthorityNotPlaceholder => {
                input.promotion_authority_is_placeholder = false
            }
            Mutation::CrossRebootProven => input.cross_reboot_proven = true,
            Mutation::PersistenceClaimed => input.persistence_claimed = true,
        }
    }

    #[test]
    fn denial_truth_table_names_first_failed_pin() {
        let cases = [
            (Mutation::MissingMethod, "missing_method"),
            (Mutation::WrongMethod, "method_out_of_scope"),
            (Mutation::MissingTarget, "missing_target_id"),
            (Mutation::WrongTarget, "target_out_of_scope"),
            (Mutation::MissingSchema, "missing_record_schema"),
            (Mutation::WrongSchema, "record_schema_mismatch"),
            (Mutation::MissingMarker, "missing_region_marker"),
            (Mutation::WrongMarker, "region_marker_mismatch"),
            (Mutation::MissingFrameLen, "missing_frame_len"),
            (Mutation::BadFrameLen, "frame_len_out_of_scope"),
            (Mutation::MissingWriteOffset, "missing_write_offset"),
            (Mutation::MisalignedWriteOffset, "write_offset_misaligned"),
            (Mutation::MissingByteCount, "missing_reclog_byte_count"),
            (Mutation::StoreFull, "durable_store_full"),
            (Mutation::MissingStartLba, "missing_absolute_start_lba"),
            (Mutation::MissingLbaCount, "missing_reclog_lba_count"),
            (Mutation::SpanOut, "write_span_out_of_reclog"),
            (Mutation::MissingCount, "missing_count"),
            (Mutation::MissingSeq, "missing_seq"),
            (Mutation::MissingTailSeq, "missing_tail_seq"),
            (Mutation::BadSeq, "bad_seq"),
            (Mutation::MissingPrev, "missing_prev_frame_sha256"),
            (Mutation::MissingTail, "missing_tail_frame_sha256"),
            (Mutation::BadPrev, "bad_prev"),
            (Mutation::BadPayloadHash, "bad_payload_sha256"),
            (Mutation::FrameHashMismatch, "frame_sha256_mismatch"),
            (Mutation::WriteNotAttempted, "write_not_attempted"),
            (Mutation::WriteNotCompleted, "write_not_completed"),
            (Mutation::ReadbackNotCompleted, "readback_not_completed"),
            (Mutation::ReadbackHashMismatch, "readback_hash_mismatch"),
            (Mutation::ReparseNotValid, "reparse_not_valid"),
            (Mutation::SpanFlagFalse, "span_not_in_bounds"),
            (Mutation::MissingDecisionStatus, "missing_decision_status"),
            (
                Mutation::WrongDecisionStatus,
                "decision_status_out_of_scope",
            ),
            (
                Mutation::ReverifiedWithoutWould,
                "reverified_decision_missing_would_reinstate",
            ),
            (
                Mutation::DenialWithWould,
                "denial_decision_cannot_would_reinstate",
            ),
            (Mutation::AuthorizesLoad, "authorizes_load_not_allowed"),
            (
                Mutation::WrongTrustTier,
                "trust_tier_not_dev_key_not_owner_sealed",
            ),
            (Mutation::OwnerSealed, "owner_sealed_not_allowed"),
            (
                Mutation::PromotionAuthorityNotPlaceholder,
                "promotion_authority_not_placeholder",
            ),
            (
                Mutation::CrossRebootProven,
                "cross_reboot_proven_not_allowed",
            ),
            (
                Mutation::PersistenceClaimed,
                "persistence_claimed_not_allowed",
            ),
        ];

        let mut idx = 0usize;
        while idx < cases.len() {
            let mut input = valid_input("reverified");
            apply(&mut input, cases[idx].0);
            let decision = evaluate_scoped_recovery_load_append(&input);
            assert_eq!(decision.status, "denied");
            assert_eq!(decision.reason, cases[idx].1);
            assert!(!decision.performed);

            let mut other = 0usize;
            while other < idx {
                assert_ne!(cases[idx].1, cases[other].1);
                other += 1;
            }
            idx += 1;
        }
    }
}
