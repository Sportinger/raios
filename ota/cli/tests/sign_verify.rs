use std::fs::File;
use std::io::Write;

use ota_tools::{KeyMaterial, SignedBlob, SignerCertificate};

#[test]
fn sign_and_verify_roundtrip() -> anyhow::Result<()> {
    let tempdir = tempfile::tempdir()?;
    let blob_path = tempdir.path().join("blob.bin");
    let mut file = File::create(&blob_path)?;
    file.write_all(b"seed os test payload")?;

    let root = KeyMaterial::from_seed("root", "test")?;
    let online = KeyMaterial::from_seed("online", "test")?;
    let cert = SignerCertificate::new(&online, &root, 1_700_000_000_000, 60_000)?;
    let manifest = SignedBlob::sign(&blob_path, &online, &cert, None)?;

    let root_public = root.public_key()?;
    manifest.verify(&blob_path, &root_public)?;
    Ok(())
}
