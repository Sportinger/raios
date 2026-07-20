//! Builds the one fixed DEV predecessor used by the ADR 0024 rollback predicate.
//!
//! This is deliberately not a generic RECLOG injector.  Every service id, surface,
//! provenance field, signature key, record order, and transaction kind is pinned here.
//! The Python disk builder only places the verified RECLOG and ARTSTOR bytes into GPT.

#![recursion_limit = "256"]

use anyhow::{anyhow, bail, Context, Result};
use p256::ecdsa::{signature::Signer, Signature, SigningKey};
use raios_core::{
    artifact_blob_frame::{parse_artifact_blob_frame, plan_artifact_blob_write},
    durable_record_frame::{
        parse_reclog_frame, plan_reclog_append, scan_reclog, RECLOG_FRAME_HEADER_LEN,
    },
    module_load_gate::{
        computed_module_grant_hash, computed_module_local_attestation_reference_hash_from_sequences,
    },
    project_install::{
        install_action_signature_payload_sha256, seal_granted_candidate_install_envelope,
        seal_install_action, GrantedCandidateInstallEnvelope, ProjectInstallAction,
        ProjectInstallActionKind, ProjectInstallAuthority,
        GRANTED_CANDIDATE_INSTALL_ENVELOPE_VERSION, PROJECT_INSTALL_TRUST_TIER,
    },
    promotion_attestation::{
        verify_promotion_authority_signature, PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256,
    },
    repromotion_reverify::{
        reverify_persisted_artifact, RepromotionReverifyInput, RepromotionTransactionFields,
    },
    sha256_bytes,
    wasm_import_grant_event::{
        GrantScope, GrantTargetEntry, GrantTargetSnapshot, HostImportId,
        GRANT_TARGET_SNAPSHOT_SCHEMA,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{env, fs, path::Path};

const SERVICE_ID: &str = "svc.dev.granted_candidate";
const TRUST_TIER: &str = PROJECT_INSTALL_TRUST_TIER;
const ARTIFACT_ID: &str = "wasm:external:granted_candidate";
const REQUESTED_CAPABILITY: &str = "cap.module.load_ephemeral.dev_tier.current_boot";
const LOAD_MODE: &str = "wasmi_interpreter_ram_only";
const IMPORT_SET_CANONICAL: &[u8] = b"env.log\n";
const RECORD_COUNT: usize = 4;
const RECLOG_BYTE_COUNT: usize = 4096 * 512;
const ARTSTOR_BYTE_COUNT: u64 = (256 * 1024 * 1024 - 8192 * 512) as u64;
const EXPECTED_WAT: &str = r#"(module
  (import "env" "log" (func $log (param i32 i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "rollback target log-only")
  (func (export "raios_service_main") (result i32)
    i32.const 0
    i32.const 24
    call $log
    i32.const 0))"#;

const DEV_PRIVATE_SCALAR: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Metadata {
    schema: String,
    fixture: String,
    artifact_sha256: String,
    artifact_byte_len: u64,
    import_inventory: Vec<String>,
    import_set_sha256: String,
    grant_target_entries: Vec<String>,
    grant_target_count: u64,
    grant_target_snapshot_sha256: String,
    artstor_blob_len: u64,
    artstor_blob_frame_sha256: String,
    install_authorization_frame_sha256: String,
    promote_frame_sha256: String,
    artifact_persist_frame_sha256: String,
    unpromote_frame_sha256: String,
    reclog_count: u64,
    reclog_tail_sha256: String,
    promotion_signature_verified: bool,
    install_signature_verified: bool,
    signed_v2_linkage_verified: bool,
    kernel_repromotion_semantics_verified: bool,
}

#[derive(Clone)]
struct Bundle {
    artifact: Vec<u8>,
    artstor: Vec<u8>,
    reclog: Vec<u8>,
    metadata: Metadata,
}

#[derive(Clone)]
struct Derived {
    artifact_sha256: [u8; 32],
    computed_grant_hash: [u8; 32],
    manifest_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
    manifest_reference_hash: [u8; 32],
    artifact_reference_hash: [u8; 32],
    vm_report_reference_hash: [u8; 32],
    attestation_reference_hash: [u8; 32],
    activation_approval_sha256: [u8; 32],
    w7_invocation_sha256: [u8; 32],
    w7_receipt_sha256: [u8; 32],
    install_envelope_sha256: [u8; 32],
    install_action_sha256: [u8; 32],
    install_action_message_sha256: [u8; 32],
    authority_evidence_sha256: [u8; 32],
    physical_approval_sha256: [u8; 32],
    install_signature_der: Vec<u8>,
    promotion_signature_der: Vec<u8>,
    snapshot: GrantTargetSnapshot,
}

#[derive(Debug)]
struct ParsedReclog {
    records: Vec<Value>,
    frame_sha256: Vec<[u8; 32]>,
    frame_len: Vec<u64>,
    frame_offset: Vec<usize>,
}

#[derive(Debug)]
struct VerifiedRecords {
    frame_sha256: [[u8; 32]; RECORD_COUNT],
    artifact_sha256: [u8; 32],
    manifest_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    computed_grant_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
    attestation_reference_hash: [u8; 32],
    promotion_signature_der: Vec<u8>,
    install_action_message_sha256: [u8; 32],
    install_signature_der: Vec<u8>,
    snapshot_count: u64,
    snapshot_sha256: [u8; 32],
    import_set_sha256: [u8; 32],
    artstor_blob_offset: u64,
    artstor_blob_len: u64,
    artstor_blob_frame_sha256: [u8; 32],
}

#[derive(Debug)]
struct PromotionEvidence {
    manifest_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    computed_grant_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
    attestation_reference_hash: [u8; 32],
    promotion_signature_der: Vec<u8>,
}

const MAX_WASM_BYTES: usize = 64 * 1024;
const MAX_WASM_SECTION_BYTES: usize = 32 * 1024;
const MAX_WASM_VECTOR_ITEMS: u32 = 16;
const MAX_WASM_NAME_BYTES: usize = 64;
const MAX_WASM_DATA_BYTES: usize = 64;
const MAX_WASM_MEMORY_PAGES: u32 = 65_536;
const EXPECTED_LOG_DATA: &[u8] = b"rollback target log-only";

#[derive(Clone, Debug, PartialEq, Eq)]
struct WasmFuncType {
    params: Vec<u8>,
    results: Vec<u8>,
}

#[derive(Debug)]
enum WasmImportDesc {
    Function(u32),
    Table,
    Memory,
    Global,
}

#[derive(Debug)]
struct WasmImport {
    module: String,
    name: String,
    desc: WasmImportDesc,
}

#[derive(Debug)]
struct WasmLimits {
    min: u32,
    max: Option<u32>,
}

#[derive(Debug)]
struct WasmExport {
    name: String,
    kind: u8,
    index: u32,
}

#[derive(Debug)]
struct VerifiedWasm {
    artifact_sha256: [u8; 32],
    byte_len: u64,
    import_inventory: Vec<String>,
    import_set_canonical: Vec<u8>,
    grant_target_entries: Vec<GrantTargetEntry>,
    grant_target_inventory: Vec<String>,
}

struct WasmReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> WasmReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn byte(&mut self) -> Result<u8> {
        let value = self
            .bytes
            .get(self.offset)
            .copied()
            .ok_or_else(|| anyhow!("wasm_binary_invalid"))?;
        self.offset += 1;
        Ok(value)
    }

    fn bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(len)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| anyhow!("wasm_binary_invalid"))?;
        let value = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(value)
    }

    fn u32_leb(&mut self) -> Result<u32> {
        let mut value = 0u32;
        for byte_index in 0..5 {
            let byte = self.byte()?;
            let payload = u32::from(byte & 0x7f);
            if byte_index == 4 && byte & 0xf0 != 0 {
                bail!("wasm_binary_invalid")
            }
            value |= payload << (byte_index * 7);
            if byte & 0x80 == 0 {
                if byte_index != 0 && payload == 0 {
                    bail!("wasm_binary_invalid")
                }
                return Ok(value);
            }
        }
        bail!("wasm_binary_invalid")
    }

    fn i32_leb(&mut self) -> Result<i32> {
        let mut value = 0i64;
        for byte_index in 0..5 {
            let byte = self.byte()?;
            let payload = i64::from(byte & 0x7f);
            if byte_index == 4
                && (byte & 0x80 != 0 || !((byte & 0x7f) <= 0x07 || (byte & 0x7f) >= 0x78))
            {
                bail!("wasm_binary_invalid")
            }
            let shift = byte_index * 7;
            value |= payload << shift;
            if byte & 0x80 == 0 {
                let used_bits = shift + 7;
                if byte & 0x40 != 0 {
                    value |= (!0i64) << used_bits;
                }
                let value = i32::try_from(value).map_err(|_| anyhow!("wasm_binary_invalid"))?;
                if signed_i32_leb_len(value) != byte_index + 1 {
                    bail!("wasm_binary_invalid")
                }
                return Ok(value);
            }
        }
        bail!("wasm_binary_invalid")
    }

    fn vector_len(&mut self) -> Result<u32> {
        let count = self.u32_leb()?;
        if count > MAX_WASM_VECTOR_ITEMS {
            bail!("wasm_binary_invalid")
        }
        Ok(count)
    }

    fn name(&mut self) -> Result<&'a str> {
        let len = usize::try_from(self.u32_leb()?).map_err(|_| anyhow!("wasm_binary_invalid"))?;
        if len > MAX_WASM_NAME_BYTES {
            bail!("wasm_binary_invalid")
        }
        std::str::from_utf8(self.bytes(len)?).map_err(|_| anyhow!("wasm_binary_invalid"))
    }

    fn subreader(&mut self, len: usize) -> Result<WasmReader<'a>> {
        Ok(WasmReader::new(self.bytes(len)?))
    }

    fn is_finished(&self) -> bool {
        self.offset == self.bytes.len()
    }

    fn finish(&self) -> Result<()> {
        if !self.is_finished() {
            bail!("wasm_binary_invalid")
        }
        Ok(())
    }

    fn skip_remaining(&mut self) {
        self.offset = self.bytes.len();
    }
}

