#![no_std]

use core::{panic::PanicInfo, ptr::addr_of_mut};
use raios_w7_acquire_logic as w7;
use w7::W7Abi as _;

static mut BUFFERS: Buffers = Buffers::new();

struct Buffers {
    request: [u8; w7::MAX_REQUEST_LEN],
    tx: [u8; 1024],
    record: [u8; w7::MAX_TLS_RECORD_CIPHERTEXT],
    plain: [u8; w7::MAX_TLS_RECORD_CIPHERTEXT],
    transcript: [u8; w7::MAX_TRANSCRIPT_LEN],
    handshake: [u8; w7::MAX_TRANSCRIPT_LEN],
    spki: [u8; 512],
    verify: [u8; 256],
    http_head: [u8; w7::MAX_HTTP_HEADER_LEN],
    chunk: [u8; w7::CHUNK_LEN],
}

impl Buffers {
    const fn new() -> Self {
        Self {
            request: [0; w7::MAX_REQUEST_LEN],
            tx: [0; 1024],
            record: [0; w7::MAX_TLS_RECORD_CIPHERTEXT],
            plain: [0; w7::MAX_TLS_RECORD_CIPHERTEXT],
            transcript: [0; w7::MAX_TRANSCRIPT_LEN],
            handshake: [0; w7::MAX_TRANSCRIPT_LEN],
            spki: [0; 512],
            verify: [0; 256],
            http_head: [0; w7::MAX_HTTP_HEADER_LEN],
            chunk: [0; w7::CHUNK_LEN],
        }
    }
}

#[link(wasm_import_module = "env")]
extern "C" {
    #[link_name = "input_len"]
    fn input_len() -> i32;
    #[link_name = "input_read"]
    fn input_read(ptr: i32, len: i32) -> i32;
}

#[link(wasm_import_module = "net")]
extern "C" {
    #[link_name = "tcp_open"]
    fn tcp_open() -> i32;
    #[link_name = "tcp_send"]
    fn tcp_send(connection: i32, ptr: i32, len: i32) -> i32;
    #[link_name = "tcp_recv"]
    fn tcp_recv(connection: i32, ptr: i32, cap: i32) -> i32;
    #[link_name = "tcp_close"]
    fn tcp_close(connection: i32) -> i32;
}

#[link(wasm_import_module = "crypto")]
extern "C" {
    #[link_name = "tls13_session_open"]
    fn tls13_session_open(connection: i32, public_ptr: i32, public_len: i32) -> i32;
    #[link_name = "sha256"]
    fn sha256(session: i32, ptr: i32, len: i32, out32: i32) -> i32;
    #[link_name = "p256_verify"]
    fn p256_verify(
        session: i32,
        spki_ptr: i32,
        spki_len: i32,
        msg_ptr: i32,
        msg_len: i32,
        sig_ptr: i32,
        sig_len: i32,
    ) -> i32;
    #[link_name = "tls13_handshake_keys"]
    fn tls13_handshake_keys(session: i32, peer_ptr: i32, peer_len: i32, hello_hash_ptr: i32)
        -> i32;
    #[link_name = "tls13_application_keys"]
    fn tls13_application_keys(session: i32, handshake_hash_ptr: i32) -> i32;
    #[link_name = "tls13_finished"]
    fn tls13_finished(
        session: i32,
        mode: i32,
        transcript_hash_ptr: i32,
        proof_ptr: i32,
        proof_len: i32,
    ) -> i32;
    #[link_name = "tls13_aead_seal"]
    fn tls13_aead_seal(
        session: i32,
        header_ptr: i32,
        plain_ptr: i32,
        plain_len: i32,
        out_ptr: i32,
        out_cap: i32,
    ) -> i32;
    #[link_name = "tls13_aead_open"]
    fn tls13_aead_open(
        session: i32,
        header_ptr: i32,
        cipher_ptr: i32,
        cipher_len: i32,
        out_ptr: i32,
        out_cap: i32,
    ) -> i32;
}

#[link(wasm_import_module = "acquire")]
extern "C" {
    #[link_name = "chunk_accept"]
    fn chunk_accept(index: i32, ptr: i32, len: i32) -> i32;
    #[link_name = "finalize"]
    fn finalize() -> i32;
}

struct GuestAbi;

