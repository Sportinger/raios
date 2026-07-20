use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt::{self, Write as _};
use wasmparser_nostd::{Encoding, FuncType, Parser, Payload, Type, TypeRef, ValType};

pub mod contract;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Inventory {
    file_length: u64,
    file_sha256: String,
    imports_sha256: String,
    imports: Vec<Import>,
}

impl Inventory {
    pub fn file_length(&self) -> u64 {
        self.file_length
    }

    pub fn file_sha256(&self) -> &str {
        &self.file_sha256
    }

    pub fn import_count(&self) -> usize {
        self.imports.len()
    }

    pub fn imports_sha256(&self) -> &str {
        &self.imports_sha256
    }

    pub fn imports(&self) -> &[Import] {
        &self.imports
    }

    pub fn to_canonical_json(&self) -> Vec<u8> {
        let imports = canonical_imports_json(&self.imports);
        let mut json = String::new();
        json.push('{');
        write_json_key(&mut json, "file_length");
        write!(&mut json, "{},", self.file_length).expect("writing to String cannot fail");
        write_json_key(&mut json, "file_sha256");
        write_json_string(&mut json, &self.file_sha256);
        json.push(',');
        write_json_key(&mut json, "import_count");
        write!(&mut json, "{},", self.imports.len()).expect("writing to String cannot fail");
        write_json_key(&mut json, "imports_sha256");
        write_json_string(&mut json, &self.imports_sha256);
        json.push(',');
        write_json_key(&mut json, "imports");
        json.push_str(&imports);
        json.push_str("}\n");
        json.into_bytes()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Import {
    pub module: String,
    pub name: String,
    pub kind: ImportKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportKind {
    Func {
        type_index: u32,
        params: Vec<ValueType>,
        results: Vec<ValueType>,
    },
    Memory {
        initial: u64,
        maximum: Option<u64>,
        shared: bool,
        memory64: bool,
    },
    Global {
        value_type: ValueType,
        mutable: bool,
    },
    Table {
        element_type: ValueType,
        initial: u32,
        maximum: Option<u32>,
    },
    Tag,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
    V128,
    FuncRef,
    ExternRef,
}

impl ValueType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::V128 => "v128",
            Self::FuncRef => "funcref",
            Self::ExternRef => "externref",
        }
    }
}