fn signed_i32_leb_len(mut value: i32) -> usize {
    let mut len = 0;
    loop {
        let byte = (value as u8) & 0x7f;
        value >>= 7;
        len += 1;
        if (value == 0 && byte & 0x40 == 0) || (value == -1 && byte & 0x40 != 0) {
            return len;
        }
    }
}

fn read_value_type(reader: &mut WasmReader<'_>) -> Result<u8> {
    let value_type = reader.byte()?;
    if !matches!(value_type, 0x7f | 0x7e | 0x7d | 0x7c) {
        bail!("wasm_binary_invalid")
    }
    Ok(value_type)
}

fn read_limits(reader: &mut WasmReader<'_>, memory: bool) -> Result<WasmLimits> {
    let flags = reader.u32_leb()?;
    if flags > 1 {
        bail!("wasm_binary_invalid")
    }
    let min = reader.u32_leb()?;
    let max = if flags == 1 {
        Some(reader.u32_leb()?)
    } else {
        None
    };
    if max.is_some_and(|max| max < min)
        || (memory
            && (min > MAX_WASM_MEMORY_PAGES || max.is_some_and(|max| max > MAX_WASM_MEMORY_PAGES)))
    {
        bail!("wasm_binary_invalid")
    }
    Ok(WasmLimits { min, max })
}

fn read_type_section(reader: &mut WasmReader<'_>) -> Result<Vec<WasmFuncType>> {
    let count = reader.vector_len()?;
    let mut types = Vec::with_capacity(count as usize);
    for _ in 0..count {
        if reader.byte()? != 0x60 {
            bail!("wasm_binary_invalid")
        }
        let param_count = reader.vector_len()?;
        let mut params = Vec::with_capacity(param_count as usize);
        for _ in 0..param_count {
            params.push(read_value_type(reader)?);
        }
        let result_count = reader.vector_len()?;
        let mut results = Vec::with_capacity(result_count as usize);
        for _ in 0..result_count {
            results.push(read_value_type(reader)?);
        }
        types.push(WasmFuncType { params, results });
    }
    Ok(types)
}

fn read_import_section(reader: &mut WasmReader<'_>) -> Result<Vec<WasmImport>> {
    let count = reader.vector_len()?;
    let mut imports = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let module = reader.name()?.to_owned();
        let name = reader.name()?.to_owned();
        let desc = match reader.byte()? {
            0x00 => WasmImportDesc::Function(reader.u32_leb()?),
            0x01 => {
                if reader.byte()? != 0x70 {
                    bail!("wasm_binary_invalid")
                }
                let limits = read_limits(reader, false)?;
                let _ = (limits.min, limits.max);
                WasmImportDesc::Table
            }
            0x02 => {
                let limits = read_limits(reader, true)?;
                let _ = (limits.min, limits.max);
                WasmImportDesc::Memory
            }
            0x03 => {
                read_value_type(reader)?;
                if reader.byte()? > 1 {
                    bail!("wasm_binary_invalid")
                }
                WasmImportDesc::Global
            }
            _ => bail!("wasm_binary_invalid"),
        };
        imports.push(WasmImport { module, name, desc });
    }
    Ok(imports)
}

fn read_function_section(reader: &mut WasmReader<'_>) -> Result<Vec<u32>> {
    let count = reader.vector_len()?;
    (0..count).map(|_| reader.u32_leb()).collect()
}

fn read_memory_section(reader: &mut WasmReader<'_>) -> Result<Vec<WasmLimits>> {
    let count = reader.vector_len()?;
    (0..count).map(|_| read_limits(reader, true)).collect()
}

fn read_export_section(reader: &mut WasmReader<'_>) -> Result<Vec<WasmExport>> {
    let count = reader.vector_len()?;
    let mut exports = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let name = reader.name()?.to_owned();
        let kind = reader.byte()?;
        if kind > 3 {
            bail!("wasm_binary_invalid")
        }
        exports.push(WasmExport {
            name,
            kind,
            index: reader.u32_leb()?,
        });
    }
    Ok(exports)
}

fn read_code_section(reader: &mut WasmReader<'_>) -> Result<u32> {
    let count = reader.vector_len()?;
    for _ in 0..count {
        let body_len =
            usize::try_from(reader.u32_leb()?).map_err(|_| anyhow!("wasm_binary_invalid"))?;
        if body_len > MAX_WASM_SECTION_BYTES {
            bail!("wasm_binary_invalid")
        }
        let mut body = reader.subreader(body_len)?;
        let local_group_count = body.vector_len()?;
        let mut local_count = 0u32;
        for _ in 0..local_group_count {
            local_count = local_count
                .checked_add(body.u32_leb()?)
                .filter(|count| *count <= MAX_WASM_VECTOR_ITEMS)
                .ok_or_else(|| anyhow!("wasm_binary_invalid"))?;
            read_value_type(&mut body)?;
        }
        if local_count != 0
            || body.byte()? != 0x41
            || body.i32_leb()? != 0
            || body.byte()? != 0x41
            || body.i32_leb()? != EXPECTED_LOG_DATA.len() as i32
            || body.byte()? != 0x10
            || body.u32_leb()? != 0
            || body.byte()? != 0x41
            || body.i32_leb()? != 0
            || body.byte()? != 0x0b
        {
            bail!("wasm_code_contract_mismatch")
        }
        body.finish()?;
    }
    Ok(count)
}

fn read_data_section(reader: &mut WasmReader<'_>) -> Result<u32> {
    let count = reader.vector_len()?;
    for _ in 0..count {
        if reader.u32_leb()? != 0
            || reader.byte()? != 0x41
            || reader.i32_leb()? != 0
            || reader.byte()? != 0x0b
        {
            bail!("wasm_data_contract_mismatch")
        }
        let len = usize::try_from(reader.u32_leb()?).map_err(|_| anyhow!("wasm_binary_invalid"))?;
        if len > MAX_WASM_DATA_BYTES {
            bail!("wasm_binary_invalid")
        }
        if reader.bytes(len)? != EXPECTED_LOG_DATA {
            bail!("wasm_data_contract_mismatch")
        }
    }
    Ok(count)
}

fn verify_log_only_wasm(artifact: &[u8]) -> Result<VerifiedWasm> {
    if artifact.len() > MAX_WASM_BYTES {
        bail!("wasm_binary_invalid")
    }
    let mut module = WasmReader::new(artifact);
    if module.bytes(4)? != b"\0asm" || module.bytes(4)? != [1, 0, 0, 0] {
        bail!("wasm_header_invalid")
    }

    let mut seen_sections = 0u16;
    let mut last_section = 0u8;
    let mut types = None;
    let mut imports = None;
    let mut functions = None;
    let mut memories = None;
    let mut exports = None;
    let mut code_count = None;
    let mut data_count = None;
    while !module.is_finished() {
        let section_id = module.byte()?;
        let section_len =
            usize::try_from(module.u32_leb()?).map_err(|_| anyhow!("wasm_binary_invalid"))?;
        if section_len > MAX_WASM_SECTION_BYTES {
            bail!("wasm_binary_invalid")
        }
        let mut section = module.subreader(section_len)?;
        if section_id == 0 {
            section.name()?;
            section.skip_remaining();
            continue;
        }
        let section_bit = 1u16
            .checked_shl(u32::from(section_id))
            .ok_or_else(|| anyhow!("wasm_section_contract_mismatch"))?;
        if section_id <= last_section || seen_sections & section_bit != 0 {
            bail!("wasm_section_order_invalid")
        }
        last_section = section_id;
        seen_sections |= section_bit;
        match section_id {
            1 => types = Some(read_type_section(&mut section)?),
            2 => imports = Some(read_import_section(&mut section)?),
            3 => functions = Some(read_function_section(&mut section)?),
            5 => memories = Some(read_memory_section(&mut section)?),
            7 => exports = Some(read_export_section(&mut section)?),
            10 => code_count = Some(read_code_section(&mut section)?),
            11 => data_count = Some(read_data_section(&mut section)?),
            _ => bail!("wasm_section_contract_mismatch"),
        }
        section.finish()?;
    }

    const REQUIRED_SECTIONS: u16 =
        (1 << 1) | (1 << 2) | (1 << 3) | (1 << 5) | (1 << 7) | (1 << 10) | (1 << 11);
    if seen_sections != REQUIRED_SECTIONS {
        bail!("wasm_section_contract_mismatch")
    }
    let types = types.ok_or_else(|| anyhow!("wasm_section_contract_mismatch"))?;
    let imports = imports.ok_or_else(|| anyhow!("wasm_section_contract_mismatch"))?;
    let functions = functions.ok_or_else(|| anyhow!("wasm_section_contract_mismatch"))?;
    let memories = memories.ok_or_else(|| anyhow!("wasm_section_contract_mismatch"))?;
    let exports = exports.ok_or_else(|| anyhow!("wasm_section_contract_mismatch"))?;

    let log_type = WasmFuncType {
        params: vec![0x7f, 0x7f],
        results: Vec::new(),
    };
    let main_type = WasmFuncType {
        params: Vec::new(),
        results: vec![0x7f],
    };
    if types.len() != 2 {
        bail!("wasm_type_contract_mismatch")
    }
    let [import] = imports.as_slice() else {
        bail!("wasm_import_contract_mismatch")
    };
    let import_type_index = match import.desc {
        WasmImportDesc::Function(index) => index,
        _ => bail!("wasm_import_contract_mismatch"),
    };
    if import.module != "env"
        || import.name != "log"
        || types.get(import_type_index as usize) != Some(&log_type)
    {
        bail!("wasm_import_contract_mismatch")
    }
    let [main_type_index] = functions.as_slice() else {
        bail!("wasm_function_contract_mismatch")
    };
    if types.get(*main_type_index as usize) != Some(&main_type)
        || !types.contains(&log_type)
        || !types.contains(&main_type)
        || code_count != Some(1)
    {
        bail!("wasm_function_contract_mismatch")
    }
    let [memory] = memories.as_slice() else {
        bail!("wasm_memory_contract_mismatch")
    };
    if memory.min != 1 || memory.max.is_some() {
        bail!("wasm_memory_contract_mismatch")
    }
    if exports.len() != 2
        || !exports
            .iter()
            .any(|export| export.name == "memory" && export.kind == 2 && export.index == 0)
        || !exports.iter().any(|export| {
            export.name == "raios_service_main" && export.kind == 0 && export.index == 1
        })
        || exports[0].name == exports[1].name
    {
        bail!("wasm_export_contract_mismatch")
    }
    if data_count != Some(1) {
        bail!("wasm_data_contract_mismatch")
    }

    let import_set_canonical = format!("{}.{}\n", import.module, import.name).into_bytes();
    Ok(VerifiedWasm {
        artifact_sha256: sha256_bytes(artifact),
        byte_len: artifact.len() as u64,
        import_inventory: vec![format!("{}.{}:(i32,i32)->()", import.module, import.name)],
        import_set_canonical,
        grant_target_entries: vec![GrantTargetEntry {
            host_import_id: HostImportId::EnvLog,
            scope: GrantScope::HostCall,
        }],
        grant_target_inventory: vec![format!("{}.{}:host_call", import.module, import.name)],
    })
}

