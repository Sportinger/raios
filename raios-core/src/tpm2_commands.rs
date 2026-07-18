//! Bounded TPM 2.0 command encoding and response-header validation.
//!
//! This module is deliberately transport- and policy-free.  It can encode only
//! the commands the Vault keyring needs; callers cannot construct arbitrary
//! TPM command bytes through this API.

pub const TPM2_COMMAND_HEADER_LEN: usize = 10;
pub const TPM2_RESPONSE_HEADER_LEN: usize = 10;
pub const MAX_TPM2_COMMAND_BYTES: usize = 4096;
pub const MAX_TPM2_RESPONSE_BYTES: usize = 4096;
pub const MAX_TPM2B_BYTES: usize = 2048;
pub const MAX_TPM2_AUTH_SESSIONS: usize = 3;
pub const MAX_TPM2_AUTH_BYTES: usize = 128;
pub const MAX_TPM2_PCR_SELECTIONS: usize = 16;
pub const MAX_TPM2_PCR_SELECT_BYTES: usize = 4;

pub const TPM2_ST_NO_SESSIONS: u16 = 0x8001;
pub const TPM2_ST_SESSIONS: u16 = 0x8002;
pub const TPM2_RC_SUCCESS: u32 = 0;
pub const TPM2_RH_NULL: u32 = 0x4000_0007;
pub const TPM2_ALG_NULL: u16 = 0x0010;
pub const TPM2_ALG_SHA256: u16 = 0x000b;
pub const TPM2_SE_POLICY: u8 = 0x01;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Tpm2CommandCode {
    Create = 0x0000_0153,
    Load = 0x0000_0157,
    Unseal = 0x0000_015e,
    FlushContext = 0x0000_0165,
    StartAuthSession = 0x0000_0176,
    PolicyPcr = 0x0000_017f,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tpm2CodecError {
    BufferTooLarge,
    BufferLengthOverflow,
    CommandTooLarge,
    SessionListEmpty,
    TooManySessions,
    SessionTooLarge,
    InvalidPcrSelection,
    ResponseTooShort,
    ResponseTooLarge,
    ResponseSizeMismatch,
    InvalidResponseTag,
    ResponseNotSuccessful { code: u32 },
    InvalidUnsealResponse,
}

/// A validated TPM2B value.  Its bytes are written with a canonical u16
/// big-endian length prefix by every command encoder below.
#[derive(Clone, Copy)]
pub struct Tpm2Buffer<'a> {
    bytes: &'a [u8],
}

impl<'a> Tpm2Buffer<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, Tpm2CodecError> {
        if bytes.len() > MAX_TPM2B_BYTES || bytes.len() > u16::MAX as usize {
            return Err(Tpm2CodecError::BufferTooLarge);
        }
        Ok(Self { bytes })
    }

    pub const fn as_bytes(self) -> &'a [u8] {
        self.bytes
    }
}

/// One bounded TPMS_AUTH_COMMAND.  The keyring supplies a policy session or
/// the TPM password session; no other command surface is exposed here.
#[derive(Clone, Copy)]
pub struct Tpm2AuthSession<'a> {
    pub handle: u32,
    pub nonce: &'a [u8],
    pub attributes: u8,
    pub hmac: &'a [u8],
}

impl<'a> Tpm2AuthSession<'a> {
    fn validate(self) -> Result<(), Tpm2CodecError> {
        if self.nonce.len() > MAX_TPM2_AUTH_BYTES || self.hmac.len() > MAX_TPM2_AUTH_BYTES {
            return Err(Tpm2CodecError::SessionTooLarge);
        }
        Ok(())
    }
}

/// A canonical TPML_PCR_SELECTION byte sequence.
#[derive(Clone, Copy)]
pub struct Tpm2PcrSelections<'a> {
    encoded: &'a [u8],
}

