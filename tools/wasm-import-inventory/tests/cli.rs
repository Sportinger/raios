use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use wasm_import_inventory::inventory_from_bytes;

fn temporary_directory(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "wasm-import-inventory-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&path).expect("temporary directory must be created");
    path
}

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wasm-import-inventory"))
}

#[test]
fn valid_wasm_with_matching_hash_writes_canonical_output_file() {
    let directory = temporary_directory("success");
    let input = directory.join("valid.wasm");
    let output = directory.join("inventory.json");
    let wasm = wat::parse_str(r#"(module (import "m" "f" (func (param i32))))"#)
        .expect("fixture WAT must compile");
    let inventory = inventory_from_bytes(&wasm).expect("fixture must be inventoried");
    fs::write(&input, wasm).expect("fixture must be written");

    let stdout_result = command().arg(&input).output().expect("CLI must run");
    assert!(stdout_result.status.success());
    assert_eq!(stdout_result.stdout, inventory.to_canonical_json());
    assert!(stdout_result.stderr.is_empty());

    let result = command()
        .arg(&input)
        .arg("--expect-sha256")
        .arg(inventory.file_sha256().to_ascii_uppercase())
        .arg("--output")
        .arg(&output)
        .output()
        .expect("CLI must run");

    assert!(result.status.success());
    assert!(result.stdout.is_empty());
    assert!(result.stderr.is_empty());
    assert_eq!(
        fs::read(&output).expect("inventory output must exist"),
        inventory.to_canonical_json()
    );
    fs::remove_dir_all(directory).expect("temporary directory must be removed");
}

#[test]
fn missing_input_exits_with_error_without_creating_output() {
    let directory = temporary_directory("missing");
    let input = directory.join("missing.wasm");
    let output = directory.join("inventory.json");

    let result = command()
        .arg(&input)
        .arg("--output")
        .arg(&output)
        .output()
        .expect("CLI must run");

    assert!(!result.status.success());
    assert!(!output.exists());
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("cannot read"));
    fs::remove_dir_all(directory).expect("temporary directory must be removed");
}

#[test]
fn truncated_wasm_exits_with_error_without_creating_output() {
    let directory = temporary_directory("truncated");
    let input = directory.join("truncated.wasm");
    let output = directory.join("inventory.json");
    let mut wasm =
        wat::parse_str(r#"(module (import "m" "f" (func)))"#).expect("fixture WAT must compile");
    wasm.pop();
    fs::write(&input, wasm).expect("fixture must be written");

    let result = command()
        .arg(&input)
        .arg("--output")
        .arg(&output)
        .output()
        .expect("CLI must run");

    assert!(!result.status.success());
    assert!(!output.exists());
    assert!(result.stdout.is_empty());
    fs::remove_dir_all(directory).expect("temporary directory must be removed");
}

#[test]
fn wrong_expected_sha256_exits_with_error() {
    let directory = temporary_directory("sha-mismatch");
    let input = directory.join("valid.wasm");
    let output = directory.join("inventory.json");
    let wasm = wat::parse_str("(module)").expect("fixture WAT must compile");
    fs::write(&input, wasm).expect("fixture must be written");

    let result = command()
        .arg(&input)
        .arg("--expect-sha256")
        .arg("0000000000000000000000000000000000000000000000000000000000000000")
        .arg("--output")
        .arg(&output)
        .output()
        .expect("CLI must run");

    assert!(!result.status.success());
    assert!(!output.exists());
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("SHA-256 mismatch"));
    fs::remove_dir_all(directory).expect("temporary directory must be removed");
}
