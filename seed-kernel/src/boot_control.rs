use alloc::{vec, vec::Vec};

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_sha_or_null, record_str as s,
    },
    ahci, event_log, pci, ui,
};
use raios_core::{
    boot_control::{
        boot_control_read_fields, evaluate_boot_control, parse_boot_slot, plan_boot_success_mark,
        BootControlDecision, BootPosture, BootSlotId, BootStorageSlot, BootSuccessMarkPlan,
        ParsedBootControl, BOOTCTL_REGION_BYTE_COUNT, BOOTCTL_SLOT_SIZE, BOOT_CONTROL_READ_SCHEMA,
        BOOT_CONTROL_SCHEMA,
    },
    durable_record_frame::{
        parse_reclog_frame, plan_reclog_append, scan_reclog, PlannedAppend, RecordLogScan,
    },
    record::{write_json, Field, Value as V},
    scoped_boot_control_replace::{
        evaluate_scoped_boot_control_replace, ScopedBootControlReplaceInput,
        EXPECTED_METHOD as BOOTCTL_REPLACE_METHOD,
        EXPECTED_RECORD_SCHEMA as BOOTCTL_REPLACE_RECORD_SCHEMA,
        EXPECTED_REGION_MARKER as BOOTCTL_REPLACE_REGION_MARKER,
        EXPECTED_TARGET_ID as BOOTCTL_REPLACE_TARGET_ID,
    },
    scoped_seed_data_append::{
        evaluate_scoped_seed_data_append, ScopedSeedDataAppendInput,
        EXPECTED_METHOD as RECLOG_APPEND_METHOD,
        EXPECTED_RECORD_SCHEMA as RECLOG_APPEND_RECORD_SCHEMA,
        EXPECTED_REGION_MARKER as RECLOG_APPEND_REGION_MARKER,
        EXPECTED_TARGET_ID as RECLOG_APPEND_TARGET_ID,
    },
    sha256_bytes, ByteSink,
};

const READ_METHOD: &str = "boot.control_read";
const MARK_METHOD: &str = "boot.control_mark_success";
const MARK_SCHEMA: &str = "raios.boot_control_success_mark.v0";
const MARK_ID: &str = "boot_control_success_mark.seed_data.current_boot.v0";

struct BootControlEvidence {
    reason: &'static str,
    controller: Option<pci::PciMassStorageController>,
    controller_present: bool,
    source_port_index: Option<u8>,
    layout_status: &'static str,
    data_layout_status: &'static str,
    read_attempted: bool,
    read_completed: bool,
    region_bounds_valid: bool,
    bootctl_absolute_start_lba: u64,
    bootctl_lba_count: u64,
    bootctl_byte_count: u64,
    slot_a_record: Result<ParsedBootControl, &'static str>,
    slot_b_record: Result<ParsedBootControl, &'static str>,
    decision: BootControlDecision,
    authoritative_record: Option<ParsedBootControl>,
}

impl BootControlEvidence {
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
            bootctl_absolute_start_lba: 0,
            bootctl_lba_count: 0,
            bootctl_byte_count: 0,
            slot_a_record: Err(reason),
            slot_b_record: Err(reason),
            decision: BootControlDecision::persistence_unavailable(reason),
            authoritative_record: None,
        }
    }
}

#[derive(Clone, Copy)]
struct BootSuccessCriteria {
    serial_agent_command_received: bool,
    framebuffer_active: bool,
    serial_or_framebuffer_active: bool,
    heap_allocator_live: bool,
    ahci_probe_complete: bool,
    seed_data_superblock_present: bool,
    boot_control_valid: bool,
    event_log_ready: bool,
    agent_protocol_handler_reached: bool,
    system_snapshot_method_resolves: bool,
    no_panic_before_mark: bool,
    network_provider_required: bool,
}

impl BootSuccessCriteria {
    fn all_passed(self) -> bool {
        self.serial_or_framebuffer_active
            && self.heap_allocator_live
            && self.ahci_probe_complete
            && self.seed_data_superblock_present
            && self.boot_control_valid
            && self.event_log_ready
            && self.agent_protocol_handler_reached
            && self.system_snapshot_method_resolves
            && self.no_panic_before_mark
            && !self.network_provider_required
    }

