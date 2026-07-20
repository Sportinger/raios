use buildfs_pack::pack_directory;
use raios_core::buildfs_manifest::BUILD_FS_CHUNK_SIZE;
use raios_core::{sha256_bytes, sha256_hex};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn temporary_directory(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "buildfs-pack-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&path).expect("temporary directory must be created");
    path
}

fn hex(digest: &[u8; 32]) -> String {
    String::from_utf8(sha256_hex(digest).to_vec()).expect("digest hex is ASCII")
}

fn chunk_file_names(output: &Path) -> Vec<String> {
    let mut names = fs::read_dir(output.join("chunks"))
        .expect("chunk directory must be readable")
        .map(|entry| {
            entry
                .expect("chunk entry must be readable")
                .file_name()
                .into_string()
                .expect("chunk name must be UTF-8")
        })
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn create_fixture(input: &Path) {
    fs::create_dir(input.join("alpha")).expect("alpha directory");
    fs::create_dir_all(input.join("nested/empty")).expect("nested empty directory");
    fs::write(input.join("empty-file"), []).expect("empty file");
    fs::write(
        input.join("exact.bin"),
        vec![0x5a; BUILD_FS_CHUNK_SIZE as usize],
    )
    .expect("exact chunk file");
    let over = (0..BUILD_FS_CHUNK_SIZE as usize + 37)
        .map(|index| ((index * 37 + 11) % 251) as u8)
        .collect::<Vec<_>>();
    fs::write(input.join("nested/over.bin"), over).expect("over-boundary file");
    fs::write(input.join("alpha/same-a"), b"deduplicated chunk").expect("duplicate A");
    fs::write(input.join("alpha/same-b"), b"deduplicated chunk").expect("duplicate B");
}

#[test]
fn fixture_manifest_and_every_chunk_validate_and_deduplicate() {
    let directory = temporary_directory("fixture");
    let input = directory.join("input");
    let output = directory.join("output");
    fs::create_dir(&input).expect("input directory");
    create_fixture(&input);

    let packed = pack_directory(&input, &output, None).expect("fixture must pack");
    packed
        .manifest()
        .validate()
        .expect("raios-core must validate packed manifest");
    assert_eq!(
        fs::read(output.join("manifest.bin")).expect("manifest bytes"),
        packed
            .manifest()
            .canonical_bytes()
            .expect("canonical bytes")
    );
    assert_eq!(
        fs::read(output.join("manifest.sha256")).expect("manifest hash"),
        sha256_hex(&packed.manifest_sha256())
    );
    let directory_paths = packed
        .manifest()
        .directories
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(directory_paths, ["alpha", "nested", "nested/empty"]);
    let file = |path: &str| {
        packed
            .manifest()
            .files
            .iter()
            .find(|file| file.path == path)
            .expect("fixture file must be in manifest")
    };
    assert!(file("empty-file").chunks.is_empty());
    assert_eq!(file("exact.bin").chunks.len(), 1);
    assert_eq!(file("exact.bin").chunks[0].len, BUILD_FS_CHUNK_SIZE);
    assert_eq!(file("nested/over.bin").chunks.len(), 2);
    assert_eq!(file("nested/over.bin").chunks[1].len, 37);

    let mut referenced_chunks = BTreeSet::new();
    for file in &packed.manifest().files {
        let source = fs::read(input.join(file.path.replace('/', std::path::MAIN_SEPARATOR_STR)))
            .expect("source file must be readable");
        assert_eq!(file.len, source.len() as u64);
        assert_eq!(file.sha256, sha256_bytes(&source));
        let mut reconstructed = Vec::new();
        for chunk in &file.chunks {
            let name = hex(&chunk.sha256);
            referenced_chunks.insert(name.clone());
            let bytes = fs::read(output.join("chunks").join(&name)).expect("chunk must exist");
            assert_eq!(bytes.len() as u64, chunk.len);
            assert_eq!(sha256_bytes(&bytes), chunk.sha256);
            reconstructed.extend_from_slice(&bytes);
        }
        assert_eq!(reconstructed, source);
    }

    let same_a = file("alpha/same-a");
    let same_b = file("alpha/same-b");
    assert_eq!(same_a.chunks, same_b.chunks);
    assert_eq!(chunk_file_names(&output).len(), referenced_chunks.len());

    fs::remove_dir_all(directory).expect("temporary directory cleanup");
}

#[test]
fn two_runs_produce_identical_manifest_and_chunk_file_list() {
    let directory = temporary_directory("determinism");
    let input = directory.join("input");
    let first = directory.join("first");
    let second = directory.join("second");
    fs::create_dir(&input).expect("input directory");
    create_fixture(&input);

    pack_directory(&input, &first, None).expect("first pack");
    pack_directory(&input, &second, None).expect("second pack");
    assert_eq!(
        fs::read(first.join("manifest.bin")).expect("first manifest"),
        fs::read(second.join("manifest.bin")).expect("second manifest")
    );
    assert_eq!(chunk_file_names(&first), chunk_file_names(&second));

    fs::remove_dir_all(directory).expect("temporary directory cleanup");
}

#[test]
fn nonempty_output_is_rejected_without_touching_it() {
    let directory = temporary_directory("nonempty-output");
    let input = directory.join("input");
    let output = directory.join("output");
    fs::create_dir(&input).expect("input directory");
    fs::create_dir(&output).expect("output directory");
    fs::write(output.join("keep"), b"untouched").expect("sentinel");

    let error = pack_directory(&input, &output, None).expect_err("nonempty output must fail");
    assert!(error.to_string().contains("not empty"));
    assert_eq!(
        fs::read(output.join("keep")).expect("sentinel remains"),
        b"untouched"
    );

    fs::remove_dir_all(directory).expect("temporary directory cleanup");
}

#[cfg(unix)]
#[test]
fn symbolic_link_and_non_utf8_name_are_rejected_and_output_is_removed() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::symlink;

    let directory = temporary_directory("unix-negatives");
    let input = directory.join("input");
    fs::create_dir(&input).expect("input directory");
    fs::write(input.join("target"), b"target").expect("target file");
    symlink(input.join("target"), input.join("link")).expect("fixture symlink");
    let output = directory.join("link-output");
    let error = pack_directory(&input, &output, None).expect_err("symlink must fail");
    assert!(error.to_string().contains("symbolic links"));
    assert!(!output.exists());

    fs::remove_file(input.join("link")).expect("remove fixture symlink");
    fs::write(
        input.join(OsString::from_vec(vec![b'b', b'a', b'd', 0xff])),
        b"bad",
    )
    .expect("non-UTF-8 fixture");
    let output = directory.join("utf8-output");
    let error = pack_directory(&input, &output, None).expect_err("non-UTF-8 must fail");
    assert!(error.to_string().contains("non-UTF-8"));
    assert!(!output.exists());

    fs::remove_dir_all(directory).expect("temporary directory cleanup");
}

