use alloc::{vec, vec::Vec};
use core::str;

use crate::{
    agent_protocol::durable_store,
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_sha_or_null, record_str as s,
    },
    ahci,
    granted_candidate_service::PromotionRecord,
    module_candidate_intake::RetainedExternalWasmCandidate,
    pci,
};
use raios_core::{
    artifact_blob_frame::{
        build_artifact_blob_frame, parse_artifact_blob_frame, plan_artifact_blob_write,
        ARTIFACT_BLOB_FRAME_HEADER_LEN, ARTIFACT_BLOB_MAGIC,
    },
    boot_control::BootPosture,
    durable_record_frame::{
        parse_reclog_frame, plan_reclog_append, scan_reclog, PlannedAppend, RecordLogScan,
        RECLOG_FRAME_HEADER_LEN,
    },
    promotion_attestation::PROMOTION_AUTHORITY_IS_PLACEHOLDER,
    record::{write_json, Value as V},
    scoped_artifact_persist_append::{
        evaluate_scoped_artifact_persist_append, ScopedArtifactPersistAppendInput,
        EXPECTED_METHOD as ARTIFACT_PERSIST_EXPECTED_METHOD,
        EXPECTED_RECORD_SCHEMA as ARTIFACT_PERSIST_EXPECTED_RECORD_SCHEMA,
        EXPECTED_REGION_MARKER as ARTIFACT_PERSIST_EXPECTED_REGION_MARKER,
        EXPECTED_TARGET_ID as ARTIFACT_PERSIST_EXPECTED_TARGET_ID,
        EXPECTED_TRUST_TIER as ARTIFACT_PERSIST_EXPECTED_TRUST_TIER,
    },
    scoped_artifact_store_blob::{
        evaluate_scoped_artifact_store_blob, ScopedArtifactStoreBlobInput,
        EXPECTED_METHOD as ARTIFACT_BLOB_EXPECTED_METHOD,
        EXPECTED_RECORD_SCHEMA as ARTIFACT_BLOB_EXPECTED_RECORD_SCHEMA,
        EXPECTED_REGION_MARKER as ARTIFACT_BLOB_EXPECTED_REGION_MARKER,
        EXPECTED_TARGET_ID as ARTIFACT_BLOB_EXPECTED_TARGET_ID,
        EXPECTED_TRUST_TIER as ARTIFACT_BLOB_EXPECTED_TRUST_TIER,
    },
    sha256_bytes, ByteSink,
};

const ARTIFACT_PERSIST_RESPONSE_SCHEMA: &str = "raios.artifact_persist_append.v0";
const ARTIFACT_PERSIST_RESPONSE_ID: &str =
    "artifact_persist_append.seed_data.origin_boot.svc.dev.granted_candidate.v0";
const ARTIFACT_PERSIST_RECORD_ID: &str =
    "artifact_persist.origin_boot.svc.dev.granted_candidate.v0";
const ARTIFACT_STORE_SCAN_METHOD: &str = "artifact.store_scan";
const ARTIFACT_STORE_SCAN_SCHEMA: &str = "raios.artifact_store_scan.v0";
const ARTIFACT_STORE_SCAN_ID: &str = "artifact_store_scan.seed_data.current_boot.v0";
const ARTIFACT_STORE_SELFTEST_METHOD: &str = "module.artifact_store_selftest";
const ARTIFACT_STORE_SELFTEST_SCHEMA: &str = "raios.artifact_store_selftest.v0";
const ARTIFACT_RECORD_SCOPE: &str = "origin_boot";
const GRANTED_CANDIDATE_IMPORT_SET_CANONICAL: &[u8] = b"env.log\nenv.counter_get\n";
const ARTSTOR_SCAN_WINDOW_BYTES: u64 = 512;

#[derive(Clone, Copy)]
pub(crate) struct ArtifactPersistEvidence {
    pub(crate) performed: bool,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) authority: &'static str,
    pub(crate) blob_write_status: &'static str,
    pub(crate) artstor_blob_offset: Option<u64>,
    pub(crate) artstor_blob_len: Option<u64>,
    pub(crate) blob_frame_sha256: Option<[u8; 32]>,
    pub(crate) artifact_sha256: Option<[u8; 32]>,
    pub(crate) artifact_persist_seq: Option<u64>,
    pub(crate) artifact_persist_frame_sha256: Option<[u8; 32]>,
    pub(crate) artifact_persist_readback_sha256: Option<[u8; 32]>,
    pub(crate) blob_written_to_disk: bool,
    pub(crate) authorizes_load: bool,
    pub(crate) owner_sealed: bool,
    pub(crate) cross_reboot_proven: bool,
    pub(crate) persistence_claimed: bool,
}