fn main() {
    if let Err(error) = run() {
        eprintln!("dev-rollback-fixture: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [command, wat, wasm] if command == "write-artifact" => {
            write_fixed_artifact(Path::new(wat), Path::new(wasm))
        }
        [command, wasm, out] if command == "generate" => {
            let artifact = read_exact_artifact(Path::new(wasm))?;
            write_bundle(Path::new(out), &build_bundle(artifact)?)
        }
        [command, out] if command == "verify" => verify_directory(Path::new(out)),
        [command, wasm] if command == "self-test" => self_test(Path::new(wasm)),
        _ => bail!(
            "usage: dev-rollback-fixture write-artifact <fixture.wat> <fixture.wasm> | generate <fixture.wasm> <output-dir> | verify <output-dir> | self-test <fixture.wasm>"
        ),
    }
}

fn fixed_wasm_bytes() -> Vec<u8> {
    let mut out = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    // types: (i32,i32)->() and ()->i32
    out.extend_from_slice(&[
        0x01, 0x0a, 0x02, 0x60, 0x02, 0x7f, 0x7f, 0x00, 0x60, 0x00, 0x01, 0x7f,
    ]);
    // import env.log using type 0
    out.extend_from_slice(&[
        0x02, 0x0b, 0x01, 0x03, b'e', b'n', b'v', 0x03, b'l', b'o', b'g', 0x00, 0x00,
    ]);
    // one defined function, type 1; one memory
    out.extend_from_slice(&[0x03, 0x02, 0x01, 0x01, 0x05, 0x03, 0x01, 0x00, 0x01]);
    // exports: memory and raios_service_main (defined function index 1)
    out.extend_from_slice(&[0x07, 0x1f, 0x02, 0x06]);
    out.extend_from_slice(b"memory");
    out.extend_from_slice(&[0x02, 0x00, 0x12]);
    out.extend_from_slice(b"raios_service_main");
    out.extend_from_slice(&[0x00, 0x01]);
    // body: log(0,24); return 0
    out.extend_from_slice(&[
        0x0a, 0x0c, 0x01, 0x0a, 0x00, 0x41, 0x00, 0x41, 0x18, 0x10, 0x00, 0x41, 0x00, 0x0b,
    ]);
    // active data segment at memory offset zero
    out.extend_from_slice(&[0x0b, 0x1e, 0x01, 0x00, 0x41, 0x00, 0x0b, 0x18]);
    out.extend_from_slice(b"rollback target log-only");
    out
}

fn normalize_text(value: &str) -> String {
    value.replace("\r\n", "\n").trim_end().to_owned()
}

fn write_fixed_artifact(wat: &Path, wasm: &Path) -> Result<()> {
    let source = fs::read_to_string(wat).with_context(|| format!("read {}", wat.display()))?;
    if normalize_text(&source) != EXPECTED_WAT {
        bail!("WAT is not the pinned rollback-target-log-only fixture")
    }
    fs::write(wasm, fixed_wasm_bytes()).with_context(|| format!("write {}", wasm.display()))
}

fn read_exact_artifact(path: &Path) -> Result<Vec<u8>> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    if bytes != fixed_wasm_bytes() {
        bail!("artifact bytes are not the pinned log-only module")
    }
    Ok(bytes)
}

fn label_hash(label: &str) -> [u8; 32] {
    let mut bytes = b"raios.dev_rollback_fixture.v1\0".to_vec();
    bytes.extend_from_slice(label.as_bytes());
    sha256_bytes(&bytes)
}

fn sign(hash: &[u8; 32]) -> Vec<u8> {
    let key = SigningKey::from_slice(&DEV_PRIVATE_SCALAR).expect("scalar one is valid");
    let signature: Signature = key.sign(hash);
    signature.to_der().as_bytes().to_vec()
}

fn event_id(sequence: u64) -> String {
    format!("event.current_boot.{sequence:08}")
}

fn sha(value: [u8; 32]) -> String {
    format!("sha256:{}", hex::encode(value))
}

fn derive(artifact: &[u8]) -> Result<Derived> {
    let artifact_sha256 = sha256_bytes(artifact);
    let manifest_hash = label_hash("manifest-log-only");
    let vm_report_hash = label_hash("vm-report-log-only");
    let local_attestation_hash = label_hash("local-attestation-log-only");
    let computed_grant_hash = computed_module_grant_hash(
        manifest_hash,
        artifact_sha256,
        vm_report_hash,
        local_attestation_hash,
    );
    let manifest_reference_hash = label_hash("manifest-reference-log-only");
    let artifact_reference_hash = label_hash("artifact-reference-log-only");
    let vm_report_reference_hash = label_hash("vm-report-reference-log-only");
    let attestation_reference_hash =
        computed_module_local_attestation_reference_hash_from_sequences(
            101,
            102,
            103,
            104,
            manifest_reference_hash,
            artifact_reference_hash,
            vm_report_reference_hash,
            manifest_hash,
            artifact_sha256,
            computed_grant_hash,
            vm_report_hash,
            local_attestation_hash,
        );
    let activation_approval_sha256 = label_hash("activation-approval");
    let w7_invocation_sha256 = label_hash("w7-invocation");
    let w7_receipt_sha256 = label_hash("w7-receipt");
    let snapshot = GrantTargetSnapshot::seal(
        SERVICE_ID,
        artifact_sha256,
        vec![GrantTargetEntry {
            host_import_id: HostImportId::EnvLog,
            scope: GrantScope::HostCall,
        }],
    )
    .map_err(|e| anyhow!("snapshot: {e:?}"))?;
    let envelope = seal_granted_candidate_install_envelope(GrantedCandidateInstallEnvelope {
        envelope_version: GRANTED_CANDIDATE_INSTALL_ENVELOPE_VERSION,
        service_id: SERVICE_ID.to_owned(),
        candidate_sha256: artifact_sha256,
        candidate_byte_len: artifact.len() as u64,
        activation_approval_sha256,
        computed_grant_sha256: computed_grant_hash,
        attestation_reference_sha256: attestation_reference_hash,
        grant_target_schema: GRANT_TARGET_SNAPSHOT_SCHEMA.to_owned(),
        grant_target_count: snapshot.entry_count,
        grant_target_snapshot_sha256: snapshot.snapshot_sha256,
        w7_invocation_sha256,
        w7_receipt_sha256,
        receiver_content_sha256: artifact_sha256,
        receiver_candidate_sha256: artifact_sha256,
        catalog_candidate_sha256: artifact_sha256,
        generation: 1,
        auto_start: true,
        trust_tier: TRUST_TIER.to_owned(),
        envelope_sha256: [0; 32],
    })
    .map_err(|e| anyhow!(e.reason()))?;
    let authority_evidence_sha256 = label_hash("physical-owner-authority-evidence");
    let physical_approval_sha256 = label_hash("physical-install-approval");
    let unsigned = ProjectInstallAction {
        kind: ProjectInstallActionKind::Install,
        authority: ProjectInstallAuthority::PhysicalOwner,
        service_id: SERVICE_ID.to_owned(),
        generation: 1,
        log_sequence: 1,
        previous_commit_sha256: None,
        install_envelope_sha256: Some(envelope.envelope_sha256),
        target_install_commit_sha256: None,
        authority_evidence_sha256,
        physical_approval_sha256: Some(physical_approval_sha256),
        authority_key_sha256: Some(PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256),
        authority_signature: Vec::new(),
        action_sha256: [0; 32],
    };
    let install_action_message_sha256 =
        install_action_signature_payload_sha256(&unsigned).map_err(|e| anyhow!(e.reason()))?;
    let install_signature_der = sign(&install_action_message_sha256);
    let mut signed = unsigned;
    signed.authority_signature = install_signature_der.clone();
    let signed = seal_install_action(signed).map_err(|e| anyhow!(e.reason()))?;
    Ok(Derived {
        artifact_sha256,
        computed_grant_hash,
        manifest_hash,
        vm_report_hash,
        local_attestation_hash,
        manifest_reference_hash,
        artifact_reference_hash,
        vm_report_reference_hash,
        attestation_reference_hash,
        activation_approval_sha256,
        w7_invocation_sha256,
        w7_receipt_sha256,
        install_envelope_sha256: envelope.envelope_sha256,
        install_action_sha256: signed.action_sha256,
        install_action_message_sha256,
        authority_evidence_sha256,
        physical_approval_sha256,
        install_signature_der,
        promotion_signature_der: sign(&attestation_reference_hash),
        snapshot,
    })
}

fn canonical_payload(value: Value) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(&value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn json_text<'a>(record: &'a Value, field: &str) -> Result<&'a str> {
    record
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("record_field_invalid:{field}"))
}

fn json_u64(record: &Value, field: &str) -> Result<u64> {
    record
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| anyhow!("record_field_invalid:{field}"))
}