impl<'a> Tpm2PcrSelections<'a> {
    pub fn new(encoded: &'a [u8]) -> Result<Self, Tpm2CodecError> {
        validate_pcr_selections(encoded)?;
        Ok(Self { encoded })
    }

    pub const fn as_bytes(self) -> &'a [u8] {
        self.encoded
    }
}

pub struct Tpm2Create<'a> {
    pub parent_handle: u32,
    pub sessions: &'a [Tpm2AuthSession<'a>],
    pub sensitive_create: Tpm2Buffer<'a>,
    pub public: Tpm2Buffer<'a>,
    pub outside_info: Tpm2Buffer<'a>,
    pub creation_pcr: Tpm2PcrSelections<'a>,
}

pub struct Tpm2Load<'a> {
    pub parent_handle: u32,
    pub sessions: &'a [Tpm2AuthSession<'a>],
    pub private: Tpm2Buffer<'a>,
    pub public: Tpm2Buffer<'a>,
}

pub struct Tpm2StartPolicySession<'a> {
    pub nonce_caller: Tpm2Buffer<'a>,
    pub encrypted_salt: Tpm2Buffer<'a>,
    pub auth_hash: u16,
}

pub struct Tpm2PolicyPcr<'a> {
    pub policy_session: u32,
    pub pcr_digest: Tpm2Buffer<'a>,
    pub pcrs: Tpm2PcrSelections<'a>,
}

pub struct Tpm2Unseal<'a> {
    pub item_handle: u32,
    pub sessions: &'a [Tpm2AuthSession<'a>],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tpm2ResponseHeader {
    pub tag: u16,
    pub size: usize,
    pub code: u32,
}

/// A fixed-capacity command produced only by the typed constructors below.
pub struct Tpm2CommandBuffer {
    bytes: [u8; MAX_TPM2_COMMAND_BYTES],
    len: usize,
}

