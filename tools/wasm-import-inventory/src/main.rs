use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use wasm_import_inventory::inventory_from_bytes;

const USAGE: &str =
    "usage: wasm-import-inventory <input.wasm> [--expect-sha256 <hex>] [--output <file>]";

struct Arguments {
    input: PathBuf,
    expected_sha256: Option<String>,
    output: Option<PathBuf>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("wasm-import-inventory: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let arguments = parse_arguments(env::args_os().skip(1))?;
    let bytes = fs::read(&arguments.input)
        .map_err(|error| format!("cannot read {}: {error}", arguments.input.display()))?;
    let inventory = inventory_from_bytes(&bytes).map_err(|error| error.to_string())?;

    if let Some(expected) = arguments.expected_sha256 {
        if inventory.file_sha256() != expected {
            return Err(format!(
                "SHA-256 mismatch: expected {expected}, got {}",
                inventory.file_sha256()
            ));
        }
    }

    let json = inventory.to_canonical_json();
    match arguments.output {
        Some(path) => fs::write(&path, json)
            .map_err(|error| format!("cannot write {}: {error}", path.display())),
        None => {
            let mut stdout = io::stdout().lock();
            stdout
                .write_all(&json)
                .and_then(|()| stdout.flush())
                .map_err(|error| format!("cannot write stdout: {error}"))
        }
    }
}

fn parse_arguments(arguments: impl IntoIterator<Item = OsString>) -> Result<Arguments, String> {
    let mut arguments = arguments.into_iter();
    let input = arguments
        .next()
        .filter(|argument| argument != "--expect-sha256" && argument != "--output")
        .map(PathBuf::from)
        .ok_or_else(|| USAGE.to_owned())?;
    let mut expected_sha256 = None;
    let mut output = None;

    while let Some(argument) = arguments.next() {
        if argument == "--expect-sha256" {
            if expected_sha256.is_some() {
                return Err("--expect-sha256 may be specified only once".to_owned());
            }
            let value = arguments
                .next()
                .ok_or_else(|| "--expect-sha256 requires a value".to_owned())?;
            let value = value
                .into_string()
                .map_err(|_| "--expect-sha256 must be valid UTF-8".to_owned())?;
            if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err("--expect-sha256 must be exactly 64 hexadecimal digits".to_owned());
            }
            expected_sha256 = Some(value.to_ascii_lowercase());
        } else if argument == "--output" {
            if output.is_some() {
                return Err("--output may be specified only once".to_owned());
            }
            output = Some(PathBuf::from(
                arguments
                    .next()
                    .ok_or_else(|| "--output requires a value".to_owned())?,
            ));
        } else {
            return Err(format!("unknown argument {:?}; {USAGE}", argument));
        }
    }

    Ok(Arguments {
        input,
        expected_sha256,
        output,
    })
}