fn json_bool(record: &Value, field: &str) -> Result<bool> {
    record
        .get(field)
        .and_then(Value::as_bool)
        .ok_or_else(|| anyhow!("record_field_invalid:{field}"))
}

fn json_sha(record: &Value, field: &str) -> Result<[u8; 32]> {
    let value = json_text(record, field)?
        .strip_prefix("sha256:")
        .ok_or_else(|| anyhow!("record_sha_invalid:{field}"))?;
    let bytes = hex::decode(value).map_err(|_| anyhow!("record_sha_invalid:{field}"))?;
    bytes
        .try_into()
        .map_err(|_| anyhow!("record_sha_invalid:{field}"))
}

fn json_is_null(record: &Value, field: &str) -> bool {
    matches!(record.get(field), Some(Value::Null))
}

fn parse_reclog_records(reclog: &[u8]) -> Result<ParsedReclog> {
    let mut padded = vec![0u8; RECLOG_BYTE_COUNT];
    if reclog.len() > padded.len() {
        bail!("reclog_oversize")
    }
    padded[..reclog.len()].copy_from_slice(reclog);
    let scan = scan_reclog(&padded);
    if !scan.full_region_valid || scan.count != RECORD_COUNT as u64 {
        bail!("reclog_chain_invalid")
    }
    let mut records = Vec::new();
    let mut frame_sha256 = Vec::new();
    let mut frame_len = Vec::new();
    let mut frame_offset = Vec::new();
    let mut offset = 0usize;
    let mut prev = [0u8; 32];
    for seq in 1..=RECORD_COUNT as u64 {
        let frame = parse_reclog_frame(&padded, offset, seq, prev)
            .map_err(|_| anyhow!("reclog_chain_invalid"))?;
        let payload_start = offset + RECLOG_FRAME_HEADER_LEN;
        let payload_end = payload_start + frame.payload_len as usize;
        let payload = &padded[payload_start..payload_end];
        let record: Value =
            serde_json::from_slice(payload).map_err(|_| anyhow!("reclog_payload_invalid"))?;
        let canonical =
            canonical_payload(record.clone()).map_err(|_| anyhow!("reclog_payload_invalid"))?;
        if canonical != payload {
            bail!("reclog_payload_not_canonical")
        }
        records.push(record);
        frame_sha256.push(frame.frame_sha256);
        frame_len.push(frame.frame_len.into());
        frame_offset.push(offset);
        offset += frame.frame_len as usize;
        prev = frame.frame_sha256;
    }

    if offset != reclog.len() || scan.first_invalid_offset as usize != reclog.len() {
        bail!("reclog_range_invalid")
    }

    Ok(ParsedReclog {
        records,
        frame_sha256,
        frame_len,
        frame_offset,
    })
}

fn record_identity_is(
    record: &Value,
    schema: &str,
    record_kind: &str,
    transaction_kind: Option<&str>,
) -> bool {
    json_text(record, "schema").ok() == Some(schema)
        && json_text(record, "record_kind").ok() == Some(record_kind)
        && match transaction_kind {
            Some(kind) => json_text(record, "transaction_kind").ok() == Some(kind),
            None => record.get("transaction_kind").is_none(),
        }
}

fn verify_promotion_record(
    record: &Value,
    transaction_kind: &str,
    artifact_sha256: [u8; 32],
    authorization_frame_sha256: [u8; 32],
    authorization_computed_grant_sha256: [u8; 32],
    snapshot_count: u64,
    snapshot_sha256: [u8; 32],
) -> Result<PromotionEvidence> {
    let unpromote = transaction_kind == "unpromote";
    if json_text(record, "id")?
        != format!(
            "promotion_transaction.origin_boot.{transaction_kind}.svc.dev.granted_candidate.v0"
        )
        || json_text(record, "scope")? != "origin_boot"
        || json_text(record, "classification")? != "local_only"
        || json_text(record, "service_id")? != SERVICE_ID
        || json_text(record, "artifact_id")? != ARTIFACT_ID
        || json_text(record, "requested_capability")? != REQUESTED_CAPABILITY
        || json_text(record, "load_mode")? != LOAD_MODE
        || json_text(record, "trust_tier")? != TRUST_TIER
        || json_u64(record, "generation")? != 1
        || json_text(record, "load_event_id")? != event_id(105)
    {
        bail!("promotion_contract_mismatch")
    }

    let manifest_hash = json_sha(record, "manifest_hash")?;
    let record_artifact_sha256 = json_sha(record, "artifact_hash")?;
    let vm_report_hash = json_sha(record, "vm_report_hash")?;
    let local_attestation_hash = json_sha(record, "local_attestation_hash")?;
    let computed_grant_hash = json_sha(record, "computed_grant_hash")?;
    if record_artifact_sha256 != artifact_sha256 {
        bail!("candidate_artifact_binding_mismatch")
    }
    if manifest_hash != label_hash("manifest-log-only")
        || vm_report_hash != label_hash("vm-report-log-only")
        || local_attestation_hash != label_hash("local-attestation-log-only")
        || computed_grant_hash
            != computed_module_grant_hash(
                manifest_hash,
                artifact_sha256,
                vm_report_hash,
                local_attestation_hash,
            )
        || computed_grant_hash != authorization_computed_grant_sha256
    {
        bail!("promotion_grant_formula_mismatch")
    }

    let manifest_reference_hash = json_sha(record, "manifest_reference_hash")?;
    let artifact_reference_hash = json_sha(record, "artifact_reference_hash")?;
    let vm_report_reference_hash = json_sha(record, "vm_report_reference_hash")?;
    if json_text(record, "retained_manifest_reference_event_id")? != event_id(101)
        || json_text(record, "retained_artifact_reference_event_id")? != event_id(102)
        || json_text(record, "retained_vm_report_reference_event_id")? != event_id(103)
        || json_text(record, "retained_computed_grant_reference_event_id")? != event_id(104)
        || manifest_reference_hash != label_hash("manifest-reference-log-only")
        || artifact_reference_hash != label_hash("artifact-reference-log-only")
        || vm_report_reference_hash != label_hash("vm-report-reference-log-only")
    {
        bail!("promotion_reference_contract_mismatch")
    }
    let attestation_reference_hash =
        computed_module_local_attestation_reference_hash_from_sequences(
            101,
            102,
            103,
            104,
            manifest_reference_hash,
            artifact_reference_hash,
            vm_report_reference_hash,
            manifest_hash,
            artifact_sha256,
            computed_grant_hash,
            vm_report_hash,
            local_attestation_hash,
        );
    if json_sha(record, "attestation_reference_hash")? != attestation_reference_hash
        || json_sha(record, "install_authorization_frame_sha256")? != authorization_frame_sha256
        || json_text(record, "grant_target_schema")? != GRANT_TARGET_SNAPSHOT_SCHEMA
        || json_u64(record, "grant_target_count")? != snapshot_count
        || json_sha(record, "grant_target_snapshot_sha256")? != snapshot_sha256
        || json_u64(record, "install_envelope_version")?
            != GRANTED_CANDIDATE_INSTALL_ENVELOPE_VERSION as u64
    {
        bail!("promotion_authorization_link_mismatch")
    }

    let true_fields = [
        "signature_verified",
        "grant_binds_capability",
        "install_authorization_present",
        "install_envelope_binds_activation",
        "install_action_signature_verified",
        "physical_install_approval_consumed",
        "promotion_authority_is_placeholder",
    ];
    if true_fields
        .iter()
        .any(|field| json_bool(record, field).ok() != Some(true))
        || json_bool(record, "owner_sealed")? != false
        || json_bool(record, "cross_reboot_proven")? != false
        || json_bool(record, "persistence_claimed")? != false
        || json_sha(record, "promotion_authority_key_sha256")?
            != PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256
        || json_sha(record, "rollback_plan_hash")? != label_hash("rollback-plan-log-only")
        || json_sha(record, "pre_load_inventory_hash")? != label_hash("pre-load-inventory-log-only")
        || json_text(record, "ram_only_service_slot_id")?
            != "ram.slot.granted_candidate.current_boot"
    {
        bail!("promotion_contract_mismatch")
    }

    if json_bool(record, "restore_hash_verified")? != unpromote
        || json_bool(record, "stopped")? != unpromote
        || json_bool(record, "drop_clear_bytes")? != unpromote
        || json_bool(record, "free_slot")? != unpromote
        || json_bool(record, "remove_inventory")? != unpromote
        || if unpromote {
            json_text(record, "rollback_apply_event_id")? != event_id(106)
                || json_sha(record, "reprojected_inventory_hash")?
                    != label_hash("reprojected-inventory-log-only")
        } else {
            !json_is_null(record, "rollback_apply_event_id")
                || !json_is_null(record, "reprojected_inventory_hash")
        }
    {
        bail!("promotion_install_action_relationship_mismatch")
    }

    let promotion_signature_der = hex::decode(json_text(record, "promotion_signature_der")?)
        .map_err(|_| anyhow!("promotion_signature_invalid"))?;
    if promotion_signature_der.len() as u64 != json_u64(record, "promotion_signature_len")?
        || !verify_promotion_authority_signature(
            &promotion_signature_der,
            &attestation_reference_hash,
        )
    {
        bail!("promotion_signature_invalid")
    }

    Ok(PromotionEvidence {
        manifest_hash,
        vm_report_hash,
        computed_grant_hash,
        local_attestation_hash,
        attestation_reference_hash,
        promotion_signature_der,
    })
}