    fn first_failure(self) -> Option<&'static str> {
        if !self.serial_or_framebuffer_active {
            Some("boot_success_criteria_serial_or_framebuffer_missing")
        } else if !self.heap_allocator_live {
            Some("boot_success_criteria_heap_allocator_not_live")
        } else if !self.ahci_probe_complete {
            Some("boot_success_criteria_ahci_probe_incomplete")
        } else if !self.seed_data_superblock_present {
            Some("boot_success_criteria_seed_data_superblock_not_present")
        } else if !self.boot_control_valid {
            Some("boot_success_criteria_boot_control_not_markable")
        } else if !self.event_log_ready {
            Some("boot_success_criteria_event_log_not_ready")
        } else if !self.agent_protocol_handler_reached {
            Some("boot_success_criteria_agent_protocol_handler_not_reached")
        } else if !self.system_snapshot_method_resolves {
            Some("boot_success_criteria_system_snapshot_unresolved")
        } else if !self.no_panic_before_mark {
            Some("boot_success_criteria_panic_before_mark")
        } else if self.network_provider_required {
            Some("boot_success_criteria_network_provider_unexpectedly_required")
        } else {
            None
        }
    }
}

struct ReclogScanEvidence {
    reclog_absolute_start_lba: u64,
    reclog_lba_count: u64,
    reclog_byte_count: u64,
    scan: RecordLogScan,
}

struct AuditAppendEvidence {
    performed: bool,
    status: &'static str,
    reason: &'static str,
    authority: &'static str,
    seq: Option<u64>,
    frame_len: Option<u64>,
    write_offset: Option<u64>,
    payload_sha256: Option<[u8; 32]>,
    frame_sha256: Option<[u8; 32]>,
    readback_sha256: Option<[u8; 32]>,
    reparse_valid: bool,
    write_attempted: bool,
    write_completed: bool,
    readback_completed: bool,
    readback_matches_planned: bool,
    span_in_bounds: bool,
}

impl AuditAppendEvidence {
    fn denied(reason: &'static str) -> Self {
        Self {
            performed: false,
            status: "capability_denied",
            reason,
            authority: "evidence_only",
            seq: None,
            frame_len: None,
            write_offset: None,
            payload_sha256: None,
            frame_sha256: None,
            readback_sha256: None,
            reparse_valid: false,
            write_attempted: false,
            write_completed: false,
            readback_completed: false,
            readback_matches_planned: false,
            span_in_bounds: false,
        }
    }
}

struct VecSink(Vec<u8>);

impl ByteSink for VecSink {
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }
}

pub(crate) fn emit_boot_control_read() {
    let evidence = current_boot_bootctl_read();
    let mut fields =
        boot_control_read_fields(&evidence.decision, evidence.authoritative_record.as_ref());
    fields.push(f("query_method", s(READ_METHOD)));
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
        "bootctl_absolute_start_lba",
        V::U64(evidence.bootctl_absolute_start_lba),
    ));
    fields.push(f("bootctl_lba_count", V::U64(evidence.bootctl_lba_count)));
    fields.push(f("bootctl_byte_count", V::U64(evidence.bootctl_byte_count)));
    fields.push(f("authority", s("evidence_only")));
    fields.push(f("record_model_entry", s(BOOT_CONTROL_READ_SCHEMA)));

    begin_response(READ_METHOD);
    emit_record_fields(fields, 6);
    end_response(READ_METHOD);
}

