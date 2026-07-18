//! Pure validation for guest-memory ranges decoded by the kernel adapter.

use alloc::vec::Vec;
use core::ops::Range;

pub type GuestRange = Range<u32>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuestFault {
    Oob,
    Overflow,
    Unaligned,
    InvalidAlignment,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuestIovec {
    pub ptr: u32,
    pub len: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedIovecs {
    pub ranges: Vec<GuestRange>,
    pub total_len: u32,
}

/// Validates a byte range without reading or writing guest memory.
pub fn checked_range(ptr: u32, len: u32, mem_len: u32) -> Result<GuestRange, GuestFault> {
    let end = ptr.checked_add(len).ok_or(GuestFault::Overflow)?;
    if end > mem_len {
        return Err(GuestFault::Oob);
    }
    Ok(ptr..end)
}

/// Validates a naturally aligned typed range. Plain preview1 byte buffers use
/// [`checked_range`]; adapters may use this for scalar and struct accesses.
pub fn checked_aligned_range(
    ptr: u32,
    len: u32,
    mem_len: u32,
    alignment: u32,
) -> Result<GuestRange, GuestFault> {
    if alignment == 0 || !alignment.is_power_of_two() {
        return Err(GuestFault::InvalidAlignment);
    }
    if ptr & (alignment - 1) != 0 {
        return Err(GuestFault::Unaligned);
    }
    checked_range(ptr, len, mem_len)
}

/// Validates the guest byte range occupied by `count` preview1 iovec records.
pub fn checked_iovec_list(ptr: u32, count: u32, mem_len: u32) -> Result<GuestRange, GuestFault> {
    const IOVEC_SIZE: u32 = 8;
    let len = count.checked_mul(IOVEC_SIZE).ok_or(GuestFault::Overflow)?;
    checked_aligned_range(ptr, len, mem_len, 4)
}

/// Validates already-decoded `(ptr, len)` pairs and their checked total.
/// Overlap is intentionally allowed; preview1 requires bounds, not disjointness.
pub fn checked_iovecs(iovecs: &[GuestIovec], mem_len: u32) -> Result<CheckedIovecs, GuestFault> {
    let mut ranges = Vec::with_capacity(iovecs.len());
    let mut total_len = 0u32;
    for iovec in iovecs {
        let range = checked_range(iovec.ptr, iovec.len, mem_len)?;
        total_len = total_len
            .checked_add(iovec.len)
            .ok_or(GuestFault::Overflow)?;
        ranges.push(range);
    }
    Ok(CheckedIovecs { ranges, total_len })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_wrap_and_out_of_bounds_fail_closed() {
        assert_eq!(
            checked_range(u32::MAX - 1, 4, u32::MAX),
            Err(GuestFault::Overflow)
        );
        assert_eq!(checked_range(8, 5, 12), Err(GuestFault::Oob));
        assert_eq!(checked_range(12, 0, 12), Ok(12..12));
    }

    #[test]
    fn typed_alignment_and_iovec_list_bytes_are_checked() {
        assert_eq!(
            checked_aligned_range(2, 4, 16, 4),
            Err(GuestFault::Unaligned)
        );
        assert_eq!(
            checked_aligned_range(4, 4, 16, 3),
            Err(GuestFault::InvalidAlignment)
        );
        assert_eq!(checked_iovec_list(4, 2, 20), Ok(4..20));
        assert_eq!(
            checked_iovec_list(4, u32::MAX, u32::MAX),
            Err(GuestFault::Overflow)
        );
    }

    #[test]
    fn iovec_total_overflow_is_atomic_and_overlap_is_allowed() {
        let overflowing = [
            GuestIovec {
                ptr: 0,
                len: u32::MAX,
            },
            GuestIovec { ptr: 0, len: 1 },
        ];
        let before = overflowing;
        assert_eq!(
            checked_iovecs(&overflowing, u32::MAX),
            Err(GuestFault::Overflow)
        );
        assert_eq!(overflowing, before);

        let overlapping = [GuestIovec { ptr: 4, len: 4 }, GuestIovec { ptr: 6, len: 4 }];
        let checked = checked_iovecs(&overlapping, 10).unwrap();
        assert_eq!(checked.ranges, [4..8, 6..10]);
        assert_eq!(checked.total_len, 8);
    }
}
