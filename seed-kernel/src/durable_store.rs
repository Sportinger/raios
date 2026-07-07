use alloc::{vec, vec::Vec};
use core::str;

use spin::Mutex;

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_sha_or_null, record_str as s,
    },
    ahci, event_log, pci,
};
use raios_core::{
    boot_control::BootPosture,
    durable_record_frame::{
        durable_record_log_scan_fields, parse_reclog_frame, plan_reclog_append, scan_reclog,
        PlannedAppend, RecordLogScan, DURABLE_RECORD_LOG_SCAN_SCHEMA, RECLOG_FRAME_HEADER_LEN,
        RECLOG_SECTOR_SIZE,
    },
    memory_record::MemoryRecord,
    promotion_attestation::{
        PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256, PROMOTION_AUTHORITY_IS_PLACEHOLDER,
    },
    record::{write_json, Field, Value as V},
    scoped_memory_record_append::{
        evaluate_scoped_memory_record_append, ScopedMemoryRecordAppendInput,
        EXPECTED_METHOD as MEMORY_EXPECTED_METHOD,
        EXPECTED_RECORD_SCHEMA as MEMORY_EXPECTED_RECORD_SCHEMA,
        EXPECTED_REGION_MARKER as MEMORY_EXPECTED_REGION_MARKER,
        EXPECTED_TARGET_ID as MEMORY_EXPECTED_TARGET_ID,
        EXPECTED_TRUST_TIER as MEMORY_EXPECTED_TRUST_TIER,
    },
    scoped_promotion_transaction_append::{
        evaluate_scoped_promotion_transaction_append, ScopedPromotionTransactionAppendInput,
        EXPECTED_METHOD as PROMOTION_EXPECTED_METHOD,
        EXPECTED_RECORD_SCHEMA as PROMOTION_EXPECTED_RECORD_SCHEMA,
        EXPECTED_REGION_MARKER as PROMOTION_EXPECTED_REGION_MARKER,
        EXPECTED_TARGET_ID as PROMOTION_EXPECTED_TARGET_ID,
        EXPECTED_TRUST_TIER as PROMOTION_EXPECTED_TRUST_TIER,
    },
    scoped_recovery_action_append::{
        evaluate_scoped_recovery_action_append, ScopedRecoveryActionAppendInput,
        EXPECTED_ACTION_KIND as RECOVERY_ACTION_EXPECTED_ACTION_KIND,
        EXPECTED_ACTION_KIND_RESTART_LAST_GOOD as RECOVERY_ACTION_EXPECTED_ACTION_KIND_RESTART_LAST_GOOD,
        EXPECTED_METHOD as RECOVERY_ACTION_EXPECTED_METHOD,
        EXPECTED_RECORD_SCHEMA as RECOVERY_ACTION_EXPECTED_RECORD_SCHEMA,
        EXPECTED_REGION_MARKER as RECOVERY_ACTION_EXPECTED_REGION_MARKER,
        EXPECTED_TARGET_ID as RECOVERY_ACTION_EXPECTED_TARGET_ID,
        EXPECTED_TRUST_TIER as RECOVERY_ACTION_EXPECTED_TRUST_TIER,
    },
    scoped_recovery_load_append::{
        evaluate_scoped_recovery_load_append, ScopedRecoveryLoadAppendInput,
        DECISION_STATUS_REINSTATED as RECOVERY_LOAD_DECISION_STATUS_REINSTATED,
        EXPECTED_METHOD as RECOVERY_LOAD_EXPECTED_METHOD,
        EXPECTED_RECORD_SCHEMA as RECOVERY_LOAD_EXPECTED_RECORD_SCHEMA,
        EXPECTED_REGION_MARKER as RECOVERY_LOAD_EXPECTED_REGION_MARKER,
        EXPECTED_TARGET_ID as RECOVERY_LOAD_EXPECTED_TARGET_ID,
        EXPECTED_TRUST_TIER as RECOVERY_LOAD_EXPECTED_TRUST_TIER,
    },
    scoped_seed_data_append::{
        evaluate_scoped_seed_data_append, ScopedSeedDataAppendInput, EXPECTED_METHOD,
        EXPECTED_RECORD_SCHEMA, EXPECTED_REGION_MARKER, EXPECTED_TARGET_ID,
    },
    sha256_bytes, ByteSink,
};

const METHOD: &str = "durable.record_log_scan";
const APPEND_METHOD: &str = "durable.record_log_append";
const APPEND_SCHEMA: &str = "raios.durable_record_log_append.v0";
const APPEND_ID: &str = "durable_record_log_append.seed_data.current_boot.v0";
const PROMOTION_APPEND_METHOD: &str = "durable.promotion_transaction_append";
const PROMOTION_APPEND_SCHEMA: &str = "raios.promotion_transaction_append.v0";
const PROMOTION_APPEND_ID: &str = "promotion_transaction_append.seed_data.origin_boot.v0";
const PROMOTION_TRANSACTION_SELFTEST_METHOD: &str = "module.promotion_transaction_selftest";
const PROMOTION_TRANSACTION_SELFTEST_SCHEMA: &str =
    "raios.promotion_transaction_append_selftest.v0";
const PROMOTION_TRANSACTION_RECORD_KIND: &str = "promotion_transaction";
const PROMOTION_TRANSACTION_RECORD_SCOPE: &str = "origin_boot";
const PROMOTION_TRANSACTION_TRUST_TIER: &str = "dev_key_not_owner_sealed";
const PROMOTION_TRANSACTION_MAX_PAYLOAD_LEN: usize = 4096 - RECLOG_FRAME_HEADER_LEN;
const RECOVERY_ACTION_SELFTEST_METHOD: &str = "recovery.disable_module_selftest";
const RECOVERY_ACTION_SELFTEST_SCHEMA: &str = "raios.recovery_action_append_selftest.v0";
const RECOVERY_ACTION_RECORD_KIND: &str = "recovery_action";
const RECOVERY_ACTION_RECORD_SCOPE: &str = "current_boot";
const RECOVERY_ACTION_TRUST_TIER: &str = "dev_key_not_owner_sealed";
const RECOVERY_ACTION_MAX_PAYLOAD_LEN: usize = 4096 - RECLOG_FRAME_HEADER_LEN;
pub(crate) const RECOVERY_ACTION_KIND_DISABLE_MODULE: &str = RECOVERY_ACTION_EXPECTED_ACTION_KIND;
pub(crate) const RECOVERY_ACTION_KIND_RESTART_LAST_GOOD: &str =
    RECOVERY_ACTION_EXPECTED_ACTION_KIND_RESTART_LAST_GOOD;
pub(crate) const RECOVERY_ACTION_SCHEMA: &str = RECOVERY_ACTION_EXPECTED_RECORD_SCHEMA;
pub(crate) const RECOVERY_ACTION_APPEND_TARGET_ID: &str = RECOVERY_ACTION_EXPECTED_TARGET_ID;
pub(crate) const RECOVERY_ACTION_APPEND_REGION_MARKER: &str =
    RECOVERY_ACTION_EXPECTED_REGION_MARKER;
const RECOVERY_LOAD_RECORD_KIND: &str = "recovery_load";
const RECOVERY_LOAD_RECORD_SCOPE: &str = "current_boot";
const RECOVERY_LOAD_RECORD_CLASSIFICATION: &str = "local_only";
const RECOVERY_LOAD_TRUST_TIER: &str = "dev_key_not_owner_sealed";
const RECOVERY_LOAD_SERVICE_ID: &str = "svc.dev.granted_candidate";
pub(crate) const RECOVERY_LOAD_SCHEMA: &str = RECOVERY_LOAD_EXPECTED_RECORD_SCHEMA;
pub(crate) const RECOVERY_LOAD_APPEND_TARGET_ID: &str = RECOVERY_LOAD_EXPECTED_TARGET_ID;
pub(crate) const RECOVERY_LOAD_APPEND_REGION_MARKER: &str = RECOVERY_LOAD_EXPECTED_REGION_MARKER;

struct DurableRecordLogScanEvidence {
    reason: &'static str,
    controller: Option<pci::PciMassStorageController>,
    controller_present: bool,
    source_port_index: Option<u8>,
    layout_status: &'static str,
    data_layout_status: &'static str,
    read_attempted: bool,
    read_completed: bool,
    region_bounds_valid: bool,
    reclog_absolute_start_lba: u64,
    reclog_lba_count: u64,
    reclog_byte_count: u64,
    scan: RecordLogScan,
}

impl DurableRecordLogScanEvidence {
    fn absent(reason: &'static str) -> Self {
        Self {
            reason,
            controller: None,
            controller_present: false,
            source_port_index: None,
            layout_status: "absent",
            data_layout_status: "absent",
            read_attempted: false,
            read_completed: false,
            region_bounds_valid: false,
            reclog_absolute_start_lba: 0,
            reclog_lba_count: 0,
            reclog_byte_count: 0,
            scan: RecordLogScan::not_scanned(reason),
        }
    }
}

pub(crate) fn emit_durable_record_log_scan() {
    let evidence = current_boot_reclog_scan();
    let mut fields = durable_record_log_scan_fields(&evidence.scan);
    fields.push(f("query_method", s(METHOD)));
    fields.push(f("io_reason", s(evidence.reason)));
    fields.push(f("controller_present", b(evidence.controller_present)));
    fields.push(f(
        "source_port_index",
        match evidence.source_port_index {
            Some(port) => V::U64(port as u64),
            None => V::Null,
        },
    ));
    fields.push(f("layout_status", s(evidence.layout_status)));
    fields.push(f("data_layout_status", s(evidence.data_layout_status)));
    fields.push(f("read_attempted", b(evidence.read_attempted)));
    fields.push(f("read_completed", b(evidence.read_completed)));
    fields.push(f("region_bounds_valid", b(evidence.region_bounds_valid)));
    fields.push(f(
        "reclog_absolute_start_lba",
        V::U64(evidence.reclog_absolute_start_lba),
    ));
    fields.push(f("reclog_lba_count", V::U64(evidence.reclog_lba_count)));
    fields.push(f("reclog_byte_count", V::U64(evidence.reclog_byte_count)));
    fields.push(f("authority", s("evidence_only")));
    fields.push(f("durable_append", s("capability_denied")));
    fields.push(f("record_model_entry", s(DURABLE_RECORD_LOG_SCAN_SCHEMA)));

    begin_response(METHOD);
    emit_record_fields(fields, 6);
    end_response(METHOD);
}

pub(crate) fn emit_durable_record_log_append() {
    let evidence = current_boot_reclog_scan();
    // M7C-2a: boot-control SAFE posture disables the durable append (more-restrictive
    // precondition, evaluated before any plan/write). Normal|Probation preserve prior
    // behavior; Safe|PersistenceUnavailable now deny. Probation MUST stay allowed so a
    // boot-success audit append can escape probation (M7C-2b).
    if !matches!(
        super::boot_control::current_boot_posture(),
        BootPosture::Normal | BootPosture::Probation
    ) {
        emit_append_record(
            &evidence,
            None,
            None,
            None,
            false,
            "capability_denied",
            "boot_control_safe_mode",
            "evidence_only",
            None,
        );
        return;
    }
    let payload = durable_record_payload_bytes();
    let planned = match plan_reclog_append(&evidence.scan, &payload, evidence.reclog_byte_count) {
        Ok(planned) => planned,
        Err(denied) => {
            emit_append_record(
                &evidence,
                None,
                None,
                None,
                false,
                "capability_denied",
                denied.reason(),
                "evidence_only",
                None,
            );
            return;
        }
    };

    let Some(controller) = evidence.controller else {
        emit_append_record(
            &evidence,
            Some(&planned),
            None,
            None,
            false,
            "capability_denied",
            "ahci_controller_not_observed",
            "evidence_only",
            None,
        );
        return;
    };

    let write = unsafe {
        ahci::write_readback_reclog_append(controller, planned.write_offset, &planned.frame)
    };
    let readback_sha256 = write.readback.as_deref().map(sha256_bytes);
    let reparse_valid = write
        .readback
        .as_deref()
        .map(|bytes| parse_reclog_frame(bytes, 0, planned.seq, planned.prev_frame_sha256).is_ok())
        .unwrap_or(false);
    let decision = evaluate_scoped_seed_data_append(&ScopedSeedDataAppendInput {
        method: Some(EXPECTED_METHOD),
        target_id: Some(EXPECTED_TARGET_ID),
        record_schema: Some(EXPECTED_RECORD_SCHEMA),
        region_marker: Some(EXPECTED_REGION_MARKER),
        frame_len: Some(planned.frame_len),
        write_offset: Some(planned.write_offset),
        reclog_byte_count: Some(evidence.reclog_byte_count),
        absolute_start_lba: Some(evidence.reclog_absolute_start_lba),
        reclog_lba_count: Some(evidence.reclog_lba_count),
        seq: Some(planned.seq),
        tail_seq: Some(evidence.scan.tail_seq),
        count: Some(evidence.scan.count),
        prev_frame_sha256: Some(planned.prev_frame_sha256),
        tail_frame_sha256: evidence.scan.tail_frame_sha256,
        payload_sha256: Some(planned.payload_sha256),
        planned_payload_sha256: Some(planned.payload_sha256),
        planned_frame_sha256: Some(planned.frame_sha256),
        readback_frame_sha256: readback_sha256,
        write_attempted: write.write_attempted,
        write_completed: write.write_completed,
        readback_completed: write.readback_completed,
        readback_matches_planned: readback_sha256 == Some(planned.frame_sha256),
        reparse_valid,
        span_in_bounds: write.span_in_bounds,
    });

    if !decision.performed {
        emit_append_record(
            &evidence,
            Some(&planned),
            Some(&write),
            readback_sha256,
            reparse_valid,
            "capability_denied",
            decision.reason,
            "evidence_only",
            None,
        );
        return;
    }

    let after = current_boot_reclog_scan();
    let rescan_ok = evidence
        .scan
        .count
        .checked_add(1)
        .map(|count| after.scan.count == count)
        .unwrap_or(false)
        && after.scan.tail_seq == planned.seq
        && after.scan.tail_frame_sha256 == Some(planned.frame_sha256);
    if !rescan_ok {
        emit_append_record(
            &evidence,
            Some(&planned),
            Some(&write),
            readback_sha256,
            reparse_valid,
            "capability_denied",
            "post_append_rescan_mismatch",
            "evidence_only",
            Some(&after),
        );
        return;
    }

    emit_append_record(
        &evidence,
        Some(&planned),
        Some(&write),
        readback_sha256,
        true,
        "appended",
        decision.reason,
        "scoped_seed_data_append_authorized",
        Some(&after),
    );
}