fn verify_signed_v2_linkage(reclog: &[u8], wasm: &VerifiedWasm) -> Result<VerifiedRecords> {
    let parsed = parse_reclog_records(reclog)?;
    let authorization = &parsed.records[0];
    let promotion = &parsed.records[1];
    let artifact_persist = &parsed.records[2];
    let unpromote = &parsed.records[3];
    if !record_identity_is(
        authorization,
        "raios.install_authorization.v0",
        "install_authorization",
        None,
    ) || !record_identity_is(
        promotion,
        "raios.promotion_transaction.v0",
        "promotion_transaction",
        Some("promote"),
    ) || !record_identity_is(
        artifact_persist,
        "raios.artifact_persist.v0",
        "artifact_persist",
        None,
    ) || !record_identity_is(
        unpromote,
        "raios.promotion_transaction.v0",
        "promotion_transaction",
        Some("unpromote"),
    ) {
        bail!("reclog_record_order_mismatch")
    }

    let artifact_sha256 = wasm.artifact_sha256;
    if json_text(authorization, "id")?
        != "install_authorization.origin_boot.svc.dev.granted_candidate.v0"
        || json_text(authorization, "scope")? != "origin_boot"
        || json_text(authorization, "classification")? != "local_only"
        || json_text(authorization, "service_id")? != SERVICE_ID
        || json_u64(authorization, "install_envelope_version")?
            != GRANTED_CANDIDATE_INSTALL_ENVELOPE_VERSION as u64
        || json_text(authorization, "grant_target_schema")? != GRANT_TARGET_SNAPSHOT_SCHEMA
        || json_text(authorization, "trust_tier")? != TRUST_TIER
        || json_u64(authorization, "install_generation")? != 1
        || json_u64(authorization, "install_log_sequence")? != 1
    {
        bail!("authorization_contract_mismatch")
    }
    if json_sha(authorization, "install_candidate_sha256")? != artifact_sha256
        || json_sha(authorization, "receiver_content_sha256")? != artifact_sha256
        || json_sha(authorization, "receiver_candidate_sha256")? != artifact_sha256
        || json_sha(authorization, "catalog_candidate_sha256")? != artifact_sha256
        || json_u64(authorization, "install_candidate_byte_len")? != wasm.byte_len
    {
        bail!("candidate_artifact_binding_mismatch")
    }

    let snapshot = GrantTargetSnapshot::seal(
        SERVICE_ID,
        artifact_sha256,
        wasm.grant_target_entries.clone(),
    )
    .map_err(|_| anyhow!("grant_snapshot_invalid"))?;
    if json_u64(authorization, "grant_target_count")? != snapshot.entry_count
        || json_sha(authorization, "grant_target_snapshot_sha256")? != snapshot.snapshot_sha256
    {
        bail!("grant_snapshot_mismatch")
    }

    let envelope = seal_granted_candidate_install_envelope(GrantedCandidateInstallEnvelope {
        envelope_version: json_u64(authorization, "install_envelope_version")? as u16,
        service_id: SERVICE_ID.to_owned(),
        candidate_sha256: artifact_sha256,
        candidate_byte_len: json_u64(authorization, "install_candidate_byte_len")?,
        activation_approval_sha256: json_sha(authorization, "activation_approval_sha256")?,
        computed_grant_sha256: json_sha(authorization, "computed_grant_sha256")?,
        attestation_reference_sha256: json_sha(authorization, "attestation_reference_sha256")?,
        grant_target_schema: json_text(authorization, "grant_target_schema")?.to_owned(),
        grant_target_count: snapshot.entry_count,
        grant_target_snapshot_sha256: snapshot.snapshot_sha256,
        w7_invocation_sha256: json_sha(authorization, "w7_invocation_sha256")?,
        w7_receipt_sha256: json_sha(authorization, "w7_receipt_sha256")?,
        receiver_content_sha256: artifact_sha256,
        receiver_candidate_sha256: artifact_sha256,
        catalog_candidate_sha256: artifact_sha256,
        generation: json_u64(authorization, "install_generation")?,
        auto_start: true,
        trust_tier: TRUST_TIER.to_owned(),
        envelope_sha256: [0; 32],
    })
    .map_err(|_| anyhow!("authorization_envelope_invalid"))?;
    if envelope.envelope_sha256 != json_sha(authorization, "install_envelope_sha256")? {
        bail!("authorization_envelope_hash_mismatch")
    }

    let signature = hex::decode(json_text(authorization, "install_signature_der")?)
        .map_err(|_| anyhow!("authorization_signature_invalid"))?;
    let unsigned_action = ProjectInstallAction {
        kind: ProjectInstallActionKind::Install,
        authority: ProjectInstallAuthority::PhysicalOwner,
        service_id: SERVICE_ID.to_owned(),
        generation: json_u64(authorization, "install_generation")?,
        log_sequence: json_u64(authorization, "install_log_sequence")?,
        previous_commit_sha256: None,
        install_envelope_sha256: Some(envelope.envelope_sha256),
        target_install_commit_sha256: None,
        authority_evidence_sha256: json_sha(authorization, "authority_evidence_sha256")?,
        physical_approval_sha256: Some(json_sha(authorization, "physical_approval_sha256")?),
        authority_key_sha256: Some(json_sha(authorization, "install_authority_key_sha256")?),
        authority_signature: Vec::new(),
        action_sha256: [0; 32],
    };
    let message_sha256 = install_action_signature_payload_sha256(&unsigned_action)
        .map_err(|_| anyhow!("authorization_signature_payload_invalid"))?;
    if message_sha256 != json_sha(authorization, "install_action_signature_message_sha256")? {
        bail!("authorization_signature_payload_mismatch")
    }
    if signature.len() as u64 != json_u64(authorization, "install_signature_len")?
        || json_sha(authorization, "install_authority_key_sha256")?
            != PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256
        || !verify_promotion_authority_signature(&signature, &message_sha256)
    {
        bail!("authorization_signature_invalid")
    }
    let signed_action = seal_install_action(ProjectInstallAction {
        authority_signature: signature.clone(),
        ..unsigned_action
    })
    .map_err(|_| anyhow!("authorization_action_invalid"))?;
    if signed_action.action_sha256 != json_sha(authorization, "install_action_sha256")? {
        bail!("authorization_action_hash_mismatch")
    }
    if json_sha(authorization, "activation_approval_sha256")? != label_hash("activation-approval")
        || json_sha(authorization, "w7_invocation_sha256")? != label_hash("w7-invocation")
        || json_sha(authorization, "w7_receipt_sha256")? != label_hash("w7-receipt")
        || json_sha(authorization, "authority_evidence_sha256")?
            != label_hash("physical-owner-authority-evidence")
        || json_sha(authorization, "physical_approval_sha256")?
            != label_hash("physical-install-approval")
    {
        bail!("authorization_contract_mismatch")
    }

    let promotion_evidence = verify_promotion_record(
        promotion,
        "promote",
        artifact_sha256,
        parsed.frame_sha256[0],
        json_sha(authorization, "computed_grant_sha256")?,
        snapshot.entry_count,
        snapshot.snapshot_sha256,
    )?;
    let unpromote_evidence = verify_promotion_record(
        unpromote,
        "unpromote",
        artifact_sha256,
        parsed.frame_sha256[0],
        json_sha(authorization, "computed_grant_sha256")?,
        snapshot.entry_count,
        snapshot.snapshot_sha256,
    )?;
    if promotion_evidence.manifest_hash != unpromote_evidence.manifest_hash
        || promotion_evidence.vm_report_hash != unpromote_evidence.vm_report_hash
        || promotion_evidence.computed_grant_hash != unpromote_evidence.computed_grant_hash
        || promotion_evidence.local_attestation_hash != unpromote_evidence.local_attestation_hash
        || promotion_evidence.attestation_reference_hash
            != unpromote_evidence.attestation_reference_hash
        || promotion_evidence.promotion_signature_der != unpromote_evidence.promotion_signature_der
        || json_sha(authorization, "attestation_reference_sha256")?
            != promotion_evidence.attestation_reference_hash
    {
        bail!("promotion_unpromote_link_mismatch")
    }

    let import_set_sha256 = json_sha(artifact_persist, "import_set_hash")?;
    if import_set_sha256 != sha256_bytes(&wasm.import_set_canonical)
        || wasm.grant_target_inventory.len() != 1
        || wasm.grant_target_inventory[0] != json_text(artifact_persist, "grant_target_entries")?
    {
        bail!("import_set_mismatch")
    }
    if json_text(artifact_persist, "id")?
        != "artifact_persist.origin_boot.svc.dev.granted_candidate.v0"
        || json_text(artifact_persist, "scope")? != "origin_boot"
        || json_text(artifact_persist, "classification")? != "local_only"
        || json_text(artifact_persist, "service_id")? != SERVICE_ID
        || json_sha(artifact_persist, "artifact_sha256")? != artifact_sha256
        || json_sha(artifact_persist, "manifest_hash")? != promotion_evidence.manifest_hash
        || json_sha(artifact_persist, "vm_report_hash")? != promotion_evidence.vm_report_hash
        || json_sha(artifact_persist, "grant_hash")? != promotion_evidence.computed_grant_hash
        || json_text(artifact_persist, "grant_target_schema")? != GRANT_TARGET_SNAPSHOT_SCHEMA
        || json_u64(artifact_persist, "grant_target_count")? != snapshot.entry_count
        || json_sha(artifact_persist, "grant_target_snapshot_sha256")? != snapshot.snapshot_sha256
        || json_sha(artifact_persist, "promotion_transaction_sha256")? != parsed.frame_sha256[1]
        || json_bool(artifact_persist, "blob_written_to_disk")? != true
        || json_bool(artifact_persist, "authorizes_load")? != false
        || json_bool(artifact_persist, "owner_sealed")? != false
        || json_bool(artifact_persist, "cross_reboot_proven")? != false
        || json_bool(artifact_persist, "persistence_claimed")? != false
    {
        bail!("artifact_persist_contract_mismatch")
    }

    let frame_sha256: [[u8; 32]; RECORD_COUNT] = parsed
        .frame_sha256
        .try_into()
        .map_err(|_| anyhow!("reclog_record_count_invalid"))?;
    Ok(VerifiedRecords {
        frame_sha256,
        artifact_sha256,
        manifest_hash: promotion_evidence.manifest_hash,
        vm_report_hash: promotion_evidence.vm_report_hash,
        computed_grant_hash: promotion_evidence.computed_grant_hash,
        local_attestation_hash: promotion_evidence.local_attestation_hash,
        attestation_reference_hash: promotion_evidence.attestation_reference_hash,
        promotion_signature_der: promotion_evidence.promotion_signature_der,
        install_action_message_sha256: message_sha256,
        install_signature_der: signature,
        snapshot_count: snapshot.entry_count,
        snapshot_sha256: snapshot.snapshot_sha256,
        import_set_sha256,
        artstor_blob_offset: json_u64(artifact_persist, "artstor_blob_offset")?,
        artstor_blob_len: json_u64(artifact_persist, "artstor_blob_len")?,
        artstor_blob_frame_sha256: json_sha(artifact_persist, "artstor_blob_frame_sha256")?,
    })
}

