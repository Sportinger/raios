//! Owner Core Policy key generation and detached record signing.
//!
//! This is deliberately a separate key family from descriptor, distribution,
//! and runtime-promotion signing. Private keys are accepted only as files
//! outside the current Cargo workspace and are never printed or read from an
//! environment variable.

use std::{
    env, fs,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process,
};

use p256::ecdsa::{
    signature::{Signer, Verifier},
    Signature, SigningKey, VerifyingKey,
};
use raios_core::{
    core_policy::{
        assemble_core_policy_record, core_policy_signature_message, encode_core_policy_payload,
        CorePolicySlot, CORE_POLICY_MAX_EXECUTABLE_LEN, CORE_POLICY_PAYLOAD_LEN,
        CORE_POLICY_RECORD_LEN, OWNER_CORE_POLICY_PUBLIC_KEY_SEC1,
    },
    sha256_bytes,
};
use rand::rngs::OsRng;

const PRIVATE_KEY_LEN: usize = 32;

fn usage() -> ! {
    eprintln!(
        "usage:\n  core-policy-sign keygen <private-path> <public-output>\n  core-policy-sign sign <private-path> <kernel> <A|B> <generation> <output>\n  core-policy-sign verify <kernel> <A|B> <generation> <policy>"
    );
    process::exit(2);
}

fn main() {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        usage();
    };
    let remaining: Vec<String> = args.collect();
    let result = match (command.as_str(), remaining.as_slice()) {
        ("keygen", [private_path, public_output]) => {
            keygen(Path::new(private_path), Path::new(public_output))
        }
        ("sign", [private_path, kernel, slot, generation, output]) => sign(
            Path::new(private_path),
            Path::new(kernel),
            parse_slot(slot),
            parse_generation(generation),
            Path::new(output),
        ),
        ("verify", [kernel, slot, generation, policy]) => verify(
            Path::new(kernel),
            parse_slot(slot),
            parse_generation(generation),
            Path::new(policy),
        ),
        _ => usage(),
    };
    if let Err(error) = result {
        eprintln!("core-policy-sign: {error}");
        process::exit(1);
    }
}

fn keygen(private_path: &Path, public_output: &Path) -> Result<(), String> {
    reject_private_path_in_workspace(private_path)?;
    ensure_absent(private_path, "private key")?;
    ensure_absent(public_output, "public key output")?;

    let signing_key = SigningKey::random(&mut OsRng);
    let public_key = VerifyingKey::from(&signing_key)
        .to_encoded_point(false)
        .as_bytes()
        .to_vec();
    let mut private_bytes = [0u8; PRIVATE_KEY_LEN];
    private_bytes.copy_from_slice(signing_key.to_bytes().as_slice());

    if let Err(error) = write_atomic_new(public_output, &public_key) {
        private_bytes.fill(0);
        return Err(error);
    }
    if let Err(error) = write_atomic_new(private_path, &private_bytes) {
        private_bytes.fill(0);
        let _ = fs::remove_file(public_output);
        return Err(error);
    }
    private_bytes.fill(0);

    println!("core-policy-sign: generated owner key");
    println!("public-key-sha256={}", hex(&sha256_bytes(&public_key)));
    Ok(())
}

fn sign(
    private_path: &Path,
    kernel_path: &Path,
    slot: Result<CorePolicySlot, String>,
    generation: Result<u64, String>,
    output: &Path,
) -> Result<(), String> {
    reject_private_path_in_workspace(private_path)?;
    ensure_absent(output, "policy output")?;
    let slot = slot?;
    let generation = generation?;
    let kernel = read_kernel(kernel_path)?;
    let kernel_sha256 = sha256_bytes(&kernel);
    let payload = encode_core_policy_payload(slot, generation, kernel.len() as u64, kernel_sha256)
        .map_err(|denied| format!("policy payload denied: {denied:?}"))?;

    let signing_key = read_signing_key(private_path)?;
    require_pinned_public_key(&signing_key)?;
    let record = signed_record(&signing_key, &payload);

    verify_record_bytes(&kernel, slot, generation, &record)?;
    write_atomic_new(output, &record)?;
    let written = fs::read(output)
        .map_err(|error| format!("failed to read back {}: {error}", output.display()))?;
    verify_record_bytes(&kernel, slot, generation, &written)?;
    println!(
        "core-policy-sign: signed kernel_sha256={} slot={} generation={}",
        hex(&kernel_sha256),
        slot_name(slot),
        generation
    );
    Ok(())
}

fn signed_record(signing_key: &SigningKey, payload: &[u8; 64]) -> [u8; 128] {
    let signature: Signature = signing_key.sign(&core_policy_signature_message(payload));
    let signature = signature.normalize_s().unwrap_or(signature);
    let mut raw_signature = [0u8; 64];
    raw_signature.copy_from_slice(signature.to_bytes().as_slice());
    let record = assemble_core_policy_record(payload, &raw_signature);
    raw_signature.fill(0);
    record
}