impl ArtifactPersistEvidence {
    fn denied(reason: &'static str) -> Self {
        Self {
            performed: false,
            status: "capability_denied",
            reason,
            authority: "evidence_only",
            blob_write_status: "not_attempted",
            artstor_blob_offset: None,
            artstor_blob_len: None,
            blob_frame_sha256: None,
            artifact_sha256: None,
            artifact_persist_seq: None,
            artifact_persist_frame_sha256: None,
            artifact_persist_readback_sha256: None,
            blob_written_to_disk: false,
            authorizes_load: false,
            owner_sealed: false,
            cross_reboot_proven: false,
            persistence_claimed: false,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ArtifactPersistRecord {
    pub(crate) seq: u64,
    pub(crate) reclog_offset: u64,
    pub(crate) artstor_blob_offset: u64,
    pub(crate) artstor_blob_len: u64,
    pub(crate) artstor_blob_frame_sha256: [u8; 32],
    pub(crate) artifact_sha256: [u8; 32],
    pub(crate) manifest_hash: [u8; 32],
    pub(crate) vm_report_hash: [u8; 32],
    pub(crate) grant_hash: [u8; 32],
    pub(crate) promotion_transaction_sha256: [u8; 32],
    pub(crate) authorizes_load: bool,
}

pub(crate) struct ReclogScanEvidence {
    pub(crate) reason: &'static str,
    pub(crate) read_completed: bool,
    pub(crate) reclog_absolute_start_lba: u64,
    pub(crate) reclog_lba_count: u64,
    pub(crate) reclog_byte_count: u64,
    pub(crate) scan: RecordLogScan,
    pub(crate) bytes: Option<Vec<u8>>,
}

pub(crate) struct VerifiedArtifactPayload {
    pub(crate) blob_bytes: Vec<u8>,
    pub(crate) payload: Vec<u8>,
}

struct VecSink(Vec<u8>);

impl ByteSink for VecSink {
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }
}

pub(crate) fn granted_candidate_import_set_hash() -> [u8; 32] {
    sha256_bytes(GRANTED_CANDIDATE_IMPORT_SET_CANONICAL)
}

pub(crate) fn artifact_persist_denied(reason: &'static str) -> ArtifactPersistEvidence {
    ArtifactPersistEvidence::denied(reason)
}

pub(crate) fn artifact_persist_evidence_record(evidence: ArtifactPersistEvidence) -> V<'static> {
    V::Object(vec![
        f("schema", s(ARTIFACT_PERSIST_RESPONSE_SCHEMA)),
        f("id", s(ARTIFACT_PERSIST_RESPONSE_ID)),
        f("method", s(ARTIFACT_PERSIST_EXPECTED_METHOD)),
        f("scope", s(ARTIFACT_RECORD_SCOPE)),
        f("classification", s("local_only")),
        f(
            "code",
            s(if evidence.performed {
                "ok"
            } else {
                "capability_denied"
            }),
        ),
        f("status", s(evidence.status)),
        f("performed", b(evidence.performed)),
        f("reason", s(evidence.reason)),
        f("authority", s(evidence.authority)),
        f("blob_write_status", s(evidence.blob_write_status)),
        f("target_id", s(ARTIFACT_PERSIST_EXPECTED_TARGET_ID)),
        f("record_schema", s(ARTIFACT_PERSIST_EXPECTED_RECORD_SCHEMA)),
        f("region_marker", s(ARTIFACT_PERSIST_EXPECTED_REGION_MARKER)),
        f(
            "artstor_blob_offset",
            optional_u64(evidence.artstor_blob_offset),
        ),
        f("artstor_blob_len", optional_u64(evidence.artstor_blob_len)),
        f(
            "artstor_blob_frame_sha256",
            record_sha_or_null(evidence.blob_frame_sha256),
        ),
        f(
            "artifact_sha256",
            record_sha_or_null(evidence.artifact_sha256),
        ),
        f(
            "artifact_persist_seq",
            optional_u64(evidence.artifact_persist_seq),
        ),
        f(
            "artifact_persist_frame_sha256",
            record_sha_or_null(evidence.artifact_persist_frame_sha256),
        ),
        f(
            "artifact_persist_readback_sha256",
            record_sha_or_null(evidence.artifact_persist_readback_sha256),
        ),
        f("blob_written_to_disk", b(evidence.blob_written_to_disk)),
        f("authorizes_load", b(evidence.authorizes_load)),
        f("owner_sealed", b(evidence.owner_sealed)),
        f("cross_reboot_proven", b(evidence.cross_reboot_proven)),
        f("persistence_claimed", b(evidence.persistence_claimed)),
    ])
}

pub(crate) fn persist_promoted_artifact(
    promotion: PromotionRecord,
    durable_promotion_transaction: &durable_store::PromotionTransactionAppendEvidence,
    retained: Option<&RetainedExternalWasmCandidate>,
    manifest_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    computed_grant_hash: [u8; 32],
    service_id: &'static str,
    import_set_hash: [u8; 32],
) -> ArtifactPersistEvidence {
    if !matches!(
        super::boot_control::current_boot_posture(),
        BootPosture::Normal | BootPosture::Probation
    ) {
        return ArtifactPersistEvidence::denied("boot_control_safe_mode");
    }
    if !durable_promotion_transaction.performed
        || durable_promotion_transaction.transaction_kind != "promote"
    {
        return ArtifactPersistEvidence::denied("promotion_transaction_not_verified_this_boot");
    }
    let Some(promotion_transaction_sha256) = durable_promotion_transaction.frame_sha256 else {
        return ArtifactPersistEvidence::denied("promotion_transaction_not_verified_this_boot");
    };
    let Some(retained) = retained else {
        return ArtifactPersistEvidence::denied("retained_artifact_missing");
    };
    if !retained.wasm_valid {
        return ArtifactPersistEvidence::denied("retained_artifact_wasm_invalid");
    }
    if retained.sha256 != promotion.artifact_hash {
        return ArtifactPersistEvidence::denied("retained_artifact_hash_mismatch");
    }

    let frame = build_artifact_blob_frame(&retained.bytes);
    let Ok(parsed_frame) = parse_artifact_blob_frame(&frame, 0) else {
        return ArtifactPersistEvidence::denied("artifact_blob_frame_invalid");
    };
    if parsed_frame.payload_sha256 != retained.sha256 {
        return ArtifactPersistEvidence::denied("artifact_blob_payload_hash_mismatch");
    }

    let Some(controller) = pci::find_mass_storage_controller() else {
        return ArtifactPersistEvidence::denied("ahci_controller_not_observed");
    };
    let artstor_probe = ahci::read_persist_artstor_region(controller, 0, ARTSTOR_SCAN_WINDOW_BYTES);
    if !artstor_probe.read_completed {
        return ArtifactPersistEvidence::denied(artstor_probe.reason);
    }
    let _ = (
        artstor_probe.layout.controller_present,
        artstor_probe.read_offset,
        artstor_probe.read_len,
    );
    let before = current_boot_reclog_scan(controller);
    if !before.scan.full_region_valid {
        return ArtifactPersistEvidence::denied("reclog_not_appendable");
    }
    let records = before
        .bytes
        .as_deref()
        .map(artifact_persist_records_from_reclog)
        .unwrap_or_default();
    let next_free = match next_free_artstor_offset(&records) {
        Ok(offset) => offset,
        Err(reason) => return ArtifactPersistEvidence::denied(reason),
    };
    let planned_blob =
        match plan_artifact_blob_write(next_free, &retained.bytes, artstor_probe.byte_count) {
            Ok(planned) => planned,
            Err(denied) => return ArtifactPersistEvidence::denied(denied.reason()),
        };

    let blob_write = unsafe {
        ahci::write_readback_artstor_blob(
            controller,
            planned_blob.write_offset,
            &planned_blob.frame,
        )
    };
    let readback_blob_sha256 = blob_write.readback.as_deref().map(sha256_bytes);
    let _ = (
        blob_write.absolute_start_lba,
        blob_write.artstor_lba_count,
        blob_write.byte_count,
        blob_write.write_offset,
    );
    let readback_blob_parse = blob_write
        .readback
        .as_deref()
        .and_then(|bytes| parse_artifact_blob_frame(bytes, 0).ok());
    let blob_decision = evaluate_scoped_artifact_store_blob(&ScopedArtifactStoreBlobInput {
        method: Some(ARTIFACT_BLOB_EXPECTED_METHOD),
        target_id: Some(ARTIFACT_BLOB_EXPECTED_TARGET_ID),
        record_schema: Some(ARTIFACT_BLOB_EXPECTED_RECORD_SCHEMA),
        region_marker: Some(ARTIFACT_BLOB_EXPECTED_REGION_MARKER),
        frame_len: Some(planned_blob.frame_len),
        write_offset: Some(planned_blob.write_offset),
        artstor_byte_count: Some(artstor_probe.byte_count),
        artstor_absolute_start_lba: Some(artstor_probe.absolute_start_lba),
        artstor_lba_count: Some(artstor_probe.lba_count),
        payload_sha256: readback_blob_parse.map(|frame| frame.payload_sha256),
        planned_payload_sha256: Some(planned_blob.payload_sha256),
        planned_frame_sha256: Some(planned_blob.frame_sha256),
        readback_frame_sha256: readback_blob_sha256,
        write_attempted: blob_write.write_attempted,
        write_completed: blob_write.write_completed,
        readback_completed: blob_write.readback_completed,
        readback_matches_planned: readback_blob_sha256 == Some(planned_blob.frame_sha256),
        reparse_valid: readback_blob_parse.is_some(),
        span_in_bounds: blob_write.span_in_bounds,
        signature_verified: durable_promotion_transaction.signature_verified,
        grant_binds_capability: durable_promotion_transaction.grant_binds_capability,
        trust_tier: Some(ARTIFACT_BLOB_EXPECTED_TRUST_TIER),
        owner_sealed: false,
        promotion_authority_is_placeholder: PROMOTION_AUTHORITY_IS_PLACEHOLDER,
        promotion_transaction_verified: true,
        persistence_claimed: false,
    });
    if !blob_decision.performed {
        return blob_denied_evidence(
            blob_decision.reason,
            &planned_blob,
            readback_blob_sha256,
            blob_write.readback_completed,
        );
    }

    let payload = artifact_persist_payload_bytes(
        planned_blob.write_offset,
        planned_blob.frame_len,
        planned_blob.frame_sha256,
        planned_blob.payload_sha256,
        manifest_hash,
        vm_report_hash,
        computed_grant_hash,
        service_id,
        import_set_hash,
        promotion_transaction_sha256,
    );
    let planned_append = match plan_reclog_append(&before.scan, &payload, before.reclog_byte_count)
    {
        Ok(planned) => planned,
        Err(denied) => {
            return blob_denied_evidence(denied.reason(), &planned_blob, readback_blob_sha256, true)
        }
    };
    let record_write = unsafe {
        ahci::write_readback_reclog_append(
            controller,
            planned_append.write_offset,
            &planned_append.frame,
        )
    };
    let readback_record_sha256 = record_write.readback.as_deref().map(sha256_bytes);
    let record_reparse_valid = record_write
        .readback
        .as_deref()
        .map(|bytes| {
            parse_reclog_frame(
                bytes,
                0,
                planned_append.seq,
                planned_append.prev_frame_sha256,
            )
            .is_ok()
        })
        .unwrap_or(false);
    let append_decision =
        evaluate_scoped_artifact_persist_append(&ScopedArtifactPersistAppendInput {
            method: Some(ARTIFACT_PERSIST_EXPECTED_METHOD),
            target_id: Some(ARTIFACT_PERSIST_EXPECTED_TARGET_ID),
            record_schema: Some(ARTIFACT_PERSIST_EXPECTED_RECORD_SCHEMA),
            region_marker: Some(ARTIFACT_PERSIST_EXPECTED_REGION_MARKER),
            frame_len: Some(planned_append.frame_len),
            write_offset: Some(planned_append.write_offset),
            reclog_byte_count: Some(before.reclog_byte_count),
            absolute_start_lba: Some(before.reclog_absolute_start_lba),
            reclog_lba_count: Some(before.reclog_lba_count),
            seq: Some(planned_append.seq),
            tail_seq: Some(before.scan.tail_seq),
            count: Some(before.scan.count),
            prev_frame_sha256: Some(planned_append.prev_frame_sha256),
            tail_frame_sha256: before.scan.tail_frame_sha256,
            payload_sha256: Some(planned_append.payload_sha256),
            planned_payload_sha256: Some(planned_append.payload_sha256),
            planned_frame_sha256: Some(planned_append.frame_sha256),
            readback_frame_sha256: readback_record_sha256,
            write_attempted: record_write.write_attempted,
            write_completed: record_write.write_completed,
            readback_completed: record_write.readback_completed,
            readback_matches_planned: readback_record_sha256 == Some(planned_append.frame_sha256),
            reparse_valid: record_reparse_valid,
            span_in_bounds: record_write.span_in_bounds,
            signature_verified: durable_promotion_transaction.signature_verified,
            grant_binds_capability: durable_promotion_transaction.grant_binds_capability,
            trust_tier: Some(ARTIFACT_PERSIST_EXPECTED_TRUST_TIER),
            owner_sealed: false,
            promotion_authority_is_placeholder: PROMOTION_AUTHORITY_IS_PLACEHOLDER,
            promotion_transaction_verified: true,
            persistence_claimed: false,
        });
    if !append_decision.performed {
        return append_denied_evidence(
            append_decision.reason,
            &planned_blob,
            &planned_append,
            readback_blob_sha256,
            readback_record_sha256,
        );
    }

    let after = current_boot_reclog_scan(controller);
    let rescan_ok = before
        .scan
        .count
        .checked_add(1)
        .map(|count| after.scan.count == count)
        .unwrap_or(false)
        && after.scan.tail_seq == planned_append.seq
        && after.scan.tail_frame_sha256 == Some(planned_append.frame_sha256);
    if !rescan_ok {
        return append_denied_evidence(
            "post_append_rescan_mismatch",
            &planned_blob,
            &planned_append,
            readback_blob_sha256,
            readback_record_sha256,
        );
    }

    ArtifactPersistEvidence {
        performed: true,
        status: "appended",
        reason: append_decision.reason,
        authority: "scoped_artifact_persist_append_authorized",
        blob_write_status: blob_decision.status,
        artstor_blob_offset: Some(planned_blob.write_offset),
        artstor_blob_len: Some(planned_blob.frame_len),
        blob_frame_sha256: Some(planned_blob.frame_sha256),
        artifact_sha256: Some(planned_blob.payload_sha256),
        artifact_persist_seq: Some(planned_append.seq),
        artifact_persist_frame_sha256: Some(planned_append.frame_sha256),
        artifact_persist_readback_sha256: readback_record_sha256,
        blob_written_to_disk: true,
        authorizes_load: false,
        owner_sealed: false,
        cross_reboot_proven: false,
        persistence_claimed: false,
    }
}

pub(crate) fn emit_artifact_store_scan() {
    let Some(controller) = pci::find_mass_storage_controller() else {
        emit_scan_record(
            "ahci_controller_not_observed",
            false,
            RecordLogScan::not_scanned("ahci_controller_not_observed"),
            Vec::new(),
            Vec::new(),
        );
        return;
    };
    let reclog = current_boot_reclog_scan(controller);
    let records = reclog
        .bytes
        .as_deref()
        .map(artifact_persist_records_from_reclog)
        .unwrap_or_default();
    let record_views = scan_record_views(controller, &records);
    let garbage = scan_garbage_blobs(controller, &records);
    emit_scan_record(
        reclog.reason,
        reclog.read_completed,
        reclog.scan,
        record_views,
        garbage,
    );
}

pub(crate) fn emit_artifact_store_selftest() {
    let cases = artifact_store_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);
    let case_records = cases.iter().map(record_selftest_case).collect();

