//! Bounded TPM 2.0 CRB/TIS MMIO command transport.
//!
//! This module moves typed command bytes to a TPM and validates bounded reply
//! framing.  It makes no policy decision and cannot assert that a Vault VMK is
//! sealed, unsealed, or eligible for automatic unlock.

use core::{hint, ptr};

use raios_core::tpm2_commands::{
    validate_response_header, Tpm2CodecError, Tpm2CommandBuffer, MAX_TPM2_COMMAND_BYTES,
    MAX_TPM2_RESPONSE_BYTES, TPM2_RESPONSE_HEADER_LEN,
};

use crate::memory;

const CRB_CONTROL_WINDOW_LEN: usize = 0x1000;
const CRB_CTRL_CANCEL: usize = 0x48;
const CRB_CTRL_START: usize = 0x4c;
const CRB_CTRL_CMD_SIZE: usize = 0x58;
const CRB_CTRL_CMD_LADDR: usize = 0x5c;
const CRB_CTRL_CMD_HADDR: usize = 0x60;
const CRB_CTRL_RSP_SIZE: usize = 0x64;
const CRB_CTRL_RSP_LADDR: usize = 0x68;
const CRB_CTRL_RSP_HADDR: usize = 0x6c;

const TIS_WINDOW_LEN: usize = 0x1000;
const TIS_ACCESS: usize = 0x0000;
const TIS_STS: usize = 0x0018;
const TIS_DATA_FIFO: usize = 0x0024;
const TIS_ACCESS_REQUEST_USE: u8 = 0x02;
const TIS_ACCESS_ACTIVE_LOCALITY: u8 = 0x20;
const TIS_ACCESS_RELINQUISH_LOCALITY: u8 = 0x20;
const TIS_STS_VALID: u32 = 0x80;
const TIS_STS_COMMAND_READY: u32 = 0x40;
const TIS_STS_GO: u32 = 0x20;
const TIS_STS_DATA_AVAIL: u32 = 0x10;
const TIS_STS_DATA_EXPECT: u32 = 0x08;

