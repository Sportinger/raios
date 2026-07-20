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
/// Scope doc firmware section: PCIE_HOST_INT_STATUS = 0xC30.
pub const PCIE_HOST_INT_STATUS: u32 = 0xC30;
/// Linux mwifiex pcie.h: PCIE_HOST_INT_MASK = 0xC34.
pub const PCIE_HOST_INT_MASK: u32 = 0xC34;
/// Scope doc firmware section: PCIE_HOST_INT_STATUS_MASK = 0xC3C.
pub const PCIE_HOST_INT_STATUS_MASK: u32 = 0xC3C;
/// Scope doc firmware section: CPU_INTR_DOOR_BELL = BIT1.
pub const CPU_INTR_DOOR_BELL: u32 = 1 << 1;
/// Scope doc firmware section: HOST_INTR_DNLD_DONE = BIT0.
pub const HOST_INTR_DNLD_DONE: u32 = 1 << 0;
/// Scope doc firmware section: HOST_INTR_UPLD_RDY = BIT1.
pub const HOST_INTR_UPLD_RDY: u32 = 1 << 1;
/// Scope doc firmware section: HOST_INTR_CMD_DONE = BIT2.
pub const HOST_INTR_CMD_DONE: u32 = 1 << 2;
/// Scope doc firmware section: HOST_INTR_EVENT_RDY = BIT3.
pub const HOST_INTR_EVENT_RDY: u32 = 1 << 3;
/// Scope doc firmware section: HOST_INTR_MASK covers all first-cut polled host bits.
pub const HOST_INTR_MASK: u32 =
    HOST_INTR_DNLD_DONE | HOST_INTR_UPLD_RDY | HOST_INTR_CMD_DONE | HOST_INTR_EVENT_RDY;
/// Scope doc firmware section: FIRMWARE_READY_PCIE = 0xfedcba00.
pub const FIRMWARE_READY_PCIE: u32 = 0xfedcba00;
/// Linux mwifiex main.h: staging buffer cap for PCIe firmware upload.
pub const MWIFIEX_UPLD_SIZE: usize = 2312;
/// Scope doc firmware section: MWIFIEX_PCIE_BLOCK_SIZE_FW_DNLD = 256.
pub const FW_BLOCK_SIZE: usize = 256;
/// Padded transfer length needed when a 2312-byte payload rounds to 256-byte chunks.
pub const FW_DMA_STAGING_SIZE: usize =
    ((MWIFIEX_UPLD_SIZE + FW_BLOCK_SIZE - 1) / FW_BLOCK_SIZE) * FW_BLOCK_SIZE;