impl From<ValType> for ValueType {
    fn from(value: ValType) -> Self {
        match value {
            ValType::I32 => Self::I32,
            ValType::I64 => Self::I64,
            ValType::F32 => Self::F32,
            ValType::F64 => Self::F64,
            ValType::V128 => Self::V128,
            ValType::FuncRef => Self::FuncRef,
            ValType::ExternRef => Self::ExternRef,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryError {
    message: String,
}

impl InventoryError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for InventoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for InventoryError {}

enum RawImportKind {
    Func(u32),
    Memory {
        initial: u64,
        maximum: Option<u64>,
        shared: bool,
        memory64: bool,
    },
    Global {
        value_type: ValueType,
        mutable: bool,
    },
    Table {
        element_type: ValueType,
        initial: u32,
        maximum: Option<u32>,
    },
    Tag,
}

struct RawImport {
    module: String,
    name: String,
    kind: RawImportKind,
}

pub fn inventory_from_bytes(bytes: &[u8]) -> Result<Inventory, InventoryError> {
    let mut types = Vec::<FuncType>::new();
    let mut raw_imports = Vec::<RawImport>::new();
    let mut saw_module_header = false;

    for payload in Parser::new(0).parse_all(bytes) {
        match payload
            .map_err(|error| InventoryError::new(format!("WebAssembly parse error: {error}")))?
        {
            Payload::Version { encoding, .. } => {
                if encoding != Encoding::Module {
                    return Err(InventoryError::new(
                        "WebAssembly components are unsupported; expected a core module",
                    ));
                }
                saw_module_header = true;
            }
            Payload::TypeSection(section) => {
                for entry in section {
                    let entry = entry.map_err(|error| {
                        InventoryError::new(format!("type section parse error: {error}"))
                    })?;
                    match entry {
                        Type::Func(function_type) => types.push(function_type),
                    }
                }
            }
            Payload::ImportSection(section) => {
                for entry in section {
                    let entry = entry.map_err(|error| {
                        InventoryError::new(format!("import section parse error: {error}"))
                    })?;
                    let kind = match entry.ty {
                        TypeRef::Func(type_index) => RawImportKind::Func(type_index),
                        TypeRef::Memory(memory) => RawImportKind::Memory {
                            initial: memory.initial,
                            maximum: memory.maximum,
                            shared: memory.shared,
                            memory64: memory.memory64,
                        },
                        TypeRef::Global(global) => RawImportKind::Global {
                            value_type: global.content_type.into(),
                            mutable: global.mutable,
                        },
                        TypeRef::Table(table) => RawImportKind::Table {
                            element_type: table.element_type.into(),
                            initial: table.initial,
                            maximum: table.maximum,
                        },
                        TypeRef::Tag(_) => RawImportKind::Tag,
                    };
                    raw_imports.push(RawImport {
                        module: entry.module.to_owned(),
                        name: entry.name.to_owned(),
                        kind,
                    });
                }
            }
            _ => {}
        }
    }

    if !saw_module_header {
        return Err(InventoryError::new("missing WebAssembly module header"));
    }

    let mut imports = Vec::with_capacity(raw_imports.len());
    for (import_index, raw) in raw_imports.into_iter().enumerate() {
        let kind = match raw.kind {
            RawImportKind::Func(type_index) => {
                let function_type = types.get(type_index as usize).ok_or_else(|| {
                    InventoryError::new(format!(
                        "unresolvable function type index {type_index} for import {import_index} ({}::{})",
                        raw.module, raw.name
                    ))
                })?;
                ImportKind::Func {
                    type_index,
                    params: function_type
                        .params()
                        .iter()
                        .copied()
                        .map(ValueType::from)
                        .collect(),
                    results: function_type
                        .results()
                        .iter()
                        .copied()
                        .map(ValueType::from)
                        .collect(),
                }
            }
            RawImportKind::Memory {
                initial,
                maximum,
                shared,
                memory64,
            } => ImportKind::Memory {
                initial,
                maximum,
                shared,
                memory64,
            },
            RawImportKind::Global {
                value_type,
                mutable,
            } => ImportKind::Global {
                value_type,
                mutable,
            },
            RawImportKind::Table {
                element_type,
                initial,
                maximum,
            } => ImportKind::Table {
                element_type,
                initial,
                maximum,
            },
            RawImportKind::Tag => ImportKind::Tag,
        };
        imports.push(Import {
            module: raw.module,
            name: raw.name,
            kind,
        });
    }

    let imports_json = canonical_imports_json(&imports);
    Ok(Inventory {
        file_length: bytes.len() as u64,
        file_sha256: sha256_hex(bytes),
        imports_sha256: sha256_hex(imports_json.as_bytes()),
        imports,
    })
}

pub fn canonical_inventory_json(bytes: &[u8]) -> Result<Vec<u8>, InventoryError> {
    Ok(inventory_from_bytes(bytes)?.to_canonical_json())
}

fn canonical_imports_json(imports: &[Import]) -> String {
    let mut json = String::new();
    json.push('[');
    for (index, import) in imports.iter().enumerate() {
        if index != 0 {
            json.push(',');
        }
        json.push('{');
        write_json_key(&mut json, "module");
        write_json_string(&mut json, &import.module);
        json.push(',');
        write_json_key(&mut json, "name");
        write_json_string(&mut json, &import.name);
        json.push(',');
        write_json_key(&mut json, "kind");
        match &import.kind {
            ImportKind::Func {
                type_index,
                params,
                results,
            } => {
                write_json_string(&mut json, "func");
                json.push(',');
                write_json_key(&mut json, "type_index");
                write!(&mut json, "{type_index},").expect("writing to String cannot fail");
                write_json_key(&mut json, "params");
                write_value_types(&mut json, params);
                json.push(',');
                write_json_key(&mut json, "results");
                write_value_types(&mut json, results);
            }
            ImportKind::Memory {
                initial,
                maximum,
                shared,
                memory64,
            } => {
                write_json_string(&mut json, "memory");
                json.push(',');
                write_json_key(&mut json, "initial");
                write!(&mut json, "{initial},").expect("writing to String cannot fail");
                write_json_key(&mut json, "maximum");
                write_optional_number(&mut json, *maximum);
                json.push(',');
                write_json_key(&mut json, "shared");
                write!(&mut json, "{shared},").expect("writing to String cannot fail");
                write_json_key(&mut json, "memory64");
                write!(&mut json, "{memory64}").expect("writing to String cannot fail");
            }
            ImportKind::Global {
                value_type,
                mutable,
            } => {
                write_json_string(&mut json, "global");
                json.push(',');
                write_json_key(&mut json, "value_type");
                write_json_string(&mut json, value_type.as_str());
                json.push(',');
                write_json_key(&mut json, "mutable");
                write!(&mut json, "{mutable}").expect("writing to String cannot fail");
            }
            ImportKind::Table {
                element_type,
                initial,
                maximum,
            } => {
                write_json_string(&mut json, "table");
                json.push(',');
                write_json_key(&mut json, "element_type");
                write_json_string(&mut json, element_type.as_str());
                json.push(',');
                write_json_key(&mut json, "initial");
                write!(&mut json, "{initial},").expect("writing to String cannot fail");
                write_json_key(&mut json, "maximum");
                write_optional_number(&mut json, *maximum);
            }
            ImportKind::Tag => write_json_string(&mut json, "tag"),
        }
        json.push('}');
    }
    json.push(']');
    json
}

fn write_value_types(output: &mut String, types: &[ValueType]) {
    output.push('[');
    for (index, value_type) in types.iter().enumerate() {
        if index != 0 {
            output.push(',');
        }
        write_json_string(output, value_type.as_str());
    }
    output.push(']');
}

fn write_optional_number<T: fmt::Display>(output: &mut String, number: Option<T>) {
    match number {
        Some(number) => write!(output, "{number}").expect("writing to String cannot fail"),
        None => output.push_str("null"),
    }
}

fn write_json_key(output: &mut String, key: &str) {
    write_json_string(output, key);
    output.push(':');
}

fn write_json_string(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character <= '\u{1f}' => {
                write!(output, "\\u{:04x}", character as u32)
                    .expect("writing to String cannot fail");
            }
            character => output.push(character),
        }
    }
    output.push('"');
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::write_json_string;

    #[test]
    fn json_string_escaping_covers_quotes_slashes_and_controls() {
        let mut output = String::new();
        write_json_string(&mut output, "a\"\\\u{08}\u{0c}\n\r\t\u{00}z");
        assert_eq!(output, "\"a\\\"\\\\\\b\\f\\n\\r\\t\\u0000z\"");
    }
}