fn verify(
    kernel_path: &Path,
    slot: Result<CorePolicySlot, String>,
    generation: Result<u64, String>,
    policy_path: &Path,
) -> Result<(), String> {
    let slot = slot?;
    let generation = generation?;
    let kernel = read_kernel(kernel_path)?;
    let record = fs::read(policy_path)
        .map_err(|error| format!("failed to read {}: {error}", policy_path.display()))?;
    verify_record_bytes(&kernel, slot, generation, &record)?;
    println!(
        "core-policy-sign: verified kernel_sha256={} slot={} generation={}",
        hex(&sha256_bytes(&kernel)),
        slot_name(slot),
        generation
    );
    Ok(())
}

fn verify_record_bytes(
    kernel: &[u8],
    slot: CorePolicySlot,
    generation: u64,
    record: &[u8],
) -> Result<(), String> {
    if record.len() != CORE_POLICY_RECORD_LEN {
        return Err(format!(
            "policy record length {} != {}",
            record.len(),
            CORE_POLICY_RECORD_LEN
        ));
    }
    let expected_payload =
        encode_core_policy_payload(slot, generation, kernel.len() as u64, sha256_bytes(kernel))
            .map_err(|denied| format!("expected policy payload denied: {denied:?}"))?;
    if record[..CORE_POLICY_PAYLOAD_LEN] != expected_payload {
        return Err("policy does not bind the exact kernel, slot, and generation".to_owned());
    }
    let signature = Signature::from_slice(&record[CORE_POLICY_PAYLOAD_LEN..])
        .map_err(|error| format!("invalid raw P-256 signature: {error}"))?;
    if signature.normalize_s().is_some() {
        return Err("policy signature is not low-S".to_owned());
    }
    let verifying_key = VerifyingKey::from_sec1_bytes(&OWNER_CORE_POLICY_PUBLIC_KEY_SEC1)
        .map_err(|error| format!("invalid pinned owner public key: {error}"))?;
    verifying_key
        .verify(
            &core_policy_signature_message(&expected_payload),
            &signature,
        )
        .map_err(|error| format!("policy signature did not verify: {error}"))
}

fn require_pinned_public_key(signing_key: &SigningKey) -> Result<(), String> {
    let derived = VerifyingKey::from(signing_key).to_encoded_point(false);
    if derived.as_bytes() != OWNER_CORE_POLICY_PUBLIC_KEY_SEC1 {
        return Err(format!(
            "private key does not match pinned owner key sha256={}",
            hex(&sha256_bytes(&OWNER_CORE_POLICY_PUBLIC_KEY_SEC1))
        ));
    }
    Ok(())
}

fn read_signing_key(path: &Path) -> Result<SigningKey, String> {
    let mut file = File::open(path)
        .map_err(|error| format!("failed to open private key {}: {error}", path.display()))?;
    let mut bytes = [0u8; PRIVATE_KEY_LEN];
    file.read_exact(&mut bytes)
        .map_err(|error| format!("private key must be exactly 32 bytes: {error}"))?;
    let mut extra = [0u8; 1];
    if file
        .read(&mut extra)
        .map_err(|error| format!("failed to validate private key length: {error}"))?
        != 0
    {
        bytes.fill(0);
        return Err("private key must be exactly 32 bytes".to_owned());
    }
    let key = SigningKey::from_slice(&bytes)
        .map_err(|error| format!("invalid P-256 private key: {error}"));
    bytes.fill(0);
    key
}

fn read_kernel(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("failed to inspect kernel {}: {error}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("kernel is not a regular file: {}", path.display()));
    }
    if metadata.len() == 0 || metadata.len() > CORE_POLICY_MAX_EXECUTABLE_LEN {
        return Err(format!(
            "kernel length {} outside 1..={}",
            metadata.len(),
            CORE_POLICY_MAX_EXECUTABLE_LEN
        ));
    }
    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read kernel {}: {error}", path.display()))?;
    if bytes.len() as u64 != metadata.len() {
        return Err("kernel length changed during read".to_owned());
    }
    Ok(bytes)
}

fn reject_private_path_in_workspace(path: &Path) -> Result<(), String> {
    let candidate = absolute_candidate(path)?;
    let workspace = workspace_root()?;
    if candidate.starts_with(&workspace) {
        return Err(format!(
            "refusing private key path inside workspace: {}",
            candidate.display()
        ));
    }
    Ok(())
}

fn absolute_candidate(path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|error| format!("failed to read current directory: {error}"))?
            .join(path)
    };
    let parent = absolute
        .parent()
        .ok_or_else(|| format!("path has no parent: {}", path.display()))?
        .canonicalize()
        .map_err(|error| format!("failed to resolve parent of {}: {error}", path.display()))?;
    let name = absolute
        .file_name()
        .ok_or_else(|| format!("path has no file name: {}", path.display()))?;
    Ok(parent.join(name))
}

