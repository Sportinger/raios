#![cfg_attr(not(test), no_std)]

//! Bounded `RAIOS_WASM_IR_V1` parser and canonical WebAssembly emitter.
//!
//! The grammar is exact, ASCII-only, and LF-terminated:
//!
//! ```text
//! RAIOS_WASM_IR_V1
//! func <name> -> i32
//! const.i32 <canonical-i32>
//! return
//! end
//! [memory <min-pages> <max-pages>]
//! [data <offset> <lowercase-even-hex>]*
//! ```
//!
//! There are no comments, blank lines, imports, locals, calls, tables, or
//! control-flow instructions. The function is exported under `<name>`.

use core::str;

pub const MAX_IR_SOURCE_BYTES: usize = 4096;
pub const MAX_WASM_OUTPUT_BYTES: usize = 4096;
pub const MAX_FUNCTION_NAME_BYTES: usize = 64;
pub const MAX_MEMORY_PAGES: u32 = 4;
pub const MAX_DATA_SEGMENTS: usize = 8;
pub const MAX_DATA_BYTES: usize = 2000;

pub const RETURN_42_IR: &[u8] =
    b"RAIOS_WASM_IR_V1\nfunc raios_service_main -> i32\nconst.i32 42\nreturn\nend\n";

pub const ERR_DUPLICATE_SECTION: &str = "duplicate_section";
pub const ERR_INVALID_FUNCTION_NAME: &str = "invalid_function_name";
pub const ERR_MISSING_TERMINATOR: &str = "missing_terminator";
pub const ERR_NON_ASCII_SOURCE: &str = "non_ascii_source";
pub const ERR_NON_CANONICAL_HEX_TEXT: &str = "non_canonical_hex_text";
pub const ERR_NON_CANONICAL_INTEGER_TEXT: &str = "non_canonical_integer_text";
pub const ERR_OVERSIZE_OUTPUT: &str = "oversize_output";
pub const ERR_OVERSIZE_SOURCE: &str = "oversize_source";
pub const ERR_QUOTA_EXCEEDED: &str = "quota_exceeded";
pub const ERR_TRAILING_CONTENT: &str = "trailing_content";
pub const ERR_UNKNOWN_DIRECTIVE: &str = "unknown_directive";
pub const ERR_WRONG_VERSION_LINE: &str = "wrong_version_line";

const VERSION_LINE: &str = "RAIOS_WASM_IR_V1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmModuleBytes {
    bytes: [u8; MAX_WASM_OUTPUT_BYTES],
    len: usize,
}

impl WasmModuleBytes {
    const fn new() -> Self {
        Self {
            bytes: [0; MAX_WASM_OUTPUT_BYTES],
            len: 0,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn push(&mut self, byte: u8) -> Result<(), &'static str> {
        if self.len == self.bytes.len() {
            return Err(ERR_OVERSIZE_OUTPUT);
        }
        self.bytes[self.len] = byte;
        self.len += 1;
        Ok(())
    }

    fn extend(&mut self, bytes: &[u8]) -> Result<(), &'static str> {
        let end = self
            .len
            .checked_add(bytes.len())
            .ok_or(ERR_OVERSIZE_OUTPUT)?;
        let Some(output) = self.bytes.get_mut(self.len..end) else {
            return Err(ERR_OVERSIZE_OUTPUT);
        };
        output.copy_from_slice(bytes);
        self.len = end;
        Ok(())
    }

    fn push_u32_leb(&mut self, mut value: u32) -> Result<(), &'static str> {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            self.push(byte)?;
            if value == 0 {
                return Ok(());
            }
        }
    }

    fn push_i32_leb(&mut self, mut value: i32) -> Result<(), &'static str> {
        loop {
            let mut byte = (value as u8) & 0x7f;
            value >>= 7;
            let done = (value == 0 && byte & 0x40 == 0) || (value == -1 && byte & 0x40 != 0);
            if !done {
                byte |= 0x80;
            }
            self.push(byte)?;
            if done {
                return Ok(());
            }
        }
    }
}

