#![cfg_attr(not(test), no_std)]

//! Pure, allocation-free W7 TLS/HTTP/acquisition decisions shared by the
//! signed guest, host fixture tests, and the kernel's grants-nothing probe.

use core::str;

pub const REQUEST_MAGIC: &[u8; 8] = b"RW7REQ01";
pub const REQUEST_HEADER_LEN: usize = 176;
pub const MAX_REQUEST_LEN: usize = 512;
pub const FIXED_SNI: &[u8] = b"w7.test.raios";
pub const CAS_PATH_PREFIX: &[u8] = b"/raios/cas/sha256/";
pub const MAX_BODY_LEN: usize = 256 * 1024;
pub const CHUNK_LEN: usize = 64 * 1024;
pub const MAX_CHUNKS: usize = 4;
pub const MAX_HTTP_HEADER_LEN: usize = 4096;
static OVERSIZED_HTTP_HEADER_FIXTURE: [u8; MAX_HTTP_HEADER_LEN + 1] =
    [b'a'; MAX_HTTP_HEADER_LEN + 1];
pub const MAX_TLS_RECORD_PLAINTEXT: usize = 16 * 1024;
pub const MAX_TLS_RECORD_CIPHERTEXT: usize = MAX_TLS_RECORD_PLAINTEXT + 256;
pub const MAX_TRANSCRIPT_LEN: usize = 16 * 1024;
pub const TLS_PUBLIC_OUTPUT_LEN: usize = 97;
pub const TLS_RECORD_HEADER_LEN: usize = 5;
pub const TLS_TAG_LEN: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum Error {
    RequestFraming = 100,
    RequestSource = 101,
    RequestGeometry = 102,
    BufferTooSmall = 103,
    TlsRecord = 110,
    TlsHandshake = 111,
    TlsSequence = 112,
    TlsServerHello = 113,
    TlsCertificate = 114,
    TlsCertificateVerify = 115,
    TlsFinished = 116,
    HttpHeader = 120,
    HttpStatus = 121,
    HttpContentLength = 122,
    HttpContentType = 123,
    HttpEncoding = 124,
    HttpBodyLength = 125,
    ChunkGeometry = 130,
    HostCall = 140,
}

impl Error {
    pub const fn guest_result(self) -> i32 {
        -(self as i32)
    }
}

#[derive(Clone, Copy)]
pub struct Request<'a> {
    pub sni: &'a [u8],
    pub path: &'a [u8],
    pub total_len: usize,
    pub whole_sha256: [u8; 32],
    pub chunk_count: usize,
    pub chunk_sha256: [[u8; 32]; MAX_CHUNKS],
}

pub fn parse_request(input: &[u8]) -> Result<Request<'_>, Error> {
    if input.len() < REQUEST_HEADER_LEN
        || input.len() > MAX_REQUEST_LEN
        || &input[..8] != REQUEST_MAGIC
    {
        return Err(Error::RequestFraming);
    }
    let sni_len = input[8] as usize;
    let path_len = u16::from_be_bytes([input[9], input[10]]) as usize;
    let total_len = u32::from_be_bytes([input[11], input[12], input[13], input[14]]) as usize;
    let chunk_count = input[15] as usize;
    let strings_len = sni_len.checked_add(path_len).ok_or(Error::RequestFraming)?;
    if input.len()
        != REQUEST_HEADER_LEN
            .checked_add(strings_len)
            .ok_or(Error::RequestFraming)?
    {
        return Err(Error::RequestFraming);
    }
    let mut whole_sha256 = [0u8; 32];
    whole_sha256.copy_from_slice(&input[16..48]);
    let mut chunk_sha256 = [[0u8; 32]; MAX_CHUNKS];
    let mut index = 0;
    while index < MAX_CHUNKS {
        chunk_sha256[index].copy_from_slice(&input[48 + index * 32..80 + index * 32]);
        index += 1;
    }
    let sni = &input[REQUEST_HEADER_LEN..REQUEST_HEADER_LEN + sni_len];
    let path = &input[REQUEST_HEADER_LEN + sni_len..];
    if sni != FIXED_SNI || !path_matches_hash(path, &whole_sha256) {
        return Err(Error::RequestSource);
    }
    let expected_count = chunk_count_for(total_len).ok_or(Error::RequestGeometry)?;
    if total_len == 0 || total_len > MAX_BODY_LEN || chunk_count != expected_count {
        return Err(Error::RequestGeometry);
    }
    for (idx, hash) in chunk_sha256.iter().enumerate() {
        if (idx < chunk_count) == hash.iter().all(|byte| *byte == 0) {
            return Err(Error::RequestGeometry);
        }
    }
    Ok(Request {
        sni,
        path,
        total_len,
        whole_sha256,
        chunk_count,
        chunk_sha256,
    })
}

