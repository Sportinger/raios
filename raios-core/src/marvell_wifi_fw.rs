//! Pure firmware-download sequencer for the Marvell 88W8897.
//!
//! This is hardware-independent control logic: it performs no I/O, touches no
//! registers, and is not wired to any boot or driver path. It is
//! HARDWARE-UNTESTED. It models the mwifiex PCIe block-download protocol and is
//! grounded in `docs/marvell-88w8897-wifi-driver-scope.md`.

/// Scope doc firmware section: cmd_addr_lo (SCRATCH_0) = 0xC10.
pub const CMD_ADDR_LO: u32 = 0xC10;
/// Scope doc firmware section: cmd_addr_hi (SCRATCH_1) = 0xC14.
pub const CMD_ADDR_HI: u32 = 0xC14;
/// Scope doc firmware section: cmd_size (SCRATCH_2) = 0xC40.
pub const CMD_SIZE: u32 = 0xC40;
/// Scope doc firmware section: fw_status (SCRATCH_3) = 0xC44.
pub const FW_STATUS: u32 = 0xC44;
/// Scope doc firmware section: cmdrsp_addr_lo = 0xCD0.
pub const CMDRSP_ADDR_LO: u32 = 0xCD0;
/// Scope doc firmware section: cmdrsp_addr_hi = 0xCD4.
pub const CMDRSP_ADDR_HI: u32 = 0xCD4;
/// Scope doc firmware section: drv_rdy (SCRATCH_12) = 0xCF0.
pub const DRV_READY: u32 = 0xCF0;
/// Scope doc firmware section: fw_dump_ctrl (SCRATCH_13) = 0xCF4.
pub const FW_DUMP_CTRL: u32 = 0xCF4;
/// Scope doc firmware section: PCIE_CPU_INT_EVENT = 0xC18.
pub const PCIE_CPU_INT_EVENT: u32 = 0xC18;
/// Scope doc firmware section: PCIE_CPU_INT_STATUS = 0xC1C.
pub const PCIE_CPU_INT_STATUS: u32 = 0xC1C;
/// Scope doc firmware section: CPU_INTR_DOOR_BELL = BIT1.
pub const CPU_INTR_DOOR_BELL: u32 = 1 << 1;
/// Scope doc firmware section: FIRMWARE_READY_PCIE = 0xfedcba00.
pub const FIRMWARE_READY_PCIE: u32 = 0xfedcba00;
/// Scope doc firmware section: MWIFIEX_PCIE_BLOCK_SIZE_FW_DNLD = 256.
pub const FW_BLOCK_SIZE: usize = 256;
/// Scope doc firmware section: MAX_WRITE_IOMEM_RETRY = 2.
pub const MAX_WRITE_RETRY: u32 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegisterReads {
    pub cmd_size_reg: u32,
    pub fw_status_reg: u32,
    pub int_status_reg: u32,
}

impl RegisterReads {
    pub const fn zero() -> Self {
        Self {
            cmd_size_reg: 0,
            fw_status_reg: 0,
            int_status_reg: 0,
        }
    }

    pub const fn with_cmd_size(cmd_size_reg: u32) -> Self {
        Self {
            cmd_size_reg,
            fw_status_reg: 0,
            int_status_reg: 0,
        }
    }

