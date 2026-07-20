use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn temporary_directory(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "buildfs-pack-cli-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&path).expect("temporary directory must be created");
    path
}

fn command(input: &PathBuf, output: &PathBuf) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_buildfs-pack"));
    command.arg(input).arg("--output").arg(output);
    command
}

fn run_with_expected(input: &PathBuf, output: &PathBuf, expected: &str) -> Output {
    command(input, output)
        .arg("--expect-manifest-sha256")
        .arg(expected)
        .output()
        .expect("CLI must run")
}

#[test]
fn cli_writes_package_and_accepts_its_expected_manifest_hash() {
    let directory = temporary_directory("success");
    let input = directory.join("input");
    let first = directory.join("first");
    let second = directory.join("second");
    fs::create_dir(&input).expect("input directory");
    fs::write(input.join("file"), b"content").expect("fixture file");

    let first_result = command(&input, &first).output().expect("CLI must run");
    assert!(first_result.status.success());
    assert!(first_result.stdout.is_empty());
    assert!(first_result.stderr.is_empty());
    let manifest_hash =
        fs::read_to_string(first.join("manifest.sha256")).expect("manifest hash must be readable");
    assert_eq!(manifest_hash.len(), 64);

    let second_result = run_with_expected(&input, &second, &manifest_hash);
    assert!(second_result.status.success());
    assert_eq!(
        fs::read(first.join("manifest.bin")).expect("first manifest"),
        fs::read(second.join("manifest.bin")).expect("second manifest")
    );

    fs::remove_dir_all(directory).expect("temporary directory cleanup");
}

#[test]
fn expected_manifest_mismatch_exits_with_error_and_removes_output() {
    let directory = temporary_directory("mismatch");
    let input = directory.join("input");
    let output = directory.join("output");
    fs::create_dir(&input).expect("input directory");
    fs::write(input.join("file"), b"content").expect("fixture file");

    let result = run_with_expected(
        &input,
        &output,
        "0000000000000000000000000000000000000000000000000000000000000000",
    );
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("manifest SHA-256 mismatch"));
    assert!(!output.exists());

    fs::remove_dir_all(directory).expect("temporary directory cleanup");
}