impl w7::W7Abi for GuestAbi {
    fn tcp_open(&mut self) -> i32 {
        unsafe { tcp_open() }
    }
    fn tcp_send(&mut self, connection: i32, bytes: &[u8]) -> i32 {
        unsafe { tcp_send(connection, bytes.as_ptr() as i32, bytes.len() as i32) }
    }
    fn tcp_recv(&mut self, connection: i32, bytes: &mut [u8]) -> i32 {
        unsafe { tcp_recv(connection, bytes.as_mut_ptr() as i32, bytes.len() as i32) }
    }
    fn tcp_close(&mut self, connection: i32) -> i32 {
        unsafe { tcp_close(connection) }
    }
    fn tls13_session_open(&mut self, connection: i32, public: &mut [u8; 97]) -> i32 {
        unsafe { tls13_session_open(connection, public.as_mut_ptr() as i32, 97) }
    }
    fn sha256(&mut self, session: i32, bytes: &[u8], out: &mut [u8; 32]) -> i32 {
        unsafe {
            sha256(
                session,
                bytes.as_ptr() as i32,
                bytes.len() as i32,
                out.as_mut_ptr() as i32,
            )
        }
    }
    fn p256_verify(&mut self, session: i32, spki: &[u8], message: &[u8], signature: &[u8]) -> i32 {
        unsafe {
            p256_verify(
                session,
                spki.as_ptr() as i32,
                spki.len() as i32,
                message.as_ptr() as i32,
                message.len() as i32,
                signature.as_ptr() as i32,
                signature.len() as i32,
            )
        }
    }
    fn tls13_handshake_keys(
        &mut self,
        session: i32,
        peer_public: &[u8; 65],
        hello_hash: &[u8; 32],
    ) -> i32 {
        unsafe {
            tls13_handshake_keys(
                session,
                peer_public.as_ptr() as i32,
                65,
                hello_hash.as_ptr() as i32,
            )
        }
    }
    fn tls13_application_keys(&mut self, session: i32, handshake_hash: &[u8; 32]) -> i32 {
        unsafe { tls13_application_keys(session, handshake_hash.as_ptr() as i32) }
    }
    fn tls13_finished(
        &mut self,
        session: i32,
        mode: i32,
        transcript_hash: &[u8; 32],
        proof: &mut [u8; 32],
    ) -> i32 {
        unsafe {
            tls13_finished(
                session,
                mode,
                transcript_hash.as_ptr() as i32,
                proof.as_mut_ptr() as i32,
                32,
            )
        }
    }
    fn tls13_aead_seal(
        &mut self,
        session: i32,
        header: &[u8; 5],
        plain: &[u8],
        out: &mut [u8],
    ) -> i32 {
        unsafe {
            tls13_aead_seal(
                session,
                header.as_ptr() as i32,
                plain.as_ptr() as i32,
                plain.len() as i32,
                out.as_mut_ptr() as i32,
                out.len() as i32,
            )
        }
    }
    fn tls13_aead_open(
        &mut self,
        session: i32,
        header: &[u8; 5],
        cipher: &[u8],
        out: &mut [u8],
    ) -> i32 {
        unsafe {
            tls13_aead_open(
                session,
                header.as_ptr() as i32,
                cipher.as_ptr() as i32,
                cipher.len() as i32,
                out.as_mut_ptr() as i32,
                out.len() as i32,
            )
        }
    }
    fn chunk_accept(&mut self, index: usize, bytes: &[u8]) -> i32 {
        unsafe { chunk_accept(index as i32, bytes.as_ptr() as i32, bytes.len() as i32) }
    }
    fn finalize(&mut self) -> i32 {
        unsafe { finalize() }
    }
}

#[no_mangle]
pub extern "C" fn raios_service_main() -> i32 {
    let raw_len = unsafe { input_len() };
    if raw_len < w7::REQUEST_HEADER_LEN as i32 || raw_len > w7::MAX_REQUEST_LEN as i32 {
        return w7::Error::RequestFraming.guest_result();
    }
    let buffers = unsafe { &mut *addr_of_mut!(BUFFERS) };
    let request_ptr = buffers.request.as_mut_ptr() as i32;
    if unsafe { input_read(request_ptr, raw_len) } != raw_len {
        return w7::Error::HostCall.guest_result();
    }
    let parsed = match w7::parse_request(&buffers.request[..raw_len as usize]) {
        Ok(request) => request,
        Err(error) => return error.guest_result(),
    };
    let mut path = [0u8; 128];
    if parsed.path.len() > path.len() {
        return w7::Error::RequestSource.guest_result();
    }
    path[..parsed.path.len()].copy_from_slice(parsed.path);
    let request = w7::Request {
        sni: w7::FIXED_SNI,
        path: &path[..parsed.path.len()],
        total_len: parsed.total_len,
        whole_sha256: parsed.whole_sha256,
        chunk_count: parsed.chunk_count,
        chunk_sha256: parsed.chunk_sha256,
    };
    let mut abi = GuestAbi;
    let connection = abi.tcp_open();
    if connection <= 0 {
        return if connection < 0 {
            connection
        } else {
            w7::Error::HostCall.guest_result()
        };
    }
    let result = run(&mut abi, connection, request, buffers);
    let close = abi.tcp_close(connection);
    if result == 0 && close < 0 {
        close
    } else {
        result
    }
}