    begin_response(ARTIFACT_STORE_SELFTEST_METHOD);
    emit_record_fields(
        vec![
            f("schema", s(ARTIFACT_STORE_SELFTEST_SCHEMA)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", b(false)),
            f("writes_persistent_state", b(false)),
            f("loads_artifact", b(false)),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
            f("cases", V::Array(case_records)),
            f("evidence_complete", b(true)),
        ],
        6,
    );
    end_response(ARTIFACT_STORE_SELFTEST_METHOD);
}

pub(crate) fn current_boot_reclog_scan(
    controller: pci::PciMassStorageController,
) -> ReclogScanEvidence {
    let read = ahci::read_persist_reclog_region(controller);
    let scan = match read.bytes.as_deref() {
        Some(bytes) => scan_reclog(bytes),
        None => RecordLogScan::not_scanned(read.reason),
    };
    ReclogScanEvidence {
        reason: read.reason,
        read_completed: read.read_completed,
        reclog_absolute_start_lba: read.absolute_start_lba,
        reclog_lba_count: read.lba_count,
        reclog_byte_count: read.byte_count,
        scan,
        bytes: read.bytes,
    }
}

fn artifact_persist_payload_bytes(
    artstor_blob_offset: u64,
    artstor_blob_len: u64,
    artstor_blob_frame_sha256: [u8; 32],
    artifact_sha256: [u8; 32],
    manifest_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    computed_grant_hash: [u8; 32],
    service_id: &'static str,
    import_set_hash: [u8; 32],
    promotion_transaction_sha256: [u8; 32],
) -> Vec<u8> {
    let record = V::Object(vec![
        f("schema", s(ARTIFACT_PERSIST_EXPECTED_RECORD_SCHEMA)),
        f("id", s(ARTIFACT_PERSIST_RECORD_ID)),
        f("scope", s(ARTIFACT_RECORD_SCOPE)),
        f("classification", s("local_only")),
        f("record_kind", s("artifact_persist")),
        f("artstor_blob_offset", V::U64(artstor_blob_offset)),
        f("artstor_blob_len", V::U64(artstor_blob_len)),
        f(
            "artstor_blob_frame_sha256",
            V::Sha256(artstor_blob_frame_sha256),
        ),
        f("artifact_sha256", V::Sha256(artifact_sha256)),
        f("manifest_hash", V::Sha256(manifest_hash)),
        f("vm_report_hash", V::Sha256(vm_report_hash)),
        f("grant_hash", V::Sha256(computed_grant_hash)),
        f("service_id", s(service_id)),
        f("import_set_hash", V::Sha256(import_set_hash)),
        f(
            "promotion_transaction_sha256",
            V::Sha256(promotion_transaction_sha256),
        ),
        f("blob_written_to_disk", b(true)),
        f("authorizes_load", b(false)),
        f("owner_sealed", b(false)),
        f("cross_reboot_proven", b(false)),
        f("persistence_claimed", b(false)),
    ]);
    let mut sink = VecSink(Vec::new());
    write_json(&record, &mut sink, 0);
    sink.0
}

fn blob_denied_evidence(
    reason: &'static str,
    planned_blob: &raios_core::artifact_blob_frame::PlannedArtifactBlobWrite,
    _readback_blob_sha256: Option<[u8; 32]>,
    blob_written_to_disk: bool,
) -> ArtifactPersistEvidence {
    ArtifactPersistEvidence {
        performed: false,
        status: "capability_denied",
        reason,
        authority: "evidence_only",
        blob_write_status: if blob_written_to_disk {
            "written"
        } else {
            "capability_denied"
        },
        artstor_blob_offset: Some(planned_blob.write_offset),
        artstor_blob_len: Some(planned_blob.frame_len),
        blob_frame_sha256: Some(planned_blob.frame_sha256),
        artifact_sha256: Some(planned_blob.payload_sha256),
        artifact_persist_seq: None,
        artifact_persist_frame_sha256: None,
        artifact_persist_readback_sha256: None,
        blob_written_to_disk,
        authorizes_load: false,
        owner_sealed: false,
        cross_reboot_proven: false,
        persistence_claimed: false,
    }
}

fn append_denied_evidence(
    reason: &'static str,
    planned_blob: &raios_core::artifact_blob_frame::PlannedArtifactBlobWrite,
    planned_append: &PlannedAppend,
    _readback_blob_sha256: Option<[u8; 32]>,
    readback_record_sha256: Option<[u8; 32]>,
) -> ArtifactPersistEvidence {
    ArtifactPersistEvidence {
        performed: false,
        status: "capability_denied",
        reason,
        authority: "evidence_only",
        blob_write_status: "written",
        artstor_blob_offset: Some(planned_blob.write_offset),
        artstor_blob_len: Some(planned_blob.frame_len),
        blob_frame_sha256: Some(planned_blob.frame_sha256),
        artifact_sha256: Some(planned_blob.payload_sha256),
        artifact_persist_seq: Some(planned_append.seq),
        artifact_persist_frame_sha256: Some(planned_append.frame_sha256),
        artifact_persist_readback_sha256: readback_record_sha256,
        blob_written_to_disk: true,
        authorizes_load: false,
        owner_sealed: false,
        cross_reboot_proven: false,
        persistence_claimed: false,
    }
}

fn next_free_artstor_offset(records: &[ArtifactPersistRecord]) -> Result<u64, &'static str> {
    let mut next = 0u64;
    let mut idx = 0usize;
    while idx < records.len() {
        let Some(end) = records[idx]
            .artstor_blob_offset
            .checked_add(records[idx].artstor_blob_len)
        else {
            return Err("artifact_store_record_span_overflow");
        };
        if end > next {
            next = end;
        }
        idx += 1;
    }
    Ok(next)
}

