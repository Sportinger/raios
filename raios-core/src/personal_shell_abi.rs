//! Fixed V1 packets staged for one `svc.user.shell` invocation.
//!
//! These packets deliberately contain only bounded numeric UI state and
//! sanitized input. They carry no strings, pointers, provider output, or
//! secret-bearing values.

pub const PERSONAL_SHELL_ABI_VERSION: u16 = 1;
pub const CONTEXT_LEN: usize = 32;
pub const INPUT_HEADER_LEN: usize = 16;
pub const INPUT_EVENT_LEN: usize = 16;
pub const MAX_INPUT_EVENTS: usize = 64;
pub const MAX_INPUT_LEN: usize = INPUT_HEADER_LEN + MAX_INPUT_EVENTS * INPUT_EVENT_LEN;

const CONTEXT_MAGIC: &[u8; 4] = b"RCTX";
const INPUT_MAGIC: &[u8; 4] = b"RINP";
const CONTEXT_PERSONAL_FOCUS: u16 = 1;
const CONTEXT_RECOVERY_READY: u16 = 1 << 1;
const INPUT_PRESSED: u8 = 1;
const INPUT_REPEAT: u8 = 1 << 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonalShellAbiError {
    WrongAbiVersion,
    Malformed,
    LimitExceeded,
    InvocationIdMismatch,
}

/// Immutable, redacted V1 context for one personal-shell invocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PersonalShellContext {
    pub invocation_id: u32,
    pub viewport_width: u16,
    pub viewport_height: u16,
    pub service_count: u16,
    pub problem_count: u16,
    pub denied_capability_count: u16,
    pub personal_focus: bool,
    pub recovery_ready: bool,
    pub active_task_id: u32,
}

impl PersonalShellContext {
    pub const fn new(
        invocation_id: u32,
        viewport_width: u16,
        viewport_height: u16,
        service_count: u16,
        problem_count: u16,
        denied_capability_count: u16,
        personal_focus: bool,
        recovery_ready: bool,
        active_task_id: u32,
    ) -> Self {
        Self {
            invocation_id,
            viewport_width,
            viewport_height,
            service_count,
            problem_count,
            denied_capability_count,
            personal_focus,
            recovery_ready,
            active_task_id,
        }
    }

