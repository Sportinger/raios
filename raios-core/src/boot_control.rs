use alloc::{vec, vec::Vec};

use crate::{
    record::{Field, Value},
    seed_data_layout::{BOOTCTL_LBA_COUNT, BOOTCTL_START_LBA},
    sha256_bytes,
};

pub const BOOT_CONTROL_SCHEMA: &str = "raios.boot_control.v0";
pub const BOOT_CONTROL_READ_SCHEMA: &str = "raios.boot_control_read.v0";
pub const BOOT_CONTROL_READ_ID: &str = "boot_control_read.seed_data.current_boot.v0";

pub const BOOTCTL_MAGIC: &[u8; 8] = b"RAIOSBC0";
pub const BOOTCTL_PAYLOAD_MAGIC: &[u8; 8] = b"RAIOSBP0";
pub const BOOT_CONTROL_VERSION: u32 = 0;
pub const BOOTCTL_SLOT_SIZE: usize = 2048;
pub const BOOTCTL_SLOT_HEADER_LEN: usize = 52;
pub const BOOTCTL_SLOT_COUNT: usize = 2;
pub const BOOTCTL_SLOT_MAX_PAYLOAD_LEN: usize = BOOTCTL_SLOT_SIZE - BOOTCTL_SLOT_HEADER_LEN;
pub const BOOTCTL_REGION_BYTE_COUNT: usize = BOOTCTL_SLOT_SIZE * BOOTCTL_SLOT_COUNT;
pub const BOOTCTL_STORAGE_SLOT_SECTORS: u64 = 4;

pub const BOOTCTL_MAGIC_OFFSET: usize = 0;
pub const BOOTCTL_PAYLOAD_LEN_OFFSET: usize = 8;
pub const BOOTCTL_SEQ_OFFSET: usize = 12;
pub const BOOTCTL_PAYLOAD_SHA256_OFFSET: usize = 20;
pub const BOOTCTL_PAYLOAD_OFFSET: usize = 52;

pub const BOOTCTL_PAYLOAD_MAGIC_OFFSET: usize = 0;
pub const BOOTCTL_PAYLOAD_VERSION_OFFSET: usize = 8;
pub const BOOTCTL_ACTIVE_SLOT_OFFSET: usize = 12;
pub const BOOTCTL_LAST_GOOD_SLOT_OFFSET: usize = 13;
pub const BOOTCTL_PENDING_SLOT_OFFSET: usize = 14;
pub const BOOTCTL_SAFE_MODE_OFFSET: usize = 15;
pub const BOOTCTL_BOOT_ATTEMPT_SLOT_OFFSET: usize = 16;
pub const BOOTCTL_BOOT_ATTEMPT_SUCCESS_MARKED_OFFSET: usize = 17;
pub const BOOTCTL_BOOT_ATTEMPT_GENERATION_OFFSET: usize = 18;
pub const BOOTCTL_SLOT_A_GENERATION_OFFSET: usize = 26;
pub const BOOTCTL_SLOT_A_STATE_OFFSET: usize = 34;
pub const BOOTCTL_SLOT_A_FAILURE_COUNT_OFFSET: usize = 35;
pub const BOOTCTL_SLOT_B_GENERATION_OFFSET: usize = 39;
pub const BOOTCTL_SLOT_B_STATE_OFFSET: usize = 47;
pub const BOOTCTL_SLOT_B_FAILURE_COUNT_OFFSET: usize = 48;
pub const BOOTCTL_PAYLOAD_LEN: usize = 52;

pub const SLOT_ID_NONE: u8 = 0;
pub const SLOT_ID_A: u8 = 1;
pub const SLOT_ID_B: u8 = 2;

pub const SLOT_STATE_EMPTY: u8 = 0;
pub const SLOT_STATE_CANDIDATE: u8 = 1;
pub const SLOT_STATE_PENDING: u8 = 2;
pub const SLOT_STATE_GOOD: u8 = 3;
pub const SLOT_STATE_BAD: u8 = 4;

/// v0-provisional owner-overridable threshold from image-layout-v0.md.
pub const MAX_PENDING_BOOT_ATTEMPTS: u32 = 3;

const SHA256_LEN: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootStorageSlot {
    A,
    B,
}

impl BootStorageSlot {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootSlotId {
    None,
    A,
    B,
}

impl BootSlotId {
    pub const fn as_str(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::A => Some("A"),
            Self::B => Some("B"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootPayloadSlotState {
    Empty,
    Candidate,
    Pending,
    Good,
    Bad,
}

impl BootPayloadSlotState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Candidate => "candidate",
            Self::Pending => "pending",
            Self::Good => "good",
            Self::Bad => "bad",
        }
    }