fn authorization_payload(d: &Derived, artifact_len: u64) -> Result<Vec<u8>> {
    canonical_payload(json!({
        "schema": "raios.install_authorization.v0",
        "id": "install_authorization.origin_boot.svc.dev.granted_candidate.v0",
        "scope": "origin_boot",
        "classification": "local_only",
        "record_kind": "install_authorization",
        "service_id": SERVICE_ID,
        "install_envelope_version": GRANTED_CANDIDATE_INSTALL_ENVELOPE_VERSION,
        "activation_approval_sha256": sha(d.activation_approval_sha256),
        "computed_grant_sha256": sha(d.computed_grant_hash),
        "attestation_reference_sha256": sha(d.attestation_reference_hash),
        "grant_target_schema": GRANT_TARGET_SNAPSHOT_SCHEMA,
        "grant_target_count": d.snapshot.entry_count,
        "grant_target_snapshot_sha256": sha(d.snapshot.snapshot_sha256),
        "w7_invocation_sha256": sha(d.w7_invocation_sha256),
        "w7_receipt_sha256": sha(d.w7_receipt_sha256),
        "receiver_content_sha256": sha(d.artifact_sha256),
        "receiver_candidate_sha256": sha(d.artifact_sha256),
        "catalog_candidate_sha256": sha(d.artifact_sha256),
        "install_envelope_sha256": sha(d.install_envelope_sha256),
        "install_action_sha256": sha(d.install_action_sha256),
        "install_action_signature_message_sha256": sha(d.install_action_message_sha256),
        "authority_evidence_sha256": sha(d.authority_evidence_sha256),
        "physical_approval_sha256": sha(d.physical_approval_sha256),
        "install_authority_key_sha256": sha(PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256),
        "install_signature_der": hex::encode(&d.install_signature_der),
        "install_signature_len": d.install_signature_der.len(),
        "install_candidate_sha256": sha(d.artifact_sha256),
        "install_candidate_byte_len": artifact_len,
        "install_generation": 1,
        "install_log_sequence": 1,
        "trust_tier": TRUST_TIER
    }))
}

fn promotion_payload(
    d: &Derived,
    kind: &str,
    authorization_frame_sha256: [u8; 32],
) -> Result<Vec<u8>> {
    let unpromote = kind == "unpromote";
    canonical_payload(json!({
        "schema": "raios.promotion_transaction.v0",
        "id": format!("promotion_transaction.origin_boot.{kind}.svc.dev.granted_candidate.v0"),
        "scope": "origin_boot",
        "classification": "local_only",
        "record_kind": "promotion_transaction",
        "transaction_kind": kind,
        "service_id": SERVICE_ID,
        "artifact_id": ARTIFACT_ID,
        "requested_capability": REQUESTED_CAPABILITY,
        "load_mode": LOAD_MODE,
        "computed_grant_hash": sha(d.computed_grant_hash),
        "manifest_hash": sha(d.manifest_hash),
        "artifact_hash": sha(d.artifact_sha256),
        "vm_report_hash": sha(d.vm_report_hash),
        "local_attestation_hash": sha(d.local_attestation_hash),
        "retained_manifest_reference_event_id": event_id(101),
        "retained_artifact_reference_event_id": event_id(102),
        "retained_vm_report_reference_event_id": event_id(103),
        "retained_computed_grant_reference_event_id": event_id(104),
        "manifest_reference_hash": sha(d.manifest_reference_hash),
        "artifact_reference_hash": sha(d.artifact_reference_hash),
        "vm_report_reference_hash": sha(d.vm_report_reference_hash),
        "attestation_reference_hash": sha(d.attestation_reference_hash),
        "promotion_signature_der": hex::encode(&d.promotion_signature_der),
        "promotion_signature_len": d.promotion_signature_der.len(),
        "promotion_authority_key_sha256": sha(PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256),
        "signature_verified": true,
        "grant_binds_capability": true,
        "install_authorization_present": true,
        "install_envelope_binds_activation": true,
        "install_action_signature_verified": true,
        "physical_install_approval_consumed": true,
        "install_authorization_frame_sha256": sha(authorization_frame_sha256),
        "install_envelope_version": GRANTED_CANDIDATE_INSTALL_ENVELOPE_VERSION,
        "grant_target_schema": GRANT_TARGET_SNAPSHOT_SCHEMA,
        "grant_target_count": d.snapshot.entry_count,
        "grant_target_snapshot_sha256": sha(d.snapshot.snapshot_sha256),
        "trust_tier": TRUST_TIER,
        "promotion_authority_is_placeholder": true,
        "owner_sealed": false,
        "cross_reboot_proven": false,
        "persistence_claimed": false,
        "rollback_plan_hash": sha(label_hash("rollback-plan-log-only")),
        "pre_load_inventory_hash": sha(label_hash("pre-load-inventory-log-only")),
        "ram_only_service_slot_id": "ram.slot.granted_candidate.current_boot",
        "generation": 1,
        "load_event_id": event_id(105),
        "rollback_apply_event_id": if unpromote { Value::String(event_id(106)) } else { Value::Null },
        "reprojected_inventory_hash": if unpromote { Value::String(sha(label_hash("reprojected-inventory-log-only"))) } else { Value::Null },
        "restore_hash_verified": unpromote,
        "stopped": unpromote,
        "drop_clear_bytes": unpromote,
        "free_slot": unpromote,
        "remove_inventory": unpromote
    }))
}

fn artifact_payload(
    d: &Derived,
    artstor_len: u64,
    artstor_sha256: [u8; 32],
    promote_sha256: [u8; 32],
) -> Result<Vec<u8>> {
    canonical_payload(json!({
        "schema": "raios.artifact_persist.v0",
        "id": "artifact_persist.origin_boot.svc.dev.granted_candidate.v0",
        "scope": "origin_boot",
        "classification": "local_only",
        "record_kind": "artifact_persist",
        "artstor_blob_offset": 0,
        "artstor_blob_len": artstor_len,
        "artstor_blob_frame_sha256": sha(artstor_sha256),
        "artifact_sha256": sha(d.artifact_sha256),
        "manifest_hash": sha(d.manifest_hash),
        "vm_report_hash": sha(d.vm_report_hash),
        "grant_hash": sha(d.computed_grant_hash),
        "service_id": SERVICE_ID,
        "import_set_hash": sha(sha256_bytes(IMPORT_SET_CANONICAL)),
        "grant_target_schema": GRANT_TARGET_SNAPSHOT_SCHEMA,
        "grant_target_entries": "env.log:host_call",
        "grant_target_count": d.snapshot.entry_count,
        "grant_target_snapshot_sha256": sha(d.snapshot.snapshot_sha256),
        "promotion_transaction_sha256": sha(promote_sha256),
        "blob_written_to_disk": true,
        "authorizes_load": false,
        "owner_sealed": false,
        "cross_reboot_proven": false,
        "persistence_claimed": false
    }))
}

fn append_record(region: &mut [u8], payload: &[u8]) -> Result<[u8; 32]> {
    let scan = scan_reclog(region);
    let planned =
        plan_reclog_append(&scan, payload, region.len() as u64).map_err(|e| anyhow!(e.reason()))?;
    let start = planned.write_offset as usize;
    let end = start + planned.frame.len();
    region[start..end].copy_from_slice(&planned.frame);
    Ok(planned.frame_sha256)
}

fn build_bundle(artifact: Vec<u8>) -> Result<Bundle> {
    let wasm = verify_log_only_wasm(&artifact).context("generated Wasm semantic verification")?;
    let d = derive(&artifact)?;
    let artstor = plan_artifact_blob_write(0, &artifact, ARTSTOR_BYTE_COUNT)
        .map_err(|e| anyhow!(e.reason()))?;
    let mut region = vec![0u8; RECLOG_BYTE_COUNT];
    let authorization_sha = append_record(
        &mut region,
        &authorization_payload(&d, artifact.len() as u64)?,
    )?;
    let promote_sha = append_record(
        &mut region,
        &promotion_payload(&d, "promote", authorization_sha)?,
    )?;
    let artifact_sha = append_record(
        &mut region,
        &artifact_payload(&d, artstor.frame_len, artstor.frame_sha256, promote_sha)?,
    )?;
    let unpromote_sha = append_record(
        &mut region,
        &promotion_payload(&d, "unpromote", authorization_sha)?,
    )?;
    let scan = scan_reclog(&region);
    if !scan.full_region_valid || scan.count != RECORD_COUNT as u64 {
        bail!("generated RECLOG did not re-scan as four valid frames")
    }
    let used = scan.first_invalid_offset as usize;
    let signed_v2_linkage_verified = {
        verify_signed_v2_linkage(&region[..used], &wasm)
            .context("generated semantic RECLOG verification")?;
        true
    };
    let metadata = Metadata {
        schema: "raios.dev_rollback_fixture.v1".to_owned(),
        fixture: "rollback-target-log-only".to_owned(),
        artifact_sha256: sha(d.artifact_sha256),
        artifact_byte_len: artifact.len() as u64,
        import_inventory: wasm.import_inventory.clone(),
        import_set_sha256: sha(sha256_bytes(&wasm.import_set_canonical)),
        grant_target_entries: wasm.grant_target_inventory.clone(),
        grant_target_count: d.snapshot.entry_count,
        grant_target_snapshot_sha256: sha(d.snapshot.snapshot_sha256),
        artstor_blob_len: artstor.frame_len,
        artstor_blob_frame_sha256: sha(artstor.frame_sha256),
        install_authorization_frame_sha256: sha(authorization_sha),
        promote_frame_sha256: sha(promote_sha),
        artifact_persist_frame_sha256: sha(artifact_sha),
        unpromote_frame_sha256: sha(unpromote_sha),
        reclog_count: scan.count,
        reclog_tail_sha256: sha(scan.tail_frame_sha256.unwrap_or([0; 32])),
        promotion_signature_verified: verify_promotion_authority_signature(
            &d.promotion_signature_der,
            &d.attestation_reference_hash,
        ),
        install_signature_verified: verify_promotion_authority_signature(
            &d.install_signature_der,
            &d.install_action_message_sha256,
        ),
        signed_v2_linkage_verified,
        kernel_repromotion_semantics_verified: m6_grant_formula_valid(&d)
            && verify_repromotion_semantics(&d, &artstor.frame),
    };
    if !metadata.promotion_signature_verified
        || !metadata.install_signature_verified
        || !metadata.signed_v2_linkage_verified
        || !metadata.kernel_repromotion_semantics_verified
    {
        bail!("generated authority evidence did not pass the raios-core verifiers")
    }
    Ok(Bundle {
        artifact,
        artstor: artstor.frame,
        reclog: region[..used].to_vec(),
        metadata,
    })
}

