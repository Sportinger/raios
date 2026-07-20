//! Pure, fail-closed predicates for Marvell DMA layout and publication.
//!
//! This module models host-side validation only. It performs no hardware I/O
//! and makes no claim that an IOMMU is present or configured.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DmaSpan {
    start: u64,
    end_exclusive: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DmaSpanError {
    ZeroLength,
    AddressOverflow,
    InvalidAlignment,
    MisalignedStart,
    MisalignedLength,
    InvalidAddressWidth,
    AddressMaskExceeded,
}

impl DmaSpan {
    pub fn new(
        start: u64,
        len: u64,
        required_alignment: u64,
        dma_address_bits: u8,
    ) -> Result<Self, DmaSpanError> {
        if len == 0 {
            return Err(DmaSpanError::ZeroLength);
        }
        let end_exclusive = start
            .checked_add(len)
            .ok_or(DmaSpanError::AddressOverflow)?;
        if required_alignment == 0 || !required_alignment.is_power_of_two() {
            return Err(DmaSpanError::InvalidAlignment);
        }
        if start & (required_alignment - 1) != 0 {
            return Err(DmaSpanError::MisalignedStart);
        }
        if len & (required_alignment - 1) != 0 {
            return Err(DmaSpanError::MisalignedLength);
        }
        let address_mask = match dma_address_bits {
            1..=63 => (1u64 << dma_address_bits) - 1,
            64 => u64::MAX,
            _ => return Err(DmaSpanError::InvalidAddressWidth),
        };
        if end_exclusive - 1 > address_mask {
            return Err(DmaSpanError::AddressMaskExceeded);
        }
        Ok(Self {
            start,
            end_exclusive,
        })
    }

    pub const fn start(self) -> u64 {
        self.start
    }

    pub const fn end_exclusive(self) -> u64 {
        self.end_exclusive
    }

    pub const fn len(self) -> u64 {
        self.end_exclusive - self.start
    }

