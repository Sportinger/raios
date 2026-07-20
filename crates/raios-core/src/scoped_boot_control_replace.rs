use crate::boot_control::{
    BootStorageSlot, BOOTCTL_REGION_BYTE_COUNT, BOOTCTL_SLOT_SIZE, BOOTCTL_STORAGE_SLOT_SECTORS,
    BOOT_CONTROL_SCHEMA,
};

pub const SCOPED_BOOT_CONTROL_REPLACE_DECISION_SCHEMA: &str =
    "raios.scoped_boot_control_replace_authorization_decision.v0";
pub const SCOPED_BOOT_CONTROL_REPLACE_DECISION_ID: &str =
    "scoped_boot_control_replace_authorization.current_boot.seed_data.bootctl.v0";
pub const SCOPED_BOOT_CONTROL_REPLACE_DECISION_MARKER: &str =
    "RAIOS_BOOT_CONTROL_REPLACE_SCOPE_DECISION";

pub const EXPECTED_METHOD: &str = "boot.control_mark_success";
pub const EXPECTED_TARGET_ID: &str = "replace.boot_control.seed_data";
pub const EXPECTED_RECORD_SCHEMA: &str = BOOT_CONTROL_SCHEMA;
pub const EXPECTED_REGION_MARKER: &str = "RAIOS_DATA_BOOTCTL";
const SECTOR_SIZE: u64 = 512;

#[derive(Clone, Copy)]
pub struct ScopedBootControlReplaceInput<'a> {
    pub method: Option<&'a str>,
    pub target_id: Option<&'a str>,
    pub record_schema: Option<&'a str>,
    pub region_marker: Option<&'a str>,
    pub slot_len: Option<u64>,
    pub write_offset: Option<u64>,
    pub bootctl_byte_count: Option<u64>,
    pub absolute_start_lba: Option<u64>,
    pub bootctl_lba_count: Option<u64>,
    pub new_seq: Option<u64>,
    pub slot_a_seq: Option<u64>,
    pub slot_b_seq: Option<u64>,
    pub authoritative_storage_slot: Option<BootStorageSlot>,
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
}