struct VecSink(Vec<u8>);

impl ByteSink for VecSink {
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }
}

fn durable_record_payload_bytes() -> Vec<u8> {
    let record = V::Object(vec![
        f("schema", s(EXPECTED_RECORD_SCHEMA)),
        f("id", s("durable_record.boot_lifecycle.current_boot.v0")),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("record_kind", s("boot_lifecycle_marker")),
        f("mirrors", s("current_boot.boot_marker")),
        f("persistence_claimed", b(false)),
    ]);
    let mut sink = VecSink(Vec::new());
    write_json(&record, &mut sink, 0);
    sink.0
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PromotionTransactionKind {
    Promote,
    Unpromote,
}

impl PromotionTransactionKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Promote => "promote",
            Self::Unpromote => "unpromote",
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PromotionTransactionRecord {
    pub(crate) transaction_kind: PromotionTransactionKind,
    pub(crate) computed_grant_hash: [u8; 32],
    pub(crate) manifest_hash: [u8; 32],
    pub(crate) artifact_hash: [u8; 32],
    pub(crate) vm_report_hash: [u8; 32],
    pub(crate) local_attestation_hash: [u8; 32],
    pub(crate) retained_manifest_reference_event_id: event_log::EventId,
    pub(crate) retained_artifact_reference_event_id: event_log::EventId,
    pub(crate) retained_vm_report_reference_event_id: event_log::EventId,
    pub(crate) retained_reference_event_id: event_log::EventId,
    pub(crate) manifest_reference_hash: [u8; 32],
    pub(crate) artifact_reference_hash: [u8; 32],
    pub(crate) vm_report_reference_hash: [u8; 32],
    pub(crate) attestation_reference_hash: [u8; 32],
    pub(crate) signature_der: [u8; event_log::MAX_PROMOTION_SIGNATURE_DER_LEN],
    pub(crate) signature_len: usize,
    pub(crate) signature_verified: bool,
    pub(crate) signature_attestation_reference_hash: [u8; 32],
    pub(crate) promotion_authority_key_sha256: [u8; 32],
    pub(crate) grant_binds_capability: bool,
    pub(crate) rollback_plan_hash: [u8; 32],
    pub(crate) pre_load_inventory_hash: [u8; 32],
    pub(crate) ram_only_service_slot_id: &'static str,
    pub(crate) generation: u64,
    pub(crate) load_event_id: event_log::EventId,
    pub(crate) rollback_apply_event_id: Option<event_log::EventId>,
    pub(crate) reprojected_inventory_hash: Option<[u8; 32]>,
    pub(crate) restore_hash_verified: bool,
    pub(crate) stopped: bool,
    pub(crate) drop_clear_bytes: bool,
    pub(crate) free_slot: bool,
    pub(crate) remove_inventory: bool,
    pub(crate) service_id: &'static str,
    pub(crate) artifact_id: &'static str,
    pub(crate) requested_capability: &'static str,
    pub(crate) load_mode: &'static str,
}

impl PromotionTransactionRecord {
    fn signature_bytes(&self) -> &[u8] {
        &self.signature_der[..self
            .signature_len
            .min(event_log::MAX_PROMOTION_SIGNATURE_DER_LEN)]
    }

    fn scoped_signature_verified(&self) -> bool {
        self.signature_verified
            && self.signature_len > 0
            && self.signature_attestation_reference_hash == self.attestation_reference_hash
            && self.promotion_authority_key_sha256
                == PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PromotionTransactionAppendEvidence {
    pub(crate) transaction_kind: &'static str,
    pub(crate) durable_append: &'static str,
    pub(crate) performed: bool,
    pub(crate) reason: &'static str,
    pub(crate) authority: &'static str,
    pub(crate) seq: Option<u64>,
    pub(crate) write_offset: Option<u64>,
    pub(crate) frame_len: Option<u64>,
    pub(crate) payload_sha256: Option<[u8; 32]>,
    pub(crate) frame_sha256: Option<[u8; 32]>,
    pub(crate) readback_sha256: Option<[u8; 32]>,
    pub(crate) reparse_valid: bool,
    pub(crate) tail_seq_after: Option<u64>,
    pub(crate) count_after: Option<u64>,
    pub(crate) signature_verified: bool,
    pub(crate) grant_binds_capability: bool,
    pub(crate) trust_tier: &'static str,
    pub(crate) promotion_authority_is_placeholder: bool,
    pub(crate) owner_sealed: bool,
    pub(crate) cross_reboot_proven: bool,
    pub(crate) persistence_claimed: bool,
}

pub(crate) fn promotion_transaction_append_denied(
    transaction_kind: PromotionTransactionKind,
    reason: &'static str,
) -> PromotionTransactionAppendEvidence {
    promotion_transaction_append_evidence(
        transaction_kind.as_str(),
        "capability_denied",
        reason,
        "evidence_only",
        None,
        None,
        None,
        false,
        None,
        false,
        false,
    )
}

pub(crate) fn promotion_transaction_append_evidence_record(
    evidence: PromotionTransactionAppendEvidence,
) -> V<'static> {
    V::Object(vec![
        f("schema", s(PROMOTION_APPEND_SCHEMA)),
        f("id", s(PROMOTION_APPEND_ID)),
        f("method", s(PROMOTION_APPEND_METHOD)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("transaction_kind", s(evidence.transaction_kind)),
        f("durable_append", s(evidence.durable_append)),
        f(
            "code",
            s(if evidence.performed {
                "ok"
            } else {
                "capability_denied"
            }),
        ),
        f("performed", b(evidence.performed)),
        f("reason", s(evidence.reason)),
        f("authority", s(evidence.authority)),
        f("target_id", s(PROMOTION_EXPECTED_TARGET_ID)),
        f("record_schema", s(PROMOTION_EXPECTED_RECORD_SCHEMA)),
        f("region_marker", s(PROMOTION_EXPECTED_REGION_MARKER)),
        f("seq", optional_u64(evidence.seq)),
        f("write_offset", optional_u64(evidence.write_offset)),
        f("frame_len", optional_u64(evidence.frame_len)),
        f(
            "payload_sha256",
            record_sha_or_null(evidence.payload_sha256),
        ),
        f("frame_sha256", record_sha_or_null(evidence.frame_sha256)),
        f(
            "readback_sha256",
            record_sha_or_null(evidence.readback_sha256),
        ),
        f("reparse_valid", b(evidence.reparse_valid)),
        f("tail_seq_after", optional_u64(evidence.tail_seq_after)),
        f("count_after", optional_u64(evidence.count_after)),
        f("signature_verified", b(evidence.signature_verified)),
        f("grant_binds_capability", b(evidence.grant_binds_capability)),
        f("trust_tier", s(evidence.trust_tier)),
        f(
            "promotion_authority_is_placeholder",
            b(evidence.promotion_authority_is_placeholder),
        ),
        f("owner_sealed", b(evidence.owner_sealed)),
        f("cross_reboot_proven", b(evidence.cross_reboot_proven)),
        f("persistence_claimed", b(evidence.persistence_claimed)),
    ])
}

fn optional_u64(value: Option<u64>) -> V<'static> {
    match value {
        Some(value) => V::U64(value),
        None => V::Null,
    }
}

pub(crate) fn append_promotion_transaction(
    record: &PromotionTransactionRecord,
) -> PromotionTransactionAppendEvidence {
    if let Some(reason) = promotion_transaction_preflight_denial(
        super::boot_control::current_boot_posture(),
        record.signature_len,
    ) {
        return promotion_transaction_append_denied(record.transaction_kind, reason);
    }

    let evidence = current_boot_reclog_scan();
    let payload = promotion_transaction_payload_bytes(record);
    let planned = match plan_reclog_append(&evidence.scan, &payload, evidence.reclog_byte_count) {
        Ok(planned) => planned,
        Err(denied) => {
            return promotion_transaction_append_evidence(
                record.transaction_kind.as_str(),
                "capability_denied",
                denied.reason(),
                "evidence_only",
                None,
                None,
                None,
                false,
                None,
                record.scoped_signature_verified(),
                record.grant_binds_capability,
            )
        }
    };

    let Some(controller) = evidence.controller else {
        return promotion_transaction_append_evidence(
            record.transaction_kind.as_str(),
            "capability_denied",
            "ahci_controller_not_observed",
            "evidence_only",
            Some(&planned),
            None,
            None,
            false,
            None,
            record.scoped_signature_verified(),
            record.grant_binds_capability,
        );
    };

    let write = unsafe {
        ahci::write_readback_reclog_append(controller, planned.write_offset, &planned.frame)
    };
    let readback_sha256 = write.readback.as_deref().map(sha256_bytes);
    let reparse_valid = write
        .readback
        .as_deref()
        .map(|bytes| parse_reclog_frame(bytes, 0, planned.seq, planned.prev_frame_sha256).is_ok())
        .unwrap_or(false);
    let scoped_signature_verified = record.scoped_signature_verified();
    let decision =
        evaluate_scoped_promotion_transaction_append(&ScopedPromotionTransactionAppendInput {
            method: Some(PROMOTION_EXPECTED_METHOD),
            target_id: Some(PROMOTION_EXPECTED_TARGET_ID),
            record_schema: Some(PROMOTION_EXPECTED_RECORD_SCHEMA),
            region_marker: Some(PROMOTION_EXPECTED_REGION_MARKER),
            frame_len: Some(planned.frame_len),
            write_offset: Some(planned.write_offset),
            reclog_byte_count: Some(evidence.reclog_byte_count),
            absolute_start_lba: Some(evidence.reclog_absolute_start_lba),
            reclog_lba_count: Some(evidence.reclog_lba_count),
            seq: Some(planned.seq),
            tail_seq: Some(evidence.scan.tail_seq),
            count: Some(evidence.scan.count),
            prev_frame_sha256: Some(planned.prev_frame_sha256),
            tail_frame_sha256: evidence.scan.tail_frame_sha256,
            payload_sha256: Some(planned.payload_sha256),
            planned_payload_sha256: Some(planned.payload_sha256),
            planned_frame_sha256: Some(planned.frame_sha256),
            readback_frame_sha256: readback_sha256,
            write_attempted: write.write_attempted,
            write_completed: write.write_completed,
            readback_completed: write.readback_completed,
            readback_matches_planned: readback_sha256 == Some(planned.frame_sha256),
            reparse_valid,
            span_in_bounds: write.span_in_bounds,
            transaction_kind: Some(record.transaction_kind.as_str()),
            signature_verified: scoped_signature_verified,
            grant_binds_capability: record.grant_binds_capability,
            trust_tier: Some(PROMOTION_EXPECTED_TRUST_TIER),
            owner_sealed: false,
            promotion_authority_is_placeholder: PROMOTION_AUTHORITY_IS_PLACEHOLDER,
            persistence_claimed: false,
        });

    if !decision.performed {
        return promotion_transaction_append_evidence(
            record.transaction_kind.as_str(),
            "capability_denied",
            decision.reason,
            "evidence_only",
            Some(&planned),
            Some(&write),
            readback_sha256,
            reparse_valid,
            None,
            scoped_signature_verified,
            record.grant_binds_capability,
        );
    }

    let after = current_boot_reclog_scan();
    let rescan_ok = evidence
        .scan
        .count
        .checked_add(1)
        .map(|count| after.scan.count == count)
        .unwrap_or(false)
        && after.scan.tail_seq == planned.seq
        && after.scan.tail_frame_sha256 == Some(planned.frame_sha256);
    if !rescan_ok {
        return promotion_transaction_append_evidence(
            record.transaction_kind.as_str(),
            "capability_denied",
            "post_append_rescan_mismatch",
            "evidence_only",
            Some(&planned),
            Some(&write),
            readback_sha256,
            reparse_valid,
            Some(&after),
            scoped_signature_verified,
            record.grant_binds_capability,
        );
    }

    promotion_transaction_append_evidence(
        record.transaction_kind.as_str(),
        "appended",
        decision.reason,
        "scoped_promotion_transaction_append_authorized",
        Some(&planned),
        Some(&write),
        readback_sha256,
        true,
        Some(&after),
        scoped_signature_verified,
        record.grant_binds_capability,
    )
}

fn promotion_transaction_payload_bytes(record: &PromotionTransactionRecord) -> Vec<u8> {
    let manifest_event = EventIdString::new(record.retained_manifest_reference_event_id);
    let artifact_event = EventIdString::new(record.retained_artifact_reference_event_id);
    let vm_report_event = EventIdString::new(record.retained_vm_report_reference_event_id);
    let computed_grant_event = EventIdString::new(record.retained_reference_event_id);
    let record = V::Object(vec![
        f("schema", s(PROMOTION_EXPECTED_RECORD_SCHEMA)),
        f(
            "id",
            s(match record.transaction_kind {
                PromotionTransactionKind::Promote => {
                    "promotion_transaction.origin_boot.promote.svc.dev.granted_candidate.v0"
                }
                PromotionTransactionKind::Unpromote => {
                    "promotion_transaction.origin_boot.unpromote.svc.dev.granted_candidate.v0"
                }
            }),
        ),
        f("scope", s(PROMOTION_TRANSACTION_RECORD_SCOPE)),
        f("classification", s("local_only")),
        f("record_kind", s(PROMOTION_TRANSACTION_RECORD_KIND)),
        f("transaction_kind", s(record.transaction_kind.as_str())),
        f("service_id", s(record.service_id)),
        f("artifact_id", s(record.artifact_id)),
        f("requested_capability", s(record.requested_capability)),
        f("load_mode", s(record.load_mode)),
        f("computed_grant_hash", V::Sha256(record.computed_grant_hash)),
        f("manifest_hash", V::Sha256(record.manifest_hash)),
        f("artifact_hash", V::Sha256(record.artifact_hash)),
        f("vm_report_hash", V::Sha256(record.vm_report_hash)),
        f(
            "local_attestation_hash",
            V::Sha256(record.local_attestation_hash),
        ),
        f(
            "retained_manifest_reference_event_id",
            V::Str(manifest_event.as_str()),
        ),
        f(
            "retained_artifact_reference_event_id",
            V::Str(artifact_event.as_str()),
        ),
        f(
            "retained_vm_report_reference_event_id",
            V::Str(vm_report_event.as_str()),
        ),
        f(
            "retained_computed_grant_reference_event_id",
            V::Str(computed_grant_event.as_str()),
        ),
        f(
            "manifest_reference_hash",
            V::Sha256(record.manifest_reference_hash),
        ),
        f(
            "artifact_reference_hash",
            V::Sha256(record.artifact_reference_hash),
        ),
        f(
            "vm_report_reference_hash",
            V::Sha256(record.vm_report_reference_hash),
        ),
        f(
            "attestation_reference_hash",
            V::Sha256(record.attestation_reference_hash),
        ),
        f(
            "promotion_signature_der",
            V::HexBytes(record.signature_bytes()),
        ),
        f(
            "promotion_signature_len",
            V::U64(record.signature_len as u64),
        ),
        f(
            "promotion_authority_key_sha256",
            V::Sha256(record.promotion_authority_key_sha256),
        ),
        f("signature_verified", b(record.scoped_signature_verified())),
        f("grant_binds_capability", b(record.grant_binds_capability)),
        f("trust_tier", s(PROMOTION_TRANSACTION_TRUST_TIER)),
        f(
            "promotion_authority_is_placeholder",
            b(PROMOTION_AUTHORITY_IS_PLACEHOLDER),
        ),
        f("owner_sealed", b(false)),
        f("cross_reboot_proven", b(false)),
        f("persistence_claimed", b(false)),
        f("rollback_plan_hash", V::Sha256(record.rollback_plan_hash)),
        f(
            "pre_load_inventory_hash",
            V::Sha256(record.pre_load_inventory_hash),
        ),
        f(
            "ram_only_service_slot_id",
            s(record.ram_only_service_slot_id),
        ),
        f("generation", V::U64(record.generation)),
        f(
            "load_event_id",
            V::EventSequence(record.load_event_id.sequence()),
        ),
        f(
            "rollback_apply_event_id",
            optional_event_sequence(record.rollback_apply_event_id),
        ),
        f(
            "reprojected_inventory_hash",
            record_sha_or_null(record.reprojected_inventory_hash),
        ),
        f("restore_hash_verified", b(record.restore_hash_verified)),
        f("stopped", b(record.stopped)),
        f("drop_clear_bytes", b(record.drop_clear_bytes)),
        f("free_slot", b(record.free_slot)),
        f("remove_inventory", b(record.remove_inventory)),
    ]);
    let mut sink = VecSink(Vec::new());
    write_json(&record, &mut sink, 0);
    sink.0
}

fn optional_event_sequence(value: Option<event_log::EventId>) -> V<'static> {
    match value {
        Some(value) => V::EventSequence(value.sequence()),
        None => V::Null,
    }
}

struct EventIdString {
    bytes: [u8; 27],
}

impl EventIdString {
    fn new(event_id: event_log::EventId) -> Self {
        let mut bytes = [0u8; 27];
        bytes[..19].copy_from_slice(b"event.current_boot.");
        let mut sequence = event_id.sequence();
        let mut idx = 27usize;
        while idx > 19 {
            idx -= 1;
            bytes[idx] = b'0' + (sequence % 10) as u8;
            sequence /= 10;
        }
        Self { bytes }
    }

    fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes) }
    }
}

