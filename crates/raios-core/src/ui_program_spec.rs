//! Text authoring syntax for bounded [`crate::ui_program::Program`] values.
//!
//! The text form is only an authoring convenience. `Program::new` remains the
//! authority for references, limits, canonical encoding, and runtime behavior.

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::str;

use crate::ui_program::{
    Action, Condition, ConditionOp, KeyBinding, Program, ProgramError, Rect, Rule, Widget,
    MAX_PROGRAM_BYTES,
};

const HEADER: &str = "RAIOS_UI_SPEC_V1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpecError {
    InputTooLarge,
    InvalidUtf8,
    Malformed,
    UnknownCommand,
    InvalidNumber,
    Program(ProgramError),
}

impl From<ProgramError> for SpecError {
    fn from(error: ProgramError) -> Self {
        Self::Program(error)
    }
}

/// Parses the bounded textual authoring form into the canonical program model.
pub fn parse(input: &[u8]) -> Result<Program, SpecError> {
    if input.len() > MAX_PROGRAM_BYTES {
        return Err(SpecError::InputTooLarge);
    }
    let input = str::from_utf8(input).map_err(|_| SpecError::InvalidUtf8)?;
    let mut state = Vec::new();
    let mut text_slot_capacities = Vec::new();
    let mut widgets = Vec::new();
    let mut bindings = Vec::new();
    let mut rules = Vec::new();
    let mut active_rule: Option<Rule> = None;
    let mut header_seen = false;
    let mut end_seen = false;
    let mut textstate_seen = false;

    for line in input.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let tokens = tokenize(line)?;
        if tokens.is_empty() {
            continue;
        }
        if !header_seen {
            if tokens.len() != 1 || tokens[0].quoted || tokens[0].text != HEADER {
                return Err(SpecError::Malformed);
            }
            header_seen = true;
            continue;
        }
        if end_seen {
            return Err(SpecError::Malformed);
        }

        let command = bare(&tokens[0])?;
        if let Some(rule) = active_rule.as_mut() {
            match command {
                "when" => {
                    exact(&tokens, 4)?;
                    rule.conditions.push(Condition {
                        state_slot: number(&tokens[1])?,
                        op: match bare(&tokens[2])? {
                            "eq" => ConditionOp::Equal,
                            "ne" => ConditionOp::NotEqual,
                            _ => return Err(SpecError::Malformed),
                        },
                        value: number(&tokens[3])?,
                    });
                }
                "set" => {
                    exact(&tokens, 3)?;
                    rule.actions.push(Action::Set {
                        dst: number(&tokens[1])?,
                        value: number(&tokens[2])?,
                    });
                }
                "copy" => {
                    exact(&tokens, 3)?;
                    rule.actions.push(Action::Copy {
                        dst: number(&tokens[1])?,
                        src: number(&tokens[2])?,
                    });
                }
                "add" | "sub" | "mul" | "div" => {
                    exact(&tokens, 4)?;
                    let dst = number(&tokens[1])?;
                    let left = number(&tokens[2])?;
                    let right = number(&tokens[3])?;
                    rule.actions.push(match command {
                        "add" => Action::Add { dst, left, right },
                        "sub" => Action::Subtract { dst, left, right },
                        "mul" => Action::Multiply { dst, left, right },
                        _ => Action::Divide { dst, left, right },
                    });
                }
                "muladd" => {
                    exact(&tokens, 5)?;
                    rule.actions.push(Action::MulAdd {
                        dst: number(&tokens[1])?,
                        src: number(&tokens[2])?,
                        multiplier: number(&tokens[3])?,
                        addend: number(&tokens[4])?,
                    });
                }
                "textclear" => {
                    exact(&tokens, 2)?;
                    rule.actions.push(Action::TextClear {
                        slot: number(&tokens[1])?,
                    });
                }
                "endrule" => {
                    exact(&tokens, 1)?;
                    rules.push(active_rule.take().expect("active rule exists"));
                }
                _ => return Err(SpecError::UnknownCommand),
            }
            continue;
        }

        match command {
            "state" => {
                exact(&tokens, 2)?;
                state.push(number(&tokens[1])?);
            }
            "textstate" => {
                exact(&tokens, 2)?;
                if textstate_seen {
                    return Err(SpecError::Malformed);
                }
                textstate_seen = true;
                text_slot_capacities.push(number(&tokens[1])?);
            }
            "text" => {
                exact(&tokens, 6)?;
                widgets.push(Widget::Text {
                    rect: rect(&tokens[1..5])?,
                    text: label(&tokens[5])?,
                });
            }
            "display" => {
                exact(&tokens, 6)?;
                widgets.push(Widget::Display {
                    rect: rect(&tokens[1..5])?,
                    state_slot: number(&tokens[5])?,
                });
            }
            "button" => {
                exact(&tokens, 7)?;
                widgets.push(Widget::Button {
                    rect: rect(&tokens[1..5])?,
                    event_id: number(&tokens[5])?,
                    text: label(&tokens[6])?,
                });
            }
            "editbox" => {
                exact(&tokens, 6)?;
                widgets.push(Widget::EditBox {
                    rect: rect(&tokens[1..5])?,
                    text_slot: number(&tokens[5])?,
                });
            }
            "key" => {
                exact(&tokens, 4)?;
                bindings.push(KeyBinding {
                    code: number(&tokens[1])?,
                    modifiers: number(&tokens[2])?,
                    event_id: number(&tokens[3])?,
                });
            }
            "rule" => {
                exact(&tokens, 2)?;
                active_rule = Some(Rule {
                    event_id: number(&tokens[1])?,
                    conditions: Vec::new(),
                    actions: Vec::new(),
                });
            }
            "end" => {
                exact(&tokens, 1)?;
                end_seen = true;
            }
            _ => return Err(SpecError::UnknownCommand),
        }
    }

    if !header_seen || !end_seen || active_rule.is_some() {
        return Err(SpecError::Malformed);
    }
    Program::new_with_text_slots(state, text_slot_capacities, widgets, bindings, rules)
        .map_err(Into::into)
}