impl<'a> ScopedBootControlReplaceInput<'a> {
    pub const fn empty() -> Self {
        Self {
            method: None,
            target_id: None,
            record_schema: None,
            region_marker: None,
            slot_len: None,
            write_offset: None,
            bootctl_byte_count: None,
            absolute_start_lba: None,
            bootctl_lba_count: None,
            new_seq: None,
            slot_a_seq: None,
            slot_b_seq: None,
            authoritative_storage_slot: None,
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
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopedBootControlReplaceDecision {
    pub performed: bool,
    pub status: &'static str,
    pub reason: &'static str,
}

pub fn evaluate_scoped_boot_control_replace(
    input: &ScopedBootControlReplaceInput<'_>,
) -> ScopedBootControlReplaceDecision {
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

    let slot_len = match input.slot_len {
        Some(value) if value == BOOTCTL_SLOT_SIZE as u64 && value % SECTOR_SIZE == 0 => value,
        Some(_) => return denied("slot_len_out_of_scope"),
        None => return denied("missing_slot_len"),
    };
    let write_offset = match input.write_offset {
        Some(value)
            if value % SECTOR_SIZE == 0 && (value == 0 || value == BOOTCTL_SLOT_SIZE as u64) =>
        {
            value
        }
        Some(_) => return denied("write_offset_not_slot_aligned"),
        None => return denied("missing_write_offset"),
    };
    let bootctl_byte_count = match input.bootctl_byte_count {
        Some(value) if value == BOOTCTL_REGION_BYTE_COUNT as u64 => value,
        Some(_) => return denied("bootctl_byte_count_out_of_scope"),
        None => return denied("missing_bootctl_byte_count"),
    };
    if write_offset
        .checked_add(slot_len)
        .map(|end| end > bootctl_byte_count)
        .unwrap_or(true)
    {
        return denied("bootctl_region_full");
    }

    let absolute_start_lba = match input.absolute_start_lba {
        Some(value) => value,
        None => return denied("missing_absolute_start_lba"),
    };
    let bootctl_lba_count = match input.bootctl_lba_count {
        Some(value) => value,
        None => return denied("missing_bootctl_lba_count"),
    };
    if !write_span_inside_bootctl(
        absolute_start_lba,
        bootctl_lba_count,
        write_offset,
        slot_len,
    ) {
        return denied("write_span_out_of_bootctl");
    }

    let new_seq = match input.new_seq {
        Some(value) => value,
        None => return denied("missing_new_seq"),
    };
    let slot_a_seq = match input.slot_a_seq {
        Some(value) => value,
        None => return denied("missing_slot_a_seq"),
    };
    let slot_b_seq = match input.slot_b_seq {
        Some(value) => value,
        None => return denied("missing_slot_b_seq"),
    };
    if new_seq <= slot_a_seq || new_seq <= slot_b_seq {
        return denied("bad_seq_not_strictly_greater");
    }

    let authoritative_storage_slot = match input.authoritative_storage_slot {
        Some(value) => value,
        None => return denied("missing_authoritative_storage_slot"),
    };
    if !targets_loser_slot(authoritative_storage_slot, write_offset) {
        return denied("target_not_loser_slot");
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

    ScopedBootControlReplaceDecision {
        performed: true,
        status: "replaced",
        reason: "authorized_boot_control_replace_write_readback_reparse_verified",
    }
}

fn require_str(
    actual: Option<&str>,
    expected: &str,
    missing: &'static str,
    mismatch: &'static str,
) -> Result<(), ScopedBootControlReplaceDecision> {
    match actual {
        Some(actual) if actual == expected => Ok(()),
        Some(_) => Err(denied(mismatch)),
        None => Err(denied(missing)),
    }
}

fn write_span_inside_bootctl(
    absolute_start_lba: u64,
    bootctl_lba_count: u64,
    write_offset: u64,
    slot_len: u64,
) -> bool {
    if slot_len / SECTOR_SIZE != BOOTCTL_STORAGE_SLOT_SECTORS {
        return false;
    }
    if write_offset != 0 && write_offset != BOOTCTL_SLOT_SIZE as u64 {
        return false;
    }
    let Some(bootctl_end_lba) = absolute_start_lba.checked_add(bootctl_lba_count) else {
        return false;
    };
    let sector_offset = write_offset / SECTOR_SIZE;
    let sector_count = slot_len / SECTOR_SIZE;
    let mut idx = 0u64;
    while idx < sector_count {
        let Some(lba) = absolute_start_lba
            .checked_add(sector_offset)
            .and_then(|base| base.checked_add(idx))
        else {
            return false;
        };
        if lba < absolute_start_lba || lba >= bootctl_end_lba {
            return false;
        }
        let slot_sector = if write_offset == 0 {
            idx
        } else {
            BOOTCTL_STORAGE_SLOT_SECTORS + idx
        };
        if sector_offset + idx != slot_sector {
            return false;
        }
        idx += 1;
    }
    true
}

fn targets_loser_slot(authoritative: BootStorageSlot, write_offset: u64) -> bool {
    match authoritative {
        BootStorageSlot::A => write_offset == BOOTCTL_SLOT_SIZE as u64,
        BootStorageSlot::B => write_offset == 0,
    }
}

fn denied(reason: &'static str) -> ScopedBootControlReplaceDecision {
    ScopedBootControlReplaceDecision {
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

    fn valid_input() -> ScopedBootControlReplaceInput<'static> {
        ScopedBootControlReplaceInput {
            method: Some(EXPECTED_METHOD),
            target_id: Some(EXPECTED_TARGET_ID),
            record_schema: Some(EXPECTED_RECORD_SCHEMA),
            region_marker: Some(EXPECTED_REGION_MARKER),
            slot_len: Some(BOOTCTL_SLOT_SIZE as u64),
            write_offset: Some(BOOTCTL_SLOT_SIZE as u64),
            bootctl_byte_count: Some(BOOTCTL_REGION_BYTE_COUNT as u64),
            absolute_start_lba: Some(100),
            bootctl_lba_count: Some(8),
            new_seq: Some(3),
            slot_a_seq: Some(1),
            slot_b_seq: Some(2),
            authoritative_storage_slot: Some(BootStorageSlot::A),
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
        }
    }

    #[test]
    fn performs_only_exact_boot_control_replace_evidence() {
        assert_eq!(
            evaluate_scoped_boot_control_replace(&valid_input()),
            ScopedBootControlReplaceDecision {
                performed: true,
                status: "replaced",
                reason: "authorized_boot_control_replace_write_readback_reparse_verified"
            }
        );
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
        MissingSlotLen,
        BadSlotLen,
        MissingWriteOffset,
        BadWriteOffset,
        MissingByteCount,
        BadByteCount,
        MissingStartLba,
        MissingLbaCount,
        SpanOut,
        MissingNewSeq,
        MissingSlotASeq,
        MissingSlotBSeq,
        BadSeq,
        MissingAuthoritativeSlot,
        TargetNotLoser,
        BadPayloadHash,
        FrameHashMismatch,
        WriteNotAttempted,
        WriteNotCompleted,
        ReadbackNotCompleted,
        ReadbackHashMismatch,
        ReparseNotValid,
        SpanFlagFalse,
    }

    fn apply(input: &mut ScopedBootControlReplaceInput<'static>, mutation: Mutation) {
        match mutation {
            Mutation::MissingMethod => input.method = None,
            Mutation::WrongMethod => input.method = Some("boot.control_read"),
            Mutation::MissingTarget => input.target_id = None,
            Mutation::WrongTarget => input.target_id = Some("append.record_log.seed_data"),
            Mutation::MissingSchema => input.record_schema = None,
            Mutation::WrongSchema => input.record_schema = Some("raios.durable_record.v0"),
            Mutation::MissingMarker => input.region_marker = None,
            Mutation::WrongMarker => input.region_marker = Some("RAIOS_DATA_RECLOG"),
            Mutation::MissingSlotLen => input.slot_len = None,
            Mutation::BadSlotLen => input.slot_len = Some(512),
            Mutation::MissingWriteOffset => input.write_offset = None,
            Mutation::BadWriteOffset => input.write_offset = Some(512),
            Mutation::MissingByteCount => input.bootctl_byte_count = None,
            Mutation::BadByteCount => input.bootctl_byte_count = Some(2048),
            Mutation::MissingStartLba => input.absolute_start_lba = None,
            Mutation::MissingLbaCount => input.bootctl_lba_count = None,
            Mutation::SpanOut => input.bootctl_lba_count = Some(4),
            Mutation::MissingNewSeq => input.new_seq = None,
            Mutation::MissingSlotASeq => input.slot_a_seq = None,
            Mutation::MissingSlotBSeq => input.slot_b_seq = None,
            Mutation::BadSeq => input.new_seq = Some(2),
            Mutation::MissingAuthoritativeSlot => input.authoritative_storage_slot = None,
            Mutation::TargetNotLoser => input.write_offset = Some(0),
            Mutation::BadPayloadHash => input.payload_sha256 = Some(h(9)),
            Mutation::FrameHashMismatch => input.readback_frame_sha256 = Some(h(9)),
            Mutation::WriteNotAttempted => input.write_attempted = false,
            Mutation::WriteNotCompleted => input.write_completed = false,
            Mutation::ReadbackNotCompleted => input.readback_completed = false,
            Mutation::ReadbackHashMismatch => input.readback_matches_planned = false,
            Mutation::ReparseNotValid => input.reparse_valid = false,
            Mutation::SpanFlagFalse => input.span_in_bounds = false,
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
            (Mutation::MissingSlotLen, "missing_slot_len"),
            (Mutation::BadSlotLen, "slot_len_out_of_scope"),
            (Mutation::MissingWriteOffset, "missing_write_offset"),
            (Mutation::BadWriteOffset, "write_offset_not_slot_aligned"),
            (Mutation::MissingByteCount, "missing_bootctl_byte_count"),
            (Mutation::BadByteCount, "bootctl_byte_count_out_of_scope"),
            (Mutation::MissingStartLba, "missing_absolute_start_lba"),
            (Mutation::MissingLbaCount, "missing_bootctl_lba_count"),
            (Mutation::SpanOut, "write_span_out_of_bootctl"),
            (Mutation::MissingNewSeq, "missing_new_seq"),
            (Mutation::MissingSlotASeq, "missing_slot_a_seq"),
            (Mutation::MissingSlotBSeq, "missing_slot_b_seq"),
            (Mutation::BadSeq, "bad_seq_not_strictly_greater"),
            (
                Mutation::MissingAuthoritativeSlot,
                "missing_authoritative_storage_slot",
            ),
            (Mutation::TargetNotLoser, "target_not_loser_slot"),
            (Mutation::BadPayloadHash, "bad_payload_sha256"),
            (Mutation::FrameHashMismatch, "frame_sha256_mismatch"),
            (Mutation::WriteNotAttempted, "write_not_attempted"),
            (Mutation::WriteNotCompleted, "write_not_completed"),
            (Mutation::ReadbackNotCompleted, "readback_not_completed"),
            (Mutation::ReadbackHashMismatch, "readback_hash_mismatch"),
            (Mutation::ReparseNotValid, "reparse_not_valid"),
            (Mutation::SpanFlagFalse, "span_not_in_bounds"),
        ];

        let mut idx = 0usize;
        while idx < cases.len() {
            let mut input = valid_input();
            apply(&mut input, cases[idx].0);
            let decision = evaluate_scoped_boot_control_replace(&input);
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