pub(crate) fn artifact_persist_records_from_reclog(bytes: &[u8]) -> Vec<ArtifactPersistRecord> {
    let scan = scan_reclog(bytes);
    let mut records = Vec::new();
    let mut offset = 0usize;
    let mut seq = 1u64;
    let mut prev = [0u8; 32];
    let mut count = 0u64;
    while count < scan.count && offset < bytes.len() {
        let Ok(frame) = parse_reclog_frame(bytes, offset, seq, prev) else {
            break;
        };
        let payload_start = offset + RECLOG_FRAME_HEADER_LEN;
        let payload_end = payload_start + frame.payload_len as usize;
        if let Ok(payload) = str::from_utf8(&bytes[payload_start..payload_end]) {
            if let Some(record) = parse_artifact_persist_payload(payload, frame.seq, frame.offset) {
                records.push(record);
            }
        }
        offset += frame.frame_len as usize;
        seq = seq.saturating_add(1);
        prev = frame.frame_sha256;
        count += 1;
    }
    records
}

fn parse_artifact_persist_payload(
    payload: &str,
    seq: u64,
    reclog_offset: u64,
) -> Option<ArtifactPersistRecord> {
    if !contains_bytes(
        payload.as_bytes(),
        b"\"schema\": \"raios.artifact_persist.v0\"",
    ) {
        return None;
    }
    Some(ArtifactPersistRecord {
        seq,
        reclog_offset,
        artstor_blob_offset: extract_u64(payload, b"\"artstor_blob_offset\": ")?,
        artstor_blob_len: extract_u64(payload, b"\"artstor_blob_len\": ")?,
        artstor_blob_frame_sha256: extract_sha256(payload, b"\"artstor_blob_frame_sha256\": \"")?,
        artifact_sha256: extract_sha256(payload, b"\"artifact_sha256\": \"")?,
        manifest_hash: extract_sha256(payload, b"\"manifest_hash\": \"")?,
        vm_report_hash: extract_sha256(payload, b"\"vm_report_hash\": \"")?,
        grant_hash: extract_sha256(payload, b"\"grant_hash\": \"")?,
        promotion_transaction_sha256: extract_sha256(
            payload,
            b"\"promotion_transaction_sha256\": \"",
        )?,
        authorizes_load: extract_bool(payload, b"\"authorizes_load\": ")?,
    })
}

