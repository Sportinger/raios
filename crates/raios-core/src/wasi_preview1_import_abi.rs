//! Typed declaration ABI for the measured WASI build-guest import surface.
//!
//! This module describes imports only. It does not link or implement them.
//! Declaration order is part of the contract and is preserved by the
//! canonical encoding.

use alloc::vec::Vec;

use crate::sha256_bytes;

pub const WASI_BUILD_IMPORT_ABI_V1: &str = "raios.wasi_build_imports.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasiValueType {
    I32,
    I64,
    F32,
    F64,
    V128,
    FuncRef,
    ExternRef,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasiImportKind<'a> {
    Func {
        params: &'a [WasiValueType],
        results: &'a [WasiValueType],
    },
    Memory {
        initial: u64,
        maximum: Option<u64>,
        shared: bool,
    },
    Global {
        value_type: WasiValueType,
        mutable: bool,
    },
    Table {
        element_type: WasiValueType,
        initial: u64,
        maximum: Option<u64>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WasiImportDeclaration<'a> {
    pub module: &'a str,
    pub name: &'a str,
    pub kind: WasiImportKind<'a>,
}

impl<'a> WasiImportDeclaration<'a> {
    pub const fn func(
        module: &'a str,
        name: &'a str,
        params: &'a [WasiValueType],
        results: &'a [WasiValueType],
    ) -> Self {
        Self {
            module,
            name,
            kind: WasiImportKind::Func { params, results },
        }
    }
}

