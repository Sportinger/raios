use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_str as s,
    },
    ahci, pci,
};
use raios_core::{
    boot_control::{
        boot_control_read_fields, evaluate_boot_control, parse_boot_slot, BootControlDecision,
        BootPosture, BootStorageSlot, ParsedBootControl, BOOTCTL_REGION_BYTE_COUNT,
        BOOTCTL_SLOT_SIZE, BOOT_CONTROL_READ_SCHEMA,
    },
    record::Value as V,
};

const METHOD: &str = "boot.control_read";

struct BootControlEvidence {
    reason: &'static str,
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
    decision: BootControlDecision,
    authoritative_record: Option<ParsedBootControl>,
}

impl BootControlEvidence {
    fn absent(reason: &'static str) -> Self {
        Self {
            reason,
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
            decision: BootControlDecision::persistence_unavailable(reason),
            authoritative_record: None,
        }
    }
}

pub(crate) fn emit_boot_control_read() {
    let evidence = current_boot_bootctl_read();
    let mut fields =
        boot_control_read_fields(&evidence.decision, evidence.authoritative_record.as_ref());
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
        "bootctl_absolute_start_lba",
        V::U64(evidence.bootctl_absolute_start_lba),
    ));
    fields.push(f("bootctl_lba_count", V::U64(evidence.bootctl_lba_count)));
    fields.push(f("bootctl_byte_count", V::U64(evidence.bootctl_byte_count)));
    fields.push(f("authority", s("evidence_only")));
    fields.push(f("record_model_entry", s(BOOT_CONTROL_READ_SCHEMA)));

    begin_response(METHOD);
    emit_record_fields(fields, 6);
    end_response(METHOD);
}

#[allow(dead_code)]
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
    evidence.decision = decision;
    evidence.authoritative_record = authoritative_record;
    evidence
}