pub(crate) fn read_verified_artstor_payload(
    controller: pci::PciMassStorageController,
    record: &ArtifactPersistRecord,
) -> Result<VerifiedArtifactPayload, &'static str> {
    let read = ahci::read_persist_artstor_region(
        controller,
        record.artstor_blob_offset,
        record.artstor_blob_len,
    );
    let Some(bytes) = read.bytes else {
        return Err(read.reason);
    };
    if sha256_bytes(&bytes) != record.artstor_blob_frame_sha256 {
        return Err("blob_hash_mismatch");
    }
    let parsed = parse_artifact_blob_frame(&bytes, 0).map_err(|_| "artifact_blob_parse_failed")?;
    if parsed.payload_sha256 != record.artifact_sha256 {
        return Err("artifact_sha_mismatch");
    }
    let payload_start = ARTIFACT_BLOB_FRAME_HEADER_LEN;
    let payload_end = payload_start
        .checked_add(parsed.payload_len as usize)
        .ok_or("artifact_blob_payload_range_overflow")?;
    if payload_end > bytes.len() {
        return Err("artifact_blob_payload_range_out_of_bounds");
    }
    Ok(VerifiedArtifactPayload {
        payload: bytes[payload_start..payload_end].to_vec(),
        blob_bytes: bytes,
    })
}

struct ScanRecordView {
    record: ArtifactPersistRecord,
    read_reason: &'static str,
    present: bool,
    blob_hash_verified: bool,
    parsed_payload_sha256: Option<[u8; 32]>,
    computed_blob_frame_sha256: Option<[u8; 32]>,
}

