use core::fmt::{self, Write};
use core::str;

use spin::Mutex;

use crate::{
    agent_protocol,
    agent_protocol_support::{
        begin_response, end_response, json_event_id, json_opt_str, json_str, method_eq,
        method_head_eq, raw, raw_bool, raw_line,
    },
    event_log, input, marvell_wifi_pcie, owner_key, provider, provider_config, serial,
    system_status, ui, wifi,
};

const COMMAND_WIDTH: usize = 4096;
const OUTPUT_WIDTH: usize = 2048;
const OUTPUT_LINES: usize = 8;
const CHAT_LINES: usize = 10;
const MAX_BYTES_PER_POLL: usize = 64;
const AGENT_COMMAND_ENVELOPE_METHOD: &str = "agent.command_envelope";
const AGENT_COMMAND_ENVELOPE_SCHEMA: &str = "raios.agent_command_envelope.v0";

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
    pub wifi_passphrase_entry_result: WifiPassphraseEntryResult,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WifiPassphraseEntryResult {
    None,
    Set,
    Rejected,
    Cancelled,
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
    SettingsVault,
    SettingsApiKey,
    SettingsClear,
    SettingsWifiSsid,
    SettingsWifiPassphrase,
    SettingsWifiClear,
    SettingsWifiFirmware,
    SettingsWifiScan,
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
    wifi_passphrase_entry_result: WifiPassphraseEntryResult,
}