impl AsRef<[u8]> for WasmModuleBytes {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

#[derive(Clone, Copy)]
struct DataSegment<'a> {
    offset: u32,
    hex: &'a [u8],
    len: usize,
}

struct ParsedModule<'a> {
    function_name: &'a str,
    value: i32,
    memory: Option<(u32, u32)>,
    data: [Option<DataSegment<'a>>; MAX_DATA_SEGMENTS],
    data_count: usize,
}

/// Parses one bounded IR source and emits its sole canonical WebAssembly form.
pub fn assemble(source: &[u8]) -> Result<WasmModuleBytes, &'static str> {
    emit(&parse(source)?)
}

fn parse(source: &[u8]) -> Result<ParsedModule<'_>, &'static str> {
    if source.len() > MAX_IR_SOURCE_BYTES {
        return Err(ERR_OVERSIZE_SOURCE);
    }
    if source.iter().any(|byte| !byte.is_ascii()) {
        return Err(ERR_NON_ASCII_SOURCE);
    }
    if !source.ends_with(b"\n") {
        return Err(ERR_MISSING_TERMINATOR);
    }
    let text = str::from_utf8(source).map_err(|_| ERR_NON_ASCII_SOURCE)?;
    let mut lines = text.split_terminator('\n');
    if lines.next() != Some(VERSION_LINE) {
        return Err(ERR_WRONG_VERSION_LINE);
    }

    let function_line = lines.next().ok_or(ERR_MISSING_TERMINATOR)?;
    let function_name = parse_function(function_line)?;
    let const_line = lines.next().ok_or(ERR_MISSING_TERMINATOR)?;
    let value = const_line
        .strip_prefix("const.i32 ")
        .ok_or_else(|| unexpected_directive(const_line))
        .and_then(parse_i32)?;
    expect_line(lines.next(), "return")?;
    expect_line(lines.next(), "end")?;

    let mut parsed = ParsedModule {
        function_name,
        value,
        memory: None,
        data: [None; MAX_DATA_SEGMENTS],
        data_count: 0,
    };
    let mut data_bytes = 0usize;
    for line in lines {
        if line.starts_with("func ") || line == VERSION_LINE {
            return Err(ERR_DUPLICATE_SECTION);
        }
        if line.starts_with("memory ") {
            if parsed.memory.is_some() {
                return Err(ERR_DUPLICATE_SECTION);
            }
            if parsed.data_count != 0 {
                return Err(ERR_TRAILING_CONTENT);
            }
            parsed.memory = Some(parse_memory(line)?);
            continue;
        }
        if line.starts_with("data ") {
            if parsed.data_count == MAX_DATA_SEGMENTS {
                return Err(ERR_QUOTA_EXCEEDED);
            }
            let segment = parse_data(line, parsed.memory)?;
            data_bytes = data_bytes
                .checked_add(segment.len)
                .ok_or(ERR_OVERSIZE_OUTPUT)?;
            if data_bytes > MAX_DATA_BYTES {
                return Err(ERR_OVERSIZE_OUTPUT);
            }
            parsed.data[parsed.data_count] = Some(segment);
            parsed.data_count += 1;
            continue;
        }
        return Err(ERR_TRAILING_CONTENT);
    }
    Ok(parsed)
}

fn parse_function(line: &str) -> Result<&str, &'static str> {
    let name = line
        .strip_prefix("func ")
        .and_then(|rest| rest.strip_suffix(" -> i32"))
        .ok_or(ERR_UNKNOWN_DIRECTIVE)?;
    let mut bytes = name.bytes();
    let Some(first) = bytes.next() else {
        return Err(ERR_INVALID_FUNCTION_NAME);
    };
    if name.len() > MAX_FUNCTION_NAME_BYTES
        || !first.is_ascii_lowercase()
        || !bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'-')
        })
    {
        return Err(ERR_INVALID_FUNCTION_NAME);
    }
    Ok(name)
}

fn expect_line(line: Option<&str>, expected: &str) -> Result<(), &'static str> {
    let line = line.ok_or(ERR_MISSING_TERMINATOR)?;
    if line == expected {
        return Ok(());
    }
    Err(unexpected_directive(line))
}