struct GarbageBlobView {
    offset: u64,
    len: u64,
    frame_sha256: [u8; 32],
    status: &'static str,
}

fn scan_record_views(
    controller: pci::PciMassStorageController,
    records: &[ArtifactPersistRecord],
) -> Vec<ScanRecordView> {
    let mut out = Vec::new();
    let mut idx = 0usize;
    while idx < records.len() {
        let record = records[idx];
        let read = ahci::read_persist_artstor_region(
            controller,
            record.artstor_blob_offset,
            record.artstor_blob_len,
        );
        let parsed = read
            .bytes
            .as_deref()
            .and_then(|bytes| parse_artifact_blob_frame(bytes, 0).ok());
        let computed = read.bytes.as_deref().map(sha256_bytes);
        out.push(ScanRecordView {
            record,
            read_reason: read.reason,
            present: read.read_completed && parsed.is_some(),
            blob_hash_verified: computed == Some(record.artstor_blob_frame_sha256),
            parsed_payload_sha256: parsed.map(|frame| frame.payload_sha256),
            computed_blob_frame_sha256: computed,
        });
        idx += 1;
    }
    out
}

fn scan_garbage_blobs(
    controller: pci::PciMassStorageController,
    records: &[ArtifactPersistRecord],
) -> Vec<GarbageBlobView> {
    let mut out = Vec::new();
    if records.iter().any(|record| record.artstor_blob_offset == 0) {
        return out;
    }
    let read = ahci::read_persist_artstor_region(controller, 0, ARTSTOR_SCAN_WINDOW_BYTES);
    let Some(bytes) = read.bytes.as_deref() else {
        return out;
    };
    if bytes.len() >= ARTIFACT_BLOB_MAGIC.len()
        && &bytes[..ARTIFACT_BLOB_MAGIC.len()] == ARTIFACT_BLOB_MAGIC
    {
        if let Ok(frame) = parse_artifact_blob_frame(bytes, 0) {
            out.push(GarbageBlobView {
                offset: 0,
                len: frame.frame_len as u64,
                frame_sha256: frame.frame_sha256,
                status: "garbage",
            });
        }
    }
    out
}

fn emit_scan_record(
    reason: &'static str,
    reclog_read_completed: bool,
    scan: RecordLogScan,
    records: Vec<ScanRecordView>,
    garbage: Vec<GarbageBlobView>,
) {
    let record_count = records.len() as u64;
    let garbage_count = garbage.len() as u64;
    let records_value = records.iter().map(record_scan_view).collect();
    let garbage_value = garbage.iter().map(record_garbage_view).collect();
    begin_response(ARTIFACT_STORE_SCAN_METHOD);
    emit_record_fields(
        vec![
            f("schema", s(ARTIFACT_STORE_SCAN_SCHEMA)),
            f("id", s(ARTIFACT_STORE_SCAN_ID)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("method", s(ARTIFACT_STORE_SCAN_METHOD)),
            f("reason", s(reason)),
            f("reclog_read_completed", b(reclog_read_completed)),
            f("reclog_status", s(scan.status())),
            f("reclog_count", V::U64(scan.count)),
            f("artifact_persist_record_count", V::U64(record_count)),
            f("records", V::Array(records_value)),
            f("garbage_blob_count", V::U64(garbage_count)),
            f("garbage_blobs", V::Array(garbage_value)),
            f("authority", s("reclog_chained_records_only")),
            f("read_only", b(true)),
            f("write_attempted", b(false)),
            f("authorizes_load", b(false)),
            f("runtime_path_enabled", b(false)),
            f("persistence_claimed", b(false)),
            f("evidence_complete", b(true)),
        ],
        6,
    );
    end_response(ARTIFACT_STORE_SCAN_METHOD);
}

fn record_scan_view(view: &ScanRecordView) -> V<'static> {
    V::Object(vec![
        f("seq", V::U64(view.record.seq)),
        f("reclog_offset", V::U64(view.record.reclog_offset)),
        f(
            "artstor_blob_offset",
            V::U64(view.record.artstor_blob_offset),
        ),
        f("artstor_blob_len", V::U64(view.record.artstor_blob_len)),
        f(
            "artstor_blob_frame_sha256",
            V::Sha256(view.record.artstor_blob_frame_sha256),
        ),
        f(
            "computed_blob_frame_sha256",
            record_sha_or_null(view.computed_blob_frame_sha256),
        ),
        f("artifact_sha256", V::Sha256(view.record.artifact_sha256)),
        f(
            "parsed_payload_sha256",
            record_sha_or_null(view.parsed_payload_sha256),
        ),
        f(
            "promotion_transaction_sha256",
            V::Sha256(view.record.promotion_transaction_sha256),
        ),
        f("read_reason", s(view.read_reason)),
        f("present", b(view.present)),
        f("blob_hash_verified", b(view.blob_hash_verified)),
        f("authorizes_load", b(false)),
        f("record_authorizes_load", b(view.record.authorizes_load)),
    ])
}

fn record_garbage_view(view: &GarbageBlobView) -> V<'static> {
    V::Object(vec![
        f("offset", V::U64(view.offset)),
        f("len", V::U64(view.len)),
        f("frame_sha256", V::Sha256(view.frame_sha256)),
        f("status", s(view.status)),
        f("reason", s("blob_without_chained_reclog_record")),
        f("authorizes_load", b(false)),
    ])
}

#[derive(Clone, Copy)]
struct SelftestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    blob_authorized: bool,
    record_authorized: bool,
    garbage_reported: bool,
    passed: bool,
}

fn artifact_store_selftest_cases() -> Vec<SelftestCase> {
    vec![
        artifact_store_positive_selftest(),
        artifact_store_preflight_case(
            "safe_posture_denied",
            BootPosture::Safe,
            true,
            "boot_control_safe_mode",
        ),
        artifact_store_preflight_case(
            "promotion_transaction_not_verified_denied",
            BootPosture::Normal,
            false,
            "promotion_transaction_not_verified_this_boot",
        ),
        artifact_store_full_selftest(),
        artifact_store_mutation_selftest("wrong_schema_denied", SelftestMutation::WrongSchema),
        artifact_store_mutation_selftest("wrong_target_denied", SelftestMutation::WrongTarget),
        artifact_store_garbage_selftest(),
    ]
}