    pub const fn overlaps(self, other: Self) -> bool {
        self.start < other.end_exclusive && other.start < self.end_exclusive
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageTranslation {
    pub virtual_page_start: u64,
    pub physical_page_start: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContiguousTranslation {
    pub physical_span: DmaSpan,
    pub touched_pages: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TranslationError {
    ZeroLength,
    VirtualAddressOverflow,
    InvalidPageSize,
    TranslationCountMismatch,
    VirtualPageGap,
    VirtualPageRollback,
    UnalignedPhysicalPage,
    PhysicalPageDiscontinuity,
    PhysicalAddressOverflow,
    PhysicalSpan(DmaSpanError),
}

pub fn validate_contiguous_translation(
    virtual_start: u64,
    len: u64,
    page_size: u64,
    dma_address_bits: u8,
    pages: &[PageTranslation],
) -> Result<ContiguousTranslation, TranslationError> {
    if len == 0 {
        return Err(TranslationError::ZeroLength);
    }
    if page_size == 0 || !page_size.is_power_of_two() {
        return Err(TranslationError::InvalidPageSize);
    }
    let virtual_end = virtual_start
        .checked_add(len)
        .ok_or(TranslationError::VirtualAddressOverflow)?;
    let page_mask = page_size - 1;
    let first_virtual_page = virtual_start & !page_mask;
    let last_virtual_page = (virtual_end - 1) & !page_mask;
    let page_count_u64 = ((last_virtual_page - first_virtual_page) / page_size) + 1;
    let page_count =
        usize::try_from(page_count_u64).map_err(|_| TranslationError::TranslationCountMismatch)?;
    if pages.len() != page_count {
        return Err(TranslationError::TranslationCountMismatch);
    }

    let mut expected_virtual_page = first_virtual_page;
    let mut expected_physical_page = None;
    for page in pages {
        if page.virtual_page_start > expected_virtual_page {
            return Err(TranslationError::VirtualPageGap);
        }
        if page.virtual_page_start < expected_virtual_page {
            return Err(TranslationError::VirtualPageRollback);
        }
        if page.physical_page_start & page_mask != 0 {
            return Err(TranslationError::UnalignedPhysicalPage);
        }
        if let Some(expected) = expected_physical_page {
            if page.physical_page_start != expected {
                return Err(TranslationError::PhysicalPageDiscontinuity);
            }
        }
        expected_virtual_page = expected_virtual_page
            .checked_add(page_size)
            .ok_or(TranslationError::VirtualAddressOverflow)?;
        expected_physical_page = Some(
            page.physical_page_start
                .checked_add(page_size)
                .ok_or(TranslationError::PhysicalAddressOverflow)?,
        );
    }

    let offset_in_first_page = virtual_start & page_mask;
    let physical_start = pages[0]
        .physical_page_start
        .checked_add(offset_in_first_page)
        .ok_or(TranslationError::PhysicalAddressOverflow)?;
    let physical_span = DmaSpan::new(physical_start, len, 1, dma_address_bits)
        .map_err(TranslationError::PhysicalSpan)?;
    Ok(ContiguousTranslation {
        physical_span,
        touched_pages: page_count,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarvellDmaRegionKind {
    EventRing,
    RxRing,
    TxRing,
    CommandBuffer,
    ResponseBuffer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarvellDmaRegion {
    pub kind: MarvellDmaRegionKind,
    pub span: DmaSpan,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForeignRegionKind {
    Xhci,
    Kernel,
    Heap,
    Reclog,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForeignRegion {
    pub kind: ForeignRegionKind,
    pub span: DmaSpan,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionOverlapError {
    MarvellPair {
        left: MarvellDmaRegionKind,
        right: MarvellDmaRegionKind,
    },
    Foreign {
        dma: MarvellDmaRegionKind,
        foreign: ForeignRegionKind,
    },
}

pub fn validate_non_overlapping_regions(
    dma_regions: &[MarvellDmaRegion],
    foreign_regions: &[ForeignRegion],
) -> Result<(), RegionOverlapError> {
    let mut left = 0usize;
    while left < dma_regions.len() {
        let mut right = left + 1;
        while right < dma_regions.len() {
            if dma_regions[left].span.overlaps(dma_regions[right].span) {
                return Err(RegionOverlapError::MarvellPair {
                    left: dma_regions[left].kind,
                    right: dma_regions[right].kind,
                });
            }
            right += 1;
        }
        for foreign in foreign_regions {
            if dma_regions[left].span.overlaps(foreign.span) {
                return Err(RegionOverlapError::Foreign {
                    dma: dma_regions[left].kind,
                    foreign: foreign.kind,
                });
            }
        }
        left += 1;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RingKind {
    Event,
    Rx,
    Tx,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RingGeometry {
    pub count: u16,
    pub index_shift: u8,
    pub index_mask: u32,
    pub rollover_mask: u32,
}

impl RingKind {
    pub const fn geometry(self) -> RingGeometry {
        match self {
            Self::Event => RingGeometry {
                count: 8,
                index_shift: 0,
                index_mask: 0x0000_000f,
                rollover_mask: 0x0000_0080,
            },
            Self::Rx => RingGeometry {
                count: 32,
                index_shift: 0,
                index_mask: 0x0000_03ff,
                rollover_mask: 0x0000_0400,
            },
            Self::Tx => RingGeometry {
                count: 32,
                index_shift: 16,
                index_mask: 0x03ff_0000,
                rollover_mask: 0x0400_0000,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceRingPointer {
    pub index: u16,
    pub rollover: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DevicePointerError {
    AllOnes,
    ReservedBits,
    IndexOutOfRange,
}

fn decode_pointer_field(
    geometry: RingGeometry,
    raw: u32,
) -> Result<DeviceRingPointer, DevicePointerError> {
    let index = ((raw & geometry.index_mask) >> geometry.index_shift) as u16;
    if index >= geometry.count {
        return Err(DevicePointerError::IndexOutOfRange);
    }
    Ok(DeviceRingPointer {
        index,
        rollover: raw & geometry.rollover_mask != 0,
    })
}

fn encode_pointer_field(
    geometry: RingGeometry,
    pointer: DeviceRingPointer,
) -> Result<u32, DevicePointerError> {
    if pointer.index >= geometry.count {
        return Err(DevicePointerError::IndexOutOfRange);
    }
    Ok(((pointer.index as u32) << geometry.index_shift)
        | if pointer.rollover {
            geometry.rollover_mask
        } else {
            0
        })
}

pub fn decode_event_pointer(raw: u32) -> Result<DeviceRingPointer, DevicePointerError> {
    if raw == u32::MAX {
        return Err(DevicePointerError::AllOnes);
    }
    let geometry = RingKind::Event.geometry();
    let allowed_bits = geometry.index_mask | geometry.rollover_mask;
    if raw & !allowed_bits != 0 {
        return Err(DevicePointerError::ReservedBits);
    }
    decode_pointer_field(geometry, raw)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RxTxDevicePointers {
    pub rx: DeviceRingPointer,
    pub tx: DeviceRingPointer,
}

/// Decodes either shared RX-WR/TX-RD or RX-RD/TX-WR register as one value.
pub fn decode_rx_tx_pointer_register(raw: u32) -> Result<RxTxDevicePointers, DevicePointerError> {
    if raw == u32::MAX {
        return Err(DevicePointerError::AllOnes);
    }
    let rx = RingKind::Rx.geometry();
    let tx = RingKind::Tx.geometry();
    let allowed_bits = rx.index_mask | rx.rollover_mask | tx.index_mask | tx.rollover_mask;
    if raw & !allowed_bits != 0 {
        return Err(DevicePointerError::ReservedBits);
    }
    Ok(RxTxDevicePointers {
        rx: decode_pointer_field(rx, raw)?,
        tx: decode_pointer_field(tx, raw)?,
    })
}

pub fn compose_rx_tx_pointer_register(
    pointers: RxTxDevicePointers,
) -> Result<u32, DevicePointerError> {
    Ok(encode_pointer_field(RingKind::Rx.geometry(), pointers.rx)?
        | encode_pointer_field(RingKind::Tx.geometry(), pointers.tx)?)
}

pub fn update_rx_pointer_preserving_tx(
    raw: u32,
    rx: DeviceRingPointer,
) -> Result<u32, DevicePointerError> {
    let current = decode_rx_tx_pointer_register(raw)?;
    compose_rx_tx_pointer_register(RxTxDevicePointers { rx, tx: current.tx })
}

pub fn update_tx_pointer_preserving_rx(
    raw: u32,
    tx: DeviceRingPointer,
) -> Result<u32, DevicePointerError> {
    let current = decode_rx_tx_pointer_register(raw)?;
    compose_rx_tx_pointer_register(RxTxDevicePointers { rx: current.rx, tx })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostRingPointer {
    kind: RingKind,
    index: u16,
    rollover: bool,
}

impl HostRingPointer {
    pub const fn new(kind: RingKind) -> Self {
        Self {
            kind,
            index: 0,
            rollover: false,
        }
    }

    pub const fn index(self) -> u16 {
        self.index
    }

    pub const fn rollover(self) -> bool {
        self.rollover
    }

    pub const fn raw(self) -> u32 {
        let geometry = self.kind.geometry();
        ((self.index as u32) << geometry.index_shift)
            | if self.rollover {
                geometry.rollover_mask
            } else {
                0
            }
    }

    pub fn advance(&mut self) {
        let geometry = self.kind.geometry();
        self.index += 1;
        if self.index == geometry.count {
            self.index = 0;
            self.rollover = !self.rollover;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublicationStep {
    ResponseLow,
    ResponseHigh,
    CommandLow,
    CommandHigh,
    CommandSize,
    RingsPublished,
    Doorbell,
}

impl PublicationStep {
    const fn ordinal(self) -> u8 {
        match self {
            Self::ResponseLow => 0,
            Self::ResponseHigh => 1,
            Self::CommandLow => 2,
            Self::CommandHigh => 3,
            Self::CommandSize => 4,
            Self::RingsPublished => 5,
            Self::Doorbell => 6,
        }
    }
}

const PRE_BME_PUBLICATION_STEPS: u8 = 6;
const COMPLETE_PUBLICATION_STEPS: u8 = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BmeState {
    NeverEnabled,
    Enabled,
    DisableRequested,
    DisabledReadback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublicationError {
    EpochNotStarted,
    BmeNotEnabled,
    BmeStillLive,
    FieldsIncomplete,
    PublicationPoisoned,
    EpochMismatch,
    StepOutOfOrder,
    PublicationComplete,
    DisableNotRequested,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetrySafety {
    NeverActivated,
    BmeOffReadback,
    LiveDma,
    DisableUnconfirmed,
    MixedEpochLiveAddress,
    StaleLiveAddress,
}

impl RetrySafety {
    pub const fn is_safe(self) -> bool {
        matches!(self, Self::NeverActivated | Self::BmeOffReadback)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublicationModel {
    epoch: Option<u64>,
    next_step: u8,
    bme: BmeState,
    mixed_epoch: bool,
    stale_live_address: bool,
}

impl PublicationModel {
    pub const fn new() -> Self {
        Self {
            epoch: None,
            next_step: 0,
            bme: BmeState::NeverEnabled,
            mixed_epoch: false,
            stale_live_address: false,
        }
    }

    pub fn begin_epoch(&mut self, epoch: u64) -> Result<(), PublicationError> {
        if matches!(self.bme, BmeState::Enabled | BmeState::DisableRequested) {
            self.stale_live_address = true;
            return Err(PublicationError::BmeStillLive);
        }
        self.epoch = Some(epoch);
        self.next_step = 0;
        self.mixed_epoch = false;
        self.stale_live_address = false;
        Ok(())
    }

    pub fn enable_bme(&mut self) -> Result<(), PublicationError> {
        if self.epoch.is_none() {
            return Err(PublicationError::EpochNotStarted);
        }
        if matches!(self.bme, BmeState::Enabled | BmeState::DisableRequested) {
            return Err(PublicationError::BmeStillLive);
        }
        if self.next_step != PRE_BME_PUBLICATION_STEPS {
            return Err(PublicationError::FieldsIncomplete);
        }
        if self.mixed_epoch || self.stale_live_address {
            return Err(PublicationError::PublicationPoisoned);
        }
        self.bme = BmeState::Enabled;
        Ok(())
    }

    pub fn publish(&mut self, epoch: u64, step: PublicationStep) -> Result<(), PublicationError> {
        let active_epoch = self.epoch.ok_or(PublicationError::EpochNotStarted)?;
        if active_epoch != epoch {
            self.mixed_epoch = true;
            return Err(PublicationError::EpochMismatch);
        }
        if self.next_step >= COMPLETE_PUBLICATION_STEPS {
            return Err(PublicationError::PublicationComplete);
        }
        if step.ordinal() != self.next_step {
            self.stale_live_address = true;
            return Err(PublicationError::StepOutOfOrder);
        }
        if step == PublicationStep::Doorbell {
            if self.bme != BmeState::Enabled {
                return Err(PublicationError::BmeNotEnabled);
            }
        } else if !matches!(
            self.bme,
            BmeState::NeverEnabled | BmeState::DisabledReadback
        ) {
            self.stale_live_address = true;
            return Err(PublicationError::BmeStillLive);
        }
        self.next_step += 1;
        Ok(())
    }

    pub fn request_bme_disable(&mut self) -> Result<(), PublicationError> {
        if self.bme != BmeState::Enabled {
            return Err(PublicationError::BmeNotEnabled);
        }
        self.bme = BmeState::DisableRequested;
        Ok(())
    }

    pub fn observe_bme_readback(&mut self, enabled: bool) -> Result<(), PublicationError> {
        if self.bme != BmeState::DisableRequested {
            return Err(PublicationError::DisableNotRequested);
        }
        self.bme = if enabled {
            BmeState::Enabled
        } else {
            BmeState::DisabledReadback
        };
        Ok(())
    }

    pub const fn bme_state(self) -> BmeState {
        self.bme
    }

    pub const fn publication_complete(self) -> bool {
        self.next_step == COMPLETE_PUBLICATION_STEPS
    }

    pub const fn retry_safety(self) -> RetrySafety {
        match self.bme {
            BmeState::NeverEnabled => RetrySafety::NeverActivated,
            BmeState::DisabledReadback => RetrySafety::BmeOffReadback,
            BmeState::DisableRequested => RetrySafety::DisableUnconfirmed,
            BmeState::Enabled if self.mixed_epoch => RetrySafety::MixedEpochLiveAddress,
            BmeState::Enabled if self.stale_live_address => RetrySafety::StaleLiveAddress,
            BmeState::Enabled => RetrySafety::LiveDma,
        }
    }
}

impl Default for PublicationModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(start: u64, len: u64) -> DmaSpan {
        DmaSpan::new(start, len, 1, 64).unwrap()
    }

    #[test]
    fn dma_span_accepts_checked_half_open_bounds() {
        let checked = DmaSpan::new(0x2000, 0x1000, 0x1000, 32).unwrap();
        assert_eq!(checked.start(), 0x2000);
        assert_eq!(checked.end_exclusive(), 0x3000);
        assert_eq!(checked.len(), 0x1000);
        assert!(!checked.overlaps(span(0x3000, 1)));
        assert!(checked.overlaps(span(0x2fff, 1)));
    }

    #[test]
    fn dma_span_rejects_every_arithmetic_alignment_and_mask_boundary() {
        assert_eq!(DmaSpan::new(0, 0, 1, 64), Err(DmaSpanError::ZeroLength));
        assert_eq!(
            DmaSpan::new(u64::MAX, 1, 1, 64),
            Err(DmaSpanError::AddressOverflow)
        );
        assert_eq!(
            DmaSpan::new(0x1000, 0x1000, 0, 64),
            Err(DmaSpanError::InvalidAlignment)
        );
        assert_eq!(
            DmaSpan::new(0x1000, 0x1000, 3, 64),
            Err(DmaSpanError::InvalidAlignment)
        );
        assert_eq!(
            DmaSpan::new(0x1001, 0x1000, 0x1000, 64),
            Err(DmaSpanError::MisalignedStart)
        );
        assert_eq!(
            DmaSpan::new(0x1000, 0x1001, 0x1000, 64),
            Err(DmaSpanError::MisalignedLength)
        );
        assert_eq!(
            DmaSpan::new(0, 1, 1, 0),
            Err(DmaSpanError::InvalidAddressWidth)
        );
        assert_eq!(
            DmaSpan::new(0, 1, 1, 65),
            Err(DmaSpanError::InvalidAddressWidth)
        );
        assert_eq!(DmaSpan::new(0xff, 1, 1, 8), Ok(span(0xff, 1)));
        assert_eq!(
            DmaSpan::new(0x100, 1, 1, 8),
            Err(DmaSpanError::AddressMaskExceeded)
        );
    }

    #[test]
    fn translations_prove_every_touched_page_is_physically_contiguous() {
        let pages = [
            PageTranslation {
                virtual_page_start: 0x1000,
                physical_page_start: 0x8000,
            },
            PageTranslation {
                virtual_page_start: 0x2000,
                physical_page_start: 0x9000,
            },
            PageTranslation {
                virtual_page_start: 0x3000,
                physical_page_start: 0xa000,
            },
        ];
        let translated =
            validate_contiguous_translation(0x1800, 0x2000, 0x1000, 32, &pages).unwrap();
        assert_eq!(translated.touched_pages, 3);
        assert_eq!(translated.physical_span, span(0x8800, 0x2000));
    }

    #[test]
    fn translations_reject_gap_rollback_wrong_follow_page_and_bad_counts() {
        let good = [
            PageTranslation {
                virtual_page_start: 0x1000,
                physical_page_start: 0x8000,
            },
            PageTranslation {
                virtual_page_start: 0x2000,
                physical_page_start: 0x9000,
            },
        ];
        assert_eq!(
            validate_contiguous_translation(0x1000, 0x2000, 0x1000, 64, &good[..1]),
            Err(TranslationError::TranslationCountMismatch)
        );
        let gap = [
            good[0],
            PageTranslation {
                virtual_page_start: 0x3000,
                physical_page_start: 0x9000,
            },
        ];
        assert_eq!(
            validate_contiguous_translation(0x1000, 0x2000, 0x1000, 64, &gap),
            Err(TranslationError::VirtualPageGap)
        );
        let rollback = [
            good[0],
            PageTranslation {
                virtual_page_start: 0x1000,
                physical_page_start: 0x9000,
            },
        ];
        assert_eq!(
            validate_contiguous_translation(0x1000, 0x2000, 0x1000, 64, &rollback),
            Err(TranslationError::VirtualPageRollback)
        );
        let wrong_physical = [
            good[0],
            PageTranslation {
                virtual_page_start: 0x2000,
                physical_page_start: 0xa000,
            },
        ];
        assert_eq!(
            validate_contiguous_translation(0x1000, 0x2000, 0x1000, 64, &wrong_physical),
            Err(TranslationError::PhysicalPageDiscontinuity)
        );
        let physical_rollback = [
            good[0],
            PageTranslation {
                virtual_page_start: 0x2000,
                physical_page_start: 0x7000,
            },
        ];
        assert_eq!(
            validate_contiguous_translation(0x1000, 0x2000, 0x1000, 64, &physical_rollback),
            Err(TranslationError::PhysicalPageDiscontinuity)
        );
    }

    #[test]
    fn translations_reject_zero_overflow_alignment_and_dma_mask_edges() {
        let page = [PageTranslation {
            virtual_page_start: 0,
            physical_page_start: 0,
        }];
        assert_eq!(
            validate_contiguous_translation(0, 0, 0x1000, 64, &page),
            Err(TranslationError::ZeroLength)
        );
        assert_eq!(
            validate_contiguous_translation(u64::MAX, 2, 0x1000, 64, &page),
            Err(TranslationError::VirtualAddressOverflow)
        );
        assert_eq!(
            validate_contiguous_translation(0, 1, 3, 64, &page),
            Err(TranslationError::InvalidPageSize)
        );
        let unaligned = [PageTranslation {
            virtual_page_start: 0,
            physical_page_start: 1,
        }];
        assert_eq!(
            validate_contiguous_translation(0, 1, 0x1000, 64, &unaligned),
            Err(TranslationError::UnalignedPhysicalPage)
        );
        let masked = [PageTranslation {
            virtual_page_start: 0,
            physical_page_start: 0x1_0000,
        }];
        assert_eq!(
            validate_contiguous_translation(0, 1, 0x1000, 16, &masked),
            Err(TranslationError::PhysicalSpan(
                DmaSpanError::AddressMaskExceeded
            ))
        );
    }

    #[test]
    fn adjacent_regions_pass_but_one_byte_dma_overlap_fails() {
        let adjacent = [
            MarvellDmaRegion {
                kind: MarvellDmaRegionKind::EventRing,
                span: span(100, 100),
            },
            MarvellDmaRegion {
                kind: MarvellDmaRegionKind::RxRing,
                span: span(200, 100),
            },
        ];
        assert_eq!(validate_non_overlapping_regions(&adjacent, &[]), Ok(()));
        let overlap = [
            adjacent[0],
            MarvellDmaRegion {
                kind: MarvellDmaRegionKind::RxRing,
                span: span(199, 100),
            },
        ];
        assert_eq!(
            validate_non_overlapping_regions(&overlap, &[]),
            Err(RegionOverlapError::MarvellPair {
                left: MarvellDmaRegionKind::EventRing,
                right: MarvellDmaRegionKind::RxRing,
            })
        );
    }

    #[test]
    fn every_named_foreign_region_rejects_a_one_byte_overlap() {
        let dma = [MarvellDmaRegion {
            kind: MarvellDmaRegionKind::CommandBuffer,
            span: span(100, 100),
        }];
        for foreign_kind in [
            ForeignRegionKind::Xhci,
            ForeignRegionKind::Kernel,
            ForeignRegionKind::Heap,
            ForeignRegionKind::Reclog,
        ] {
            let foreign = [ForeignRegion {
                kind: foreign_kind,
                span: span(199, 2),
            }];
            assert_eq!(
                validate_non_overlapping_regions(&dma, &foreign),
                Err(RegionOverlapError::Foreign {
                    dma: MarvellDmaRegionKind::CommandBuffer,
                    foreign: foreign_kind,
                })
            );
            let adjacent = [ForeignRegion {
                kind: foreign_kind,
                span: span(200, 1),
            }];
            assert_eq!(validate_non_overlapping_regions(&dma, &adjacent), Ok(()));
        }
    }

    #[test]
    fn event_pointer_decoder_rejects_count_reserved_and_all_ones_boundaries() {
        assert_eq!(
            decode_event_pointer(8),
            Err(DevicePointerError::IndexOutOfRange)
        );
        assert_eq!(
            decode_event_pointer(15),
            Err(DevicePointerError::IndexOutOfRange)
        );
        assert_eq!(
            decode_event_pointer(0x10),
            Err(DevicePointerError::ReservedBits)
        );
        assert_eq!(
            decode_event_pointer(u32::MAX),
            Err(DevicePointerError::AllOnes)
        );
        assert_eq!(
            decode_event_pointer(7 | 0x80),
            Ok(DeviceRingPointer {
                index: 7,
                rollover: true,
            })
        );
    }

    #[test]
    fn combined_rx_tx_decoder_checks_both_counts_reserved_bits_and_all_ones() {
        assert_eq!(
            decode_rx_tx_pointer_register(32),
            Err(DevicePointerError::IndexOutOfRange)
        );
        assert_eq!(
            decode_rx_tx_pointer_register(32 << 16),
            Err(DevicePointerError::IndexOutOfRange)
        );
        assert_eq!(
            decode_rx_tx_pointer_register(u32::MAX),
            Err(DevicePointerError::AllOnes)
        );
        for reserved in [0x0000_0800, 0x0800_0000] {
            assert_eq!(
                decode_rx_tx_pointer_register(reserved),
                Err(DevicePointerError::ReservedBits)
            );
        }
    }

    #[test]
    fn combined_rx_tx_last_indices_rollovers_and_composition_are_exact() {
        let expected = RxTxDevicePointers {
            rx: DeviceRingPointer {
                index: 31,
                rollover: true,
            },
            tx: DeviceRingPointer {
                index: 31,
                rollover: true,
            },
        };
        let raw = 31 | 0x0000_0400 | (31 << 16) | 0x0400_0000;
        assert_eq!(decode_rx_tx_pointer_register(raw), Ok(expected));
        assert_eq!(compose_rx_tx_pointer_register(expected), Ok(raw));

        for invalid in [
            RxTxDevicePointers {
                rx: DeviceRingPointer {
                    index: 32,
                    rollover: false,
                },
                tx: expected.tx,
            },
            RxTxDevicePointers {
                rx: expected.rx,
                tx: DeviceRingPointer {
                    index: 32,
                    rollover: false,
                },
            },
        ] {
            assert_eq!(
                compose_rx_tx_pointer_register(invalid),
                Err(DevicePointerError::IndexOutOfRange)
            );
        }
    }

    #[test]
    fn shared_rx_tx_updates_preserve_every_peer_bit() {
        let initial = compose_rx_tx_pointer_register(RxTxDevicePointers {
            rx: DeviceRingPointer {
                index: 7,
                rollover: true,
            },
            tx: DeviceRingPointer {
                index: 29,
                rollover: true,
            },
        })
        .unwrap();
        let rx_bits = RingKind::Rx.geometry().index_mask | RingKind::Rx.geometry().rollover_mask;
        let tx_bits = RingKind::Tx.geometry().index_mask | RingKind::Tx.geometry().rollover_mask;

        let rx_updated = update_rx_pointer_preserving_tx(
            initial,
            DeviceRingPointer {
                index: 12,
                rollover: false,
            },
        )
        .unwrap();
        assert_eq!(rx_updated & tx_bits, initial & tx_bits);
        assert_eq!(
            decode_rx_tx_pointer_register(rx_updated).unwrap().tx.index,
            29
        );

        let tx_updated = update_tx_pointer_preserving_rx(
            rx_updated,
            DeviceRingPointer {
                index: 3,
                rollover: false,
            },
        )
        .unwrap();
        assert_eq!(tx_updated & rx_bits, rx_updated & rx_bits);
        assert_eq!(
            decode_rx_tx_pointer_register(tx_updated).unwrap().rx.index,
            12
        );
    }

    fn assert_three_exact_wraps(kind: RingKind) {
        let geometry = kind.geometry();
        let mut pointer = HostRingPointer::new(kind);
        let steps = usize::from(geometry.count) * 3;
        for step in 0..=steps {
            let expected_index = (step % usize::from(geometry.count)) as u16;
            let expected_rollover = (step / usize::from(geometry.count)) & 1 != 0;
            assert_eq!(pointer.index(), expected_index);
            assert_eq!(pointer.rollover(), expected_rollover);
            let expected_raw = ((expected_index as u32) << geometry.index_shift)
                | if expected_rollover {
                    geometry.rollover_mask
                } else {
                    0
                };
            assert_eq!(pointer.raw(), expected_raw);
            if step != steps {
                pointer.advance();
            }
        }
    }

    #[test]
    fn host_event_rx_and_tx_sequences_are_exact_for_three_wraps() {
        assert_three_exact_wraps(RingKind::Event);
        assert_three_exact_wraps(RingKind::Rx);
        assert_three_exact_wraps(RingKind::Tx);
    }

    const PRE_BME_STEPS: [PublicationStep; 6] = [
        PublicationStep::ResponseLow,
        PublicationStep::ResponseHigh,
        PublicationStep::CommandLow,
        PublicationStep::CommandHigh,
        PublicationStep::CommandSize,
        PublicationStep::RingsPublished,
    ];

    fn publish_pre_bme_fields(model: &mut PublicationModel, epoch: u64) {
        for step in PRE_BME_STEPS {
            model.publish(epoch, step).unwrap();
        }
    }

    fn active_before_doorbell(epoch: u64) -> PublicationModel {
        let mut model = PublicationModel::new();
        model.begin_epoch(epoch).unwrap();
        publish_pre_bme_fields(&mut model, epoch);
        model.enable_bme().unwrap();
        model
    }

    #[test]
    fn every_partial_pre_bme_publication_is_safe_but_cannot_enable_bme() {
        let untouched = PublicationModel::new();
        assert_eq!(untouched.retry_safety(), RetrySafety::NeverActivated);
        assert!(untouched.retry_safety().is_safe());

        for published in 0..PRE_BME_STEPS.len() {
            let mut model = PublicationModel::new();
            model.begin_epoch(7).unwrap();
            for step in &PRE_BME_STEPS[..published] {
                model.publish(7, *step).unwrap();
            }
            assert_eq!(model.enable_bme(), Err(PublicationError::FieldsIncomplete));
            assert!(model.retry_safety().is_safe());
            assert!(!model.publication_complete());
        }

        let mut ready = PublicationModel::new();
        ready.begin_epoch(7).unwrap();
        publish_pre_bme_fields(&mut ready, 7);
        assert_eq!(
            ready.publish(7, PublicationStep::Doorbell),
            Err(PublicationError::BmeNotEnabled)
        );
        assert!(ready.retry_safety().is_safe());
        ready.enable_bme().unwrap();
        assert!(!ready.retry_safety().is_safe());
        ready.publish(7, PublicationStep::Doorbell).unwrap();
        assert!(ready.publication_complete());
    }

    #[test]
    fn every_error_after_bme_activation_requires_confirmed_bme_off() {
        let mut mixed = active_before_doorbell(1);
        assert_eq!(
            mixed.publish(2, PublicationStep::Doorbell),
            Err(PublicationError::EpochMismatch)
        );
        assert_eq!(mixed.retry_safety(), RetrySafety::MixedEpochLiveAddress);
        assert!(!mixed.retry_safety().is_safe());

        let mut stale = active_before_doorbell(1);
        assert_eq!(
            stale.publish(1, PublicationStep::ResponseLow),
            Err(PublicationError::StepOutOfOrder)
        );
        assert_eq!(stale.begin_epoch(2), Err(PublicationError::BmeStillLive));
        assert_eq!(stale.retry_safety(), RetrySafety::StaleLiveAddress);
        assert!(!stale.retry_safety().is_safe());

        stale.request_bme_disable().unwrap();
        assert!(!stale.retry_safety().is_safe());
        stale.observe_bme_readback(true).unwrap();
        assert_eq!(stale.retry_safety(), RetrySafety::StaleLiveAddress);
        assert!(!stale.retry_safety().is_safe());
        stale.request_bme_disable().unwrap();
        stale.observe_bme_readback(false).unwrap();
        assert_eq!(stale.retry_safety(), RetrySafety::BmeOffReadback);
        assert!(stale.retry_safety().is_safe());

        let mut completed = active_before_doorbell(3);
        completed.publish(3, PublicationStep::Doorbell).unwrap();
        assert_eq!(
            completed.publish(3, PublicationStep::Doorbell),
            Err(PublicationError::PublicationComplete)
        );
        assert!(!completed.retry_safety().is_safe());
    }

    #[test]
    fn bad_off_state_publication_poison_cannot_become_live() {
        let mut model = PublicationModel::new();
        assert_eq!(model.enable_bme(), Err(PublicationError::EpochNotStarted));
        model.begin_epoch(9).unwrap();
        assert_eq!(
            model.publish(9, PublicationStep::ResponseHigh),
            Err(PublicationError::StepOutOfOrder)
        );
        assert!(model.retry_safety().is_safe());
        publish_pre_bme_fields(&mut model, 9);
        assert_eq!(
            model.enable_bme(),
            Err(PublicationError::PublicationPoisoned)
        );
        assert!(model.retry_safety().is_safe());
        assert_eq!(
            model.observe_bme_readback(false),
            Err(PublicationError::DisableNotRequested)
        );
    }

    #[test]
    fn new_epoch_after_old_completion_cannot_activate_old_or_mixed_addresses() {
        let mut model = active_before_doorbell(1);
        model.publish(1, PublicationStep::Doorbell).unwrap();
        model.request_bme_disable().unwrap();
        model.observe_bme_readback(false).unwrap();

        model.begin_epoch(2).unwrap();
        assert_eq!(model.retry_safety(), RetrySafety::BmeOffReadback);
        assert_eq!(model.enable_bme(), Err(PublicationError::FieldsIncomplete));
        assert!(model.retry_safety().is_safe());
        assert_eq!(
            model.publish(1, PublicationStep::ResponseLow),
            Err(PublicationError::EpochMismatch)
        );
        publish_pre_bme_fields(&mut model, 2);
        assert_eq!(
            model.enable_bme(),
            Err(PublicationError::PublicationPoisoned)
        );
        assert_eq!(model.retry_safety(), RetrySafety::BmeOffReadback);

        model.begin_epoch(2).unwrap();
        publish_pre_bme_fields(&mut model, 2);
        model.enable_bme().unwrap();
        assert_eq!(model.retry_safety(), RetrySafety::LiveDma);
        model.publish(2, PublicationStep::Doorbell).unwrap();
        assert!(model.publication_complete());
    }
}
