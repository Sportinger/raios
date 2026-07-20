use alloc::string::String;
use core::fmt::Write;

use crate::{
    durable_record_frame::{
        parse_reclog_frame, RECLOG_FRAME_HEADER_LEN, RECLOG_MAGIC, RECLOG_PAYLOAD_SHA256_OFFSET,
        RECLOG_PREV_FRAME_SHA256_OFFSET,
    },
    sha256_bytes,
};

pub const HW_FAILURE_TRACE_SCHEMA: &str = "raios.hw_failure_trace.v0";
pub const HW_FAILURE_TRACE_CLASSIFICATION: &str = "local_only";
pub const HW_FAILURE_TRACE_MAX_STEPS: usize = 4;
pub const HW_FAILURE_TRACE_FRAME_LEN: usize = 512;
pub const HW_FAILURE_TRACE_MAX_PAYLOAD_LEN: usize =
    HW_FAILURE_TRACE_FRAME_LEN - RECLOG_FRAME_HEADER_LEN;
/// Numeric release identity for seed-kernel 0.1.0. A later build pipeline can
/// replace this with a generated immutable artifact identifier without
/// widening the trace API to accept arbitrary strings.
pub const SEED_KERNEL_BUILD_ID_V0_1_0: u64 = 0x0000_0000_0001_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum HwFailureSubsystem {
    MarvellWifiPcie = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum HwFailurePhase {
    FirmwareTransport = 1,
    HardwareSpec = 2,
    Scan = 3,
    Authenticate = 4,
    Associate = 5,
    KeyExchange = 6,
    LinkBringup = 7,
    NetworkStateGrant = 8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum HwFailureStatus {
    Started = 1,
    Completed = 2,
    Timeout = 100,
    TransportFault = 101,
    FirmwareRejected = 102,
    UnsupportedSecurity = 103,
    AuthenticationRejected = 104,
    AssociationRejected = 105,
    KeyExchangeFailed = 106,
    LinkLost = 107,
    BootPostureDenied = 108,
    NetworkStateNotGranted = 109,
}

/// Only status-bearing registers are representable. Descriptor buffers,
/// station addresses, SSIDs, BSSIDs and key material have no trace input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum HwFailureRegister {
    None = 0,
    MarvellHostInterruptStatus = 1,
    MarvellHostInterruptMask = 2,
    MarvellFirmwareStatus = 3,
    MarvellCommandResponseStatus = 4,
    XhciUsbStatus = 5,
    XhciPortStatusChange = 6,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HwFailureTraceStep {
    pub boot_ms: u32,
    pub phase: HwFailurePhase,
    pub status: HwFailureStatus,
    pub register: HwFailureRegister,
    pub register_value: u32,
}

impl HwFailureTraceStep {
    pub const EMPTY: Self = Self {
        boot_ms: 0,
        phase: HwFailurePhase::FirmwareTransport,
        status: HwFailureStatus::Started,
        register: HwFailureRegister::None,
        register_value: 0,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HwFailureTrace {
    pub build_id: u64,
    pub subsystem: HwFailureSubsystem,
    steps: [HwFailureTraceStep; HW_FAILURE_TRACE_MAX_STEPS],
    step_count: u8,
}

impl HwFailureTrace {
    pub const fn new(build_id: u64, subsystem: HwFailureSubsystem) -> Self {
        Self {
            build_id,
            subsystem,
            steps: [HwFailureTraceStep::EMPTY; HW_FAILURE_TRACE_MAX_STEPS],
            step_count: 0,
        }
    }

    pub fn push_step(&mut self, step: HwFailureTraceStep) -> Result<(), &'static str> {
        let index = self.step_count as usize;
        if index >= HW_FAILURE_TRACE_MAX_STEPS {
            return Err("hw_failure_trace_step_limit");
        }
        if index > 0 && step.boot_ms < self.steps[index - 1].boot_ms {
            return Err("hw_failure_trace_time_not_monotonic");
        }
        self.steps[index] = step;
        self.step_count += 1;
        Ok(())
    }

    pub fn steps(&self) -> &[HwFailureTraceStep] {
        &self.steps[..self.step_count as usize]
    }

    pub const fn is_empty(&self) -> bool {
        self.step_count == 0
    }

    pub fn payload(&self) -> Result<String, &'static str> {
        if self.step_count == 0 {
            return Err("hw_failure_trace_empty");
        }
        let mut payload = String::new();
        write!(
            payload,
            "{{\"schema\":\"{}\",\"classification\":\"{}\",\"scope\":\"current_boot\",\"build_id\":{},\"subsystem\":{},\"steps\":[",
            HW_FAILURE_TRACE_SCHEMA,
            HW_FAILURE_TRACE_CLASSIFICATION,
            self.build_id,
            self.subsystem as u8,
        )
        .map_err(|_| "hw_failure_trace_format_failed")?;
        for (index, step) in self.steps().iter().enumerate() {
            if index != 0 {
                payload.push(',');
            }
            write!(
                payload,
                "[{},{},{},{},{}]",
                step.boot_ms,
                step.phase as u8,
                step.status as u16,
                step.register as u16,
                step.register_value,
            )
            .map_err(|_| "hw_failure_trace_format_failed")?;
        }
        payload.push_str("]}");
        if payload.len() > HW_FAILURE_TRACE_MAX_PAYLOAD_LEN {
            return Err("hw_failure_trace_payload_too_large");
        }
        Ok(payload)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HwFailureTraceQueueResult {
    Queued,
    InvalidTrace,
    AlreadyQueuedOrAttempted,
}

/// A boot-lifetime, one-shot slot. Taking the trace permanently consumes the
/// attempt even when media I/O later fails, preventing recursive retry storms.
pub struct HwFailureTraceLatch {
    pending: Option<HwFailureTrace>,
    attempted: bool,
}

impl HwFailureTraceLatch {
    pub const fn new() -> Self {
        Self {
            pending: None,
            attempted: false,
        }
    }

    pub fn queue(&mut self, trace: HwFailureTrace) -> HwFailureTraceQueueResult {
        if trace.is_empty() {
            return HwFailureTraceQueueResult::InvalidTrace;
        }
        if self.attempted || self.pending.is_some() {
            return HwFailureTraceQueueResult::AlreadyQueuedOrAttempted;
        }
        self.pending = Some(trace);
        HwFailureTraceQueueResult::Queued
    }

    pub fn take_for_flush(&mut self) -> Option<HwFailureTrace> {
        let trace = self.pending.take()?;
        self.attempted = true;
        Some(trace)
    }

    pub fn attempted(&self) -> bool {
        self.attempted
    }
}

impl Default for HwFailureTraceLatch {
    fn default() -> Self {
        Self::new()
    }
}

pub fn validate_hw_failure_append_target(
    target: &[u8; HW_FAILURE_TRACE_FRAME_LEN],
    append_lba: u64,
    reclog_end_lba: u64,
) -> Result<(), &'static str> {
    if append_lba >= reclog_end_lba {
        return Err("reclog_full");
    }
    if target.iter().any(|byte| *byte != 0) {
        return Err("reclog_target_not_zero");
    }
    Ok(())
}

pub fn encode_hw_failure_trace_frame(
    trace: &HwFailureTrace,
    seq: u64,
    prev_frame_sha256: [u8; 32],
) -> Result<[u8; HW_FAILURE_TRACE_FRAME_LEN], &'static str> {
    if seq == 0 {
        return Err("reclog_seq_zero");
    }
    let payload = trace.payload()?;
    let payload = payload.as_bytes();
    let mut frame = [0u8; HW_FAILURE_TRACE_FRAME_LEN];
    frame[..RECLOG_MAGIC.len()].copy_from_slice(RECLOG_MAGIC);
    write_le32(&mut frame, 8, HW_FAILURE_TRACE_FRAME_LEN as u32);
    write_le32(&mut frame, 12, payload.len() as u32);
    write_le64(&mut frame, 16, seq);
    frame[RECLOG_PREV_FRAME_SHA256_OFFSET..RECLOG_PREV_FRAME_SHA256_OFFSET + 32]
        .copy_from_slice(&prev_frame_sha256);
    frame[RECLOG_PAYLOAD_SHA256_OFFSET..RECLOG_PAYLOAD_SHA256_OFFSET + 32]
        .copy_from_slice(&sha256_bytes(payload));
    frame[RECLOG_FRAME_HEADER_LEN..RECLOG_FRAME_HEADER_LEN + payload.len()]
        .copy_from_slice(payload);
    Ok(frame)
}

pub fn verify_hw_failure_trace_readback(
    expected_frame: &[u8; HW_FAILURE_TRACE_FRAME_LEN],
    readback: &[u8; HW_FAILURE_TRACE_FRAME_LEN],
    seq: u64,
    prev_frame_sha256: [u8; 32],
) -> Result<[u8; 32], &'static str> {
    if readback != expected_frame {
        return Err("reclog_readback_mismatch");
    }
    let parsed = parse_reclog_frame(readback, 0, seq, prev_frame_sha256)
        .map_err(|_| "reclog_readback_parse_failed")?;
    let payload = &readback[RECLOG_FRAME_HEADER_LEN
        ..RECLOG_FRAME_HEADER_LEN.saturating_add(parsed.payload_len as usize)];
    if !payload.starts_with(b"{\"schema\":\"raios.hw_failure_trace.v0\",") {
        return Err("hw_failure_trace_schema_mismatch");
    }
    Ok(parsed.frame_sha256)
}

fn write_le32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_le64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_trace() -> HwFailureTrace {
        let mut trace = HwFailureTrace::new(
            SEED_KERNEL_BUILD_ID_V0_1_0,
            HwFailureSubsystem::MarvellWifiPcie,
        );
        trace
            .push_step(HwFailureTraceStep {
                boot_ms: 120,
                phase: HwFailurePhase::FirmwareTransport,
                status: HwFailureStatus::Completed,
                register: HwFailureRegister::MarvellFirmwareStatus,
                register_value: 1,
            })
            .unwrap();
        trace
            .push_step(HwFailureTraceStep {
                boot_ms: 940,
                phase: HwFailurePhase::NetworkStateGrant,
                status: HwFailureStatus::BootPostureDenied,
                register: HwFailureRegister::MarvellCommandResponseStatus,
                register_value: 7,
            })
            .unwrap();
        trace
    }

    #[test]
    fn payload_is_bounded_ordered_and_has_no_free_form_secret_surface() {
        let trace = sample_trace();
        let payload = trace.payload().unwrap();
        assert!(payload.len() <= HW_FAILURE_TRACE_MAX_PAYLOAD_LEN);
        assert!(payload.contains("\"classification\":\"local_only\""));
        assert!(payload.contains("[120,1,2,3,1],[940,8,108,4,7]"));
        for forbidden in [
            "correct horse battery staple",
            "HomeWifi",
            "aa:bb:cc:dd:ee:ff",
            "sk-live-secret-token",
            "ssid",
            "password",
            "bssid",
            "mac",
            "token",
        ] {
            assert!(!payload.to_ascii_lowercase().contains(forbidden));
        }
    }

    #[test]
    fn maximal_numeric_trace_still_fits_one_sector() {
        let mut trace = HwFailureTrace::new(u64::MAX, HwFailureSubsystem::MarvellWifiPcie);
        for _ in 0..HW_FAILURE_TRACE_MAX_STEPS {
            trace
                .push_step(HwFailureTraceStep {
                    boot_ms: u32::MAX,
                    phase: HwFailurePhase::NetworkStateGrant,
                    status: HwFailureStatus::NetworkStateNotGranted,
                    register: HwFailureRegister::XhciPortStatusChange,
                    register_value: u32::MAX,
                })
                .unwrap();
        }
        assert!(trace.payload().unwrap().len() <= HW_FAILURE_TRACE_MAX_PAYLOAD_LEN);
        assert_eq!(
            trace.push_step(HwFailureTraceStep::EMPTY),
            Err("hw_failure_trace_step_limit")
        );
    }

    #[test]
    fn trace_times_must_be_boot_relative_and_monotonic() {
        let mut trace = HwFailureTrace::new(1, HwFailureSubsystem::MarvellWifiPcie);
        trace
            .push_step(HwFailureTraceStep {
                boot_ms: 2,
                ..HwFailureTraceStep::EMPTY
            })
            .unwrap();
        assert_eq!(
            trace.push_step(HwFailureTraceStep {
                boot_ms: 1,
                ..HwFailureTraceStep::EMPTY
            }),
            Err("hw_failure_trace_time_not_monotonic")
        );
    }

    #[test]
    fn latch_allows_one_trace_and_one_flush_attempt_per_boot() {
        let mut latch = HwFailureTraceLatch::new();
        assert_eq!(
            latch.queue(HwFailureTrace::new(
                SEED_KERNEL_BUILD_ID_V0_1_0,
                HwFailureSubsystem::MarvellWifiPcie,
            )),
            HwFailureTraceQueueResult::InvalidTrace
        );
        assert!(!latch.attempted());
        assert_eq!(
            latch.queue(sample_trace()),
            HwFailureTraceQueueResult::Queued
        );
        assert_eq!(
            latch.queue(sample_trace()),
            HwFailureTraceQueueResult::AlreadyQueuedOrAttempted
        );
        assert!(latch.take_for_flush().is_some());
        assert!(latch.attempted());
        assert!(latch.take_for_flush().is_none());
        assert_eq!(
            latch.queue(sample_trace()),
            HwFailureTraceQueueResult::AlreadyQueuedOrAttempted
        );
    }

    #[test]
    fn torn_or_full_append_target_is_denied_without_write() {
        let zero = [0u8; HW_FAILURE_TRACE_FRAME_LEN];
        assert_eq!(validate_hw_failure_append_target(&zero, 99, 100), Ok(()));
        assert_eq!(
            validate_hw_failure_append_target(&zero, 100, 100),
            Err("reclog_full")
        );
        let mut torn = zero;
        torn[17] = 1;
        assert_eq!(
            validate_hw_failure_append_target(&torn, 99, 100),
            Err("reclog_target_not_zero")
        );
    }

    #[test]
    fn frame_readback_is_hash_chain_verified() {
        let previous = [0x42; 32];
        let frame = encode_hw_failure_trace_frame(&sample_trace(), 7, previous).unwrap();
        let frame_hash = verify_hw_failure_trace_readback(&frame, &frame, 7, previous).unwrap();
        assert_eq!(frame_hash, sha256_bytes(&frame));

        let mut changed = frame;
        changed[RECLOG_FRAME_HEADER_LEN + 2] ^= 1;
        assert_eq!(
            verify_hw_failure_trace_readback(&frame, &changed, 7, previous),
            Err("reclog_readback_mismatch")
        );
        assert_eq!(
            verify_hw_failure_trace_readback(&frame, &frame, 7, [0x24; 32]),
            Err("reclog_readback_parse_failed")
        );
    }
}