fn run<A: w7::W7Abi>(
    abi: &mut A,
    connection: i32,
    request: w7::Request<'_>,
    buffers: &mut Buffers,
) -> i32 {
    match run_checked(abi, connection, request, buffers) {
        Ok(()) => 0,
        Err(ServiceError::Pure(error)) => error.guest_result(),
        Err(ServiceError::Host(result)) => result,
    }
}

enum ServiceError {
    Pure(w7::Error),
    Host(i32),
}
impl From<w7::Error> for ServiceError {
    fn from(value: w7::Error) -> Self {
        Self::Pure(value)
    }
}

fn host(result: i32, expected: i32) -> Result<(), ServiceError> {
    if result == expected {
        Ok(())
    } else if result < 0 {
        Err(ServiceError::Host(result))
    } else {
        Err(ServiceError::Pure(w7::Error::HostCall))
    }
}

fn run_checked<A: w7::W7Abi>(
    abi: &mut A,
    connection: i32,
    request: w7::Request<'_>,
    buffers: &mut Buffers,
) -> Result<(), ServiceError> {
    let mut public = [0u8; w7::TLS_PUBLIC_OUTPUT_LEN];
    let session = abi.tls13_session_open(connection, &mut public);
    if session <= 0 {
        return Err(if session < 0 {
            ServiceError::Host(session)
        } else {
            w7::Error::HostCall.into()
        });
    }
    let hello_len = w7::build_client_hello(request.sni, &public, &mut buffers.tx)?;
    let mut transcript_len = hello_len - 5;
    buffers.transcript[..transcript_len].copy_from_slice(&buffers.tx[5..hello_len]);
    send_all(abi, connection, &buffers.tx[..hello_len])?;

    let mut header = [0u8; 5];
    let server_hello_len = recv_record(abi, connection, &mut header, &mut buffers.record)?;
    let record = w7::parse_tls_record_header(&header)?;
    if record.content_type != 22 || record.len != server_hello_len {
        return Err(w7::Error::TlsServerHello.into());
    }
    let (kind, body, frame_len) = w7::handshake_frame(&buffers.record[..server_hello_len])?
        .ok_or(w7::Error::TlsServerHello)?;
    if kind != 2 || frame_len != server_hello_len {
        return Err(w7::Error::TlsServerHello.into());
    }
    let peer_public = w7::parse_server_hello(body)?;
    append(
        &mut buffers.transcript,
        &mut transcript_len,
        &buffers.record[..frame_len],
    )?;
    let mut transcript_hash = [0u8; 32];
    host(
        abi.sha256(
            session,
            &buffers.transcript[..transcript_len],
            &mut transcript_hash,
        ),
        32,
    )?;
    host(
        abi.tls13_handshake_keys(session, &peer_public, &transcript_hash),
        0,
    )?;

    let mut state = w7::HandshakeState::ExpectEncryptedExtensions;
    let mut pending_len = 0usize;
    let mut spki_len = 0usize;
    while state != w7::HandshakeState::Complete {
        let record_len = recv_record(abi, connection, &mut header, &mut buffers.record)?;
        let outer = w7::parse_tls_record_header(&header)?;
        if outer.content_type == 20 && buffers.record[..record_len] == [1] {
            continue;
        }
        if outer.content_type != 23 || outer.len != record_len {
            return Err(w7::Error::TlsRecord.into());
        }
        let opened = abi.tls13_aead_open(
            session,
            &header,
            &buffers.record[..record_len],
            &mut buffers.plain,
        );
        if opened < 0 {
            return Err(ServiceError::Host(opened));
        }
        let (inner_type, content) =
            w7::strip_tls_inner_plaintext(&buffers.plain[..opened as usize])?;
        if inner_type != 22 {
            return Err(w7::Error::TlsHandshake.into());
        }
        append(&mut buffers.handshake, &mut pending_len, content)?;
        loop {
            let Some((kind, body, frame_len)) =
                w7::handshake_frame(&buffers.handshake[..pending_len])?
            else {
                break;
            };
            state.observe(kind)?;
            match kind {
                8 => {
                    w7::validate_encrypted_extensions(body)?;
                    append(
                        &mut buffers.transcript,
                        &mut transcript_len,
                        &buffers.handshake[..frame_len],
                    )?;
                }
                11 => {
                    let cert = w7::leaf_certificate(body)?;
                    let spki = raios_x509_spki::extract_p256_spki(cert)
                        .ok_or(w7::Error::TlsCertificate)?;
                    if spki.der.len() > buffers.spki.len() {
                        return Err(w7::Error::TlsCertificate.into());
                    }
                    spki_len = spki.der.len();
                    buffers.spki[..spki_len].copy_from_slice(spki.der);
                    append(
                        &mut buffers.transcript,
                        &mut transcript_len,
                        &buffers.handshake[..frame_len],
                    )?;
                }
                15 => {
                    let signature = w7::certificate_verify(body)?;
                    if spki_len == 0 {
                        return Err(w7::Error::TlsCertificate.into());
                    }
                    host(
                        abi.sha256(
                            session,
                            &buffers.transcript[..transcript_len],
                            &mut transcript_hash,
                        ),
                        32,
                    )?;
                    let verify_len =
                        w7::certificate_verify_message(&transcript_hash, &mut buffers.verify)?;
                    host(
                        abi.p256_verify(
                            session,
                            &buffers.spki[..spki_len],
                            &buffers.verify[..verify_len],
                            signature,
                        ),
                        1,
                    )?;
                    append(
                        &mut buffers.transcript,
                        &mut transcript_len,
                        &buffers.handshake[..frame_len],
                    )?;
                }
                20 => {
                    let proof = w7::finished_proof(body)?;
                    let mut server_proof = *proof;
                    host(
                        abi.sha256(
                            session,
                            &buffers.transcript[..transcript_len],
                            &mut transcript_hash,
                        ),
                        32,
                    )?;
                    host(
                        abi.tls13_finished(session, 1, &transcript_hash, &mut server_proof),
                        1,
                    )?;
                    append(
                        &mut buffers.transcript,
                        &mut transcript_len,
                        &buffers.handshake[..frame_len],
                    )?;
                    host(
                        abi.sha256(
                            session,
                            &buffers.transcript[..transcript_len],
                            &mut transcript_hash,
                        ),
                        32,
                    )?;
                    host(abi.tls13_application_keys(session, &transcript_hash), 0)?;
                    let mut client_proof = [0u8; 32];
                    host(
                        abi.tls13_finished(session, 0, &transcript_hash, &mut client_proof),
                        32,
                    )?;
                    buffers.plain[..4].copy_from_slice(&[20, 0, 0, 32]);
                    buffers.plain[4..36].copy_from_slice(&client_proof);
                    buffers.plain[36] = 22;
                    send_encrypted(
                        abi,
                        connection,
                        session,
                        &buffers.plain[..37],
                        &mut buffers.record,
                    )?;
                }
                _ => return Err(w7::Error::TlsSequence.into()),
            }
            buffers.handshake.copy_within(frame_len..pending_len, 0);
            pending_len -= frame_len;
        }
    }
    if pending_len != 0 {
        return Err(w7::Error::TlsHandshake.into());
    }

    let get_len = w7::build_fixed_get(&request, &mut buffers.tx)?;
    buffers.tx[get_len] = 23;
    send_encrypted(
        abi,
        connection,
        session,
        &buffers.tx[..get_len + 1],
        &mut buffers.record,
    )?;
    receive_http(abi, connection, session, &request, buffers)
}