pub(crate) fn emit_boot_control_success_mark(runtime: ui::RuntimeStatus) {
    let evidence = current_boot_bootctl_read();
    let criteria = boot_success_criteria(&evidence, runtime);
    if !criteria.all_passed() {
        emit_success_mark_record(
            &evidence,
            criteria,
            None,
            None,
            None,
            false,
            None,
            None,
            None,
            "capability_denied",
            criteria
                .first_failure()
                .unwrap_or("boot_success_criteria_unmet"),
            "evidence_only",
        );
        return;
    }

    let Some(authoritative_record) = evidence.authoritative_record.as_ref() else {
        emit_success_mark_record(
            &evidence,
            criteria,
            None,
            None,
            None,
            false,
            None,
            None,
            None,
            "capability_denied",
            "boot_control_authoritative_record_missing",
            "evidence_only",
        );
        return;
    };
    let Some(authoritative_storage_slot) = evidence.decision.authoritative_bootctl_slot else {
        emit_success_mark_record(
            &evidence,
            criteria,
            None,
            None,
            None,
            false,
            None,
            None,
            None,
            "capability_denied",
            "boot_control_authoritative_slot_missing",
            "evidence_only",
        );
        return;
    };
    let plan = match plan_boot_success_mark(
        authoritative_record,
        authoritative_storage_slot,
        &evidence.decision,
    ) {
        Ok(plan) => plan,
        Err(denied) => {
            emit_success_mark_record(
                &evidence,
                criteria,
                None,
                None,
                None,
                false,
                None,
                None,
                None,
                "capability_denied",
                denied.reason(),
                "evidence_only",
            );
            return;
        }
    };
    let Some(controller) = evidence.controller else {
        emit_success_mark_record(
            &evidence,
            criteria,
            Some(&plan),
            None,
            None,
            false,
            None,
            None,
            None,
            "capability_denied",
            "ahci_controller_not_observed",
            "evidence_only",
        );
        return;
    };

    let write = unsafe {
        ahci::write_readback_bootctl_slot(controller, plan.write_offset, &plan.slot_bytes)
    };
    let readback_sha256 = write.readback.as_deref().map(sha256_bytes);
    let readback_parsed = write
        .readback
        .as_deref()
        .and_then(|bytes| parse_boot_slot(bytes).ok());
    let reparse_valid = readback_parsed
        .map(|parsed| {
            parsed.seq == plan.new_seq
                && parsed.payload_sha256 == plan.payload_sha256
                && parsed.storage_sha256 == plan.frame_sha256
        })
        .unwrap_or(false);
    let decision = evaluate_scoped_boot_control_replace(&ScopedBootControlReplaceInput {
        method: Some(BOOTCTL_REPLACE_METHOD),
        target_id: Some(BOOTCTL_REPLACE_TARGET_ID),
        record_schema: Some(BOOTCTL_REPLACE_RECORD_SCHEMA),
        region_marker: Some(BOOTCTL_REPLACE_REGION_MARKER),
        slot_len: Some(BOOTCTL_SLOT_SIZE as u64),
        write_offset: Some(plan.write_offset),
        bootctl_byte_count: Some(evidence.bootctl_byte_count),
        absolute_start_lba: Some(evidence.bootctl_absolute_start_lba),
        bootctl_lba_count: Some(evidence.bootctl_lba_count),
        new_seq: Some(plan.new_seq),
        slot_a_seq: Some(slot_seq_or_zero(evidence.slot_a_record)),
        slot_b_seq: Some(slot_seq_or_zero(evidence.slot_b_record)),
        authoritative_storage_slot: Some(authoritative_storage_slot),
        payload_sha256: Some(plan.payload_sha256),
        planned_payload_sha256: Some(plan.payload_sha256),
        planned_frame_sha256: Some(plan.frame_sha256),
        readback_frame_sha256: readback_sha256,
        write_attempted: write.write_attempted,
        write_completed: write.write_completed,
        readback_completed: write.readback_completed,
        readback_matches_planned: readback_sha256 == Some(plan.frame_sha256),
        reparse_valid,
        span_in_bounds: write.span_in_bounds,
    });

    if !decision.performed {
        emit_success_mark_record(
            &evidence,
            criteria,
            Some(&plan),
            Some(&write),
            readback_sha256,
            reparse_valid,
            Some(&decision),
            None,
            None,
            "capability_denied",
            decision.reason,
            "evidence_only",
        );
        return;
    }

    let after = current_boot_bootctl_read();
    if !post_replace_rescan_ok(&plan, &after) {
        emit_success_mark_record(
            &evidence,
            criteria,
            Some(&plan),
            Some(&write),
            readback_sha256,
            reparse_valid,
            Some(&decision),
            Some(&after),
            None,
            "capability_denied",
            "post_replace_rescan_mismatch",
            "evidence_only",
        );
        return;
    }

    let audit = append_boot_success_audit(controller, &plan, &after);
    emit_success_mark_record(
        &evidence,
        criteria,
        Some(&plan),
        Some(&write),
        readback_sha256,
        reparse_valid,
        Some(&decision),
        Some(&after),
        Some(&audit),
        decision.status,
        decision.reason,
        "scoped_boot_control_replace_authorized",
    );
}

pub(crate) fn current_boot_posture() -> BootPosture {
    current_boot_bootctl_read().decision.posture
}