#[derive(Clone, Copy)]
enum SelftestMutation {
    WrongSchema,
    WrongTarget,
}

fn artifact_store_positive_selftest() -> SelftestCase {
    let (status, reason, blob_authorized, record_authorized) =
        synthetic_artifact_persist_decision(None);
    SelftestCase {
        name: "artifact_persist_authorized",
        expected_status: "appended",
        expected_reason: "authorized_artifact_persist_append_readback_reparse_verified",
        actual_status: status,
        actual_reason: reason,
        blob_authorized,
        record_authorized,
        garbage_reported: false,
        passed: status == "appended"
            && reason == "authorized_artifact_persist_append_readback_reparse_verified"
            && blob_authorized
            && record_authorized,
    }
}

fn artifact_store_preflight_case(
    name: &'static str,
    posture: BootPosture,
    promotion_transaction_verified: bool,
    expected_reason: &'static str,
) -> SelftestCase {
    let reason = artifact_persist_preflight_denial(posture, promotion_transaction_verified)
        .unwrap_or("unexpected_authorized");
    SelftestCase {
        name,
        expected_status: "denied",
        expected_reason,
        actual_status: "denied",
        actual_reason: reason,
        blob_authorized: false,
        record_authorized: false,
        garbage_reported: false,
        passed: reason == expected_reason,
    }
}

fn artifact_store_full_selftest() -> SelftestCase {
    let payload = b"wasm";
    let frame = build_artifact_blob_frame(payload);
    let reason = match plan_artifact_blob_write(0, payload, frame.len() as u64 - 1) {
        Ok(_) => "unexpected_authorized",
        Err(denied) => denied.reason(),
    };
    SelftestCase {
        name: "artifact_store_full_denied",
        expected_status: "denied",
        expected_reason: "artifact_store_full",
        actual_status: "denied",
        actual_reason: reason,
        blob_authorized: false,
        record_authorized: false,
        garbage_reported: false,
        passed: reason == "artifact_store_full",
    }
}

fn artifact_store_mutation_selftest(
    name: &'static str,
    mutation: SelftestMutation,
) -> SelftestCase {
    let (status, reason, blob_authorized, record_authorized) =
        synthetic_artifact_persist_decision(Some(mutation));
    let expected_reason = match mutation {
        SelftestMutation::WrongSchema => "record_schema_mismatch",
        SelftestMutation::WrongTarget => "target_out_of_scope",
    };
    SelftestCase {
        name,
        expected_status: "denied",
        expected_reason,
        actual_status: status,
        actual_reason: reason,
        blob_authorized,
        record_authorized,
        garbage_reported: false,
        passed: status == "denied" && reason == expected_reason,
    }
}

fn artifact_store_garbage_selftest() -> SelftestCase {
    let frame = build_artifact_blob_frame(b"garbage");
    let parsed = parse_artifact_blob_frame(&frame, 0).is_ok();
    let garbage = parsed && detect_synthetic_garbage_blob(&frame, &[]);
    SelftestCase {
        name: "blob_without_record_is_garbage",
        expected_status: "garbage",
        expected_reason: "blob_without_chained_reclog_record",
        actual_status: if garbage { "garbage" } else { "not_garbage" },
        actual_reason: if garbage {
            "blob_without_chained_reclog_record"
        } else {
            "unexpected_record_authority"
        },
        blob_authorized: false,
        record_authorized: false,
        garbage_reported: garbage,
        passed: garbage,
    }
}

