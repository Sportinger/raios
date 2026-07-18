//! Numeric errors from the WASI preview1 `errno` variant.
//!
//! Source: WebAssembly/WASI, branch `wasi-0.1`, `preview1/witx/typenames.witx`.
//! WITX variant cases are numbered from zero in declaration order.

/// The fixed preview1 errno values used by the shim core and later backends.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u16)]
pub enum Errno {
    Badf = 8,
    Exist = 20,
    Fault = 21,
    Fbig = 22,
    Inval = 28,
    Io = 29,
    Isdir = 31,
    Mfile = 33,
    Noent = 44,
    Nospc = 51,
    Notdir = 54,
    Notempty = 55,
    Perm = 63,
    Rofs = 69,
    Xdev = 75,
    Notcapable = 76,
}

impl Errno {
    pub const fn code(self) -> u16 {
        self as u16
    }
}

#[cfg(test)]
mod tests {
    use super::Errno;

    #[test]
    fn preview1_errno_codes_are_the_golden_values() {
        let golden = [
            (Errno::Badf, 8),
            (Errno::Fault, 21),
            (Errno::Inval, 28),
            (Errno::Notcapable, 76),
            (Errno::Rofs, 69),
            (Errno::Xdev, 75),
            (Errno::Nospc, 51),
            (Errno::Fbig, 22),
            (Errno::Mfile, 33),
            (Errno::Noent, 44),
            (Errno::Notdir, 54),
            (Errno::Isdir, 31),
            (Errno::Exist, 20),
            (Errno::Notempty, 55),
            (Errno::Perm, 63),
            (Errno::Io, 29),
        ];
        for (errno, code) in golden {
            assert_eq!(errno.code(), code);
        }
    }
}