#[derive(Debug)]
struct Token {
    text: String,
    quoted: bool,
}

fn tokenize(line: &str) -> Result<Vec<Token>, SpecError> {
    let mut tokens = Vec::new();
    let mut chars = line.char_indices().peekable();
    while let Some((_, ch)) = chars.peek().copied() {
        if ch.is_ascii_whitespace() {
            chars.next();
            continue;
        }
        if ch == '"' {
            chars.next();
            let mut text = String::new();
            let mut closed = false;
            while let Some((_, ch)) = chars.next() {
                match ch {
                    '"' => {
                        closed = true;
                        break;
                    }
                    '\\' => match chars.next().map(|(_, escaped)| escaped) {
                        Some('"') => text.push('"'),
                        Some('\\') => text.push('\\'),
                        _ => return Err(SpecError::Malformed),
                    },
                    _ => text.push(ch),
                }
            }
            if !closed
                || chars
                    .peek()
                    .is_some_and(|(_, ch)| !ch.is_ascii_whitespace())
            {
                return Err(SpecError::Malformed);
            }
            tokens.push(Token { text, quoted: true });
            continue;
        }

        let mut text = String::new();
        while let Some((_, ch)) = chars.peek().copied() {
            if ch.is_ascii_whitespace() {
                break;
            }
            if !ch.is_ascii() || ch == '"' || ch == '\\' {
                return Err(SpecError::Malformed);
            }
            text.push(ch);
            chars.next();
        }
        tokens.push(Token {
            text,
            quoted: false,
        });
    }
    Ok(tokens)
}

fn exact(tokens: &[Token], expected: usize) -> Result<(), SpecError> {
    if tokens.len() == expected {
        Ok(())
    } else {
        Err(SpecError::Malformed)
    }
}

fn bare(token: &Token) -> Result<&str, SpecError> {
    if token.quoted {
        Err(SpecError::Malformed)
    } else {
        Ok(&token.text)
    }
}

fn label(token: &Token) -> Result<String, SpecError> {
    if token.quoted {
        Ok(token.text.clone())
    } else {
        Err(SpecError::Malformed)
    }
}