fn receive_http<A: w7::W7Abi>(
    abi: &mut A,
    connection: i32,
    session: i32,
    request: &w7::Request<'_>,
    buffers: &mut Buffers,
) -> Result<(), ServiceError> {
    let mut header = [0u8; 5];
    let mut head_len = 0usize;
    let mut head_complete = false;
    let mut chunks = w7::ChunkState::new();
    loop {
        let record_len = recv_record(abi, connection, &mut header, &mut buffers.record)?;
        let outer = w7::parse_tls_record_header(&header)?;
        if outer.content_type != 23 || outer.len != record_len {
            return Err(w7::Error::TlsRecord.into());
        }
        let opened = abi.tls13_aead_open(
            session,
            &header,
            &buffers.record[..record_len],
            &mut buffers.plain,
        );
        if opened < 0 {
            return Err(ServiceError::Host(opened));
        }
        let (inner_type, content) =
            w7::strip_tls_inner_plaintext(&buffers.plain[..opened as usize])?;
        if inner_type == 22 {
            validate_new_session_tickets(content)?;
            continue;
        }
        if inner_type != 23 {
            return Err(w7::Error::TlsRecord.into());
        }
        let mut cursor = 0usize;
        if !head_complete {
            while cursor < content.len() && !head_complete {
                if head_len == buffers.http_head.len() {
                    return Err(w7::Error::HttpHeader.into());
                }
                buffers.http_head[head_len] = content[cursor];
                head_len += 1;
                cursor += 1;
                head_complete =
                    head_len >= 4 && &buffers.http_head[head_len - 4..head_len] == b"\r\n\r\n";
            }
            if head_complete {
                let strict =
                    w7::parse_http_head(&buffers.http_head[..head_len], request.total_len)?;
                let relocated = raios_http_parse::parse_http_head(&buffers.http_head[..head_len]);
                if strict.body_offset != head_len
                    || relocated.status_code != Some(200)
                    || relocated.content_length != Some(request.total_len as u64)
                    || relocated.chunked
                {
                    return Err(w7::Error::HttpHeader.into());
                }
            }
        }
        if head_complete && cursor < content.len() {
            chunks.push(
                abi,
                request.total_len,
                &content[cursor..],
                &mut buffers.chunk,
            )?;
        }
        if head_complete && chunks.total == request.total_len {
            chunks.finish(abi, request.total_len, &mut buffers.chunk)?;
            return Ok(());
        }
    }
}