fn unexpected_directive(line: &str) -> &'static str {
    if line.starts_with("func ") || line.starts_with("memory ") || line == VERSION_LINE {
        ERR_DUPLICATE_SECTION
    } else {
        ERR_UNKNOWN_DIRECTIVE
    }
}

fn parse_memory(line: &str) -> Result<(u32, u32), &'static str> {
    let mut fields = line.split(' ');
    if fields.next() != Some("memory") {
        return Err(ERR_UNKNOWN_DIRECTIVE);
    }
    let min = fields
        .next()
        .ok_or(ERR_UNKNOWN_DIRECTIVE)
        .and_then(parse_u32)?;
    let max = fields
        .next()
        .ok_or(ERR_UNKNOWN_DIRECTIVE)
        .and_then(parse_u32)?;
    if fields.next().is_some() {
        return Err(ERR_UNKNOWN_DIRECTIVE);
    }
    if min > max || max > MAX_MEMORY_PAGES {
        return Err(ERR_QUOTA_EXCEEDED);
    }
    Ok((min, max))
}

fn parse_data(line: &str, memory: Option<(u32, u32)>) -> Result<DataSegment<'_>, &'static str> {
    let mut fields = line.split(' ');
    if fields.next() != Some("data") {
        return Err(ERR_UNKNOWN_DIRECTIVE);
    }
    let offset = fields
        .next()
        .ok_or(ERR_UNKNOWN_DIRECTIVE)
        .and_then(parse_u32)?;
    let hex = fields.next().ok_or(ERR_NON_CANONICAL_HEX_TEXT)?;
    if fields.next().is_some()
        || hex.is_empty()
        || hex.len() % 2 != 0
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(ERR_NON_CANONICAL_HEX_TEXT);
    }
    let len = hex.len() / 2;
    let Some((min_pages, _)) = memory else {
        return Err(ERR_QUOTA_EXCEEDED);
    };
    let end = offset
        .checked_add(u32::try_from(len).map_err(|_| ERR_QUOTA_EXCEEDED)?)
        .ok_or(ERR_QUOTA_EXCEEDED)?;
    if end > min_pages * 65_536 {
        return Err(ERR_QUOTA_EXCEEDED);
    }
    Ok(DataSegment {
        offset,
        hex: hex.as_bytes(),
        len,
    })
}

fn parse_i32(text: &str) -> Result<i32, &'static str> {
    let bytes = text.as_bytes();
    let canonical = match bytes {
        b"0" => true,
        [b'1'..=b'9', rest @ ..] => rest.iter().all(u8::is_ascii_digit),
        [b'-', b'1'..=b'9', rest @ ..] => rest.iter().all(u8::is_ascii_digit),
        _ => false,
    };
    if !canonical {
        return Err(ERR_NON_CANONICAL_INTEGER_TEXT);
    }
    text.parse().map_err(|_| ERR_NON_CANONICAL_INTEGER_TEXT)
}

fn parse_u32(text: &str) -> Result<u32, &'static str> {
    let bytes = text.as_bytes();
    let canonical = match bytes {
        b"0" => true,
        [b'1'..=b'9', rest @ ..] => rest.iter().all(u8::is_ascii_digit),
        _ => false,
    };
    if !canonical {
        return Err(ERR_NON_CANONICAL_INTEGER_TEXT);
    }
    text.parse().map_err(|_| ERR_NON_CANONICAL_INTEGER_TEXT)
}

