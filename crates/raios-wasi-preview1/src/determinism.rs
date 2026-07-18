//! Fuel-derived clocks and manifest-derived deterministic random bytes.

use alloc::vec::Vec;

use crate::{readonly::sha256, Errno, EPOCH_2000_NS};

/// One unit of job fuel is exactly one nanosecond of logical monotonic time.
pub const FUEL_NS: u64 = 1;
/// `random_get` is deliberately bounded so one call cannot allocate without limit.
pub const MAX_RANDOM_GET: usize = 64 * 1024;

const RANDOM_DOMAIN: &[u8] = b"raios.wasi.random.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ClockId {
    Realtime = 0,
    Monotonic = 1,
}

impl TryFrom<u32> for ClockId {
    type Error = Errno;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Realtime),
            1 => Ok(Self::Monotonic),
            _ => Err(Errno::Inval),
        }
    }
}

/// Converts the job-wide fuel counter to logical monotonic nanoseconds.
pub const fn monotonic_ns(fuel_used: u64) -> u64 {
    fuel_used
}

/// Converts the job-wide fuel counter to fake realtime at the fixed 2000 epoch.
pub const fn realtime_ns(fuel_used: u64) -> Result<u64, Errno> {
    match EPOCH_2000_NS.checked_add(monotonic_ns(fuel_used)) {
        Some(value) => Ok(value),
        None => Err(Errno::Inval),
    }
}

pub fn clock_time_get(clock_id: u32, fuel_used: u64) -> Result<u64, Errno> {
    match ClockId::try_from(clock_id)? {
        ClockId::Realtime => realtime_ns(fuel_used),
        ClockId::Monotonic => Ok(monotonic_ns(fuel_used)),
    }
}

/// A deterministic xoshiro256** byte stream.
///
/// The stream is intentionally predictable and MUST NOT be used to generate
/// keys, tokens, nonces, credentials, or any other secret (ADR 0017).
///
/// Specification: the 32-byte SHA-256 seed is split into four little-endian
/// `u64` lanes. Each lane is passed once through the SplitMix64 `next` function
/// (add `0x9e3779b97f4a7c15`, then the two Stafford-mix multiplications), producing
/// xoshiro256**'s four-word state. The unreachable-in-practice all-zero state
/// is mapped to one further SplitMix64 output in lane zero. Each xoshiro256**
/// 1.0 output is
/// `rotl(s[1] * 5, 7) * 9`; the published state transition follows. Every
/// output word is emitted least-significant byte first, and partial words are
/// retained across calls so call boundaries do not change the byte stream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeterministicRandom {
    state: [u64; 4],
    buffered: [u8; 8],
    buffered_at: u8,
}

impl DeterministicRandom {
    pub fn from_manifest_sha256(job_manifest_sha256: [u8; 32]) -> Self {
        let mut material = Vec::with_capacity(RANDOM_DOMAIN.len() + job_manifest_sha256.len());
        material.extend_from_slice(RANDOM_DOMAIN);
        material.extend_from_slice(&job_manifest_sha256);
        let seed = sha256(&material);

        let mut state = [0u64; 4];
        for (index, lane) in seed.chunks_exact(8).enumerate() {
            let lane = u64::from_le_bytes([
                lane[0], lane[1], lane[2], lane[3], lane[4], lane[5], lane[6], lane[7],
            ]);
            state[index] = splitmix64_next(lane);
        }
        if state == [0; 4] {
            state[0] = splitmix64_next(0);
        }
        Self {
            state,
            buffered: [0; 8],
            buffered_at: 8,
        }
    }

    /// Returns the next bytes without advancing state on validation failure.
    pub fn random_get(&mut self, len: usize) -> Result<Vec<u8>, Errno> {
        if len > MAX_RANDOM_GET {
            return Err(Errno::Inval);
        }
        let mut next = self.clone();
        let mut output = Vec::with_capacity(len);
        while output.len() < len {
            if next.buffered_at == 8 {
                next.buffered = next_u64(&mut next.state).to_le_bytes();
                next.buffered_at = 0;
            }
            let available = 8usize - usize::from(next.buffered_at);
            let take = available.min(len - output.len());
            let start = usize::from(next.buffered_at);
            output.extend_from_slice(&next.buffered[start..start + take]);
            next.buffered_at += take as u8;
        }
        *self = next;
        Ok(output)
    }
}

fn splitmix64_next(mut state: u64) -> u64 {
    state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut value = state;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn next_u64(state: &mut [u64; 4]) -> u64 {
    let result = state[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
    let temporary = state[1] << 17;

    state[2] ^= state[0];
    state[3] ^= state[1];
    state[1] ^= state[2];
    state[0] ^= state[3];
    state[2] ^= temporary;
    state[3] = state[3].rotate_left(45);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clocks_are_fuel_exact_and_fail_closed() {
        assert_eq!(monotonic_ns(123), 123);
        assert_eq!(realtime_ns(123), Ok(EPOCH_2000_NS + 123));
        assert_eq!(
            clock_time_get(ClockId::Realtime as u32, 9),
            Ok(EPOCH_2000_NS + 9)
        );
        assert_eq!(clock_time_get(ClockId::Monotonic as u32, 9), Ok(9));
        assert_eq!(clock_time_get(2, 9), Err(Errno::Inval));
        assert_eq!(realtime_ns(u64::MAX), Err(Errno::Inval));
    }

    #[test]
    fn random_reference_vector_and_little_endian_stream_are_pinned() {
        let mut material = RANDOM_DOMAIN.to_vec();
        material.extend_from_slice(&[0x42; 32]);
        assert_eq!(
            sha256(&material),
            [
                0xb2, 0x2e, 0xd6, 0xdb, 0x3b, 0xde, 0xba, 0xda, 0xfc, 0x73, 0xe8, 0x02, 0xb4, 0x05,
                0x92, 0xcd, 0x33, 0xb9, 0xcd, 0x3b, 0x66, 0x1c, 0xd0, 0x54, 0x79, 0xc8, 0xb0, 0x9a,
                0x7d, 0x51, 0xa6, 0xbe,
            ]
        );
        let mut random = DeterministicRandom::from_manifest_sha256([0x42; 32]);
        assert_eq!(
            random.random_get(32).unwrap(),
            [
                39, 118, 76, 116, 18, 239, 121, 103, 60, 107, 53, 192, 153, 60, 119, 217, 237, 142,
                129, 135, 221, 59, 253, 30, 59, 248, 174, 190, 255, 5, 204, 135,
            ]
        );
    }

    #[test]
    fn random_stream_depends_on_manifest_and_not_call_boundaries() {
        let mut whole = DeterministicRandom::from_manifest_sha256([3; 32]);
        let mut pieces = DeterministicRandom::from_manifest_sha256([3; 32]);
        let expected = whole.random_get(19).unwrap();
        let mut actual = pieces.random_get(3).unwrap();
        actual.extend_from_slice(&pieces.random_get(16).unwrap());
        assert_eq!(actual, expected);

        let mut other = DeterministicRandom::from_manifest_sha256([4; 32]);
        assert_ne!(other.random_get(19).unwrap(), expected);
    }

    #[test]
    fn rejected_random_request_does_not_advance_stream() {
        let mut actual = DeterministicRandom::from_manifest_sha256([7; 32]);
        let mut expected = actual.clone();
        assert_eq!(actual.random_get(MAX_RANDOM_GET + 1), Err(Errno::Inval));
        assert_eq!(actual.random_get(17), expected.random_get(17));
    }
}
