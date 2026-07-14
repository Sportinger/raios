//! Bounded, authority-free UI programs authored as data.
//!
//! A program owns static widgets, key/pointer routing, numeric state, one
//! bounded editable text buffer, and guarded arithmetic rules. Parsing and
//! dispatch are atomic and fail closed; this module neither persists state nor
//! grants capabilities.

extern crate alloc;

use alloc::{string::String, vec, vec::Vec};
use core::str;

use crate::sha256_bytes;

pub const PROGRAM_ABI_VERSION: u16 = 1;
pub const MAX_PROGRAM_BYTES: usize = 16 * 1024;
pub const MAX_STATE_SLOTS: usize = 16;
pub const MAX_WIDGETS: usize = 64;
pub const MAX_BINDINGS: usize = 64;
pub const MAX_RULES: usize = 128;
pub const MAX_CONDITIONS_PER_RULE: usize = 2;
pub const MAX_ACTIONS_PER_RULE: usize = 8;
pub const MAX_TEXT_BYTES: usize = 2 * 1024;
pub const MAX_WIDGET_TEXT_BYTES: usize = 64;
pub const MAX_TEXT_SLOTS: usize = 1;
pub const MAX_TEXT_SLOT_CAPACITY: usize = 2 * 1024;
pub const MAX_EVENT_ID: u16 = 64;