impl Tpm2CommandBuffer {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Drop for Tpm2CommandBuffer {
    fn drop(&mut self) {
        self.bytes.fill(0);
        self.len = 0;
    }
}

pub fn encode_create(request: Tpm2Create<'_>) -> Result<Tpm2CommandBuffer, Tpm2CodecError> {
    let mut writer = CommandWriter::new(TPM2_ST_SESSIONS, Tpm2CommandCode::Create);
    writer.push_u32(request.parent_handle)?;
    writer.push_sessions(request.sessions)?;
    writer.push_tpm2b(request.sensitive_create)?;
    writer.push_tpm2b(request.public)?;
    writer.push_tpm2b(request.outside_info)?;
    writer.push_bytes(request.creation_pcr.as_bytes())?;
    writer.finish()
}

pub fn encode_load(request: Tpm2Load<'_>) -> Result<Tpm2CommandBuffer, Tpm2CodecError> {
    let mut writer = CommandWriter::new(TPM2_ST_SESSIONS, Tpm2CommandCode::Load);
    writer.push_u32(request.parent_handle)?;
    writer.push_sessions(request.sessions)?;
    writer.push_tpm2b(request.private)?;
    writer.push_tpm2b(request.public)?;
    writer.finish()
}

pub fn encode_start_policy_session(
    request: Tpm2StartPolicySession<'_>,
) -> Result<Tpm2CommandBuffer, Tpm2CodecError> {
    let mut writer = CommandWriter::new(TPM2_ST_NO_SESSIONS, Tpm2CommandCode::StartAuthSession);
    writer.push_u32(TPM2_RH_NULL)?;
    writer.push_u32(TPM2_RH_NULL)?;
    writer.push_tpm2b(request.nonce_caller)?;
    writer.push_tpm2b(request.encrypted_salt)?;
    writer.push_u8(TPM2_SE_POLICY)?;
    writer.push_u16(TPM2_ALG_NULL)?;
    writer.push_u16(request.auth_hash)?;
    writer.finish()
}

pub fn encode_policy_pcr(request: Tpm2PolicyPcr<'_>) -> Result<Tpm2CommandBuffer, Tpm2CodecError> {
    let mut writer = CommandWriter::new(TPM2_ST_NO_SESSIONS, Tpm2CommandCode::PolicyPcr);
    writer.push_u32(request.policy_session)?;
    writer.push_tpm2b(request.pcr_digest)?;
    writer.push_bytes(request.pcrs.as_bytes())?;
    writer.finish()
}

pub fn encode_unseal(request: Tpm2Unseal<'_>) -> Result<Tpm2CommandBuffer, Tpm2CodecError> {
    let mut writer = CommandWriter::new(TPM2_ST_SESSIONS, Tpm2CommandCode::Unseal);
    writer.push_u32(request.item_handle)?;
    writer.push_sessions(request.sessions)?;
    writer.finish()
}

pub fn encode_flush_context(handle: u32) -> Result<Tpm2CommandBuffer, Tpm2CodecError> {
    let mut writer = CommandWriter::new(TPM2_ST_NO_SESSIONS, Tpm2CommandCode::FlushContext);
    writer.push_u32(handle)?;
    writer.finish()
}

/// Validates the TPM response header against the exact byte slice read from a
/// bounded transport.  It intentionally exposes metadata only, not parameters.
pub fn validate_response_header(response: &[u8]) -> Result<Tpm2ResponseHeader, Tpm2CodecError> {
    if response.len() < TPM2_RESPONSE_HEADER_LEN {
        return Err(Tpm2CodecError::ResponseTooShort);
    }
    if response.len() > MAX_TPM2_RESPONSE_BYTES {
        return Err(Tpm2CodecError::ResponseTooLarge);
    }
    let tag = read_u16(response, 0).ok_or(Tpm2CodecError::ResponseTooShort)?;
    if tag != TPM2_ST_NO_SESSIONS && tag != TPM2_ST_SESSIONS {
        return Err(Tpm2CodecError::InvalidResponseTag);
    }
    let size = read_u32(response, 2).ok_or(Tpm2CodecError::ResponseTooShort)? as usize;
    if !(TPM2_RESPONSE_HEADER_LEN..=MAX_TPM2_RESPONSE_BYTES).contains(&size)
        || size != response.len()
    {
        return Err(Tpm2CodecError::ResponseSizeMismatch);
    }
    let code = read_u32(response, 6).ok_or(Tpm2CodecError::ResponseTooShort)?;
    Ok(Tpm2ResponseHeader { tag, size, code })
}

/// Copies exactly one successful TPM2B_SENSITIVE_DATA VMK response into a
/// fixed caller buffer.  No length-flexible secret response is accepted.
pub fn copy_successful_unseal_vmk(
    response: &[u8],
    output: &mut [u8; 32],
) -> Result<(), Tpm2CodecError> {
    output.fill(0);
    let header = validate_response_header(response)?;
    if header.code != TPM2_RC_SUCCESS {
        return Err(Tpm2CodecError::ResponseNotSuccessful { code: header.code });
    }
    let body = &response[TPM2_RESPONSE_HEADER_LEN..];
    if body.len() < 34 || read_u16(body, 0) != Some(32) {
        return Err(Tpm2CodecError::InvalidUnsealResponse);
    }
    output.copy_from_slice(&body[2..34]);
    Ok(())
}

struct CommandWriter {
    bytes: [u8; MAX_TPM2_COMMAND_BYTES],
    len: usize,
}

impl CommandWriter {
    fn new(tag: u16, code: Tpm2CommandCode) -> Self {
        let mut bytes = [0u8; MAX_TPM2_COMMAND_BYTES];
        bytes[..2].copy_from_slice(&tag.to_be_bytes());
        bytes[6..10].copy_from_slice(&(code as u32).to_be_bytes());
        Self {
            bytes,
            len: TPM2_COMMAND_HEADER_LEN,
        }
    }