    const fn is_bootable(self) -> bool {
        matches!(self, Self::Candidate | Self::Pending | Self::Good)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootPosture {
    Normal,
    Probation,
    Safe,
    PersistenceUnavailable,
}

impl BootPosture {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Probation => "Probation",
            Self::Safe => "Safe",
            Self::PersistenceUnavailable => "PersistenceUnavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootAttempt {
    pub slot: BootSlotId,
    pub generation: u64,
    pub success_marked: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootPayloadSlot {
    pub generation: u64,
    pub state: BootPayloadSlotState,
    pub failure_count: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootControlFields {
    pub active: BootSlotId,
    pub last_good: BootSlotId,
    pub pending: BootSlotId,
    pub safe_mode: bool,
    pub boot_attempt: BootAttempt,
    pub slot_a: BootPayloadSlot,
    pub slot_b: BootPayloadSlot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParsedBootControl {
    pub payload_len: u32,
    pub seq: u64,
    pub storage_sha256: [u8; 32],
    pub payload_sha256: [u8; 32],
    pub active: BootSlotId,
    pub last_good: BootSlotId,
    pub pending: BootSlotId,
    pub safe_mode: bool,
    pub boot_attempt: BootAttempt,
    pub slot_a: BootPayloadSlot,
    pub slot_b: BootPayloadSlot,
}

impl ParsedBootControl {
    const fn slot(self, slot: BootSlotId) -> Option<BootPayloadSlot> {
        match slot {
            BootSlotId::None => None,
            BootSlotId::A => Some(self.slot_a),
            BootSlotId::B => Some(self.slot_b),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootControlDecision {
    pub selected_storage_slot: Option<BootStorageSlot>,
    pub selected_payload_slot: BootSlotId,
    pub authoritative_bootctl_slot: Option<BootStorageSlot>,
    pub seq: Option<u64>,
    pub posture: BootPosture,
    pub pending_consumed: bool,
    pub would_mark_good: bool,
    pub failure_count: Option<u32>,
    pub reason: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootCorePolicyBinding {
    pub slot: BootSlotId,
    pub generation: u64,
}

impl BootControlDecision {
    pub const fn persistence_unavailable(reason: &'static str) -> Self {
        Self {
            selected_storage_slot: None,
            selected_payload_slot: BootSlotId::None,
            authoritative_bootctl_slot: None,
            seq: None,
            posture: BootPosture::PersistenceUnavailable,
            pending_consumed: false,
            would_mark_good: false,
            failure_count: None,
            reason,
        }
    }

    const fn safe_without_record(reason: &'static str) -> Self {
        Self {
            selected_storage_slot: None,
            selected_payload_slot: BootSlotId::None,
            authoritative_bootctl_slot: None,
            seq: None,
            posture: BootPosture::Safe,
            pending_consumed: false,
            would_mark_good: false,
            failure_count: None,
            reason,
        }
    }
}

/// Returns the exact Normal/Probation BOOTCTL payload slot and generation that
/// a signed policy for the currently executing core must bind. SAFE and
/// persistence-unavailable postures never authorize a core policy.
pub fn core_policy_boot_binding(
    decision: &BootControlDecision,
    authoritative_record: &ParsedBootControl,
) -> Result<BootCorePolicyBinding, &'static str> {
    if !matches!(
        decision.posture,
        BootPosture::Normal | BootPosture::Probation
    ) {
        return Err("boot_control_posture_not_authorizing");
    }
    let authoritative_slot = decision
        .authoritative_bootctl_slot
        .ok_or("boot_control_authoritative_slot_missing")?;
    if decision.selected_storage_slot != Some(authoritative_slot)
        || decision.seq != Some(authoritative_record.seq)
    {
        return Err("boot_control_authoritative_record_mismatch");
    }
    let selected = decision.selected_payload_slot;
    let payload_slot = authoritative_record
        .slot(selected)
        .ok_or("boot_control_policy_slot_missing")?;
    if !payload_slot.state.is_bootable() {
        return Err("boot_control_policy_slot_not_bootable");
    }
    if payload_slot.generation == 0 {
        return Err("boot_control_policy_generation_invalid");
    }
    Ok(BootCorePolicyBinding {
        slot: selected,
        generation: payload_slot.generation,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootSuccessMarkPlan {
    pub write_offset: u64,
    pub written_storage_slot: BootStorageSlot,
    pub new_seq: u64,
    pub marked_slot: BootSlotId,
    pub would_mark_good: bool,
    pub pending_consumed: bool,
    pub payload_sha256: [u8; 32],
    pub frame_sha256: [u8; 32],
    pub slot_bytes: [u8; BOOTCTL_SLOT_SIZE],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MarkDenied {
    reason: &'static str,
}

impl MarkDenied {
    pub const fn reason(self) -> &'static str {
        self.reason
    }
}

pub fn build_boot_control_slot(seq: u64, fields: &BootControlFields) -> [u8; BOOTCTL_SLOT_SIZE] {
    let payload = build_boot_control_payload(fields);
    let payload_hash = sha256_bytes(&payload);
    let mut out = [0u8; BOOTCTL_SLOT_SIZE];
    out[BOOTCTL_MAGIC_OFFSET..BOOTCTL_MAGIC_OFFSET + BOOTCTL_MAGIC.len()]
        .copy_from_slice(BOOTCTL_MAGIC);
    write_u32(&mut out, BOOTCTL_PAYLOAD_LEN_OFFSET, payload.len() as u32);
    write_u64(&mut out, BOOTCTL_SEQ_OFFSET, seq);
    out[BOOTCTL_PAYLOAD_SHA256_OFFSET..BOOTCTL_PAYLOAD_SHA256_OFFSET + SHA256_LEN]
        .copy_from_slice(&payload_hash);
    out[BOOTCTL_PAYLOAD_OFFSET..BOOTCTL_PAYLOAD_OFFSET + payload.len()].copy_from_slice(&payload);
    out
}

pub fn plan_boot_success_mark(
    current: &ParsedBootControl,
    authoritative_storage_slot: BootStorageSlot,
    decision: &BootControlDecision,
) -> Result<BootSuccessMarkPlan, MarkDenied> {
    if decision.authoritative_bootctl_slot != Some(authoritative_storage_slot) {
        return Err(mark_denied("authoritative_storage_slot_mismatch"));
    }
    if decision.seq != Some(current.seq) {
        return Err(mark_denied("authoritative_seq_mismatch"));
    }
    if current.safe_mode {
        return Err(mark_denied("safe_mode_requested"));
    }
    if !matches!(
        decision.posture,
        BootPosture::Normal | BootPosture::Probation
    ) {
        return Err(mark_denied("boot_posture_not_markable"));
    }

    let selected = decision.selected_payload_slot;
    let Some(selected_payload) = current.slot(selected) else {
        return Err(mark_denied("selected_payload_slot_missing"));
    };
    if !selected_payload.state.is_bootable() {
        return Err(mark_denied("selected_payload_slot_not_bootable"));
    }

    let mut fields = fields_from_parsed(current);
    let mut would_mark_good = false;
    let mut pending_consumed = false;

    match decision.posture {
        BootPosture::Probation => {
            if current.pending == BootSlotId::None || selected != current.pending {
                return Err(mark_denied("probation_pending_slot_mismatch"));
            }
            if current.boot_attempt.success_marked {
                return Err(mark_denied("already_marked"));
            }
            if selected_payload.state != BootPayloadSlotState::Pending {
                return Err(mark_denied("pending_slot_not_pending"));
            }
            if selected_payload.failure_count >= MAX_PENDING_BOOT_ATTEMPTS {
                return Err(mark_denied("pending_failure_count_exhausted"));
            }

            fields.active = selected;
            fields.last_good = selected;
            fields.pending = BootSlotId::None;
            fields.boot_attempt = BootAttempt {
                slot: selected,
                generation: selected_payload.generation,
                success_marked: true,
            };
            set_payload_slot(
                &mut fields,
                selected,
                BootPayloadSlot {
                    generation: selected_payload.generation,
                    state: BootPayloadSlotState::Good,
                    failure_count: 0,
                },
            );
            would_mark_good = true;
            pending_consumed = true;
        }
        BootPosture::Normal => {
            if current.pending != BootSlotId::None {
                let Some(pending_payload) = current.slot(current.pending) else {
                    return Err(mark_denied("pending_slot_missing"));
                };
                if pending_payload.failure_count < MAX_PENDING_BOOT_ATTEMPTS {
                    return Err(mark_denied("pending_not_exhausted"));
                }
                set_payload_slot(
                    &mut fields,
                    current.pending,
                    BootPayloadSlot {
                        generation: pending_payload.generation,
                        state: BootPayloadSlotState::Bad,
                        failure_count: pending_payload.failure_count,
                    },
                );
                fields.pending = BootSlotId::None;
                pending_consumed = true;
            } else if current.boot_attempt.slot == selected
                && current.boot_attempt.generation == selected_payload.generation
                && current.boot_attempt.success_marked
            {
                return Err(mark_denied("already_marked"));
            }

            fields.active = selected;
            fields.last_good = selected;
            fields.boot_attempt = BootAttempt {
                slot: selected,
                generation: selected_payload.generation,
                success_marked: true,
            };
            set_payload_slot(
                &mut fields,
                selected,
                BootPayloadSlot {
                    generation: selected_payload.generation,
                    state: BootPayloadSlotState::Good,
                    failure_count: 0,
                },
            );
        }
        BootPosture::Safe | BootPosture::PersistenceUnavailable => {
            return Err(mark_denied("boot_posture_not_markable"));
        }
    }

    let new_seq = current
        .seq
        .checked_add(1)
        .ok_or(mark_denied("seq_overflow"))?;
    let written_storage_slot = loser_slot(authoritative_storage_slot);
    let write_offset = storage_slot_write_offset(written_storage_slot);
    let slot_bytes = build_boot_control_slot(new_seq, &fields);
    let payload_sha256 = sha256_bytes(
        &slot_bytes[BOOTCTL_PAYLOAD_OFFSET..BOOTCTL_PAYLOAD_OFFSET + BOOTCTL_PAYLOAD_LEN],
    );
    let frame_sha256 = sha256_bytes(&slot_bytes);

    Ok(BootSuccessMarkPlan {
        write_offset,
        written_storage_slot,
        new_seq,
        marked_slot: selected,
        would_mark_good,
        pending_consumed,
        payload_sha256,
        frame_sha256,
        slot_bytes,
    })
}

pub fn parse_boot_slot(bytes: &[u8]) -> Result<ParsedBootControl, &'static str> {
    if bytes.len() != BOOTCTL_SLOT_SIZE {
        return Err("slot_size_mismatch");
    }
    if all_zero(bytes) {
        return Err("zero_filled_slot");
    }
    if &bytes[BOOTCTL_MAGIC_OFFSET..BOOTCTL_MAGIC_OFFSET + BOOTCTL_MAGIC.len()] != BOOTCTL_MAGIC {
        return Err("bad_magic");
    }

    let payload_len = read_u32(bytes, BOOTCTL_PAYLOAD_LEN_OFFSET) as usize;
    let seq = read_u64(bytes, BOOTCTL_SEQ_OFFSET);
    if payload_len > BOOTCTL_SLOT_MAX_PAYLOAD_LEN {
        return Err("payload_len_out_of_bounds");
    }
    if payload_len != BOOTCTL_PAYLOAD_LEN {
        return Err("payload_len_mismatch");
    }
    let payload_end = BOOTCTL_PAYLOAD_OFFSET
        .checked_add(payload_len)
        .ok_or("payload_len_out_of_bounds")?;
    if payload_end > BOOTCTL_SLOT_SIZE {
        return Err("payload_len_out_of_bounds");
    }

    let payload = &bytes[BOOTCTL_PAYLOAD_OFFSET..payload_end];
    let payload_sha256 = sha256_bytes(payload);
    if bytes[BOOTCTL_PAYLOAD_SHA256_OFFSET..BOOTCTL_PAYLOAD_SHA256_OFFSET + SHA256_LEN]
        != payload_sha256
    {
        return Err("bad_payload_sha256");
    }
    if !all_zero(&bytes[payload_end..]) {
        return Err("slot_padding_nonzero");
    }

    if &payload
        [BOOTCTL_PAYLOAD_MAGIC_OFFSET..BOOTCTL_PAYLOAD_MAGIC_OFFSET + BOOTCTL_PAYLOAD_MAGIC.len()]
        != BOOTCTL_PAYLOAD_MAGIC
    {
        return Err("bad_payload_magic");
    }
    if read_u32(payload, BOOTCTL_PAYLOAD_VERSION_OFFSET) != BOOT_CONTROL_VERSION {
        return Err("payload_version_mismatch");
    }

    let active = parse_slot_id(payload[BOOTCTL_ACTIVE_SLOT_OFFSET], "bad_active_slot")?;
    let last_good = parse_slot_id(payload[BOOTCTL_LAST_GOOD_SLOT_OFFSET], "bad_last_good_slot")?;
    let pending = parse_slot_id(payload[BOOTCTL_PENDING_SLOT_OFFSET], "bad_pending_slot")?;
    let safe_mode = parse_bool(payload[BOOTCTL_SAFE_MODE_OFFSET], "bad_safe_mode")?;
    let boot_attempt_slot = parse_slot_id(
        payload[BOOTCTL_BOOT_ATTEMPT_SLOT_OFFSET],
        "bad_boot_attempt_slot",
    )?;
    let success_marked = parse_bool(
        payload[BOOTCTL_BOOT_ATTEMPT_SUCCESS_MARKED_OFFSET],
        "bad_success_marked",
    )?;

    Ok(ParsedBootControl {
        payload_len: payload_len as u32,
        seq,
        storage_sha256: sha256_bytes(bytes),
        payload_sha256,
        active,
        last_good,
        pending,
        safe_mode,
        boot_attempt: BootAttempt {
            slot: boot_attempt_slot,
            generation: read_u64(payload, BOOTCTL_BOOT_ATTEMPT_GENERATION_OFFSET),
            success_marked,
        },
        slot_a: BootPayloadSlot {
            generation: read_u64(payload, BOOTCTL_SLOT_A_GENERATION_OFFSET),
            state: parse_slot_state(payload[BOOTCTL_SLOT_A_STATE_OFFSET], "bad_slot_a_state")?,
            failure_count: read_u32(payload, BOOTCTL_SLOT_A_FAILURE_COUNT_OFFSET),
        },
        slot_b: BootPayloadSlot {
            generation: read_u64(payload, BOOTCTL_SLOT_B_GENERATION_OFFSET),
            state: parse_slot_state(payload[BOOTCTL_SLOT_B_STATE_OFFSET], "bad_slot_b_state")?,
            failure_count: read_u32(payload, BOOTCTL_SLOT_B_FAILURE_COUNT_OFFSET),
        },
    })
}

pub fn evaluate_boot_control(
    storage_slot_a: Result<ParsedBootControl, &'static str>,
    storage_slot_b: Result<ParsedBootControl, &'static str>,
) -> BootControlDecision {
    match (storage_slot_a, storage_slot_b) {
        (Ok(a), Ok(b)) => {
            if a.seq == b.seq && a.storage_sha256 != b.storage_sha256 {
                return BootControlDecision::safe_without_record("ambiguous_boot_control_slots");
            }
            if a.seq >= b.seq {
                evaluate_chosen(BootStorageSlot::A, a)
            } else {
                evaluate_chosen(BootStorageSlot::B, b)
            }
        }
        (Ok(a), Err(_)) => evaluate_chosen(BootStorageSlot::A, a),
        (Err(_), Ok(b)) => evaluate_chosen(BootStorageSlot::B, b),
        (Err(a), Err(b)) => {
            if a == "zero_filled_slot" && b == "zero_filled_slot" {
                BootControlDecision::safe_without_record("no_valid_boot_control_slot")
            } else {
                BootControlDecision::safe_without_record("both_slots_invalid")
            }
        }
    }
}

pub fn boot_control_read_record(
    decision: &BootControlDecision,
    control: Option<&ParsedBootControl>,
) -> Value<'static> {
    Value::Object(boot_control_read_fields(decision, control))
}

pub fn boot_control_read_fields(
    decision: &BootControlDecision,
    control: Option<&ParsedBootControl>,
) -> Vec<Field<'static>> {
    let boot_attempt = match control {
        Some(control) => boot_attempt_value(control.boot_attempt),
        None => empty_boot_attempt_value(),
    };
    let slots = match control {
        Some(control) => slots_value(control.slot_a, control.slot_b),
        None => empty_slots_value(),
    };

    vec![
        Field::new("schema", Value::Str(BOOT_CONTROL_READ_SCHEMA)),
        Field::new("id", Value::Str(BOOT_CONTROL_READ_ID)),
        Field::new("scope", Value::Str("current_boot")),
        Field::new("classification", Value::Str("local_only")),
        Field::new("boot_control_schema", Value::Str(BOOT_CONTROL_SCHEMA)),
        Field::new(
            "active",
            control
                .map(|control| slot_id_value(control.active))
                .unwrap_or(Value::Null),
        ),
        Field::new(
            "last_good",
            control
                .map(|control| slot_id_value(control.last_good))
                .unwrap_or(Value::Null),
        ),
        Field::new(
            "pending",
            control
                .map(|control| slot_id_value(control.pending))
                .unwrap_or(Value::Null),
        ),
        Field::new(
            "safe_mode",
            control
                .map(|control| Value::Bool(control.safe_mode))
                .unwrap_or(Value::Null),
        ),
        Field::new("boot_attempt", boot_attempt),
        Field::new("slots", slots),
        Field::new("decision_reason", Value::Str(decision.reason)),
        Field::new(
            "selected_storage_slot",
            storage_slot_value(decision.selected_storage_slot),
        ),
        Field::new(
            "selected_payload_slot",
            slot_id_value(decision.selected_payload_slot),
        ),
        Field::new("posture", Value::Str(decision.posture.as_str())),
        Field::new(
            "authoritative_bootctl_slot",
            storage_slot_value(decision.authoritative_bootctl_slot),
        ),
        Field::new("seq", decision.seq.map(Value::U64).unwrap_or(Value::Null)),
        Field::new("pending_consumed", Value::Bool(decision.pending_consumed)),
        Field::new("would_mark_good", Value::Bool(decision.would_mark_good)),
        Field::new(
            "failure_count",
            decision
                .failure_count
                .map(|count| Value::U64(count as u64))
                .unwrap_or(Value::Null),
        ),
        Field::new(
            "max_pending_boot_attempts",
            Value::U64(MAX_PENDING_BOOT_ATTEMPTS as u64),
        ),
        Field::new("read_only", Value::Bool(true)),
        Field::new("write_attempted", Value::Bool(false)),
        Field::new("write_dma_ext_called", Value::Bool(false)),
        Field::new("writes_enabled", Value::Bool(false)),
        Field::new("persistence_claimed", Value::Bool(false)),
    ]
}

/// Read-only projection of the durable BOOTCTL last-good pointer for the recovery
/// snapshot (M8C-1). Surfaces the M7C slot / seq / boot-success-mark / safe-mode /
/// authoritative slot from an already-performed bootctl read; it renders through the
/// shared record model, mutates nothing, appends nothing, and claims no persistence
/// (`read_only:true`, `write_attempted:false`). When no authoritative record exists
/// (PersistenceUnavailable, SAFE-without-record, or no valid slot => `control` is
/// `None`) it fails honest: `available:false`, `boot_success_mark:"missing"`,
/// `last_good_slot:null`, real `reason` — never a fabricated slot / seq / hash. The
/// only hash rendered is the parsed slot's own `payload_sha256` (there is no durable
/// service-set hash at HEAD); the pointer `source` is `bootctl_slot_pointer`.
pub fn recovery_last_good_fields(
    decision: &BootControlDecision,
    control: Option<&ParsedBootControl>,
) -> Vec<Field<'static>> {
    let boot_success_mark = match control {
        Some(control) if control.boot_attempt.success_marked => "marked",
        Some(_) => "unmarked",
        None => "missing",
    };
    vec![
        Field::new("source", Value::Str("bootctl_slot_pointer")),
        Field::new("available", Value::Bool(control.is_some())),
        Field::new("reason", Value::Str(decision.reason)),
        Field::new("posture", Value::Str(decision.posture.as_str())),
        Field::new(
            "safe_mode",
            control
                .map(|control| Value::Bool(control.safe_mode))
                .unwrap_or(Value::Null),
        ),
        Field::new(
            "authoritative_bootctl_slot",
            storage_slot_value(decision.authoritative_bootctl_slot),
        ),
        Field::new("seq", decision.seq.map(Value::U64).unwrap_or(Value::Null)),
        Field::new(
            "active_slot",
            control
                .map(|control| slot_id_value(control.active))
                .unwrap_or(Value::Null),
        ),
        Field::new(
            "last_good_slot",
            control
                .map(|control| slot_id_value(control.last_good))
                .unwrap_or(Value::Null),
        ),
        Field::new(
            "pending_slot",
            control
                .map(|control| slot_id_value(control.pending))
                .unwrap_or(Value::Null),
        ),
        Field::new(
            "selected_payload_slot",
            slot_id_value(decision.selected_payload_slot),
        ),
        Field::new("boot_success_mark", Value::Str(boot_success_mark)),
        Field::new(
            "failure_count",
            decision
                .failure_count
                .map(|count| Value::U64(count as u64))
                .unwrap_or(Value::Null),
        ),
        Field::new(
            "max_pending_boot_attempts",
            Value::U64(MAX_PENDING_BOOT_ATTEMPTS as u64),
        ),
        Field::new(
            "boot_control_payload_sha256",
            control
                .map(|control| Value::Sha256(control.payload_sha256))
                .unwrap_or(Value::Null),
        ),
        Field::new("read_only", Value::Bool(true)),
        Field::new("write_attempted", Value::Bool(false)),
    ]
}

/// Read-only projection of what a boot-slot rollback-to-last-good WOULD show (M8C-1),
/// computed purely from `evaluate_boot_control` output. It mutates nothing and the
/// mutating pointer switch stays in the M7C boot-control authority: `mutates_nothing`
/// is always `true` and `mutating_rollback_available_via_lifeline` is always `false`
/// (`recovery.rollback` stays denied through the lifeline). When no durable record
/// exists (`control` is `None`) the preview is honestly unavailable with null target
/// slots and never fabricates a target.
pub fn recovery_rollback_preview_fields(
    decision: &BootControlDecision,
    control: Option<&ParsedBootControl>,
) -> Vec<Field<'static>> {
    let has_last_good_target = control
        .map(|control| control.last_good != BootSlotId::None)
        .unwrap_or(false);
    let target_payload = control.and_then(|control| control.slot(control.last_good));
    let target_bootable = target_payload.map(|slot| slot.state.is_bootable());
    // A rollback is only "available" when the last-good pointer exists AND that slot is
    // actually bootable — a non-bootable pointer is no usable rollback target. The raw
    // pointer stays honestly surfaced via rollback_target_slot + target_bootable.
    let rollback_available = has_last_good_target && target_bootable == Some(true);
    let pending_payload = control.and_then(|control| control.slot(control.pending));
    let pending_failure_count = pending_payload.map(|slot| slot.failure_count);
    let would_switch = rollback_available
        && control
            .map(|control| control.active != control.last_good)
            .unwrap_or(false);
    // Only true when there is genuinely a last-good slot to fall back to. With pending
    // exhausted and NO last-good, the boot logic degrades to SAFE, not a last-good fallback.
    let would_fall_back = control.map(|control| {
        has_last_good_target
            && control.pending != BootSlotId::None
            && pending_failure_count
                .map(|count| count >= MAX_PENDING_BOOT_ATTEMPTS)
                .unwrap_or(false)
    });
    vec![
        Field::new("kind", Value::Str("boot_slot_last_good_rollback_preview")),
        Field::new("available", Value::Bool(rollback_available)),
        Field::new("mutates_nothing", Value::Bool(true)),
        Field::new("would_switch", Value::Bool(would_switch)),
        Field::new(
            "current_slot",
            control
                .map(|control| slot_id_value(control.active))
                .unwrap_or(Value::Null),
        ),
        Field::new(
            "rollback_target_slot",
            control
                .map(|control| slot_id_value(control.last_good))
                .unwrap_or(Value::Null),
        ),
        Field::new(
            "target_bootable",
            target_bootable.map(Value::Bool).unwrap_or(Value::Null),
        ),
        Field::new(
            "pending_slot",
            control
                .map(|control| slot_id_value(control.pending))
                .unwrap_or(Value::Null),
        ),
        Field::new(
            "pending_failure_count",
            pending_failure_count
                .map(|count| Value::U64(count as u64))
                .unwrap_or(Value::Null),
        ),
        Field::new(
            "would_fall_back_to_last_good_on_next_boot",
            would_fall_back.map(Value::Bool).unwrap_or(Value::Null),
        ),
        Field::new(
            "mutating_rollback_available_via_lifeline",
            Value::Bool(false),
        ),
        Field::new("reason", Value::Str(decision.reason)),
    ]
}

fn evaluate_chosen(
    storage_slot: BootStorageSlot,
    control: ParsedBootControl,
) -> BootControlDecision {
    if control.safe_mode {
        return chosen(
            storage_slot,
            control,
            BootSlotId::None,
            BootPosture::Safe,
            "safe_mode_requested",
            pending_failure_count(control),
        );
    }
    if control.pending == BootSlotId::None {
        return select_payload_slot(
            storage_slot,
            control,
            control.active,
            BootPosture::Normal,
            "active_slot_selected",
            None,
            "active_slot_missing",
        );
    }

    let failure_count = control
        .slot(control.pending)
        .map(|slot| slot.failure_count)
        .unwrap_or(0);
    if failure_count >= MAX_PENDING_BOOT_ATTEMPTS {
        return select_payload_slot(
            storage_slot,
            control,
            control.last_good,
            BootPosture::Normal,
            "pending_failure_count_exhausted",
            Some(failure_count),
            "last_good_slot_missing",
        );
    }
    if !control.boot_attempt.success_marked {
        return select_payload_slot(
            storage_slot,
            control,
            control.pending,
            BootPosture::Probation,
            "pending_slot_selected_for_probation",
            Some(failure_count),
            "pending_slot_missing",
        );
    }
    select_payload_slot(
        storage_slot,
        control,
        control.active,
        BootPosture::Normal,
        "pending_success_already_marked_read_only",
        Some(failure_count),
        "active_slot_missing",
    )
}

fn select_payload_slot(
    storage_slot: BootStorageSlot,
    control: ParsedBootControl,
    payload_slot_id: BootSlotId,
    posture: BootPosture,
    reason: &'static str,
    failure_count: Option<u32>,
    missing_reason: &'static str,
) -> BootControlDecision {
    let Some(payload_slot) = control.slot(payload_slot_id) else {
        return chosen(
            storage_slot,
            control,
            BootSlotId::None,
            BootPosture::Safe,
            missing_reason,
            failure_count,
        );
    };
    if !payload_slot.state.is_bootable() {
        return chosen(
            storage_slot,
            control,
            BootSlotId::None,
            BootPosture::Safe,
            "selected_payload_slot_not_bootable",
            failure_count,
        );
    }
    chosen(
        storage_slot,
        control,
        payload_slot_id,
        posture,
        reason,
        failure_count,
    )
}

fn chosen(
    storage_slot: BootStorageSlot,
    control: ParsedBootControl,
    selected_payload_slot: BootSlotId,
    posture: BootPosture,
    reason: &'static str,
    failure_count: Option<u32>,
) -> BootControlDecision {
    BootControlDecision {
        selected_storage_slot: Some(storage_slot),
        selected_payload_slot,
        authoritative_bootctl_slot: Some(storage_slot),
        seq: Some(control.seq),
        posture,
        pending_consumed: false,
        would_mark_good: false,
        failure_count,
        reason,
    }
}

fn pending_failure_count(control: ParsedBootControl) -> Option<u32> {
    control.slot(control.pending).map(|slot| slot.failure_count)
}

fn build_boot_control_payload(fields: &BootControlFields) -> [u8; BOOTCTL_PAYLOAD_LEN] {
    let mut out = [0u8; BOOTCTL_PAYLOAD_LEN];
    out[BOOTCTL_PAYLOAD_MAGIC_OFFSET..BOOTCTL_PAYLOAD_MAGIC_OFFSET + BOOTCTL_PAYLOAD_MAGIC.len()]
        .copy_from_slice(BOOTCTL_PAYLOAD_MAGIC);
    write_u32(
        &mut out,
        BOOTCTL_PAYLOAD_VERSION_OFFSET,
        BOOT_CONTROL_VERSION,
    );
    out[BOOTCTL_ACTIVE_SLOT_OFFSET] = slot_id_byte(fields.active);
    out[BOOTCTL_LAST_GOOD_SLOT_OFFSET] = slot_id_byte(fields.last_good);
    out[BOOTCTL_PENDING_SLOT_OFFSET] = slot_id_byte(fields.pending);
    out[BOOTCTL_SAFE_MODE_OFFSET] = fields.safe_mode as u8;
    out[BOOTCTL_BOOT_ATTEMPT_SLOT_OFFSET] = slot_id_byte(fields.boot_attempt.slot);
    out[BOOTCTL_BOOT_ATTEMPT_SUCCESS_MARKED_OFFSET] = fields.boot_attempt.success_marked as u8;
    write_u64(
        &mut out,
        BOOTCTL_BOOT_ATTEMPT_GENERATION_OFFSET,
        fields.boot_attempt.generation,
    );
    write_u64(
        &mut out,
        BOOTCTL_SLOT_A_GENERATION_OFFSET,
        fields.slot_a.generation,
    );
    out[BOOTCTL_SLOT_A_STATE_OFFSET] = state_byte(fields.slot_a.state);
    write_u32(
        &mut out,
        BOOTCTL_SLOT_A_FAILURE_COUNT_OFFSET,
        fields.slot_a.failure_count,
    );
    write_u64(
        &mut out,
        BOOTCTL_SLOT_B_GENERATION_OFFSET,
        fields.slot_b.generation,
    );
    out[BOOTCTL_SLOT_B_STATE_OFFSET] = state_byte(fields.slot_b.state);
    write_u32(
        &mut out,
        BOOTCTL_SLOT_B_FAILURE_COUNT_OFFSET,
        fields.slot_b.failure_count,
    );
    out
}

fn fields_from_parsed(control: &ParsedBootControl) -> BootControlFields {
    BootControlFields {
        active: control.active,
        last_good: control.last_good,
        pending: control.pending,
        safe_mode: control.safe_mode,
        boot_attempt: control.boot_attempt,
        slot_a: control.slot_a,
        slot_b: control.slot_b,
    }
}

fn set_payload_slot(fields: &mut BootControlFields, slot: BootSlotId, payload: BootPayloadSlot) {
    match slot {
        BootSlotId::None => {}
        BootSlotId::A => fields.slot_a = payload,
        BootSlotId::B => fields.slot_b = payload,
    }
}

const fn loser_slot(slot: BootStorageSlot) -> BootStorageSlot {
    match slot {
        BootStorageSlot::A => BootStorageSlot::B,
        BootStorageSlot::B => BootStorageSlot::A,
    }
}

pub const fn storage_slot_write_offset(slot: BootStorageSlot) -> u64 {
    match slot {
        BootStorageSlot::A => 0,
        BootStorageSlot::B => BOOTCTL_SLOT_SIZE as u64,
    }
}

fn mark_denied(reason: &'static str) -> MarkDenied {
    MarkDenied { reason }
}

fn boot_attempt_value(attempt: BootAttempt) -> Value<'static> {
    Value::Object(vec![
        Field::new("slot", slot_id_value(attempt.slot)),
        Field::new("generation", Value::U64(attempt.generation)),
        Field::new("attempt_id", Value::Null),
        Field::new("started_utc", Value::Null),
        Field::new("success_marked", Value::Bool(attempt.success_marked)),
    ])
}

fn empty_boot_attempt_value() -> Value<'static> {
    Value::Object(vec![
        Field::new("slot", Value::Null),
        Field::new("generation", Value::Null),
        Field::new("attempt_id", Value::Null),
        Field::new("started_utc", Value::Null),
        Field::new("success_marked", Value::Null),
    ])
}

fn slots_value(slot_a: BootPayloadSlot, slot_b: BootPayloadSlot) -> Value<'static> {
    Value::Object(vec![
        Field::new("A", payload_slot_value(slot_a)),
        Field::new("B", payload_slot_value(slot_b)),
    ])
}

fn empty_slots_value() -> Value<'static> {
    Value::Object(vec![
        Field::new("A", empty_payload_slot_value()),
        Field::new("B", empty_payload_slot_value()),
    ])
}

fn payload_slot_value(slot: BootPayloadSlot) -> Value<'static> {
    Value::Object(vec![
        Field::new("generation", Value::U64(slot.generation)),
        Field::new("state", Value::Str(slot.state.as_str())),
        Field::new("failure_count", Value::U64(slot.failure_count as u64)),
        Field::new("last_success_utc", Value::Null),
    ])
}

fn empty_payload_slot_value() -> Value<'static> {
    Value::Object(vec![
        Field::new("generation", Value::Null),
        Field::new("state", Value::Null),
        Field::new("failure_count", Value::Null),
        Field::new("last_success_utc", Value::Null),
    ])
}

fn slot_id_value(slot: BootSlotId) -> Value<'static> {
    match slot.as_str() {
        Some(value) => Value::Str(value),
        None => Value::Null,
    }
}

