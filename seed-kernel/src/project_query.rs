use alloc::{string::String, vec::Vec};

use raios_core::{
    project_workspace::{Classification, ProjectId, MAX_PATH_BYTES},
    sha256_bytes,
};

use crate::{module_candidate_channel::decode_base64_chunk, project_store};

pub(crate) const MAX_READ_BYTES: usize = 512;
pub(crate) const MAX_QUERY_BYTES: usize = 128;
pub(crate) const MAX_SEARCH_RESULTS: usize = 16;

pub(crate) struct ReadResult {
    pub(crate) project_id: [u8; 16],
    pub(crate) revision_sha256: [u8; 32],
    pub(crate) path: String,
    pub(crate) classification: Classification,
    pub(crate) media_type: String,
    pub(crate) blob_sha256: [u8; 32],
    pub(crate) file_byte_len: usize,
    pub(crate) offset: usize,
    pub(crate) requested_len: usize,
    pub(crate) bytes: Vec<u8>,
    pub(crate) eof: bool,
}

pub(crate) struct SearchMatch {
    pub(crate) path: String,
    pub(crate) byte_offset: usize,
    pub(crate) match_len: usize,
}

pub(crate) struct SearchResult {
    pub(crate) project_id: [u8; 16],
    pub(crate) revision_sha256: [u8; 32],
    pub(crate) query_sha256: [u8; 32],
    pub(crate) query_byte_len: usize,
    pub(crate) searched_file_count: usize,
    pub(crate) limit: usize,
    pub(crate) truncated: bool,
    pub(crate) matches: Vec<SearchMatch>,
}

pub(crate) fn read(input: &str) -> Result<ReadResult, &'static str> {
    let args = args::<4>(input, "project.read")?;
    let project_id = parse_project_id(args[0])?;
    let path = decode_path(args[1])?;
    let offset = parse_usize(args[2]).ok_or("project_read_offset_invalid")?;
    let requested_len = parse_usize(args[3]).ok_or("project_read_length_invalid")?;
    if requested_len > MAX_READ_BYTES {
        return Err("project_read_limit_exceeded");
    }
    let loaded = project_store::load(ProjectId::new(project_id))
        .map_err(|error| error.reason())?
        .ok_or("project_not_found")?;
    let file = loaded
        .files
        .iter()
        .find(|file| file.entry.path == path)
        .ok_or("project_path_not_found")?;
    if offset > file.bytes.len() {
        return Err("project_read_offset_out_of_bounds");
    }
    let end = offset
        .checked_add(requested_len)
        .unwrap_or(usize::MAX)
        .min(file.bytes.len());
    Ok(ReadResult {
        project_id,
        revision_sha256: loaded.revision.revision_sha256,
        path,
        classification: file.entry.classification,
        media_type: file.entry.media_type.clone(),
        blob_sha256: file.entry.blob_sha256,
        file_byte_len: file.bytes.len(),
        offset,
        requested_len,
        bytes: file.bytes[offset..end].to_vec(),
        eof: end == file.bytes.len(),
    })
}

pub(crate) fn search(input: &str) -> Result<SearchResult, &'static str> {
    let args = args::<3>(input, "project.search")?;
    let project_id = parse_project_id(args[0])?;
    if args[1].len() > encoded_len_ceiling(MAX_QUERY_BYTES) {
        return Err("project_search_query_limit_exceeded");
    }
    let query = decode_base64_chunk(args[1]).map_err(|_| "project_search_query_invalid")?;
    if query.is_empty() || query.len() > MAX_QUERY_BYTES {
        return Err("project_search_query_limit_exceeded");
    }
    let limit = parse_usize(args[2]).ok_or("project_search_limit_invalid")?;
    if !(1..=MAX_SEARCH_RESULTS).contains(&limit) {
        return Err("project_search_limit_invalid");
    }
    let loaded = project_store::load(ProjectId::new(project_id))
        .map_err(|error| error.reason())?
        .ok_or("project_not_found")?;
    let mut matches = Vec::new();
    let mut searched_file_count = 0usize;
    let mut truncated = false;
    'files: for file in &loaded.files {
        if !file.entry.media_type.starts_with("text/")
            || core::str::from_utf8(&file.bytes).is_err()
            || query.len() > file.bytes.len()
        {
            continue;
        }
        searched_file_count += 1;
        for byte_offset in 0..=file.bytes.len() - query.len() {
            if file.bytes[byte_offset..].starts_with(&query) {
                if matches.len() == limit {
                    truncated = true;
                    break 'files;
                }
                matches.push(SearchMatch {
                    path: file.entry.path.clone(),
                    byte_offset,
                    match_len: query.len(),
                });
            }
        }
    }
    Ok(SearchResult {
        project_id,
        revision_sha256: loaded.revision.revision_sha256,
        query_sha256: sha256_bytes(&query),
        query_byte_len: query.len(),
        searched_file_count,
        limit,
        truncated,
        matches,
    })
}

fn args<'a, const N: usize>(input: &'a str, method: &str) -> Result<[&'a str; N], &'static str> {
    let mut values = input.trim().split_ascii_whitespace();
    if !values
        .next()
        .is_some_and(|value| value.eq_ignore_ascii_case(method))
    {
        return Err("project_query_malformed");
    }
    let mut out = [""; N];
    for value in &mut out {
        *value = values.next().ok_or("project_query_malformed")?;
    }
    if values.next().is_some() {
        return Err("project_query_malformed");
    }
    Ok(out)
}

fn decode_path(value: &str) -> Result<String, &'static str> {
    if value.len() > encoded_len_ceiling(MAX_PATH_BYTES) {
        return Err("project_path_invalid");
    }
    let decoded = decode_base64_chunk(value).map_err(|_| "project_path_invalid")?;
    if decoded.is_empty() || decoded.len() > MAX_PATH_BYTES {
        return Err("project_path_invalid");
    }
    Ok(String::from(
        core::str::from_utf8(&decoded).map_err(|_| "project_path_invalid")?,
    ))
}

fn parse_project_id(value: &str) -> Result<[u8; 16], &'static str> {
    if value.len() != 32 {
        return Err("project_id_invalid");
    }
    let mut out = [0u8; 16];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        out[index] = (hex(pair[0]).ok_or("project_id_invalid")? << 4)
            | hex(pair[1]).ok_or("project_id_invalid")?;
    }
    Ok(out)
}

fn hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn parse_usize(value: &str) -> Option<usize> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.bytes().try_fold(0usize, |parsed, byte| {
        parsed.checked_mul(10)?.checked_add((byte - b'0') as usize)
    })
}

fn encoded_len_ceiling(decoded_len: usize) -> usize {
    decoded_len.saturating_add(2) / 3 * 4
}