fn promotion_transaction_preflight_denial(
    posture: BootPosture,
    signature_len: usize,
) -> Option<&'static str> {
    if !matches!(posture, BootPosture::Normal | BootPosture::Probation) {
        Some("boot_control_safe_mode")
    } else if signature_len == 0 || signature_len > event_log::MAX_PROMOTION_SIGNATURE_DER_LEN {
        Some("promotion_signature_reference_missing")
    } else {
        None
    }
}

fn promotion_transaction_append_evidence(
    transaction_kind: &'static str,
    durable_append: &'static str,
    reason: &'static str,
    authority: &'static str,
    planned: Option<&PlannedAppend>,
    _write: Option<&ahci::ReclogAppendWriteEvidence>,
    readback_sha256: Option<[u8; 32]>,
    reparse_valid: bool,
    after: Option<&DurableRecordLogScanEvidence>,
    signature_verified: bool,
    grant_binds_capability: bool,
) -> PromotionTransactionAppendEvidence {
    PromotionTransactionAppendEvidence {
        transaction_kind,
        durable_append,
        performed: durable_append == "appended",
        reason,
        authority,
        seq: planned.map(|planned| planned.seq),
        write_offset: planned.map(|planned| planned.write_offset),
        frame_len: planned.map(|planned| planned.frame_len),
        payload_sha256: planned.map(|planned| planned.payload_sha256),
        frame_sha256: planned.map(|planned| planned.frame_sha256),
        readback_sha256,
        reparse_valid,
        tail_seq_after: after.map(|after| after.scan.tail_seq),
        count_after: after.map(|after| after.scan.count),
        signature_verified,
        grant_binds_capability,
        trust_tier: PROMOTION_TRANSACTION_TRUST_TIER,
        promotion_authority_is_placeholder: PROMOTION_AUTHORITY_IS_PLACEHOLDER,
        owner_sealed: false,
        cross_reboot_proven: false,
        persistence_claimed: false,
    }
}

pub(crate) fn emit_promotion_transaction_selftest() {
    let cases = promotion_transaction_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);
    let case_records = cases.iter().map(record_promotion_selftest_case).collect();

    begin_response(PROMOTION_TRANSACTION_SELFTEST_METHOD);
    emit_record_fields(
        vec![
            f("schema", s(PROMOTION_TRANSACTION_SELFTEST_SCHEMA)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", b(false)),
            f("writes_persistent_state", b(false)),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
            f("cases", V::Array(case_records)),
            f("evidence_complete", b(true)),
        ],
        6,
    );
    end_response(PROMOTION_TRANSACTION_SELFTEST_METHOD);
}

#[derive(Clone, Copy)]
struct PromotionTransactionSelftestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    reparsed: bool,
    passed: bool,
}

fn promotion_transaction_selftest_cases() -> Vec<PromotionTransactionSelftestCase> {
    vec![
        promotion_transaction_positive_case(PromotionTransactionKind::Promote),
        promotion_transaction_positive_case(PromotionTransactionKind::Unpromote),
        promotion_transaction_denial_case("wrong_schema_denied", SelftestMutation::WrongSchema),
        promotion_transaction_denial_case("wrong_target_denied", SelftestMutation::WrongTarget),
        promotion_transaction_denial_case("wrong_kind_denied", SelftestMutation::WrongKind),
        promotion_transaction_denial_case(
            "signature_not_verified_denied",
            SelftestMutation::SignatureNotVerified,
        ),
        promotion_transaction_preflight_case(
            "safe_posture_denied",
            BootPosture::Safe,
            event_log::MAX_PROMOTION_SIGNATURE_DER_LEN.min(70),
            "boot_control_safe_mode",
        ),
        promotion_transaction_preflight_case(
            "signature_reference_missing_denied",
            BootPosture::Normal,
            0,
            "promotion_signature_reference_missing",
        ),
    ]
}

#[derive(Clone, Copy)]
enum SelftestMutation {
    WrongSchema,
    WrongTarget,
    WrongKind,
    SignatureNotVerified,
}

fn promotion_transaction_positive_case(
    kind: PromotionTransactionKind,
) -> PromotionTransactionSelftestCase {
    let (status, reason, reparsed) = synthetic_promotion_append_decision(kind, None);
    PromotionTransactionSelftestCase {
        name: match kind {
            PromotionTransactionKind::Promote => "promote_authorized_reparse",
            PromotionTransactionKind::Unpromote => "unpromote_authorized_reparse",
        },
        expected_status: "appended",
        expected_reason: "authorized_promotion_transaction_append_readback_reparse_verified",
        actual_status: status,
        actual_reason: reason,
        reparsed,
        passed: status == "appended"
            && reason == "authorized_promotion_transaction_append_readback_reparse_verified"
            && reparsed,
    }
}

fn promotion_transaction_denial_case(
    name: &'static str,
    mutation: SelftestMutation,
) -> PromotionTransactionSelftestCase {
    let (status, reason, reparsed) =
        synthetic_promotion_append_decision(PromotionTransactionKind::Promote, Some(mutation));
    let expected_reason = match mutation {
        SelftestMutation::WrongSchema => "record_schema_mismatch",
        SelftestMutation::WrongTarget => "target_out_of_scope",
        SelftestMutation::WrongKind => "transaction_kind_out_of_scope",
        SelftestMutation::SignatureNotVerified => "signature_not_verified",
    };
    PromotionTransactionSelftestCase {
        name,
        expected_status: "denied",
        expected_reason,
        actual_status: status,
        actual_reason: reason,
        reparsed,
        passed: status == "denied" && reason == expected_reason,
    }
}

fn promotion_transaction_preflight_case(
    name: &'static str,
    posture: BootPosture,
    signature_len: usize,
    expected_reason: &'static str,
) -> PromotionTransactionSelftestCase {
    let reason = promotion_transaction_preflight_denial(posture, signature_len)
        .unwrap_or("unexpected_authorized");
    PromotionTransactionSelftestCase {
        name,
        expected_status: "denied",
        expected_reason,
        actual_status: "denied",
        actual_reason: reason,
        reparsed: false,
        passed: reason == expected_reason,
    }
}