pub const fn chunk_count_for(total_len: usize) -> Option<usize> {
    if total_len == 0 || total_len > MAX_BODY_LEN {
        return None;
    }
    Some((total_len + CHUNK_LEN - 1) / CHUNK_LEN)
}

pub const fn canonical_chunk_len(total_len: usize, index: usize) -> Option<usize> {
    let count = match chunk_count_for(total_len) {
        Some(count) => count,
        None => return None,
    };
    if index >= count {
        return None;
    }
    let consumed = index * CHUNK_LEN;
    let remaining = total_len - consumed;
    Some(if remaining > CHUNK_LEN {
        CHUNK_LEN
    } else {
        remaining
    })
}

fn path_matches_hash(path: &[u8], hash: &[u8; 32]) -> bool {
    if path.len() != CAS_PATH_PREFIX.len() + 64 || !path.starts_with(CAS_PATH_PREFIX) {
        return false;
    }
    let encoded = &path[CAS_PATH_PREFIX.len()..];
    let mut index = 0;
    while index < 32 {
        if encoded[index * 2] != hex(hash[index] >> 4)
            || encoded[index * 2 + 1] != hex(hash[index] & 0x0f)
        {
            return false;
        }
        index += 1;
    }
    true
}

const fn hex(value: u8) -> u8 {
    b"0123456789abcdef"[value as usize]
}