/// Canonical binary encoding of an ordered import declaration list.
///
/// The encoding is domain-separated by the ABI ID. Counts and string lengths
/// are little-endian `u64`; strings are raw UTF-8; variants and value types use
/// the fixed tags below. No sorting or map iteration occurs.
pub fn wasi_import_declarations_canonical_bytes(imports: &[WasiImportDeclaration<'_>]) -> Vec<u8> {
    let mut bytes = Vec::new();
    write_bytes(&mut bytes, WASI_BUILD_IMPORT_ABI_V1.as_bytes());
    write_len(&mut bytes, imports.len());

    for import in imports {
        write_bytes(&mut bytes, import.module.as_bytes());
        write_bytes(&mut bytes, import.name.as_bytes());
        match import.kind {
            WasiImportKind::Func { params, results } => {
                bytes.push(0);
                write_value_types(&mut bytes, params);
                write_value_types(&mut bytes, results);
            }
            WasiImportKind::Memory {
                initial,
                maximum,
                shared,
            } => {
                bytes.push(1);
                bytes.extend_from_slice(&initial.to_le_bytes());
                write_optional_u64(&mut bytes, maximum);
                bytes.push(u8::from(shared));
            }
            WasiImportKind::Global {
                value_type,
                mutable,
            } => {
                bytes.push(2);
                bytes.push(value_type_tag(value_type));
                bytes.push(u8::from(mutable));
            }
            WasiImportKind::Table {
                element_type,
                initial,
                maximum,
            } => {
                bytes.push(3);
                bytes.push(value_type_tag(element_type));
                bytes.extend_from_slice(&initial.to_le_bytes());
                write_optional_u64(&mut bytes, maximum);
            }
        }
    }
    bytes
}

pub fn wasi_import_declarations_sha256(imports: &[WasiImportDeclaration<'_>]) -> [u8; 32] {
    sha256_bytes(&wasi_import_declarations_canonical_bytes(imports))
}

fn write_len(output: &mut Vec<u8>, len: usize) {
    output.extend_from_slice(&(len as u64).to_le_bytes());
}

fn write_bytes(output: &mut Vec<u8>, value: &[u8]) {
    write_len(output, value.len());
    output.extend_from_slice(value);
}

fn write_value_types(output: &mut Vec<u8>, values: &[WasiValueType]) {
    write_len(output, values.len());
    for value in values {
        output.push(value_type_tag(*value));
    }
}

fn write_optional_u64(output: &mut Vec<u8>, value: Option<u64>) {
    match value {
        Some(value) => {
            output.push(1);
            output.extend_from_slice(&value.to_le_bytes());
        }
        None => output.push(0),
    }
}

fn value_type_tag(value: WasiValueType) -> u8 {
    match value {
        WasiValueType::I32 => 0,
        WasiValueType::I64 => 1,
        WasiValueType::F32 => 2,
        WasiValueType::F64 => 3,
        WasiValueType::V128 => 4,
        WasiValueType::FuncRef => 5,
        WasiValueType::ExternRef => 6,
    }
}

const I32: WasiValueType = WasiValueType::I32;
const I64: WasiValueType = WasiValueType::I64;
const R_I32: &[WasiValueType] = &[I32];
const P_0: &[WasiValueType] = &[];
const P_I32: &[WasiValueType] = &[I32];
const P_2I32: &[WasiValueType] = &[I32, I32];
const P_3I32: &[WasiValueType] = &[I32, I32, I32];
const P_4I32: &[WasiValueType] = &[I32, I32, I32, I32];
const P_5I32: &[WasiValueType] = &[I32, I32, I32, I32, I32];
const P_6I32: &[WasiValueType] = &[I32, I32, I32, I32, I32, I32];
const P_7I32: &[WasiValueType] = &[I32, I32, I32, I32, I32, I32, I32];
const P_I32_I64: &[WasiValueType] = &[I32, I64];
const P_I32_I64_2I32: &[WasiValueType] = &[I32, I64, I32, I32];
const P_3I32_I64_I32: &[WasiValueType] = &[I32, I32, I32, I64, I32];
const P_5I32_2I64_2I32: &[WasiValueType] = &[I32, I32, I32, I32, I32, I64, I64, I32, I32];

/// Typed, binary-order reference surface measured from compiler artifact
/// `c6dccf3e5f01631b942a0a008b9f2f5312987e7d8590f8c61024cd00687a5791`.
pub const RUSTC_WASM_C6DCCF3E_IMPORTS: &[WasiImportDeclaration<'static>] = &[
    WasiImportDeclaration::func("wasi_snapshot_preview1", "random_get", P_2I32, R_I32),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "args_get", P_2I32, R_I32),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "args_sizes_get", P_2I32, R_I32),
    WasiImportDeclaration::func(
        "wasi_snapshot_preview1",
        "clock_time_get",
        &[I32, I64, I32],
        R_I32,
    ),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "fd_filestat_get", P_2I32, R_I32),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "fd_read", P_4I32, R_I32),
    WasiImportDeclaration::func(
        "wasi_snapshot_preview1",
        "fd_readdir",
        P_3I32_I64_I32,
        R_I32,
    ),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "fd_seek", P_I32_I64_2I32, R_I32),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "fd_write", P_4I32, R_I32),
    WasiImportDeclaration::func(
        "wasi_snapshot_preview1",
        "path_create_directory",
        P_3I32,
        R_I32,
    ),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "path_filestat_get", P_5I32, R_I32),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "path_link", P_7I32, R_I32),
    WasiImportDeclaration::func(
        "wasi_snapshot_preview1",
        "path_open",
        P_5I32_2I64_2I32,
        R_I32,
    ),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "path_readlink", P_6I32, R_I32),
    WasiImportDeclaration::func(
        "wasi_snapshot_preview1",
        "path_remove_directory",
        P_3I32,
        R_I32,
    ),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "path_rename", P_6I32, R_I32),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "path_unlink_file", P_3I32, R_I32),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "poll_oneoff", P_4I32, R_I32),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "sched_yield", P_0, R_I32),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "environ_get", P_2I32, R_I32),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "environ_sizes_get", P_2I32, R_I32),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "fd_close", P_I32, R_I32),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "fd_fdstat_get", P_2I32, R_I32),
    WasiImportDeclaration::func(
        "wasi_snapshot_preview1",
        "fd_filestat_set_size",
        P_I32_I64,
        R_I32,
    ),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "fd_pread", P_3I32_I64_I32, R_I32),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "fd_prestat_get", P_2I32, R_I32),
    WasiImportDeclaration::func(
        "wasi_snapshot_preview1",
        "fd_prestat_dir_name",
        P_3I32,
        R_I32,
    ),
    WasiImportDeclaration::func("wasi_snapshot_preview1", "proc_exit", P_I32, P_0),
    WasiImportDeclaration::func("wasi", "thread-spawn", P_I32, R_I32),
    WasiImportDeclaration {
        module: "env",
        name: "memory",
        kind: WasiImportKind::Memory {
            initial: 399,
            maximum: Some(16_384),
            shared: true,
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_bytes_and_hash_are_deterministic() {
        let first_bytes = wasi_import_declarations_canonical_bytes(RUSTC_WASM_C6DCCF3E_IMPORTS);
        let second_bytes = wasi_import_declarations_canonical_bytes(RUSTC_WASM_C6DCCF3E_IMPORTS);
        let first_hash = wasi_import_declarations_sha256(RUSTC_WASM_C6DCCF3E_IMPORTS);
        let second_hash = wasi_import_declarations_sha256(RUSTC_WASM_C6DCCF3E_IMPORTS);

        assert_eq!(first_bytes, second_bytes);
        assert_eq!(first_hash, second_hash);
    }

    #[test]
    fn measured_reference_has_all_thirty_typed_imports() {
        assert_eq!(RUSTC_WASM_C6DCCF3E_IMPORTS.len(), 30);
        assert_eq!(
            RUSTC_WASM_C6DCCF3E_IMPORTS[27],
            WasiImportDeclaration::func("wasi_snapshot_preview1", "proc_exit", &[I32], &[])
        );
        assert_eq!(
            RUSTC_WASM_C6DCCF3E_IMPORTS[28],
            WasiImportDeclaration::func("wasi", "thread-spawn", &[I32], &[I32])
        );
        assert_eq!(
            RUSTC_WASM_C6DCCF3E_IMPORTS[29].kind,
            WasiImportKind::Memory {
                initial: 399,
                maximum: Some(16_384),
                shared: true,
            }
        );
    }
}