fn validate_new_session_tickets(mut bytes: &[u8]) -> Result<(), ServiceError> {
    while !bytes.is_empty() {
        let (kind, _, len) = w7::handshake_frame(bytes)?.ok_or(w7::Error::TlsHandshake)?;
        if kind != 4 {
            return Err(w7::Error::TlsSequence.into());
        }
        bytes = &bytes[len..];
    }
    Ok(())
}

fn append(target: &mut [u8], len: &mut usize, bytes: &[u8]) -> Result<(), ServiceError> {
    let end = len
        .checked_add(bytes.len())
        .ok_or(w7::Error::BufferTooSmall)?;
    if end > target.len() {
        return Err(w7::Error::BufferTooSmall.into());
    }
    target[*len..end].copy_from_slice(bytes);
    *len = end;
    Ok(())
}

fn send_all<A: w7::W7Abi>(
    abi: &mut A,
    connection: i32,
    mut bytes: &[u8],
) -> Result<(), ServiceError> {
    while !bytes.is_empty() {
        let take = bytes.len().min(4096);
        let sent = abi.tcp_send(connection, &bytes[..take]);
        if sent <= 0 {
            return Err(ServiceError::Host(sent));
        }
        if sent as usize > take {
            return Err(w7::Error::HostCall.into());
        }
        bytes = &bytes[sent as usize..];
    }
    Ok(())
}

fn recv_exact<A: w7::W7Abi>(
    abi: &mut A,
    connection: i32,
    mut out: &mut [u8],
) -> Result<(), ServiceError> {
    while !out.is_empty() {
        let take = out.len().min(4096);
        let received = abi.tcp_recv(connection, &mut out[..take]);
        if received <= 0 {
            return Err(ServiceError::Host(if received == 0 {
                -5
            } else {
                received
            }));
        }
        if received as usize > take {
            return Err(w7::Error::HostCall.into());
        }
        out = &mut out[received as usize..];
    }
    Ok(())
}

fn recv_record<A: w7::W7Abi>(
    abi: &mut A,
    connection: i32,
    header: &mut [u8; 5],
    body: &mut [u8],
) -> Result<usize, ServiceError> {
    recv_exact(abi, connection, header)?;
    let parsed = w7::parse_tls_record_header(header)?;
    if body.len() < parsed.len {
        return Err(w7::Error::BufferTooSmall.into());
    }
    recv_exact(abi, connection, &mut body[..parsed.len])?;
    Ok(parsed.len)
}

fn send_encrypted<A: w7::W7Abi>(
    abi: &mut A,
    connection: i32,
    session: i32,
    plain: &[u8],
    cipher: &mut [u8],
) -> Result<(), ServiceError> {
    let header = w7::make_record_header(plain.len() + w7::TLS_TAG_LEN)?;
    let sealed = abi.tls13_aead_seal(session, &header, plain, cipher);
    if sealed < 0 {
        return Err(ServiceError::Host(sealed));
    }
    if sealed as usize != plain.len() + w7::TLS_TAG_LEN {
        return Err(w7::Error::TlsRecord.into());
    }
    send_all(abi, connection, &header)?;
    send_all(abi, connection, &cipher[..sealed as usize])
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}