const MAGIC: &[u8; 4] = b"RUIP";
const HEADER_LEN: usize = 32;
const WIDGET_PREFIX_LEN: usize = 16;
const BINDING_LEN: usize = 8;
const RULE_PREFIX_LEN: usize = 32;
const ACTION_LEN: usize = 24;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgramError {
    WrongAbiVersion,
    Malformed,
    LimitExceeded,
    UnknownOpcode,
    InvalidReference,
    DuplicateBinding,
    OverlappingButtons,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeError {
    StateMismatch,
    AmbiguousEvent,
    Overflow,
    DivisionByZero,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    fn contains(self, x: u16, y: u16) -> bool {
        let right = self.x.saturating_add(self.width);
        let bottom = self.y.saturating_add(self.height);
        x >= self.x && x < right && y >= self.y && y < bottom
    }

    fn overlaps(self, other: Self) -> bool {
        self.x < other.x.saturating_add(other.width)
            && other.x < self.x.saturating_add(self.width)
            && self.y < other.y.saturating_add(other.height)
            && other.y < self.y.saturating_add(self.height)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Widget {
    Text {
        rect: Rect,
        text: String,
    },
    Display {
        rect: Rect,
        state_slot: u8,
    },
    Button {
        rect: Rect,
        event_id: u16,
        text: String,
    },
    EditBox {
        rect: Rect,
        text_slot: u8,
    },
}

impl Widget {
    pub const fn rect(&self) -> Rect {
        match self {
            Self::Text { rect, .. }
            | Self::Display { rect, .. }
            | Self::Button { rect, .. }
            | Self::EditBox { rect, .. } => *rect,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyBinding {
    pub code: u16,
    pub modifiers: u16,
    pub event_id: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConditionOp {
    Equal,
    NotEqual,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Condition {
    pub state_slot: u8,
    pub op: ConditionOp,
    pub value: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Set {
        dst: u8,
        value: i64,
    },
    Copy {
        dst: u8,
        src: u8,
    },
    Add {
        dst: u8,
        left: u8,
        right: u8,
    },
    Subtract {
        dst: u8,
        left: u8,
        right: u8,
    },
    Multiply {
        dst: u8,
        left: u8,
        right: u8,
    },
    Divide {
        dst: u8,
        left: u8,
        right: u8,
    },
    MulAdd {
        dst: u8,
        src: u8,
        multiplier: i64,
        addend: i64,
    },
    TextClear {
        slot: u8,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rule {
    pub event_id: u16,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    initial_state: Vec<i64>,
    text_slot_capacities: Vec<u16>,
    widgets: Vec<Widget>,
    bindings: Vec<KeyBinding>,
    rules: Vec<Rule>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProgramIdentity {
    pub sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiInput {
    Key { code: u16, modifiers: u16 },
    PointerClick { x: u16, y: u16 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispatchOutcome {
    Ignored,
    Handled { event_id: u16 },
    Edited,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProgramState {
    values: [i64; MAX_STATE_SLOTS],
    len: u8,
    text_buffer: [u8; MAX_TEXT_SLOT_CAPACITY],
    text_len: u16,
}

impl ProgramState {
    pub fn from_values(values: &[i64]) -> Result<Self, RuntimeError> {
        if values.is_empty() || values.len() > MAX_STATE_SLOTS {
            return Err(RuntimeError::StateMismatch);
        }
        let mut state = Self {
            values: [0; MAX_STATE_SLOTS],
            len: values.len() as u8,
            text_buffer: [0; MAX_TEXT_SLOT_CAPACITY],
            text_len: 0,
        };
        state.values[..values.len()].copy_from_slice(values);
        Ok(state)
    }

    pub fn values(&self) -> &[i64] {
        &self.values[..self.len as usize]
    }

    pub fn get(&self, slot: usize) -> Option<i64> {
        self.values().get(slot).copied()
    }

    pub fn text(&self) -> &[u8] {
        &self.text_buffer[..self.text_len as usize]
    }

    pub fn from_values_and_text(values: &[i64], text: &[u8]) -> Result<Self, RuntimeError> {
        if text.len() > MAX_TEXT_SLOT_CAPACITY {
            return Err(RuntimeError::StateMismatch);
        }
        let mut state = Self::from_values(values)?;
        state.text_buffer[..text.len()].copy_from_slice(text);
        state.text_len = text.len() as u16;
        Ok(state)
    }
}

impl Program {
    pub fn new(
        initial_state: Vec<i64>,
        widgets: Vec<Widget>,
        bindings: Vec<KeyBinding>,
        rules: Vec<Rule>,
    ) -> Result<Self, ProgramError> {
        Self::new_with_text_slots(initial_state, Vec::new(), widgets, bindings, rules)
    }

    pub fn new_with_text_slots(
        initial_state: Vec<i64>,
        text_slot_capacities: Vec<u16>,
        widgets: Vec<Widget>,
        bindings: Vec<KeyBinding>,
        rules: Vec<Rule>,
    ) -> Result<Self, ProgramError> {
        let program = Self {
            initial_state,
            text_slot_capacities,
            widgets,
            bindings,
            rules,
        };
        program.validate()?;
        if program.encode_unchecked().len() > MAX_PROGRAM_BYTES {
            return Err(ProgramError::LimitExceeded);
        }
        Ok(program)
    }

    pub fn widgets(&self) -> &[Widget] {
        &self.widgets
    }

    pub fn bindings(&self) -> &[KeyBinding] {
        &self.bindings
    }

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    pub fn text_slot_capacity(&self) -> Option<u16> {
        self.text_slot_capacities.first().copied()
    }

    pub fn has_edit_box(&self) -> bool {
        self.widgets
            .iter()
            .any(|widget| matches!(widget, Widget::EditBox { .. }))
    }

    pub fn initial_state(&self) -> ProgramState {
        let mut values = [0; MAX_STATE_SLOTS];
        values[..self.initial_state.len()].copy_from_slice(&self.initial_state);
        ProgramState {
            values,
            len: self.initial_state.len() as u8,
            text_buffer: [0; MAX_TEXT_SLOT_CAPACITY],
            text_len: 0,
        }
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        self.encode_unchecked()
    }

    pub fn identity(&self) -> ProgramIdentity {
        ProgramIdentity {
            sha256: sha256_bytes(&self.canonical_bytes()),
        }
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, ProgramError> {
        if bytes.len() > MAX_PROGRAM_BYTES {
            return Err(ProgramError::LimitExceeded);
        }
        if bytes.len() < HEADER_LEN || bytes.get(..4) != Some(MAGIC) {
            return Err(ProgramError::Malformed);
        }
        if read_u16(bytes, 4)? != PROGRAM_ABI_VERSION {
            return Err(ProgramError::WrongAbiVersion);
        }
        if read_u16(bytes, 6)? as usize != HEADER_LEN
            || read_u32(bytes, 8)? as usize != bytes.len()
            || bytes[17..HEADER_LEN].iter().any(|byte| *byte != 0)
        {
            return Err(ProgramError::Malformed);
        }
        let state_count = bytes[12] as usize;
        let widget_count = bytes[13] as usize;
        let binding_count = bytes[14] as usize;
        let rule_count = bytes[15] as usize;
        let text_slot_count = bytes[16] as usize;
        check_counts(
            state_count,
            text_slot_count,
            widget_count,
            binding_count,
            rule_count,
        )?;

        let mut cursor = Cursor::new(bytes, HEADER_LEN);
        let mut initial_state = Vec::with_capacity(state_count);
        for _ in 0..state_count {
            initial_state.push(cursor.i64()?);
        }
        let mut text_slot_capacities = Vec::with_capacity(text_slot_count);
        for _ in 0..text_slot_count {
            let slot = cursor.take(4)?;
            if read_u16(slot, 2)? != 0 {
                return Err(ProgramError::Malformed);
            }
            let capacity = read_u16(slot, 0)?;
            check_text_slot_capacity(capacity)?;
            text_slot_capacities.push(capacity);
        }
        let mut widgets = Vec::with_capacity(widget_count);
        for _ in 0..widget_count {
            widgets.push(parse_widget(&mut cursor)?);
        }
        let mut bindings = Vec::with_capacity(binding_count);
        for _ in 0..binding_count {
            let chunk = cursor.take(BINDING_LEN)?;
            if read_u16(chunk, 6)? != 0 {
                return Err(ProgramError::Malformed);
            }
            bindings.push(KeyBinding {
                code: read_u16(chunk, 0)?,
                modifiers: read_u16(chunk, 2)?,
                event_id: read_u16(chunk, 4)?,
            });
        }
        let mut rules = Vec::with_capacity(rule_count);
        for _ in 0..rule_count {
            rules.push(parse_rule(&mut cursor)?);
        }
        if cursor.position != bytes.len() {
            return Err(ProgramError::Malformed);
        }
        let program = Self::new_with_text_slots(
            initial_state,
            text_slot_capacities,
            widgets,
            bindings,
            rules,
        )?;
        if program.canonical_bytes().as_slice() != bytes {
            return Err(ProgramError::Malformed);
        }
        Ok(program)
    }

    pub fn dispatch(
        &self,
        state: &mut ProgramState,
        input: UiInput,
    ) -> Result<DispatchOutcome, RuntimeError> {
        if state.len as usize != self.initial_state.len()
            || match self.text_slot_capacity() {
                Some(capacity) => state.text_len > capacity,
                None => state.text_len != 0,
            }
        {
            return Err(RuntimeError::StateMismatch);
        }
        if let Some(event_id) = self.resolve_event(input)? {
            let mut selected = None;
            for rule in &self.rules {
                if rule.event_id == event_id && conditions_match(&rule.conditions, state) {
                    if selected.is_some() {
                        return Err(RuntimeError::AmbiguousEvent);
                    }
                    selected = Some(rule);
                }
            }
            let Some(rule) = selected else {
                return Ok(DispatchOutcome::Ignored);
            };
            let mut candidate = *state;
            for action in &rule.actions {
                apply_action(*action, &mut candidate)?;
            }
            *state = candidate;
            return Ok(DispatchOutcome::Handled { event_id });
        }
        if !self.has_edit_box() {
            return Ok(DispatchOutcome::Ignored);
        }
        let mut candidate = *state;
        match input {
            UiInput::Key {
                code: 8,
                modifiers: 0,
            } if candidate.text_len != 0 => {
                candidate.text_len -= 1;
            }
            UiInput::Key {
                code: 13,
                modifiers: 0,
            } if candidate.text_len < self.text_slot_capacity().unwrap_or(0) => {
                candidate.text_buffer[candidate.text_len as usize] = b'\n';
                candidate.text_len += 1;
            }
            UiInput::Key {
                code: code @ 32..=126,
                modifiers: 0,
            } if candidate.text_len < self.text_slot_capacity().unwrap_or(0) => {
                candidate.text_buffer[candidate.text_len as usize] = code as u8;
                candidate.text_len += 1;
            }
            _ => return Ok(DispatchOutcome::Ignored),
        }
        *state = candidate;
        Ok(DispatchOutcome::Edited)
    }

    fn resolve_event(&self, input: UiInput) -> Result<Option<u16>, RuntimeError> {
        let mut found = None;
        match input {
            UiInput::Key { code, modifiers } => {
                for binding in &self.bindings {
                    if binding.code == code && binding.modifiers == modifiers {
                        if found.replace(binding.event_id).is_some() {
                            return Err(RuntimeError::AmbiguousEvent);
                        }
                    }
                }
            }
            UiInput::PointerClick { x, y } => {
                for widget in &self.widgets {
                    if let Widget::Button { rect, event_id, .. } = widget {
                        if rect.contains(x, y) && found.replace(*event_id).is_some() {
                            return Err(RuntimeError::AmbiguousEvent);
                        }
                    }
                }
            }
        }
        Ok(found)
    }

    fn validate(&self) -> Result<(), ProgramError> {
        check_counts(
            self.initial_state.len(),
            self.text_slot_capacities.len(),
            self.widgets.len(),
            self.bindings.len(),
            self.rules.len(),
        )?;
        for capacity in &self.text_slot_capacities {
            check_text_slot_capacity(*capacity)?;
        }
        let mut total_text = 0usize;
        let mut edit_box_count = 0usize;
        for (index, widget) in self.widgets.iter().enumerate() {
            let rect = widget.rect();
            if rect.width == 0 || rect.height == 0 {
                return Err(ProgramError::InvalidReference);
            }
            match widget {
                Widget::Text { text, .. } => validate_text(text, &mut total_text)?,
                Widget::Display { state_slot, .. } => self.check_slot(*state_slot)?,
                Widget::Button { event_id, text, .. } => {
                    check_event(*event_id)?;
                    validate_text(text, &mut total_text)?;
                    for other in self.widgets.iter().take(index) {
                        if let Widget::Button {
                            rect: other_rect, ..
                        } = other
                        {
                            if rect.overlaps(*other_rect) {
                                return Err(ProgramError::OverlappingButtons);
                            }
                        }
                    }
                }
                Widget::EditBox { text_slot, .. } => {
                    edit_box_count += 1;
                    self.check_text_slot(*text_slot)?;
                }
            }
        }
        if edit_box_count > 1 {
            return Err(ProgramError::LimitExceeded);
        }
        if total_text > MAX_TEXT_BYTES {
            return Err(ProgramError::LimitExceeded);
        }
        for (index, binding) in self.bindings.iter().enumerate() {
            check_event(binding.event_id)?;
            if self.bindings[..index]
                .iter()
                .any(|prior| prior.code == binding.code && prior.modifiers == binding.modifiers)
            {
                return Err(ProgramError::DuplicateBinding);
            }
        }
        for rule in &self.rules {
            check_event(rule.event_id)?;
            if rule.conditions.len() > MAX_CONDITIONS_PER_RULE
                || rule.actions.is_empty()
                || rule.actions.len() > MAX_ACTIONS_PER_RULE
            {
                return Err(ProgramError::LimitExceeded);
            }
            for condition in &rule.conditions {
                self.check_slot(condition.state_slot)?;
            }
            for action in &rule.actions {
                self.validate_action(*action)?;
            }
        }
        for widget in &self.widgets {
            if let Widget::Button { event_id, .. } = widget {
                if !self.rules.iter().any(|rule| rule.event_id == *event_id) {
                    return Err(ProgramError::InvalidReference);
                }
            }
        }
        for binding in &self.bindings {
            if !self
                .rules
                .iter()
                .any(|rule| rule.event_id == binding.event_id)
            {
                return Err(ProgramError::InvalidReference);
            }
        }
        Ok(())
    }

    fn check_slot(&self, slot: u8) -> Result<(), ProgramError> {
        if (slot as usize) < self.initial_state.len() {
            Ok(())
        } else {
            Err(ProgramError::InvalidReference)
        }
    }

    fn check_text_slot(&self, slot: u8) -> Result<(), ProgramError> {
        if (slot as usize) < self.text_slot_capacities.len() {
            Ok(())
        } else {
            Err(ProgramError::InvalidReference)
        }
    }

    fn validate_action(&self, action: Action) -> Result<(), ProgramError> {
        match action {
            Action::Set { dst, .. } => self.check_slot(dst),
            Action::Copy { dst, src } => {
                self.check_slot(dst)?;
                self.check_slot(src)
            }
            Action::Add { dst, left, right }
            | Action::Subtract { dst, left, right }
            | Action::Multiply { dst, left, right }
            | Action::Divide { dst, left, right } => {
                self.check_slot(dst)?;
                self.check_slot(left)?;
                self.check_slot(right)
            }
            Action::MulAdd { dst, src, .. } => {
                self.check_slot(dst)?;
                self.check_slot(src)
            }
            Action::TextClear { slot } => self.check_text_slot(slot),
        }
    }

    fn encode_unchecked(&self) -> Vec<u8> {
        let mut out = vec![0; HEADER_LEN];
        out[..4].copy_from_slice(MAGIC);
        put_u16(&mut out, 4, PROGRAM_ABI_VERSION);
        put_u16(&mut out, 6, HEADER_LEN as u16);
        out[12] = self.initial_state.len() as u8;
        out[13] = self.widgets.len() as u8;
        out[14] = self.bindings.len() as u8;
        out[15] = self.rules.len() as u8;
        out[16] = self.text_slot_capacities.len() as u8;
        for value in &self.initial_state {
            out.extend_from_slice(&value.to_le_bytes());
        }
        for capacity in &self.text_slot_capacities {
            out.extend_from_slice(&capacity.to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
        }
        for widget in &self.widgets {
            encode_widget(widget, &mut out);
        }
        for binding in &self.bindings {
            out.extend_from_slice(&binding.code.to_le_bytes());
            out.extend_from_slice(&binding.modifiers.to_le_bytes());
            out.extend_from_slice(&binding.event_id.to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
        }
        for rule in &self.rules {
            encode_rule(rule, &mut out);
        }
        let total_len = out.len() as u32;
        put_u32(&mut out, 8, total_len);
        out
    }
}

fn check_counts(
    states: usize,
    text_slots: usize,
    widgets: usize,
    bindings: usize,
    rules: usize,
) -> Result<(), ProgramError> {
    if states == 0
        || states > MAX_STATE_SLOTS
        || text_slots > MAX_TEXT_SLOTS
        || widgets > MAX_WIDGETS
        || bindings > MAX_BINDINGS
        || rules > MAX_RULES
    {
        Err(ProgramError::LimitExceeded)
    } else {
        Ok(())
    }
}

fn check_text_slot_capacity(capacity: u16) -> Result<(), ProgramError> {
    if capacity == 0 || capacity as usize > MAX_TEXT_SLOT_CAPACITY {
        Err(ProgramError::LimitExceeded)
    } else {
        Ok(())
    }
}

fn check_event(event_id: u16) -> Result<(), ProgramError> {
    if event_id == 0 || event_id > MAX_EVENT_ID {
        Err(ProgramError::InvalidReference)
    } else {
        Ok(())
    }
}

fn validate_text(text: &str, total: &mut usize) -> Result<(), ProgramError> {
    if text.is_empty() || text.len() > MAX_WIDGET_TEXT_BYTES {
        return Err(ProgramError::LimitExceeded);
    }
    *total = total.saturating_add(text.len());
    Ok(())
}

fn conditions_match(conditions: &[Condition], state: &ProgramState) -> bool {
    conditions.iter().all(|condition| {
        let equal = state.values[condition.state_slot as usize] == condition.value;
        match condition.op {
            ConditionOp::Equal => equal,
            ConditionOp::NotEqual => !equal,
        }
    })
}

fn apply_action(action: Action, state: &mut ProgramState) -> Result<(), RuntimeError> {
    let values = &mut state.values;
    match action {
        Action::Set { dst, value } => values[dst as usize] = value,
        Action::Copy { dst, src } => values[dst as usize] = values[src as usize],
        Action::Add { dst, left, right } => {
            values[dst as usize] = values[left as usize]
                .checked_add(values[right as usize])
                .ok_or(RuntimeError::Overflow)?;
        }
        Action::Subtract { dst, left, right } => {
            values[dst as usize] = values[left as usize]
                .checked_sub(values[right as usize])
                .ok_or(RuntimeError::Overflow)?;
        }
        Action::Multiply { dst, left, right } => {
            values[dst as usize] = values[left as usize]
                .checked_mul(values[right as usize])
                .ok_or(RuntimeError::Overflow)?;
        }
        Action::Divide { dst, left, right } => {
            let divisor = values[right as usize];
            if divisor == 0 {
                return Err(RuntimeError::DivisionByZero);
            }
            values[dst as usize] = values[left as usize]
                .checked_div(divisor)
                .ok_or(RuntimeError::Overflow)?;
        }
        Action::MulAdd {
            dst,
            src,
            multiplier,
            addend,
        } => {
            values[dst as usize] = values[src as usize]
                .checked_mul(multiplier)
                .and_then(|value| value.checked_add(addend))
                .ok_or(RuntimeError::Overflow)?;
        }
        Action::TextClear { .. } => state.text_len = 0,
    }
    Ok(())
}

fn encode_widget(widget: &Widget, out: &mut Vec<u8>) {
    let start = out.len();
    out.resize(start + WIDGET_PREFIX_LEN, 0);
    let (kind, rect, target, text) = match widget {
        Widget::Text { rect, text } => (1, *rect, 0, text.as_bytes()),
        Widget::Display { rect, state_slot } => (2, *rect, *state_slot as u16, &[][..]),
        Widget::Button {
            rect,
            event_id,
            text,
        } => (3, *rect, *event_id, text.as_bytes()),
        Widget::EditBox { rect, text_slot } => (4, *rect, *text_slot as u16, &[][..]),
    };
    out[start] = kind;
    put_u16(out, start + 2, text.len() as u16);
    put_u16(out, start + 4, rect.x);
    put_u16(out, start + 6, rect.y);
    put_u16(out, start + 8, rect.width);
    put_u16(out, start + 10, rect.height);
    put_u16(out, start + 12, target);
    out.extend_from_slice(text);
    while out.len() % 4 != 0 {
        out.push(0);
    }
}

fn parse_widget(cursor: &mut Cursor<'_>) -> Result<Widget, ProgramError> {
    let prefix = cursor.take(WIDGET_PREFIX_LEN)?;
    if prefix[1] != 0 || read_u16(prefix, 14)? != 0 {
        return Err(ProgramError::Malformed);
    }
    let text_len = read_u16(prefix, 2)? as usize;
    if prefix[0] == 4 && text_len != 0 {
        return Err(ProgramError::Malformed);
    }
    if text_len > MAX_WIDGET_TEXT_BYTES {
        return Err(ProgramError::LimitExceeded);
    }
    let rect = Rect {
        x: read_u16(prefix, 4)?,
        y: read_u16(prefix, 6)?,
        width: read_u16(prefix, 8)?,
        height: read_u16(prefix, 10)?,
    };
    let target = read_u16(prefix, 12)?;
    let text = cursor.take(text_len)?;
    let padding = (4 - text_len % 4) % 4;
    if cursor.take(padding)?.iter().any(|byte| *byte != 0) {
        return Err(ProgramError::Malformed);
    }
    match prefix[0] {
        1 if target == 0 => Ok(Widget::Text {
            rect,
            text: parse_text(text)?,
        }),
        2 if target <= u8::MAX as u16 && text_len == 0 => Ok(Widget::Display {
            rect,
            state_slot: target as u8,
        }),
        3 => Ok(Widget::Button {
            rect,
            event_id: target,
            text: parse_text(text)?,
        }),
        4 if target <= u8::MAX as u16 => Ok(Widget::EditBox {
            rect,
            text_slot: target as u8,
        }),
        4 => Err(ProgramError::InvalidReference),
        1..=3 => Err(ProgramError::Malformed),
        _ => Err(ProgramError::UnknownOpcode),
    }
}

fn parse_text(bytes: &[u8]) -> Result<String, ProgramError> {
    str::from_utf8(bytes)
        .map(String::from)
        .map_err(|_| ProgramError::Malformed)
}

fn encode_rule(rule: &Rule, out: &mut Vec<u8>) {
    let start = out.len();
    out.resize(start + RULE_PREFIX_LEN, 0);
    put_u16(out, start, rule.event_id);
    out[start + 2] = rule.conditions.len() as u8;
    out[start + 3] = rule.actions.len() as u8;
    for (index, condition) in rule.conditions.iter().enumerate() {
        let offset = start + 8 + index * 12;
        out[offset] = condition.state_slot;
        out[offset + 1] = match condition.op {
            ConditionOp::Equal => 1,
            ConditionOp::NotEqual => 2,
        };
        out[offset + 4..offset + 12].copy_from_slice(&condition.value.to_le_bytes());
    }
    for action in &rule.actions {
        encode_action(*action, out);
    }
}

fn parse_rule(cursor: &mut Cursor<'_>) -> Result<Rule, ProgramError> {
    let prefix = cursor.take(RULE_PREFIX_LEN)?;
    if prefix[4..8].iter().any(|byte| *byte != 0) {
        return Err(ProgramError::Malformed);
    }
    let condition_count = prefix[2] as usize;
    let action_count = prefix[3] as usize;
    if condition_count > MAX_CONDITIONS_PER_RULE || action_count > MAX_ACTIONS_PER_RULE {
        return Err(ProgramError::LimitExceeded);
    }
    let mut conditions = Vec::with_capacity(condition_count);
    for index in 0..MAX_CONDITIONS_PER_RULE {
        let offset = 8 + index * 12;
        if index < condition_count {
            if read_u16(prefix, offset + 2)? != 0 {
                return Err(ProgramError::Malformed);
            }
            let op = match prefix[offset + 1] {
                1 => ConditionOp::Equal,
                2 => ConditionOp::NotEqual,
                _ => return Err(ProgramError::UnknownOpcode),
            };
            conditions.push(Condition {
                state_slot: prefix[offset],
                op,
                value: read_i64(prefix, offset + 4)?,
            });
        } else if prefix[offset..offset + 12].iter().any(|byte| *byte != 0) {
            return Err(ProgramError::Malformed);
        }
    }
    let mut actions = Vec::with_capacity(action_count);
    for _ in 0..action_count {
        actions.push(parse_action(cursor.take(ACTION_LEN)?)?);
    }
    Ok(Rule {
        event_id: read_u16(prefix, 0)?,
        conditions,
        actions,
    })
}

fn encode_action(action: Action, out: &mut Vec<u8>) {
    let start = out.len();
    out.resize(start + ACTION_LEN, 0);
    let (opcode, dst, left, right, first, second) = match action {
        Action::Set { dst, value } => (1, dst, 0, 0, value, 0),
        Action::Copy { dst, src } => (2, dst, src, 0, 0, 0),
        Action::Add { dst, left, right } => (3, dst, left, right, 0, 0),
        Action::Subtract { dst, left, right } => (4, dst, left, right, 0, 0),
        Action::Multiply { dst, left, right } => (5, dst, left, right, 0, 0),
        Action::Divide { dst, left, right } => (6, dst, left, right, 0, 0),
        Action::MulAdd {
            dst,
            src,
            multiplier,
            addend,
        } => (7, dst, src, 0, multiplier, addend),
        Action::TextClear { slot } => (8, slot, 0, 0, 0, 0),
    };
    out[start] = opcode;
    out[start + 1] = dst;
    out[start + 2] = left;
    out[start + 3] = right;
    out[start + 8..start + 16].copy_from_slice(&first.to_le_bytes());
    out[start + 16..start + 24].copy_from_slice(&second.to_le_bytes());
}

fn parse_action(bytes: &[u8]) -> Result<Action, ProgramError> {
    if bytes[4..8].iter().any(|byte| *byte != 0) {
        return Err(ProgramError::Malformed);
    }
    let dst = bytes[1];
    let left = bytes[2];
    let right = bytes[3];
    let first = read_i64(bytes, 8)?;
    let second = read_i64(bytes, 16)?;
    match bytes[0] {
        1 => Ok(Action::Set { dst, value: first }),
        2 => Ok(Action::Copy { dst, src: left }),
        3 => Ok(Action::Add { dst, left, right }),
        4 => Ok(Action::Subtract { dst, left, right }),
        5 => Ok(Action::Multiply { dst, left, right }),
        6 => Ok(Action::Divide { dst, left, right }),
        7 => Ok(Action::MulAdd {
            dst,
            src: left,
            multiplier: first,
            addend: second,
        }),
        8 if bytes[2..].iter().all(|byte| *byte == 0) => Ok(Action::TextClear { slot: dst }),
        8 => Err(ProgramError::Malformed),
        _ => Err(ProgramError::UnknownOpcode),
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8], position: usize) -> Self {
        Self { bytes, position }
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], ProgramError> {
        let end = self
            .position
            .checked_add(len)
            .ok_or(ProgramError::Malformed)?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or(ProgramError::Malformed)?;
        self.position = end;
        Ok(value)
    }

    fn i64(&mut self) -> Result<i64, ProgramError> {
        read_i64(self.take(8)?, 0)
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, ProgramError> {
    let raw = bytes
        .get(offset..offset + 2)
        .ok_or(ProgramError::Malformed)?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, ProgramError> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or(ProgramError::Malformed)?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn read_i64(bytes: &[u8], offset: usize) -> Result<i64, ProgramError> {
    let raw = bytes
        .get(offset..offset + 8)
        .ok_or(ProgramError::Malformed)?;
    Ok(i64::from_le_bytes(
        raw.try_into().map_err(|_| ProgramError::Malformed)?,
    ))
}

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

const DISPLAY: u8 = 0;
const ACCUMULATOR: u8 = 1;
const PENDING: u8 = 2;
const ENTERING: u8 = 3;
const EVENT_ADD: u16 = 11;
const EVENT_SUBTRACT: u16 = 12;
const EVENT_MULTIPLY: u16 = 13;
const EVENT_DIVIDE: u16 = 14;
const EVENT_EQUALS: u16 = 15;
const EVENT_CLEAR: u16 = 16;

/// A complete signed-integer offline calculator expressed only as program data.
pub fn calculator_program() -> Program {
    let mut widgets = vec![
        Widget::Text {
            rect: rect(0, 0, 320, 32),
            text: String::from("Calculator"),
        },
        Widget::Display {
            rect: rect(0, 40, 320, 56),
            state_slot: DISPLAY,
        },
    ];
    let labels = [
        (7, 8),
        (8, 9),
        (9, 10),
        (EVENT_DIVIDE, 14),
        (4, 5),
        (5, 6),
        (6, 7),
        (EVENT_MULTIPLY, 13),
        (1, 2),
        (2, 3),
        (3, 4),
        (EVENT_SUBTRACT, 12),
        (EVENT_CLEAR, 16),
        (0, 1),
        (EVENT_EQUALS, 15),
        (EVENT_ADD, 11),
    ];
    for (index, (value, event)) in labels.iter().enumerate() {
        let col = (index % 4) as u16;
        let row = (index / 4) as u16;
        let text = match *event {
            EVENT_ADD => "+",
            EVENT_SUBTRACT => "-",
            EVENT_MULTIPLY => "*",
            EVENT_DIVIDE => "/",
            EVENT_EQUALS => "=",
            EVENT_CLEAR => "C",
            _ => match *value {
                0 => "0",
                1 => "1",
                2 => "2",
                3 => "3",
                4 => "4",
                5 => "5",
                6 => "6",
                7 => "7",
                8 => "8",
                _ => "9",
            },
        };
        widgets.push(Widget::Button {
            rect: rect(col * 80, 104 + row * 64, 72, 56),
            event_id: *event,
            text: String::from(text),
        });
    }

    let mut bindings = Vec::new();
    for digit in 0..=9u16 {
        bindings.push(KeyBinding {
            code: b'0' as u16 + digit,
            modifiers: 0,
            event_id: digit + 1,
        });
    }
    for (code, event_id) in [
        (b'+' as u16, EVENT_ADD),
        (b'-' as u16, EVENT_SUBTRACT),
        (b'*' as u16, EVENT_MULTIPLY),
        (b'/' as u16, EVENT_DIVIDE),
        (b'=' as u16, EVENT_EQUALS),
        (13, EVENT_EQUALS),
        (b'c' as u16, EVENT_CLEAR),
        (b'C' as u16, EVENT_CLEAR),
    ] {
        bindings.push(KeyBinding {
            code,
            modifiers: 0,
            event_id,
        });
    }

    let mut rules = Vec::new();
    for digit in 0..=9i64 {
        let event_id = digit as u16 + 1;
        rules.push(rule(
            event_id,
            vec![eq(ENTERING, 0)],
            vec![
                Action::Set {
                    dst: DISPLAY,
                    value: digit,
                },
                Action::Set {
                    dst: ENTERING,
                    value: 1,
                },
            ],
        ));
        rules.push(rule(
            event_id,
            vec![eq(ENTERING, 1)],
            vec![Action::MulAdd {
                dst: DISPLAY,
                src: DISPLAY,
                multiplier: 10,
                addend: digit,
            }],
        ));
    }
    for (event_id, pending) in [
        (EVENT_ADD, 1),
        (EVENT_SUBTRACT, 2),
        (EVENT_MULTIPLY, 3),
        (EVENT_DIVIDE, 4),
    ] {
        append_operator_rules(&mut rules, event_id, pending);
    }
    append_equals_rules(&mut rules);
    rules.push(rule(
        EVENT_CLEAR,
        vec![],
        vec![
            Action::Set {
                dst: DISPLAY,
                value: 0,
            },
            Action::Set {
                dst: ACCUMULATOR,
                value: 0,
            },
            Action::Set {
                dst: PENDING,
                value: 0,
            },
            Action::Set {
                dst: ENTERING,
                value: 0,
            },
        ],
    ));

    Program::new(vec![0, 0, 0, 0], widgets, bindings, rules)
        .expect("built-in calculator must satisfy the frozen limits")
}

/// A bounded current-boot text editor expressed only as program data.
pub fn editor_program() -> Program {
    Program::new_with_text_slots(
        vec![0],
        vec![MAX_TEXT_SLOT_CAPACITY as u16],
        vec![
            Widget::Text {
                rect: rect(8, 8, 240, 24),
                text: String::from("raiOS EDIT  F12=EXIT"),
            },
            Widget::EditBox {
                rect: rect(8, 40, 608, 384),
                text_slot: 0,
            },
            Widget::Button {
                rect: rect(8, 436, 96, 36),
                event_id: 1,
                text: String::from("CLEAR"),
            },
        ],
        vec![],
        vec![rule(1, vec![], vec![Action::TextClear { slot: 0 }])],
    )
    .expect("built-in editor must satisfy the frozen limits")
}

const fn rect(x: u16, y: u16, width: u16, height: u16) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}

const fn eq(state_slot: u8, value: i64) -> Condition {
    Condition {
        state_slot,
        op: ConditionOp::Equal,
        value,
    }
}

fn rule(event_id: u16, conditions: Vec<Condition>, actions: Vec<Action>) -> Rule {
    Rule {
        event_id,
        conditions,
        actions,
    }
}

fn append_operator_rules(rules: &mut Vec<Rule>, event_id: u16, next_pending: i64) {
    rules.push(rule(
        event_id,
        vec![eq(ENTERING, 0)],
        vec![Action::Set {
            dst: PENDING,
            value: next_pending,
        }],
    ));
    rules.push(rule(
        event_id,
        vec![eq(ENTERING, 1), eq(PENDING, 0)],
        vec![
            Action::Copy {
                dst: ACCUMULATOR,
                src: DISPLAY,
            },
            Action::Set {
                dst: PENDING,
                value: next_pending,
            },
            Action::Set {
                dst: ENTERING,
                value: 0,
            },
        ],
    ));
    for pending in 1..=4 {
        let mut actions = vec![arithmetic_action(pending)];
        actions.extend_from_slice(&[
            Action::Copy {
                dst: DISPLAY,
                src: ACCUMULATOR,
            },
            Action::Set {
                dst: PENDING,
                value: next_pending,
            },
            Action::Set {
                dst: ENTERING,
                value: 0,
            },
        ]);
        rules.push(rule(
            event_id,
            vec![eq(ENTERING, 1), eq(PENDING, pending)],
            actions,
        ));
    }
}

fn append_equals_rules(rules: &mut Vec<Rule>) {
    rules.push(rule(
        EVENT_EQUALS,
        vec![eq(ENTERING, 0)],
        vec![Action::Set {
            dst: PENDING,
            value: 0,
        }],
    ));
    rules.push(rule(
        EVENT_EQUALS,
        vec![eq(ENTERING, 1), eq(PENDING, 0)],
        vec![Action::Set {
            dst: ENTERING,
            value: 0,
        }],
    ));
    for pending in 1..=4 {
        rules.push(rule(
            EVENT_EQUALS,
            vec![eq(ENTERING, 1), eq(PENDING, pending)],
            vec![
                arithmetic_action(pending),
                Action::Copy {
                    dst: DISPLAY,
                    src: ACCUMULATOR,
                },
                Action::Set {
                    dst: PENDING,
                    value: 0,
                },
                Action::Set {
                    dst: ENTERING,
                    value: 0,
                },
            ],
        ));
    }
}

fn arithmetic_action(pending: i64) -> Action {
    match pending {
        1 => Action::Add {
            dst: ACCUMULATOR,
            left: ACCUMULATOR,
            right: DISPLAY,
        },
        2 => Action::Subtract {
            dst: ACCUMULATOR,
            left: ACCUMULATOR,
            right: DISPLAY,
        },
        3 => Action::Multiply {
            dst: ACCUMULATOR,
            left: ACCUMULATOR,
            right: DISPLAY,
        },
        _ => Action::Divide {
            dst: ACCUMULATOR,
            left: ACCUMULATOR,
            right: DISPLAY,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(
        program: &Program,
        state: &mut ProgramState,
        value: u8,
    ) -> Result<DispatchOutcome, RuntimeError> {
        program.dispatch(
            state,
            UiInput::Key {
                code: value as u16,
                modifiers: 0,
            },
        )
    }

    fn sequence(program: &Program, bytes: &[u8]) -> Result<ProgramState, RuntimeError> {
        let mut state = program.initial_state();
        for byte in bytes {
            key(program, &mut state, *byte)?;
        }
        Ok(state)
    }

    fn hex(bytes: &[u8]) -> alloc::string::String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = alloc::string::String::new();
        for byte in bytes {
            output.push(HEX[(byte >> 4) as usize] as char);
            output.push(HEX[(byte & 0x0f) as usize] as char);
        }
        output
    }

    fn base64(bytes: &[u8]) -> alloc::string::String {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut output = alloc::string::String::new();
        let mut index = 0usize;
        while index + 3 <= bytes.len() {
            let value = ((bytes[index] as u32) << 16)
                | ((bytes[index + 1] as u32) << 8)
                | bytes[index + 2] as u32;
            output.push(ALPHABET[((value >> 18) & 0x3f) as usize] as char);
            output.push(ALPHABET[((value >> 12) & 0x3f) as usize] as char);
            output.push(ALPHABET[((value >> 6) & 0x3f) as usize] as char);
            output.push(ALPHABET[(value & 0x3f) as usize] as char);
            index += 3;
        }
        match bytes.len() - index {
            1 => {
                let value = (bytes[index] as u32) << 16;
                output.push(ALPHABET[((value >> 18) & 0x3f) as usize] as char);
                output.push(ALPHABET[((value >> 12) & 0x3f) as usize] as char);
                output.push('=');
                output.push('=');
            }
            2 => {
                let value = ((bytes[index] as u32) << 16) | ((bytes[index + 1] as u32) << 8);
                output.push(ALPHABET[((value >> 18) & 0x3f) as usize] as char);
                output.push(ALPHABET[((value >> 12) & 0x3f) as usize] as char);
                output.push(ALPHABET[((value >> 6) & 0x3f) as usize] as char);
                output.push('=');
            }
            _ => {}
        }
        output
    }

    #[test]
    fn canonical_round_trip_and_identity_are_deterministic() {
        let program = calculator_program();
        let bytes = program.canonical_bytes();
        let parsed = Program::parse(&bytes).expect("canonical program parses");
        assert_eq!(parsed, program);
        assert_eq!(parsed.canonical_bytes(), bytes);
        assert_eq!(parsed.identity(), program.identity());
        assert_eq!(program.identity().sha256, sha256_bytes(&bytes));
    }

    #[test]
    fn calculator_canonical_bytes_are_pinned() {
        let bytes = calculator_program().canonical_bytes();
        assert_eq!(bytes.len(), 5372);
        assert_eq!(
            sha256_bytes(&bytes),
            [
                0x7c, 0xa0, 0xaa, 0x21, 0xd6, 0x9b, 0xaa, 0xe0, 0x72, 0x67, 0x5c, 0x20, 0xf7, 0xb4,
                0x4d, 0x0e, 0x2d, 0x9f, 0x99, 0xac, 0x4e, 0x72, 0xd6, 0xaa, 0x64, 0xe7, 0xa2, 0x55,
                0x86, 0xdf, 0xcd, 0x6e,
            ]
        );
    }

    #[test]
    fn editor_program_canonical_round_trip_and_dispatch() {
        let program = editor_program();
        let bytes = program.canonical_bytes();
        assert_eq!(bytes.len(), 176);
        let digest = hex(&sha256_bytes(&bytes));
        println!("editor canonical sha256={digest}");
        println!("editor canonical base64={}", base64(&bytes));
        assert_eq!(
            digest,
            "34f726d13818d174e23ef0614ca183a2967b9449c8cf4447151aef13d277d815"
        );
        assert_eq!(Program::parse(&bytes), Ok(program.clone()));
        assert_eq!(Program::parse(&bytes).unwrap().canonical_bytes(), bytes);
        assert_eq!(program.text_slot_capacity(), Some(2048));
        assert!(program.has_edit_box());

        let mut state = program.initial_state();
        assert_eq!(key(&program, &mut state, b'a'), Ok(DispatchOutcome::Edited));
        assert_eq!(key(&program, &mut state, b'b'), Ok(DispatchOutcome::Edited));
        assert_eq!(key(&program, &mut state, 13), Ok(DispatchOutcome::Edited));
        assert_eq!(state.text(), b"ab\n");
        assert_eq!(key(&program, &mut state, 8), Ok(DispatchOutcome::Edited));
        assert_eq!(state.text(), b"ab");
        assert_eq!(
            program.dispatch(&mut state, UiInput::PointerClick { x: 9, y: 437 }),
            Ok(DispatchOutcome::Handled { event_id: 1 })
        );
        assert!(state.text().is_empty());

        let full = Program::new_with_text_slots(
            vec![0],
            vec![1],
            vec![Widget::EditBox {
                rect: rect(0, 0, 1, 1),
                text_slot: 0,
            }],
            vec![],
            vec![],
        )
        .unwrap();
        let mut full_state = full.initial_state();
        assert_eq!(
            key(&full, &mut full_state, b'x'),
            Ok(DispatchOutcome::Edited)
        );
        let before = full_state;
        assert_eq!(
            key(&full, &mut full_state, b'y'),
            Ok(DispatchOutcome::Ignored)
        );
        assert_eq!(full_state, before);

        let bound = Program::new_with_text_slots(
            vec![0],
            vec![8],
            vec![Widget::EditBox {
                rect: rect(0, 0, 1, 1),
                text_slot: 0,
            }],
            vec![KeyBinding {
                code: b'a' as u16,
                modifiers: 0,
                event_id: 1,
            }],
            vec![rule(1, vec![], vec![Action::Set { dst: 0, value: 7 }])],
        )
        .unwrap();
        let mut bound_state = bound.initial_state();
        assert_eq!(
            key(&bound, &mut bound_state, b'a'),
            Ok(DispatchOutcome::Handled { event_id: 1 })
        );
        assert_eq!(bound_state.get(0), Some(7));
        assert!(bound_state.text().is_empty());

        let calculator = calculator_program();
        let mut calculator_state = calculator.initial_state();
        assert_eq!(
            key(&calculator, &mut calculator_state, b'a'),
            Ok(DispatchOutcome::Ignored)
        );
    }

    #[test]
    fn text_slots_and_edit_encoding_fail_closed() {
        assert_eq!(
            Program::new_with_text_slots(
                vec![0],
                vec![1; MAX_TEXT_SLOTS + 1],
                vec![],
                vec![],
                vec![]
            ),
            Err(ProgramError::LimitExceeded)
        );
        assert_eq!(
            Program::new_with_text_slots(vec![0], vec![0], vec![], vec![], vec![]),
            Err(ProgramError::LimitExceeded)
        );
        assert_eq!(
            Program::new_with_text_slots(
                vec![0],
                vec![(MAX_TEXT_SLOT_CAPACITY + 1) as u16],
                vec![],
                vec![],
                vec![]
            ),
            Err(ProgramError::LimitExceeded)
        );
        assert_eq!(
            Program::new(
                vec![0],
                vec![Widget::EditBox {
                    rect: rect(0, 0, 1, 1),
                    text_slot: 0,
                }],
                vec![],
                vec![],
            ),
            Err(ProgramError::InvalidReference)
        );
        assert_eq!(
            Program::new_with_text_slots(
                vec![0],
                vec![1],
                vec![
                    Widget::EditBox {
                        rect: rect(0, 0, 1, 1),
                        text_slot: 0,
                    },
                    Widget::EditBox {
                        rect: rect(2, 0, 1, 1),
                        text_slot: 0,
                    },
                ],
                vec![],
                vec![],
            ),
            Err(ProgramError::LimitExceeded)
        );
        assert_eq!(
            Program::new_with_text_slots(
                vec![0],
                vec![1],
                vec![],
                vec![],
                vec![rule(1, vec![], vec![Action::TextClear { slot: 1 }])],
            ),
            Err(ProgramError::InvalidReference)
        );

        let mut bytes = editor_program().canonical_bytes();
        let text_slot = HEADER_LEN + 8;
        bytes[text_slot] = 0;
        bytes[text_slot + 1] = 0;
        assert_eq!(Program::parse(&bytes), Err(ProgramError::LimitExceeded));

        let mut bytes = editor_program().canonical_bytes();
        bytes[text_slot + 2] = 1;
        assert_eq!(Program::parse(&bytes), Err(ProgramError::Malformed));

        let edit_box = HEADER_LEN + 8 + 4 + WIDGET_PREFIX_LEN + "raiOS EDIT  F12=EXIT".len();
        let mut bytes = editor_program().canonical_bytes();
        bytes[edit_box + 2] = 1;
        assert_eq!(Program::parse(&bytes), Err(ProgramError::Malformed));
        let mut bytes = editor_program().canonical_bytes();
        bytes[edit_box + 14] = 1;
        assert_eq!(Program::parse(&bytes), Err(ProgramError::Malformed));

        let action = edit_box + WIDGET_PREFIX_LEN + WIDGET_PREFIX_LEN + 8 + RULE_PREFIX_LEN;
        let mut bytes = editor_program().canonical_bytes();
        bytes[action + 2] = 1;
        assert_eq!(Program::parse(&bytes), Err(ProgramError::Malformed));

        let mismatched = ProgramState::from_values_and_text(&[0], b"xx").unwrap();
        let one_byte = Program::new_with_text_slots(
            vec![0],
            vec![1],
            vec![Widget::EditBox {
                rect: rect(0, 0, 1, 1),
                text_slot: 0,
            }],
            vec![],
            vec![],
        )
        .unwrap();
        let mut mismatched = mismatched;
        assert_eq!(
            one_byte.dispatch(
                &mut mismatched,
                UiInput::Key {
                    code: b'a' as u16,
                    modifiers: 0,
                },
            ),
            Err(RuntimeError::StateMismatch)
        );
    }

    #[test]
    fn calculator_handles_all_operations_clear_keyboard_and_pointer() {
        let program = calculator_program();
        assert_eq!(sequence(&program, b"12+30=").unwrap().get(0), Some(42));
        assert_eq!(sequence(&program, b"50-8=").unwrap().get(0), Some(42));
        assert_eq!(sequence(&program, b"6*7=").unwrap().get(0), Some(42));
        assert_eq!(sequence(&program, b"84/2=").unwrap().get(0), Some(42));
        assert_eq!(sequence(&program, b"99C").unwrap().get(0), Some(0));

        let mut state = program.initial_state();
        assert_eq!(
            program.dispatch(&mut state, UiInput::PointerClick { x: 81, y: 105 }),
            Ok(DispatchOutcome::Handled { event_id: 9 })
        );
        assert_eq!(state.get(0), Some(8));
    }

    #[test]
    fn runtime_errors_are_atomic() {
        let program = calculator_program();
        assert_eq!(
            ProgramState::from_values(&[]),
            Err(RuntimeError::StateMismatch)
        );
        assert_eq!(
            ProgramState::from_values(&[0; MAX_STATE_SLOTS + 1]),
            Err(RuntimeError::StateMismatch)
        );
        let mut state = program.initial_state();
        for byte in b"9/0" {
            key(&program, &mut state, *byte).unwrap();
        }
        let before = state;
        assert_eq!(
            key(&program, &mut state, b'='),
            Err(RuntimeError::DivisionByZero)
        );
        assert_eq!(state, before);
    }

    #[test]
    fn malformed_oversize_and_unknown_opcode_fail_closed() {
        let bytes = calculator_program().canonical_bytes();
        assert_eq!(
            Program::parse(&bytes[..bytes.len() - 1]),
            Err(ProgramError::Malformed)
        );
        assert_eq!(
            Program::parse(&vec![0; MAX_PROGRAM_BYTES + 1]),
            Err(ProgramError::LimitExceeded)
        );

        let mut unknown = bytes.clone();
        let state_bytes = 4 * 8;
        let first_widget = HEADER_LEN + state_bytes;
        unknown[first_widget] = 0xff;
        assert_eq!(Program::parse(&unknown), Err(ProgramError::UnknownOpcode));
    }

    #[test]
    fn exact_limits_and_references_are_enforced() {
        let program = Program::new(vec![0; MAX_STATE_SLOTS], vec![], vec![], vec![]).unwrap();
        assert_eq!(Program::parse(&program.canonical_bytes()), Ok(program));
        assert_eq!(
            Program::new(vec![0; MAX_STATE_SLOTS + 1], vec![], vec![], vec![]),
            Err(ProgramError::LimitExceeded)
        );
        assert_eq!(
            Program::new(
                vec![0],
                vec![Widget::Display {
                    rect: rect(0, 0, 1, 1),
                    state_slot: 1
                }],
                vec![],
                vec![],
            ),
            Err(ProgramError::InvalidReference)
        );
    }
}