fn current_boot_bootctl_read() -> BootControlEvidence {
    let Some(controller) = pci::find_mass_storage_controller() else {
        return BootControlEvidence::absent("ahci_controller_not_observed");
    };
    let read = ahci::read_persist_bootctl_region(controller);
    let source_port_index = if read.layout.port_index == u8::MAX {
        None
    } else {
        Some(read.layout.port_index)
    };
    let mut evidence = BootControlEvidence {
        reason: read.reason,
        controller: Some(controller),
        controller_present: read.layout.controller_present,
        source_port_index,
        layout_status: read.layout.status(),
        data_layout_status: read.layout.data.status.as_str(),
        read_attempted: read.read_attempted,
        read_completed: read.read_completed,
        region_bounds_valid: read.region_bounds_valid,
        bootctl_absolute_start_lba: read.absolute_start_lba,
        bootctl_lba_count: read.lba_count,
        bootctl_byte_count: read.byte_count,
        slot_a_record: Err("bootctl_not_parsed"),
        slot_b_record: Err("bootctl_not_parsed"),
        decision: BootControlDecision::persistence_unavailable(read.reason),
        authoritative_record: None,
    };

    let Some(bytes) = read.bytes.as_deref() else {
        return evidence;
    };
    if bytes.len() != BOOTCTL_REGION_BYTE_COUNT {
        evidence.reason = "bootctl_region_byte_count_mismatch";
        evidence.decision = BootControlDecision::persistence_unavailable(evidence.reason);
        return evidence;
    }

    let storage_slot_a = parse_boot_slot(&bytes[..BOOTCTL_SLOT_SIZE]);
    let storage_slot_b = parse_boot_slot(&bytes[BOOTCTL_SLOT_SIZE..]);
    let decision = evaluate_boot_control(storage_slot_a, storage_slot_b);
    let authoritative_record = match decision.authoritative_bootctl_slot {
        Some(BootStorageSlot::A) => storage_slot_a.ok(),
        Some(BootStorageSlot::B) => storage_slot_b.ok(),
        None => None,
    };
    evidence.slot_a_record = storage_slot_a;
    evidence.slot_b_record = storage_slot_b;
    evidence.decision = decision;
    evidence.authoritative_record = authoritative_record;
    evidence
}

fn boot_success_criteria(
    evidence: &BootControlEvidence,
    runtime: ui::RuntimeStatus,
) -> BootSuccessCriteria {
    let serial_agent_command_received = true;
    let framebuffer_active = runtime.framebuffer.is_some();
    let boot_control_valid = evidence.authoritative_record.is_some()
        && matches!(
            evidence.decision.posture,
            BootPosture::Normal | BootPosture::Probation
        );
    BootSuccessCriteria {
        serial_agent_command_received,
        framebuffer_active,
        serial_or_framebuffer_active: serial_agent_command_received || framebuffer_active,
        heap_allocator_live: evidence.read_completed,
        ahci_probe_complete: evidence.controller_present && evidence.read_completed,
        seed_data_superblock_present: evidence.layout_status == "present"
            && evidence.data_layout_status == "present",
        boot_control_valid,
        event_log_ready: event_log::recent_events(1).capacity > 0,
        agent_protocol_handler_reached: true,
        system_snapshot_method_resolves: super::method_registered_exact("system.snapshot"),
        no_panic_before_mark: true,
        network_provider_required: false,
    }
}

fn append_boot_success_audit(
    controller: pci::PciMassStorageController,
    plan: &BootSuccessMarkPlan,
    after: &BootControlEvidence,
) -> AuditAppendEvidence {
    let scan = current_boot_reclog_scan(controller);
    let payload = boot_success_audit_payload_bytes(plan, after);
    let planned = match plan_reclog_append(&scan.scan, &payload, scan.reclog_byte_count) {
        Ok(planned) => planned,
        Err(denied) => return AuditAppendEvidence::denied(denied.reason()),
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
        method: Some(RECLOG_APPEND_METHOD),
        target_id: Some(RECLOG_APPEND_TARGET_ID),
        record_schema: Some(RECLOG_APPEND_RECORD_SCHEMA),
        region_marker: Some(RECLOG_APPEND_REGION_MARKER),
        frame_len: Some(planned.frame_len),
        write_offset: Some(planned.write_offset),
        reclog_byte_count: Some(scan.reclog_byte_count),
        absolute_start_lba: Some(scan.reclog_absolute_start_lba),
        reclog_lba_count: Some(scan.reclog_lba_count),
        seq: Some(planned.seq),
        tail_seq: Some(scan.scan.tail_seq),
        count: Some(scan.scan.count),
        prev_frame_sha256: Some(planned.prev_frame_sha256),
        tail_frame_sha256: scan.scan.tail_frame_sha256,
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
        return audit_from_write(
            "capability_denied",
            decision.reason,
            "evidence_only",
            &planned,
            &write,
            readback_sha256,
            reparse_valid,
        );
    }

    let after_scan = current_boot_reclog_scan(controller);
    let rescan_ok = scan
        .scan
        .count
        .checked_add(1)
        .map(|count| after_scan.scan.count == count)
        .unwrap_or(false)
        && after_scan.scan.tail_seq == planned.seq
        && after_scan.scan.tail_frame_sha256 == Some(planned.frame_sha256);
    if !rescan_ok {
        return audit_from_write(
            "capability_denied",
            "audit_post_append_rescan_mismatch",
            "evidence_only",
            &planned,
            &write,
            readback_sha256,
            reparse_valid,
        );
    }

    audit_from_write(
        "appended",
        decision.reason,
        "scoped_seed_data_append_authorized",
        &planned,
        &write,
        readback_sha256,
        reparse_valid,
    )
}