const DEFAULT_POLL_LIMIT: u32 = 100_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Tpm2TransportKind {
    Crb,
    Tis,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Tpm2TransportPlan {
    pub(crate) kind: Tpm2TransportKind,
    pub(crate) control_area: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Tpm2TransportError {
    ControlAreaMissing,
    UnsupportedStartMethod { start_method: u32 },
    AddressOverflow,
    MmioMapFailed,
    CommandWindowInvalid,
    ResponseWindowInvalid,
    CommandTooLarge,
    ResponseTooLarge,
    PollTimeout,
    ResponseMalformed(Tpm2CodecError),
}

/// Plans only the ACPI interface kinds that have a bounded local transport.
/// Parsing an ACPI table or obtaining this plan carries no seal/unseal claim.
pub(crate) fn plan_from_acpi(
    start_method: u32,
    control_area: u64,
) -> Result<Tpm2TransportPlan, Tpm2TransportError> {
    if control_area == 0 {
        return Err(Tpm2TransportError::ControlAreaMissing);
    }
    let kind = match start_method {
        7 | 8 => Tpm2TransportKind::Crb,
        6 => Tpm2TransportKind::Tis,
        _ => return Err(Tpm2TransportError::UnsupportedStartMethod { start_method }),
    };
    control_area
        .checked_add(CRB_CONTROL_WINDOW_LEN as u64)
        .ok_or(Tpm2TransportError::AddressOverflow)?;
    Ok(Tpm2TransportPlan { kind, control_area })
}

/// A mutable response buffer that zeroizes itself after a keyring operation.
/// It stays crate-private so no normal service receives raw TPM response bytes.
pub(crate) struct Tpm2ResponseBuffer {
    bytes: [u8; MAX_TPM2_RESPONSE_BYTES],
    len: usize,
}

impl Tpm2ResponseBuffer {
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

impl Drop for Tpm2ResponseBuffer {
    fn drop(&mut self) {
        self.bytes.fill(0);
        self.len = 0;
    }
}

pub(crate) enum Tpm2Transport {
    Crb(CrbTransport),
    Tis(TisTransport),
}

impl Tpm2Transport {
    pub(crate) fn open(plan: Tpm2TransportPlan) -> Result<Self, Tpm2TransportError> {
        match plan.kind {
            Tpm2TransportKind::Crb => Ok(Self::Crb(CrbTransport::open(plan.control_area)?)),
            Tpm2TransportKind::Tis => Ok(Self::Tis(TisTransport::open(plan.control_area)?)),
        }
    }

    pub(crate) fn execute(
        &mut self,
        command: &Tpm2CommandBuffer,
    ) -> Result<Tpm2ResponseBuffer, Tpm2TransportError> {
        match self {
            Self::Crb(transport) => transport.execute(command),
            Self::Tis(transport) => transport.execute(command),
        }
    }
}

struct MmioRegion {
    base: *mut u8,
    len: usize,
}

impl MmioRegion {
    fn map(physical: u64, len: usize) -> Result<Self, Tpm2TransportError> {
        if len == 0 || physical.checked_add(len as u64).is_none() {
            return Err(Tpm2TransportError::AddressOverflow);
        }
        let mapping =
            memory::map_mmio(physical, len).map_err(|_| Tpm2TransportError::MmioMapFailed)?;
        Ok(Self {
            base: mapping.as_ptr::<u8>(),
            len: mapping.len(),
        })
    }

    fn contains(&self, offset: usize, width: usize) -> bool {
        offset.checked_add(width).is_some_and(|end| end <= self.len)
    }

    unsafe fn read_u8(&self, offset: usize) -> u8 {
        ptr::read_volatile(self.base.add(offset))
    }

    unsafe fn write_u8(&self, offset: usize, value: u8) {
        ptr::write_volatile(self.base.add(offset), value);
    }

    unsafe fn read_u32(&self, offset: usize) -> u32 {
        ptr::read_volatile(self.base.add(offset).cast::<u32>())
    }

    unsafe fn write_u32(&self, offset: usize, value: u32) {
        ptr::write_volatile(self.base.add(offset).cast::<u32>(), value);
    }
}

pub(crate) struct CrbTransport {
    control: MmioRegion,
    command: MmioRegion,
    response: MmioRegion,
    poll_limit: u32,
}

impl CrbTransport {
    fn open(control_area: u64) -> Result<Self, Tpm2TransportError> {
        let control = MmioRegion::map(control_area, CRB_CONTROL_WINDOW_LEN)?;
        if !control.contains(CRB_CTRL_RSP_HADDR, 4) {
            return Err(Tpm2TransportError::CommandWindowInvalid);
        }
        let command_size = unsafe { control.read_u32(CRB_CTRL_CMD_SIZE) } as usize;
        let response_size = unsafe { control.read_u32(CRB_CTRL_RSP_SIZE) } as usize;
        let command_addr = read_u64_pair(&control, CRB_CTRL_CMD_LADDR, CRB_CTRL_CMD_HADDR)?;
        let response_addr = read_u64_pair(&control, CRB_CTRL_RSP_LADDR, CRB_CTRL_RSP_HADDR)?;
        let command_len = bounded_window_len(command_addr, command_size)
            .ok_or(Tpm2TransportError::CommandWindowInvalid)?;
        let response_len = bounded_window_len(response_addr, response_size)
            .ok_or(Tpm2TransportError::ResponseWindowInvalid)?;

        Ok(Self {
            control,
            command: MmioRegion::map(command_addr, command_len)?,
            response: MmioRegion::map(response_addr, response_len)?,
            poll_limit: DEFAULT_POLL_LIMIT,
        })
    }

    fn execute(
        &mut self,
        command: &Tpm2CommandBuffer,
    ) -> Result<Tpm2ResponseBuffer, Tpm2TransportError> {
        if command.len() > self.command.len {
            return Err(Tpm2TransportError::CommandTooLarge);
        }
        for (index, byte) in command.as_bytes().iter().copied().enumerate() {
            unsafe { self.command.write_u8(index, byte) };
        }
        unsafe { self.control.write_u32(CRB_CTRL_START, 1) };
        if !poll(self.poll_limit, || unsafe {
            self.control.read_u32(CRB_CTRL_START) & 1 == 0
        }) {
            unsafe { self.control.write_u32(CRB_CTRL_CANCEL, 1) };
            self.zero_command(command.len());
            return Err(Tpm2TransportError::PollTimeout);
        }

        let result = self.read_response();
        self.zero_command(command.len());
        result
    }

    fn read_response(&self) -> Result<Tpm2ResponseBuffer, Tpm2TransportError> {
        if self.response.len < TPM2_RESPONSE_HEADER_LEN {
            return Err(Tpm2TransportError::ResponseWindowInvalid);
        }
        let mut out = Tpm2ResponseBuffer {
            bytes: [0; MAX_TPM2_RESPONSE_BYTES],
            len: TPM2_RESPONSE_HEADER_LEN,
        };
        for index in 0..TPM2_RESPONSE_HEADER_LEN {
            out.bytes[index] = unsafe { self.response.read_u8(index) };
        }
        let response_len = declared_response_len(&out.bytes[..TPM2_RESPONSE_HEADER_LEN])?;
        if response_len > self.response.len || response_len > MAX_TPM2_RESPONSE_BYTES {
            return Err(Tpm2TransportError::ResponseTooLarge);
        }
        for index in TPM2_RESPONSE_HEADER_LEN..response_len {
            out.bytes[index] = unsafe { self.response.read_u8(index) };
        }
        out.len = response_len;
        validate_response_header(out.as_bytes()).map_err(Tpm2TransportError::ResponseMalformed)?;
        Ok(out)
    }

    fn zero_command(&self, len: usize) {
        for index in 0..len {
            unsafe { self.command.write_u8(index, 0) };
        }
    }
}

pub(crate) struct TisTransport {
    region: MmioRegion,
    poll_limit: u32,
}

impl TisTransport {
    fn open(control_area: u64) -> Result<Self, Tpm2TransportError> {
        Ok(Self {
            region: MmioRegion::map(control_area, TIS_WINDOW_LEN)?,
            poll_limit: DEFAULT_POLL_LIMIT,
        })
    }

    fn execute(
        &mut self,
        command: &Tpm2CommandBuffer,
    ) -> Result<Tpm2ResponseBuffer, Tpm2TransportError> {
        if command.len() > MAX_TPM2_COMMAND_BYTES {
            return Err(Tpm2TransportError::CommandTooLarge);
        }
        unsafe { self.region.write_u8(TIS_ACCESS, TIS_ACCESS_REQUEST_USE) };
        if !poll(self.poll_limit, || unsafe {
            self.region.read_u8(TIS_ACCESS) & TIS_ACCESS_ACTIVE_LOCALITY != 0
        }) {
            self.release_locality();
            return Err(Tpm2TransportError::PollTimeout);
        }

        unsafe { self.region.write_u32(TIS_STS, TIS_STS_COMMAND_READY) };
        if !poll(self.poll_limit, || self.command_ready()) {
            self.release_locality();
            return Err(Tpm2TransportError::PollTimeout);
        }

        for byte in command.as_bytes().iter().copied() {
            if !poll(self.poll_limit, || self.data_expected()) {
                self.abort_command();
                return Err(Tpm2TransportError::PollTimeout);
            }
            unsafe { self.region.write_u8(TIS_DATA_FIFO, byte) };
        }
        if self.data_expected() {
            self.abort_command();
            return Err(Tpm2TransportError::CommandTooLarge);
        }
        unsafe { self.region.write_u32(TIS_STS, TIS_STS_GO) };

        let result = self.read_response();
        self.abort_command();
        result
    }

    fn read_response(&self) -> Result<Tpm2ResponseBuffer, Tpm2TransportError> {
        let mut out = Tpm2ResponseBuffer {
            bytes: [0; MAX_TPM2_RESPONSE_BYTES],
            len: TPM2_RESPONSE_HEADER_LEN,
        };
        for index in 0..TPM2_RESPONSE_HEADER_LEN {
            if !poll(self.poll_limit, || self.response_available()) {
                return Err(Tpm2TransportError::PollTimeout);
            }
            out.bytes[index] = unsafe { self.region.read_u8(TIS_DATA_FIFO) };
        }
        let response_len = declared_response_len(&out.bytes[..TPM2_RESPONSE_HEADER_LEN])?;
        if response_len > MAX_TPM2_RESPONSE_BYTES {
            return Err(Tpm2TransportError::ResponseTooLarge);
        }
        for index in TPM2_RESPONSE_HEADER_LEN..response_len {
            if !poll(self.poll_limit, || self.response_available()) {
                return Err(Tpm2TransportError::PollTimeout);
            }
            out.bytes[index] = unsafe { self.region.read_u8(TIS_DATA_FIFO) };
        }
        out.len = response_len;
        validate_response_header(out.as_bytes()).map_err(Tpm2TransportError::ResponseMalformed)?;
        Ok(out)
    }

    fn command_ready(&self) -> bool {
        let status = unsafe { self.region.read_u32(TIS_STS) };
        status & (TIS_STS_VALID | TIS_STS_COMMAND_READY) == (TIS_STS_VALID | TIS_STS_COMMAND_READY)
    }

    fn data_expected(&self) -> bool {
        let status = unsafe { self.region.read_u32(TIS_STS) };
        status & (TIS_STS_VALID | TIS_STS_DATA_EXPECT) == (TIS_STS_VALID | TIS_STS_DATA_EXPECT)
    }

    fn response_available(&self) -> bool {
        let status = unsafe { self.region.read_u32(TIS_STS) };
        status & (TIS_STS_VALID | TIS_STS_DATA_AVAIL) == (TIS_STS_VALID | TIS_STS_DATA_AVAIL)
    }

    fn abort_command(&self) {
        unsafe { self.region.write_u32(TIS_STS, TIS_STS_COMMAND_READY) };
        self.release_locality();
    }

    fn release_locality(&self) {
        unsafe {
            self.region
                .write_u8(TIS_ACCESS, TIS_ACCESS_RELINQUISH_LOCALITY)
        };
    }
}

fn read_u64_pair(
    region: &MmioRegion,
    low_offset: usize,
    high_offset: usize,
) -> Result<u64, Tpm2TransportError> {
    if !region.contains(low_offset, 4) || !region.contains(high_offset, 4) {
        return Err(Tpm2TransportError::CommandWindowInvalid);
    }
    let low = unsafe { region.read_u32(low_offset) } as u64;
    let high = unsafe { region.read_u32(high_offset) } as u64;
    let address = low | (high << 32);
    if address == 0 {
        return Err(Tpm2TransportError::CommandWindowInvalid);
    }
    Ok(address)
}

fn bounded_window_len(address: u64, reported_len: usize) -> Option<usize> {
    if reported_len < TPM2_RESPONSE_HEADER_LEN || address == 0 {
        return None;
    }
    let bounded = reported_len.min(MAX_TPM2_COMMAND_BYTES);
    address.checked_add(bounded as u64)?;
    Some(bounded)
}

fn declared_response_len(header: &[u8]) -> Result<usize, Tpm2TransportError> {
    if header.len() != TPM2_RESPONSE_HEADER_LEN {
        return Err(Tpm2TransportError::ResponseMalformed(
            Tpm2CodecError::ResponseTooShort,
        ));
    }
    let len = u32::from_be_bytes([header[2], header[3], header[4], header[5]]) as usize;
    if !(TPM2_RESPONSE_HEADER_LEN..=MAX_TPM2_RESPONSE_BYTES).contains(&len) {
        return Err(Tpm2TransportError::ResponseTooLarge);
    }
    Ok(len)
}

fn poll(mut limit: u32, mut ready: impl FnMut() -> bool) -> bool {
    while limit != 0 {
        if ready() {
            return true;
        }
        hint::spin_loop();
        limit -= 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acpi_plan_accepts_only_bounded_crb_or_tis_interfaces() {
        assert_eq!(
            plan_from_acpi(7, 0xfed4_0000).unwrap(),
            Tpm2TransportPlan {
                kind: Tpm2TransportKind::Crb,
                control_area: 0xfed4_0000,
            }
        );
        assert_eq!(
            plan_from_acpi(6, 0xfed4_0000).unwrap().kind,
            Tpm2TransportKind::Tis
        );
        assert_eq!(
            plan_from_acpi(13, 0xfed4_0000),
            Err(Tpm2TransportError::UnsupportedStartMethod { start_method: 13 })
        );
        assert_eq!(
            plan_from_acpi(7, 0),
            Err(Tpm2TransportError::ControlAreaMissing)
        );
    }

    #[test]
    fn window_and_response_lengths_fail_closed() {
        assert_eq!(bounded_window_len(0, 64), None);
        assert_eq!(bounded_window_len(0x1000, 9), None);
        assert_eq!(bounded_window_len(u64::MAX - 1, 64), None);
        assert_eq!(
            bounded_window_len(0x1000, 8192),
            Some(MAX_TPM2_COMMAND_BYTES)
        );

        let mut header = [0u8; TPM2_RESPONSE_HEADER_LEN];
        header[2..6].copy_from_slice(&9u32.to_be_bytes());
        assert_eq!(
            declared_response_len(&header),
            Err(Tpm2TransportError::ResponseTooLarge)
        );
    }
}