pub fn build_client_hello(
    sni: &[u8],
    public: &[u8; TLS_PUBLIC_OUTPUT_LEN],
    out: &mut [u8],
) -> Result<usize, Error> {
    if sni != FIXED_SNI || public[32] != 4 {
        return Err(Error::RequestSource);
    }
    let sni_ext_len = 5 + sni.len();
    let extensions_len = 4 + sni_ext_len + 7 + 8 + 8 + 75;
    let body_len = 2 + 32 + 1 + 2 + 2 + 1 + 1 + 2 + extensions_len;
    let handshake_len = 4 + body_len;
    let record_len = TLS_RECORD_HEADER_LEN + handshake_len;
    if out.len() < record_len || handshake_len > u16::MAX as usize {
        return Err(Error::BufferTooSmall);
    }
    out[..record_len].fill(0);
    out[0..5].copy_from_slice(&[22, 3, 1, (handshake_len >> 8) as u8, handshake_len as u8]);
    out[5] = 1;
    put_u24(&mut out[6..9], body_len);
    let mut cursor = 9;
    out[cursor..cursor + 2].copy_from_slice(&[3, 3]);
    cursor += 2;
    out[cursor..cursor + 32].copy_from_slice(&public[..32]);
    cursor += 32;
    out[cursor] = 0;
    cursor += 1;
    out[cursor..cursor + 4].copy_from_slice(&[0, 2, 0x13, 0x01]);
    cursor += 4;
    out[cursor..cursor + 2].copy_from_slice(&[1, 0]);
    cursor += 2;
    out[cursor..cursor + 2].copy_from_slice(&(extensions_len as u16).to_be_bytes());
    cursor += 2;
    out[cursor..cursor + 4].copy_from_slice(&[0, 0, (sni_ext_len >> 8) as u8, sni_ext_len as u8]);
    cursor += 4;
    out[cursor..cursor + 2].copy_from_slice(&((sni.len() + 3) as u16).to_be_bytes());
    cursor += 2;
    out[cursor] = 0;
    out[cursor + 1..cursor + 3].copy_from_slice(&(sni.len() as u16).to_be_bytes());
    cursor += 3;
    out[cursor..cursor + sni.len()].copy_from_slice(sni);
    cursor += sni.len();
    for extension in [
        &[0, 43, 0, 3, 2, 3, 4][..],
        &[0, 10, 0, 4, 0, 2, 0, 23][..],
        &[0, 13, 0, 4, 0, 2, 4, 3][..],
    ] {
        out[cursor..cursor + extension.len()].copy_from_slice(extension);
        cursor += extension.len();
    }
    out[cursor..cursor + 6].copy_from_slice(&[0, 51, 0, 71, 0, 69]);
    cursor += 6;
    out[cursor..cursor + 4].copy_from_slice(&[0, 23, 0, 65]);
    cursor += 4;
    out[cursor..cursor + 65].copy_from_slice(&public[32..]);
    cursor += 65;
    if cursor != record_len {
        return Err(Error::TlsHandshake);
    }
    Ok(record_len)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TlsRecordHeader {
    pub content_type: u8,
    pub len: usize,
}

pub fn parse_tls_record_header(header: &[u8]) -> Result<TlsRecordHeader, Error> {
    if header.len() != 5 || header[1] != 3 || !matches!(header[2], 1 | 3) {
        return Err(Error::TlsRecord);
    }
    let len = u16::from_be_bytes([header[3], header[4]]) as usize;
    if len == 0 || len > MAX_TLS_RECORD_CIPHERTEXT {
        return Err(Error::TlsRecord);
    }
    Ok(TlsRecordHeader {
        content_type: header[0],
        len,
    })
}

pub fn make_record_header(cipher_len: usize) -> Result<[u8; 5], Error> {
    if cipher_len == 0 || cipher_len > MAX_TLS_RECORD_CIPHERTEXT || cipher_len > u16::MAX as usize {
        return Err(Error::TlsRecord);
    }
    Ok([23, 3, 3, (cipher_len >> 8) as u8, cipher_len as u8])
}

pub fn handshake_frame(input: &[u8]) -> Result<Option<(u8, &[u8], usize)>, Error> {
    if input.len() < 4 {
        return Ok(None);
    }
    let len = read_u24(&input[1..4]);
    let total = 4usize.checked_add(len).ok_or(Error::TlsHandshake)?;
    if total > MAX_TRANSCRIPT_LEN {
        return Err(Error::TlsHandshake);
    }
    if input.len() < total {
        return Ok(None);
    }
    Ok(Some((input[0], &input[4..total], total)))
}

pub fn parse_server_hello(body: &[u8]) -> Result<[u8; 65], Error> {
    if body.len() < 40 || body[..2] != [3, 3] {
        return Err(Error::TlsServerHello);
    }
    let mut cursor = 34;
    let session_len = *body.get(cursor).ok_or(Error::TlsServerHello)? as usize;
    if session_len != 0 {
        return Err(Error::TlsServerHello);
    }
    cursor += 1;
    cursor = cursor
        .checked_add(session_len)
        .ok_or(Error::TlsServerHello)?;
    if body.get(cursor..cursor + 3) != Some(&[0x13, 0x01, 0][..]) {
        return Err(Error::TlsServerHello);
    }
    cursor += 3;
    let extensions_len = read_u16(body, cursor)?;
    cursor += 2;
    if cursor + extensions_len != body.len() {
        return Err(Error::TlsServerHello);
    }
    let end = body.len();
    let mut version_ok = false;
    let mut key = None;
    while cursor < end {
        let kind = read_u16(body, cursor)?;
        let len = read_u16(body, cursor + 2)?;
        cursor += 4;
        let value = body
            .get(cursor..cursor + len)
            .ok_or(Error::TlsServerHello)?;
        cursor += len;
        match kind {
            43 if value == [3, 4] => version_ok = true,
            51 if value.len() == 69 && value[..4] == [0, 23, 0, 65] && value[4] == 4 => {
                let mut point = [0u8; 65];
                point.copy_from_slice(&value[4..]);
                key = Some(point);
            }
            _ => {}
        }
    }
    if version_ok {
        key.ok_or(Error::TlsServerHello)
    } else {
        Err(Error::TlsServerHello)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandshakeState {
    ExpectEncryptedExtensions,
    ExpectCertificate,
    ExpectCertificateVerify,
    ExpectFinished,
    Complete,
}

impl HandshakeState {
    pub fn observe(&mut self, kind: u8) -> Result<(), Error> {
        *self = match (*self, kind) {
            (Self::ExpectEncryptedExtensions, 8) => Self::ExpectCertificate,
            (Self::ExpectCertificate, 11) => Self::ExpectCertificateVerify,
            (Self::ExpectCertificateVerify, 15) => Self::ExpectFinished,
            (Self::ExpectFinished, 20) => Self::Complete,
            _ => return Err(Error::TlsSequence),
        };
        Ok(())
    }
}

pub fn validate_encrypted_extensions(body: &[u8]) -> Result<(), Error> {
    let len = read_u16(body, 0)?;
    if len + 2 == body.len() {
        Ok(())
    } else {
        Err(Error::TlsHandshake)
    }
}

pub fn leaf_certificate(body: &[u8]) -> Result<&[u8], Error> {
    let context_len = *body.first().ok_or(Error::TlsCertificate)? as usize;
    let mut cursor = 1usize
        .checked_add(context_len)
        .ok_or(Error::TlsCertificate)?;
    if cursor + 3 > body.len() {
        return Err(Error::TlsCertificate);
    }
    let list_len = read_u24(&body[cursor..cursor + 3]);
    cursor += 3;
    if cursor + list_len != body.len() || list_len < 5 {
        return Err(Error::TlsCertificate);
    }
    let cert_len = read_u24(body.get(cursor..cursor + 3).ok_or(Error::TlsCertificate)?);
    cursor += 3;
    let cert = body
        .get(cursor..cursor + cert_len)
        .ok_or(Error::TlsCertificate)?;
    cursor += cert_len;
    let extensions_len = read_u16(body, cursor)?;
    cursor += 2;
    if cursor + extensions_len > body.len() || cert.is_empty() {
        return Err(Error::TlsCertificate);
    }
    Ok(cert)
}

pub fn certificate_verify(body: &[u8]) -> Result<&[u8], Error> {
    if body.len() < 4 || body[..2] != [4, 3] {
        return Err(Error::TlsCertificateVerify);
    }
    let len = u16::from_be_bytes([body[2], body[3]]) as usize;
    if len == 0 || len + 4 != body.len() {
        return Err(Error::TlsCertificateVerify);
    }
    Ok(&body[4..])
}

pub fn finished_proof(body: &[u8]) -> Result<&[u8; 32], Error> {
    body.try_into().map_err(|_| Error::TlsFinished)
}

pub fn certificate_verify_message(hash: &[u8; 32], out: &mut [u8]) -> Result<usize, Error> {
    const CONTEXT: &[u8] = b"TLS 1.3, server CertificateVerify\0";
    let len = 64 + CONTEXT.len() + 32;
    if out.len() < len {
        return Err(Error::BufferTooSmall);
    }
    out[..64].fill(0x20);
    out[64..64 + CONTEXT.len()].copy_from_slice(CONTEXT);
    out[64 + CONTEXT.len()..len].copy_from_slice(hash);
    Ok(len)
}

pub fn strip_tls_inner_plaintext(plain: &[u8]) -> Result<(u8, &[u8]), Error> {
    let type_index = plain
        .iter()
        .rposition(|byte| *byte != 0)
        .ok_or(Error::TlsRecord)?;
    let content_type = plain[type_index];
    if !matches!(content_type, 21 | 22 | 23) {
        return Err(Error::TlsRecord);
    }
    Ok((content_type, &plain[..type_index]))
}

pub fn build_fixed_get(request: &Request<'_>, out: &mut [u8]) -> Result<usize, Error> {
    let parts: [&[u8]; 7] = [
        b"GET ",
        request.path,
        b" HTTP/1.1\r\nHost: ",
        request.sni,
        b"\r\nAccept: application/octet-stream\r\nConnection: close\r\n",
        b"\r\n",
        b"",
    ];
    let len = parts.iter().try_fold(0usize, |sum, part| {
        sum.checked_add(part.len()).ok_or(Error::BufferTooSmall)
    })?;
    if out.len() < len {
        return Err(Error::BufferTooSmall);
    }
    let mut cursor = 0;
    for part in parts {
        out[cursor..cursor + part.len()].copy_from_slice(part);
        cursor += part.len();
    }
    Ok(len)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HttpHead {
    pub body_offset: usize,
    pub content_length: usize,
}

pub fn find_http_head_end(bytes: &[u8]) -> Result<Option<usize>, Error> {
    if bytes.len() > MAX_HTTP_HEADER_LEN {
        return Err(Error::HttpHeader);
    }
    Ok(find(bytes, b"\r\n\r\n").map(|index| index + 4))
}

pub fn parse_http_head(bytes: &[u8], expected_len: usize) -> Result<HttpHead, Error> {
    let body_offset = find_http_head_end(bytes)?.ok_or(Error::HttpHeader)?;
    let head = &bytes[..body_offset - 2];
    let first_end = find(head, b"\r\n").ok_or(Error::HttpHeader)?;
    let status = str::from_utf8(&head[..first_end]).map_err(|_| Error::HttpHeader)?;
    let mut fields = status.split_ascii_whitespace();
    if fields.next() != Some("HTTP/1.1") || fields.next() != Some("200") {
        return Err(Error::HttpStatus);
    }
    let mut content_length = None;
    let mut content_type = None;
    let mut cursor = first_end + 2;
    while cursor < head.len() {
        let next = find(&head[cursor..], b"\r\n")
            .map(|offset| cursor + offset)
            .unwrap_or(head.len());
        let line = &head[cursor..next];
        if line.is_empty() || line[0].is_ascii_whitespace() {
            return Err(Error::HttpHeader);
        }
        let colon = line
            .iter()
            .position(|byte| *byte == b':')
            .ok_or(Error::HttpHeader)?;
        let name = &line[..colon];
        let value = trim(&line[colon + 1..]);
        if eq(name, b"content-length") {
            if content_length.is_some() {
                return Err(Error::HttpContentLength);
            }
            content_length = Some(decimal(value).ok_or(Error::HttpContentLength)?);
        } else if eq(name, b"content-type") {
            if content_type.is_some() {
                return Err(Error::HttpContentType);
            }
            content_type = Some(value);
        } else if eq(name, b"transfer-encoding") || eq(name, b"content-encoding") {
            return Err(Error::HttpEncoding);
        }
        cursor = next.saturating_add(2);
    }
    if content_length != Some(expected_len) {
        return Err(Error::HttpContentLength);
    }
    if content_type.is_none_or(|value| !eq(value, b"application/octet-stream")) {
        return Err(Error::HttpContentType);
    }
    Ok(HttpHead {
        body_offset,
        content_length: expected_len,
    })
}

pub trait W7Abi {
    fn tcp_open(&mut self) -> i32;
    fn tcp_send(&mut self, connection: i32, bytes: &[u8]) -> i32;
    fn tcp_recv(&mut self, connection: i32, bytes: &mut [u8]) -> i32;
    fn tcp_close(&mut self, connection: i32) -> i32;
    fn tls13_session_open(
        &mut self,
        connection: i32,
        public: &mut [u8; TLS_PUBLIC_OUTPUT_LEN],
    ) -> i32;
    fn sha256(&mut self, session: i32, bytes: &[u8], out: &mut [u8; 32]) -> i32;
    fn p256_verify(&mut self, session: i32, spki: &[u8], message: &[u8], signature: &[u8]) -> i32;
    fn tls13_handshake_keys(
        &mut self,
        session: i32,
        peer_public: &[u8; 65],
        hello_hash: &[u8; 32],
    ) -> i32;
    fn tls13_application_keys(&mut self, session: i32, handshake_hash: &[u8; 32]) -> i32;
    fn tls13_finished(
        &mut self,
        session: i32,
        mode: i32,
        transcript_hash: &[u8; 32],
        proof: &mut [u8; 32],
    ) -> i32;
    fn tls13_aead_seal(
        &mut self,
        session: i32,
        header: &[u8; 5],
        plain: &[u8],
        out: &mut [u8],
    ) -> i32;
    fn tls13_aead_open(
        &mut self,
        session: i32,
        header: &[u8; 5],
        cipher: &[u8],
        out: &mut [u8],
    ) -> i32;
    fn chunk_accept(&mut self, index: usize, bytes: &[u8]) -> i32;
    fn finalize(&mut self) -> i32;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChunkState {
    pub total: usize,
    pub buffered: usize,
    pub index: usize,
}

impl ChunkState {
    pub const fn new() -> Self {
        Self {
            total: 0,
            buffered: 0,
            index: 0,
        }
    }

    pub fn push<A: W7Abi>(
        &mut self,
        abi: &mut A,
        expected_total: usize,
        input: &[u8],
        chunk: &mut [u8; CHUNK_LEN],
    ) -> Result<(), Error> {
        if self
            .total
            .checked_add(input.len())
            .is_none_or(|total| total > expected_total)
        {
            return Err(Error::HttpBodyLength);
        }
        let mut input = input;
        while !input.is_empty() {
            let take = input.len().min(CHUNK_LEN - self.buffered);
            chunk[self.buffered..self.buffered + take].copy_from_slice(&input[..take]);
            self.buffered += take;
            self.total += take;
            input = &input[take..];
            if self.buffered == CHUNK_LEN {
                self.flush(abi, chunk)?;
            }
        }
        Ok(())
    }

    pub fn finish<A: W7Abi>(
        &mut self,
        abi: &mut A,
        expected_total: usize,
        chunk: &mut [u8; CHUNK_LEN],
    ) -> Result<(), Error> {
        if self.total != expected_total || self.index >= MAX_CHUNKS {
            return Err(Error::HttpBodyLength);
        }
        if self.buffered != 0 {
            self.flush(abi, chunk)?;
        }
        if self.index != chunk_count_for(expected_total).ok_or(Error::ChunkGeometry)?
            || abi.finalize() != 0
        {
            return Err(Error::HostCall);
        }
        Ok(())
    }

    fn flush<A: W7Abi>(&mut self, abi: &mut A, chunk: &mut [u8; CHUNK_LEN]) -> Result<(), Error> {
        if self.index >= MAX_CHUNKS || abi.chunk_accept(self.index, &chunk[..self.buffered]) != 0 {
            return Err(Error::HostCall);
        }
        self.index += 1;
        self.buffered = 0;
        Ok(())
    }
}

impl Default for ChunkState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
pub struct VectorReport {
    pub client_hello_positive: bool,
    pub tls_positive: bool,
    pub malformed_tls_negative_count: u32,
    pub http_positive: bool,
    pub malformed_http_negative_count: u32,
    pub chunk_geometry_positive: bool,
    pub all_positive: bool,
}

pub fn run_fixture_vectors() -> VectorReport {
    let mut public = [0u8; TLS_PUBLIC_OUTPUT_LEN];
    public[32] = 4;
    let mut hello = [0u8; 512];
    let client_hello_positive = build_client_hello(FIXED_SNI, &public, &mut hello)
        .is_ok_and(|len| len > 100 && hello[0] == 22 && find(&hello[..len], FIXED_SNI).is_some());
    let mut state = HandshakeState::ExpectEncryptedExtensions;
    let tls_positive = [8, 11, 15, 20]
        .into_iter()
        .all(|kind| state.observe(kind).is_ok())
        && state == HandshakeState::Complete;
    let malformed_tls_negative_count = [
        parse_tls_record_header(&[23, 2, 3, 0, 1]).is_err(),
        parse_tls_record_header(&[23, 3, 3, 0, 0]).is_err(),
        handshake_frame(&[2, 0, 0, 1]).is_ok_and(|frame| frame.is_none()),
        parse_server_hello(&[3, 3, 0]).is_err(),
        HandshakeState::ExpectCertificate.observe(20).is_err(),
        certificate_verify(&[4, 1, 0, 0]).is_err(),
        finished_proof(&[0; 31]).is_err(),
    ]
    .into_iter()
    .filter(|ok| *ok)
    .count() as u32;
    let valid_http = b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 3\r\nConnection: close\r\n\r\nabc";
    let http_positive =
        parse_http_head(valid_http, 3).is_ok_and(|head| &valid_http[head.body_offset..] == b"abc");
    let invalid_http: [&[u8]; 10] = [
        b"HTTP/1.1 302 Found\r\nContent-Type: application/octet-stream\r\nContent-Length: 3\r\n\r\nabc",
        b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\n\r\nabc",
        b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 3\r\nContent-Length: 3\r\n\r\nabc",
        b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: x\r\n\r\nabc",
        b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 3\r\nTransfer-Encoding: chunked\r\n\r\nabc",
        b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 3\r\nContent-Encoding: gzip\r\n\r\nabc",
        b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 3\r\n\r\nabc",
        b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 2\r\n\r\nabc",
        b"HTTP/1.1 200 OK\r\n Content-Type: application/octet-stream\r\nContent-Length: 3\r\n\r\nabc",
        &OVERSIZED_HTTP_HEADER_FIXTURE,
    ];
    let malformed_http_negative_count = invalid_http
        .into_iter()
        .filter(|fixture| parse_http_head(fixture, 3).is_err())
        .count() as u32;
    let chunk_geometry_positive = chunk_count_for(MAX_BODY_LEN) == Some(4)
        && canonical_chunk_len(CHUNK_LEN + 1, 0) == Some(CHUNK_LEN)
        && canonical_chunk_len(CHUNK_LEN + 1, 1) == Some(1)
        && canonical_chunk_len(CHUNK_LEN + 1, 2).is_none();
    VectorReport {
        client_hello_positive,
        tls_positive,
        malformed_tls_negative_count,
        http_positive,
        malformed_http_negative_count,
        chunk_geometry_positive,
        all_positive: client_hello_positive
            && tls_positive
            && malformed_tls_negative_count == 7
            && http_positive
            && malformed_http_negative_count == 10
            && chunk_geometry_positive,
    }
}

fn put_u24(out: &mut [u8], value: usize) {
    out[0] = (value >> 16) as u8;
    out[1] = (value >> 8) as u8;
    out[2] = value as u8;
}

fn read_u24(bytes: &[u8]) -> usize {
    ((bytes[0] as usize) << 16) | ((bytes[1] as usize) << 8) | bytes[2] as usize
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<usize, Error> {
    let pair = bytes.get(offset..offset + 2).ok_or(Error::TlsHandshake)?;
    Ok(u16::from_be_bytes([pair[0], pair[1]]) as usize)
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn trim(mut bytes: &[u8]) -> &[u8] {
    while bytes.first().is_some_and(|byte| byte.is_ascii_whitespace()) {
        bytes = &bytes[1..];
    }
    while bytes.last().is_some_and(|byte| byte.is_ascii_whitespace()) {
        bytes = &bytes[..bytes.len() - 1];
    }
    bytes
}

fn eq(left: &[u8], right: &[u8]) -> bool {
    left.eq_ignore_ascii_case(right)
}

fn decimal(bytes: &[u8]) -> Option<usize> {
    if bytes.is_empty() {
        return None;
    }
    bytes.iter().try_fold(0usize, |value, byte| {
        if !byte.is_ascii_digit() {
            return None;
        }
        value.checked_mul(10)?.checked_add((byte - b'0') as usize)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    struct MockAbi {
        calls: Vec<(usize, usize)>,
        finalized: bool,
    }

    impl MockAbi {
        fn new() -> Self {
            Self {
                calls: Vec::new(),
                finalized: false,
            }
        }
    }

    impl W7Abi for MockAbi {
        fn tcp_open(&mut self) -> i32 {
            1
        }
        fn tcp_send(&mut self, _: i32, bytes: &[u8]) -> i32 {
            bytes.len() as i32
        }
        fn tcp_recv(&mut self, _: i32, _: &mut [u8]) -> i32 {
            -9
        }
        fn tcp_close(&mut self, _: i32) -> i32 {
            0
        }
        fn tls13_session_open(&mut self, _: i32, public: &mut [u8; 97]) -> i32 {
            public[32] = 4;
            2
        }
        fn sha256(&mut self, _: i32, _: &[u8], out: &mut [u8; 32]) -> i32 {
            out.fill(1);
            32
        }
        fn p256_verify(&mut self, _: i32, _: &[u8], _: &[u8], _: &[u8]) -> i32 {
            1
        }
        fn tls13_handshake_keys(&mut self, _: i32, _: &[u8; 65], _: &[u8; 32]) -> i32 {
            0
        }
        fn tls13_application_keys(&mut self, _: i32, _: &[u8; 32]) -> i32 {
            0
        }
        fn tls13_finished(&mut self, _: i32, _: i32, _: &[u8; 32], proof: &mut [u8; 32]) -> i32 {
            proof.fill(2);
            32
        }
        fn tls13_aead_seal(&mut self, _: i32, _: &[u8; 5], plain: &[u8], _: &mut [u8]) -> i32 {
            (plain.len() + 16) as i32
        }
        fn tls13_aead_open(&mut self, _: i32, _: &[u8; 5], cipher: &[u8], _: &mut [u8]) -> i32 {
            (cipher.len() - 16) as i32
        }
        fn chunk_accept(&mut self, index: usize, bytes: &[u8]) -> i32 {
            self.calls.push((index, bytes.len()));
            0
        }
        fn finalize(&mut self) -> i32 {
            self.finalized = true;
            0
        }
    }

    fn request_fixture(total_len: usize) -> Vec<u8> {
        let hash = [0x2au8; 32];
        let mut path = CAS_PATH_PREFIX.to_vec();
        for byte in hash {
            path.extend_from_slice(&[hex(byte >> 4), hex(byte & 15)]);
        }
        let count = chunk_count_for(total_len).unwrap();
        let mut out = vec![0u8; REQUEST_HEADER_LEN];
        out[..8].copy_from_slice(REQUEST_MAGIC);
        out[8] = FIXED_SNI.len() as u8;
        out[9..11].copy_from_slice(&(path.len() as u16).to_be_bytes());
        out[11..15].copy_from_slice(&(total_len as u32).to_be_bytes());
        out[15] = count as u8;
        out[16..48].copy_from_slice(&hash);
        for index in 0..count {
            out[48 + index * 32..80 + index * 32].fill(index as u8 + 1);
        }
        out.extend_from_slice(FIXED_SNI);
        out.extend_from_slice(&path);
        out
    }

    #[test]
    fn fixture_vectors_cover_tls_http_and_geometry() {
        assert!(run_fixture_vectors().all_positive);
        let request = request_fixture(CHUNK_LEN + 1);
        let parsed = parse_request(&request).unwrap();
        assert_eq!(parsed.total_len, CHUNK_LEN + 1);
        assert_eq!(parsed.chunk_count, 2);
        let mut malformed = request;
        malformed[15] = 1;
        assert!(parse_request(&malformed).is_err());
    }

    #[test]
    fn server_hello_fixture_extracts_the_fixed_p256_key_share() {
        let mut body = vec![3, 3];
        body.extend_from_slice(&[7; 32]);
        body.extend_from_slice(&[0, 0x13, 0x01, 0]);
        body.extend_from_slice(&79u16.to_be_bytes());
        body.extend_from_slice(&[0, 43, 0, 2, 3, 4]);
        body.extend_from_slice(&[0, 51, 0, 69, 0, 23, 0, 65]);
        body.extend_from_slice(&[4; 65]);
        let key = parse_server_hello(&body).unwrap();
        assert_eq!(key, [4; 65]);
    }

    #[test]
    fn mock_abi_observes_canonical_64k_chunks_and_finalize() {
        let body = vec![0x5a; CHUNK_LEN + 1];
        let mut chunk = [0u8; CHUNK_LEN];
        let mut state = ChunkState::new();
        let mut abi = MockAbi::new();
        for part in body.chunks(4093) {
            state.push(&mut abi, body.len(), part, &mut chunk).unwrap();
        }
        state.finish(&mut abi, body.len(), &mut chunk).unwrap();
        assert_eq!(abi.calls, vec![(0, CHUNK_LEN), (1, 1)]);
        assert!(abi.finalized);
    }

    #[test]
    fn malformed_tls_and_http_vectors_fail_closed() {
        let report = run_fixture_vectors();
        assert_eq!(report.malformed_tls_negative_count, 7);
        assert_eq!(report.malformed_http_negative_count, 10);
        assert_eq!(
            parse_http_head(b"HTTP/1.1 200 OK\r\n\r\n", 1),
            Err(Error::HttpContentLength)
        );
        assert!(leaf_certificate(&[0, 0, 0, 1, 0]).is_err());
        let mut abi = MockAbi::new();
        let mut chunk = [0u8; CHUNK_LEN];
        let mut state = ChunkState::new();
        assert_eq!(
            state.push(&mut abi, 2, b"abc", &mut chunk),
            Err(Error::HttpBodyLength)
        );
        let mut state = ChunkState::new();
        state.push(&mut abi, 3, b"ab", &mut chunk).unwrap();
        assert_eq!(
            state.finish(&mut abi, 3, &mut chunk),
            Err(Error::HttpBodyLength)
        );
        assert!(!abi.finalized);
    }
}