fn audit_from_write(
    status: &'static str,
    reason: &'static str,
    authority: &'static str,
    planned: &PlannedAppend,
    write: &ahci::ReclogAppendWriteEvidence,
    readback_sha256: Option<[u8; 32]>,
    reparse_valid: bool,
) -> AuditAppendEvidence {
    AuditAppendEvidence {
        performed: status == "appended",
        status,
        reason,
        authority,
        seq: Some(planned.seq),
        frame_len: Some(planned.frame_len),
        write_offset: Some(planned.write_offset),
        payload_sha256: Some(planned.payload_sha256),
        frame_sha256: Some(planned.frame_sha256),
        readback_sha256,
        reparse_valid,
        write_attempted: write.write_attempted,
        write_completed: write.write_completed,
        readback_completed: write.readback_completed,
        readback_matches_planned: readback_sha256 == Some(planned.frame_sha256),
        span_in_bounds: write.span_in_bounds,
    }
}

fn current_boot_reclog_scan(controller: pci::PciMassStorageController) -> ReclogScanEvidence {
    let read = ahci::read_persist_reclog_region(controller);
    let scan = match read.bytes.as_deref() {
        Some(bytes) => scan_reclog(bytes),
        None => RecordLogScan::not_scanned(read.reason),
    };
    ReclogScanEvidence {
        reclog_absolute_start_lba: read.absolute_start_lba,
        reclog_lba_count: read.lba_count,
        reclog_byte_count: read.byte_count,
        scan,
    }
}

fn boot_success_audit_payload_bytes(
    plan: &BootSuccessMarkPlan,
    after: &BootControlEvidence,
) -> Vec<u8> {
    let record = V::Object(vec![
        f("schema", s(RECLOG_APPEND_RECORD_SCHEMA)),
        f("id", s("durable_record.boot_success_mark.current_boot.v0")),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("record_kind", s("boot_lifecycle_boot_success_mark")),
        f("boot_control_schema", s(BOOT_CONTROL_SCHEMA)),
        f("marked_slot", slot_id_value(plan.marked_slot)),
        f(
            "written_storage_slot",
            storage_slot_value(Some(plan.written_storage_slot)),
        ),
        f("new_seq", V::U64(plan.new_seq)),
        f("would_mark_good", b(plan.would_mark_good)),
        f("pending_consumed", b(plan.pending_consumed)),
        f(
            "after_last_good",
            after_record_slot(after, |record| record.last_good),
        ),
        f(
            "after_pending",
            after_record_slot(after, |record| record.pending),
        ),
        f("after_posture", s(after.decision.posture.as_str())),
        f("persistence_claimed", b(false)),
    ]);
    let mut sink = VecSink(Vec::new());
    write_json(&record, &mut sink, 0);
    sink.0
}