fn storage_slot_value(slot: Option<BootStorageSlot>) -> Value<'static> {
    match slot {
        Some(slot) => Value::Str(slot.as_str()),
        None => Value::Null,
    }
}

fn parse_slot_id(value: u8, reason: &'static str) -> Result<BootSlotId, &'static str> {
    match value {
        SLOT_ID_NONE => Ok(BootSlotId::None),
        SLOT_ID_A => Ok(BootSlotId::A),
        SLOT_ID_B => Ok(BootSlotId::B),
        _ => Err(reason),
    }
}

fn parse_slot_state(value: u8, reason: &'static str) -> Result<BootPayloadSlotState, &'static str> {
    match value {
        SLOT_STATE_EMPTY => Ok(BootPayloadSlotState::Empty),
        SLOT_STATE_CANDIDATE => Ok(BootPayloadSlotState::Candidate),
        SLOT_STATE_PENDING => Ok(BootPayloadSlotState::Pending),
        SLOT_STATE_GOOD => Ok(BootPayloadSlotState::Good),
        SLOT_STATE_BAD => Ok(BootPayloadSlotState::Bad),
        _ => Err(reason),
    }
}

fn parse_bool(value: u8, reason: &'static str) -> Result<bool, &'static str> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(reason),
    }
}

fn slot_id_byte(slot: BootSlotId) -> u8 {
    match slot {
        BootSlotId::None => SLOT_ID_NONE,
        BootSlotId::A => SLOT_ID_A,
        BootSlotId::B => SLOT_ID_B,
    }
}

fn state_byte(state: BootPayloadSlotState) -> u8 {
    match state {
        BootPayloadSlotState::Empty => SLOT_STATE_EMPTY,
        BootPayloadSlotState::Candidate => SLOT_STATE_CANDIDATE,
        BootPayloadSlotState::Pending => SLOT_STATE_PENDING,
        BootPayloadSlotState::Good => SLOT_STATE_GOOD,
        BootPayloadSlotState::Bad => SLOT_STATE_BAD,
    }
}

fn all_zero(bytes: &[u8]) -> bool {
    bytes.iter().all(|byte| *byte == 0)
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    (bytes[offset] as u32)
        | ((bytes[offset + 1] as u32) << 8)
        | ((bytes[offset + 2] as u32) << 16)
        | ((bytes[offset + 3] as u32) << 24)
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    (read_u32(bytes, offset) as u64) | ((read_u32(bytes, offset + 4) as u64) << 32)
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset] = value as u8;
    bytes[offset + 1] = (value >> 8) as u8;
    bytes[offset + 2] = (value >> 16) as u8;
    bytes[offset + 3] = (value >> 24) as u8;
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
    write_u32(bytes, offset, value as u32);
    write_u32(bytes, offset + 4, (value >> 32) as u32);
}