    fn push_u8(&mut self, value: u8) -> Result<(), Tpm2CodecError> {
        self.push_bytes(&[value])
    }

    fn push_u16(&mut self, value: u16) -> Result<(), Tpm2CodecError> {
        self.push_bytes(&value.to_be_bytes())
    }

    fn push_u32(&mut self, value: u32) -> Result<(), Tpm2CodecError> {
        self.push_bytes(&value.to_be_bytes())
    }

    fn push_tpm2b(&mut self, value: Tpm2Buffer<'_>) -> Result<(), Tpm2CodecError> {
        let len = u16::try_from(value.bytes.len()).map_err(|_| Tpm2CodecError::BufferTooLarge)?;
        self.push_u16(len)?;
        self.push_bytes(value.bytes)
    }

    fn push_sessions(&mut self, sessions: &[Tpm2AuthSession<'_>]) -> Result<(), Tpm2CodecError> {
        if sessions.is_empty() {
            return Err(Tpm2CodecError::SessionListEmpty);
        }
        if sessions.len() > MAX_TPM2_AUTH_SESSIONS {
            return Err(Tpm2CodecError::TooManySessions);
        }
        let size_offset = self.len;
        self.push_u32(0)?;
        let before = self.len;
        for session in sessions {
            session.validate()?;
            self.push_u32(session.handle)?;
            self.push_tpm2b(Tpm2Buffer::new(session.nonce)?)?;
            self.push_u8(session.attributes)?;
            self.push_tpm2b(Tpm2Buffer::new(session.hmac)?)?;
        }
        let size = self
            .len
            .checked_sub(before)
            .ok_or(Tpm2CodecError::BufferLengthOverflow)?;
        let size = u32::try_from(size).map_err(|_| Tpm2CodecError::BufferLengthOverflow)?;
        self.bytes[size_offset..size_offset + 4].copy_from_slice(&size.to_be_bytes());
        Ok(())
    }

    fn push_bytes(&mut self, bytes: &[u8]) -> Result<(), Tpm2CodecError> {
        let end = self
            .len
            .checked_add(bytes.len())
            .ok_or(Tpm2CodecError::BufferLengthOverflow)?;
        if end > MAX_TPM2_COMMAND_BYTES {
            return Err(Tpm2CodecError::CommandTooLarge);
        }
        self.bytes[self.len..end].copy_from_slice(bytes);
        self.len = end;
        Ok(())
    }

    fn finish(mut self) -> Result<Tpm2CommandBuffer, Tpm2CodecError> {
        let size = u32::try_from(self.len).map_err(|_| Tpm2CodecError::CommandTooLarge)?;
        self.bytes[2..6].copy_from_slice(&size.to_be_bytes());
        Ok(Tpm2CommandBuffer {
            bytes: self.bytes,
            len: self.len,
        })
    }
}

fn validate_pcr_selections(encoded: &[u8]) -> Result<(), Tpm2CodecError> {
    let count = read_u32(encoded, 0).ok_or(Tpm2CodecError::InvalidPcrSelection)? as usize;
    if count > MAX_TPM2_PCR_SELECTIONS {
        return Err(Tpm2CodecError::InvalidPcrSelection);
    }
    let mut offset = 4usize;
    for _ in 0..count {
        let select_len = *encoded
            .get(
                offset
                    .checked_add(2)
                    .ok_or(Tpm2CodecError::InvalidPcrSelection)?,
            )
            .ok_or(Tpm2CodecError::InvalidPcrSelection)? as usize;
        if select_len > MAX_TPM2_PCR_SELECT_BYTES {
            return Err(Tpm2CodecError::InvalidPcrSelection);
        }
        offset = offset
            .checked_add(3)
            .and_then(|value| value.checked_add(select_len))
            .ok_or(Tpm2CodecError::InvalidPcrSelection)?;
        if offset > encoded.len() {
            return Err(Tpm2CodecError::InvalidPcrSelection);
        }
    }
    if offset != encoded.len() {
        return Err(Tpm2CodecError::InvalidPcrSelection);
    }
    Ok(())
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let value = bytes.get(offset..offset.checked_add(2)?)?;
    Some(u16::from_be_bytes([value[0], value[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let value = bytes.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_be_bytes([value[0], value[1], value[2], value[3]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY_PCRS: [u8; 4] = [0, 0, 0, 0];
    const PASSWORD_SESSION: Tpm2AuthSession<'static> = Tpm2AuthSession {
        handle: 0x4000_0009,
        nonce: &[],
        attributes: 0,
        hmac: &[],
    };

    #[test]
    fn create_has_a_canonical_header_and_sessions_area() {
        let pcrs = Tpm2PcrSelections::new(&EMPTY_PCRS).unwrap();
        let command = encode_create(Tpm2Create {
            parent_handle: 0x8100_0000,
            sessions: &[PASSWORD_SESSION],
            sensitive_create: Tpm2Buffer::new(&[0, 0, 0, 0]).unwrap(),
            public: Tpm2Buffer::new(&[1, 2, 3]).unwrap(),
            outside_info: Tpm2Buffer::new(&[]).unwrap(),
            creation_pcr: pcrs,
        })
        .unwrap();

        let bytes = command.as_bytes();
        assert_eq!(read_u16(bytes, 0), Some(TPM2_ST_SESSIONS));
        assert_eq!(read_u32(bytes, 2), Some(command.len() as u32));
        assert_eq!(read_u32(bytes, 6), Some(Tpm2CommandCode::Create as u32));
        assert_eq!(read_u32(bytes, 14), Some(9));
    }

    #[test]
    fn pcr_selection_requires_exact_consumption_and_bounds() {
        assert!(Tpm2PcrSelections::new(&[0, 0, 0, 1, 0, 0x0b, 3, 1, 2, 3]).is_ok());
        assert!(matches!(
            Tpm2PcrSelections::new(&[0, 0, 0, 1, 0, 0x0b, 5, 1, 2, 3, 4, 5]),
            Err(Tpm2CodecError::InvalidPcrSelection)
        ));
        assert!(matches!(
            Tpm2PcrSelections::new(&[0, 0, 0, 0, 0]),
            Err(Tpm2CodecError::InvalidPcrSelection)
        ));
    }

    #[test]
    fn response_header_refuses_mismatch_and_unseal_requires_exact_vmk() {
        let mut response = [0u8; 44];
        response[..2].copy_from_slice(&TPM2_ST_NO_SESSIONS.to_be_bytes());
        response[2..6].copy_from_slice(&44u32.to_be_bytes());
        response[6..10].copy_from_slice(&TPM2_RC_SUCCESS.to_be_bytes());
        response[10..12].copy_from_slice(&32u16.to_be_bytes());
        response[12..44].copy_from_slice(&[0x5a; 32]);
        let mut vmk = [0u8; 32];
        copy_successful_unseal_vmk(&response, &mut vmk).unwrap();
        assert_eq!(vmk, [0x5a; 32]);

        response[2..6].copy_from_slice(&43u32.to_be_bytes());
        assert_eq!(
            validate_response_header(&response),
            Err(Tpm2CodecError::ResponseSizeMismatch)
        );
    }

    #[test]
    fn encoders_reject_oversized_values_and_missing_sessions() {
        let oversized = [0u8; MAX_TPM2B_BYTES + 1];
        assert!(matches!(
            Tpm2Buffer::new(&oversized),
            Err(Tpm2CodecError::BufferTooLarge)
        ));
        assert!(matches!(
            encode_unseal(Tpm2Unseal {
                item_handle: 0x8000_0000,
                sessions: &[],
            }),
            Err(Tpm2CodecError::SessionListEmpty)
        ));
    }
}