fn synthetic_promotion_append_decision(
    kind: PromotionTransactionKind,
    mutation: Option<SelftestMutation>,
) -> (&'static str, &'static str, bool) {
    let scan = RecordLogScan {
        head_seq: 1,
        tail_seq: 2,
        count: 2,
        terminated_reason: "zero_filled",
        torn_tail: false,
        first_invalid_offset: 1024,
        valid_prefix_chain: true,
        full_region_valid: true,
        head_frame_sha256: Some([1; 32]),
        tail_frame_sha256: Some([2; 32]),
    };
    let record = promotion_transaction_selftest_record(kind);
    let payload = promotion_transaction_payload_bytes(&record);
    if payload.len() > PROMOTION_TRANSACTION_MAX_PAYLOAD_LEN {
        return ("denied", "payload_too_large_for_frame", false);
    }
    let Ok(planned) = plan_reclog_append(&scan, &payload, 4096) else {
        return ("denied", "plan_failed", false);
    };
    let reparsed =
        parse_reclog_frame(&planned.frame, 0, planned.seq, planned.prev_frame_sha256).is_ok();
    let mut input = ScopedPromotionTransactionAppendInput {
        method: Some(PROMOTION_EXPECTED_METHOD),
        target_id: Some(PROMOTION_EXPECTED_TARGET_ID),
        record_schema: Some(PROMOTION_EXPECTED_RECORD_SCHEMA),
        region_marker: Some(PROMOTION_EXPECTED_REGION_MARKER),
        frame_len: Some(planned.frame_len),
        write_offset: Some(planned.write_offset),
        reclog_byte_count: Some(4096),
        absolute_start_lba: Some(100),
        reclog_lba_count: Some(8),
        seq: Some(planned.seq),
        tail_seq: Some(scan.tail_seq),
        count: Some(scan.count),
        prev_frame_sha256: Some(planned.prev_frame_sha256),
        tail_frame_sha256: scan.tail_frame_sha256,
        payload_sha256: Some(planned.payload_sha256),
        planned_payload_sha256: Some(planned.payload_sha256),
        planned_frame_sha256: Some(planned.frame_sha256),
        readback_frame_sha256: Some(planned.frame_sha256),
        write_attempted: true,
        write_completed: true,
        readback_completed: true,
        readback_matches_planned: true,
        reparse_valid: reparsed,
        span_in_bounds: true,
        transaction_kind: Some(kind.as_str()),
        signature_verified: true,
        grant_binds_capability: true,
        trust_tier: Some(PROMOTION_EXPECTED_TRUST_TIER),
        owner_sealed: false,
        promotion_authority_is_placeholder: PROMOTION_AUTHORITY_IS_PLACEHOLDER,
        persistence_claimed: false,
    };
    if let Some(mutation) = mutation {
        match mutation {
            SelftestMutation::WrongSchema => input.record_schema = Some("raios.durable_record.v0"),
            SelftestMutation::WrongTarget => input.target_id = Some("append.record_log.seed_data"),
            SelftestMutation::WrongKind => input.transaction_kind = Some("install"),
            SelftestMutation::SignatureNotVerified => input.signature_verified = false,
        }
    }
    let decision = evaluate_scoped_promotion_transaction_append(&input);
    (decision.status, decision.reason, reparsed)
}

fn promotion_transaction_selftest_record(
    kind: PromotionTransactionKind,
) -> PromotionTransactionRecord {
    let signature_len = 4usize;
    let mut signature_der = [0u8; event_log::MAX_PROMOTION_SIGNATURE_DER_LEN];
    signature_der[..signature_len].copy_from_slice(&[0x30, 0x02, 0x01, 0x01]);
    PromotionTransactionRecord {
        transaction_kind: kind,
        computed_grant_hash: [0x10; 32],
        manifest_hash: [0x11; 32],
        artifact_hash: [0x12; 32],
        vm_report_hash: [0x13; 32],
        local_attestation_hash: [0x14; 32],
        retained_manifest_reference_event_id: selftest_event_id(26),
        retained_artifact_reference_event_id: selftest_event_id(28),
        retained_vm_report_reference_event_id: selftest_event_id(29),
        retained_reference_event_id: selftest_event_id(27),
        manifest_reference_hash: [0x21; 32],
        artifact_reference_hash: [0x22; 32],
        vm_report_reference_hash: [0x23; 32],
        attestation_reference_hash: [0x24; 32],
        signature_der,
        signature_len,
        signature_verified: true,
        signature_attestation_reference_hash: [0x24; 32],
        promotion_authority_key_sha256: PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256,
        grant_binds_capability: true,
        rollback_plan_hash: [0x31; 32],
        pre_load_inventory_hash: [0x32; 32],
        ram_only_service_slot_id: "ram_only:svc.dev.granted_candidate",
        generation: 1,
        load_event_id: selftest_event_id(61),
        rollback_apply_event_id: if kind == PromotionTransactionKind::Unpromote {
            Some(selftest_event_id(72))
        } else {
            None
        },
        reprojected_inventory_hash: if kind == PromotionTransactionKind::Unpromote {
            Some([0x32; 32])
        } else {
            None
        },
        restore_hash_verified: kind == PromotionTransactionKind::Unpromote,
        stopped: kind == PromotionTransactionKind::Unpromote,
        drop_clear_bytes: kind == PromotionTransactionKind::Unpromote,
        free_slot: kind == PromotionTransactionKind::Unpromote,
        remove_inventory: kind == PromotionTransactionKind::Unpromote,
        service_id: "svc.dev.granted_candidate",
        artifact_id: "wasm:external.granted_candidate",
        requested_capability: "cap.module.load_ephemeral.dev_tier.current_boot",
        load_mode: "wasmi_interpreter_ram_only",
    }
}

fn selftest_event_id(sequence: u64) -> event_log::EventId {
    let mut candidate = sequence;
    loop {
        if let Some(event_id) = event_log::EventId::from_sequence(candidate) {
            return event_id;
        }
        candidate = 1;
    }
}

fn record_promotion_selftest_case(case: &PromotionTransactionSelftestCase) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(case.name)),
        f("expected_status", s(case.expected_status)),
        f("expected_reason", s(case.expected_reason)),
        f("actual_status", s(case.actual_status)),
        f("actual_reason", s(case.actual_reason)),
        f("reparsed", b(case.reparsed)),
        f("passed", b(case.passed)),
    ])
}