fn verify_repromotion_semantics(d: &Derived, artstor: &[u8]) -> bool {
    reverify_persisted_artifact(&RepromotionReverifyInput {
        artstor_blob_bytes: artstor,
        record_artstor_blob_frame_sha256: sha256_bytes(artstor),
        record_artifact_sha256: d.artifact_sha256,
        record_manifest_hash: d.manifest_hash,
        record_vm_report_hash: d.vm_report_hash,
        record_grant_hash: d.computed_grant_hash,
        transaction: Some(RepromotionTransactionFields {
            computed_grant_hash: d.computed_grant_hash,
            manifest_hash: d.manifest_hash,
            artifact_hash: d.artifact_sha256,
            vm_report_hash: d.vm_report_hash,
            local_attestation_hash: d.local_attestation_hash,
            attestation_reference_hash: d.attestation_reference_hash,
            promotion_signature_der: &d.promotion_signature_der,
            promotion_authority_key_sha256: PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256,
            signature_verified: true,
            grant_binds_capability: true,
            install_authorization_present: true,
            install_envelope_binds_activation: true,
            install_action_signature_message_sha256: d.install_action_message_sha256,
            install_action_signature_der: &d.install_signature_der,
            install_authority_key_sha256: PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256,
            install_action_signature_verified: true,
            physical_install_approval_consumed: true,
        }),
        recomputed_attestation_reference_hash: d.attestation_reference_hash,
        promotion_authority_is_placeholder: true,
    })
    .reverified
}

fn m6_grant_formula_valid(d: &Derived) -> bool {
    d.computed_grant_hash
        == computed_module_grant_hash(
            d.manifest_hash,
            d.artifact_sha256,
            d.vm_report_hash,
            d.local_attestation_hash,
        )
}

fn write_bundle(out: &Path, bundle: &Bundle) -> Result<()> {
    fs::create_dir_all(out).with_context(|| format!("create {}", out.display()))?;
    fs::write(out.join("artifact.wasm"), &bundle.artifact)?;
    fs::write(out.join("artstor.bin"), &bundle.artstor)?;
    fs::write(out.join("reclog.bin"), &bundle.reclog)?;
    let mut metadata = serde_json::to_vec_pretty(&bundle.metadata)?;
    metadata.push(b'\n');
    fs::write(out.join("metadata.json"), metadata)?;
    Ok(())
}

fn verify_directory(out: &Path) -> Result<()> {
    let artifact =
        fs::read(out.join("artifact.wasm")).map_err(|_| anyhow!("artifact_read_failed"))?;
    let wasm = verify_log_only_wasm(&artifact)?;
    let actual_reclog =
        fs::read(out.join("reclog.bin")).map_err(|_| anyhow!("reclog_read_failed"))?;
    let records = verify_signed_v2_linkage(&actual_reclog, &wasm)?;
    let actual_artstor =
        fs::read(out.join("artstor.bin")).map_err(|_| anyhow!("artstor_read_failed"))?;
    verify_storage_bytes(&records, &actual_artstor, &wasm)?;
    let metadata_bytes =
        fs::read(out.join("metadata.json")).map_err(|_| anyhow!("metadata_read_failed"))?;
    let actual_metadata: Metadata =
        serde_json::from_slice(&metadata_bytes).map_err(|_| anyhow!("metadata_parse_invalid"))?;
    verify_metadata(&actual_metadata, &records, &wasm, &actual_artstor)?;
    println!("verified: semantic metadata + 4 ordered RECLOG records + exact ARTSTOR range/payload + persisted install/promotion signatures");
    Ok(())
}

fn verify_storage_bytes(
    records: &VerifiedRecords,
    artstor: &[u8],
    wasm: &VerifiedWasm,
) -> Result<()> {
    if records.artstor_blob_offset != 0
        || records.artstor_blob_len != artstor.len() as u64
        || records
            .artstor_blob_offset
            .checked_add(records.artstor_blob_len)
            .filter(|end| *end <= ARTSTOR_BYTE_COUNT)
            .is_none()
    {
        bail!("artstor_range_mismatch")
    }
    let parsed = parse_artifact_blob_frame(artstor, records.artstor_blob_offset as usize)
        .map_err(|_| anyhow!("artstor_frame_invalid"))?;
    if parsed.frame_len as u64 != records.artstor_blob_len
        || parsed.frame_len as usize != artstor.len()
    {
        bail!("artstor_range_mismatch")
    }
    if sha256_bytes(artstor) != records.artstor_blob_frame_sha256 {
        bail!("artstor_frame_hash_mismatch")
    }
    if records.artifact_sha256 != wasm.artifact_sha256 {
        bail!("candidate_artifact_binding_mismatch")
    }
    let reverified = reverify_persisted_artifact(&RepromotionReverifyInput {
        artstor_blob_bytes: artstor,
        record_artstor_blob_frame_sha256: records.artstor_blob_frame_sha256,
        record_artifact_sha256: records.artifact_sha256,
        record_manifest_hash: records.manifest_hash,
        record_vm_report_hash: records.vm_report_hash,
        record_grant_hash: records.computed_grant_hash,
        transaction: Some(RepromotionTransactionFields {
            computed_grant_hash: records.computed_grant_hash,
            manifest_hash: records.manifest_hash,
            artifact_hash: records.artifact_sha256,
            vm_report_hash: records.vm_report_hash,
            local_attestation_hash: records.local_attestation_hash,
            attestation_reference_hash: records.attestation_reference_hash,
            promotion_signature_der: &records.promotion_signature_der,
            promotion_authority_key_sha256: PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256,
            signature_verified: true,
            grant_binds_capability: true,
            install_authorization_present: true,
            install_envelope_binds_activation: true,
            install_action_signature_message_sha256: records.install_action_message_sha256,
            install_action_signature_der: &records.install_signature_der,
            install_authority_key_sha256: PLACEHOLDER_PROMOTION_AUTHORITY_PUBLIC_KEY_SHA256,
            install_action_signature_verified: true,
            physical_install_approval_consumed: true,
        }),
        recomputed_attestation_reference_hash: records.attestation_reference_hash,
        promotion_authority_is_placeholder: true,
    });
    if !reverified.reverified {
        bail!("artstor_payload_hash_mismatch")
    }
    Ok(())
}

fn verify_metadata(
    metadata: &Metadata,
    records: &VerifiedRecords,
    wasm: &VerifiedWasm,
    artstor: &[u8],
) -> Result<()> {
    if metadata.schema != "raios.dev_rollback_fixture.v1"
        || metadata.fixture != "rollback-target-log-only"
        || metadata.artifact_sha256 != sha(wasm.artifact_sha256)
        || metadata.artifact_byte_len != wasm.byte_len
        || metadata.import_inventory != wasm.import_inventory
        || metadata.import_set_sha256 != sha(sha256_bytes(&wasm.import_set_canonical))
        || records.import_set_sha256 != sha256_bytes(&wasm.import_set_canonical)
        || metadata.grant_target_entries != wasm.grant_target_inventory
        || metadata.grant_target_count != records.snapshot_count
        || metadata.grant_target_snapshot_sha256 != sha(records.snapshot_sha256)
        || metadata.artstor_blob_len != artstor.len() as u64
        || metadata.artstor_blob_frame_sha256 != sha(records.artstor_blob_frame_sha256)
        || metadata.install_authorization_frame_sha256 != sha(records.frame_sha256[0])
        || metadata.promote_frame_sha256 != sha(records.frame_sha256[1])
        || metadata.artifact_persist_frame_sha256 != sha(records.frame_sha256[2])
        || metadata.unpromote_frame_sha256 != sha(records.frame_sha256[3])
        || metadata.reclog_count != RECORD_COUNT as u64
        || metadata.reclog_tail_sha256 != sha(records.frame_sha256[3])
        || !metadata.promotion_signature_verified
        || !metadata.install_signature_verified
        || !metadata.signed_v2_linkage_verified
        || !metadata.kernel_repromotion_semantics_verified
    {
        bail!("metadata_contract_mismatch")
    }
    Ok(())
}

fn rewrite_reclog<F>(root: &Path, mutate: F) -> Result<()>
where
    F: FnOnce(&mut Vec<Value>) -> Result<()>,
{
    let bytes = fs::read(root.join("reclog.bin"))?;
    let parsed = parse_reclog_records(&bytes)?;
    let mut records = parsed.records;
    mutate(&mut records)?;
    let mut region = vec![0u8; RECLOG_BYTE_COUNT];
    for record in records {
        append_record(&mut region, &canonical_payload(record)?)?;
    }
    let scan = scan_reclog(&region);
    if !scan.full_region_valid || scan.count != RECORD_COUNT as u64 {
        bail!("selftest_reclog_reframe_failed")
    }
    fs::write(
        root.join("reclog.bin"),
        &region[..scan.first_invalid_offset as usize],
    )?;
    Ok(())
}