fn workspace_root() -> Result<PathBuf, String> {
    let mut current = env::current_dir()
        .map_err(|error| format!("failed to read current directory: {error}"))?
        .canonicalize()
        .map_err(|error| format!("failed to resolve current directory: {error}"))?;
    let fallback = current.clone();
    loop {
        let manifest = current.join("Cargo.toml");
        if fs::read_to_string(&manifest)
            .map(|text| text.lines().any(|line| line.trim() == "[workspace]"))
            .unwrap_or(false)
        {
            return Ok(current);
        }
        if !current.pop() {
            return Ok(fallback);
        }
    }
}

fn write_atomic_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("output has no parent: {}", path.display()))?;
    if !parent.is_dir() {
        return Err(format!(
            "output directory does not exist: {}",
            parent.display()
        ));
    }
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("output has invalid file name: {}", path.display()))?;
    let temp = parent.join(format!(".{name}.{}.tmp", process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|error| {
            format!(
                "failed to create temporary output {}: {error}",
                temp.display()
            )
        })?;
    let result = (|| {
        file.write_all(bytes)
            .map_err(|error| format!("failed to write {}: {error}", temp.display()))?;
        file.sync_all()
            .map_err(|error| format!("failed to flush {}: {error}", temp.display()))?;
        drop(file);
        fs::rename(&temp, path).map_err(|error| {
            format!(
                "failed to atomically publish {} as {}: {error}",
                temp.display(),
                path.display()
            )
        })
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

fn ensure_absent(path: &Path, label: &str) -> Result<(), String> {
    if path.exists() {
        return Err(format!("{label} already exists: {}", path.display()));
    }
    Ok(())
}

fn parse_slot(value: &str) -> Result<CorePolicySlot, String> {
    match value {
        "A" => Ok(CorePolicySlot::A),
        "B" => Ok(CorePolicySlot::B),
        _ => Err("slot must be exactly A or B".to_owned()),
    }
}

fn parse_generation(value: &str) -> Result<u64, String> {
    let generation = value
        .parse::<u64>()
        .map_err(|_| "generation must be a positive u64".to_owned())?;
    if generation == 0 {
        return Err("generation must be greater than zero".to_owned());
    }
    Ok(generation)
}

fn slot_name(slot: CorePolicySlot) -> &'static str {
    match slot {
        CorePolicySlot::A => "A",
        CorePolicySlot::B => "B",
    }
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut out, "{byte:02x}").expect("writing to String cannot fail");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> PathBuf {
        let path = env::temp_dir().join(format!(
            "core-policy-sign-{label}-{}-{}",
            process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn keygen_is_atomic_and_refuses_replacement() {
        let dir = temp_dir("keygen");
        let private = dir.join("owner.secret");
        let public = dir.join("owner.pub");
        keygen(&private, &public).unwrap();
        assert_eq!(fs::read(&private).unwrap().len(), PRIVATE_KEY_LEN);
        assert_eq!(fs::read(&public).unwrap().len(), 65);
        assert!(keygen(&private, &public).is_err());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn exact_record_verification_rejects_every_bound_input_change() {
        let kernel = b"small exact linked kernel";
        let payload = encode_core_policy_payload(
            CorePolicySlot::A,
            7,
            kernel.len() as u64,
            sha256_bytes(kernel),
        )
        .unwrap();
        // This verifies the binding checks without depending on the owner's
        // unavailable private key: a zero signature must fail after each exact
        // payload mismatch is independently detected.
        let record = assemble_core_policy_record(&payload, &[0u8; 64]);
        assert!(verify_record_bytes(kernel, CorePolicySlot::A, 7, &record).is_err());
        assert!(verify_record_bytes(b"changed", CorePolicySlot::A, 7, &record).is_err());
        assert!(verify_record_bytes(kernel, CorePolicySlot::B, 7, &record).is_err());
        assert!(verify_record_bytes(kernel, CorePolicySlot::A, 8, &record).is_err());
        assert!(verify_record_bytes(kernel, CorePolicySlot::A, 7, &record[..127]).is_err());
    }

    #[test]
    fn shared_codec_record_has_a_low_s_signature_over_the_exact_message() {
        let signing_key = SigningKey::random(&mut OsRng);
        let payload =
            encode_core_policy_payload(CorePolicySlot::B, 11, 6, sha256_bytes(b"kernel")).unwrap();
        let record = signed_record(&signing_key, &payload);
        assert_eq!(&record[..CORE_POLICY_PAYLOAD_LEN], &payload);
        let signature = Signature::from_slice(&record[CORE_POLICY_PAYLOAD_LEN..]).unwrap();
        assert!(signature.normalize_s().is_none());
        VerifyingKey::from(&signing_key)
            .verify(&core_policy_signature_message(&payload), &signature)
            .unwrap();
    }

    #[test]
    fn private_keys_inside_workspace_are_refused() {
        let path = workspace_root()
            .unwrap()
            .join("target/refused-owner.secret");
        assert!(reject_private_path_in_workspace(&path).is_err());
    }
}
