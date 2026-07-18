//! Host-side conformance surface for the vendored wasmi engine.
//!
//! The crate exists so the T-family (threads-in-the-cage) work on
//! `vendor/wasmi-0.31.2` is provable on the host, without QEMU. Tests live in
//! `tests/`; this library only offers shared helpers.

/// Builds a wasmi engine configuration used by conformance tests.
pub fn base_config() -> wasmi::Config {
    wasmi::Config::default()
}