impl ConsoleState {
    const fn new() -> Self {
        Self {
            // Genesis owns the only framebuffer shell; keyboard text must reach
            // its Composer immediately. Serial command handling remains separate.
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
            wifi_passphrase_entry_result: WifiPassphraseEntryResult::None,
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
            wifi_passphrase_entry_result: self.wifi_passphrase_entry_result,
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
            // ShellHost consumes physical Enter on this focus before Console.
            // Serial has no Vault action and can only request a redraw here.
            UiFocus::SettingsVault => ByteAction::Redraw,
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
                self.wifi_passphrase_entry_result = WifiPassphraseEntryResult::None;
                ByteAction::ShowWifiPassphraseEntry
            }
            UiFocus::SettingsWifiClear => {
                wifi::clear_config();
                ByteAction::ShowSetupMessage(SetupMessage::WifiConfigCleared)
            }
            UiFocus::SettingsWifiFirmware => ByteAction::StartWifiFirmware,
            UiFocus::SettingsWifiScan => ByteAction::StartWifiScan,
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
            b'7' => ByteAction::StartWifiFirmware,
            b'8' => ByteAction::StartWifiScan,
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
                self.wifi_passphrase_entry_result = if result.is_ok() {
                    WifiPassphraseEntryResult::Set
                } else {
                    WifiPassphraseEntryResult::Rejected
                };
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
                self.wifi_passphrase_entry_result = WifiPassphraseEntryResult::Cancelled;
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
const SETTINGS_FOCUS_ORDER: [UiFocus; 13] = [
    UiFocus::SettingsProvider,
    UiFocus::SettingsVault,
    UiFocus::SettingsApiKey,
    UiFocus::SettingsClear,
    UiFocus::SettingsWifiSsid,
    UiFocus::SettingsWifiPassphrase,
    UiFocus::SettingsWifiClear,
    UiFocus::SettingsWifiFirmware,
    UiFocus::SettingsWifiScan,
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
    StartWifiFirmware,
    StartWifiScan,
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

/// Polls serial plus physical input. The caller receives each physical event
/// first so the core-owned shell can consume secure attention or route input
/// to an active bounded personal surface before console text handling.
pub fn poll<F>(runtime: ui::RuntimeStatus, mut route_input: F) -> bool
where
    F: FnMut(input::InputEvent) -> bool,
{
    let mut changed = false;
    let mut processed = 0usize;

    while processed < MAX_BYTES_PER_POLL {
        let Some(byte) = serial::try_read_byte() else {
            break;
        };

        changed |= process_serial_byte(byte, runtime);
        processed += 1;
    }

    input::drain(|event| {
        if route_input(event) {
            changed = true;
        } else if let Some(input) = input::event_to_console_input(event) {
            changed |= process_input(input, runtime);
        }
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

pub fn activate_focus(focus: UiFocus) -> bool {
    let action = {
        let mut state = CONSOLE.lock();
        state.focus = focus;
        state.activate_focus()
    };
    apply_action(action, ui::RuntimeStatus::new())
}

pub fn submit_wifi_passphrase_entry() -> bool {
    let action = {
        let mut state = CONSOLE.lock();
        if state.mode != ConsoleMode::WifiPassphraseEntry {
            return false;
        }
        state.handle_wifi_passphrase_byte(b'\r')
    };
    apply_action(action, ui::RuntimeStatus::new())
}

pub fn cancel_wifi_passphrase_entry() -> bool {
    let action = {
        let mut state = CONSOLE.lock();
        if state.mode != ConsoleMode::WifiPassphraseEntry {
            return false;
        }
        state.handle_wifi_passphrase_byte(0x1b)
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
        ByteAction::StartWifiFirmware => {
            start_wifi_firmware();
            show_setup_menu();
            true
        }
        ByteAction::StartWifiScan => {
            start_wifi_scan();
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
        let command_mode = state.mode == ConsoleMode::Command;
        (state.handle_serial_byte(byte), command_mode)
    };
    apply_serial_action(action.0, runtime, action.1)
}

fn apply_serial_action(action: ByteAction, runtime: ui::RuntimeStatus, command_mode: bool) -> bool {
    if command_mode {
        match action {
            ByteAction::Echo(byte) => {
                serial::write_byte(byte);
                return false;
            }
            ByteAction::Backspace => {
                serial::write_fmt(format_args!("\x08 \x08"));
                return false;
            }
            ByteAction::Bell => {
                serial::write_byte(0x07);
                return false;
            }
            _ => {}
        }
    }

    apply_action(action, runtime)
}

fn execute(command_line: CommandLine, runtime: ui::RuntimeStatus) {
    let command = command_line.command_word();
    if command.as_str().is_empty() {
        return;
    }

    write_output(format_args!("> {}", command_line.trimmed_str()));

    if let Some(method) =
        agent_protocol::console_dispatch_method(command.as_str(), command_line.trimmed_str())
    {
        command_agent_protocol(method, runtime);
        return;
    }

    match command.as_str() {
        "help" => command_help(),
        "status" => command_status(runtime),
        "devices" => command_devices(runtime),
        "log" => command_log(),
        "agent" => command_agent(command_line.arguments_after_command(), runtime),
        "provider" => command_provider_status(),
        "openai" => command_openai_status(),
        "wifi" => command_wifi_status(),
        "ownerkey" => command_owner_key_status(),
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
        "COMMANDS: help status devices log provider openai wifi ownerkey setup ask <text>"
    ));
    write_output(format_args!(
        "SETUP: key 7 starts WiFi firmware bring-up once; key 8 starts live scan or self-test fallback"
    ));
    write_output(format_args!(
        "AGENT: describe snapshot caps bootlog services problems device.graph memory.profile"
    ));
    write_output(format_args!(
        "AGENT RAW: service.health service.rollback_preview service.rollback_apply recovery.rollback_inspect recovery.rollback_inspect_source_reference_selftest recovery.rollback_materialize_dry_run service.descriptor_source_trust_selftest service.artifact_reference_trust_selftest service.artifact_load_plan_preflight_selftest memory.context provider.context_export provider.context_gate provider.context_gate_selftest provider.context_injection_gate provider.context_injection_gate_selftest memory.query memory.trace memory.recent_events"
    ));
    write_output(format_args!(
        "AGENT ENVELOPE: agent command_envelope schema=raios.agent_command_envelope.v0 target_method=system.describe|system.snapshot|system.boot_log|system.capabilities|device.graph|service.inventory|service.health|service.rollback_preview|recovery.rollback_inspect|module.audit_rollback_availability|module.audit_rollback_write_policy|module.audit_rollback_storage_layout|module.audit_rollback_append_engine|module.audit_rollback_append_contract|module.audit_rollback_append_payload_hash|module.audit_rollback_append_intent|module.audit_rollback_write_boundary|problem.list|recovery.lifeline.status requested_capability=cap.<target>.read classification=local_only"
    ));
    write_output(format_args!(
        "RECOVERY: recovery.load_artifact module.load_recovery_artifact recovery.lifeline_command_admission recovery.lifeline_command_envelope_diagnostic recovery.lifeline_command_dispatch_diagnostic recovery.lifeline_command_body_canonicalization_diagnostic recovery.lifeline_command_handler_binding_diagnostic recovery.lifeline_status_read_handler_diagnostic recovery.rollback_preview_authorization_diagnostic recovery.rollback_apply_authorization_diagnostic recovery.disable_module_target_binding_diagnostic recovery.restart_last_good_target_binding_diagnostic recovery.load_artifact_by_hash_target_binding_diagnostic recovery.memory_write_authority_diagnostic recovery.durable_audit_rollback_write_authority_diagnostic recovery.service_inventory_side_effect_boundary_diagnostic recovery.lifeline_command_dispatch_behavior_diagnostic recovery.lifeline_command_executor_capability_table_diagnostic recovery.lifeline_command_side_effect_gate_diagnostic recovery.lifeline_command_execution_enablement_diagnostic recovery.lifeline_command_execution_preflight_diagnostic recovery.lifeline_command_execution_intent_diagnostic recovery.lifeline_command_execution_commit_gate_diagnostic recovery.lifeline_command_execution_result_denial_diagnostic"
    ));
    write_output(format_args!(
        "RECOVERY EXEC: recovery.lifeline_command_execution_audit_denial_diagnostic recovery.lifeline_command_execution_observation_denial_diagnostic recovery.lifeline_command_execution_completion_denial_diagnostic recovery.lifeline_status_execution_result_diagnostic recovery.lifeline_status_result_read recovery.lifeline.status"
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

fn command_agent(arguments: &str, runtime: ui::RuntimeStatus) {
    let arguments = arguments.trim();
    if method_head_eq(arguments, "command_envelope") {
        command_agent_command_envelope(arguments["command_envelope".len()..].trim(), runtime);
    } else {
        command_agent_protocol(arguments, runtime);
    }
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

#[derive(Clone, Copy)]
struct AgentCommandEnvelope<'a> {
    schema: Option<&'a str>,
    target_method: Option<&'a str>,
    requested_capability: Option<&'a str>,
    classification: Option<&'a str>,
    malformed_token: Option<&'a str>,
    unexpected_field: Option<&'a str>,
    duplicate_field: Option<&'a str>,
}

impl<'a> AgentCommandEnvelope<'a> {
    const fn empty() -> Self {
        Self {
            schema: None,
            target_method: None,
            requested_capability: None,
            classification: None,
            malformed_token: None,
            unexpected_field: None,
            duplicate_field: None,
        }
    }
}

fn command_agent_command_envelope(arguments: &str, runtime: ui::RuntimeStatus) {
    let envelope = parse_agent_command_envelope(arguments);
    let reason = agent_command_envelope_reason(envelope);
    let accepted = reason == "accepted";
    let binding = agent_command_envelope_event_binding(envelope, reason, accepted);
    let event_id = event_log::record_agent_command_envelope_decision(binding);
    emit_agent_command_envelope(envelope, reason, accepted, event_id);
    if accepted {
        record_event(format_args!("AGENT COMMAND ENVELOPE ACCEPTED"));
        command_agent_protocol(agent_command_envelope_dispatch_method(envelope), runtime);
    } else {
        record_event(format_args!("AGENT COMMAND ENVELOPE DENIED"));
        serial::write_line("AGENT COMMAND ENVELOPE DENIED");
    }
}

fn agent_command_envelope_event_binding(
    envelope: AgentCommandEnvelope<'_>,
    reason: &'static str,
    accepted: bool,
) -> event_log::AgentCommandEnvelopeBinding {
    let target_method = agent_command_envelope_event_target(envelope.target_method);
    let requested_capability =
        agent_command_envelope_event_capability(envelope.requested_capability);
    let submitted_classification =
        agent_command_envelope_event_classification(envelope.classification);
    event_log::AgentCommandEnvelopeBinding {
        schema_ok: method_eq(envelope.schema.unwrap_or(""), AGENT_COMMAND_ENVELOPE_SCHEMA),
        target_method,
        target_method_allowed: agent_command_envelope_allowed_target(envelope.target_method)
            .is_some(),
        requested_capability,
        requested_capability_allowed: agent_command_envelope_capability_allowed(envelope),
        submitted_classification,
        classification_allowed: method_eq(envelope.classification.unwrap_or(""), "local_only"),
        accepted,
        code: agent_command_envelope_code(reason),
        reason,
        dispatches_existing_agent_method: accepted,
        creates_parallel_dispatcher: false,
        provider_write: "not_attempted",
        loads_candidate_bytes: false,
        writes_persistent_state: false,
        writes_durable_audit_log: false,
        installs_rollback_plan: false,
        grants_broad_mutation: false,
    }
}

fn agent_command_envelope_lookup_allowed_target(
    value: Option<&str>,
) -> Option<agent_protocol::CommandEnvelopeTarget> {
    agent_protocol::command_envelope_target(value)
}

fn agent_command_envelope_lookup_allowed_capability(value: Option<&str>) -> Option<&'static str> {
    agent_protocol::command_envelope_capability(value)
}

fn agent_command_envelope_event_target(value: Option<&str>) -> Option<&'static str> {
    if let Some(target) = agent_command_envelope_lookup_allowed_target(value) {
        Some(target.method)
    } else if method_eq(value.unwrap_or(""), "module.load_ephemeral") {
        Some("module.load_ephemeral")
    } else {
        None
    }
}

fn agent_command_envelope_allowed_target(value: Option<&str>) -> Option<&'static str> {
    if let Some(target) = agent_command_envelope_lookup_allowed_target(value) {
        Some(target.method)
    } else {
        None
    }
}

fn agent_command_envelope_dispatch_method(envelope: AgentCommandEnvelope<'_>) -> &'static str {
    if let Some(target) = agent_command_envelope_lookup_allowed_target(envelope.target_method) {
        target.dispatch_method
    } else {
        agent_command_envelope_default_target().dispatch_method
    }
}

fn agent_command_envelope_event_capability(value: Option<&str>) -> Option<&'static str> {
    if let Some(capability) = agent_command_envelope_lookup_allowed_capability(value) {
        Some(capability)
    } else if method_eq(value.unwrap_or(""), "cap.module.load_ephemeral") {
        Some("cap.module.load_ephemeral")
    } else {
        None
    }
}

fn agent_command_envelope_event_classification(value: Option<&str>) -> Option<&'static str> {
    let value = value.unwrap_or("");
    if method_eq(value, "local_only") {
        Some("local_only")
    } else {
        None
    }
}

fn agent_command_envelope_expected_capability(envelope: AgentCommandEnvelope<'_>) -> &'static str {
    if let Some(target) = agent_command_envelope_lookup_allowed_target(envelope.target_method) {
        target.capability
    } else {
        agent_command_envelope_default_target().capability
    }
}

fn agent_command_envelope_capability_allowed(envelope: AgentCommandEnvelope<'_>) -> bool {
    agent_command_envelope_allowed_target(envelope.target_method).is_some()
        && method_eq(
            envelope.requested_capability.unwrap_or(""),
            agent_command_envelope_expected_capability(envelope),
        )
}

fn parse_agent_command_envelope(arguments: &str) -> AgentCommandEnvelope<'_> {
    let mut envelope = AgentCommandEnvelope::empty();
    for token in arguments.split_ascii_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            envelope.malformed_token = envelope.malformed_token.or(Some(token));
            continue;
        };
        if method_eq(key, "schema") {
            if envelope.schema.replace(value).is_some() {
                envelope.duplicate_field = envelope.duplicate_field.or(Some(key));
            }
        } else if method_eq(key, "target_method") {
            if envelope.target_method.replace(value).is_some() {
                envelope.duplicate_field = envelope.duplicate_field.or(Some(key));
            }
        } else if method_eq(key, "requested_capability") {
            if envelope.requested_capability.replace(value).is_some() {
                envelope.duplicate_field = envelope.duplicate_field.or(Some(key));
            }
        } else if method_eq(key, "classification") {
            if envelope.classification.replace(value).is_some() {
                envelope.duplicate_field = envelope.duplicate_field.or(Some(key));
            }
        } else {
            envelope.unexpected_field = envelope.unexpected_field.or(Some(key));
        }
    }
    envelope
}

fn agent_command_envelope_reason(envelope: AgentCommandEnvelope<'_>) -> &'static str {
    if envelope.malformed_token.is_some() {
        "malformed_key_value"
    } else if envelope.unexpected_field.is_some() {
        "unexpected_field"
    } else if envelope.duplicate_field.is_some() {
        "duplicate_field"
    } else if envelope.schema.is_none()
        || envelope.target_method.is_none()
        || envelope.requested_capability.is_none()
        || envelope.classification.is_none()
    {
        "missing_required_field"
    } else if !method_eq(envelope.schema.unwrap_or(""), AGENT_COMMAND_ENVELOPE_SCHEMA) {
        "schema_mismatch"
    } else if !method_eq(envelope.classification.unwrap_or(""), "local_only") {
        "classification_denied"
    } else if agent_command_envelope_allowed_target(envelope.target_method).is_none() {
        "target_method_not_allowed"
    } else if !agent_command_envelope_capability_allowed(envelope) {
        "requested_capability_denied"
    } else {
        "accepted"
    }
}

fn agent_command_envelope_code(reason: &'static str) -> &'static str {
    if reason == "accepted" {
        "accepted"
    } else if reason == "target_method_not_allowed"
        || reason == "requested_capability_denied"
        || reason == "classification_denied"
    {
        "capability_denied"
    } else {
        "invalid_envelope"
    }
}

fn emit_agent_command_envelope(
    envelope: AgentCommandEnvelope<'_>,
    reason: &'static str,
    accepted: bool,
    event_id: event_log::EventId,
) {
    begin_response(AGENT_COMMAND_ENVELOPE_METHOD);
    raw("      \"schema\": ");
    json_str(AGENT_COMMAND_ENVELOPE_SCHEMA);
    raw_line(",");
    raw("      \"id\": ");
    json_str(agent_command_envelope_response_id(envelope));
    raw_line(",");
    raw("      \"event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw("      \"audit_event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"transport\": \"serial-console\",");
    raw("      \"accepted\": ");
    raw_bool(accepted);
    raw_line(",");
    raw("      \"code\": ");
    json_str(agent_command_envelope_code(reason));
    raw_line(",");
    raw("      \"reason\": ");
    json_str(reason);
    raw_line(",");
    raw("      \"submitted_schema\": ");
    json_opt_str(envelope.schema);
    raw_line(",");
    raw("      \"target_method\": ");
    json_opt_str(envelope.target_method);
    raw_line(",");
    raw("      \"requested_capability\": ");
    json_opt_str(envelope.requested_capability);
    raw_line(",");
    raw("      \"submitted_classification\": ");
    json_opt_str(envelope.classification);
    raw_line(",");
    raw("      \"allowed_target_method\": ");
    json_str(agent_command_envelope_default_target().method);
    raw_line(",");
    raw("      \"allowed_target_methods\": [");
    let mut allowed_index = 0usize;
    while allowed_index < agent_protocol::command_envelope_target_count() {
        if allowed_index > 0 {
            raw(", ");
        }
        if let Some(target) = agent_protocol::command_envelope_target_at(allowed_index) {
            json_str(target.method);
        }
        allowed_index += 1;
    }
    raw_line("],");
    raw("      \"allowed_requested_capability\": ");
    json_str(agent_command_envelope_expected_capability(envelope));
    raw_line(",");
    raw_line(
        "      \"target_allowlist\": \"system_describe_system_snapshot_system_boot_log_system_capabilities_device_graph_service_inventory_service_health_service_rollback_preview_recovery_rollback_inspect_module_audit_rollback_availability_module_audit_rollback_write_policy_module_audit_rollback_storage_layout_module_audit_rollback_append_engine_module_audit_rollback_append_contract_module_audit_rollback_append_payload_hash_module_audit_rollback_append_intent_module_audit_rollback_write_boundary_problem_list_recovery_lifeline_status_read_only\",",
    );
    raw("      \"dispatches_existing_agent_method\": ");
    raw_bool(accepted);
    raw_line(",");
    raw_line("      \"creates_parallel_dispatcher\": false,");
    raw_line("      \"provider_write\": \"not_attempted\",");
    raw_line("      \"loads_candidate_bytes\": false,");
    raw_line("      \"writes_persistent_state\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"grants_broad_mutation\": false");
    end_response(AGENT_COMMAND_ENVELOPE_METHOD);
}

fn agent_command_envelope_response_id(envelope: AgentCommandEnvelope<'_>) -> &'static str {
    if let Some(target) = agent_command_envelope_lookup_allowed_target(envelope.target_method) {
        target.response_id
    } else {
        agent_command_envelope_default_target().response_id
    }
}

fn agent_command_envelope_default_target() -> agent_protocol::CommandEnvelopeTarget {
    agent_protocol::command_envelope_target_at(0).unwrap_or(agent_protocol::CommandEnvelopeTarget {
        method: "system.describe",
        capability: "cap.system.describe.read",
        response_id: "agent_command_envelope.current_boot.serial.system_describe.v0",
        dispatch_method: "system.describe",
    })
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
        "5 WIFI KEY: {}    6 CLEAR WIFI",
        api_key_status(wifi.passphrase_set)
    ));
    write_output(format_args!("7 START WIFI FW    8 SCAN NETWORKS    Q EXIT"));
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

fn start_wifi_firmware() {
    write_output(format_args!(
        "WIFI FIRMWARE BRING-UP ATTEMPT (unaudited blob; DMA not IOMMU-confined)"
    ));
    let result = marvell_wifi_pcie::start_bring_up_firmware();
    write_output(format_args!("WIFI FIRMWARE START: {}", result.label()));
    if let marvell_wifi_pcie::FirmwareBringupTriggerResult::Failed(error) = result {
        write_output(format_args!("WIFI FIRMWARE FAILED: {}", error.label()));
    }
}

fn start_wifi_scan() {
    write_output(format_args!("WIFI SCAN START REQUEST"));
    let result = marvell_wifi_pcie::start_scan_ext_24ghz();
    write_output(format_args!("WIFI SCAN CMD: {}", result.label()));
    match result {
        marvell_wifi_pcie::ScanCmdTriggerResult::Started => {
            write_output(format_args!(
                "WIFI SCAN: LEGACY RESPONSE SCAN SENT NEXT TICK"
            ));
        }
        marvell_wifi_pcie::ScanCmdTriggerResult::AlreadyRunning => {
            write_output(format_args!("WIFI SCAN: COMMAND ALREADY RUNNING"));
        }
        marvell_wifi_pcie::ScanCmdTriggerResult::Failed(error) => {
            write_output(format_args!(
                "WIFI SCAN LIVE NOT STARTED: {}; RUNNING SELF-TEST FALLBACK",
                error.label()
            ));
            wifi::run_scan_selftest();
            write_output(format_args!(
                "WIFI SCAN SELF-TEST RUN (LIVE SCAN NOT STARTED)"
            ));
        }
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
    let firmware = marvell_wifi_pcie::snapshot();
    let hw_spec = marvell_wifi_pcie::hw_spec_snapshot();
    let scan_cmd = marvell_wifi_pcie::scan_cmd_snapshot();
    let event_ring = marvell_wifi_pcie::event_ring_snapshot();
    let rx_ring = marvell_wifi_pcie::rx_ring_snapshot();
    write_output(format_args!(
        "WIFI TARGET: {}    SSID: {}    KEY: {}",
        wifi_state_status(snapshot.state),
        wifi_ssid_status(&snapshot.ssid),
        api_key_status(snapshot.passphrase_set)
    ));
    write_output(format_args!(
        "WIFI FW: {} {}/{} RESULT {}",
        firmware.stage.label(),
        firmware.downloaded,
        firmware.total,
        firmware
            .result
            .map(|result| result.label())
            .unwrap_or("pending")
    ));
    write_output(format_args!(
        "WIFI HW_SPEC: {} RESULT {}",
        hw_spec.stage.label(),
        hw_spec
            .result
            .map(|result| result.label())
            .unwrap_or("pending")
    ));
    if scan_cmd.attempted {
        write_output(format_args!(
            "WIFI SCAN CMD: {} RESULT {} LEN {} HOST_INT 0x{:08X}",
            scan_cmd.stage.label(),
            scan_cmd
                .result
                .map(|result| result.label())
                .unwrap_or("pending"),
            scan_cmd.command_len,
            scan_cmd.host_int_status
        ));
    }
    if event_ring.attempted {
        write_output(format_args!(
            "WIFI EVENT RING: {} RESULT {} RD 0x{:X} WR 0x{:X} TYPE 0x{:04X} CAUSE 0x{:08X} LEN {} HOST_INT 0x{:08X}",
            event_ring.stage.label(),
            event_ring
                .result
                .map(|result| result.label())
                .unwrap_or("pending"),
            event_ring.rdptr,
            event_ring.wrptr,
            event_ring.event_type,
            event_ring.event_cause,
            event_ring.event_len,
            event_ring.host_int_status
        ));
    }
    if rx_ring.attempted {
        write_output(format_args!(
            "WIFI RX RING: {} RESULT {} RD 0x{:X} WR 0x{:X} TYPE 0x{:04X} LEN {} HOST_INT 0x{:08X}",
            rx_ring.stage.label(),
            rx_ring
                .result
                .map(|result| result.label())
                .unwrap_or("pending"),
            rx_ring.rdptr,
            rx_ring.wrptr,
            rx_ring.rx_type,
            rx_ring.rx_len,
            rx_ring.host_int_status
        ));
    }
    if firmware.registers.valid {
        write_output(format_args!(
            "WIFI FW REGS: C40=0x{:08X} C44=0x{:08X} CF0=0x{:08X} C30=0x{:08X}",
            firmware.registers.cmd_size,
            firmware.registers.fw_status,
            firmware.registers.drv_ready,
            firmware.registers.host_int_status
        ));
    }
}

fn command_owner_key_status() {
    let snapshot = owner_key::snapshot();
    let hardware = snapshot.hardware_binding;
    write_output(format_args!(
        "OWNER KEY: RAM {} HANDLE {}",
        yes_no(snapshot.generated),
        snapshot.handle.unwrap_or("NONE")
    ));
    write_owner_key_fingerprint(snapshot.fingerprint);
    write_output(format_args!(
        "TPM2 ACPI: PRESENT {} PHYS 0x{:016X} LEN {} REV {}",
        yes_no(hardware.tpm2_acpi_table_present),
        hardware.tpm2_acpi_table_phys,
        hardware.tpm2_acpi_table_length,
        hardware.tpm2_acpi_table_revision
    ));
    write_output(format_args!(
        "TPM2 IFACE: KIND {} START {} CONTROL 0x{:016X} DETAILS {}",
        hardware.tpm2_interface_kind,
        hardware.tpm2_start_method,
        hardware.tpm2_control_area,
        yes_no(hardware.tpm2_table_details_valid)
    ));
    write_output(format_args!(
        "TPM2 STATUS: {} REASON {}",
        hardware.tpm2_interface_status, hardware.tpm2_interface_status_reason
    ));
    write_output(format_args!(
        "TPM2 STATUS READ: PLAN {} KIND {} PHYS 0x{:016X} WIDTH {} REASON {}",
        yes_no(hardware.tpm2_status_read_plan_available),
        hardware.tpm2_status_register_kind,
        hardware.tpm2_status_register_phys,
        hardware.tpm2_status_register_width_bytes,
        hardware.tpm2_status_read_plan_reason
    ));
    write_output(format_args!(
        "OWNER AUTH: SEAL NO PERSIST NO LOAD NO DURABLE NO"
    ));
}

fn write_owner_key_fingerprint(fingerprint: Option<[u8; 32]>) {
    let Some(fingerprint) = fingerprint else {
        write_output(format_args!("OWNER KEY FINGERPRINT: NONE"));
        return;
    };

    let mut line = ConsoleLine::empty();
    let _ = line.write_str("OWNER KEY FINGERPRINT: sha256:");
    for byte in fingerprint {
        let _ = write!(line, "{:02x}", byte);
    }
    write_output_line(line);
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
    write_output_line(line);
}

fn write_output_line(line: ConsoleLine) {
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