fn post_replace_rescan_ok(plan: &BootSuccessMarkPlan, after: &BootControlEvidence) -> bool {
    if after.decision.authoritative_bootctl_slot != Some(plan.written_storage_slot)
        || after.decision.seq != Some(plan.new_seq)
    {
        return false;
    }
    let Some(record) = after.authoritative_record else {
        return false;
    };
    if plan.would_mark_good {
        record.last_good == plan.marked_slot
            && record.pending == BootSlotId::None
            && plan.pending_consumed
    } else {
        true
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_success_mark_record(
    evidence: &BootControlEvidence,
    criteria: BootSuccessCriteria,
    plan: Option<&BootSuccessMarkPlan>,
    write: Option<&ahci::BootctlSlotWriteEvidence>,
    readback_sha256: Option<[u8; 32]>,
    reparse_valid: bool,
    scoped_decision: Option<
        &raios_core::scoped_boot_control_replace::ScopedBootControlReplaceDecision,
    >,
    after: Option<&BootControlEvidence>,
    audit: Option<&AuditAppendEvidence>,
    status: &'static str,
    reason: &'static str,
    authority: &'static str,
) {
    let mut fields = vec![
        f("schema", s(MARK_SCHEMA)),
        f("id", s(MARK_ID)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("query_method", s(MARK_METHOD)),
        f("status", s(status)),
        f("performed", b(status == "replaced")),
        f(
            "code",
            s(if status == "replaced" {
                "ok"
            } else {
                "capability_denied"
            }),
        ),
        f("reason", s(reason)),
        f("authority", s(authority)),
        f("target_id", s(BOOTCTL_REPLACE_TARGET_ID)),
        f("record_schema", s(BOOTCTL_REPLACE_RECORD_SCHEMA)),
        f("region_marker", s(BOOTCTL_REPLACE_REGION_MARKER)),
        f("criteria", criteria_value(criteria)),
        f("boot_control_reason_before", s(evidence.decision.reason)),
        f("posture_before", s(evidence.decision.posture.as_str())),
        f(
            "authoritative_bootctl_slot_before",
            storage_slot_value(evidence.decision.authoritative_bootctl_slot),
        ),
        f(
            "selected_payload_slot_before",
            slot_id_value(evidence.decision.selected_payload_slot),
        ),
        f(
            "last_good_before",
            before_record_slot(evidence, |record| record.last_good),
        ),
        f(
            "pending_before",
            before_record_slot(evidence, |record| record.pending),
        ),
        f(
            "seq_before",
            evidence.decision.seq.map(V::U64).unwrap_or(V::Null),
        ),
    ];

    push_plan_fields(&mut fields, plan);
    fields.push(f("readback_sha256", record_sha_or_null(readback_sha256)));
    fields.push(f(
        "readback_matches",
        b(readback_sha256 == plan.map(|plan| plan.frame_sha256)),
    ));
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
        "writer_bootctl_lba_count",
        write
            .map(|write| V::U64(write.bootctl_lba_count))
            .unwrap_or(V::Null),
    ));
    fields.push(f(
        "writer_bootctl_byte_count",
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
        "scoped_decision_status",
        s(scoped_decision
            .map(|decision| decision.status)
            .unwrap_or("not_evaluated")),
    ));
    fields.push(f(
        "scoped_decision_reason",
        s(scoped_decision
            .map(|decision| decision.reason)
            .unwrap_or("not_evaluated")),
    ));
    fields.push(f(
        "after_last_good",
        after
            .map(|after| after_record_slot(after, |record| record.last_good))
            .unwrap_or(V::Null),
    ));
    fields.push(f(
        "after_authoritative_bootctl_slot",
        after
            .map(|after| storage_slot_value(after.decision.authoritative_bootctl_slot))
            .unwrap_or(V::Null),
    ));
    fields.push(f(
        "after_pending",
        after
            .map(|after| after_record_slot(after, |record| record.pending))
            .unwrap_or(V::Null),
    ));
    fields.push(f(
        "after_posture",
        after
            .map(|after| s(after.decision.posture.as_str()))
            .unwrap_or(V::Null),
    ));
    fields.push(f(
        "audit_append",
        audit
            .map(audit_value)
            .unwrap_or_else(|| audit_value(&AuditAppendEvidence::denied("not_attempted"))),
    ));
    fields.push(f("persistence_claimed", b(false)));
    fields.push(f("deterministic_slot_boot_claimed", b(false)));

    begin_response(MARK_METHOD);
    emit_record_fields(fields, 6);
    end_response(MARK_METHOD);
}

fn push_plan_fields(fields: &mut Vec<Field<'static>>, plan: Option<&BootSuccessMarkPlan>) {
    if let Some(plan) = plan {
        fields.push(f("would_mark_good", b(plan.would_mark_good)));
        fields.push(f("pending_consumed", b(plan.pending_consumed)));
        fields.push(f(
            "written_storage_slot",
            storage_slot_value(Some(plan.written_storage_slot)),
        ));
        fields.push(f("new_seq", V::U64(plan.new_seq)));
        fields.push(f("marked_slot", slot_id_value(plan.marked_slot)));
        fields.push(f("write_offset", V::U64(plan.write_offset)));
        fields.push(f("slot_len", V::U64(BOOTCTL_SLOT_SIZE as u64)));
        fields.push(f("payload_sha256", V::Sha256(plan.payload_sha256)));
        fields.push(f("frame_sha256", V::Sha256(plan.frame_sha256)));
    } else {
        fields.push(f("would_mark_good", b(false)));
        fields.push(f("pending_consumed", b(false)));
        fields.push(f("written_storage_slot", V::Null));
        fields.push(f("new_seq", V::Null));
        fields.push(f("marked_slot", V::Null));
        fields.push(f("write_offset", V::Null));
        fields.push(f("slot_len", V::Null));
        fields.push(f("payload_sha256", V::Null));
        fields.push(f("frame_sha256", V::Null));
    }
}

fn criteria_value(criteria: BootSuccessCriteria) -> V<'static> {
    V::Object(vec![
        f(
            "serial_agent_command_received",
            b(criteria.serial_agent_command_received),
        ),
        f("framebuffer_active", b(criteria.framebuffer_active)),
        f(
            "serial_or_framebuffer_active",
            b(criteria.serial_or_framebuffer_active),
        ),
        f("heap_allocator_live", b(criteria.heap_allocator_live)),
        f("ahci_probe_complete", b(criteria.ahci_probe_complete)),
        f(
            "seed_data_superblock_present",
            b(criteria.seed_data_superblock_present),
        ),
        f("boot_control_valid", b(criteria.boot_control_valid)),
        f("event_log_ready", b(criteria.event_log_ready)),
        f(
            "agent_protocol_handler_reached",
            b(criteria.agent_protocol_handler_reached),
        ),
        f(
            "system_snapshot_method_resolves",
            b(criteria.system_snapshot_method_resolves),
        ),
        f("no_panic_before_mark", b(criteria.no_panic_before_mark)),
        f(
            "network_provider_required",
            b(criteria.network_provider_required),
        ),
        f("all_passed", b(criteria.all_passed())),
    ])
}