fn set_record_field(
    records: &mut [Value],
    record_index: usize,
    field: &str,
    value: Value,
) -> Result<()> {
    let record = records
        .get_mut(record_index)
        .and_then(Value::as_object_mut)
        .ok_or_else(|| anyhow!("selftest_record_missing"))?;
    if !record.contains_key(field) {
        bail!("selftest_field_missing:{field}")
    }
    record.insert(field.to_owned(), value);
    Ok(())
}

fn expect_mutation<F>(root: &Path, label: &str, expected_reason: &str, mutate: F) -> Result<()>
where
    F: FnOnce(&Path) -> Result<()>,
{
    const FILES: [&str; 4] = [
        "artifact.wasm",
        "artstor.bin",
        "reclog.bin",
        "metadata.json",
    ];
    let original = FILES
        .iter()
        .map(|file| Ok((*file, fs::read(root.join(file))?)))
        .collect::<Result<Vec<_>>>()?;
    mutate(root)?;
    let observed = match verify_directory(root) {
        Ok(()) => "accepted".to_owned(),
        Err(error) => error.to_string(),
    };
    for (file, bytes) in original {
        fs::write(root.join(file), bytes)?;
    }
    if observed != expected_reason {
        bail!("negative {label}: expected {expected_reason}, observed {observed}")
    }
    println!("negative {label}: {observed}");
    Ok(())
}

fn expect_wasm_decoder_rejection<F>(
    artifact: &[u8],
    label: &str,
    expected_reason: &str,
    mutate: F,
) -> Result<()>
where
    F: FnOnce(&mut Vec<u8>) -> Result<()>,
{
    let mut changed = artifact.to_vec();
    mutate(&mut changed)?;
    let observed = match verify_log_only_wasm(&changed) {
        Ok(_) => "accepted".to_owned(),
        Err(error) => error.to_string(),
    };
    if observed != expected_reason {
        bail!("negative Wasm decoder {label}: expected {expected_reason}, observed {observed}")
    }
    println!("negative Wasm decoder {label}: {observed}");
    Ok(())
}

fn self_test(wasm: &Path) -> Result<()> {
    let artifact = read_exact_artifact(wasm)?;
    let derived = derive(&artifact)?;
    if !m6_grant_formula_valid(&derived) {
        bail!("positive M6 computed grant formula rejected")
    }
    let mut reframed_grant = derived.clone();
    reframed_grant.computed_grant_hash[0] ^= 1;
    if m6_grant_formula_valid(&reframed_grant) {
        bail!("mutated M6 computed grant formula unexpectedly accepted")
    }
    expect_wasm_decoder_rejection(
        &artifact,
        "noncanonical LEB",
        "wasm_binary_invalid",
        |bytes| {
            if bytes.get(8) != Some(&1) || bytes.get(9) != Some(&10) {
                bail!("selftest_wasm_type_section_missing")
            }
            bytes[9] = 0x8a;
            bytes.insert(10, 0x00);
            Ok(())
        },
    )?;
    expect_wasm_decoder_rejection(
        &artifact,
        "duplicate section",
        "wasm_section_order_invalid",
        |bytes| {
            bytes.extend_from_slice(&[0x01, 0x01, 0x00]);
            Ok(())
        },
    )?;
    expect_wasm_decoder_rejection(
        &artifact,
        "unknown section",
        "wasm_section_contract_mismatch",
        |bytes| {
            bytes.extend_from_slice(&[0x0d, 0x00]);
            Ok(())
        },
    )?;
    expect_wasm_decoder_rejection(
        &artifact,
        "truncated trailing section",
        "wasm_binary_invalid",
        |bytes| {
            bytes.extend_from_slice(&[0x00, 0x05]);
            Ok(())
        },
    )?;
    let root = env::temp_dir().join(format!(
        "raios-dev-rollback-fixture-selftest-{}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    write_bundle(&root, &build_bundle(artifact)?)?;
    verify_directory(&root)?;

    expect_mutation(
        &root,
        "candidate/artifact binding",
        "candidate_artifact_binding_mismatch",
        |root| {
            rewrite_reclog(root, |records| {
                set_record_field(
                    records,
                    0,
                    "install_candidate_sha256",
                    Value::String(sha(label_hash("mutated-candidate"))),
                )
            })
        },
    )?;
    expect_mutation(&root, "import set", "import_set_mismatch", |root| {
        rewrite_reclog(root, |records| {
            set_record_field(
                records,
                2,
                "import_set_hash",
                Value::String(sha(label_hash("mutated-import-set"))),
            )
        })
    })?;
    expect_mutation(
        &root,
        "authorization field",
        "authorization_signature_payload_mismatch",
        |root| {
            rewrite_reclog(root, |records| {
                set_record_field(
                    records,
                    0,
                    "authority_evidence_sha256",
                    Value::String(sha(label_hash("mutated-authority-evidence"))),
                )
            })
        },
    )?;
    expect_mutation(
        &root,
        "promotion field",
        "promotion_contract_mismatch",
        |root| {
            rewrite_reclog(root, |records| {
                set_record_field(records, 1, "signature_verified", Value::Bool(false))
            })
        },
    )?;
    expect_mutation(
        &root,
        "promotion signature",
        "promotion_signature_invalid",
        |root| {
            rewrite_reclog(root, |records| {
                let mut signature =
                    hex::decode(json_text(&records[1], "promotion_signature_der")?)?;
                let last = signature
                    .last_mut()
                    .ok_or_else(|| anyhow!("selftest_signature_empty"))?;
                *last ^= 1;
                set_record_field(
                    records,
                    1,
                    "promotion_signature_der",
                    Value::String(hex::encode(signature)),
                )
            })
        },
    )?;
    expect_mutation(
        &root,
        "ARTSTOR payload",
        "artstor_payload_hash_mismatch",
        |root| {
            let mut changed_payload = fs::read(root.join("artifact.wasm"))?;
            changed_payload[8] ^= 1;
            let changed_frame = plan_artifact_blob_write(0, &changed_payload, ARTSTOR_BYTE_COUNT)
                .map_err(|e| anyhow!(e.reason()))?;
            fs::write(root.join("artstor.bin"), &changed_frame.frame)?;
            rewrite_reclog(root, |records| {
                set_record_field(
                    records,
                    2,
                    "artstor_blob_len",
                    json!(changed_frame.frame_len),
                )?;
                set_record_field(
                    records,
                    2,
                    "artstor_blob_frame_sha256",
                    Value::String(sha(changed_frame.frame_sha256)),
                )
            })
        },
    )?;
    expect_mutation(&root, "ARTSTOR frame", "artstor_frame_invalid", |root| {
        let path = root.join("artstor.bin");
        let mut bytes = fs::read(&path)?;
        bytes[0] ^= 1;
        fs::write(path, bytes)?;
        Ok(())
    })?;
    expect_mutation(
        &root,
        "ARTSTOR frame hash",
        "artstor_frame_hash_mismatch",
        |root| {
            rewrite_reclog(root, |records| {
                set_record_field(
                    records,
                    2,
                    "artstor_blob_frame_sha256",
                    Value::String(sha(label_hash("mutated-artstor-frame"))),
                )
            })
        },
    )?;
    expect_mutation(&root, "ARTSTOR range", "artstor_range_mismatch", |root| {
        rewrite_reclog(root, |records| {
            set_record_field(records, 2, "artstor_blob_offset", json!(1))
        })
    })?;
    expect_mutation(&root, "RECLOG link", "reclog_chain_invalid", |root| {
        let path = root.join("reclog.bin");
        let mut bytes = fs::read(&path)?;
        let parsed = parse_reclog_records(&bytes)?;
        if parsed.frame_offset[1] != parsed.frame_len[0] as usize {
            bail!("selftest_second_frame_offset_invalid")
        }
        let header_start = parsed.frame_offset[1];
        let header_end = header_start + RECLOG_FRAME_HEADER_LEN;
        let link_offset = bytes[header_start..header_end]
            .windows(32)
            .position(|window| window == parsed.frame_sha256[0])
            .ok_or_else(|| anyhow!("selftest_reclog_link_missing"))?;
        bytes[header_start + link_offset] ^= 1;
        fs::write(path, bytes)?;
        Ok(())
    })?;
    expect_mutation(
        &root,
        "RECLOG order",
        "reclog_record_order_mismatch",
        |root| {
            rewrite_reclog(root, |records| {
                records.swap(1, 2);
                Ok(())
            })
        },
    )?;
    expect_mutation(
        &root,
        "Wasm import contract",
        "wasm_import_contract_mismatch",
        |root| {
            let path = root.join("artifact.wasm");
            let mut bytes = fs::read(&path)?;
            let import_name = bytes
                .windows(3)
                .position(|window| window == b"log")
                .ok_or_else(|| anyhow!("selftest_wasm_import_missing"))?;
            bytes[import_name + 2] = b'x';
            fs::write(path, bytes)?;
            Ok(())
        },
    )?;
    expect_mutation(
        &root,
        "semantic-equivalent Wasm binding",
        "candidate_artifact_binding_mismatch",
        |root| {
            let path = root.join("artifact.wasm");
            let mut bytes = fs::read(&path)?;
            bytes.extend_from_slice(&[0x00, 0x05, 0x03, b'a', b'l', b't', 0x00]);
            fs::write(path, bytes)?;
            Ok(())
        },
    )?;
    expect_mutation(
        &root,
        "metadata contract",
        "metadata_contract_mismatch",
        |root| {
            let path = root.join("metadata.json");
            let mut metadata: Value = serde_json::from_slice(&fs::read(&path)?)?;
            metadata["promotion_signature_verified"] = Value::Bool(false);
            fs::write(path, canonical_payload(metadata)?)?;
            Ok(())
        },
    )?;

    verify_directory(&root)?;
    fs::remove_dir_all(&root)?;
    println!("self-test: semantic positive + M6 formula negative + 4 decoder boundaries + 14 precise cross-file mutations passed");
    Ok(())
}
