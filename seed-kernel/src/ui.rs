//! Transitional non-visual compatibility path for protocol and provider callers.
//!
//! The framebuffer host is `shell_host::ShellHost`; this module intentionally owns
//! no renderer or fallback path.

pub use crate::system_status::RuntimeStatus;
