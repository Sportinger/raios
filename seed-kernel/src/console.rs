use core::fmt::{self, Write};
use core::str;

use spin::Mutex;

use crate::{agent_protocol, input, provider, provider_config, serial, system_status, ui, wifi};

const COMMAND_WIDTH: usize = 1536;
const OUTPUT_WIDTH: usize = 104;
const OUTPUT_LINES: usize = 8;
const CHAT_LINES: usize = 10;
const MAX_BYTES_PER_POLL: usize = 64;

static CONSOLE: Mutex<ConsoleState> = Mutex::new(ConsoleState::new());

#[derive(Clone, Copy)]
pub struct ConsoleLine {
    bytes: [u8; OUTPUT_WIDTH],
    len: usize,
}

impl ConsoleLine {
    pub const fn empty() -> Self {
        Self {
            bytes: [0; OUTPUT_WIDTH],
            len: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }

    fn trimmed_bounds(&self) -> (usize, usize) {
        let mut start = 0usize;
        let mut end = self.len;
        while start < end && self.bytes[start].is_ascii_whitespace() {
            start += 1;
        }
        while end > start && self.bytes[end - 1].is_ascii_whitespace() {
            end -= 1;
        }

        (start, end)
    }

    fn trimmed_str(&self) -> &str {
        let (start, end) = self.trimmed_bounds();
        unsafe { str::from_utf8_unchecked(&self.bytes[start..end]) }
    }
}

impl Write for ConsoleLine {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        push_str_truncated(&mut self.bytes, &mut self.len, s);
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct ConsoleSnapshot {
    pub lines: [ConsoleLine; OUTPUT_LINES],
    pub input: ConsoleLine,
    pub view: UiView,
    pub focus: UiFocus,
    pub chat_lines: [ChatLine; CHAT_LINES],
    pub chat_input: ConsoleLine,
    pub settings_entry_active: bool,
    pub api_key_set: bool,
    pub provider_name: &'static str,
    pub provider_phase: &'static str,
    pub provider_model: &'static str,
    pub wifi_ssid: wifi::WifiSsid,
    pub wifi_passphrase_set: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UiView {
    Ai,
    Console,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UiFocus {
    NavAi,
    NavConsole,
    NavSettings,
    ChatInput,
    ConsoleInput,
    SettingsProvider,
    SettingsApiKey,
    SettingsClear,
    SettingsWifiSsid,
    SettingsWifiPassphrase,
    SettingsWifiClear,
    SettingsClose,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChatSpeaker {
    User,
    Assistant,
    System,
}

#[derive(Clone, Copy)]
pub struct ChatLine {
    pub speaker: ChatSpeaker,
    pub text: ConsoleLine,
}

impl ChatLine {
    const fn empty() -> Self {
        Self {
            speaker: ChatSpeaker::System,
            text: ConsoleLine::empty(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConsoleMode {
    Command,
    SetupMenu,
    ApiKeyEntry,
    WifiSsidEntry,
    WifiPassphraseEntry,
}

struct ConsoleState {
    view: UiView,
    focus: UiFocus,
    mode: ConsoleMode,
    input: CommandLine,
    serial_input: CommandLine,
    lines: [ConsoleLine; OUTPUT_LINES],
    next_line: usize,
    line_count: usize,
    chat_lines: [ChatLine; CHAT_LINES],
    chat_next: usize,
    chat_count: usize,
}

impl ConsoleState {
    const fn new() -> Self {
        Self {
            view: UiView::Ai,
            focus: UiFocus::ChatInput,
            mode: ConsoleMode::Command,
            input: CommandLine::new(),
            serial_input: CommandLine::new(),
            lines: [ConsoleLine::empty(); OUTPUT_LINES],
            next_line: 0,
            line_count: 0,
            chat_lines: [ChatLine::empty(); CHAT_LINES],
            chat_next: 0,
            chat_count: 0,
        }
    }

    fn push_line(&mut self, line: ConsoleLine) {
        self.lines[self.next_line] = line;
        self.next_line = (self.next_line + 1) % OUTPUT_LINES;
        self.line_count = usize::min(self.line_count + 1, OUTPUT_LINES);
    }

    fn push_chat(&mut self, speaker: ChatSpeaker, text: ConsoleLine) {
        self.chat_lines[self.chat_next] = ChatLine { speaker, text };
        self.chat_next = (self.chat_next + 1) % CHAT_LINES;
        self.chat_count = usize::min(self.chat_count + 1, CHAT_LINES);
    }

    fn snapshot(&self) -> ConsoleSnapshot {
        let mut lines = [ConsoleLine::empty(); OUTPUT_LINES];
        let oldest = if self.line_count == OUTPUT_LINES {
            self.next_line
        } else {
            0
        };
        let start = OUTPUT_LINES - self.line_count;
        let mut idx = 0usize;
        while idx < self.line_count {
            let source = (oldest + idx) % OUTPUT_LINES;
            lines[start + idx] = self.lines[source];
            idx += 1;
        }

        let mut chat_lines = [ChatLine::empty(); CHAT_LINES];
        let chat_oldest = if self.chat_count == CHAT_LINES {
            self.chat_next
        } else {
            0
        };
        let chat_start = CHAT_LINES - self.chat_count;
        let mut chat_idx = 0usize;
        while chat_idx < self.chat_count {
            let source = (chat_oldest + chat_idx) % CHAT_LINES;
            chat_lines[chat_start + chat_idx] = self.chat_lines[source];
            chat_idx += 1;
        }

        let mut input = ConsoleLine::empty();
        match self.mode {
            ConsoleMode::Command => {
                let _ = write!(input, "> {}", self.input.as_str());
            }
            ConsoleMode::SetupMenu => {
                let _ = write!(input, "setup> {}", self.input.as_str());
            }
            ConsoleMode::ApiKeyEntry => {
                let _ = write!(input, "api key> ");
                let mut idx = 0usize;
                while idx < self.input.len() {
                    let _ = input.write_str("*");
                    idx += 1;
                }
            }
            ConsoleMode::WifiSsidEntry => {
                let _ = write!(input, "wifi ssid> {}", self.input.as_str());
            }
            ConsoleMode::WifiPassphraseEntry => {
                let _ = write!(input, "wifi key> ");
                let mut idx = 0usize;
                while idx < self.input.len() {
                    let _ = input.write_str("*");
                    idx += 1;
                }
            }
        }

        let mut chat_input = ConsoleLine::empty();
        let _ = write!(chat_input, "{}", self.input.as_str());

        let provider = provider::snapshot();
        let wifi = wifi::snapshot();
        ConsoleSnapshot {
            lines,
            input,
            view: self.view,
            focus: self.focus,
            chat_lines,
            chat_input,
            settings_entry_active: matches!(
                self.mode,
                ConsoleMode::ApiKeyEntry
                    | ConsoleMode::WifiSsidEntry
                    | ConsoleMode::WifiPassphraseEntry
            ),
            api_key_set: provider.api_key_set,
            provider_name: provider.provider_name,
            provider_phase: provider.direct_phase,
            provider_model: provider.direct_model,
            wifi_ssid: wifi.ssid,
            wifi_passphrase_set: wifi.passphrase_set,
        }
    }

    fn handle_keyboard_byte(&mut self, byte: u8) -> ByteAction {
        match (self.view, self.mode) {
            (UiView::Ai, ConsoleMode::Command) => self.handle_chat_byte(byte),
            (_, ConsoleMode::Command) => self.handle_command_byte(byte),
            (_, ConsoleMode::SetupMenu) => self.handle_setup_menu_byte(byte),
            (_, ConsoleMode::ApiKeyEntry) => self.handle_api_key_byte(byte),
            (_, ConsoleMode::WifiSsidEntry) => self.handle_wifi_ssid_byte(byte),
            (_, ConsoleMode::WifiPassphraseEntry) => self.handle_wifi_passphrase_byte(byte),
        }
    }

    fn handle_keyboard_input(&mut self, input: input::ConsoleInput) -> ByteAction {
        match input {
            input::ConsoleInput::Byte(byte) => self.handle_keyboard_byte(byte),
            input::ConsoleInput::Special(key) => self.handle_special_key(key),
        }
    }

    fn handle_serial_byte(&mut self, byte: u8) -> ByteAction {
        match self.mode {
            ConsoleMode::Command => self.handle_serial_command_byte(byte),
            ConsoleMode::SetupMenu => self.handle_setup_menu_byte(byte),
            ConsoleMode::ApiKeyEntry => self.handle_api_key_byte(byte),
            ConsoleMode::WifiSsidEntry => self.handle_wifi_ssid_byte(byte),
            ConsoleMode::WifiPassphraseEntry => self.handle_wifi_passphrase_byte(byte),
        }
    }

    fn set_view(&mut self, view: UiView) -> ByteAction {
        self.view = view;
        if view == UiView::Settings {
            self.mode = ConsoleMode::SetupMenu;
            self.focus = UiFocus::SettingsProvider;
            self.input.clear();
            ByteAction::ShowSetupMenu
        } else if self.mode != ConsoleMode::Command {
            self.mode = ConsoleMode::Command;
            self.focus = focus_for_view(view);
            self.input.clear();
            ByteAction::Redraw
        } else {
            self.focus = focus_for_view(view);
            ByteAction::Redraw
        }
    }

    fn enter_setup_menu(&mut self) {
        self.mode = ConsoleMode::SetupMenu;
        self.focus = UiFocus::SettingsProvider;
        self.input.clear();
    }

    fn handle_special_key(&mut self, key: input::SpecialKey) -> ByteAction {
        match self.mode {
            ConsoleMode::ApiKeyEntry => match key {
                input::SpecialKey::Enter => self.handle_api_key_byte(b'\r'),
                input::SpecialKey::Escape => self.handle_api_key_byte(0x1b),
                input::SpecialKey::Tab
                | input::SpecialKey::BackTab
                | input::SpecialKey::Up
                | input::SpecialKey::Down
                | input::SpecialKey::Left
                | input::SpecialKey::Right => ByteAction::Redraw,
            },
            ConsoleMode::WifiSsidEntry => match key {
                input::SpecialKey::Enter => self.handle_wifi_ssid_byte(b'\r'),
                input::SpecialKey::Escape => self.handle_wifi_ssid_byte(0x1b),
                input::SpecialKey::Tab
                | input::SpecialKey::BackTab
                | input::SpecialKey::Up
                | input::SpecialKey::Down
                | input::SpecialKey::Left
                | input::SpecialKey::Right => ByteAction::Redraw,
            },
            ConsoleMode::WifiPassphraseEntry => match key {
                input::SpecialKey::Enter => self.handle_wifi_passphrase_byte(b'\r'),
                input::SpecialKey::Escape => self.handle_wifi_passphrase_byte(0x1b),
                input::SpecialKey::Tab
                | input::SpecialKey::BackTab
                | input::SpecialKey::Up
                | input::SpecialKey::Down
                | input::SpecialKey::Left
                | input::SpecialKey::Right => ByteAction::Redraw,
            },
            _ => match key {
                input::SpecialKey::Enter => self.activate_focus(),
                input::SpecialKey::Escape => self.handle_escape_key(),
                input::SpecialKey::Tab | input::SpecialKey::Right | input::SpecialKey::Down => {
                    self.move_focus(1);
                    ByteAction::Redraw
                }
                input::SpecialKey::BackTab | input::SpecialKey::Left | input::SpecialKey::Up => {
                    self.move_focus(-1);
                    ByteAction::Redraw
                }
            },
        }
    }

    fn handle_escape_key(&mut self) -> ByteAction {
        if self.mode == ConsoleMode::SetupMenu || self.view == UiView::Settings {
            self.mode = ConsoleMode::Command;
            self.view = UiView::Ai;
            self.focus = UiFocus::ChatInput;
            self.input.clear();
            ByteAction::SetupClosed
        } else {
            self.focus = focus_for_view(self.view);
            ByteAction::Redraw
        }
    }

    fn activate_focus(&mut self) -> ByteAction {
        match self.focus {
            UiFocus::NavAi => self.set_view(UiView::Ai),
            UiFocus::NavConsole => self.set_view(UiView::Console),
            UiFocus::NavSettings => self.set_view(UiView::Settings),
            UiFocus::ChatInput => self.handle_chat_byte(b'\r'),
            UiFocus::ConsoleInput => self.handle_command_byte(b'\r'),
            UiFocus::SettingsProvider => ByteAction::ShowProviderStatus,
            UiFocus::SettingsApiKey => {
                self.mode = ConsoleMode::ApiKeyEntry;
                self.input.clear();
                ByteAction::ShowApiKeyEntry
            }
            UiFocus::SettingsClear => {
                provider_config::clear_api_key();
                ByteAction::ShowSetupMessage(SetupMessage::ApiKeyCleared)
            }
            UiFocus::SettingsWifiSsid => {
                self.mode = ConsoleMode::WifiSsidEntry;
                self.input.clear();
                ByteAction::ShowWifiSsidEntry
            }
            UiFocus::SettingsWifiPassphrase => {
                self.mode = ConsoleMode::WifiPassphraseEntry;
                self.input.clear();
                ByteAction::ShowWifiPassphraseEntry
            }
            UiFocus::SettingsWifiClear => {
                wifi::clear_config();
                ByteAction::ShowSetupMessage(SetupMessage::WifiConfigCleared)
            }
            UiFocus::SettingsClose => {
                self.mode = ConsoleMode::Command;
                self.view = UiView::Ai;
                self.focus = UiFocus::ChatInput;
                self.input.clear();
                ByteAction::SetupClosed
            }
        }
    }

    fn move_focus(&mut self, delta: isize) {
        let order = focus_order(self.view, self.mode);
        let mut current = 0usize;
        while current < order.len() {
            if order[current] == self.focus {
                break;
            }
            current += 1;
        }
        if current == order.len() {
            self.focus = order[0];
            return;
        }

        let len = order.len() as isize;
        let next = (current as isize + delta).rem_euclid(len) as usize;
        self.focus = order[next];
    }

    fn handle_command_byte(&mut self, byte: u8) -> ByteAction {
        match byte {
            b'\r' | b'\n' => {
                if self.input.is_empty() {
                    ByteAction::Noop
                } else {
                    let command = self.input;
                    self.input.clear();
                    ByteAction::Execute(command)
                }
            }
            0x08 | 0x7f => {
                if self.input.pop_char() {
                    ByteAction::Backspace
                } else {
                    ByteAction::Noop
                }
            }
            b if is_text_byte(b) => {
                if self.input.push_text_byte(b) {
                    ByteAction::Echo(b)
                } else {
                    ByteAction::Bell
                }
            }
            _ => ByteAction::Noop,
        }
    }

    fn handle_serial_command_byte(&mut self, byte: u8) -> ByteAction {
        match byte {
            b'\r' | b'\n' => {
                if self.serial_input.is_empty() {
                    ByteAction::Noop
                } else {
                    let command = self.serial_input;
                    self.serial_input.clear();
                    ByteAction::Execute(command)
                }
            }
            0x08 | 0x7f => {
                if self.serial_input.pop_char() {
                    ByteAction::Backspace
                } else {
                    ByteAction::Noop
                }
            }
            b if is_text_byte(b) => {
                if self.serial_input.push_text_byte(b) {
                    ByteAction::Echo(b)
                } else {
                    ByteAction::Bell
                }
            }
            _ => ByteAction::Noop,
        }
    }

    fn handle_chat_byte(&mut self, byte: u8) -> ByteAction {
        match byte {
            b'\r' | b'\n' => {
                if self.input.is_empty() {
                    ByteAction::Noop
                } else {
                    let prompt = self.input.as_line();
                    self.input.clear();
                    ByteAction::SubmitChat(prompt)
                }
            }
            0x08 | 0x7f => {
                if self.input.pop_char() {
                    ByteAction::Redraw
                } else {
                    ByteAction::Noop
                }
            }
            b if is_text_byte(b) => {
                if self.input.push_text_byte(b) {
                    ByteAction::Redraw
                } else {
                    ByteAction::Bell
                }
            }
            _ => ByteAction::Noop,
        }
    }

    fn handle_setup_menu_byte(&mut self, byte: u8) -> ByteAction {
        match byte.to_ascii_lowercase() {
            b'1' => ByteAction::ShowProviderStatus,
            b'2' => {
                self.mode = ConsoleMode::ApiKeyEntry;
                self.input.clear();
                ByteAction::ShowApiKeyEntry
            }
            b'3' => {
                provider_config::clear_api_key();
                ByteAction::ShowSetupMessage(SetupMessage::ApiKeyCleared)
            }
            b'4' => {
                self.mode = ConsoleMode::WifiSsidEntry;
                self.input.clear();
                ByteAction::ShowWifiSsidEntry
            }
            b'5' => {
                self.mode = ConsoleMode::WifiPassphraseEntry;
                self.input.clear();
                ByteAction::ShowWifiPassphraseEntry
            }
            b'6' => {
                wifi::clear_config();
                ByteAction::ShowSetupMessage(SetupMessage::WifiConfigCleared)
            }
            b'q' | 0x1b => {
                self.mode = ConsoleMode::Command;
                self.view = UiView::Ai;
                self.input.clear();
                ByteAction::SetupClosed
            }
            b'\r' | b'\n' => ByteAction::Noop,
            _ => ByteAction::Bell,
        }
    }

    fn handle_api_key_byte(&mut self, byte: u8) -> ByteAction {
        match byte {
            b'\r' | b'\n' => {
                let result = provider_config::set_api_key(self.input.as_bytes());
                self.input.clear();
                self.mode = ConsoleMode::SetupMenu;
                match result {
                    Ok(()) => ByteAction::ShowSetupMessage(SetupMessage::ApiKeySet),
                    Err(provider_config::ApiKeyError::Empty) => {
                        ByteAction::ShowSetupMessage(SetupMessage::ApiKeyEmpty)
                    }
                    Err(provider_config::ApiKeyError::TooLong) => {
                        ByteAction::ShowSetupMessage(SetupMessage::ApiKeyTooLong)
                    }
                    Err(provider_config::ApiKeyError::InvalidByte) => {
                        ByteAction::ShowSetupMessage(SetupMessage::ApiKeyInvalid)
                    }
                }
            }
            0x1b => {
                self.input.clear();
                self.mode = ConsoleMode::SetupMenu;
                ByteAction::ShowSetupMessage(SetupMessage::ApiKeyCancelled)
            }
            0x08 | 0x7f => {
                if self.input.pop_char() {
                    ByteAction::Redraw
                } else {
                    ByteAction::Noop
                }
            }
            b if b.is_ascii_graphic() => {
                if self.input.push_byte(b) {
                    ByteAction::Redraw
                } else {
                    ByteAction::Bell
                }
            }
            _ => ByteAction::Noop,
        }
    }

    fn handle_wifi_ssid_byte(&mut self, byte: u8) -> ByteAction {
        match byte {
            b'\r' | b'\n' => {
                let result = wifi::set_ssid(self.input.as_bytes());
                self.input.clear();
                self.mode = ConsoleMode::SetupMenu;
                match result {
                    Ok(()) => ByteAction::ShowSetupMessage(SetupMessage::WifiSsidSet),
                    Err(wifi::WifiConfigError::EmptySsid) => {
                        ByteAction::ShowSetupMessage(SetupMessage::WifiSsidEmpty)
                    }
                    Err(wifi::WifiConfigError::SsidTooLong) => {
                        ByteAction::ShowSetupMessage(SetupMessage::WifiSsidTooLong)
                    }
                    Err(wifi::WifiConfigError::InvalidByte) => {
                        ByteAction::ShowSetupMessage(SetupMessage::WifiConfigInvalid)
                    }
                    Err(
                        wifi::WifiConfigError::PassphraseTooShort
                        | wifi::WifiConfigError::PassphraseTooLong,
                    ) => ByteAction::ShowSetupMessage(SetupMessage::WifiConfigInvalid),
                }
            }
            0x1b => {
                self.input.clear();
                self.mode = ConsoleMode::SetupMenu;
                ByteAction::ShowSetupMessage(SetupMessage::WifiEntryCancelled)
            }
            0x08 | 0x7f => {
                if self.input.pop_char() {
                    ByteAction::Redraw
                } else {
                    ByteAction::Noop
                }
            }
            b if is_text_byte(b) => {
                if self.input.push_text_byte(b) {
                    ByteAction::Redraw
                } else {
                    ByteAction::Bell
                }
            }
            _ => ByteAction::Noop,
        }
    }

    fn handle_wifi_passphrase_byte(&mut self, byte: u8) -> ByteAction {
        match byte {
            b'\r' | b'\n' => {
                let result = wifi::set_passphrase(self.input.as_bytes());
                self.input.clear();
                self.mode = ConsoleMode::SetupMenu;
                match result {
                    Ok(()) => ByteAction::ShowSetupMessage(SetupMessage::WifiPassphraseSet),
                    Err(wifi::WifiConfigError::PassphraseTooShort) => {
                        ByteAction::ShowSetupMessage(SetupMessage::WifiPassphraseTooShort)
                    }
                    Err(wifi::WifiConfigError::PassphraseTooLong) => {
                        ByteAction::ShowSetupMessage(SetupMessage::WifiPassphraseTooLong)
                    }
                    Err(wifi::WifiConfigError::InvalidByte) => {
                        ByteAction::ShowSetupMessage(SetupMessage::WifiConfigInvalid)
                    }
                    Err(wifi::WifiConfigError::EmptySsid | wifi::WifiConfigError::SsidTooLong) => {
                        ByteAction::ShowSetupMessage(SetupMessage::WifiConfigInvalid)
                    }
                }
            }
            0x1b => {
                self.input.clear();
                self.mode = ConsoleMode::SetupMenu;
                ByteAction::ShowSetupMessage(SetupMessage::WifiEntryCancelled)
            }
            0x08 | 0x7f => {
                if self.input.pop_char() {
                    ByteAction::Redraw
                } else {
                    ByteAction::Noop
                }
            }
            b if is_text_byte(b) => {
                if self.input.push_text_byte(b) {
                    ByteAction::Redraw
                } else {
                    ByteAction::Bell
                }
            }
            _ => ByteAction::Noop,
        }
    }
}

const AI_FOCUS_ORDER: [UiFocus; 4] = [
    UiFocus::ChatInput,
    UiFocus::NavAi,
    UiFocus::NavConsole,
    UiFocus::NavSettings,
];
const CONSOLE_FOCUS_ORDER: [UiFocus; 4] = [
    UiFocus::ConsoleInput,
    UiFocus::NavAi,
    UiFocus::NavConsole,
    UiFocus::NavSettings,
];
const SETTINGS_FOCUS_ORDER: [UiFocus; 10] = [
    UiFocus::SettingsProvider,
    UiFocus::SettingsApiKey,
    UiFocus::SettingsClear,
    UiFocus::SettingsWifiSsid,
    UiFocus::SettingsWifiPassphrase,
    UiFocus::SettingsWifiClear,
    UiFocus::SettingsClose,
    UiFocus::NavAi,
    UiFocus::NavConsole,
    UiFocus::NavSettings,
];

fn focus_for_view(view: UiView) -> UiFocus {
    match view {
        UiView::Ai => UiFocus::ChatInput,
        UiView::Console => UiFocus::ConsoleInput,
        UiView::Settings => UiFocus::SettingsProvider,
    }
}

fn focus_order(view: UiView, mode: ConsoleMode) -> &'static [UiFocus] {
    if mode == ConsoleMode::SetupMenu {
        return &SETTINGS_FOCUS_ORDER;
    }

    match view {
        UiView::Ai => &AI_FOCUS_ORDER,
        UiView::Console => &CONSOLE_FOCUS_ORDER,
        UiView::Settings => &SETTINGS_FOCUS_ORDER,
    }
}

enum ByteAction {
    Noop,
    Echo(u8),
    Backspace,
    Bell,
    Execute(CommandLine),
    SubmitChat(ConsoleLine),
    Redraw,
    ShowApiKeyEntry,
    ShowWifiSsidEntry,
    ShowWifiPassphraseEntry,
    ShowSetupMenu,
    ShowProviderStatus,
    ShowSetupMessage(SetupMessage),
    SetupClosed,
}

enum SetupMessage {
    ApiKeySet,
    ApiKeyCleared,
    ApiKeyEmpty,
    ApiKeyTooLong,
    ApiKeyInvalid,
    ApiKeyCancelled,
    WifiSsidSet,
    WifiPassphraseSet,
    WifiConfigCleared,
    WifiSsidEmpty,
    WifiSsidTooLong,
    WifiPassphraseTooShort,
    WifiPassphraseTooLong,
    WifiConfigInvalid,
    WifiEntryCancelled,
}

#[derive(Clone, Copy)]
struct CommandLine {
    bytes: [u8; COMMAND_WIDTH],
    len: usize,
}

impl CommandLine {
    const fn new() -> Self {
        Self {
            bytes: [0; COMMAND_WIDTH],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }

    fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    fn trimmed_bounds(&self) -> (usize, usize) {
        let mut start = 0usize;
        let mut end = self.len;
        while start < end && self.bytes[start].is_ascii_whitespace() {
            start += 1;
        }
        while end > start && self.bytes[end - 1].is_ascii_whitespace() {
            end -= 1;
        }

        (start, end)
    }

    fn trimmed_str(&self) -> &str {
        let (start, end) = self.trimmed_bounds();
        unsafe { str::from_utf8_unchecked(&self.bytes[start..end]) }
    }

    fn command_word(&self) -> CommandText {
        let (start, end) = self.trimmed_bounds();
        let mut text = CommandText::new();
        let mut idx = start;
        while idx < end && !self.bytes[idx].is_ascii_whitespace() {
            let byte = self.bytes[idx];
            if !byte.is_ascii() {
                break;
            }
            text.push_byte(byte.to_ascii_lowercase());
            idx += 1;
        }
        text
    }

    fn arguments_after_command(&self) -> &str {
        let (mut idx, end) = self.trimmed_bounds();
        while idx < end && !self.bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        while idx < end && self.bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        unsafe { str::from_utf8_unchecked(&self.bytes[idx..end]) }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn as_line(&self) -> ConsoleLine {
        let mut line = ConsoleLine::empty();
        let _ = line.write_str(self.as_str());
        line
    }

    fn clear(&mut self) {
        self.bytes[..self.len].fill(0);
        self.len = 0;
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn push_byte(&mut self, byte: u8) -> bool {
        if self.len == self.bytes.len() {
            return false;
        }
        self.bytes[self.len] = byte;
        self.len += 1;
        true
    }

    fn push_text_byte(&mut self, byte: u8) -> bool {
        if self.len == self.bytes.len() {
            return false;
        }
        self.bytes[self.len] = byte;
        self.len += 1;
        if is_valid_utf8_prefix(&self.bytes[..self.len]) {
            true
        } else {
            self.len -= 1;
            self.bytes[self.len] = 0;
            false
        }
    }

    fn pop_char(&mut self) -> bool {
        if self.len == 0 {
            return false;
        }
        let old_len = self.len;
        let mut new_len = old_len - 1;
        while new_len > 0 && is_utf8_continuation(self.bytes[new_len]) {
            new_len -= 1;
        }
        self.bytes[new_len..old_len].fill(0);
        self.len = new_len;
        true
    }
}

struct CommandText {
    bytes: [u8; COMMAND_WIDTH],
    len: usize,
}

impl CommandText {
    const fn new() -> Self {
        Self {
            bytes: [0; COMMAND_WIDTH],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }

    fn push_byte(&mut self, byte: u8) {
        if self.len == self.bytes.len() {
            return;
        }
        self.bytes[self.len] = byte;
        self.len += 1;
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

fn is_text_byte(byte: u8) -> bool {
    byte == b' ' || byte.is_ascii_graphic() || byte >= 0x80
}

fn is_valid_utf8_prefix(bytes: &[u8]) -> bool {
    match str::from_utf8(bytes) {
        Ok(_) => true,
        Err(error) => error.error_len().is_none(),
    }
}

fn is_utf8_continuation(byte: u8) -> bool {
    (byte & 0b1100_0000) == 0b1000_0000
}

pub fn init() {
    write_output(format_args!("SERIAL CONSOLE READY"));
}

pub fn poll(runtime: ui::RuntimeStatus) -> bool {
    let mut changed = false;
    let mut processed = 0usize;

    while processed < MAX_BYTES_PER_POLL {
        let Some(byte) = serial::try_read_byte() else {
            break;
        };

        changed |= process_serial_byte(byte, runtime);
        processed += 1;
    }

    input::drain_console_input(|input| {
        changed |= process_input(input, runtime);
    });

    changed
}

pub fn snapshot() -> ConsoleSnapshot {
    CONSOLE.lock().snapshot()
}

pub fn set_view(view: UiView) -> bool {
    let action = {
        let mut state = CONSOLE.lock();
        state.set_view(view)
    };
    apply_action(action, ui::RuntimeStatus::new())
}

pub fn record_event(args: fmt::Arguments<'_>) {
    let mut line = ConsoleLine::empty();
    let _ = line.write_fmt(args);
    CONSOLE.lock().push_line(line);
}

pub fn write_event(args: fmt::Arguments<'_>) {
    let mut line = ConsoleLine::empty();
    let _ = line.write_fmt(args);
    serial::write_line(line.as_str());
    {
        let mut state = CONSOLE.lock();
        state.push_line(line);
        if let Some(answer) = line.as_str().strip_prefix("OPENAI: ") {
            let mut chat = ConsoleLine::empty();
            let _ = chat.write_str(answer);
            state.push_chat(ChatSpeaker::Assistant, chat);
        } else {
            state.push_chat(ChatSpeaker::System, line);
        }
    }
}

fn process_input(input: input::ConsoleInput, runtime: ui::RuntimeStatus) -> bool {
    let action = {
        let mut state = CONSOLE.lock();
        state.handle_keyboard_input(input)
    };

    apply_action(action, runtime)
}

fn apply_action(action: ByteAction, runtime: ui::RuntimeStatus) -> bool {
    match action {
        ByteAction::Noop => false,
        ByteAction::Echo(byte) => {
            serial::write_byte(byte);
            true
        }
        ByteAction::Backspace => {
            serial::write_fmt(format_args!("\x08 \x08"));
            true
        }
        ByteAction::Bell => {
            serial::write_byte(0x07);
            false
        }
        ByteAction::Execute(command) => {
            serial::write_line("");
            execute(command, runtime);
            true
        }
        ByteAction::SubmitChat(prompt) => {
            submit_chat(prompt, runtime);
            true
        }
        ByteAction::Redraw => true,
        ByteAction::ShowApiKeyEntry => {
            show_api_key_entry();
            true
        }
        ByteAction::ShowWifiSsidEntry => {
            show_wifi_ssid_entry();
            true
        }
        ByteAction::ShowWifiPassphraseEntry => {
            show_wifi_passphrase_entry();
            true
        }
        ByteAction::ShowSetupMenu => {
            show_setup_menu();
            true
        }
        ByteAction::ShowProviderStatus => {
            show_provider_status();
            true
        }
        ByteAction::ShowSetupMessage(message) => {
            show_setup_message(message);
            show_setup_menu();
            true
        }
        ByteAction::SetupClosed => {
            write_output(format_args!("SETUP CLOSED"));
            true
        }
    }
}

fn process_serial_byte(byte: u8, runtime: ui::RuntimeStatus) -> bool {
    let action = {
        let mut state = CONSOLE.lock();
        state.handle_serial_byte(byte)
    };
    apply_action(action, runtime)
}

fn execute(command_line: CommandLine, runtime: ui::RuntimeStatus) {
    let command = command_line.command_word();
    if command.as_str().is_empty() {
        return;
    }

    write_output(format_args!("> {}", command_line.trimmed_str()));

    match command.as_str() {
        "help" => command_help(),
        "status" => command_status(runtime),
        "devices" => command_devices(runtime),
        "log" => command_log(),
        "describe" | "system.describe" => command_agent_protocol("system.describe", runtime),
        "snapshot" | "system.snapshot" => command_agent_protocol("system.snapshot", runtime),
        "caps" | "capabilities" | "system.capabilities" => {
            command_agent_protocol("system.capabilities", runtime)
        }
        "bootlog" | "system.bootlog" | "system.boot_log" => {
            command_agent_protocol("system.boot_log", runtime)
        }
        "devicegraph" | "device.graph" => command_agent_protocol("device.graph", runtime),
        "problems" | "problem.list" => command_agent_protocol("problem.list", runtime),
        "services" | "service.inventory" => command_agent_protocol("service.inventory", runtime),
        "memory.profile" | "memprofile" => command_agent_protocol("memory.profile", runtime),
        "memory.context" | "memctx" => command_agent_protocol(command_line.trimmed_str(), runtime),
        "memory.query" | "memquery" => command_agent_protocol(command_line.trimmed_str(), runtime),
        "memory.trace" | "memtrace" => command_agent_protocol(command_line.trimmed_str(), runtime),
        "memory.recent_events" | "audit.events" | "events" => {
            command_agent_protocol(command_line.trimmed_str(), runtime)
        }
        "agent" => command_agent_protocol(command_line.arguments_after_command(), runtime),
        "memory.record_observation"
        | "memory.propose_policy"
        | "memory.supersede_fact"
        | "memory.redact"
        | "memory.compact" => command_agent_protocol(command.as_str(), runtime),
        "provider.context_export" | "provider.export_context" => {
            command_agent_protocol(command_line.trimmed_str(), runtime)
        }
        "provider.context_gate"
        | "provider.context_export_status"
        | "provider.context_gate_selftest"
        | "provider.context_injection_gate"
        | "provider.context_injection_gate_selftest" => {
            command_agent_protocol(command_line.trimmed_str(), runtime)
        }
        "module.propose"
        | "module.build_result"
        | "module.test_request"
        | "module.test_result"
        | "module.load_ephemeral"
        | "module.load_recovery_artifact"
        | "module.persist"
        | "module.rollback"
        | "recovery.load_artifact"
        | "service.load_ephemeral"
        | "service.restart"
        | "service.start"
        | "service.stop"
        | "config.apply"
        | "apply_config" => command_agent_protocol(command.as_str(), runtime),
        "provider" => command_provider_status(),
        "openai" => command_openai_status(),
        "wifi" => command_wifi_status(),
        "setup" => command_setup_enter(),
        "ask" => command_ask(command_line.arguments_after_command(), runtime),
        _ => write_output(format_args!(
            "UNKNOWN COMMAND: {}",
            command_line.trimmed_str()
        )),
    }
}

fn command_help() {
    write_output(format_args!(
        "COMMANDS: help status devices log provider openai wifi setup ask <text>"
    ));
    write_output(format_args!(
        "AGENT: describe snapshot caps bootlog services problems device.graph memory.profile"
    ));
    write_output(format_args!(
        "AGENT RAW: memory.context provider.context_export provider.context_gate provider.context_gate_selftest provider.context_injection_gate provider.context_injection_gate_selftest memory.query memory.trace memory.recent_events"
    ));
    write_output(format_args!(
        "RECOVERY: recovery.load_artifact module.load_recovery_artifact recovery.lifeline_command_admission recovery.lifeline_command_envelope_diagnostic recovery.lifeline_command_dispatch_diagnostic recovery.lifeline_command_body_canonicalization_diagnostic recovery.lifeline_command_handler_binding_diagnostic recovery.lifeline_status_read_handler_diagnostic recovery.rollback_preview_authorization_diagnostic recovery.rollback_apply_authorization_diagnostic recovery.disable_module_target_binding_diagnostic recovery.restart_last_good_target_binding_diagnostic recovery.load_artifact_by_hash_target_binding_diagnostic recovery.memory_write_authority_diagnostic recovery.durable_audit_rollback_write_authority_diagnostic recovery.service_inventory_side_effect_boundary_diagnostic recovery.lifeline_command_dispatch_behavior_diagnostic"
    ));
}

fn command_status(runtime: ui::RuntimeStatus) {
    let status = system_status::SystemSnapshot::collect(None, runtime);
    write_output(format_args!(
        "FRAMEBUFFER: SEE UI    ENTROPY: {} {}",
        status.entropy.state.as_str(),
        status.entropy.detail.as_str()
    ));
    write_output(format_args!(
        "USB-XHCI: {}    WIFI: {}    NETWORK: {}    INPUT: {}",
        status.usb_xhci.state.as_str(),
        status.wifi.state.as_str(),
        status.network.state.as_str(),
        status.input.state.as_str()
    ));
}

fn command_devices(runtime: ui::RuntimeStatus) {
    let status = system_status::SystemSnapshot::collect(None, runtime);
    write_output(format_args!("FRAMEBUFFER: SEE UI"));
    write_status_line(&status.entropy);
    write_status_line(&status.usb_xhci);
    write_status_line(&status.wifi);
    write_status_line(&status.network);
    write_status_line(&status.input);
}

fn write_status_line(line: &system_status::StatusLine) {
    write_output(format_args!(
        "{}: {} - {}",
        line.label,
        line.state.as_str(),
        line.detail.as_str()
    ));
}

fn command_log() {
    let snapshot = snapshot();
    serial::write_line("RECENT LOG:");
    let mut idx = 0usize;
    while idx < OUTPUT_LINES {
        let line = snapshot.lines[idx];
        if !line.as_str().is_empty() {
            serial::write_line(line.as_str());
        }
        idx += 1;
    }
    record_event(format_args!("RECENT LOG WRITTEN TO SERIAL"));
}

fn command_agent_protocol(method: &str, runtime: ui::RuntimeStatus) {
    match agent_protocol::dispatch(method, runtime) {
        agent_protocol::DispatchOutcome::Response(method) => {
            record_event(format_args!("AGENT {} WRITTEN TO SERIAL", method));
            serial::write_line("AGENT RESPONSE WRITTEN TO SERIAL");
        }
        agent_protocol::DispatchOutcome::Denied(method) => {
            record_event(format_args!("AGENT {} DENIED", method));
            serial::write_line("AGENT CAPABILITY DENIED WRITTEN TO SERIAL");
        }
        agent_protocol::DispatchOutcome::Unknown => {
            write_output(format_args!("UNKNOWN AGENT METHOD: {}", method.trim()));
        }
    }
}

fn command_setup_enter() {
    {
        let mut state = CONSOLE.lock();
        state.view = UiView::Settings;
        state.enter_setup_menu();
    }

    write_output(format_args!("SETUP"));
    show_setup_menu();
}

fn show_setup_menu() {
    let provider = provider_config::snapshot();
    let wifi = wifi::snapshot();
    write_output(format_args!(
        "1 PROVIDER: {} DIRECT    2 API KEY: {}",
        provider.provider_name,
        api_key_status(provider.api_key_set)
    ));
    write_output(format_args!(
        "3 CLEAR API KEY    4 WIFI SSID: {}",
        wifi_ssid_status(&wifi.ssid)
    ));
    write_output(format_args!(
        "5 WIFI KEY: {}    6 CLEAR WIFI    Q EXIT",
        api_key_status(wifi.passphrase_set)
    ));
}

fn show_api_key_entry() {
    write_output(format_args!("API KEY ENTRY"));
    write_output(format_args!("TYPE KEY, ENTER TO SAVE, ESC TO CANCEL"));
}

fn show_wifi_ssid_entry() {
    write_output(format_args!("WIFI SSID ENTRY"));
    write_output(format_args!("TYPE SSID, ENTER TO SAVE, ESC TO CANCEL"));
}

fn show_wifi_passphrase_entry() {
    write_output(format_args!("WIFI KEY ENTRY"));
    write_output(format_args!("TYPE WPA KEY, ENTER TO SAVE, ESC TO CANCEL"));
}

fn show_provider_status() {
    let snapshot = provider::snapshot();
    write_output(format_args!(
        "PROVIDER: {}    ROUTE: {}",
        snapshot.provider_name,
        snapshot.route.as_str()
    ));
    write_output(format_args!(
        "API KEY: {}    ENDPOINT: {}",
        api_key_status(snapshot.api_key_set),
        snapshot.direct_endpoint
    ));
    write_output(format_args!("TLS TRUST: {}", snapshot.trust_state));
    if !snapshot.api_key_set {
        write_output(format_args!("OPENAI REQUIRES API KEY"));
    }
}

fn show_setup_message(message: SetupMessage) {
    match message {
        SetupMessage::ApiKeySet => write_output(format_args!("API KEY SET (RAM ONLY)")),
        SetupMessage::ApiKeyCleared => write_output(format_args!("API KEY CLEARED")),
        SetupMessage::ApiKeyEmpty => write_output(format_args!("API KEY NOT CHANGED: EMPTY")),
        SetupMessage::ApiKeyTooLong => write_output(format_args!("API KEY NOT CHANGED: TOO LONG")),
        SetupMessage::ApiKeyInvalid => {
            write_output(format_args!("API KEY NOT CHANGED: INVALID BYTE"))
        }
        SetupMessage::ApiKeyCancelled => write_output(format_args!("API KEY ENTRY CANCELLED")),
        SetupMessage::WifiSsidSet => write_output(format_args!("WIFI SSID SET (RAM ONLY)")),
        SetupMessage::WifiPassphraseSet => write_output(format_args!("WIFI KEY SET (RAM ONLY)")),
        SetupMessage::WifiConfigCleared => write_output(format_args!("WIFI CONFIG CLEARED")),
        SetupMessage::WifiSsidEmpty => write_output(format_args!("WIFI SSID NOT CHANGED: EMPTY")),
        SetupMessage::WifiSsidTooLong => {
            write_output(format_args!("WIFI SSID NOT CHANGED: TOO LONG"))
        }
        SetupMessage::WifiPassphraseTooShort => {
            write_output(format_args!("WIFI KEY NOT CHANGED: TOO SHORT"))
        }
        SetupMessage::WifiPassphraseTooLong => {
            write_output(format_args!("WIFI KEY NOT CHANGED: TOO LONG"))
        }
        SetupMessage::WifiConfigInvalid => {
            write_output(format_args!("WIFI CONFIG NOT CHANGED: INVALID BYTE"))
        }
        SetupMessage::WifiEntryCancelled => write_output(format_args!("WIFI ENTRY CANCELLED")),
    }
}

fn command_provider_status() {
    let snapshot = provider::snapshot();
    write_output(format_args!(
        "PROVIDER: {}    API KEY: {}",
        snapshot.provider_name,
        api_key_status(snapshot.api_key_set)
    ));

    write_output(format_args!("ROUTE: {}", snapshot.route.as_str()));
    write_output(format_args!("TLS TRUST: {}", snapshot.trust_state));
    command_openai_status();
}

fn command_openai_status() {
    let snapshot = provider::snapshot();
    write_output(format_args!(
        "OPENAI DIRECT: {}    MODEL: {}",
        snapshot.direct_phase, snapshot.direct_model
    ));
    write_output(format_args!("ENDPOINT: {}", snapshot.direct_endpoint));
    if let Some(pin_kind) = snapshot.trust_pin_kind {
        if let Some(pin_id) = snapshot.trust_pin_id {
            write_output(format_args!(
                "TRUST: {}    PIN: {} {}",
                snapshot.trust_state, pin_kind, pin_id
            ));
        } else {
            write_output(format_args!(
                "TRUST: {}    PIN: {}",
                snapshot.trust_state, pin_kind
            ));
        }
    } else {
        write_output(format_args!("TRUST: {}", snapshot.trust_state));
    }
    if let Some(id) = snapshot.direct_pending_id {
        write_output(format_args!("OPENAI REQUEST {} PENDING", id));
    }
    if let Some(id) = snapshot.direct_last_request_id {
        write_output(format_args!(
            "LAST OPENAI REQUEST {}: {}",
            id,
            snapshot.direct_last_prompt.as_str()
        ));
    }
    if !snapshot.direct_last_event.as_str().is_empty() {
        write_output(format_args!(
            "OPENAI EVENT: {}",
            snapshot.direct_last_event.as_str()
        ));
    }
    if !snapshot.direct_last_error.as_str().is_empty() {
        write_output(format_args!(
            "OPENAI ERROR: {}",
            snapshot.direct_last_error.as_str()
        ));
    }
    if let Some(tcp) = snapshot.tcp {
        write_output(format_args!(
            "TCP: {} SEND {} RECV {}",
            tcp.state,
            yes_no(tcp.may_send),
            yes_no(tcp.may_recv)
        ));
    }
}

fn command_wifi_status() {
    let snapshot = wifi::snapshot();
    write_output(format_args!(
        "WIFI TARGET: {}    SSID: {}    KEY: {}",
        wifi_state_status(snapshot.state),
        wifi_ssid_status(&snapshot.ssid),
        api_key_status(snapshot.passphrase_set)
    ));
    write_output(format_args!(
        "WIFI DRIVER: MARVELL 88W8897 FIRMWARE/SCAN TODO"
    ));
}

fn api_key_status(set: bool) -> &'static str {
    if set {
        "SET"
    } else {
        "MISSING"
    }
}

fn wifi_state_status(state: wifi::WifiState) -> &'static str {
    match state {
        wifi::WifiState::NotProbed => "PENDING",
        wifi::WifiState::Missing => "TARGET MISSING",
        wifi::WifiState::Detected => "TARGET DETECTED",
    }
}

fn wifi_ssid_status(ssid: &wifi::WifiSsid) -> &str {
    if ssid.is_empty() {
        "NONE"
    } else {
        ssid.as_str()
    }
}

fn command_ask(prompt: &str, runtime: ui::RuntimeStatus) {
    submit_prompt(prompt, runtime);
}

fn submit_chat(prompt: ConsoleLine, runtime: ui::RuntimeStatus) {
    submit_prompt(prompt.trimmed_str(), runtime);
}

fn submit_prompt(prompt: &str, runtime: ui::RuntimeStatus) {
    match provider::submit_text(prompt, runtime) {
        Ok(submitted) => {
            let _route = submitted.route;
            push_chat_str(ChatSpeaker::User, prompt);
            write_output(format_args!(
                "OPENAI DIRECT REQUEST {} STARTED",
                submitted.id
            ))
        }
        Err(provider::SubmitError::Empty) => {
            push_chat_args(ChatSpeaker::System, format_args!("ASK REQUIRES TEXT"));
            write_output(format_args!("ASK REQUIRES TEXT"));
        }
        Err(provider::SubmitError::MissingApiKey) => {
            push_chat_args(ChatSpeaker::System, format_args!("OPENAI REQUIRES API KEY"));
            write_output(format_args!("OPENAI REQUIRES API KEY"));
        }
        Err(provider::SubmitError::TrustDenied { state }) => {
            push_chat_args(
                ChatSpeaker::System,
                format_args!("OPENAI TLS TRUST DENIED: {}", state),
            );
            write_output(format_args!("OPENAI TLS TRUST DENIED: {}", state));
        }
        Err(provider::SubmitError::Busy { route, id }) => {
            push_chat_args(
                ChatSpeaker::System,
                format_args!("{} BUSY: REQUEST {} PENDING", route.as_str(), id),
            );
            write_output(format_args!(
                "{} BUSY: REQUEST {} PENDING",
                route.as_str(),
                id
            ));
        }
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "YES"
    } else {
        "NO"
    }
}

fn write_output(args: fmt::Arguments<'_>) {
    let mut line = ConsoleLine::empty();
    let _ = line.write_fmt(args);
    serial::write_line(line.as_str());
    CONSOLE.lock().push_line(line);
}

fn push_chat_args(speaker: ChatSpeaker, args: fmt::Arguments<'_>) {
    let mut line = ConsoleLine::empty();
    let _ = line.write_fmt(args);
    CONSOLE.lock().push_chat(speaker, line);
}

fn push_chat_str(speaker: ChatSpeaker, value: &str) {
    let mut line = ConsoleLine::empty();
    let _ = line.write_str(value);
    CONSOLE.lock().push_chat(speaker, line);
}