    pub const fn with_fw_status(fw_status_reg: u32) -> Self {
        Self {
            cmd_size_reg: 0,
            fw_status_reg,
            int_status_reg: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FwPhase {
    Downloading,
    SignalingReady,
    PollingReady,
    Done,
    Failed(FwError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FwAction {
    WriteBlock {
        image_offset: usize,
        len: usize,
    },
    RingDoorbell,
    WriteDrvReady {
        value: u32,
    },
    /// Poll the current wait register again: cmd_size while downloading,
    /// fw_status while waiting for firmware ready.
    PollFwStatus,
    Retry {
        image_offset: usize,
    },
    Done,
    Fail(FwError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FwError {
    RetryCapExceeded,
    EmptyFirmware,
    BlockLenOutOfRange,
    DoorbellStillPending,
    UnexpectedFwStatus,
    ImpossibleState,
}

impl FwError {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::RetryCapExceeded => "retry_cap_exceeded",
            Self::EmptyFirmware => "empty_firmware",
            Self::BlockLenOutOfRange => "block_len_out_of_range",
            Self::DoorbellStillPending => "doorbell_still_pending",
            Self::UnexpectedFwStatus => "unexpected_fw_status",
            Self::ImpossibleState => "impossible_state",
        }
    }
}

pub struct FirmwareDownload<'a> {
    firmware: &'a [u8],
    offset: usize,
    retries_used: u32,
    phase: FwPhase,
    pending_doorbell_len: Option<usize>,
}

impl<'a> FirmwareDownload<'a> {
    pub const fn new(firmware: &'a [u8]) -> Self {
        Self {
            firmware,
            offset: 0,
            retries_used: 0,
            phase: FwPhase::Downloading,
            pending_doorbell_len: None,
        }
    }

    pub const fn offset(&self) -> usize {
        self.offset
    }

    pub const fn retries_used(&self) -> u32 {
        self.retries_used
    }

    pub const fn phase(&self) -> FwPhase {
        self.phase
    }

    pub fn step(&mut self, observed: RegisterReads) -> FwAction {
        if self.firmware.is_empty() {
            return self.fail(FwError::EmptyFirmware);
        }

        match self.phase {
            FwPhase::Downloading => self.step_downloading(observed),
            FwPhase::SignalingReady => {
                self.phase = FwPhase::PollingReady;
                FwAction::WriteDrvReady {
                    value: FIRMWARE_READY_PCIE,
                }
            }
            FwPhase::PollingReady => {
                if observed.fw_status_reg == FIRMWARE_READY_PCIE {
                    self.phase = FwPhase::Done;
                    FwAction::Done
                } else {
                    FwAction::PollFwStatus
                }
            }
            FwPhase::Done => FwAction::Done,
            FwPhase::Failed(err) => FwAction::Fail(err),
        }
    }

    fn step_downloading(&mut self, observed: RegisterReads) -> FwAction {
        if observed.fw_status_reg == FIRMWARE_READY_PCIE {
            return self.fail(FwError::UnexpectedFwStatus);
        }

        if let Some(len) = self.pending_doorbell_len {
            if observed.int_status_reg & CPU_INTR_DOOR_BELL != 0 {
                return self.fail(FwError::DoorbellStillPending);
            }
            self.pending_doorbell_len = None;
            self.offset = match self.offset.checked_add(len) {
                Some(offset) => offset,
                None => return self.fail(FwError::ImpossibleState),
            };
            if self.offset > self.firmware.len() {
                return self.fail(FwError::ImpossibleState);
            }
            if self.offset == self.firmware.len() {
                self.phase = FwPhase::SignalingReady;
            }
            return FwAction::RingDoorbell;
        }

        if self.offset >= self.firmware.len() {
            self.phase = FwPhase::SignalingReady;
            return FwAction::WriteDrvReady {
                value: FIRMWARE_READY_PCIE,
            };
        }

        let requested = observed.cmd_size_reg;
        if requested == 0 {
            return FwAction::PollFwStatus;
        }
        if requested & 1 != 0 {
            return self.retry_or_fail();
        }

        let requested_len = requested as usize;
        if requested_len == 0 || requested_len > FW_BLOCK_SIZE {
            return self.fail(FwError::BlockLenOutOfRange);
        }

        let remaining = self.firmware.len() - self.offset;
        let len = core::cmp::min(remaining, requested_len);
        if len == 0 {
            return self.fail(FwError::ImpossibleState);
        }

        let image_offset = self.offset;
        self.retries_used = 0;
        self.pending_doorbell_len = Some(len);
        FwAction::WriteBlock { image_offset, len }
    }

    fn retry_or_fail(&mut self) -> FwAction {
        if self.retries_used >= MAX_WRITE_RETRY {
            return self.fail(FwError::RetryCapExceeded);
        }
        self.retries_used += 1;
        FwAction::Retry {
            image_offset: self.offset,
        }
    }

    fn fail(&mut self, err: FwError) -> FwAction {
        self.phase = FwPhase::Failed(err);
        FwAction::Fail(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block_request() -> RegisterReads {
        RegisterReads::with_cmd_size(FW_BLOCK_SIZE as u32)
    }

    #[test]
    fn happy_path_whole_blocks_write_ring_signal_poll_done() {
        let firmware = [0x5a; FW_BLOCK_SIZE * 2];
        let mut download = FirmwareDownload::new(&firmware);

        assert_eq!(
            download.step(block_request()),
            FwAction::WriteBlock {
                image_offset: 0,
                len: FW_BLOCK_SIZE
            }
        );
        assert_eq!(download.step(RegisterReads::zero()), FwAction::RingDoorbell);
        assert_eq!(download.offset(), FW_BLOCK_SIZE);

        assert_eq!(
            download.step(block_request()),
            FwAction::WriteBlock {
                image_offset: FW_BLOCK_SIZE,
                len: FW_BLOCK_SIZE
            }
        );
        assert_eq!(download.step(RegisterReads::zero()), FwAction::RingDoorbell);
        assert_eq!(download.offset(), FW_BLOCK_SIZE * 2);

        assert_eq!(
            download.step(RegisterReads::zero()),
            FwAction::WriteDrvReady {
                value: FIRMWARE_READY_PCIE
            }
        );
        assert_eq!(
            download.step(RegisterReads::with_fw_status(0)),
            FwAction::PollFwStatus
        );
        assert_eq!(
            download.step(RegisterReads::with_fw_status(FIRMWARE_READY_PCIE)),
            FwAction::Done
        );
        assert_eq!(download.phase(), FwPhase::Done);
    }

    #[test]
    fn crc_error_retries_same_offset_and_increments_count() {
        let firmware = [0x11; FW_BLOCK_SIZE];
        let mut download = FirmwareDownload::new(&firmware);

        assert_eq!(
            download.step(RegisterReads::with_cmd_size(1)),
            FwAction::Retry { image_offset: 0 }
        );
        assert_eq!(download.offset(), 0);
        assert_eq!(download.retries_used(), 1);

        assert_eq!(
            download.step(block_request()),
            FwAction::WriteBlock {
                image_offset: 0,
                len: FW_BLOCK_SIZE
            }
        );
        assert_eq!(download.retries_used(), 0);
    }

    #[test]
    fn retry_cap_exceeded_after_max_plus_one_crc_errors() {
        let firmware = [0x22; FW_BLOCK_SIZE];
        let mut download = FirmwareDownload::new(&firmware);

        assert_eq!(
            download.step(RegisterReads::with_cmd_size(1)),
            FwAction::Retry { image_offset: 0 }
        );
        assert_eq!(
            download.step(RegisterReads::with_cmd_size(1)),
            FwAction::Retry { image_offset: 0 }
        );
        assert_eq!(
            download.step(RegisterReads::with_cmd_size(1)),
            FwAction::Fail(FwError::RetryCapExceeded)
        );
        assert_eq!(FwError::RetryCapExceeded.reason(), "retry_cap_exceeded");
    }

    #[test]
    fn empty_firmware_fails_closed() {
        let mut download = FirmwareDownload::new(&[]);

        assert_eq!(
            download.step(RegisterReads::zero()),
            FwAction::Fail(FwError::EmptyFirmware)
        );
        assert_eq!(FwError::EmptyFirmware.reason(), "empty_firmware");
        assert_eq!(download.phase(), FwPhase::Failed(FwError::EmptyFirmware));
    }

    #[test]
    fn final_partial_block_uses_remaining_image_len() {
        let firmware = [0x33; FW_BLOCK_SIZE + 7];
        let mut download = FirmwareDownload::new(&firmware);

        assert_eq!(
            download.step(block_request()),
            FwAction::WriteBlock {
                image_offset: 0,
                len: FW_BLOCK_SIZE
            }
        );
        assert_eq!(download.step(RegisterReads::zero()), FwAction::RingDoorbell);
        assert_eq!(
            download.step(block_request()),
            FwAction::WriteBlock {
                image_offset: FW_BLOCK_SIZE,
                len: 7
            }
        );
    }

    #[test]
    fn fw_status_not_ready_keeps_polling_without_false_done() {
        let firmware = [0x44; FW_BLOCK_SIZE];
        let mut download = FirmwareDownload::new(&firmware);

        assert_eq!(
            download.step(block_request()),
            FwAction::WriteBlock {
                image_offset: 0,
                len: FW_BLOCK_SIZE,
            }
        );
        assert_eq!(download.step(RegisterReads::zero()), FwAction::RingDoorbell);
        assert_eq!(
            download.step(RegisterReads::zero()),
            FwAction::WriteDrvReady {
                value: FIRMWARE_READY_PCIE
            }
        );

        assert_eq!(
            download.step(RegisterReads::with_fw_status(0x1234_5678)),
            FwAction::PollFwStatus
        );
        assert_eq!(
            download.step(RegisterReads::with_fw_status(0)),
            FwAction::PollFwStatus
        );
        assert_eq!(download.phase(), FwPhase::PollingReady);
    }

    #[test]
    fn zero_cmd_size_polls_without_advancing() {
        let firmware = [0x55; FW_BLOCK_SIZE];
        let mut download = FirmwareDownload::new(&firmware);

        assert_eq!(
            download.step(RegisterReads::with_cmd_size(0)),
            FwAction::PollFwStatus
        );
        assert_eq!(download.offset(), 0);
        assert_eq!(download.phase(), FwPhase::Downloading);
    }

    #[test]
    fn block_len_above_doc_limit_fails_closed() {
        let firmware = [0x66; FW_BLOCK_SIZE];
        let mut download = FirmwareDownload::new(&firmware);

        assert_eq!(
            download.step(RegisterReads::with_cmd_size((FW_BLOCK_SIZE + 2) as u32)),
            FwAction::Fail(FwError::BlockLenOutOfRange)
        );
        assert_eq!(
            FwError::BlockLenOutOfRange.reason(),
            "block_len_out_of_range"
        );
    }
}