    pub fn encode(self) -> [u8; CONTEXT_LEN] {
        let mut bytes = [0u8; CONTEXT_LEN];
        bytes[..4].copy_from_slice(CONTEXT_MAGIC);
        write_u16(&mut bytes, 4, PERSONAL_SHELL_ABI_VERSION);
        write_u16(&mut bytes, 6, CONTEXT_LEN as u16);
        write_u32(&mut bytes, 8, self.invocation_id);
        write_u16(&mut bytes, 12, self.viewport_width);
        write_u16(&mut bytes, 14, self.viewport_height);
        write_u16(&mut bytes, 16, self.service_count);
        write_u16(&mut bytes, 18, self.problem_count);
        write_u16(&mut bytes, 20, self.denied_capability_count);
        let mut flags = 0;
        if self.personal_focus {
            flags |= CONTEXT_PERSONAL_FOCUS;
        }
        if self.recovery_ready {
            flags |= CONTEXT_RECOVERY_READY;
        }
        write_u16(&mut bytes, 22, flags);
        write_u32(&mut bytes, 24, self.active_task_id);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PersonalShellAbiError> {
        if bytes.len() != CONTEXT_LEN || bytes.get(..4) != Some(CONTEXT_MAGIC) {
            return Err(PersonalShellAbiError::Malformed);
        }
        if read_u16(bytes, 4) != Some(PERSONAL_SHELL_ABI_VERSION) {
            return Err(PersonalShellAbiError::WrongAbiVersion);
        }
        if read_u16(bytes, 6) != Some(CONTEXT_LEN as u16) || read_u16(bytes, 28) != Some(0) {
            return Err(PersonalShellAbiError::Malformed);
        }
        let flags = read_u16(bytes, 22).ok_or(PersonalShellAbiError::Malformed)?;
        if flags & !(CONTEXT_PERSONAL_FOCUS | CONTEXT_RECOVERY_READY) != 0 {
            return Err(PersonalShellAbiError::Malformed);
        }

        Ok(Self {
            invocation_id: read_u32(bytes, 8).ok_or(PersonalShellAbiError::Malformed)?,
            viewport_width: read_u16(bytes, 12).ok_or(PersonalShellAbiError::Malformed)?,
            viewport_height: read_u16(bytes, 14).ok_or(PersonalShellAbiError::Malformed)?,
            service_count: read_u16(bytes, 16).ok_or(PersonalShellAbiError::Malformed)?,
            problem_count: read_u16(bytes, 18).ok_or(PersonalShellAbiError::Malformed)?,
            denied_capability_count: read_u16(bytes, 20).ok_or(PersonalShellAbiError::Malformed)?,
            personal_focus: flags & CONTEXT_PERSONAL_FOCUS != 0,
            recovery_ready: flags & CONTEXT_RECOVERY_READY != 0,
            active_task_id: read_u32(bytes, 24).ok_or(PersonalShellAbiError::Malformed)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SanitizedInputKind {
    Key,
    PointerMove,
    PointerButton,
}

impl SanitizedInputKind {
    const fn encode(self) -> u8 {
        match self {
            Self::Key => 1,
            Self::PointerMove => 2,
            Self::PointerButton => 3,
        }
    }

    fn decode(value: u8) -> Result<Self, PersonalShellAbiError> {
        match value {
            1 => Ok(Self::Key),
            2 => Ok(Self::PointerMove),
            3 => Ok(Self::PointerButton),
            _ => Err(PersonalShellAbiError::Malformed),
        }
    }
}

/// One fixed-width sanitized event. There is intentionally no text or raw input
/// payload: all fields are numeric and each input packet is capped at 64 events.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SanitizedInputEvent {
    pub kind: SanitizedInputKind,
    pub pressed: bool,
    pub repeat: bool,
    pub code: u16,
    pub x: i16,
    pub y: i16,
    pub dx: i16,
    pub dy: i16,
    pub modifiers: u16,
}

impl SanitizedInputEvent {
    pub const fn new(
        kind: SanitizedInputKind,
        pressed: bool,
        repeat: bool,
        code: u16,
        x: i16,
        y: i16,
        dx: i16,
        dy: i16,
        modifiers: u16,
    ) -> Self {
        Self {
            kind,
            pressed,
            repeat,
            code,
            x,
            y,
            dx,
            dy,
            modifiers,
        }
    }
}

const EMPTY_EVENT: SanitizedInputEvent = SanitizedInputEvent {
    kind: SanitizedInputKind::Key,
    pressed: false,
    repeat: false,
    code: 0,
    x: 0,
    y: 0,
    dx: 0,
    dy: 0,
    modifiers: 0,
};

/// A no-allocation, bounded V1 input packet for one invocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PersonalShellInput {
    invocation_id: u32,
    events: [SanitizedInputEvent; MAX_INPUT_EVENTS],
    event_count: usize,
}

impl PersonalShellInput {
    pub const fn new(invocation_id: u32) -> Self {
        Self {
            invocation_id,
            events: [EMPTY_EVENT; MAX_INPUT_EVENTS],
            event_count: 0,
        }
    }

    pub const fn invocation_id(&self) -> u32 {
        self.invocation_id
    }

    pub fn events(&self) -> &[SanitizedInputEvent] {
        &self.events[..self.event_count]
    }

    pub fn push(&mut self, event: SanitizedInputEvent) -> Result<(), PersonalShellAbiError> {
        if self.event_count == MAX_INPUT_EVENTS {
            return Err(PersonalShellAbiError::LimitExceeded);
        }
        self.events[self.event_count] = event;
        self.event_count += 1;
        Ok(())
    }

    pub fn encode(&self) -> EncodedPersonalShellInput {
        let mut bytes = [0u8; MAX_INPUT_LEN];
        let len = INPUT_HEADER_LEN + self.event_count * INPUT_EVENT_LEN;
        bytes[..4].copy_from_slice(INPUT_MAGIC);
        write_u16(&mut bytes, 4, PERSONAL_SHELL_ABI_VERSION);
        write_u16(&mut bytes, 6, INPUT_HEADER_LEN as u16);
        write_u32(&mut bytes, 8, self.invocation_id);
        write_u16(&mut bytes, 12, self.event_count as u16);

        let mut index = 0usize;
        while index < self.event_count {
            encode_event(
                &mut bytes[INPUT_HEADER_LEN + index * INPUT_EVENT_LEN..][..INPUT_EVENT_LEN],
                self.events[index],
            );
            index += 1;
        }
        EncodedPersonalShellInput { bytes, len }
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PersonalShellAbiError> {
        if bytes.len() > MAX_INPUT_LEN {
            return Err(PersonalShellAbiError::LimitExceeded);
        }
        if bytes.len() < INPUT_HEADER_LEN || bytes.get(..4) != Some(INPUT_MAGIC) {
            return Err(PersonalShellAbiError::Malformed);
        }
        if read_u16(bytes, 4) != Some(PERSONAL_SHELL_ABI_VERSION) {
            return Err(PersonalShellAbiError::WrongAbiVersion);
        }
        if read_u16(bytes, 6) != Some(INPUT_HEADER_LEN as u16) || read_u16(bytes, 14) != Some(0) {
            return Err(PersonalShellAbiError::Malformed);
        }
        let event_count = read_u16(bytes, 12).ok_or(PersonalShellAbiError::Malformed)? as usize;
        if event_count > MAX_INPUT_EVENTS {
            return Err(PersonalShellAbiError::LimitExceeded);
        }
        let expected_len = INPUT_HEADER_LEN + event_count * INPUT_EVENT_LEN;
        if bytes.len() != expected_len {
            return Err(PersonalShellAbiError::Malformed);
        }

        let mut input = Self::new(read_u32(bytes, 8).ok_or(PersonalShellAbiError::Malformed)?);
        let mut index = 0usize;
        while index < event_count {
            let offset = INPUT_HEADER_LEN + index * INPUT_EVENT_LEN;
            input.push(decode_event(&bytes[offset..offset + INPUT_EVENT_LEN])?)?;
            index += 1;
        }
        Ok(input)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EncodedPersonalShellInput {
    bytes: [u8; MAX_INPUT_LEN],
    len: usize,
}

impl EncodedPersonalShellInput {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

pub fn validate_invocation_pair(
    context: &PersonalShellContext,
    input: &PersonalShellInput,
) -> Result<(), PersonalShellAbiError> {
    if context.invocation_id == input.invocation_id {
        Ok(())
    } else {
        Err(PersonalShellAbiError::InvocationIdMismatch)
    }
}

fn encode_event(bytes: &mut [u8], event: SanitizedInputEvent) {
    bytes[0] = event.kind.encode();
    if event.pressed {
        bytes[1] |= INPUT_PRESSED;
    }
    if event.repeat {
        bytes[1] |= INPUT_REPEAT;
    }
    write_u16(bytes, 2, event.code);
    write_i16(bytes, 4, event.x);
    write_i16(bytes, 6, event.y);
    write_i16(bytes, 8, event.dx);
    write_i16(bytes, 10, event.dy);
    write_u16(bytes, 12, event.modifiers);
}

fn decode_event(bytes: &[u8]) -> Result<SanitizedInputEvent, PersonalShellAbiError> {
    if bytes.len() != INPUT_EVENT_LEN || bytes[1] & !(INPUT_PRESSED | INPUT_REPEAT) != 0 {
        return Err(PersonalShellAbiError::Malformed);
    }
    if read_u16(bytes, 14) != Some(0) {
        return Err(PersonalShellAbiError::Malformed);
    }
    Ok(SanitizedInputEvent {
        kind: SanitizedInputKind::decode(bytes[0])?,
        pressed: bytes[1] & INPUT_PRESSED != 0,
        repeat: bytes[1] & INPUT_REPEAT != 0,
        code: read_u16(bytes, 2).ok_or(PersonalShellAbiError::Malformed)?,
        x: read_i16(bytes, 4).ok_or(PersonalShellAbiError::Malformed)?,
        y: read_i16(bytes, 6).ok_or(PersonalShellAbiError::Malformed)?,
        dx: read_i16(bytes, 8).ok_or(PersonalShellAbiError::Malformed)?,
        dy: read_i16(bytes, 10).ok_or(PersonalShellAbiError::Malformed)?,
        modifiers: read_u16(bytes, 12).ok_or(PersonalShellAbiError::Malformed)?,
    })
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let raw = bytes.get(offset..offset.checked_add(2)?)?;
    Some(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_i16(bytes: &[u8], offset: usize) -> Option<i16> {
    let raw = bytes.get(offset..offset.checked_add(2)?)?;
    Some(i16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let raw = bytes.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_i16(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> PersonalShellContext {
        PersonalShellContext::new(0x4433_2211, 800, 600, 3, 4, 5, true, true, 0x8877_6655)
    }

    fn event() -> SanitizedInputEvent {
        SanitizedInputEvent::new(
            SanitizedInputKind::PointerButton,
            true,
            true,
            0x3412,
            -2,
            3,
            -4,
            5,
            0x7856,
        )
    }

    #[test]
    fn context_v1_bytes_are_exact_and_round_trip() {
        let bytes = context().encode();
        assert_eq!(
            bytes,
            [
                b'R', b'C', b'T', b'X', 1, 0, 32, 0, 0x11, 0x22, 0x33, 0x44, 0x20, 3, 0x58, 2, 3,
                0, 4, 0, 5, 0, 3, 0, 0x55, 0x66, 0x77, 0x88, 0, 0, 0, 0,
            ]
        );
        assert_eq!(PersonalShellContext::decode(&bytes), Ok(context()));
    }

    #[test]
    fn input_v1_bytes_are_exact_and_round_trip() {
        let mut input = PersonalShellInput::new(0x4433_2211);
        input.push(event()).unwrap();
        let bytes = input.encode();
        assert_eq!(
            bytes.as_bytes(),
            &[
                b'R', b'I', b'N', b'P', 1, 0, 16, 0, 0x11, 0x22, 0x33, 0x44, 1, 0, 0, 0, 3, 3,
                0x12, 0x34, 0xfe, 0xff, 3, 0, 0xfc, 0xff, 5, 0, 0x56, 0x78, 0, 0,
            ]
        );
        assert_eq!(PersonalShellInput::decode(bytes.as_bytes()), Ok(input));
    }

    #[test]
    fn reserved_bits_and_fields_fail_closed() {
        let mut context_bytes = context().encode();
        context_bytes[22] = 0x80;
        assert_eq!(
            PersonalShellContext::decode(&context_bytes),
            Err(PersonalShellAbiError::Malformed)
        );
        context_bytes = context().encode();
        context_bytes[28] = 1;
        assert_eq!(
            PersonalShellContext::decode(&context_bytes),
            Err(PersonalShellAbiError::Malformed)
        );

        let mut input = PersonalShellInput::new(context().invocation_id);
        input.push(event()).unwrap();
        assert_eq!(validate_invocation_pair(&context(), &input), Ok(()));
        let mut input_bytes = input.encode();
        input_bytes.bytes[14] = 1;
        assert_eq!(
            PersonalShellInput::decode(input_bytes.as_bytes()),
            Err(PersonalShellAbiError::Malformed)
        );
        input_bytes = input.encode();
        input_bytes.bytes[17] = 0x80;
        assert_eq!(
            PersonalShellInput::decode(input_bytes.as_bytes()),
            Err(PersonalShellAbiError::Malformed)
        );
        input_bytes = input.encode();
        input_bytes.bytes[30] = 1;
        assert_eq!(
            PersonalShellInput::decode(input_bytes.as_bytes()),
            Err(PersonalShellAbiError::Malformed)
        );
    }

    #[test]
    fn wrong_versions_and_invocation_mismatch_fail_closed() {
        let mut context_bytes = context().encode();
        context_bytes[4] = 2;
        assert_eq!(
            PersonalShellContext::decode(&context_bytes),
            Err(PersonalShellAbiError::WrongAbiVersion)
        );

        let mut input = PersonalShellInput::new(context().invocation_id);
        input.push(event()).unwrap();
        let mut input_bytes = input.encode();
        input_bytes.bytes[4] = 2;
        assert_eq!(
            PersonalShellInput::decode(input_bytes.as_bytes()),
            Err(PersonalShellAbiError::WrongAbiVersion)
        );

        let other_input = PersonalShellInput::new(context().invocation_id + 1);
        assert_eq!(
            validate_invocation_pair(&context(), &other_input),
            Err(PersonalShellAbiError::InvocationIdMismatch)
        );
    }

    #[test]
    fn input_is_bounded_to_exactly_sixty_four_events() {
        let mut input = PersonalShellInput::new(context().invocation_id);
        for _ in 0..MAX_INPUT_EVENTS {
            input.push(event()).unwrap();
        }
        assert_eq!(input.encode().as_bytes().len(), MAX_INPUT_LEN);
        assert_eq!(
            PersonalShellInput::decode(input.encode().as_bytes()),
            Ok(input)
        );
        assert_eq!(
            input.push(event()),
            Err(PersonalShellAbiError::LimitExceeded)
        );
        let mut oversized_count = input.encode();
        oversized_count.bytes[12] = (MAX_INPUT_EVENTS + 1) as u8;
        assert_eq!(
            PersonalShellInput::decode(oversized_count.as_bytes()),
            Err(PersonalShellAbiError::LimitExceeded)
        );
    }

    #[test]
    fn packets_have_no_secret_value_channel() {
        let context_bytes = context().encode();
        let mut input = PersonalShellInput::new(context().invocation_id);
        input.push(event()).unwrap();
        let input_bytes = input.encode();

        // The exact V1 packets contain only fixed numeric fields and zeroed
        // reserved bytes; no variable text, pointer, or opaque payload exists.
        assert_eq!(context_bytes.len(), CONTEXT_LEN);
        assert_eq!(
            input_bytes.as_bytes().len(),
            INPUT_HEADER_LEN + INPUT_EVENT_LEN
        );
        assert!(context_bytes[28..].iter().all(|byte| *byte == 0));
        assert!(input_bytes.as_bytes()[14..16].iter().all(|byte| *byte == 0));
        assert!(input_bytes.as_bytes()[30..32].iter().all(|byte| *byte == 0));
    }
}
