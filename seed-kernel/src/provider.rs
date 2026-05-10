use core::fmt::{self, Write};
use core::str;

use crate::{net, openai, provider_config};

const LINE_CAPACITY: usize = 104;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Route {
    OpenAiDirect,
}

impl Route {
    pub fn as_str(self) -> &'static str {
        match self {
            Route::OpenAiDirect => "OPENAI DIRECT",
        }
    }
}

#[derive(Clone, Copy)]
pub struct AgentRequest<'a> {
    pub prompt: &'a str,
    pub model: Option<&'a str>,
    pub max_output: Option<u16>,
}

impl<'a> AgentRequest<'a> {
    pub fn text(prompt: &'a str) -> Self {
        Self {
            prompt,
            model: None,
            max_output: None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Submitted {
    pub route: Route,
    pub id: u32,
}

#[derive(Clone, Copy)]
pub enum SubmitError {
    Empty,
    MissingApiKey,
    Busy { route: Route, id: u32 },
}

#[derive(Clone, Copy)]
pub struct Event {
    pub route: Route,
    pub line: FixedLine,
}

#[derive(Clone, Copy)]
pub struct FixedLine {
    bytes: [u8; LINE_CAPACITY],
    len: usize,
}

impl FixedLine {
    const fn empty() -> Self {
        Self {
            bytes: [0; LINE_CAPACITY],
            len: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }

    fn set_from_str(&mut self, value: &str) {
        self.len = 0;
        push_str_truncated(&mut self.bytes, &mut self.len, value);
    }
}

impl Write for FixedLine {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        push_str_truncated(&mut self.bytes, &mut self.len, s);
        Ok(())
    }
}

fn push_str_truncated(bytes: &mut [u8], len: &mut usize, value: &str) {
    for ch in value.chars() {
        let char_len = ch.len_utf8();
        if (*len).saturating_add(char_len) > bytes.len() {
            break;
        }
        ch.encode_utf8(&mut bytes[*len..*len + char_len]);
        *len += char_len;
    }
}

#[derive(Clone, Copy)]
pub struct Snapshot {
    pub provider_name: &'static str,
    pub api_key_set: bool,
    pub route: Route,
    pub direct_phase: &'static str,
    pub direct_pending_id: Option<u32>,
    pub direct_last_request_id: Option<u32>,
    pub direct_last_prompt: FixedLine,
    pub direct_last_event: FixedLine,
    pub direct_last_error: FixedLine,
    pub direct_endpoint: &'static str,
    pub direct_model: &'static str,
    pub tcp: Option<net::TcpSnapshot>,
}

pub fn submit_text(prompt: &str) -> Result<Submitted, SubmitError> {
    submit(AgentRequest::text(prompt))
}

pub fn submit(request: AgentRequest<'_>) -> Result<Submitted, SubmitError> {
    let prompt = request.prompt.trim();
    let _model = request.model;
    let _max_output = request.max_output;
    if prompt.is_empty() {
        return Err(SubmitError::Empty);
    }

    if !provider_config::api_key_set() {
        return Err(SubmitError::MissingApiKey);
    }

    match openai::submit_request(prompt) {
        Ok(id) => Ok(Submitted {
            route: Route::OpenAiDirect,
            id,
        }),
        Err(openai::SubmitError::Empty) => Err(SubmitError::Empty),
        Err(openai::SubmitError::Busy(id)) => Err(SubmitError::Busy {
            route: Route::OpenAiDirect,
            id,
        }),
    }
}

pub fn poll() -> Option<Event> {
    openai::poll().map(|line| {
        let mut event_line = FixedLine::empty();
        event_line.set_from_str(line.as_str());
        Event {
            route: Route::OpenAiDirect,
            line: event_line,
        }
    })
}

pub fn snapshot() -> Snapshot {
    let config = provider_config::snapshot();
    let direct = openai::snapshot();
    let mut direct_last_prompt = FixedLine::empty();
    let mut direct_last_event = FixedLine::empty();
    let mut direct_last_error = FixedLine::empty();

    direct_last_prompt.set_from_str(direct.last_prompt.as_str());
    direct_last_event.set_from_str(direct.last_event.as_str());
    direct_last_error.set_from_str(direct.last_error.as_str());

    Snapshot {
        provider_name: config.provider_name,
        api_key_set: config.api_key_set,
        route: Route::OpenAiDirect,
        direct_phase: direct.phase,
        direct_pending_id: direct.pending_id,
        direct_last_request_id: direct.last_request_id,
        direct_last_prompt,
        direct_last_event,
        direct_last_error,
        direct_endpoint: direct.endpoint,
        direct_model: direct.model,
        tcp: net::tcp_snapshot(),
    }
}