// --- M8B-1 recovery.disable_module durable action record ---------------------------
//
// The mutating half of the recovery lifeline appends a `raios.recovery_action.v0`
// record through the SHARED reclog mechanism (current_boot_reclog_scan / plan / ahci
// write_readback / parse) authorized ONLY by evaluate_scoped_recovery_action_append.
// This mirrors append_promotion_transaction EXACTLY, swapping the scoped evaluator and
// the record payload; it forks no second durable-append implementation. Deny before
// mutate: SAFE posture and every invalid target class are rejected in the preflight
// before any plan/write.

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecoveryTargetClassification {
    LifelineEndpoint,
    CoreOwned,
    Unknown,
    KnownDisablable,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryActionRecord {
    pub(crate) action_kind: &'static str,
    pub(crate) disable_target_id: &'static str,
    pub(crate) disable_target_is_lifeline_endpoint: bool,
    pub(crate) disable_target_core_owned: bool,
    pub(crate) disable_target_known_disablable: bool,
    pub(crate) grants_new_capability: bool,
    // Restart-only (action_kind == restart_last_good). For disable_module records
    // both are false and unused: they never reach the restart-gated payload field or
    // the kind-guarded evaluator pin, so the disable evaluation/payload stay identical.
    pub(crate) restores_known_good: bool,
    pub(crate) restart_target_restorable: bool,
}

impl RecoveryActionRecord {
    pub(crate) fn classification(&self) -> RecoveryTargetClassification {
        if self.disable_target_is_lifeline_endpoint {
            RecoveryTargetClassification::LifelineEndpoint
        } else if self.disable_target_core_owned {
            RecoveryTargetClassification::CoreOwned
        } else if !self.disable_target_known_disablable {
            RecoveryTargetClassification::Unknown
        } else {
            RecoveryTargetClassification::KnownDisablable
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryActionAppendEvidence {
    pub(crate) action_kind: &'static str,
    pub(crate) disable_target_id: &'static str,
    pub(crate) durable_append: &'static str,
    pub(crate) performed: bool,
    pub(crate) reason: &'static str,
    pub(crate) authority: &'static str,
    pub(crate) seq: Option<u64>,
    pub(crate) write_offset: Option<u64>,
    pub(crate) frame_len: Option<u64>,
    pub(crate) payload_sha256: Option<[u8; 32]>,
    pub(crate) frame_sha256: Option<[u8; 32]>,
    pub(crate) readback_sha256: Option<[u8; 32]>,
    pub(crate) reparse_valid: bool,
    pub(crate) tail_seq_before: Option<u64>,
    pub(crate) count_before: Option<u64>,
    pub(crate) tail_seq_after: Option<u64>,
    pub(crate) count_after: Option<u64>,
    pub(crate) disable_target_is_lifeline_endpoint: bool,
    pub(crate) disable_target_core_owned: bool,
    pub(crate) disable_target_known_disablable: bool,
    pub(crate) grants_new_capability: bool,
    pub(crate) restores_known_good: bool,
    pub(crate) trust_tier: &'static str,
    pub(crate) promotion_authority_is_placeholder: bool,
    pub(crate) owner_sealed: bool,
    pub(crate) persistence_claimed: bool,
}

/// SAFE-mode + invalid-target preflight, mirroring `promotion_transaction_preflight_denial`.
/// SAFE (posture not Normal|Probation) denies the write before any plan/write; then the
/// lifeline/core/unknown target classes deny with their pinned reasons. `KnownDisablable`
/// in a writable posture is the only case that proceeds to the durable gauntlet.
pub(crate) fn recovery_action_preflight_denial(
    posture: BootPosture,
    classification: RecoveryTargetClassification,
) -> Option<&'static str> {
    if !matches!(posture, BootPosture::Normal | BootPosture::Probation) {
        Some("boot_control_safe_mode")
    } else {
        match classification {
            RecoveryTargetClassification::LifelineEndpoint => Some("target_is_lifeline_endpoint"),
            RecoveryTargetClassification::CoreOwned => Some("target_core_owned"),
            RecoveryTargetClassification::Unknown => Some("target_unknown"),
            RecoveryTargetClassification::KnownDisablable => None,
        }
    }
}

/// M8B-2b restart preflight: the SHARED SAFE + target-class denials FIRST (deny
/// before mutate AND before append), then the restart-only `target_not_restartable`
/// pin. Mirrors the evaluator's ordering exactly (SAFE -> lifeline -> core -> unknown
/// -> not-restartable), so a non-restorable restart is rejected before any plan/write.
pub(crate) fn recovery_restart_preflight_denial(
    posture: BootPosture,
    classification: RecoveryTargetClassification,
    restorable: bool,
) -> Option<&'static str> {
    if let Some(reason) = recovery_action_preflight_denial(posture, classification) {
        Some(reason)
    } else if !restorable {
        Some("target_not_restartable")
    } else {
        None
    }
}

pub(crate) fn recovery_action_append_denied(
    record: &RecoveryActionRecord,
    reason: &'static str,
) -> RecoveryActionAppendEvidence {
    recovery_action_append_evidence(
        record,
        "capability_denied",
        reason,
        "evidence_only",
        None,
        None,
        None,
        false,
        None,
    )
}

pub(crate) fn append_recovery_action(
    record: &RecoveryActionRecord,
) -> RecoveryActionAppendEvidence {
    if let Some(reason) = recovery_action_preflight_denial(
        super::boot_control::current_boot_posture(),
        record.classification(),
    ) {
        return recovery_action_append_denied(record, reason);
    }

    let before = current_boot_reclog_scan();
    let payload = recovery_action_payload_bytes(record);
    let planned = match plan_reclog_append(&before.scan, &payload, before.reclog_byte_count) {
        Ok(planned) => planned,
        Err(denied) => {
            return recovery_action_append_evidence(
                record,
                "capability_denied",
                denied.reason(),
                "evidence_only",
                Some(&before),
                None,
                None,
                false,
                None,
            )
        }
    };

    let Some(controller) = before.controller else {
        return recovery_action_append_evidence(
            record,
            "capability_denied",
            "ahci_controller_not_observed",
            "evidence_only",
            Some(&before),
            Some(&planned),
            None,
            false,
            None,
        );
    };

    let write = unsafe {
        ahci::write_readback_reclog_append(controller, planned.write_offset, &planned.frame)
    };
    let readback_sha256 = write.readback.as_deref().map(sha256_bytes);
    let reparse_valid = write
        .readback
        .as_deref()
        .map(|bytes| parse_reclog_frame(bytes, 0, planned.seq, planned.prev_frame_sha256).is_ok())
        .unwrap_or(false);
    let decision = evaluate_scoped_recovery_action_append(&ScopedRecoveryActionAppendInput {
        method: Some(RECOVERY_ACTION_EXPECTED_METHOD),
        target_id: Some(RECOVERY_ACTION_EXPECTED_TARGET_ID),
        record_schema: Some(RECOVERY_ACTION_EXPECTED_RECORD_SCHEMA),
        region_marker: Some(RECOVERY_ACTION_EXPECTED_REGION_MARKER),
        frame_len: Some(planned.frame_len),
        write_offset: Some(planned.write_offset),
        reclog_byte_count: Some(before.reclog_byte_count),
        absolute_start_lba: Some(before.reclog_absolute_start_lba),
        reclog_lba_count: Some(before.reclog_lba_count),
        seq: Some(planned.seq),
        tail_seq: Some(before.scan.tail_seq),
        count: Some(before.scan.count),
        prev_frame_sha256: Some(planned.prev_frame_sha256),
        tail_frame_sha256: before.scan.tail_frame_sha256,
        payload_sha256: Some(planned.payload_sha256),
        planned_payload_sha256: Some(planned.payload_sha256),
        planned_frame_sha256: Some(planned.frame_sha256),
        readback_frame_sha256: readback_sha256,
        write_attempted: write.write_attempted,
        write_completed: write.write_completed,
        readback_completed: write.readback_completed,
        readback_matches_planned: readback_sha256 == Some(planned.frame_sha256),
        reparse_valid,
        span_in_bounds: write.span_in_bounds,
        action_kind: Some(record.action_kind),
        // The restart pin is kind-guarded in the evaluator: for a disable_module
        // record this is false and never reached, so the disable evaluation stays
        // byte-identical; for restart_last_good it carries the live restorability.
        restart_target_restorable: record.restart_target_restorable,
        disable_target_is_lifeline_endpoint: record.disable_target_is_lifeline_endpoint,
        disable_target_core_owned: record.disable_target_core_owned,
        disable_target_known_disablable: record.disable_target_known_disablable,
        grants_new_capability: record.grants_new_capability,
        trust_tier: Some(RECOVERY_ACTION_EXPECTED_TRUST_TIER),
        owner_sealed: false,
        persistence_claimed: false,
    });

    if !decision.performed {
        return recovery_action_append_evidence(
            record,
            "capability_denied",
            decision.reason,
            "evidence_only",
            Some(&before),
            Some(&planned),
            readback_sha256,
            reparse_valid,
            None,
        );
    }

    let after = current_boot_reclog_scan();
    let rescan_ok = before
        .scan
        .count
        .checked_add(1)
        .map(|count| after.scan.count == count)
        .unwrap_or(false)
        && after.scan.tail_seq == planned.seq
        && after.scan.tail_frame_sha256 == Some(planned.frame_sha256);
    if !rescan_ok {
        return recovery_action_append_evidence(
            record,
            "capability_denied",
            "post_append_rescan_mismatch",
            "evidence_only",
            Some(&before),
            Some(&planned),
            readback_sha256,
            reparse_valid,
            Some(&after),
        );
    }

    recovery_action_append_evidence(
        record,
        "appended",
        decision.reason,
        "scoped_recovery_action_append_authorized",
        Some(&before),
        Some(&planned),
        readback_sha256,
        true,
        Some(&after),
    )
}

#[allow(clippy::too_many_arguments)]
fn recovery_action_append_evidence(
    record: &RecoveryActionRecord,
    durable_append: &'static str,
    reason: &'static str,
    authority: &'static str,
    before: Option<&DurableRecordLogScanEvidence>,
    planned: Option<&PlannedAppend>,
    readback_sha256: Option<[u8; 32]>,
    reparse_valid: bool,
    after: Option<&DurableRecordLogScanEvidence>,
) -> RecoveryActionAppendEvidence {
    RecoveryActionAppendEvidence {
        action_kind: record.action_kind,
        disable_target_id: record.disable_target_id,
        durable_append,
        performed: durable_append == "appended",
        reason,
        authority,
        seq: planned.map(|planned| planned.seq),
        write_offset: planned.map(|planned| planned.write_offset),
        frame_len: planned.map(|planned| planned.frame_len),
        payload_sha256: planned.map(|planned| planned.payload_sha256),
        frame_sha256: planned.map(|planned| planned.frame_sha256),
        readback_sha256,
        reparse_valid,
        tail_seq_before: before.map(|before| before.scan.tail_seq),
        count_before: before.map(|before| before.scan.count),
        tail_seq_after: after.map(|after| after.scan.tail_seq),
        count_after: after.map(|after| after.scan.count),
        disable_target_is_lifeline_endpoint: record.disable_target_is_lifeline_endpoint,
        disable_target_core_owned: record.disable_target_core_owned,
        disable_target_known_disablable: record.disable_target_known_disablable,
        grants_new_capability: record.grants_new_capability,
        restores_known_good: record.restores_known_good,
        trust_tier: RECOVERY_ACTION_TRUST_TIER,
        promotion_authority_is_placeholder: PROMOTION_AUTHORITY_IS_PLACEHOLDER,
        owner_sealed: false,
        persistence_claimed: false,
    }
}

pub(crate) fn recovery_action_payload_bytes(record: &RecoveryActionRecord) -> Vec<u8> {
    // Branch on action_kind so the DISABLE payload stays BYTE-IDENTICAL: the disable
    // branch emits exactly the M8B-1 field set/order, and the restart-only fields are
    // appended ONLY for restart_last_good (with its own record id).
    let is_restart = record.action_kind == RECOVERY_ACTION_KIND_RESTART_LAST_GOOD;
    let id = if is_restart {
        "recovery_action.current_boot.restart_last_good.svc.demo.echo.v0"
    } else {
        "recovery_action.current_boot.disable_module.svc.demo.echo.v0"
    };
    let mut fields = vec![
        f("schema", s(RECOVERY_ACTION_EXPECTED_RECORD_SCHEMA)),
        f("id", s(id)),
        f("scope", s(RECOVERY_ACTION_RECORD_SCOPE)),
        f("classification", s("local_only")),
        f("record_kind", s(RECOVERY_ACTION_RECORD_KIND)),
        f("action_kind", s(record.action_kind)),
        f("disable_target_id", s(record.disable_target_id)),
        f(
            "disable_target_is_lifeline_endpoint",
            b(record.disable_target_is_lifeline_endpoint),
        ),
        f(
            "disable_target_core_owned",
            b(record.disable_target_core_owned),
        ),
        f(
            "disable_target_known_disablable",
            b(record.disable_target_known_disablable),
        ),
        f("grants_new_capability", b(record.grants_new_capability)),
        f("trust_tier", s(RECOVERY_ACTION_TRUST_TIER)),
        f(
            "promotion_authority_is_placeholder",
            b(PROMOTION_AUTHORITY_IS_PLACEHOLDER),
        ),
        f("owner_sealed", b(false)),
        f("reversible_this_boot", b(false)),
        f("mutates_live_state", b(true)),
        f("persistence_claimed", b(false)),
    ];
    if is_restart {
        fields.push(f("restores_known_good", b(record.restores_known_good)));
        fields.push(f(
            "restart_target_restorable",
            b(record.restart_target_restorable),
        ));
    }
    let payload = V::Object(fields);
    let mut sink = VecSink(Vec::new());
    write_json(&payload, &mut sink, 0);
    sink.0
}

// --- M8D-2 recovery.load_artifact_by_hash durable reinstatement audit ----------------
//
// The recovery-lifeline authority flip (M8D-2) appends a `raios.recovery_load.v0` record
// through the SHARED reclog mechanism (current_boot_reclog_scan / plan / ahci
// write_readback / parse) authorized ONLY by evaluate_scoped_recovery_load_append. Like
// append_recovery_action / append_promotion_transaction, it forks NO second durable-append
// implementation — it swaps only the scoped evaluator and the record payload. The lifeline
// calls this ONLY after the UNMODIFIED M6 gate genuinely re-instated the RAM-only
// current-boot service (append-after-gate, matching repromotion.run's audit discipline),
// so the record honestly attests a real reinstatement; every denial path in the lifeline
// fails closed BEFORE reaching this append.

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLoadRecord {
    pub(crate) artifact_persist_seq: u64,
    pub(crate) artifact_sha256: [u8; 32],
    pub(crate) manifest_hash: [u8; 32],
    pub(crate) vm_report_hash: [u8; 32],
    pub(crate) grant_hash: [u8; 32],
    pub(crate) promotion_transaction_sha256: [u8; 32],
    pub(crate) artstor_blob_offset: u64,
    pub(crate) artstor_blob_len: u64,
    pub(crate) artstor_blob_frame_sha256: [u8; 32],
    pub(crate) load_granted: bool,
    pub(crate) service_loaded: bool,
    pub(crate) service_started: bool,
    pub(crate) authorizes_load: bool,
    pub(crate) cross_reboot_proven: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLoadAppendEvidence {
    pub(crate) durable_append: &'static str,
    pub(crate) performed: bool,
    pub(crate) reason: &'static str,
    pub(crate) authority: &'static str,
    pub(crate) seq: Option<u64>,
    pub(crate) write_offset: Option<u64>,
    pub(crate) frame_len: Option<u64>,
    pub(crate) payload_sha256: Option<[u8; 32]>,
    pub(crate) frame_sha256: Option<[u8; 32]>,
    pub(crate) readback_sha256: Option<[u8; 32]>,
    pub(crate) reparse_valid: bool,
    pub(crate) tail_seq_before: Option<u64>,
    pub(crate) count_before: Option<u64>,
    pub(crate) tail_seq_after: Option<u64>,
    pub(crate) count_after: Option<u64>,
}

pub(crate) fn recovery_load_append_denied(reason: &'static str) -> RecoveryLoadAppendEvidence {
    recovery_load_append_evidence(
        "capability_denied",
        reason,
        "evidence_only",
        None,
        None,
        None,
        false,
        None,
    )
}

/// Append the durable `raios.recovery_load.v0` reinstatement audit through the SHARED
/// reclog gauntlet (scan -> plan -> ahci write_readback -> readback-sha -> reparse ->
/// evaluate_scoped_recovery_load_append -> rescan), mirroring append_recovery_action
/// EXACTLY. SAFE/PersistenceUnavailable posture denies before any plan/write. The caller
/// (recovery lifeline) invokes this ONLY on a genuine gate reinstatement, so the record's
/// decision_status is always the reinstated success status.
pub(crate) fn append_recovery_load(record: &RecoveryLoadRecord) -> RecoveryLoadAppendEvidence {
    if !matches!(
        super::boot_control::current_boot_posture(),
        BootPosture::Normal | BootPosture::Probation
    ) {
        return recovery_load_append_denied("boot_control_safe_mode");
    }

    let before = current_boot_reclog_scan();
    let payload = recovery_load_payload_bytes(record);
    let planned = match plan_reclog_append(&before.scan, &payload, before.reclog_byte_count) {
        Ok(planned) => planned,
        Err(denied) => {
            return recovery_load_append_evidence(
                "capability_denied",
                denied.reason(),
                "evidence_only",
                Some(&before),
                None,
                None,
                false,
                None,
            )
        }
    };

    let Some(controller) = before.controller else {
        return recovery_load_append_evidence(
            "capability_denied",
            "ahci_controller_not_observed",
            "evidence_only",
            Some(&before),
            Some(&planned),
            None,
            false,
            None,
        );
    };

    let write = unsafe {
        ahci::write_readback_reclog_append(controller, planned.write_offset, &planned.frame)
    };
    let readback_sha256 = write.readback.as_deref().map(sha256_bytes);
    let reparse_valid = write
        .readback
        .as_deref()
        .map(|bytes| parse_reclog_frame(bytes, 0, planned.seq, planned.prev_frame_sha256).is_ok())
        .unwrap_or(false);
    let decision = evaluate_scoped_recovery_load_append(&ScopedRecoveryLoadAppendInput {
        method: Some(RECOVERY_LOAD_EXPECTED_METHOD),
        target_id: Some(RECOVERY_LOAD_EXPECTED_TARGET_ID),
        record_schema: Some(RECOVERY_LOAD_EXPECTED_RECORD_SCHEMA),
        region_marker: Some(RECOVERY_LOAD_EXPECTED_REGION_MARKER),
        frame_len: Some(planned.frame_len),
        write_offset: Some(planned.write_offset),
        reclog_byte_count: Some(before.reclog_byte_count),
        absolute_start_lba: Some(before.reclog_absolute_start_lba),
        reclog_lba_count: Some(before.reclog_lba_count),
        seq: Some(planned.seq),
        tail_seq: Some(before.scan.tail_seq),
        count: Some(before.scan.count),
        prev_frame_sha256: Some(planned.prev_frame_sha256),
        tail_frame_sha256: before.scan.tail_frame_sha256,
        payload_sha256: Some(planned.payload_sha256),
        planned_payload_sha256: Some(planned.payload_sha256),
        planned_frame_sha256: Some(planned.frame_sha256),
        readback_frame_sha256: readback_sha256,
        write_attempted: write.write_attempted,
        write_completed: write.write_completed,
        readback_completed: write.readback_completed,
        readback_matches_planned: readback_sha256 == Some(planned.frame_sha256),
        reparse_valid,
        span_in_bounds: write.span_in_bounds,
        // The lifeline only appends on a genuine reinstatement, so the audit always
        // carries the reinstated success status with the REAL gate load authority.
        decision_status: Some(RECOVERY_LOAD_DECISION_STATUS_REINSTATED),
        would_reinstate: true,
        authorizes_load: record.authorizes_load,
        trust_tier: Some(RECOVERY_LOAD_EXPECTED_TRUST_TIER),
        owner_sealed: false,
        promotion_authority_is_placeholder: PROMOTION_AUTHORITY_IS_PLACEHOLDER,
        cross_reboot_proven: record.cross_reboot_proven,
        persistence_claimed: false,
    });

    if !decision.performed {
        return recovery_load_append_evidence(
            "capability_denied",
            decision.reason,
            "evidence_only",
            Some(&before),
            Some(&planned),
            readback_sha256,
            reparse_valid,
            None,
        );
    }

    let after = current_boot_reclog_scan();
    let rescan_ok = before
        .scan
        .count
        .checked_add(1)
        .map(|count| after.scan.count == count)
        .unwrap_or(false)
        && after.scan.tail_seq == planned.seq
        && after.scan.tail_frame_sha256 == Some(planned.frame_sha256);
    if !rescan_ok {
        return recovery_load_append_evidence(
            "capability_denied",
            "post_append_rescan_mismatch",
            "evidence_only",
            Some(&before),
            Some(&planned),
            readback_sha256,
            reparse_valid,
            Some(&after),
        );
    }

    recovery_load_append_evidence(
        "appended",
        decision.reason,
        "scoped_recovery_load_append_authorized",
        Some(&before),
        Some(&planned),
        readback_sha256,
        true,
        Some(&after),
    )
}

#[allow(clippy::too_many_arguments)]
fn recovery_load_append_evidence(
    durable_append: &'static str,
    reason: &'static str,
    authority: &'static str,
    before: Option<&DurableRecordLogScanEvidence>,
    planned: Option<&PlannedAppend>,
    readback_sha256: Option<[u8; 32]>,
    reparse_valid: bool,
    after: Option<&DurableRecordLogScanEvidence>,
) -> RecoveryLoadAppendEvidence {
    RecoveryLoadAppendEvidence {
        durable_append,
        performed: durable_append == "appended",
        reason,
        authority,
        seq: planned.map(|planned| planned.seq),
        write_offset: planned.map(|planned| planned.write_offset),
        frame_len: planned.map(|planned| planned.frame_len),
        payload_sha256: planned.map(|planned| planned.payload_sha256),
        frame_sha256: planned.map(|planned| planned.frame_sha256),
        readback_sha256,
        reparse_valid,
        tail_seq_before: before.map(|before| before.scan.tail_seq),
        count_before: before.map(|before| before.scan.count),
        tail_seq_after: after.map(|after| after.scan.tail_seq),
        count_after: after.map(|after| after.scan.count),
    }
}

pub(crate) fn recovery_load_payload_bytes(record: &RecoveryLoadRecord) -> Vec<u8> {
    let payload = V::Object(vec![
        f("schema", s(RECOVERY_LOAD_EXPECTED_RECORD_SCHEMA)),
        f(
            "id",
            s("recovery_load.current_boot.reinstate.svc.dev.granted_candidate.v0"),
        ),
        f("scope", s(RECOVERY_LOAD_RECORD_SCOPE)),
        f("classification", s(RECOVERY_LOAD_RECORD_CLASSIFICATION)),
        f("record_kind", s(RECOVERY_LOAD_RECORD_KIND)),
        f("method", s(RECOVERY_LOAD_EXPECTED_METHOD)),
        f(
            "decision_status",
            s(RECOVERY_LOAD_DECISION_STATUS_REINSTATED),
        ),
        f("service_id", s(RECOVERY_LOAD_SERVICE_ID)),
        f("artifact_persist_seq", V::U64(record.artifact_persist_seq)),
        f("artifact_sha256", V::Sha256(record.artifact_sha256)),
        f("manifest_hash", V::Sha256(record.manifest_hash)),
        f("vm_report_hash", V::Sha256(record.vm_report_hash)),
        f("grant_hash", V::Sha256(record.grant_hash)),
        f(
            "promotion_transaction_sha256",
            V::Sha256(record.promotion_transaction_sha256),
        ),
        f("artstor_blob_offset", V::U64(record.artstor_blob_offset)),
        f("artstor_blob_len", V::U64(record.artstor_blob_len)),
        f(
            "artstor_blob_frame_sha256",
            V::Sha256(record.artstor_blob_frame_sha256),
        ),
        f("load_granted", b(record.load_granted)),
        f("service_loaded", b(record.service_loaded)),
        f("service_started", b(record.service_started)),
        f("authorizes_load", b(record.authorizes_load)),
        f("cross_reboot_proven", b(record.cross_reboot_proven)),
        f("would_reinstate", b(true)),
        f("grants_new_capability", b(false)),
        f("trust_tier", s(RECOVERY_LOAD_TRUST_TIER)),
        f(
            "promotion_authority_is_placeholder",
            b(PROMOTION_AUTHORITY_IS_PLACEHOLDER),
        ),
        f("owner_sealed", b(false)),
        f("reversible_this_boot", b(false)),
        f("mutates_live_state", b(true)),
        f("persistence_claimed", b(false)),
    ]);
    let mut sink = VecSink(Vec::new());
    write_json(&payload, &mut sink, 0);
    sink.0
}

pub(crate) fn emit_recovery_action_selftest() {
    let cases = recovery_action_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);
    let case_records = cases
        .iter()
        .map(record_recovery_action_selftest_case)
        .collect();

    begin_response(RECOVERY_ACTION_SELFTEST_METHOD);
    emit_record_fields(
        vec![
            f("schema", s(RECOVERY_ACTION_SELFTEST_SCHEMA)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", b(false)),
            f("writes_persistent_state", b(false)),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
            f("cases", V::Array(case_records)),
            f("evidence_complete", b(true)),
        ],
        6,
    );
    end_response(RECOVERY_ACTION_SELFTEST_METHOD);
}

#[derive(Clone, Copy)]
struct RecoveryActionSelftestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    reparsed: bool,
    passed: bool,
}

#[derive(Clone, Copy)]
enum RecoverySelftestMutation {
    WrongSchema,
    WrongTarget,
    WrongActionKind,
    LifelineEndpoint,
    CoreOwned,
    TargetUnknown,
    GrantsCapability,
}

fn recovery_action_selftest_cases() -> Vec<RecoveryActionSelftestCase> {
    vec![
        recovery_action_positive_case(),
        recovery_action_restart_positive_case(),
        recovery_action_denial_case("wrong_schema_denied", RecoverySelftestMutation::WrongSchema),
        recovery_action_denial_case("wrong_target_denied", RecoverySelftestMutation::WrongTarget),
        recovery_action_denial_case(
            "wrong_action_kind_denied",
            RecoverySelftestMutation::WrongActionKind,
        ),
        recovery_action_denial_case(
            "lifeline_endpoint_denied",
            RecoverySelftestMutation::LifelineEndpoint,
        ),
        recovery_action_denial_case("core_owned_denied", RecoverySelftestMutation::CoreOwned),
        recovery_action_denial_case(
            "target_unknown_denied",
            RecoverySelftestMutation::TargetUnknown,
        ),
        recovery_action_restart_not_restorable_case(),
        recovery_action_restart_shared_pin_case(),
        recovery_action_denial_case(
            "grants_capability_denied",
            RecoverySelftestMutation::GrantsCapability,
        ),
        recovery_action_preflight_case(
            "safe_posture_denied",
            BootPosture::Safe,
            RecoveryTargetClassification::KnownDisablable,
            "boot_control_safe_mode",
        ),
        recovery_action_preflight_case(
            "preflight_lifeline_endpoint_denied",
            BootPosture::Normal,
            RecoveryTargetClassification::LifelineEndpoint,
            "target_is_lifeline_endpoint",
        ),
        recovery_action_preflight_case(
            "preflight_core_owned_denied",
            BootPosture::Normal,
            RecoveryTargetClassification::CoreOwned,
            "target_core_owned",
        ),
        recovery_action_preflight_case(
            "preflight_target_unknown_denied",
            BootPosture::Normal,
            RecoveryTargetClassification::Unknown,
            "target_unknown",
        ),
    ]
}

fn recovery_action_positive_case() -> RecoveryActionSelftestCase {
    let (status, reason, reparsed) =
        synthetic_recovery_action_decision(RECOVERY_ACTION_EXPECTED_ACTION_KIND, false, None);
    RecoveryActionSelftestCase {
        name: "disable_module_authorized_reparse",
        expected_status: "appended",
        expected_reason: "authorized_recovery_action_append_readback_reparse_verified",
        actual_status: status,
        actual_reason: reason,
        reparsed,
        passed: status == "appended"
            && reason == "authorized_recovery_action_append_readback_reparse_verified"
            && reparsed,
    }
}

/// M8B-2a: the widened restart_last_good kind authorizes the SAME durable
/// gauntlet with the SAME success reason, once its target is restorable.
fn recovery_action_restart_positive_case() -> RecoveryActionSelftestCase {
    let (status, reason, reparsed) = synthetic_recovery_action_decision(
        RECOVERY_ACTION_EXPECTED_ACTION_KIND_RESTART_LAST_GOOD,
        true,
        None,
    );
    RecoveryActionSelftestCase {
        name: "restart_last_good_authorized_reparse",
        expected_status: "appended",
        expected_reason: "authorized_recovery_action_append_readback_reparse_verified",
        actual_status: status,
        actual_reason: reason,
        reparsed,
        passed: status == "appended"
            && reason == "authorized_recovery_action_append_readback_reparse_verified"
            && reparsed,
    }
}

/// M8B-2a: restart_last_good against a non-restorable target denies on the new
/// kind-guarded pin. A disable_module evaluation can never reach this reason.
fn recovery_action_restart_not_restorable_case() -> RecoveryActionSelftestCase {
    let (status, reason, reparsed) = synthetic_recovery_action_decision(
        RECOVERY_ACTION_EXPECTED_ACTION_KIND_RESTART_LAST_GOOD,
        false,
        None,
    );
    RecoveryActionSelftestCase {
        name: "restart_not_restorable_denied",
        expected_status: "denied",
        expected_reason: "target_not_restartable",
        actual_status: status,
        actual_reason: reason,
        reparsed,
        passed: status == "denied" && reason == "target_not_restartable",
    }
}

/// M8B-2a: even a restorable restart_last_good target still hits the SHARED
/// target-class pins first — core-owned denies before the restart pin — proving
/// the widening did not let restart bypass any shared gate.
fn recovery_action_restart_shared_pin_case() -> RecoveryActionSelftestCase {
    let (status, reason, reparsed) = synthetic_recovery_action_decision(
        RECOVERY_ACTION_EXPECTED_ACTION_KIND_RESTART_LAST_GOOD,
        true,
        Some(RecoverySelftestMutation::CoreOwned),
    );
    RecoveryActionSelftestCase {
        name: "restart_core_owned_denied",
        expected_status: "denied",
        expected_reason: "target_core_owned",
        actual_status: status,
        actual_reason: reason,
        reparsed,
        passed: status == "denied" && reason == "target_core_owned",
    }
}

fn recovery_action_denial_case(
    name: &'static str,
    mutation: RecoverySelftestMutation,
) -> RecoveryActionSelftestCase {
    let (status, reason, reparsed) = synthetic_recovery_action_decision(
        RECOVERY_ACTION_EXPECTED_ACTION_KIND,
        false,
        Some(mutation),
    );
    let expected_reason = match mutation {
        RecoverySelftestMutation::WrongSchema => "record_schema_mismatch",
        RecoverySelftestMutation::WrongTarget => "target_out_of_scope",
        RecoverySelftestMutation::WrongActionKind => "action_kind_out_of_scope",
        RecoverySelftestMutation::LifelineEndpoint => "target_is_lifeline_endpoint",
        RecoverySelftestMutation::CoreOwned => "target_core_owned",
        RecoverySelftestMutation::TargetUnknown => "target_unknown",
        RecoverySelftestMutation::GrantsCapability => "grants_capability_not_allowed",
    };
    RecoveryActionSelftestCase {
        name,
        expected_status: "denied",
        expected_reason,
        actual_status: status,
        actual_reason: reason,
        reparsed,
        passed: status == "denied" && reason == expected_reason,
    }
}

fn recovery_action_preflight_case(
    name: &'static str,
    posture: BootPosture,
    classification: RecoveryTargetClassification,
    expected_reason: &'static str,
) -> RecoveryActionSelftestCase {
    let reason = recovery_action_preflight_denial(posture, classification)
        .unwrap_or("unexpected_authorized");
    RecoveryActionSelftestCase {
        name,
        expected_status: "denied",
        expected_reason,
        actual_status: "denied",
        actual_reason: reason,
        reparsed: false,
        passed: reason == expected_reason,
    }
}

fn synthetic_recovery_action_decision(
    action_kind: &'static str,
    restart_target_restorable: bool,
    mutation: Option<RecoverySelftestMutation>,
) -> (&'static str, &'static str, bool) {
    let scan = RecordLogScan {
        head_seq: 1,
        tail_seq: 2,
        count: 2,
        terminated_reason: "zero_filled",
        torn_tail: false,
        first_invalid_offset: 1024,
        valid_prefix_chain: true,
        full_region_valid: true,
        head_frame_sha256: Some([1; 32]),
        tail_frame_sha256: Some([2; 32]),
    };
    let record = recovery_action_selftest_record();
    let payload = recovery_action_payload_bytes(&record);
    if payload.len() > RECOVERY_ACTION_MAX_PAYLOAD_LEN {
        return ("denied", "payload_too_large_for_frame", false);
    }
    let Ok(planned) = plan_reclog_append(&scan, &payload, 4096) else {
        return ("denied", "plan_failed", false);
    };
    let reparsed =
        parse_reclog_frame(&planned.frame, 0, planned.seq, planned.prev_frame_sha256).is_ok();
    let mut input = ScopedRecoveryActionAppendInput {
        method: Some(RECOVERY_ACTION_EXPECTED_METHOD),
        target_id: Some(RECOVERY_ACTION_EXPECTED_TARGET_ID),
        record_schema: Some(RECOVERY_ACTION_EXPECTED_RECORD_SCHEMA),
        region_marker: Some(RECOVERY_ACTION_EXPECTED_REGION_MARKER),
        frame_len: Some(planned.frame_len),
        write_offset: Some(planned.write_offset),
        reclog_byte_count: Some(4096),
        absolute_start_lba: Some(100),
        reclog_lba_count: Some(8),
        seq: Some(planned.seq),
        tail_seq: Some(scan.tail_seq),
        count: Some(scan.count),
        prev_frame_sha256: Some(planned.prev_frame_sha256),
        tail_frame_sha256: scan.tail_frame_sha256,
        payload_sha256: Some(planned.payload_sha256),
        planned_payload_sha256: Some(planned.payload_sha256),
        planned_frame_sha256: Some(planned.frame_sha256),
        readback_frame_sha256: Some(planned.frame_sha256),
        write_attempted: true,
        write_completed: true,
        readback_completed: true,
        readback_matches_planned: true,
        reparse_valid: reparsed,
        span_in_bounds: true,
        action_kind: Some(action_kind),
        restart_target_restorable,
        disable_target_is_lifeline_endpoint: false,
        disable_target_core_owned: false,
        disable_target_known_disablable: true,
        grants_new_capability: false,
        trust_tier: Some(RECOVERY_ACTION_EXPECTED_TRUST_TIER),
        owner_sealed: false,
        persistence_claimed: false,
    };
    if let Some(mutation) = mutation {
        match mutation {
            RecoverySelftestMutation::WrongSchema => {
                input.record_schema = Some("raios.durable_record.v0")
            }
            RecoverySelftestMutation::WrongTarget => {
                input.target_id = Some("append.promotion_transaction.seed_data")
            }
            // Retargeted at M8B-2a: "restart_last_good" is now VALID, so this
            // out-of-scope selftest must use a still-invalid kind or it inverts.
            RecoverySelftestMutation::WrongActionKind => input.action_kind = Some("rollback"),
            RecoverySelftestMutation::LifelineEndpoint => {
                input.disable_target_is_lifeline_endpoint = true
            }
            RecoverySelftestMutation::CoreOwned => input.disable_target_core_owned = true,
            RecoverySelftestMutation::TargetUnknown => {
                input.disable_target_known_disablable = false
            }
            RecoverySelftestMutation::GrantsCapability => input.grants_new_capability = true,
        }
    }
    let decision = evaluate_scoped_recovery_action_append(&input);
    (decision.status, decision.reason, reparsed)
}

fn recovery_action_selftest_record() -> RecoveryActionRecord {
    RecoveryActionRecord {
        action_kind: RECOVERY_ACTION_EXPECTED_ACTION_KIND,
        disable_target_id: "svc.demo.echo",
        disable_target_is_lifeline_endpoint: false,
        disable_target_core_owned: false,
        disable_target_known_disablable: true,
        grants_new_capability: false,
        restores_known_good: false,
        restart_target_restorable: false,
    }
}

fn record_recovery_action_selftest_case(case: &RecoveryActionSelftestCase) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(case.name)),
        f("expected_status", s(case.expected_status)),
        f("expected_reason", s(case.expected_reason)),
        f("actual_status", s(case.actual_status)),
        f("actual_reason", s(case.actual_reason)),
        f("reparsed", b(case.reparsed)),
        f("passed", b(case.passed)),
    ])
}