fn synthetic_artifact_persist_decision(
    mutation: Option<SelftestMutation>,
) -> (&'static str, &'static str, bool, bool) {
    let payload = b"wasm";
    let planned_blob = match plan_artifact_blob_write(0, payload, 4096) {
        Ok(planned) => planned,
        Err(denied) => return ("denied", denied.reason(), false, false),
    };
    let blob_parse = parse_artifact_blob_frame(&planned_blob.frame, 0).is_ok();
    let mut blob_input = ScopedArtifactStoreBlobInput {
        method: Some(ARTIFACT_BLOB_EXPECTED_METHOD),
        target_id: Some(ARTIFACT_BLOB_EXPECTED_TARGET_ID),
        record_schema: Some(ARTIFACT_BLOB_EXPECTED_RECORD_SCHEMA),
        region_marker: Some(ARTIFACT_BLOB_EXPECTED_REGION_MARKER),
        frame_len: Some(planned_blob.frame_len),
        write_offset: Some(planned_blob.write_offset),
        artstor_byte_count: Some(4096),
        artstor_absolute_start_lba: Some(100),
        artstor_lba_count: Some(8),
        payload_sha256: Some(planned_blob.payload_sha256),
        planned_payload_sha256: Some(planned_blob.payload_sha256),
        planned_frame_sha256: Some(planned_blob.frame_sha256),
        readback_frame_sha256: Some(planned_blob.frame_sha256),
        write_attempted: true,
        write_completed: true,
        readback_completed: true,
        readback_matches_planned: true,
        reparse_valid: blob_parse,
        span_in_bounds: true,
        signature_verified: true,
        grant_binds_capability: true,
        trust_tier: Some(ARTIFACT_BLOB_EXPECTED_TRUST_TIER),
        owner_sealed: false,
        promotion_authority_is_placeholder: PROMOTION_AUTHORITY_IS_PLACEHOLDER,
        promotion_transaction_verified: true,
        persistence_claimed: false,
    };
    if matches!(mutation, Some(SelftestMutation::WrongTarget)) {
        blob_input.target_id = Some("append.record_log.seed_data");
    }
    let blob_decision = evaluate_scoped_artifact_store_blob(&blob_input);
    if !blob_decision.performed {
        return (blob_decision.status, blob_decision.reason, false, false);
    }

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
    let record_payload = artifact_persist_payload_bytes(
        planned_blob.write_offset,
        planned_blob.frame_len,
        planned_blob.frame_sha256,
        planned_blob.payload_sha256,
        [0x11; 32],
        [0x22; 32],
        [0x33; 32],
        "svc.dev.granted_candidate",
        [0x44; 32],
        [0x55; 32],
    );
    let Ok(planned_append) = plan_reclog_append(&scan, &record_payload, 4096) else {
        return ("denied", "plan_failed", true, false);
    };
    let reparse_valid = parse_reclog_frame(
        &planned_append.frame,
        0,
        planned_append.seq,
        planned_append.prev_frame_sha256,
    )
    .is_ok();
    let mut append_input = ScopedArtifactPersistAppendInput {
        method: Some(ARTIFACT_PERSIST_EXPECTED_METHOD),
        target_id: Some(ARTIFACT_PERSIST_EXPECTED_TARGET_ID),
        record_schema: Some(ARTIFACT_PERSIST_EXPECTED_RECORD_SCHEMA),
        region_marker: Some(ARTIFACT_PERSIST_EXPECTED_REGION_MARKER),
        frame_len: Some(planned_append.frame_len),
        write_offset: Some(planned_append.write_offset),
        reclog_byte_count: Some(4096),
        absolute_start_lba: Some(200),
        reclog_lba_count: Some(8),
        seq: Some(planned_append.seq),
        tail_seq: Some(scan.tail_seq),
        count: Some(scan.count),
        prev_frame_sha256: Some(planned_append.prev_frame_sha256),
        tail_frame_sha256: scan.tail_frame_sha256,
        payload_sha256: Some(planned_append.payload_sha256),
        planned_payload_sha256: Some(planned_append.payload_sha256),
        planned_frame_sha256: Some(planned_append.frame_sha256),
        readback_frame_sha256: Some(planned_append.frame_sha256),
        write_attempted: true,
        write_completed: true,
        readback_completed: true,
        readback_matches_planned: true,
        reparse_valid,
        span_in_bounds: true,
        signature_verified: true,
        grant_binds_capability: true,
        trust_tier: Some(ARTIFACT_PERSIST_EXPECTED_TRUST_TIER),
        owner_sealed: false,
        promotion_authority_is_placeholder: PROMOTION_AUTHORITY_IS_PLACEHOLDER,
        promotion_transaction_verified: true,
        persistence_claimed: false,
    };
    if matches!(mutation, Some(SelftestMutation::WrongSchema)) {
        append_input.record_schema = Some("raios.promotion_transaction.v0");
    }
    let append_decision = evaluate_scoped_artifact_persist_append(&append_input);
    (
        append_decision.status,
        append_decision.reason,
        true,
        append_decision.performed,
    )
}

fn artifact_persist_preflight_denial(
    posture: BootPosture,
    promotion_transaction_verified: bool,
) -> Option<&'static str> {
    if !matches!(posture, BootPosture::Normal | BootPosture::Probation) {
        Some("boot_control_safe_mode")
    } else if !promotion_transaction_verified {
        Some("promotion_transaction_not_verified_this_boot")
    } else {
        None
    }
}

fn detect_synthetic_garbage_blob(bytes: &[u8], records: &[ArtifactPersistRecord]) -> bool {
    if records.iter().any(|record| record.artstor_blob_offset == 0) {
        return false;
    }
    parse_artifact_blob_frame(bytes, 0).is_ok()
}

fn record_selftest_case(case: &SelftestCase) -> V<'static> {
    V::Object(vec![
        f("case", s(case.name)),
        f("expected_status", s(case.expected_status)),
        f("expected_reason", s(case.expected_reason)),
        f("actual_status", s(case.actual_status)),
        f("actual_reason", s(case.actual_reason)),
        f("blob_authorized", b(case.blob_authorized)),
        f("record_authorized", b(case.record_authorized)),
        f("garbage_reported", b(case.garbage_reported)),
        f("authorizes_load", b(false)),
        f("passed", b(case.passed)),
    ])
}

fn optional_u64(value: Option<u64>) -> V<'static> {
    match value {
        Some(value) => V::U64(value),
        None => V::Null,
    }
}

pub(crate) fn extract_u64(payload: &str, needle: &[u8]) -> Option<u64> {
    let bytes = payload.as_bytes();
    let mut idx = find_bytes(bytes, needle)? + needle.len();
    while idx < bytes.len() && bytes[idx] == b' ' {
        idx += 1;
    }
    let mut value = 0u64;
    let mut found = false;
    while idx < bytes.len() && bytes[idx].is_ascii_digit() {
        found = true;
        value = value
            .checked_mul(10)?
            .checked_add((bytes[idx] - b'0') as u64)?;
        idx += 1;
    }
    if found {
        Some(value)
    } else {
        None
    }
}

pub(crate) fn extract_bool(payload: &str, needle: &[u8]) -> Option<bool> {
    let bytes = payload.as_bytes();
    let idx = find_bytes(bytes, needle)? + needle.len();
    if starts_with(&bytes[idx..], b"true") {
        Some(true)
    } else if starts_with(&bytes[idx..], b"false") {
        Some(false)
    } else {
        None
    }
}

pub(crate) fn extract_sha256(payload: &str, needle: &[u8]) -> Option<[u8; 32]> {
    let bytes = payload.as_bytes();
    let idx = find_bytes(bytes, needle)? + needle.len();
    if idx.checked_add(64)? > bytes.len() {
        return None;
    }
    let mut out = [0u8; 32];
    let mut pos = 0usize;
    while pos < 32 {
        let hi = hex_nibble(bytes[idx + pos * 2])?;
        let lo = hex_nibble(bytes[idx + pos * 2 + 1])?;
        out[pos] = (hi << 4) | lo;
        pos += 1;
    }
    Some(out)
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    find_bytes(haystack, needle).is_some()
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    let mut idx = 0usize;
    while idx + needle.len() <= haystack.len() {
        if &haystack[idx..idx + needle.len()] == needle {
            return Some(idx);
        }
        idx += 1;
    }
    None
}

fn starts_with(bytes: &[u8], prefix: &[u8]) -> bool {
    bytes.len() >= prefix.len() && &bytes[..prefix.len()] == prefix
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