pub const fn bootctl_start_lba() -> u64 {
    BOOTCTL_START_LBA
}

pub const fn bootctl_lba_count() -> u64 {
    BOOTCTL_LBA_COUNT
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::write_json;
    use std::vec::Vec;

    #[derive(Clone, Copy)]
    struct Fixture {
        active: BootSlotId,
        last_good: BootSlotId,
        pending: BootSlotId,
        safe_mode: bool,
        boot_attempt_slot: BootSlotId,
        boot_attempt_generation: u64,
        success_marked: bool,
        slot_a: BootPayloadSlot,
        slot_b: BootPayloadSlot,
    }

    impl Fixture {
        const fn normal_a() -> Self {
            Self {
                active: BootSlotId::A,
                last_good: BootSlotId::A,
                pending: BootSlotId::None,
                safe_mode: false,
                boot_attempt_slot: BootSlotId::A,
                boot_attempt_generation: 1,
                success_marked: true,
                slot_a: BootPayloadSlot {
                    generation: 1,
                    state: BootPayloadSlotState::Good,
                    failure_count: 0,
                },
                slot_b: BootPayloadSlot {
                    generation: 2,
                    state: BootPayloadSlotState::Empty,
                    failure_count: 0,
                },
            }
        }
    }

    fn render(value: &Value<'_>) -> Vec<u8> {
        let mut out = Vec::new();
        write_json(value, &mut out, 0);
        out
    }

    fn slot(seq: u64, fixture: Fixture) -> [u8; BOOTCTL_SLOT_SIZE] {
        build_boot_control_slot(seq, &fields(fixture))
    }

    fn fields(fixture: Fixture) -> BootControlFields {
        BootControlFields {
            active: fixture.active,
            last_good: fixture.last_good,
            pending: fixture.pending,
            safe_mode: fixture.safe_mode,
            boot_attempt: BootAttempt {
                slot: fixture.boot_attempt_slot,
                generation: fixture.boot_attempt_generation,
                success_marked: fixture.success_marked,
            },
            slot_a: fixture.slot_a,
            slot_b: fixture.slot_b,
        }
    }

    fn with_payload_byte(
        mut bytes: [u8; BOOTCTL_SLOT_SIZE],
        offset: usize,
        value: u8,
    ) -> [u8; BOOTCTL_SLOT_SIZE] {
        bytes[BOOTCTL_PAYLOAD_OFFSET + offset] = value;
        let payload = &bytes[BOOTCTL_PAYLOAD_OFFSET..BOOTCTL_PAYLOAD_OFFSET + BOOTCTL_PAYLOAD_LEN];
        let hash = sha256_bytes(payload);
        bytes[BOOTCTL_PAYLOAD_SHA256_OFFSET..BOOTCTL_PAYLOAD_SHA256_OFFSET + SHA256_LEN]
            .copy_from_slice(&hash);
        bytes
    }

    fn decision_from(
        storage_a: Result<ParsedBootControl, &'static str>,
        storage_b: Result<ParsedBootControl, &'static str>,
    ) -> BootControlDecision {
        evaluate_boot_control(storage_a, storage_b)
    }

    #[test]
    fn fresh_zero_filled_bootctl_enters_safe_uninitialized() {
        let zero = [0u8; BOOTCTL_SLOT_SIZE];
        let decision = decision_from(parse_boot_slot(&zero), parse_boot_slot(&zero));

        assert_eq!(decision.posture, BootPosture::Safe);
        assert_eq!(decision.reason, "no_valid_boot_control_slot");
        assert_eq!(decision.selected_storage_slot, None);
        assert_eq!(decision.selected_payload_slot, BootSlotId::None);
        assert!(!decision.pending_consumed);
    }

    #[test]
    fn valid_storage_slot_a_only_selects_normal_a() {
        let a = slot(1, Fixture::normal_a());
        let zero = [0u8; BOOTCTL_SLOT_SIZE];
        let decision = decision_from(parse_boot_slot(&a), parse_boot_slot(&zero));

        assert_eq!(decision.posture, BootPosture::Normal);
        assert_eq!(decision.selected_storage_slot, Some(BootStorageSlot::A));
        assert_eq!(
            decision.authoritative_bootctl_slot,
            Some(BootStorageSlot::A)
        );
        assert_eq!(decision.selected_payload_slot, BootSlotId::A);
        assert_eq!(decision.seq, Some(1));
        assert!(!decision.pending_consumed);
    }

    #[test]
    fn both_valid_slots_pick_highest_seq_not_storage_position() {
        let mut fixture_b = Fixture::normal_a();
        fixture_b.active = BootSlotId::B;
        fixture_b.last_good = BootSlotId::B;
        fixture_b.boot_attempt_slot = BootSlotId::B;
        fixture_b.slot_b.state = BootPayloadSlotState::Good;

        let a5 = slot(5, Fixture::normal_a());
        let b9 = slot(9, fixture_b);
        let decision = decision_from(parse_boot_slot(&a5), parse_boot_slot(&b9));
        assert_eq!(
            decision.authoritative_bootctl_slot,
            Some(BootStorageSlot::B)
        );
        assert_eq!(decision.selected_payload_slot, BootSlotId::B);
        assert_eq!(decision.seq, Some(9));

        let a9 = slot(9, Fixture::normal_a());
        let b5 = slot(5, fixture_b);
        let mirror = decision_from(parse_boot_slot(&a9), parse_boot_slot(&b5));
        assert_eq!(mirror.authoritative_bootctl_slot, Some(BootStorageSlot::A));
        assert_eq!(mirror.selected_payload_slot, BootSlotId::A);
        assert_eq!(mirror.seq, Some(9));
    }

    #[test]
    fn both_corrupted_slots_enter_safe() {
        let mut bad_magic = slot(1, Fixture::normal_a());
        bad_magic[0] = b'X';
        let mut bad_hash = slot(2, Fixture::normal_a());
        bad_hash[BOOTCTL_PAYLOAD_SHA256_OFFSET] ^= 1;

        let decision = decision_from(parse_boot_slot(&bad_magic), parse_boot_slot(&bad_hash));

        assert_eq!(decision.posture, BootPosture::Safe);
        assert_eq!(decision.reason, "both_slots_invalid");
        assert_eq!(decision.selected_payload_slot, BootSlotId::None);
    }

    #[test]
    fn safe_mode_keeps_pending_unconsumed() {
        let mut fixture = Fixture::normal_a();
        fixture.pending = BootSlotId::B;
        fixture.safe_mode = true;
        fixture.success_marked = false;
        fixture.slot_b.state = BootPayloadSlotState::Pending;
        let parsed = parse_boot_slot(&slot(1, fixture));
        let decision = decision_from(parsed, Err("zero_filled_slot"));

        assert_eq!(decision.posture, BootPosture::Safe);
        assert_eq!(decision.reason, "safe_mode_requested");
        assert_eq!(decision.selected_payload_slot, BootSlotId::None);
        assert_eq!(decision.failure_count, Some(0));
        assert!(!decision.pending_consumed);
    }

    #[test]
    fn failure_threshold_falls_back_to_last_good() {
        let mut fixture = Fixture::normal_a();
        fixture.pending = BootSlotId::B;
        fixture.success_marked = false;
        fixture.slot_b.state = BootPayloadSlotState::Pending;
        fixture.slot_b.failure_count = MAX_PENDING_BOOT_ATTEMPTS;
        let parsed = parse_boot_slot(&slot(1, fixture));
        let decision = decision_from(parsed, Err("zero_filled_slot"));

        assert_eq!(decision.posture, BootPosture::Normal);
        assert_eq!(decision.reason, "pending_failure_count_exhausted");
        assert_eq!(decision.selected_payload_slot, BootSlotId::A);
        assert_eq!(decision.failure_count, Some(MAX_PENDING_BOOT_ATTEMPTS));
        assert!(!decision.pending_consumed);
    }

    #[test]
    fn pending_below_threshold_enters_probation_read_only() {
        let mut fixture = Fixture::normal_a();
        fixture.pending = BootSlotId::B;
        fixture.success_marked = false;
        fixture.slot_b.state = BootPayloadSlotState::Pending;
        let parsed = parse_boot_slot(&slot(1, fixture));
        let decision = decision_from(parsed, Err("zero_filled_slot"));

        assert_eq!(decision.posture, BootPosture::Probation);
        assert_eq!(decision.reason, "pending_slot_selected_for_probation");
        assert_eq!(decision.selected_payload_slot, BootSlotId::B);
        assert!(!decision.pending_consumed);
        assert!(!decision.would_mark_good);
    }

    #[test]
    fn build_boot_control_slot_round_trips_fields_and_hashes() {
        let fixture = Fixture::normal_a();
        let bytes = build_boot_control_slot(42, &fields(fixture));
        let parsed = parse_boot_slot(&bytes).unwrap();

        assert_eq!(parsed.seq, 42);
        assert_eq!(parsed.active, fixture.active);
        assert_eq!(parsed.last_good, fixture.last_good);
        assert_eq!(parsed.pending, fixture.pending);
        assert_eq!(parsed.safe_mode, fixture.safe_mode);
        assert_eq!(parsed.boot_attempt.slot, fixture.boot_attempt_slot);
        assert_eq!(
            parsed.boot_attempt.generation,
            fixture.boot_attempt_generation
        );
        assert_eq!(parsed.boot_attempt.success_marked, fixture.success_marked);
        assert_eq!(parsed.slot_a, fixture.slot_a);
        assert_eq!(parsed.slot_b, fixture.slot_b);
        assert_eq!(parsed.storage_sha256, sha256_bytes(&bytes));
        assert_eq!(
            parsed.payload_sha256,
            sha256_bytes(
                &bytes[BOOTCTL_PAYLOAD_OFFSET..BOOTCTL_PAYLOAD_OFFSET + BOOTCTL_PAYLOAD_LEN]
            )
        );
    }

    #[test]
    fn plan_case_a_probation_marks_pending_good_and_advances_last_good() {
        let mut fixture = Fixture::normal_a();
        fixture.pending = BootSlotId::B;
        fixture.success_marked = false;
        fixture.slot_b.state = BootPayloadSlotState::Pending;
        let current = parse_boot_slot(&slot(1, fixture)).unwrap();
        let decision = evaluate_chosen(BootStorageSlot::A, current);

        let plan = plan_boot_success_mark(&current, BootStorageSlot::A, &decision).unwrap();
        let planned = parse_boot_slot(&plan.slot_bytes).unwrap();

        assert_eq!(plan.write_offset, BOOTCTL_SLOT_SIZE as u64);
        assert_eq!(plan.written_storage_slot, BootStorageSlot::B);
        assert_eq!(plan.new_seq, 2);
        assert_eq!(plan.marked_slot, BootSlotId::B);
        assert!(plan.would_mark_good);
        assert!(plan.pending_consumed);
        assert_eq!(planned.active, BootSlotId::B);
        assert_eq!(planned.last_good, BootSlotId::B);
        assert_eq!(planned.pending, BootSlotId::None);
        assert!(planned.boot_attempt.success_marked);
        assert_eq!(planned.slot_b.state, BootPayloadSlotState::Good);
        assert_eq!(planned.slot_b.failure_count, 0);
        assert_eq!(plan.payload_sha256, planned.payload_sha256);
        assert_eq!(plan.frame_sha256, planned.storage_sha256);

        let after = evaluate_boot_control(
            parse_boot_slot(&slot(1, fixture)),
            parse_boot_slot(&plan.slot_bytes),
        );
        assert_eq!(after.authoritative_bootctl_slot, Some(BootStorageSlot::B));
        assert_eq!(after.posture, BootPosture::Normal);
    }

    #[test]
    fn plan_case_b_normal_unmarked_marks_selected_without_double_advance() {
        let mut fixture = Fixture::normal_a();
        fixture.success_marked = false;
        let current = parse_boot_slot(&slot(7, fixture)).unwrap();
        let decision = evaluate_chosen(BootStorageSlot::A, current);

        let plan = plan_boot_success_mark(&current, BootStorageSlot::A, &decision).unwrap();
        let planned = parse_boot_slot(&plan.slot_bytes).unwrap();

        assert_eq!(plan.new_seq, 8);
        assert_eq!(plan.written_storage_slot, BootStorageSlot::B);
        assert!(!plan.would_mark_good);
        assert!(!plan.pending_consumed);
        assert_eq!(planned.active, BootSlotId::A);
        assert_eq!(planned.last_good, BootSlotId::A);
        assert_eq!(planned.pending, BootSlotId::None);
        assert!(planned.boot_attempt.success_marked);
    }

    #[test]
    fn plan_case_b_already_marked_is_idempotent_denial() {
        let current = parse_boot_slot(&slot(7, Fixture::normal_a())).unwrap();
        let decision = evaluate_chosen(BootStorageSlot::A, current);

        assert_eq!(
            plan_boot_success_mark(&current, BootStorageSlot::A, &decision)
                .unwrap_err()
                .reason(),
            "already_marked"
        );
    }

    #[test]
    fn plan_case_c_pending_exhausted_keeps_last_good() {
        let mut fixture = Fixture::normal_a();
        fixture.pending = BootSlotId::B;
        fixture.success_marked = false;
        fixture.slot_b.state = BootPayloadSlotState::Pending;
        fixture.slot_b.failure_count = MAX_PENDING_BOOT_ATTEMPTS;
        let current = parse_boot_slot(&slot(1, fixture)).unwrap();
        let decision = evaluate_chosen(BootStorageSlot::A, current);

        let plan = plan_boot_success_mark(&current, BootStorageSlot::A, &decision).unwrap();
        let planned = parse_boot_slot(&plan.slot_bytes).unwrap();

        assert_eq!(plan.new_seq, 2);
        assert_eq!(planned.last_good, BootSlotId::A);
        assert_eq!(planned.active, BootSlotId::A);
        assert_eq!(planned.pending, BootSlotId::None);
        assert_eq!(planned.slot_b.state, BootPayloadSlotState::Bad);
        assert!(!plan.would_mark_good);
        assert!(plan.pending_consumed);
    }

    #[test]
    fn plan_case_d_safe_refuses_to_mark() {
        let mut fixture = Fixture::normal_a();
        fixture.safe_mode = true;
        let current = parse_boot_slot(&slot(1, fixture)).unwrap();
        let decision = evaluate_chosen(BootStorageSlot::A, current);

        assert_eq!(
            plan_boot_success_mark(&current, BootStorageSlot::A, &decision)
                .unwrap_err()
                .reason(),
            "safe_mode_requested"
        );
    }

    #[test]
    fn plan_seq_is_strictly_greater_than_both_current_slots_and_targets_loser() {
        let fixture_a = Fixture::normal_a();
        let mut fixture_b = Fixture::normal_a();
        fixture_b.active = BootSlotId::B;
        fixture_b.last_good = BootSlotId::B;
        fixture_b.boot_attempt_slot = BootSlotId::B;
        fixture_b.success_marked = false;
        fixture_b.slot_b.state = BootPayloadSlotState::Good;
        let a = parse_boot_slot(&slot(5, fixture_a));
        let b = parse_boot_slot(&slot(9, fixture_b));
        let decision = evaluate_boot_control(a, b);
        let current = b.unwrap();

        let plan = plan_boot_success_mark(&current, BootStorageSlot::B, &decision).unwrap();

        assert_eq!(plan.new_seq, 10);
        assert!(plan.new_seq > 5);
        assert!(plan.new_seq > 9);
        assert_eq!(plan.written_storage_slot, BootStorageSlot::A);
        assert_eq!(plan.write_offset, 0);
    }

    #[test]
    fn equal_seq_different_content_is_ambiguous_safe() {
        let mut fixture_b = Fixture::normal_a();
        fixture_b.active = BootSlotId::B;
        fixture_b.last_good = BootSlotId::B;
        fixture_b.boot_attempt_slot = BootSlotId::B;
        fixture_b.slot_b.state = BootPayloadSlotState::Good;

        let decision = decision_from(
            parse_boot_slot(&slot(7, Fixture::normal_a())),
            parse_boot_slot(&slot(7, fixture_b)),
        );

        assert_eq!(decision.posture, BootPosture::Safe);
        assert_eq!(decision.reason, "ambiguous_boot_control_slots");
        assert_eq!(decision.authoritative_bootctl_slot, None);
    }

    #[test]
    fn slot_envelope_truth_table_names_first_failed_pin() {
        let valid = slot(1, Fixture::normal_a());
        let mut bad_payload_hash = valid;
        bad_payload_hash[BOOTCTL_PAYLOAD_SHA256_OFFSET] ^= 1;
        let mut padding = valid;
        padding[BOOTCTL_PAYLOAD_OFFSET + BOOTCTL_PAYLOAD_LEN] = 1;
        let mut payload_len_too_large = valid;
        write_u32(
            &mut payload_len_too_large,
            BOOTCTL_PAYLOAD_LEN_OFFSET,
            (BOOTCTL_SLOT_MAX_PAYLOAD_LEN + 1) as u32,
        );
        let mut payload_len_mismatch = valid;
        write_u32(
            &mut payload_len_mismatch,
            BOOTCTL_PAYLOAD_LEN_OFFSET,
            (BOOTCTL_PAYLOAD_LEN + 1) as u32,
        );

        let cases: [(&[u8], &str); 16] = [
            (&[0u8; BOOTCTL_SLOT_SIZE - 1], "slot_size_mismatch"),
            (&[0u8; BOOTCTL_SLOT_SIZE], "zero_filled_slot"),
            (
                &{
                    let mut bytes = valid;
                    bytes[0] = b'X';
                    bytes
                },
                "bad_magic",
            ),
            (&payload_len_too_large, "payload_len_out_of_bounds"),
            (&payload_len_mismatch, "payload_len_mismatch"),
            (&bad_payload_hash, "bad_payload_sha256"),
            (&padding, "slot_padding_nonzero"),
            (
                &with_payload_byte(valid, BOOTCTL_PAYLOAD_MAGIC_OFFSET, b'X'),
                "bad_payload_magic",
            ),
            (
                &with_payload_byte(valid, BOOTCTL_PAYLOAD_VERSION_OFFSET, 1),
                "payload_version_mismatch",
            ),
            (
                &with_payload_byte(valid, BOOTCTL_ACTIVE_SLOT_OFFSET, 9),
                "bad_active_slot",
            ),
            (
                &with_payload_byte(valid, BOOTCTL_LAST_GOOD_SLOT_OFFSET, 9),
                "bad_last_good_slot",
            ),
            (
                &with_payload_byte(valid, BOOTCTL_PENDING_SLOT_OFFSET, 9),
                "bad_pending_slot",
            ),
            (
                &with_payload_byte(valid, BOOTCTL_SAFE_MODE_OFFSET, 9),
                "bad_safe_mode",
            ),
            (
                &with_payload_byte(valid, BOOTCTL_BOOT_ATTEMPT_SLOT_OFFSET, 9),
                "bad_boot_attempt_slot",
            ),
            (
                &with_payload_byte(valid, BOOTCTL_BOOT_ATTEMPT_SUCCESS_MARKED_OFFSET, 9),
                "bad_success_marked",
            ),
            (
                &with_payload_byte(valid, BOOTCTL_SLOT_A_STATE_OFFSET, 9),
                "bad_slot_a_state",
            ),
        ];

        let mut idx = 0usize;
        while idx < cases.len() {
            assert_eq!(parse_boot_slot(cases[idx].0).unwrap_err(), cases[idx].1);
            let mut other = 0usize;
            while other < idx {
                assert_ne!(cases[idx].1, cases[other].1);
                other += 1;
            }
            idx += 1;
        }

        assert_eq!(
            parse_boot_slot(&with_payload_byte(valid, BOOTCTL_SLOT_B_STATE_OFFSET, 9)).unwrap_err(),
            "bad_slot_b_state"
        );
    }

    #[test]
    fn record_renders_exact_read_only_shape() {
        let parsed = parse_boot_slot(&slot(1, Fixture::normal_a())).unwrap();
        let decision = evaluate_chosen(BootStorageSlot::A, parsed);

        assert_eq!(
            render(&boot_control_read_record(&decision, Some(&parsed))),
            b"{\r\n  \"schema\": \"raios.boot_control_read.v0\",\r\n  \"id\": \"boot_control_read.seed_data.current_boot.v0\",\r\n  \"scope\": \"current_boot\",\r\n  \"classification\": \"local_only\",\r\n  \"boot_control_schema\": \"raios.boot_control.v0\",\r\n  \"active\": \"A\",\r\n  \"last_good\": \"A\",\r\n  \"pending\": null,\r\n  \"safe_mode\": false,\r\n  \"boot_attempt\": {\r\n    \"slot\": \"A\",\r\n    \"generation\": 1,\r\n    \"attempt_id\": null,\r\n    \"started_utc\": null,\r\n    \"success_marked\": true\r\n  },\r\n  \"slots\": {\r\n    \"A\": {\r\n      \"generation\": 1,\r\n      \"state\": \"good\",\r\n      \"failure_count\": 0,\r\n      \"last_success_utc\": null\r\n    },\r\n    \"B\": {\r\n      \"generation\": 2,\r\n      \"state\": \"empty\",\r\n      \"failure_count\": 0,\r\n      \"last_success_utc\": null\r\n    }\r\n  },\r\n  \"decision_reason\": \"active_slot_selected\",\r\n  \"selected_storage_slot\": \"A\",\r\n  \"selected_payload_slot\": \"A\",\r\n  \"posture\": \"Normal\",\r\n  \"authoritative_bootctl_slot\": \"A\",\r\n  \"seq\": 1,\r\n  \"pending_consumed\": false,\r\n  \"would_mark_good\": false,\r\n  \"failure_count\": null,\r\n  \"max_pending_boot_attempts\": 3,\r\n  \"read_only\": true,\r\n  \"write_attempted\": false,\r\n  \"write_dma_ext_called\": false,\r\n  \"writes_enabled\": false,\r\n  \"persistence_claimed\": false\r\n}"
        );
    }

    #[test]
    fn recovery_last_good_present_normal_a_surfaces_durable_pointer() {
        let parsed = parse_boot_slot(&slot(1, Fixture::normal_a())).unwrap();
        let decision = evaluate_chosen(BootStorageSlot::A, parsed);

        let last_good = render(&Value::Object(recovery_last_good_fields(
            &decision,
            Some(&parsed),
        )));
        let last_good = std::str::from_utf8(&last_good).unwrap();
        assert!(last_good.contains("\"source\": \"bootctl_slot_pointer\""));
        assert!(last_good.contains("\"available\": true"));
        assert!(last_good.contains("\"reason\": \"active_slot_selected\""));
        assert!(last_good.contains("\"posture\": \"Normal\""));
        assert!(last_good.contains("\"safe_mode\": false"));
        assert!(last_good.contains("\"authoritative_bootctl_slot\": \"A\""));
        assert!(last_good.contains("\"seq\": 1"));
        assert!(last_good.contains("\"active_slot\": \"A\""));
        assert!(last_good.contains("\"last_good_slot\": \"A\""));
        assert!(last_good.contains("\"pending_slot\": null"));
        assert!(last_good.contains("\"selected_payload_slot\": \"A\""));
        assert!(last_good.contains("\"boot_success_mark\": \"marked\""));
        assert!(!last_good.contains("\"boot_success_mark\": \"missing\""));
        assert!(last_good.contains("\"max_pending_boot_attempts\": 3"));
        assert!(last_good.contains("\"boot_control_payload_sha256\": \"sha256:"));
        assert!(last_good.contains("\"read_only\": true"));
        assert!(last_good.contains("\"write_attempted\": false"));

        let rollback = render(&Value::Object(recovery_rollback_preview_fields(
            &decision,
            Some(&parsed),
        )));
        let rollback = std::str::from_utf8(&rollback).unwrap();
        assert!(rollback.contains("\"kind\": \"boot_slot_last_good_rollback_preview\""));
        assert!(rollback.contains("\"available\": true"));
        assert!(rollback.contains("\"mutates_nothing\": true"));
        // active A == last_good A: rolling back to last-good switches nothing.
        assert!(rollback.contains("\"would_switch\": false"));
        assert!(rollback.contains("\"current_slot\": \"A\""));
        assert!(rollback.contains("\"rollback_target_slot\": \"A\""));
        assert!(rollback.contains("\"target_bootable\": true"));
        assert!(rollback.contains("\"pending_slot\": null"));
        assert!(rollback.contains("\"pending_failure_count\": null"));
        assert!(rollback.contains("\"would_fall_back_to_last_good_on_next_boot\": false"));
        assert!(rollback.contains("\"mutating_rollback_available_via_lifeline\": false"));
        assert!(rollback.contains("\"reason\": \"active_slot_selected\""));
    }

    #[test]
    fn recovery_last_good_absent_persistence_unavailable_is_honest_missing() {
        let decision = BootControlDecision::persistence_unavailable("ahci_controller_not_observed");

        let last_good = render(&Value::Object(recovery_last_good_fields(&decision, None)));
        let last_good = std::str::from_utf8(&last_good).unwrap();
        assert!(last_good.contains("\"source\": \"bootctl_slot_pointer\""));
        assert!(last_good.contains("\"available\": false"));
        assert!(last_good.contains("\"reason\": \"ahci_controller_not_observed\""));
        assert!(last_good.contains("\"posture\": \"PersistenceUnavailable\""));
        assert!(last_good.contains("\"safe_mode\": null"));
        assert!(last_good.contains("\"authoritative_bootctl_slot\": null"));
        assert!(last_good.contains("\"seq\": null"));
        assert!(last_good.contains("\"active_slot\": null"));
        assert!(last_good.contains("\"last_good_slot\": null"));
        assert!(last_good.contains("\"selected_payload_slot\": null"));
        assert!(last_good.contains("\"boot_success_mark\": \"missing\""));
        assert!(last_good.contains("\"boot_control_payload_sha256\": null"));
        assert!(last_good.contains("\"read_only\": true"));
        assert!(last_good.contains("\"write_attempted\": false"));

        let rollback = render(&Value::Object(recovery_rollback_preview_fields(
            &decision, None,
        )));
        let rollback = std::str::from_utf8(&rollback).unwrap();
        assert!(rollback.contains("\"kind\": \"boot_slot_last_good_rollback_preview\""));
        assert!(rollback.contains("\"available\": false"));
        assert!(rollback.contains("\"mutates_nothing\": true"));
        assert!(rollback.contains("\"would_switch\": false"));
        assert!(rollback.contains("\"current_slot\": null"));
        assert!(rollback.contains("\"rollback_target_slot\": null"));
        assert!(rollback.contains("\"target_bootable\": null"));
        assert!(rollback.contains("\"pending_failure_count\": null"));
        assert!(rollback.contains("\"would_fall_back_to_last_good_on_next_boot\": null"));
        assert!(rollback.contains("\"mutating_rollback_available_via_lifeline\": false"));
        assert!(rollback.contains("\"reason\": \"ahci_controller_not_observed\""));
    }
}
