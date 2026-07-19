//! Executable conformance check for the frozen Genesis floor v1.

use crate::{Import, ImportKind, Inventory, ValueType};
use raios_core::scoped_wasm_import_grant::KNOWN_HOST_IMPORTS;
use raios_core::wasi_preview1_import_abi::{
    wasi_import_declarations_sha256, WasiImportKind, WasiValueType, RUSTC_WASM_C6DCCF3E_IMPORTS,
    WASI_BUILD_IMPORT_ABI_V1,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{self, Write as _};

pub const DESCRIPTOR_JSON: &str =
    include_str!("../../../docs/architecture/genesis-floor-contract.v1.json");
const GENERAL_ID: &str = "raios.genesis_general_imports.v1";
const DESCRIPTOR_SCHEMA: &str = "raios.genesis_floor_contract.v1";
const CANONICAL_BUILD_SHA256: &str =
    "4145184d6ae43a57e1f75e1cfc2b4a19c6dd27fb1a29dd7104e1df9817616e65";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Floor {
    General,
    Build,
}

impl Floor {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Build => "build",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ContractReport {
    pub schema: &'static str,
    pub floor: &'static str,
    pub expected_import_count: usize,
    pub observed_import_count: usize,
    pub checked_import_count: usize,
    pub verdict: &'static str,
    pub reason: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_index: Option<usize>,
}

impl ContractReport {
    pub fn accepted(&self) -> bool {
        self.verdict == "accepted"
    }

    pub fn to_json(&self) -> String {
        let mut json = serde_json::to_string(self).expect("contract report is serializable");
        json.push('\n');
        json
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractError(String);

impl fmt::Display for ContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ContractError {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Descriptor {
    schema: String,
    version: u32,
    allowed_wire_types: Vec<String>,
    allowed_import_kinds: Vec<String>,
    reserved_kernel_namespaces: Vec<String>,
    reserved_dependency_kind: String,
    general_floor: ContractFloor,
    build_floor: BuildFloor,
    provenance: Provenance,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContractFloor {
    id: String,
    imports: Vec<Declaration>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildFloor {
    id: String,
    canonical_sha256: String,
    imports: Vec<Declaration>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Declaration {
    module: String,
    name: String,
    kind: String,
    #[serde(default)]
    params: Vec<String>,
    #[serde(default)]
    results: Vec<String>,
    initial: Option<u64>,
    maximum: Option<u64>,
    shared: Option<bool>,
    memory64: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Provenance {
    code: Vec<String>,
    documentation: Vec<String>,
}

pub fn validate_embedded_descriptor() -> Result<(), ContractError> {
    validate_descriptor_json(DESCRIPTOR_JSON)
}

pub fn validate_descriptor_json(json: &str) -> Result<(), ContractError> {
    let descriptor: Descriptor = serde_json::from_str(json)
        .map_err(|error| ContractError(format!("descriptor JSON invalid: {error}")))?;
    validate_descriptor(&descriptor)
}

fn validate_descriptor(descriptor: &Descriptor) -> Result<(), ContractError> {
    require(descriptor.schema == DESCRIPTOR_SCHEMA, "schema mismatch")?;
    require(descriptor.version == 1, "version mismatch")?;
    require(
        descriptor.allowed_wire_types == ["i32", "i64"],
        "allowed wire types mismatch",
    )?;
    require(
        descriptor.allowed_import_kinds == ["func", "memory"],
        "allowed import kinds mismatch",
    )?;
    require(
        descriptor.reserved_kernel_namespaces == ["kernel", "raios_kernel", "kernel_internal"],
        "reserved kernel namespaces mismatch",
    )?;
    require(
        descriptor.reserved_dependency_kind == "kernel_internal_type_dependency",
        "reserved dependency kind mismatch",
    )?;
    require(
        descriptor.general_floor.id == GENERAL_ID,
        "general floor id mismatch",
    )?;
    require(
        descriptor.build_floor.id == WASI_BUILD_IMPORT_ABI_V1,
        "build floor id mismatch",
    )?;
    require(
        descriptor
            .provenance
            .code
            .iter()
            .any(|path| path.starts_with("crates/raios-core/src/scoped_wasm_import_grant.rs:"))
            && descriptor
                .provenance
                .code
                .iter()
                .any(|path| path.starts_with("crates/raios-core/src/wasi_preview1_import_abi.rs:"))
            && descriptor
                .provenance
                .documentation
                .iter()
                .any(|path| path.starts_with("docs/architecture/genesis-layer.md:")),
        "provenance mismatch",
    )?;

    let canonical_general = &KNOWN_HOST_IMPORTS[..5];
    require(
        descriptor.general_floor.imports.len() == canonical_general.len(),
        "general import count mismatch",
    )?;
    reject_duplicate_declarations(&descriptor.general_floor.imports, "general")?;
    for (index, (declaration, canonical_pair)) in descriptor
        .general_floor
        .imports
        .iter()
        .zip(canonical_general.iter())
        .enumerate()
    {
        require(
            (declaration.module.as_str(), declaration.name.as_str()) == *canonical_pair,
            &format!("general import order/name mismatch at {index}"),
        )?;
        let (params, results) = general_signature(&declaration.name)
            .ok_or_else(|| ContractError(format!("missing general signature at {index}")))?;
        require_function(declaration, params, results, "general", index)?;
    }

    require(
        descriptor.build_floor.imports.len() == RUSTC_WASM_C6DCCF3E_IMPORTS.len(),
        "build import count mismatch",
    )?;
    reject_duplicate_declarations(&descriptor.build_floor.imports, "build")?;
    for (index, (declaration, canonical)) in descriptor
        .build_floor
        .imports
        .iter()
        .zip(RUSTC_WASM_C6DCCF3E_IMPORTS.iter())
        .enumerate()
    {
        require(
            declaration.module == canonical.module && declaration.name == canonical.name,
            &format!("build import order/name mismatch at {index}"),
        )?;
        match canonical.kind {
            WasiImportKind::Func { params, results } => require_function(
                declaration,
                &canonical_types(params),
                &canonical_types(results),
                "build",
                index,
            )?,
            WasiImportKind::Memory {
                initial,
                maximum,
                shared,
            } => {
                require(
                    declaration.kind == "memory"
                        && declaration.params.is_empty()
                        && declaration.results.is_empty()
                        && declaration.initial == Some(initial)
                        && declaration.maximum == maximum
                        && declaration.shared == Some(shared)
                        && declaration.memory64 == Some(false),
                    &format!("build memory declaration mismatch at {index}"),
                )?;
            }
            _ => return Err(ContractError(format!("canonical non-wire kind at {index}"))),
        }
    }
    let computed = hex(&wasi_import_declarations_sha256(
        RUSTC_WASM_C6DCCF3E_IMPORTS,
    ));
    require(
        computed == CANONICAL_BUILD_SHA256,
        "raios-core canonical digest drift",
    )?;
    require(
        descriptor.build_floor.canonical_sha256 == computed,
        "descriptor build digest mismatch",
    )?;
    Ok(())
}

pub fn check_contract(inventory: &Inventory, floor: Floor) -> ContractReport {
    let descriptor: Descriptor = match serde_json::from_str(DESCRIPTOR_JSON) {
        Ok(value) => value,
        Err(_) => {
            return rejected(
                floor,
                0,
                inventory.import_count(),
                0,
                "contract_descriptor_mismatch",
                None,
            )
        }
    };
    if validate_descriptor(&descriptor).is_err() {
        return rejected(
            floor,
            0,
            inventory.import_count(),
            0,
            "contract_descriptor_mismatch",
            None,
        );
    }
    let expected = match floor {
        Floor::General => &descriptor.general_floor.imports,
        Floor::Build => &descriptor.build_floor.imports,
    };

    for (index, import) in inventory.imports().iter().enumerate() {
        if descriptor
            .reserved_kernel_namespaces
            .iter()
            .any(|namespace| namespace == &import.module)
            || !is_allowed_wire_import(import, expected)
        {
            return rejected(
                floor,
                expected.len(),
                inventory.import_count(),
                index,
                "kernel_internal_type_dependency",
                Some(index),
            );
        }
    }

    let mut seen = HashSet::new();
    for (index, import) in inventory.imports().iter().enumerate() {
        if !seen.insert((&import.module, &import.name)) {
            return rejected(
                floor,
                expected.len(),
                inventory.import_count(),
                index,
                "duplicate_floor_import",
                Some(index),
            );
        }
        if !expected
            .iter()
            .any(|item| item.module == import.module && item.name == import.name)
        {
            return rejected(
                floor,
                expected.len(),
                inventory.import_count(),
                index,
                "undeclared_floor_import",
                Some(index),
            );
        }
    }
    if inventory.import_count() != expected.len() {
        return rejected(
            floor,
            expected.len(),
            inventory.import_count(),
            inventory.import_count(),
            "floor_import_count_mismatch",
            None,
        );
    }
    for (index, (observed, declaration)) in inventory.imports().iter().zip(expected).enumerate() {
        if observed.module != declaration.module || observed.name != declaration.name {
            return rejected(
                floor,
                expected.len(),
                inventory.import_count(),
                index,
                "floor_import_order_mismatch",
                Some(index),
            );
        }
        if !matches_declaration(observed, declaration) {
            return rejected(
                floor,
                expected.len(),
                inventory.import_count(),
                index,
                "floor_import_signature_mismatch",
                Some(index),
            );
        }
    }
    ContractReport {
        schema: DESCRIPTOR_SCHEMA,
        floor: floor.as_str(),
        expected_import_count: expected.len(),
        observed_import_count: inventory.import_count(),
        checked_import_count: inventory.import_count(),
        verdict: "accepted",
        reason: "documented_floor_imports_only",
        import_index: None,
    }
}

fn rejected(
    floor: Floor,
    expected: usize,
    observed: usize,
    checked: usize,
    reason: &'static str,
    import_index: Option<usize>,
) -> ContractReport {
    ContractReport {
        schema: DESCRIPTOR_SCHEMA,
        floor: floor.as_str(),
        expected_import_count: expected,
        observed_import_count: observed,
        checked_import_count: checked,
        verdict: "rejected",
        reason,
        import_index,
    }
}

fn is_allowed_wire_import(import: &Import, expected: &[Declaration]) -> bool {
    match &import.kind {
        ImportKind::Func {
            params, results, ..
        } => params
            .iter()
            .chain(results)
            .all(|ty| matches!(ty, ValueType::I32 | ValueType::I64)),
        ImportKind::Memory { .. } => expected.iter().any(|declaration| {
            declaration.module == import.module
                && declaration.name == import.name
                && declaration.kind == "memory"
                && matches_declaration(import, declaration)
        }),
        ImportKind::Global { .. } | ImportKind::Table { .. } | ImportKind::Tag => false,
    }
}

fn matches_declaration(import: &Import, declaration: &Declaration) -> bool {
    match &import.kind {
        ImportKind::Func {
            params, results, ..
        } => {
            declaration.kind == "func"
                && declaration.params == inventory_types(params)
                && declaration.results == inventory_types(results)
                && declaration.initial.is_none()
                && declaration.maximum.is_none()
                && declaration.shared.is_none()
                && declaration.memory64.is_none()
        }
        ImportKind::Memory {
            initial,
            maximum,
            shared,
            memory64,
        } => {
            declaration.kind == "memory"
                && declaration.params.is_empty()
                && declaration.results.is_empty()
                && declaration.initial == Some(*initial)
                && declaration.maximum == *maximum
                && declaration.shared == Some(*shared)
                && declaration.memory64 == Some(*memory64)
        }
        _ => false,
    }
}

fn general_signature(name: &str) -> Option<(&'static [&'static str], &'static [&'static str])> {
    match name {
        "log" => Some((&["i32", "i32"], &[])),
        "counter_get" => Some((&[], &["i64"])),
        "input_len" => Some((&[], &["i32"])),
        "input_read" | "output_write" => Some((&["i32", "i32"], &["i32"])),
        _ => None,
    }
}

fn require_function(
    declaration: &Declaration,
    params: &[&str],
    results: &[&str],
    floor: &str,
    index: usize,
) -> Result<(), ContractError> {
    require(
        declaration.kind == "func"
            && declaration
                .params
                .iter()
                .map(String::as_str)
                .eq(params.iter().copied())
            && declaration
                .results
                .iter()
                .map(String::as_str)
                .eq(results.iter().copied())
            && declaration.initial.is_none()
            && declaration.maximum.is_none()
            && declaration.shared.is_none()
            && declaration.memory64.is_none(),
        &format!("{floor} function declaration mismatch at {index}"),
    )
}

fn reject_duplicate_declarations(
    imports: &[Declaration],
    floor: &str,
) -> Result<(), ContractError> {
    let mut seen = HashSet::new();
    for declaration in imports {
        if !seen.insert((&declaration.module, &declaration.name)) {
            return Err(ContractError(format!("duplicate {floor} declaration")));
        }
    }
    Ok(())
}

fn canonical_types(types: &[WasiValueType]) -> Vec<&'static str> {
    types
        .iter()
        .map(|ty| match ty {
            WasiValueType::I32 => "i32",
            WasiValueType::I64 => "i64",
            WasiValueType::F32 => "f32",
            WasiValueType::F64 => "f64",
            WasiValueType::V128 => "v128",
            WasiValueType::FuncRef => "funcref",
            WasiValueType::ExternRef => "externref",
        })
        .collect()
}

fn inventory_types(types: &[ValueType]) -> Vec<String> {
    types.iter().map(|ty| ty.as_str().to_owned()).collect()
}

fn require(condition: bool, message: &str) -> Result<(), ContractError> {
    if condition {
        Ok(())
    } else {
        Err(ContractError(message.to_owned()))
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}
