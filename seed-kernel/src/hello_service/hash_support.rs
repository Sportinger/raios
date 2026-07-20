use super::*;

pub(crate) fn finalize_sha256(hash: Sha256) -> [u8; 32] {
    let digest = hash.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

pub(crate) fn hash_line_str(hash: &mut Sha256, key: &'static [u8], value: &str) {
    hash.update(key);
    hash.update(b"=");
    hash.update(value.as_bytes());
    hash.update(b"\n");
}

pub(crate) fn hash_line_hash(hash: &mut Sha256, key: &'static [u8], value: [u8; 32]) {
    hash.update(key);
    hash.update(b"=");
    hash.update(value);
    hash.update(b"\n");
}

pub(crate) fn hash_line_bool(hash: &mut Sha256, key: &'static [u8], value: bool) {
    hash.update(key);
    if value {
        hash.update(b"=true\n");
    } else {
        hash.update(b"=false\n");
    }
}

pub(crate) fn hash_line_u64(hash: &mut Sha256, key: &'static [u8], value: u64) {
    hash.update(key);
    hash.update(b"=");
    hash.update(value.to_le_bytes());
    hash.update(b"\n");
}

pub(crate) fn canonical_line_str_len(key: &'static [u8], value: &str) -> u64 {
    key.len() as u64 + 1 + value.len() as u64 + 1
}

pub(crate) fn canonical_line_hash_len(key: &'static [u8]) -> u64 {
    key.len() as u64 + 1 + 32 + 1
}

pub(crate) fn canonical_line_bool_len(key: &'static [u8], value: bool) -> u64 {
    key.len() as u64 + 1 + if value { 4 } else { 5 } + 1
}

pub(crate) fn write_canonical_line_str(
    image: &mut [u8; ahci::SECTOR_BYTES],
    offset: &mut usize,
    key: &'static [u8],
    value: &str,
) {
    write_sector_image_bytes(image, offset, key);
    write_sector_image_bytes(image, offset, b"=");
    write_sector_image_bytes(image, offset, value.as_bytes());
    write_sector_image_bytes(image, offset, b"\n");
}

pub(crate) fn write_canonical_line_hash(
    image: &mut [u8; ahci::SECTOR_BYTES],
    offset: &mut usize,
    key: &'static [u8],
    value: [u8; 32],
) {
    write_sector_image_bytes(image, offset, key);
    write_sector_image_bytes(image, offset, b"=");
    write_sector_image_bytes(image, offset, &value);
    write_sector_image_bytes(image, offset, b"\n");
}

pub(crate) fn write_canonical_line_bool(
    image: &mut [u8; ahci::SECTOR_BYTES],
    offset: &mut usize,
    key: &'static [u8],
    value: bool,
) {
    write_sector_image_bytes(image, offset, key);
    write_sector_image_bytes(
        image,
        offset,
        if value {
            &b"=true\n"[..]
        } else {
            &b"=false\n"[..]
        },
    );
}

pub(crate) fn write_sector_image_bytes(
    image: &mut [u8; ahci::SECTOR_BYTES],
    offset: &mut usize,
    bytes: &[u8],
) {
    let mut idx = 0usize;
    while idx < bytes.len() && *offset < image.len() {
        image[*offset] = bytes[idx];
        *offset += 1;
        idx += 1;
    }
}