/// Scope doc firmware section: MAX_WRITE_IOMEM_RETRY = 2.
pub const MAX_WRITE_RETRY: u32 = 2;
/// Linux mwifiex_check_fw_status sleeps 100ms between status polls.
pub const FW_READY_POLL_INTERVAL_MS: u64 = 100;
/// raiOS first hardware milestone budget: multi-second, compile-only until silicon.
pub const FW_READY_TIMEOUT_MS: u64 = 10_000;

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
    BlockPrepared,
    WaitingDoorbellAck,
    PollingReady,
    Done,
    Failed(FwError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FwAction {
    WriteBlock {
        image_offset: usize,
        payload_len: usize,
        wire_len: usize,
    },
    Retry {
        image_offset: usize,
        wire_len: usize,
    },
    RingDoorbell,
    PollDoorbellAck,
    WriteDrvReady {
        value: u32,
    },
    /// Poll the current wait register again: cmd_size while downloading,
    /// fw_status while waiting for firmware ready.
    PollFwStatus,
    Done,
    Fail(FwError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FwError {
    RetryCapExceeded,
    RetryWithoutBlock,
    EmptyFirmware,
    BlockLenOutOfRange,
    UnexpectedFwStatus,
    ImpossibleState,
}

impl FwError {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::RetryCapExceeded => "retry_cap_exceeded",
            Self::RetryWithoutBlock => "retry_without_block",
            Self::EmptyFirmware => "empty_firmware",
            Self::BlockLenOutOfRange => "block_len_out_of_range",
            Self::UnexpectedFwStatus => "unexpected_fw_status",
            Self::ImpossibleState => "impossible_state",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegWrite {
    pub offset: u32,
    pub value: u32,
}

pub const REG_WRITE_PLAN_CAPACITY: usize = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegWritePlan {
    pub writes: [Option<RegWrite>; REG_WRITE_PLAN_CAPACITY],
    pub len: usize,
}

impl RegWritePlan {
    const fn empty() -> Self {
        Self {
            writes: [None, None, None],
            len: 0,
        }
    }

    const fn one(write: RegWrite) -> Self {
        Self {
            writes: [Some(write), None, None],
            len: 1,
        }
    }

    const fn three(first: RegWrite, second: RegWrite, third: RegWrite) -> Self {
        Self {
            writes: [Some(first), Some(second), Some(third)],
            len: 3,
        }
    }

    pub const fn capacity(&self) -> usize {
        REG_WRITE_PLAN_CAPACITY
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get(&self, index: usize) -> Option<RegWrite> {
        if index < self.len {
            self.writes[index]
        } else {
            None
        }
    }
}

pub fn plan_register_writes(action: FwAction, block_dma_phys: u64) -> RegWritePlan {
    match action {
        FwAction::WriteBlock { wire_len, .. } | FwAction::Retry { wire_len, .. } => {
            // The hardware shell DMA-copies the image bytes to block_dma_phys
            // before these writes; this pure plan does not move bytes.
            RegWritePlan::three(
                RegWrite {
                    offset: CMD_ADDR_LO,
                    value: (block_dma_phys & 0xffff_ffff) as u32,
                },
                RegWrite {
                    offset: CMD_ADDR_HI,
                    value: (block_dma_phys >> 32) as u32,
                },
                RegWrite {
                    offset: CMD_SIZE,
                    value: wire_len as u32,
                },
            )
        }
        FwAction::RingDoorbell => RegWritePlan::one(RegWrite {
            offset: PCIE_CPU_INT_EVENT,
            value: CPU_INTR_DOOR_BELL,
        }),
        FwAction::WriteDrvReady { value } => RegWritePlan::one(RegWrite {
            offset: DRV_READY,
            value,
        }),
        FwAction::PollFwStatus | FwAction::PollDoorbellAck | FwAction::Done | FwAction::Fail(_) => {
            RegWritePlan::empty()
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SentBlock {
    image_offset: usize,
    payload_len: usize,
    wire_len: usize,
    advance_len: usize,
}

pub struct FirmwareDownload<'a> {
    firmware: &'a [u8],
    offset: usize,
    retries_used: u32,
    phase: FwPhase,
    last_block: Option<SentBlock>,
    pending_ack: Option<SentBlock>,
}

impl<'a> FirmwareDownload<'a> {
    pub const fn new(firmware: &'a [u8]) -> Self {
        Self {
            firmware,
            offset: 0,
            retries_used: 0,
            phase: FwPhase::Downloading,
            last_block: None,
            pending_ack: None,
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
            FwPhase::BlockPrepared => {
                self.phase = FwPhase::WaitingDoorbellAck;
                FwAction::RingDoorbell
            }
            FwPhase::WaitingDoorbellAck => self.step_waiting_doorbell_ack(observed),
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

        if self.offset >= self.firmware.len() {
            self.phase = FwPhase::PollingReady;
            return FwAction::WriteDrvReady {
                value: FIRMWARE_READY_PCIE,
            };
        }

        let requested = observed.cmd_size_reg;
        if requested == 0 {
            return FwAction::PollFwStatus;
        }
        if requested > MWIFIEX_UPLD_SIZE as u32 {
            return self.fail(FwError::BlockLenOutOfRange);
        }
        if requested & 1 != 0 {
            return self.retry_or_fail();
        }

        let requested_len = requested as usize;
        if requested_len == 0 || requested_len > MWIFIEX_UPLD_SIZE {
            return self.fail(FwError::BlockLenOutOfRange);
        }

        let remaining = self.firmware.len() - self.offset;
        let payload_len = core::cmp::min(remaining, requested_len);
        if payload_len == 0 {
            return self.fail(FwError::ImpossibleState);
        }
        let Some(wire_len) = padded_wire_len(payload_len) else {
            return self.fail(FwError::BlockLenOutOfRange);
        };

        let image_offset = self.offset;
        self.retries_used = 0;
        let block = SentBlock {
            image_offset,
            payload_len,
            wire_len,
            advance_len: payload_len,
        };
        self.last_block = Some(block);
        self.pending_ack = Some(block);
        self.phase = FwPhase::BlockPrepared;
        FwAction::WriteBlock {
            image_offset,
            payload_len,
            wire_len,
        }
    }

    fn step_waiting_doorbell_ack(&mut self, observed: RegisterReads) -> FwAction {
        if observed.int_status_reg & CPU_INTR_DOOR_BELL != 0 {
            return FwAction::PollDoorbellAck;
        }

        let Some(block) = self.pending_ack.take() else {
            return self.fail(FwError::ImpossibleState);
        };
        self.offset = match self.offset.checked_add(block.advance_len) {
            Some(offset) => offset,
            None => return self.fail(FwError::ImpossibleState),
        };
        if self.offset > self.firmware.len() {
            return self.fail(FwError::ImpossibleState);
        }
        if self.offset == self.firmware.len() {
            self.phase = FwPhase::PollingReady;
            return FwAction::WriteDrvReady {
                value: FIRMWARE_READY_PCIE,
            };
        }

        self.phase = FwPhase::Downloading;
        FwAction::PollFwStatus
    }

    fn retry_or_fail(&mut self) -> FwAction {
        if self.retries_used >= MAX_WRITE_RETRY {
            return self.fail(FwError::RetryCapExceeded);
        }
        let Some(block) = self.last_block else {
            return self.fail(FwError::RetryWithoutBlock);
        };
        self.retries_used += 1;
        self.pending_ack = Some(SentBlock {
            advance_len: 0,
            ..block
        });
        self.phase = FwPhase::BlockPrepared;
        FwAction::Retry {
            image_offset: block.image_offset,
            wire_len: block.wire_len,
        }
    }

    fn fail(&mut self, err: FwError) -> FwAction {
        self.phase = FwPhase::Failed(err);
        FwAction::Fail(err)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FwReadyPollDecision {
    Ready,
    StillDownloading,
    Timeout,
}

pub const fn decide_fw_ready_poll(
    fw_status: u32,
    elapsed_ms: u64,
    timeout_ms: u64,
) -> FwReadyPollDecision {
    if fw_status == FIRMWARE_READY_PCIE {
        FwReadyPollDecision::Ready
    } else if elapsed_ms >= timeout_ms {
        FwReadyPollDecision::Timeout
    } else {
        FwReadyPollDecision::StillDownloading
    }
}

pub const fn padded_wire_len(payload_len: usize) -> Option<usize> {
    if payload_len == 0 || payload_len > MWIFIEX_UPLD_SIZE {
        return None;
    }
    let with_pad = match payload_len.checked_add(FW_BLOCK_SIZE - 1) {
        Some(value) => value,
        None => return None,
    };
    let padded = (with_pad / FW_BLOCK_SIZE) * FW_BLOCK_SIZE;
    if padded > FW_DMA_STAGING_SIZE {
        None
    } else {
        Some(padded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block_request(len: usize) -> RegisterReads {
        RegisterReads::with_cmd_size(len as u32)
    }

    fn doorbell_pending() -> RegisterReads {
        RegisterReads {
            cmd_size_reg: 0,
            fw_status_reg: 0,
            int_status_reg: CPU_INTR_DOOR_BELL,
        }
    }

    #[test]
    fn plan_register_writes_write_block_orders_addr_low_high_padded_size() {
        let plan = plan_register_writes(
            FwAction::WriteBlock {
                image_offset: 17,
                payload_len: 64,
                wire_len: FW_BLOCK_SIZE,
            },
            0x1_2345_6000,
        );

        assert_eq!(plan.len(), 3);
        assert_eq!(
            plan.get(0),
            Some(RegWrite {
                offset: CMD_ADDR_LO,
                value: 0x2345_6000,
            })
        );
        assert_eq!(
            plan.get(1),
            Some(RegWrite {
                offset: CMD_ADDR_HI,
                value: 0x1,
            })
        );
        assert_eq!(
            plan.get(2),
            Some(RegWrite {
                offset: CMD_SIZE,
                value: FW_BLOCK_SIZE as u32,
            })
        );
        assert_eq!(plan.get(3), None);
    }

    #[test]
    fn plan_register_writes_retry_resends_previous_dma_block() {
        let plan = plan_register_writes(
            FwAction::Retry {
                image_offset: 0,
                wire_len: FW_DMA_STAGING_SIZE,
            },
            0x1_2345_6000,
        );

        assert_eq!(plan.len(), 3);
        assert_eq!(
            plan.get(0),
            Some(RegWrite {
                offset: CMD_ADDR_LO,
                value: 0x2345_6000,
            })
        );
        assert_eq!(
            plan.get(1),
            Some(RegWrite {
                offset: CMD_ADDR_HI,
                value: 0x1,
            })
        );
        assert_eq!(
            plan.get(2),
            Some(RegWrite {
                offset: CMD_SIZE,
                value: FW_DMA_STAGING_SIZE as u32,
            })
        );
    }

    #[test]
    fn plan_register_writes_ring_doorbell_writes_event_bit() {
        let plan = plan_register_writes(FwAction::RingDoorbell, 0);

        assert_eq!(plan.len(), 1);
        assert_eq!(
            plan.get(0),
            Some(RegWrite {
                offset: PCIE_CPU_INT_EVENT,
                value: CPU_INTR_DOOR_BELL,
            })
        );
        assert_eq!(plan.get(1), None);
    }

    #[test]
    fn plan_register_writes_write_drv_ready_writes_ready_magic() {
        let plan = plan_register_writes(
            FwAction::WriteDrvReady {
                value: FIRMWARE_READY_PCIE,
            },
            0,
        );

        assert_eq!(plan.len(), 1);
        assert_eq!(
            plan.get(0),
            Some(RegWrite {
                offset: DRV_READY,
                value: FIRMWARE_READY_PCIE,
            })
        );
        assert_eq!(plan.get(1), None);
    }

    #[test]
    fn plan_register_writes_read_terminal_and_retry_actions_are_empty() {
        let actions = [
            FwAction::PollFwStatus,
            FwAction::PollDoorbellAck,
            FwAction::Done,
            FwAction::Fail(FwError::UnexpectedFwStatus),
        ];

        for action in actions {
            let plan = plan_register_writes(action, 0x1_2345_6000);

            assert!(plan.is_empty());
            assert_eq!(plan.len(), 0);
            assert_eq!(plan.get(0), None);
        }
    }

    #[test]
    fn plan_register_writes_never_exceeds_fixed_capacity() {
        let actions = [
            FwAction::WriteBlock {
                image_offset: 0,
                payload_len: FW_BLOCK_SIZE,
                wire_len: FW_BLOCK_SIZE,
            },
            FwAction::Retry {
                image_offset: 0,
                wire_len: FW_BLOCK_SIZE,
            },
            FwAction::RingDoorbell,
            FwAction::PollDoorbellAck,
            FwAction::WriteDrvReady {
                value: FIRMWARE_READY_PCIE,
            },
            FwAction::PollFwStatus,
            FwAction::Done,
            FwAction::Fail(FwError::ImpossibleState),
        ];

        for action in actions {
            let plan = plan_register_writes(action, 0x1_2345_6000);

            assert!(plan.len() <= plan.capacity());
            assert_eq!(plan.capacity(), REG_WRITE_PLAN_CAPACITY);
        }
    }

    #[test]
    fn happy_path_whole_blocks_write_ring_signal_poll_done() {
        let firmware = [0x5a; MWIFIEX_UPLD_SIZE * 2];
        let mut download = FirmwareDownload::new(&firmware);

        assert_eq!(
            download.step(block_request(MWIFIEX_UPLD_SIZE)),
            FwAction::WriteBlock {
                image_offset: 0,
                payload_len: MWIFIEX_UPLD_SIZE,
                wire_len: FW_DMA_STAGING_SIZE,
            }
        );
        assert_eq!(download.offset(), 0);
        assert_eq!(download.phase(), FwPhase::BlockPrepared);
        assert_eq!(download.step(RegisterReads::zero()), FwAction::RingDoorbell);
        assert_eq!(download.phase(), FwPhase::WaitingDoorbellAck);
        assert_eq!(download.step(doorbell_pending()), FwAction::PollDoorbellAck);
        assert_eq!(download.offset(), 0);
        assert_eq!(download.step(RegisterReads::zero()), FwAction::PollFwStatus);
        assert_eq!(download.offset(), MWIFIEX_UPLD_SIZE);

        assert_eq!(
            download.step(block_request(MWIFIEX_UPLD_SIZE)),
            FwAction::WriteBlock {
                image_offset: MWIFIEX_UPLD_SIZE,
                payload_len: MWIFIEX_UPLD_SIZE,
                wire_len: FW_DMA_STAGING_SIZE,
            }
        );
        assert_eq!(download.step(RegisterReads::zero()), FwAction::RingDoorbell);
        assert_eq!(
            download.step(RegisterReads::zero()),
            FwAction::WriteDrvReady {
                value: FIRMWARE_READY_PCIE
            }
        );
        assert_eq!(download.offset(), MWIFIEX_UPLD_SIZE * 2);

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
    fn crc_error_retries_previous_block_without_advancing_again() {
        let firmware = [0x11; MWIFIEX_UPLD_SIZE * 2];
        let mut download = FirmwareDownload::new(&firmware);

        assert_eq!(
            download.step(block_request(MWIFIEX_UPLD_SIZE)),
            FwAction::WriteBlock {
                image_offset: 0,
                payload_len: MWIFIEX_UPLD_SIZE,
                wire_len: FW_DMA_STAGING_SIZE,
            }
        );
        assert_eq!(download.step(RegisterReads::zero()), FwAction::RingDoorbell);
        assert_eq!(download.step(RegisterReads::zero()), FwAction::PollFwStatus);
        assert_eq!(download.offset(), MWIFIEX_UPLD_SIZE);

        assert_eq!(
            download.step(RegisterReads::with_cmd_size((FW_BLOCK_SIZE | 1) as u32)),
            FwAction::Retry {
                image_offset: 0,
                wire_len: FW_DMA_STAGING_SIZE,
            }
        );
        assert_eq!(download.offset(), MWIFIEX_UPLD_SIZE);
        assert_eq!(download.retries_used(), 1);

        assert_eq!(download.step(RegisterReads::zero()), FwAction::RingDoorbell);
        assert_eq!(download.step(RegisterReads::zero()), FwAction::PollFwStatus);
        assert_eq!(download.offset(), MWIFIEX_UPLD_SIZE);
    }

    #[test]
    fn retry_cap_exceeded_after_max_plus_one_crc_errors() {
        let firmware = [0x22; MWIFIEX_UPLD_SIZE * 2];
        let mut download = FirmwareDownload::new(&firmware);

        assert_eq!(
            download.step(block_request(MWIFIEX_UPLD_SIZE)),
            FwAction::WriteBlock {
                image_offset: 0,
                payload_len: MWIFIEX_UPLD_SIZE,
                wire_len: FW_DMA_STAGING_SIZE,
            }
        );
        assert_eq!(download.step(RegisterReads::zero()), FwAction::RingDoorbell);
        assert_eq!(download.step(RegisterReads::zero()), FwAction::PollFwStatus);

        assert_eq!(
            download.step(RegisterReads::with_cmd_size((FW_BLOCK_SIZE | 1) as u32)),
            FwAction::Retry {
                image_offset: 0,
                wire_len: FW_DMA_STAGING_SIZE,
            }
        );
        assert_eq!(download.step(RegisterReads::zero()), FwAction::RingDoorbell);
        assert_eq!(download.step(RegisterReads::zero()), FwAction::PollFwStatus);
        assert_eq!(
            download.step(RegisterReads::with_cmd_size((FW_BLOCK_SIZE | 1) as u32)),
            FwAction::Retry {
                image_offset: 0,
                wire_len: FW_DMA_STAGING_SIZE,
            }
        );
        assert_eq!(download.step(RegisterReads::zero()), FwAction::RingDoorbell);
        assert_eq!(download.step(RegisterReads::zero()), FwAction::PollFwStatus);
        assert_eq!(
            download.step(RegisterReads::with_cmd_size((FW_BLOCK_SIZE | 1) as u32)),
            FwAction::Fail(FwError::RetryCapExceeded)
        );
        assert_eq!(FwError::RetryCapExceeded.reason(), "retry_cap_exceeded");
    }

    #[test]
    fn retry_without_prior_block_fails_closed() {
        let firmware = [0x99; FW_BLOCK_SIZE];
        let mut download = FirmwareDownload::new(&firmware);

        assert_eq!(
            download.step(RegisterReads::with_cmd_size(1)),
            FwAction::Fail(FwError::RetryWithoutBlock)
        );
        assert_eq!(FwError::RetryWithoutBlock.reason(), "retry_without_block");
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
    fn final_partial_block_uses_remaining_image_len_and_pads_wire_len() {
        let firmware = [0x33; MWIFIEX_UPLD_SIZE + 7];
        let mut download = FirmwareDownload::new(&firmware);

        assert_eq!(
            download.step(block_request(MWIFIEX_UPLD_SIZE)),
            FwAction::WriteBlock {
                image_offset: 0,
                payload_len: MWIFIEX_UPLD_SIZE,
                wire_len: FW_DMA_STAGING_SIZE,
            }
        );
        assert_eq!(download.step(RegisterReads::zero()), FwAction::RingDoorbell);
        assert_eq!(download.step(RegisterReads::zero()), FwAction::PollFwStatus);
        assert_eq!(
            download.step(block_request(MWIFIEX_UPLD_SIZE)),
            FwAction::WriteBlock {
                image_offset: MWIFIEX_UPLD_SIZE,
                payload_len: 7,
                wire_len: FW_BLOCK_SIZE,
            }
        );
    }

    #[test]
    fn fw_status_not_ready_keeps_polling_without_false_done() {
        let firmware = [0x44; FW_BLOCK_SIZE];
        let mut download = FirmwareDownload::new(&firmware);

        assert_eq!(
            download.step(block_request(FW_BLOCK_SIZE)),
            FwAction::WriteBlock {
                image_offset: 0,
                payload_len: FW_BLOCK_SIZE,
                wire_len: FW_BLOCK_SIZE,
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
            download.step(RegisterReads::with_cmd_size((MWIFIEX_UPLD_SIZE + 1) as u32)),
            FwAction::Fail(FwError::BlockLenOutOfRange)
        );
        assert_eq!(
            FwError::BlockLenOutOfRange.reason(),
            "block_len_out_of_range"
        );
    }

    #[test]
    fn padded_wire_len_rounds_payload_to_pcie_block_size() {
        assert_eq!(padded_wire_len(0), None);
        assert_eq!(padded_wire_len(1), Some(FW_BLOCK_SIZE));
        assert_eq!(padded_wire_len(FW_BLOCK_SIZE), Some(FW_BLOCK_SIZE));
        assert_eq!(padded_wire_len(FW_BLOCK_SIZE + 1), Some(FW_BLOCK_SIZE * 2));
        assert_eq!(
            padded_wire_len(MWIFIEX_UPLD_SIZE),
            Some(FW_DMA_STAGING_SIZE)
        );
        assert_eq!(padded_wire_len(MWIFIEX_UPLD_SIZE + 1), None);
    }

    #[test]
    fn fw_ready_poll_decision_is_ready_still_downloading_or_timeout() {
        assert_eq!(
            decide_fw_ready_poll(
                FIRMWARE_READY_PCIE,
                FW_READY_TIMEOUT_MS,
                FW_READY_TIMEOUT_MS
            ),
            FwReadyPollDecision::Ready
        );
        assert_eq!(
            decide_fw_ready_poll(0, FW_READY_TIMEOUT_MS - 1, FW_READY_TIMEOUT_MS),
            FwReadyPollDecision::StillDownloading
        );
        assert_eq!(
            decide_fw_ready_poll(0, FW_READY_TIMEOUT_MS, FW_READY_TIMEOUT_MS),
            FwReadyPollDecision::Timeout
        );
    }
}