#[cfg(windows)]
#[test]
fn symbolic_link_is_rejected_when_windows_allows_fixture_creation() {
    use std::os::windows::fs::symlink_file;

    let directory = temporary_directory("windows-link");
    let input = directory.join("input");
    fs::create_dir(&input).expect("input directory");
    fs::write(input.join("target"), b"target").expect("target file");
    if let Err(error) = symlink_file(input.join("target"), input.join("link")) {
        eprintln!(
            "SKIP: Windows denied symlink fixture creation ({error}); production code rejects all reparse points, including junctions"
        );
        fs::remove_dir_all(directory).expect("temporary directory cleanup");
        return;
    }
    let output = directory.join("output");
    let error = pack_directory(&input, &output, None).expect_err("symlink must fail");
    assert!(error.to_string().contains("symbolic links and junctions"));
    assert!(!output.exists());
    fs::remove_dir_all(directory).expect("temporary directory cleanup");
}

#[cfg(windows)]
#[test]
fn non_utf8_name_is_rejected_when_windows_allows_fixture_creation() {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    let directory = temporary_directory("windows-non-utf8");
    let input = directory.join("input");
    fs::create_dir(&input).expect("input directory");
    let invalid_name = OsString::from_wide(&[b'b' as u16, b'a' as u16, b'd' as u16, 0xd800]);
    if let Err(error) = fs::write(input.join(invalid_name), b"bad") {
        eprintln!(
            "SKIP: Windows filesystem denied the non-UTF-8 fixture name ({error}); rejection is covered on Unix"
        );
        fs::remove_dir_all(directory).expect("temporary directory cleanup");
        return;
    }

    let output = directory.join("output");
    let error = pack_directory(&input, &output, None).expect_err("non-UTF-8 name must fail");
    assert!(error.to_string().contains("non-UTF-8"));
    assert!(!output.exists());
    fs::remove_dir_all(directory).expect("temporary directory cleanup");
}