fn number<T: str::FromStr>(token: &Token) -> Result<T, SpecError> {
    bare(token)?
        .parse::<T>()
        .map_err(|_| SpecError::InvalidNumber)
}

fn rect(tokens: &[Token]) -> Result<Rect, SpecError> {
    Ok(Rect {
        x: number(&tokens[0])?,
        y: number(&tokens[1])?,
        width: number(&tokens[2])?,
        height: number(&tokens[3])?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_program::{editor_program, DispatchOutcome, UiInput, MAX_STATE_SLOTS};

    const COUNTER: &str = r#"RAIOS_UI_SPEC_V1
state 0
state 1
text 8 8 120 24 "Counter"
display 8 40 120 40 0
button 8 88 120 40 1 "+1"
key 43 0 1
rule 1
when 0 ne -1
add 0 0 1
endrule
end
"#;

    const EDITOR: &str = r#"RAIOS_UI_SPEC_V1
state 0
textstate 2048
text 8 8 240 24 "raiOS EDIT  F12=EXIT"
editbox 8 40 608 384 0
button 8 436 96 36 1 "CLEAR"
rule 1
textclear 0
endrule
end
"#;

    #[test]
    fn counter_is_canonical_and_increments() {
        let program = parse(COUNTER.as_bytes()).unwrap();
        let canonical = program.canonical_bytes();
        assert_eq!(
            Program::parse(&canonical).unwrap().canonical_bytes(),
            canonical
        );
        assert_eq!(
            parse(COUNTER.as_bytes()).unwrap().canonical_bytes(),
            canonical
        );

        let mut state = program.initial_state();
        assert_eq!(
            program.dispatch(
                &mut state,
                UiInput::Key {
                    code: 43,
                    modifiers: 0
                }
            ),
            Ok(DispatchOutcome::Handled { event_id: 1 })
        );
        assert_eq!(state.get(0), Some(1));
    }

    #[test]
    fn editor_spec_is_byte_identical_to_checked_in_program() {
        assert_eq!(
            parse(EDITOR.as_bytes()).unwrap().canonical_bytes(),
            editor_program().canonical_bytes()
        );
    }

    #[test]
    fn quoted_utf8_labels_support_only_two_escapes() {
        let source =
            "RAIOS_UI_SPEC_V1\nstate 0\ntext 0 0 10 10 \"Grüße \\\"raiOS\\\" \\\\ core\"\nend\n";
        let program = parse(source.as_bytes()).unwrap();
        assert!(matches!(
            &program.widgets()[0],
            Widget::Text { text, .. } if text == "Grüße \"raiOS\" \\ core"
        ));
        assert_eq!(
            parse(b"RAIOS_UI_SPEC_V1\nstate 0\ntext 0 0 1 1 \"bad\\n\"\nend"),
            Err(SpecError::Malformed)
        );
    }

    #[test]
    fn malformed_unknown_and_limits_fail_closed() {
        for source in [
            "state 0\nend",
            "RAIOS_UI_SPEC_V1\nstate 0",
            "RAIOS_UI_SPEC_V1\nstate 0\nwat\nend",
            "RAIOS_UI_SPEC_V1\nstate 0\nend\nstate 1",
            "RAIOS_UI_SPEC_V1\nstate 0\nrule 1\nend",
            "RAIOS_UI_SPEC_V1\nstate 0\ntext 0 0 1 1 unquoted\nend",
            "RAIOS_UI_SPEC_V1\nstate 0\ntextstate 1\ntextstate 1\nend",
        ] {
            assert!(parse(source.as_bytes()).is_err(), "accepted: {source}");
        }
        let oversized = alloc::vec![b' '; MAX_PROGRAM_BYTES + 1];
        assert_eq!(parse(&oversized), Err(SpecError::InputTooLarge));

        let mut too_many_states = String::from("RAIOS_UI_SPEC_V1\n");
        for _ in 0..=MAX_STATE_SLOTS {
            too_many_states.push_str("state 0\n");
        }
        too_many_states.push_str("end\n");
        assert_eq!(
            parse(too_many_states.as_bytes()),
            Err(SpecError::Program(ProgramError::LimitExceeded))
        );
    }
}