fn emit_append_record(
    evidence: &DurableRecordLogScanEvidence,
    planned: Option<&PlannedAppend>,
    write: Option<&ahci::ReclogAppendWriteEvidence>,
    readback_sha256: Option<[u8; 32]>,
    reparse_valid: bool,
    durable_append: &'static str,
    reason: &'static str,
    authority: &'static str,
    after: Option<&DurableRecordLogScanEvidence>,
) {
    let mut fields = append_fields_base(evidence, durable_append, reason, authority);
    if let Some(planned) = planned {
        fields.push(f("seq", V::U64(planned.seq)));
        fields.push(f("write_offset", V::U64(planned.write_offset)));
        fields.push(f("frame_len", V::U64(planned.frame_len)));
        fields.push(f("payload_sha256", V::Sha256(planned.payload_sha256)));
        fields.push(f("frame_sha256", V::Sha256(planned.frame_sha256)));
    } else {
        fields.push(f("seq", V::Null));
        fields.push(f("write_offset", V::Null));
        fields.push(f("frame_len", V::Null));
        fields.push(f("payload_sha256", V::Null));
        fields.push(f("frame_sha256", V::Null));
    }
    fields.push(f("readback_sha256", record_sha_or_null(readback_sha256)));
    fields.push(f("reparse_valid", b(reparse_valid)));
    fields.push(f(
        "io_reason",
        s(write.map(|write| write.reason).unwrap_or(evidence.reason)),
    ));
    fields.push(f(
        "write_attempted",
        b(write.map(|write| write.write_attempted).unwrap_or(false)),
    ));
    fields.push(f(
        "write_completed",
        b(write.map(|write| write.write_completed).unwrap_or(false)),
    ));
    fields.push(f(
        "readback_completed",
        b(write.map(|write| write.readback_completed).unwrap_or(false)),
    ));
    fields.push(f(
        "readback_matches_planned",
        b(readback_sha256 == planned.map(|planned| planned.frame_sha256)),
    ));
    fields.push(f(
        "span_in_bounds",
        b(write.map(|write| write.span_in_bounds).unwrap_or(false)),
    ));
    fields.push(f(
        "writer_absolute_start_lba",
        write
            .map(|write| V::U64(write.absolute_start_lba))
            .unwrap_or(V::Null),
    ));
    fields.push(f(
        "writer_reclog_lba_count",
        write
            .map(|write| V::U64(write.reclog_lba_count))
            .unwrap_or(V::Null),
    ));
    fields.push(f(
        "writer_reclog_byte_count",
        write
            .map(|write| V::U64(write.byte_count))
            .unwrap_or(V::Null),
    ));
    fields.push(f(
        "writer_write_offset",
        write
            .map(|write| V::U64(write.write_offset))
            .unwrap_or(V::Null),
    ));
    fields.push(f(
        "reclog_absolute_start_lba",
        V::U64(evidence.reclog_absolute_start_lba),
    ));
    fields.push(f("reclog_lba_count", V::U64(evidence.reclog_lba_count)));
    fields.push(f("reclog_byte_count", V::U64(evidence.reclog_byte_count)));
    fields.push(f(
        "tail_seq_after",
        after
            .map(|after| V::U64(after.scan.tail_seq))
            .unwrap_or(V::Null),
    ));
    fields.push(f(
        "count_after",
        after
            .map(|after| V::U64(after.scan.count))
            .unwrap_or(V::Null),
    ));

    begin_response(APPEND_METHOD);
    emit_record_fields(fields, 6);
    end_response(APPEND_METHOD);
}

