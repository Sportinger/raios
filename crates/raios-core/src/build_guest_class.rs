//! Typed resource contract for large, compiler-sized build guests.
//!
//! This class is separate from the small project-runtime guest classes. It
//! describes policy only: no memory is reserved or allocated by this module.

use alloc::vec::Vec;

use crate::{sha256_bytes, wasi_preview1_import_abi::WASI_BUILD_IMPORT_ABI_V1};

const BUILD_GUEST_CLASS_V1_DOMAIN: &str = "raios.build_guest_class.v1";
const WASM32_MAX_PAGES: u32 = 65_536;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SharedMemoryLimitsV1 {
    pub initial_pages: u32,
    pub max_pages: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildGuestClassV1 {
    pub shared_memory: SharedMemoryLimitsV1,
    pub thread_cap: u32,
    pub fuel_quantum: u64,
    pub max_total_fuel: u64,
    pub out_max_bytes: u64,
    pub tmp_max_bytes: u64,
    pub max_files_per_arena: u32,
    pub max_dirs_per_arena: u32,
    pub max_file_bytes: u64,
    pub max_random_get: u64,
    pub import_family: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildGuestClassValidationError {
    InitialPagesExceedMaximum { initial_pages: u32, max_pages: u32 },
    MaximumPagesExceedWasm32Limit { max_pages: u32, limit: u32 },
    ZeroThreadCap,
    ZeroFuelQuantum,
    ZeroMaxTotalFuel,
    ZeroOutMaxBytes,
    ZeroTmpMaxBytes,
    ZeroMaxFilesPerArena,
    ZeroMaxDirsPerArena,
    ZeroMaxFileBytes,
    ZeroMaxRandomGet,
}

impl BuildGuestClassV1 {
    /// Rejects invalid resource envelopes before they can authorize a job.
    pub const fn validate(&self) -> Result<(), BuildGuestClassValidationError> {
        if self.shared_memory.initial_pages > self.shared_memory.max_pages {
            return Err(BuildGuestClassValidationError::InitialPagesExceedMaximum {
                initial_pages: self.shared_memory.initial_pages,
                max_pages: self.shared_memory.max_pages,
            });
        }
        if self.shared_memory.max_pages > WASM32_MAX_PAGES {
            return Err(
                BuildGuestClassValidationError::MaximumPagesExceedWasm32Limit {
                    max_pages: self.shared_memory.max_pages,
                    limit: WASM32_MAX_PAGES,
                },
            );
        }
        if self.thread_cap == 0 {
            return Err(BuildGuestClassValidationError::ZeroThreadCap);
        }
        if self.fuel_quantum == 0 {
            return Err(BuildGuestClassValidationError::ZeroFuelQuantum);
        }
        if self.max_total_fuel == 0 {
            return Err(BuildGuestClassValidationError::ZeroMaxTotalFuel);
        }
        if self.out_max_bytes == 0 {
            return Err(BuildGuestClassValidationError::ZeroOutMaxBytes);
        }
        if self.tmp_max_bytes == 0 {
            return Err(BuildGuestClassValidationError::ZeroTmpMaxBytes);
        }
        if self.max_files_per_arena == 0 {
            return Err(BuildGuestClassValidationError::ZeroMaxFilesPerArena);
        }
        if self.max_dirs_per_arena == 0 {
            return Err(BuildGuestClassValidationError::ZeroMaxDirsPerArena);
        }
        if self.max_file_bytes == 0 {
            return Err(BuildGuestClassValidationError::ZeroMaxFileBytes);
        }
        if self.max_random_get == 0 {
            return Err(BuildGuestClassValidationError::ZeroMaxRandomGet);
        }
        Ok(())
    }
}

/// Canonical policy for the measured public rustc Wasm artifact.
///
/// The shared-memory limits are exactly the artifact's imported `env.memory`:
/// 399 initial and 16,384 maximum 64-KiB pages (1 GiB maximum). The 48-thread
/// cap exceeds ADR 0016's measured 26--32 thread demand and matches the
/// scheduler default. One fuel is one logical nanosecond, so the 1,000,000
/// quantum is 1 ms; 10,000,000,000,000 total fuel is a deliberately generous
/// patience budget of about 2.8 virtual hours and remains the job abort limit.
/// ADR 0017 gives `/out` 64 MiB, `/tmp` 256 MiB, 16,384 files and 4,096
/// directories per arena, and a 64-MiB single-file ceiling. `max_random_get`
/// mirrors `raios_wasi_preview1::MAX_RANDOM_GET` (64 KiB) so the host contract
/// and shim reject the same oversized call without coupling core to the shim.
pub const RUSTC_BUILD_GUEST_CLASS_V1: BuildGuestClassV1 = BuildGuestClassV1 {
    shared_memory: SharedMemoryLimitsV1 {
        initial_pages: 399,
        max_pages: 16_384,
    },
    thread_cap: 48,
    fuel_quantum: 1_000_000,
    max_total_fuel: 10_000_000_000_000,
    out_max_bytes: 64 * 1024 * 1024,
    tmp_max_bytes: 256 * 1024 * 1024,
    max_files_per_arena: 16_384,
    max_dirs_per_arena: 4_096,
    max_file_bytes: 64 * 1024 * 1024,
    max_random_get: 64 * 1024,
    import_family: WASI_BUILD_IMPORT_ABI_V1,
};

/// Canonically encodes every V1 field in declaration order.
///
/// Strings are a little-endian `u64` byte length followed by UTF-8 bytes;
/// integers use their declared fixed width in little-endian order. The first
/// string is the `raios.build_guest_class.v1` domain and the final string is
/// `import_family`. There is no map iteration, platform-sized integer, or
/// implicit padding in the encoding.
pub fn build_guest_class_canonical_bytes(class: &BuildGuestClassV1) -> Vec<u8> {
    let mut bytes = Vec::new();
    write_bytes(&mut bytes, BUILD_GUEST_CLASS_V1_DOMAIN.as_bytes());
    bytes.extend_from_slice(&class.shared_memory.initial_pages.to_le_bytes());
    bytes.extend_from_slice(&class.shared_memory.max_pages.to_le_bytes());
    bytes.extend_from_slice(&class.thread_cap.to_le_bytes());
    bytes.extend_from_slice(&class.fuel_quantum.to_le_bytes());
    bytes.extend_from_slice(&class.max_total_fuel.to_le_bytes());
    bytes.extend_from_slice(&class.out_max_bytes.to_le_bytes());
    bytes.extend_from_slice(&class.tmp_max_bytes.to_le_bytes());
    bytes.extend_from_slice(&class.max_files_per_arena.to_le_bytes());
    bytes.extend_from_slice(&class.max_dirs_per_arena.to_le_bytes());
    bytes.extend_from_slice(&class.max_file_bytes.to_le_bytes());
    bytes.extend_from_slice(&class.max_random_get.to_le_bytes());
    write_bytes(&mut bytes, class.import_family.as_bytes());
    bytes
}

pub fn build_guest_class_sha256(class: &BuildGuestClassV1) -> [u8; 32] {
    sha256_bytes(&build_guest_class_canonical_bytes(class))
}

fn write_bytes(output: &mut Vec<u8>, value: &[u8]) {
    output.extend_from_slice(&(value.len() as u64).to_le_bytes());
    output.extend_from_slice(value);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        sha256_hex,
        thread_scheduler::DEFAULT_THREAD_CAP,
        wasi_preview1_import_abi::{WasiImportKind, RUSTC_WASM_C6DCCF3E_IMPORTS},
    };

    fn invalid_with(mutate: impl FnOnce(&mut BuildGuestClassV1)) -> BuildGuestClassValidationError {
        let mut class = RUSTC_BUILD_GUEST_CLASS_V1;
        mutate(&mut class);
        class.validate().expect_err("mutated class must be invalid")
    }

    #[test]
    fn canonical_encoding_and_hash_are_deterministic() {
        let first_bytes = build_guest_class_canonical_bytes(&RUSTC_BUILD_GUEST_CLASS_V1);
        let second_bytes = build_guest_class_canonical_bytes(&RUSTC_BUILD_GUEST_CLASS_V1);
        assert_eq!(first_bytes, second_bytes);
        assert_eq!(
            build_guest_class_sha256(&RUSTC_BUILD_GUEST_CLASS_V1),
            build_guest_class_sha256(&RUSTC_BUILD_GUEST_CLASS_V1)
        );
    }

    #[test]
    fn rustc_build_guest_class_has_pinned_golden_hash() {
        assert_eq!(
            sha256_hex(&build_guest_class_sha256(&RUSTC_BUILD_GUEST_CLASS_V1)),
            *b"3f0d62209a069e64f2e81e0683591dae4ecb92c05a55e394870605a18159a9fa"
        );
    }

    #[test]
    fn shared_memory_matches_measured_env_memory_import() {
        let memory = RUSTC_WASM_C6DCCF3E_IMPORTS
            .iter()
            .find(|import| import.module == "env" && import.name == "memory")
            .expect("measured inventory must contain env.memory");
        assert_eq!(
            memory.kind,
            WasiImportKind::Memory {
                initial: u64::from(RUSTC_BUILD_GUEST_CLASS_V1.shared_memory.initial_pages),
                maximum: Some(u64::from(
                    RUSTC_BUILD_GUEST_CLASS_V1.shared_memory.max_pages
                )),
                shared: true,
            }
        );
    }

    #[test]
    fn thread_cap_matches_scheduler_default() {
        assert_eq!(RUSTC_BUILD_GUEST_CLASS_V1.thread_cap, DEFAULT_THREAD_CAP);
    }

    #[test]
    fn random_get_limit_matches_preview1_constant() {
        // raios-core cannot depend on the downstream shim. Read the exported
        // constant's defining module so this remains a value cross-check
        // without reversing that dependency or changing either Cargo.toml.
        let source = include_str!("../../raios-wasi-preview1/src/determinism.rs");
        let declaration = source
            .lines()
            .find(|line| line.trim_start().starts_with("pub const MAX_RANDOM_GET:"))
            .expect("preview1 must export MAX_RANDOM_GET");
        let expression = declaration
            .split_once('=')
            .expect("MAX_RANDOM_GET must have a value")
            .1
            .trim()
            .trim_end_matches(';');
        let preview1_value = expression
            .split('*')
            .map(|factor| {
                factor
                    .trim()
                    .replace('_', "")
                    .parse::<u64>()
                    .expect("MAX_RANDOM_GET factors must be integers")
            })
            .product::<u64>();

        assert_eq!(RUSTC_BUILD_GUEST_CLASS_V1.max_random_get, preview1_value);
    }

    #[test]
    fn initial_pages_must_not_exceed_maximum() {
        assert_eq!(
            invalid_with(|class| class.shared_memory.initial_pages = 16_385),
            BuildGuestClassValidationError::InitialPagesExceedMaximum {
                initial_pages: 16_385,
                max_pages: 16_384,
            }
        );
    }

    #[test]
    fn maximum_pages_must_fit_wasm32() {
        assert_eq!(
            invalid_with(|class| class.shared_memory.max_pages = 65_537),
            BuildGuestClassValidationError::MaximumPagesExceedWasm32Limit {
                max_pages: 65_537,
                limit: 65_536,
            }
        );
    }

    #[test]
    fn thread_cap_must_be_nonzero() {
        assert_eq!(
            invalid_with(|class| class.thread_cap = 0),
            BuildGuestClassValidationError::ZeroThreadCap
        );
    }

    #[test]
    fn fuel_quantum_must_be_nonzero() {
        assert_eq!(
            invalid_with(|class| class.fuel_quantum = 0),
            BuildGuestClassValidationError::ZeroFuelQuantum
        );
    }

    #[test]
    fn max_total_fuel_must_be_nonzero() {
        assert_eq!(
            invalid_with(|class| class.max_total_fuel = 0),
            BuildGuestClassValidationError::ZeroMaxTotalFuel
        );
    }

    #[test]
    fn out_quota_must_be_nonzero() {
        assert_eq!(
            invalid_with(|class| class.out_max_bytes = 0),
            BuildGuestClassValidationError::ZeroOutMaxBytes
        );
    }

    #[test]
    fn tmp_quota_must_be_nonzero() {
        assert_eq!(
            invalid_with(|class| class.tmp_max_bytes = 0),
            BuildGuestClassValidationError::ZeroTmpMaxBytes
        );
    }

    #[test]
    fn file_count_quota_must_be_nonzero() {
        assert_eq!(
            invalid_with(|class| class.max_files_per_arena = 0),
            BuildGuestClassValidationError::ZeroMaxFilesPerArena
        );
    }

    #[test]
    fn directory_count_quota_must_be_nonzero() {
        assert_eq!(
            invalid_with(|class| class.max_dirs_per_arena = 0),
            BuildGuestClassValidationError::ZeroMaxDirsPerArena
        );
    }

    #[test]
    fn single_file_quota_must_be_nonzero() {
        assert_eq!(
            invalid_with(|class| class.max_file_bytes = 0),
            BuildGuestClassValidationError::ZeroMaxFileBytes
        );
    }

    #[test]
    fn random_get_quota_must_be_nonzero() {
        assert_eq!(
            invalid_with(|class| class.max_random_get = 0),
            BuildGuestClassValidationError::ZeroMaxRandomGet
        );
    }
}
