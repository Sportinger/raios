use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_str as s,
    },
    ahci, pci,
};
use raios_core::{
    durable_record_frame::{
        durable_record_log_scan_fields, scan_reclog, RecordLogScan, DURABLE_RECORD_LOG_SCAN_SCHEMA,
    },
    record::Value as V,
};

const METHOD: &str = "durable.record_log_scan";

struct DurableRecordLogScanEvidence {
    reason: &'static str,
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