fn append_fields_base(
    evidence: &DurableRecordLogScanEvidence,
    durable_append: &'static str,
    reason: &'static str,
    authority: &'static str,
) -> Vec<Field<'static>> {
    vec![
        f("schema", s(APPEND_SCHEMA)),
        f("id", s(APPEND_ID)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("query_method", s(APPEND_METHOD)),
        f("durable_append", s(durable_append)),
        f("performed", b(durable_append == "appended")),
        f("reason", s(reason)),
        f("authority", s(authority)),
        f("target_id", s(EXPECTED_TARGET_ID)),
        f("record_schema", s(EXPECTED_RECORD_SCHEMA)),
        f("region_marker", s(EXPECTED_REGION_MARKER)),
        f("status", s(evidence.scan.status())),
        f("tail_seq_before", V::U64(evidence.scan.tail_seq)),
        f("count_before", V::U64(evidence.scan.count)),
    ]
}

fn current_boot_reclog_scan() -> DurableRecordLogScanEvidence {
    let Some(controller) = pci::find_mass_storage_controller() else {
        return DurableRecordLogScanEvidence::absent("ahci_controller_not_observed");
    };
    let read = ahci::read_persist_reclog_region(controller);
    let source_port_index = if read.layout.port_index == u8::MAX {
        None
    } else {
        Some(read.layout.port_index)
    };
    let scan = match read.bytes.as_deref() {
        Some(bytes) => scan_reclog(bytes),
        None => RecordLogScan::not_scanned(read.reason),
    };

    DurableRecordLogScanEvidence {
        reason: read.reason,
        controller: Some(controller),
        controller_present: read.layout.controller_present,
        source_port_index,
        layout_status: read.layout.status(),
        data_layout_status: read.layout.data.status.as_str(),
        read_attempted: read.read_attempted,
        read_completed: read.read_completed,
        region_bounds_valid: read.region_bounds_valid,
        reclog_absolute_start_lba: read.absolute_start_lba,
        reclog_lba_count: read.lba_count,
        reclog_byte_count: read.byte_count,
        scan,
    }
}

// --- M9A-2b durable memory record append (raios.memory_record.v0) -----------------
//
// Appends the ONE system-authored `raios.memory_record.v0` durable memory fact
// through the SAME shared reclog gauntlet as every other durable writer in this file
// (scan -> plan -> ahci write_readback -> readback-sha -> reparse ->
// evaluate_scoped_memory_record_append -> rescan), mirroring `append_recovery_load`
// EXACTLY, with ONE addition: a per-boot RAM write-quota reserved BEFORE the
// plan/write and released on any post-reserve denial (never held past a denied
// attempt, never released after success). `MemoryRecord::new` (raios-core) already
// fails closed on `secret` classification and unknown `kind`, so this function is
// never called with an invalid record; `evaluate_scoped_memory_record_append`
// re-checks those invariants anyway (deny-in-depth) plus the reclog gauntlet, owner
// trust tier, and the quota gate itself. Quota/secret/bad-kind denials are exercised
// ONLY by the synthetic `memory.record_log_append_selftest` (which calls the
// evaluator directly on synthetic input and never reaches this function, never
// touches the disk), so there is no self-recursive durable write on denial.