fn emit(module: &ParsedModule<'_>) -> Result<WasmModuleBytes, &'static str> {
    let mut output = WasmModuleBytes::new();
    output.extend(&[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00])?;

    output.extend(&[0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f])?;
    output.extend(&[0x03, 0x02, 0x01, 0x00])?;

    if let Some((min, max)) = module.memory {
        output.push(0x05)?;
        output.push_u32_leb(2 + u32_leb_len(min) + u32_leb_len(max))?;
        output.extend(&[0x01, 0x01])?;
        output.push_u32_leb(min)?;
        output.push_u32_leb(max)?;
    }

    let name = module.function_name.as_bytes();
    let export_size = 3usize
        .checked_add(u32_leb_len(name.len() as u32) as usize)
        .and_then(|size| size.checked_add(name.len()))
        .ok_or(ERR_OVERSIZE_OUTPUT)?;
    output.push(0x07)?;
    output.push_u32_leb(export_size as u32)?;
    output.push(0x01)?;
    output.push_u32_leb(name.len() as u32)?;
    output.extend(name)?;
    output.extend(&[0x00, 0x00])?;

    let body_size = 4 + i32_leb_len(module.value);
    let code_size = 1 + u32_leb_len(body_size) + body_size;
    output.push(0x0a)?;
    output.push_u32_leb(code_size)?;
    output.push(0x01)?;
    output.push_u32_leb(body_size)?;
    output.extend(&[0x00, 0x41])?;
    output.push_i32_leb(module.value)?;
    output.extend(&[0x0f, 0x0b])?;

    if module.data_count != 0 {
        let mut data_size = u32_leb_len(module.data_count as u32);
        for segment in module.data[..module.data_count].iter().flatten() {
            data_size = data_size
                .checked_add(3)
                .and_then(|size| size.checked_add(i32_leb_len(segment.offset as i32)))
                .and_then(|size| size.checked_add(u32_leb_len(segment.len as u32)))
                .and_then(|size| size.checked_add(segment.len as u32))
                .ok_or(ERR_OVERSIZE_OUTPUT)?;
        }
        output.push(0x0b)?;
        output.push_u32_leb(data_size)?;
        output.push_u32_leb(module.data_count as u32)?;
        for segment in module.data[..module.data_count].iter().flatten() {
            output.extend(&[0x00, 0x41])?;
            output.push_i32_leb(segment.offset as i32)?;
            output.push(0x0b)?;
            output.push_u32_leb(segment.len as u32)?;
            for pair in segment.hex.chunks_exact(2) {
                output.push((hex_digit(pair[0]) << 4) | hex_digit(pair[1]))?;
            }
        }
    }
    Ok(output)
}

const fn u32_leb_len(mut value: u32) -> u32 {
    let mut len = 1;
    while value >= 0x80 {
        value >>= 7;
        len += 1;
    }
    len
}

const fn i32_leb_len(mut value: i32) -> u32 {
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

const fn hex_digit(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{format, string::String};
    use wasmi::{Engine, Linker, Module, Store};

    #[test]
    fn golden_return_42_is_literal_canonical_wasm_and_runs_in_wasmi() {
        let expected = [
            // Magic and WebAssembly version 1.
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
            // Type section: one () -> i32 function type.
            0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
            // Function section: one function using type index 0.
            0x03, 0x02, 0x01, 0x00,
            // Export section: function 0 as "raios_service_main".
            0x07, 0x16, 0x01, 0x12, 0x72, 0x61, 0x69, 0x6f, 0x73, 0x5f, 0x73, 0x65, 0x72, 0x76,
            0x69, 0x63, 0x65, 0x5f, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            // Code section: no locals; i32.const 42; return; end.
            0x0a, 0x07, 0x01, 0x05, 0x00, 0x41, 0x2a, 0x0f, 0x0b,
        ];
        let bytes = assemble(RETURN_42_IR).unwrap();
        assert_eq!(bytes.as_slice(), expected);

        let engine = Engine::default();
        let module = Module::new(&engine, bytes.as_slice()).unwrap();
        let mut store = Store::new(&engine, ());
        let linker = Linker::<()>::new(&engine);
        let instance = linker
            .instantiate(&mut store, &module)
            .unwrap()
            .start(&mut store)
            .unwrap();
        let function = instance
            .get_typed_func::<(), i32>(&store, "raios_service_main")
            .unwrap();
        assert_eq!(function.call(&mut store, ()).unwrap(), 42);
    }

    #[test]
    fn golden_memory_and_negative_one() {
        let expected = [
            // Magic and WebAssembly version 1.
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
            // Type section: payload size 5; one () -> i32 function type.
            0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
            // Function section: payload size 2; function uses type index 0.
            0x03, 0x02, 0x01, 0x00, // Memory section: one memory, min 1, max 4.
            0x05, 0x04, 0x01, 0x01, 0x01, 0x04, // Export section: function 0 as "f".
            0x07, 0x05, 0x01, 0x01, 0x66, 0x00, 0x00,
            // Code section: i32.const -1 is canonical signed LEB128 0x7f.
            0x0a, 0x07, 0x01, 0x05, 0x00, 0x41, 0x7f, 0x0f, 0x0b,
        ];
        let source = b"RAIOS_WASM_IR_V1\nfunc f -> i32\nconst.i32 -1\nreturn\nend\nmemory 1 4\n";
        let bytes = assemble(source).unwrap();
        assert_eq!(bytes.as_slice(), expected);
        assert!(Module::new(&Engine::default(), bytes.as_slice()).is_ok());
    }

    #[test]
    fn golden_memory_data_and_two_byte_signed_leb() {
        let expected = [
            // Magic and WebAssembly version 1.
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
            // Type section: payload size 5; one () -> i32 function type.
            0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
            // Function section: payload size 2; function uses type index 0.
            0x03, 0x02, 0x01, 0x00, // Memory section: one memory, min 1, max 1.
            0x05, 0x04, 0x01, 0x01, 0x01, 0x01, // Export section: function 0 as "main".
            0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            // Code section: i32.const 64 is canonical signed LEB128 c0 00.
            0x0a, 0x08, 0x01, 0x06, 0x00, 0x41, 0xc0, 0x00, 0x0f, 0x0b,
            // Data section: active memory-0 segment at offset 0 containing "Hi".
            0x0b, 0x08, 0x01, 0x00, 0x41, 0x00, 0x0b, 0x02, 0x48, 0x69,
        ];
        let source = b"RAIOS_WASM_IR_V1\nfunc main -> i32\nconst.i32 64\nreturn\nend\nmemory 1 1\ndata 0 4869\n";
        let bytes = assemble(source).unwrap();
        assert_eq!(bytes.as_slice(), expected);
        assert!(Module::new(&Engine::default(), bytes.as_slice()).is_ok());
    }

    #[test]
    fn assembly_is_deterministic() {
        assert_eq!(
            assemble(RETURN_42_IR).unwrap(),
            assemble(RETURN_42_IR).unwrap()
        );
    }

    #[test]
    fn every_static_failure_reason_is_exercised() {
        let cases: &[(&[u8], &str)] = &[
            (b"RAIOS_WASM_IR_V0\n", ERR_WRONG_VERSION_LINE),
            (
                b"RAIOS_WASM_IR_V1\nfunc f -> i32\nconst.i32 +1\nreturn\nend\n",
                ERR_NON_CANONICAL_INTEGER_TEXT,
            ),
            (
                b"RAIOS_WASM_IR_V1\nfunc f -> i32\nconst.i32 1\nreturn\nend\nfunc g -> i32\n",
                ERR_DUPLICATE_SECTION,
            ),
            (
                b"RAIOS_WASM_IR_V1\nfunc f -> i32\nnop\nreturn\nend\n",
                ERR_UNKNOWN_DIRECTIVE,
            ),
            (
                b"RAIOS_WASM_IR_V1\nfunc f -> i32\nconst.i32 1\nreturn\n",
                ERR_MISSING_TERMINATOR,
            ),
            (
                b"RAIOS_WASM_IR_V1\nfunc f -> i32\nconst.i32 1\nreturn\nend\ntrailing\n",
                ERR_TRAILING_CONTENT,
            ),
            (
                b"RAIOS_WASM_IR_V1\nfunc Bad -> i32\nconst.i32 1\nreturn\nend\n",
                ERR_INVALID_FUNCTION_NAME,
            ),
            (
                b"RAIOS_WASM_IR_V1\nfunc f -> i32\nconst.i32 1\nreturn\nend\nmemory 1 1\ndata 0 0A\n",
                ERR_NON_CANONICAL_HEX_TEXT,
            ),
            (
                b"RAIOS_WASM_IR_V1\nfunc f -> i32\nconst.i32 1\nreturn\nend\nmemory 1 5\n",
                ERR_QUOTA_EXCEEDED,
            ),
        ];
        for (source, reason) in cases {
            assert_eq!(assemble(source), Err(*reason), "{reason}");
        }

        let mut non_ascii = RETURN_42_IR.to_vec();
        non_ascii[20] = 0xff;
        assert_eq!(assemble(&non_ascii), Err(ERR_NON_ASCII_SOURCE));

        let oversized_source = [b'a'; MAX_IR_SOURCE_BYTES + 1];
        assert_eq!(assemble(&oversized_source), Err(ERR_OVERSIZE_SOURCE));

        let oversized_data = source_with_data(MAX_DATA_BYTES + 1);
        assert!(oversized_data.len() <= MAX_IR_SOURCE_BYTES);
        assert_eq!(
            assemble(oversized_data.as_bytes()),
            Err(ERR_OVERSIZE_OUTPUT)
        );
    }

    #[test]
    fn decimal_and_leb128_encodings_are_canonical_at_boundaries() {
        for text in ["00", "01", "-0", "+0", "2147483648", "-2147483649"] {
            assert_eq!(parse_i32(text), Err(ERR_NON_CANONICAL_INTEGER_TEXT));
        }
        for (value, instruction) in [
            (63, &[0x41, 0x3f, 0x0f, 0x0b][..]),
            (64, &[0x41, 0xc0, 0x00, 0x0f, 0x0b][..]),
            (8191, &[0x41, 0xff, 0x3f, 0x0f, 0x0b][..]),
            (8192, &[0x41, 0x80, 0xc0, 0x00, 0x0f, 0x0b][..]),
        ] {
            let source =
                format!("RAIOS_WASM_IR_V1\nfunc f -> i32\nconst.i32 {value}\nreturn\nend\n");
            assert!(assemble(source.as_bytes())
                .unwrap()
                .as_slice()
                .windows(instruction.len())
                .any(|window| window == instruction));
        }

        let size_127 = assemble(source_with_data(121).as_bytes()).unwrap();
        let start_127 = size_127.len() - 129;
        assert_eq!(
            &size_127.as_slice()[start_127..start_127 + 2],
            &[0x0b, 0x7f]
        );

        let size_128 = assemble(source_with_data(122).as_bytes()).unwrap();
        let start_128 = size_128.len() - 131;
        assert_eq!(
            &size_128.as_slice()[start_128..start_128 + 3],
            &[0x0b, 0x80, 0x01]
        );
    }

    #[test]
    fn source_bound_accepts_exact_cap_and_rejects_one_byte_more() {
        let source = format!(
            "RAIOS_WASM_IR_V1\nfunc abcdefghijklmnopqrstuvwx -> i32\nconst.i32 0\nreturn\nend\nmemory 1 4\ndata 0 {}\n",
            "00".repeat(MAX_DATA_BYTES)
        );
        assert_eq!(source.len(), MAX_IR_SOURCE_BYTES);
        assert!(assemble(source.as_bytes()).is_ok());

        let mut oversized = source.into_bytes();
        oversized.push(b'x');
        assert_eq!(assemble(&oversized), Err(ERR_OVERSIZE_SOURCE));
    }

    #[test]
    fn actual_output_buffer_overflow_fails_closed() {
        let mut output = WasmModuleBytes::new();
        assert_eq!(
            output.extend(&[0; MAX_WASM_OUTPUT_BYTES + 1]),
            Err(ERR_OVERSIZE_OUTPUT)
        );
    }

    fn source_with_data(len: usize) -> String {
        format!(
            "RAIOS_WASM_IR_V1\nfunc f -> i32\nconst.i32 0\nreturn\nend\nmemory 1 4\ndata 0 {}\n",
            "00".repeat(len)
        )
    }
}
