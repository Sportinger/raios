use alloc::{vec, vec::Vec};
use core::str;

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
    },
    promotion_attestation::{
        PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256, PROMOTION_AUTHORITY_IS_PLACEHOLDER,
    },
    record::{write_json, Field, Value as V},
    scoped_promotion_transaction_append::{
        evaluate_scoped_promotion_transaction_append, ScopedPromotionTransactionAppendInput,
        EXPECTED_METHOD as PROMOTION_EXPECTED_METHOD,
        EXPECTED_RECORD_SCHEMA as PROMOTION_EXPECTED_RECORD_SCHEMA,
        EXPECTED_REGION_MARKER as PROMOTION_EXPECTED_REGION_MARKER,
        EXPECTED_TARGET_ID as PROMOTION_EXPECTED_TARGET_ID,
        EXPECTED_TRUST_TIER as PROMOTION_EXPECTED_TRUST_TIER,
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