/// Per-boot RAM-only durable-memory write quota. A fresh static reset to full every
/// boot by construction -- never read from or written to disk. Bounds BOTH the
/// record count and a nominal one-sector (512 B) charge per reservation, since every
/// frame this driver plans rounds up to at least one reclog sector.
const MEMORY_WRITE_QUOTA_MAX_RECORDS: u32 = 128;
const MEMORY_WRITE_QUOTA_MAX_BYTES: u64 = 32 * 1024;
const MEMORY_RECORD_APPEND_SUCCESS_AUTHORITY: &str = "scoped_memory_record_append_authorized";

/// Informational budget constants surfaced by the selftest response so the VM
/// profile can assert the quota is honestly per-boot bounded.
pub(crate) const MEMORY_WRITE_QUOTA_BUDGET_RECORDS: u64 = MEMORY_WRITE_QUOTA_MAX_RECORDS as u64;
pub(crate) const MEMORY_WRITE_QUOTA_BUDGET_BYTES: u64 = MEMORY_WRITE_QUOTA_MAX_BYTES;

struct MemoryWriteQuota {
    records_remaining: u32,
    bytes_remaining: u64,
}

impl MemoryWriteQuota {
    const fn new() -> Self {
        Self {
            records_remaining: MEMORY_WRITE_QUOTA_MAX_RECORDS,
            bytes_remaining: MEMORY_WRITE_QUOTA_MAX_BYTES,
        }
    }
}

static MEMORY_WRITE_QUOTA: Mutex<MemoryWriteQuota> = Mutex::new(MemoryWriteQuota::new());

/// Reserves one record + one sector charge against the per-boot quota. Fails closed
/// (returns `false`, reserves nothing) once either budget is exhausted.
fn memory_write_quota_try_reserve() -> bool {
    let mut quota = MEMORY_WRITE_QUOTA.lock();
    if quota.records_remaining == 0 || quota.bytes_remaining < RECLOG_SECTOR_SIZE as u64 {
        return false;
    }
    quota.records_remaining -= 1;
    quota.bytes_remaining -= RECLOG_SECTOR_SIZE as u64;
    true
}

/// Gives back a reservation after a post-reserve denial, so a transient AHCI/plan
/// failure never permanently burns quota.
fn memory_write_quota_release() {
    let mut quota = MEMORY_WRITE_QUOTA.lock();
    quota.records_remaining = quota
        .records_remaining
        .saturating_add(1)
        .min(MEMORY_WRITE_QUOTA_MAX_RECORDS);
    quota.bytes_remaining = quota
        .bytes_remaining
        .saturating_add(RECLOG_SECTOR_SIZE as u64)
        .min(MEMORY_WRITE_QUOTA_MAX_BYTES);
}

/// TEST-ONLY live probe of the REAL per-boot RAM quota (M9A-2b M1): reserves
/// against the actual `MEMORY_WRITE_QUOTA` static until `try_reserve` returns false
/// (counting the successful reservations), then releases exactly that many, and
/// finally confirms a fresh reservation succeeds again (giving it straight back).
/// This proves the live gate genuinely exhausts AND refunds — the one primitive
/// this slice adds over its `append_recovery_load` reference — WITHOUT performing a
/// single durable write. It is order-independent and non-perturbing: it restores the
/// quota to exactly its prior level, so a real `append_memory_record` in the same
/// boot is unaffected. Returns `(reservations_until_exhausted, restored_ok)`.
pub(crate) fn memory_write_quota_probe_exhaustion() -> (u32, bool) {
    let mut reserved = 0u32;
    while memory_write_quota_try_reserve() {
        reserved = reserved.saturating_add(1);
        if reserved == u32::MAX {
            break;
        }
    }
    let mut released = 0u32;
    while released < reserved {
        memory_write_quota_release();
        released = released.saturating_add(1);
    }
    let restored = memory_write_quota_try_reserve();
    if restored {
        memory_write_quota_release();
    }
    (reserved, restored)
}

#[derive(Clone, Copy)]
pub(crate) struct MemoryRecordAppendEvidence {
    pub(crate) durable_append: &'static str,
    pub(crate) performed: bool,
    pub(crate) reason: &'static str,
    pub(crate) authority: &'static str,
    pub(crate) seq: Option<u64>,
    pub(crate) write_offset: Option<u64>,
    pub(crate) frame_len: Option<u64>,
    pub(crate) payload_sha256: Option<[u8; 32]>,
    pub(crate) frame_sha256: Option<[u8; 32]>,
    pub(crate) readback_sha256: Option<[u8; 32]>,
    pub(crate) reparse_valid: bool,
    pub(crate) tail_seq_before: Option<u64>,
    pub(crate) count_before: Option<u64>,
    pub(crate) tail_seq_after: Option<u64>,
    pub(crate) count_after: Option<u64>,
    pub(crate) record_id: &'static str,
    pub(crate) record_kind: &'static str,
    pub(crate) record_classification: &'static str,
    pub(crate) record_authority: &'static str,
    pub(crate) record_schema: &'static str,
    pub(crate) region_marker: &'static str,
    pub(crate) target_id: &'static str,
    pub(crate) owner_sealed: bool,
    pub(crate) persistence_claimed: bool,
    pub(crate) trust_tier: &'static str,
}

/// Appends `record` through the shared reclog gauntlet, authorized ONLY by
/// `evaluate_scoped_memory_record_append`. The record is required `'static` because
/// this slice's one caller (`memory_store.rs`) always builds it from fully-'static
/// strings (per the M9A-2b packet), which keeps this evidence struct itself
/// borrow-free and cheap to render.
pub(crate) fn append_memory_record(record: &MemoryRecord<'static>) -> MemoryRecordAppendEvidence {
    if !matches!(
        super::boot_control::current_boot_posture(),
        BootPosture::Normal | BootPosture::Probation
    ) {
        return memory_record_append_denied(record, "boot_control_safe_mode");
    }

    if !memory_write_quota_try_reserve() {
        return memory_record_append_denied(record, "memory_write_quota_exhausted");
    }

    let before = current_boot_reclog_scan();
    let payload = memory_record_payload_bytes(record);
    let planned = match plan_reclog_append(&before.scan, &payload, before.reclog_byte_count) {
        Ok(planned) => planned,
        Err(denied) => {
            memory_write_quota_release();
            return memory_record_append_evidence(
                record,
                "capability_denied",
                denied.reason(),
                "evidence_only",
                Some(&before),
                None,
                None,
                false,
                None,
            );
        }
    };

    let Some(controller) = before.controller else {
        memory_write_quota_release();
        return memory_record_append_evidence(
            record,
            "capability_denied",
            "ahci_controller_not_observed",
            "evidence_only",
            Some(&before),
            Some(&planned),
            None,
            false,
            None,
        );
    };

    let write = unsafe {
        ahci::write_readback_reclog_append(controller, planned.write_offset, &planned.frame)
    };
    let readback_sha256 = write.readback.as_deref().map(sha256_bytes);
    let reparse_valid = write
        .readback
        .as_deref()
        .map(|bytes| parse_reclog_frame(bytes, 0, planned.seq, planned.prev_frame_sha256).is_ok())
        .unwrap_or(false);
    let decision = evaluate_scoped_memory_record_append(&ScopedMemoryRecordAppendInput {
        method: Some(MEMORY_EXPECTED_METHOD),
        target_id: Some(MEMORY_EXPECTED_TARGET_ID),
        record_schema: Some(MEMORY_EXPECTED_RECORD_SCHEMA),
        region_marker: Some(MEMORY_EXPECTED_REGION_MARKER),
        frame_len: Some(planned.frame_len),
        write_offset: Some(planned.write_offset),
        reclog_byte_count: Some(before.reclog_byte_count),
        absolute_start_lba: Some(before.reclog_absolute_start_lba),
        reclog_lba_count: Some(before.reclog_lba_count),
        seq: Some(planned.seq),
        tail_seq: Some(before.scan.tail_seq),
        count: Some(before.scan.count),
        prev_frame_sha256: Some(planned.prev_frame_sha256),
        tail_frame_sha256: before.scan.tail_frame_sha256,
        payload_sha256: Some(planned.payload_sha256),
        planned_payload_sha256: Some(planned.payload_sha256),
        planned_frame_sha256: Some(planned.frame_sha256),
        readback_frame_sha256: readback_sha256,
        write_attempted: write.write_attempted,
        write_completed: write.write_completed,
        readback_completed: write.readback_completed,
        readback_matches_planned: readback_sha256 == Some(planned.frame_sha256),
        reparse_valid,
        span_in_bounds: write.span_in_bounds,
        classification: Some(record.classification.as_str()),
        kind: Some(record.kind.as_str()),
        trust_tier: Some(MEMORY_EXPECTED_TRUST_TIER),
        owner_sealed: false,
        persistence_claimed: false,
        quota_ok: true,
    });

    if !decision.performed {
        memory_write_quota_release();
        return memory_record_append_evidence(
            record,
            "capability_denied",
            decision.reason,
            "evidence_only",
            Some(&before),
            Some(&planned),
            readback_sha256,
            reparse_valid,
            None,
        );
    }

    let after = current_boot_reclog_scan();
    let rescan_ok = before
        .scan
        .count
        .checked_add(1)
        .map(|count| after.scan.count == count)
        .unwrap_or(false)
        && after.scan.tail_seq == planned.seq
        && after.scan.tail_frame_sha256 == Some(planned.frame_sha256);
    if !rescan_ok {
        memory_write_quota_release();
        return memory_record_append_evidence(
            record,
            "capability_denied",
            "post_append_rescan_mismatch",
            "evidence_only",
            Some(&before),
            Some(&planned),
            readback_sha256,
            reparse_valid,
            Some(&after),
        );
    }

    memory_record_append_evidence(
        record,
        "appended",
        decision.reason,
        MEMORY_RECORD_APPEND_SUCCESS_AUTHORITY,
        Some(&before),
        Some(&planned),
        readback_sha256,
        true,
        Some(&after),
    )
}

fn memory_record_append_denied(
    record: &MemoryRecord<'static>,
    reason: &'static str,
) -> MemoryRecordAppendEvidence {
    memory_record_append_evidence(
        record,
        "capability_denied",
        reason,
        "evidence_only",
        None,
        None,
        None,
        false,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn memory_record_append_evidence(
    record: &MemoryRecord<'static>,
    durable_append: &'static str,
    reason: &'static str,
    authority: &'static str,
    before: Option<&DurableRecordLogScanEvidence>,
    planned: Option<&PlannedAppend>,
    readback_sha256: Option<[u8; 32]>,
    reparse_valid: bool,
    after: Option<&DurableRecordLogScanEvidence>,
) -> MemoryRecordAppendEvidence {
    MemoryRecordAppendEvidence {
        durable_append,
        performed: durable_append == "appended",
        reason,
        authority,
        seq: planned.map(|planned| planned.seq),
        write_offset: planned.map(|planned| planned.write_offset),
        frame_len: planned.map(|planned| planned.frame_len),
        payload_sha256: planned.map(|planned| planned.payload_sha256),
        frame_sha256: planned.map(|planned| planned.frame_sha256),
        readback_sha256,
        reparse_valid,
        tail_seq_before: before.map(|before| before.scan.tail_seq),
        count_before: before.map(|before| before.scan.count),
        tail_seq_after: after.map(|after| after.scan.tail_seq),
        count_after: after.map(|after| after.scan.count),
        record_id: record.id,
        record_kind: record.kind.as_str(),
        record_classification: record.classification.as_str(),
        record_authority: record.authority,
        record_schema: MEMORY_EXPECTED_RECORD_SCHEMA,
        region_marker: MEMORY_EXPECTED_REGION_MARKER,
        target_id: MEMORY_EXPECTED_TARGET_ID,
        owner_sealed: false,
        persistence_claimed: false,
        trust_tier: MEMORY_EXPECTED_TRUST_TIER,
    }
}

/// Payload bytes are `write_json(record.to_record_value())` -- IDENTICAL to
/// `recovery_load_payload_bytes`'s pattern -- so the reclog's `payload_sha256` over
/// these exact bytes equals `record.record_sha256()` (both hash the same
/// `write_json` rendering of the same `Value`).
fn memory_record_payload_bytes(record: &MemoryRecord<'static>) -> Vec<u8> {
    let mut sink = VecSink(Vec::new());
    write_json(&record.to_record_value(), &mut sink, 0);
    sink.0
}