fn audit_value(audit: &AuditAppendEvidence) -> V<'static> {
    V::Object(vec![
        f("performed", b(audit.performed)),
        f("status", s(audit.status)),
        f("reason", s(audit.reason)),
        f("authority", s(audit.authority)),
        f("seq", audit.seq.map(V::U64).unwrap_or(V::Null)),
        f("frame_len", audit.frame_len.map(V::U64).unwrap_or(V::Null)),
        f(
            "write_offset",
            audit.write_offset.map(V::U64).unwrap_or(V::Null),
        ),
        f("payload_sha256", record_sha_or_null(audit.payload_sha256)),
        f("frame_sha256", record_sha_or_null(audit.frame_sha256)),
        f("readback_sha256", record_sha_or_null(audit.readback_sha256)),
        f("reparse_valid", b(audit.reparse_valid)),
        f("write_attempted", b(audit.write_attempted)),
        f("write_completed", b(audit.write_completed)),
        f("readback_completed", b(audit.readback_completed)),
        f(
            "readback_matches_planned",
            b(audit.readback_matches_planned),
        ),
        f("span_in_bounds", b(audit.span_in_bounds)),
    ])
}

fn before_record_slot(
    evidence: &BootControlEvidence,
    get: fn(ParsedBootControl) -> BootSlotId,
) -> V<'static> {
    evidence
        .authoritative_record
        .map(get)
        .map(slot_id_value)
        .unwrap_or(V::Null)
}

fn after_record_slot(
    evidence: &BootControlEvidence,
    get: fn(ParsedBootControl) -> BootSlotId,
) -> V<'static> {
    before_record_slot(evidence, get)
}

fn slot_id_value(slot: BootSlotId) -> V<'static> {
    match slot.as_str() {
        Some(value) => s(value),
        None => V::Null,
    }
}

fn storage_slot_value(slot: Option<BootStorageSlot>) -> V<'static> {
    match slot {
        Some(slot) => s(slot.as_str()),
        None => V::Null,
    }
}

fn slot_seq_or_zero(slot: Result<ParsedBootControl, &'static str>) -> u64 {
    slot.map(|slot| slot.seq).unwrap_or(0)
}
