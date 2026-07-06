use alloc::{vec, vec::Vec};

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_sha_or_null, record_str as s,
    },
    ahci, pci,
};
use raios_core::{
    boot_control::BootPosture,
    durable_record_frame::{
        durable_record_log_scan_fields, parse_reclog_frame, plan_reclog_append, scan_reclog,
        PlannedAppend, RecordLogScan, DURABLE_RECORD_LOG_SCAN_SCHEMA,
    },
    record::{write_json, Field, Value as V},
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
