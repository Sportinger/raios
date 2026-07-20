#![cfg_attr(not(test), no_std)]

//! Bounded, deterministic `RAIOS_LANG_V1` compiler.

pub use raios_wasm_ir::ERR_OVERSIZE_OUTPUT;
use raios_wasm_ir::{TypedFunctionEmitter, TypedInstruction, WasmModuleBytes};

pub const MAX_SOURCE_BYTES: usize = 4096;
pub const MAX_TOKENS: usize = 512;
pub const MAX_NODES: usize = 256;
pub const MAX_DEPTH: u8 = 16;
pub const MAX_LETS: usize = 32;

pub const ERR_OVERSIZE_SOURCE: &str = "oversize_source";
pub const ERR_NON_ASCII_SOURCE: &str = "non_ascii_source";
pub const ERR_MISSING_TERMINATOR: &str = "missing_terminator";
pub const ERR_WRONG_VERSION_LINE: &str = "wrong_version_line";
pub const ERR_INVALID_CHARACTER: &str = "invalid_character";
pub const ERR_NON_CANONICAL_INTEGER_TEXT: &str = "non_canonical_integer_text";
pub const ERR_TOKEN_QUOTA_EXCEEDED: &str = "token_quota_exceeded";
pub const ERR_SYNTAX: &str = "syntax_error";
pub const ERR_QUOTA_EXCEEDED: &str = "quota_exceeded";
pub const ERR_DUPLICATE_BINDING: &str = "duplicate_binding";
pub const ERR_UNKNOWN_BINDING: &str = "unknown_binding";
pub const ERR_TYPE_MISMATCH: &str = "type_mismatch";
pub const ERR_DIVISION_BY_ZERO: &str = "division_by_zero";
pub const ERR_DIVISION_OVERFLOW: &str = "division_overflow";
pub const ERR_BACKEND: &str = "backend_error";

const VERSION_LINE: &[u8] = b"RAIOS_LANG_V1";
const ENTRYPOINT: &[u8] = b"raios_service_main";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TokenKind {
    Empty,
    Fn,
    I32,
    Let,
    If,
    Else,
    Ident,
    Integer(i32),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Arrow,
    Assign,
    Semicolon,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
}

#[derive(Clone, Copy)]
struct Token {
    kind: TokenKind,
    start: u16,
    end: u16,
}

impl Token {
    const EMPTY: Self = Self {
        kind: TokenKind::Empty,
        start: 0,
        end: 0,
    };
}

struct Tokens {
    items: [Token; MAX_TOKENS],
    len: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BinaryOp {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Copy)]
enum NodeKind {
    Empty,
    Integer(i32),
    Name {
        start: u16,
        end: u16,
    },
    Local(u8),
    Binary {
        op: BinaryOp,
        left: u16,
        right: u16,
    },
    If {
        condition: u16,
        then_node: u16,
        else_node: u16,
    },
}

#[derive(Clone, Copy)]
struct Node {
    kind: NodeKind,
    depth: u8,
}

impl Node {
    const EMPTY: Self = Self {
        kind: NodeKind::Empty,
        depth: 0,
    };
}

#[derive(Clone, Copy)]
struct LetDecl {
    name_start: u16,
    name_end: u16,
    value: u16,
}

impl LetDecl {
    const EMPTY: Self = Self {
        name_start: 0,
        name_end: 0,
        value: 0,
    };
}

struct Program {
    nodes: [Node; MAX_NODES],
    node_count: usize,
    lets: [LetDecl; MAX_LETS],
    let_count: usize,
    result: u16,
}

/// Compiles one complete `RAIOS_LANG_V1` source into canonical WebAssembly.
pub fn compile(source: &[u8]) -> Result<WasmModuleBytes, &'static str> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err(ERR_OVERSIZE_SOURCE);
    }
    if source.iter().any(|byte| !byte.is_ascii()) {
        return Err(ERR_NON_ASCII_SOURCE);
    }
    if !source.ends_with(b"\n") {
        return Err(ERR_MISSING_TERMINATOR);
    }
    let first_lf = source
        .iter()
        .position(|byte| *byte == b'\n')
        .ok_or(ERR_MISSING_TERMINATOR)?;
    if &source[..first_lf] != VERSION_LINE {
        return Err(ERR_WRONG_VERSION_LINE);
    }
    let body_start = first_lf + 1;
    validate_characters(&source[body_start..])?;
    let tokens = lex(source, body_start)?;
    let mut program = Parser::new(source, body_start, &tokens).parse()?;
    bind_and_typecheck(source, &mut program)?;
    validate_arithmetic(&program)?;
    emit(&program)
}

fn validate_characters(source: &[u8]) -> Result<(), &'static str> {
    if source.iter().all(|&byte| {
        byte.is_ascii_lowercase()
            || byte.is_ascii_digit()
            || matches!(
                byte,
                b' ' | b'\n'
                    | b'_'
                    | b'('
                    | b')'
                    | b'{'
                    | b'}'
                    | b';'
                    | b'='
                    | b'!'
                    | b'-'
                    | b'<'
                    | b'>'
                    | b'+'
                    | b'*'
                    | b'/'
            )
    }) {
        Ok(())
    } else {
        Err(ERR_INVALID_CHARACTER)
    }
}

fn lex(source: &[u8], start: usize) -> Result<Tokens, &'static str> {
    let mut items = [Token::EMPTY; MAX_TOKENS];
    let mut count = 0usize;
    let mut cursor = start;
    let mut primary_expected = false;
    while cursor < source.len() {
        let byte = source[cursor];
        if matches!(byte, b' ' | b'\n') {
            cursor += 1;
            continue;
        }
        let token_start = cursor;
        let kind = if byte.is_ascii_lowercase() {
            cursor += 1;
            while cursor < source.len()
                && (source[cursor].is_ascii_lowercase()
                    || source[cursor].is_ascii_digit()
                    || source[cursor] == b'_')
            {
                cursor += 1;
            }
            if cursor - token_start > 32 {
                return Err(ERR_INVALID_CHARACTER);
            }
            match &source[token_start..cursor] {
                b"fn" => TokenKind::Fn,
                b"i32" => TokenKind::I32,
                b"let" => TokenKind::Let,
                b"if" => TokenKind::If,
                b"else" => TokenKind::Else,
                _ => TokenKind::Ident,
            }
        } else if byte.is_ascii_digit()
            || (byte == b'-'
                && primary_expected
                && source
                    .get(cursor + 1)
                    .is_some_and(|next| next.is_ascii_digit()))
        {
            if byte == b'-' {
                cursor += 1;
            }
            while cursor < source.len() && source[cursor].is_ascii_digit() {
                cursor += 1;
            }
            if source.get(cursor).is_some_and(|next| {
                next.is_ascii_lowercase() || next.is_ascii_digit() || *next == b'_'
            }) {
                return Err(ERR_NON_CANONICAL_INTEGER_TEXT);
            }
            TokenKind::Integer(parse_integer(&source[token_start..cursor])?)
        } else {
            match byte {
                b'(' => {
                    cursor += 1;
                    TokenKind::LParen
                }
                b')' => {
                    cursor += 1;
                    TokenKind::RParen
                }
                b'{' => {
                    cursor += 1;
                    TokenKind::LBrace
                }
                b'}' => {
                    cursor += 1;
                    TokenKind::RBrace
                }
                b';' => {
                    cursor += 1;
                    TokenKind::Semicolon
                }
                b'=' if source.get(cursor + 1) == Some(&b'=') => {
                    cursor += 2;
                    TokenKind::Eq
                }
                b'=' => {
                    cursor += 1;
                    TokenKind::Assign
                }
                b'!' if source.get(cursor + 1) == Some(&b'=') => {
                    cursor += 2;
                    TokenKind::Ne
                }
                b'<' if source.get(cursor + 1) == Some(&b'=') => {
                    cursor += 2;
                    TokenKind::Le
                }
                b'<' => {
                    cursor += 1;
                    TokenKind::Lt
                }
                b'>' if source.get(cursor + 1) == Some(&b'=') => {
                    cursor += 2;
                    TokenKind::Ge
                }
                b'>' => {
                    cursor += 1;
                    TokenKind::Gt
                }
                b'-' if source.get(cursor + 1) == Some(&b'>') => {
                    cursor += 2;
                    TokenKind::Arrow
                }
                b'+' if primary_expected
                    && source
                        .get(cursor + 1)
                        .is_some_and(|next| next.is_ascii_digit()) =>
                {
                    return Err(ERR_NON_CANONICAL_INTEGER_TEXT);
                }
                b'+' => {
                    cursor += 1;
                    TokenKind::Plus
                }
                b'-' => {
                    cursor += 1;
                    TokenKind::Minus
                }
                b'*' => {
                    cursor += 1;
                    TokenKind::Star
                }
                b'/' => {
                    cursor += 1;
                    TokenKind::Slash
                }
                _ => return Err(ERR_INVALID_CHARACTER),
            }
        };
        primary_expected = matches!(
            kind,
            TokenKind::LParen
                | TokenKind::LBrace
                | TokenKind::Assign
                | TokenKind::Semicolon
                | TokenKind::Eq
                | TokenKind::Ne
                | TokenKind::Lt
                | TokenKind::Gt
                | TokenKind::Le
                | TokenKind::Ge
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::If
        );
        if count < MAX_TOKENS {
            items[count] = Token {
                kind,
                start: token_start as u16,
                end: cursor as u16,
            };
        }
        count += 1;
    }
    if count > MAX_TOKENS {
        return Err(ERR_TOKEN_QUOTA_EXCEEDED);
    }
    Ok(Tokens { items, len: count })
}

fn parse_integer(text: &[u8]) -> Result<i32, &'static str> {
    let (negative, digits) = if let Some(digits) = text.strip_prefix(b"-") {
        (true, digits)
    } else {
        (false, text)
    };
    if digits.is_empty()
        || (digits[0] == b'0' && (negative || digits.len() != 1))
        || (digits[0] != b'0' && !matches!(digits[0], b'1'..=b'9'))
    {
        return Err(ERR_NON_CANONICAL_INTEGER_TEXT);
    }
    let limit = if negative {
        i32::MAX as u64 + 1
    } else {
        i32::MAX as u64
    };
    let mut value = 0u64;
    for &digit in digits {
        value = value
            .checked_mul(10)
            .and_then(|value| value.checked_add((digit - b'0') as u64))
            .filter(|value| *value <= limit)
            .ok_or(ERR_NON_CANONICAL_INTEGER_TEXT)?;
    }
    if negative {
        if value == i32::MAX as u64 + 1 {
            Ok(i32::MIN)
        } else {
            Ok(-(value as i32))
        }
    } else {
        Ok(value as i32)
    }
}

struct Parser<'a> {
    source: &'a [u8],
    body_start: usize,
    tokens: &'a Tokens,
    cursor: usize,
    nodes: [Node; MAX_NODES],
    node_count: usize,
    lets: [LetDecl; MAX_LETS],
    let_count: usize,
}

impl<'a> Parser<'a> {
    fn new(source: &'a [u8], body_start: usize, tokens: &'a Tokens) -> Self {
        Self {
            source,
            body_start,
            tokens,
            cursor: 0,
            nodes: [Node::EMPTY; MAX_NODES],
            node_count: 0,
            lets: [LetDecl::EMPTY; MAX_LETS],
            let_count: 0,
        }
    }

    fn parse(mut self) -> Result<Program, &'static str> {
        let fn_token = self.expect(TokenKind::Fn)?;
        if fn_token.start as usize != self.body_start {
            return Err(ERR_SYNTAX);
        }
        let name = self.expect(TokenKind::Ident)?;
        if !has_whitespace(fn_token, name) || self.token_text(name) != ENTRYPOINT {
            return Err(ERR_SYNTAX);
        }
        let left_paren = self.expect(TokenKind::LParen)?;
        if name.end != left_paren.start {
            return Err(ERR_SYNTAX);
        }
        let right_paren = self.expect(TokenKind::RParen)?;
        if left_paren.end != right_paren.start {
            return Err(ERR_SYNTAX);
        }
        let arrow = self.expect(TokenKind::Arrow)?;
        if !has_whitespace(right_paren, arrow) {
            return Err(ERR_SYNTAX);
        }
        let result_type = self.expect(TokenKind::I32)?;
        if !has_whitespace(arrow, result_type) {
            return Err(ERR_SYNTAX);
        }
        let left_brace = self.expect(TokenKind::LBrace)?;
        if !has_whitespace(result_type, left_brace) {
            return Err(ERR_SYNTAX);
        }

        while self.peek_kind() == Some(TokenKind::Let) {
            self.parse_let()?;
        }
        let result = self.parse_expr(0)?;
        let right_brace = self.expect(TokenKind::RBrace)?;
        if self.cursor != self.tokens.len
            || right_brace.end as usize != self.source.len().saturating_sub(1)
        {
            return Err(ERR_SYNTAX);
        }
        Ok(Program {
            nodes: self.nodes,
            node_count: self.node_count,
            lets: self.lets,
            let_count: self.let_count,
            result,
        })
    }

    fn parse_let(&mut self) -> Result<(), &'static str> {
        if self.let_count == MAX_LETS {
            return Err(ERR_QUOTA_EXCEEDED);
        }
        let keyword = self.expect(TokenKind::Let)?;
        let name = self.expect(TokenKind::Ident)?;
        if !has_whitespace(keyword, name) {
            return Err(ERR_SYNTAX);
        }
        self.expect(TokenKind::Assign)?;
        let value = self.parse_expr(0)?;
        self.expect(TokenKind::Semicolon)?;
        self.lets[self.let_count] = LetDecl {
            name_start: name.start,
            name_end: name.end,
            value,
        };
        self.let_count += 1;
        Ok(())
    }

    fn parse_expr(&mut self, nesting: u8) -> Result<u16, &'static str> {
        if self.peek_kind() == Some(TokenKind::If) {
            self.parse_if(nesting)
        } else {
            self.parse_equality(nesting)
        }
    }

    fn parse_if(&mut self, nesting: u8) -> Result<u16, &'static str> {
        let keyword = self.expect(TokenKind::If)?;
        let first = self.peek().ok_or(ERR_SYNTAX)?;
        if !has_whitespace(keyword, first) {
            return Err(ERR_SYNTAX);
        }
        let condition = self.parse_expr(nesting)?;
        let (then_node, then_brace) = self.parse_branch(nesting)?;
        let else_keyword = self.expect(TokenKind::Else)?;
        if !has_whitespace(then_brace, else_keyword) {
            return Err(ERR_SYNTAX);
        }
        let else_brace = self.peek().ok_or(ERR_SYNTAX)?;
        if else_brace.kind != TokenKind::LBrace || !has_whitespace(else_keyword, else_brace) {
            return Err(ERR_SYNTAX);
        }
        let (else_node, _) = self.parse_branch(nesting)?;
        self.add_if(condition, then_node, else_node)
    }

    fn parse_branch(&mut self, nesting: u8) -> Result<(u16, Token), &'static str> {
        self.expect(TokenKind::LBrace)?;
        let node = self.parse_expr(nesting)?;
        let right_brace = self.expect(TokenKind::RBrace)?;
        Ok((node, right_brace))
    }

    fn parse_equality(&mut self, nesting: u8) -> Result<u16, &'static str> {
        let (mut left, left_relation) = self.parse_relation(nesting)?;
        if matches!(self.peek_kind(), Some(TokenKind::Eq | TokenKind::Ne)) {
            if left_relation {
                return Err(ERR_SYNTAX);
            }
            let op = if self.take(TokenKind::Eq).is_some() {
                BinaryOp::Eq
            } else {
                self.expect(TokenKind::Ne)?;
                BinaryOp::Ne
            };
            let (right, right_relation) = self.parse_relation(nesting)?;
            if right_relation {
                return Err(ERR_SYNTAX);
            }
            left = self.add_binary(op, left, right)?;
        }
        if self.peek_kind().is_some_and(is_comparison_token) {
            return Err(ERR_SYNTAX);
        }
        Ok(left)
    }

    fn parse_relation(&mut self, nesting: u8) -> Result<(u16, bool), &'static str> {
        let mut left = self.parse_additive(nesting)?;
        let Some(kind) = self.peek_kind() else {
            return Ok((left, false));
        };
        let op = match kind {
            TokenKind::Lt => BinaryOp::Lt,
            TokenKind::Gt => BinaryOp::Gt,
            TokenKind::Le => BinaryOp::Le,
            TokenKind::Ge => BinaryOp::Ge,
            _ => return Ok((left, false)),
        };
        self.cursor += 1;
        let right = self.parse_additive(nesting)?;
        left = self.add_binary(op, left, right)?;
        Ok((left, true))
    }

    fn parse_additive(&mut self, nesting: u8) -> Result<u16, &'static str> {
        let mut left = self.parse_multiply(nesting)?;
        loop {
            let op = match self.peek_kind() {
                Some(TokenKind::Plus) => BinaryOp::Add,
                Some(TokenKind::Minus) => BinaryOp::Sub,
                _ => return Ok(left),
            };
            self.cursor += 1;
            let right = self.parse_multiply(nesting)?;
            left = self.add_binary(op, left, right)?;
        }
    }

    fn parse_multiply(&mut self, nesting: u8) -> Result<u16, &'static str> {
        let mut left = self.parse_primary(nesting)?;
        loop {
            let op = match self.peek_kind() {
                Some(TokenKind::Star) => BinaryOp::Mul,
                Some(TokenKind::Slash) => BinaryOp::Div,
                _ => return Ok(left),
            };
            self.cursor += 1;
            let right = self.parse_primary(nesting)?;
            left = self.add_binary(op, left, right)?;
        }
    }

    fn parse_primary(&mut self, nesting: u8) -> Result<u16, &'static str> {
        let token = self.peek().ok_or(ERR_SYNTAX)?;
        match token.kind {
            TokenKind::Integer(value) => {
                self.cursor += 1;
                self.add_node(NodeKind::Integer(value), 1)
            }
            TokenKind::Ident => {
                self.cursor += 1;
                self.add_node(
                    NodeKind::Name {
                        start: token.start,
                        end: token.end,
                    },
                    1,
                )
            }
            TokenKind::LParen => {
                if nesting == MAX_DEPTH {
                    return Err(ERR_QUOTA_EXCEEDED);
                }
                self.cursor += 1;
                let node = self.parse_expr(nesting + 1)?;
                self.expect(TokenKind::RParen)?;
                Ok(node)
            }
            _ => Err(ERR_SYNTAX),
        }
    }

    fn add_binary(&mut self, op: BinaryOp, left: u16, right: u16) -> Result<u16, &'static str> {
        let depth = self.nodes[left as usize]
            .depth
            .max(self.nodes[right as usize].depth)
            .saturating_add(1);
        self.add_node(NodeKind::Binary { op, left, right }, depth)
    }

    fn add_if(
        &mut self,
        condition: u16,
        then_node: u16,
        else_node: u16,
    ) -> Result<u16, &'static str> {
        let depth = self.nodes[condition as usize]
            .depth
            .max(self.nodes[then_node as usize].depth)
            .max(self.nodes[else_node as usize].depth)
            .saturating_add(1);
        self.add_node(
            NodeKind::If {
                condition,
                then_node,
                else_node,
            },
            depth,
        )
    }

    fn add_node(&mut self, kind: NodeKind, depth: u8) -> Result<u16, &'static str> {
        if self.node_count == MAX_NODES || depth > MAX_DEPTH {
            return Err(ERR_QUOTA_EXCEEDED);
        }
        let index = self.node_count;
        self.nodes[index] = Node { kind, depth };
        self.node_count += 1;
        Ok(index as u16)
    }

    fn token_text(&self, token: Token) -> &[u8] {
        &self.source[token.start as usize..token.end as usize]
    }

    fn peek(&self) -> Option<Token> {
        self.tokens
            .items
            .get(self.cursor)
            .copied()
            .filter(|_| self.cursor < self.tokens.len)
    }

    fn peek_kind(&self) -> Option<TokenKind> {
        self.peek().map(|token| token.kind)
    }

    fn take(&mut self, kind: TokenKind) -> Option<Token> {
        let token = self.peek()?;
        if token.kind != kind {
            return None;
        }
        self.cursor += 1;
        Some(token)
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, &'static str> {
        self.take(kind).ok_or(ERR_SYNTAX)
    }
}

fn has_whitespace(left: Token, right: Token) -> bool {
    left.end < right.start
}

fn is_comparison_token(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Eq
            | TokenKind::Ne
            | TokenKind::Lt
            | TokenKind::Gt
            | TokenKind::Le
            | TokenKind::Ge
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Type {
    I32,
    Bool,
}

fn bind_and_typecheck(source: &[u8], program: &mut Program) -> Result<(), &'static str> {
    for right in 0..program.let_count {
        for left in 0..right {
            if binding_name(source, program.lets[left]) == binding_name(source, program.lets[right])
            {
                return Err(ERR_DUPLICATE_BINDING);
            }
        }
    }

    for index in 0..program.let_count {
        let value = program.lets[index].value;
        resolve_node(source, program, value, index)?;
    }
    let result = program.result;
    resolve_node(source, program, result, program.let_count)?;

    let mut binding_types = [Type::I32; MAX_LETS];
    for index in 0..program.let_count {
        let binding_type = type_of(program, program.lets[index].value, &binding_types)?;
        binding_types[index] = binding_type;
    }
    if type_of(program, program.result, &binding_types)? != Type::I32 {
        return Err(ERR_TYPE_MISMATCH);
    }
    Ok(())
}

fn binding_name(source: &[u8], binding: LetDecl) -> &[u8] {
    &source[binding.name_start as usize..binding.name_end as usize]
}

fn resolve_node(
    source: &[u8],
    program: &mut Program,
    node: u16,
    visible_bindings: usize,
) -> Result<(), &'static str> {
    match program.nodes[node as usize].kind {
        NodeKind::Name { start, end } => {
            let name = &source[start as usize..end as usize];
            let local = program.lets[..visible_bindings]
                .iter()
                .position(|binding| binding_name(source, *binding) == name)
                .ok_or(ERR_UNKNOWN_BINDING)?;
            program.nodes[node as usize].kind = NodeKind::Local(local as u8);
            Ok(())
        }
        NodeKind::Binary { left, right, .. } => {
            resolve_node(source, program, left, visible_bindings)?;
            resolve_node(source, program, right, visible_bindings)
        }
        NodeKind::If {
            condition,
            then_node,
            else_node,
        } => {
            resolve_node(source, program, condition, visible_bindings)?;
            resolve_node(source, program, then_node, visible_bindings)?;
            resolve_node(source, program, else_node, visible_bindings)
        }
        NodeKind::Integer(_) | NodeKind::Local(_) => Ok(()),
        NodeKind::Empty => Err(ERR_BACKEND),
    }
}

fn type_of(
    program: &Program,
    node: u16,
    binding_types: &[Type; MAX_LETS],
) -> Result<Type, &'static str> {
    match program.nodes[node as usize].kind {
        NodeKind::Integer(_) => Ok(Type::I32),
        NodeKind::Local(index) => Ok(binding_types[index as usize]),
        NodeKind::Binary { op, left, right } => {
            let left_type = type_of(program, left, binding_types)?;
            let right_type = type_of(program, right, binding_types)?;
            if left_type != Type::I32 || right_type != Type::I32 {
                return Err(ERR_TYPE_MISMATCH);
            }
            if matches!(
                op,
                BinaryOp::Eq
                    | BinaryOp::Ne
                    | BinaryOp::Lt
                    | BinaryOp::Gt
                    | BinaryOp::Le
                    | BinaryOp::Ge
            ) {
                Ok(Type::Bool)
            } else {
                Ok(Type::I32)
            }
        }
        NodeKind::If {
            condition,
            then_node,
            else_node,
        } => {
            let condition_type = type_of(program, condition, binding_types)?;
            let then_type = type_of(program, then_node, binding_types)?;
            let else_type = type_of(program, else_node, binding_types)?;
            if condition_type != Type::Bool || then_type != else_type {
                return Err(ERR_TYPE_MISMATCH);
            }
            Ok(then_type)
        }
        NodeKind::Name { .. } | NodeKind::Empty => Err(ERR_BACKEND),
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Hazard {
    Zero,
    Overflow,
}

fn validate_arithmetic(program: &Program) -> Result<(), &'static str> {
    evaluate_program(program, Hazard::Zero)?;
    evaluate_program(program, Hazard::Overflow)?;
    Ok(())
}

fn evaluate_program(program: &Program, hazard: Hazard) -> Result<i32, &'static str> {
    let mut values = [0i32; MAX_LETS];
    for index in 0..program.let_count {
        let value = evaluate_node(program, program.lets[index].value, &values, hazard)?;
        values[index] = value;
    }
    evaluate_node(program, program.result, &values, hazard)
}

fn evaluate_node(
    program: &Program,
    node: u16,
    values: &[i32; MAX_LETS],
    hazard: Hazard,
) -> Result<i32, &'static str> {
    match program.nodes[node as usize].kind {
        NodeKind::Integer(value) => Ok(value),
        NodeKind::Local(index) => Ok(values[index as usize]),
        NodeKind::Binary { op, left, right } => {
            let left = evaluate_node(program, left, values, hazard)?;
            let right = evaluate_node(program, right, values, hazard)?;
            match op {
                BinaryOp::Eq => Ok((left == right) as i32),
                BinaryOp::Ne => Ok((left != right) as i32),
                BinaryOp::Lt => Ok((left < right) as i32),
                BinaryOp::Gt => Ok((left > right) as i32),
                BinaryOp::Le => Ok((left <= right) as i32),
                BinaryOp::Ge => Ok((left >= right) as i32),
                BinaryOp::Add => Ok(left.wrapping_add(right)),
                BinaryOp::Sub => Ok(left.wrapping_sub(right)),
                BinaryOp::Mul => Ok(left.wrapping_mul(right)),
                BinaryOp::Div => {
                    if right == 0 {
                        if hazard == Hazard::Zero {
                            return Err(ERR_DIVISION_BY_ZERO);
                        }
                        return Ok(0);
                    }
                    if left == i32::MIN && right == -1 {
                        if hazard == Hazard::Overflow {
                            return Err(ERR_DIVISION_OVERFLOW);
                        }
                        return Ok(i32::MIN);
                    }
                    Ok(left / right)
                }
            }
        }
        NodeKind::If {
            condition,
            then_node,
            else_node,
        } => {
            let condition = evaluate_node(program, condition, values, hazard)?;
            let then_value = evaluate_node(program, then_node, values, hazard)?;
            let else_value = evaluate_node(program, else_node, values, hazard)?;
            Ok(if condition != 0 {
                then_value
            } else {
                else_value
            })
        }
        NodeKind::Name { .. } | NodeKind::Empty => Err(ERR_BACKEND),
    }
}

fn emit(program: &Program) -> Result<WasmModuleBytes, &'static str> {
    let mut emitter =
        TypedFunctionEmitter::new(program.let_count as u32).map_err(map_backend_error)?;
    for index in 0..program.let_count {
        emit_node(program, program.lets[index].value, &mut emitter)?;
        emitter
            .push(TypedInstruction::LocalSet(index as u32))
            .map_err(map_backend_error)?;
    }
    emit_node(program, program.result, &mut emitter)?;
    let output = emitter.finish().map_err(map_backend_error)?;
    ensure_output_bound(output.len())?;
    Ok(output)
}

fn emit_node(
    program: &Program,
    node: u16,
    emitter: &mut TypedFunctionEmitter,
) -> Result<(), &'static str> {
    match program.nodes[node as usize].kind {
        NodeKind::Integer(value) => emitter
            .push(TypedInstruction::I32Const(value))
            .map_err(map_backend_error),
        NodeKind::Local(index) => emitter
            .push(TypedInstruction::LocalGet(index as u32))
            .map_err(map_backend_error),
        NodeKind::Binary { op, left, right } => {
            emit_node(program, left, emitter)?;
            emit_node(program, right, emitter)?;
            let instruction = match op {
                BinaryOp::Eq => TypedInstruction::I32Eq,
                BinaryOp::Ne => TypedInstruction::I32Ne,
                BinaryOp::Lt => TypedInstruction::I32LtS,
                BinaryOp::Gt => TypedInstruction::I32GtS,
                BinaryOp::Le => TypedInstruction::I32LeS,
                BinaryOp::Ge => TypedInstruction::I32GeS,
                BinaryOp::Add => TypedInstruction::I32Add,
                BinaryOp::Sub => TypedInstruction::I32Sub,
                BinaryOp::Mul => TypedInstruction::I32Mul,
                BinaryOp::Div => TypedInstruction::I32DivS,
            };
            emitter.push(instruction).map_err(map_backend_error)
        }
        NodeKind::If {
            condition,
            then_node,
            else_node,
        } => {
            emit_node(program, condition, emitter)?;
            emitter
                .push(TypedInstruction::IfI32)
                .map_err(map_backend_error)?;
            emit_node(program, then_node, emitter)?;
            emitter
                .push(TypedInstruction::Else)
                .map_err(map_backend_error)?;
            emit_node(program, else_node, emitter)?;
            emitter
                .push(TypedInstruction::End)
                .map_err(map_backend_error)
        }
        NodeKind::Name { .. } | NodeKind::Empty => Err(ERR_BACKEND),
    }
}

fn map_backend_error(reason: &'static str) -> &'static str {
    if reason == ERR_OVERSIZE_OUTPUT {
        ERR_OVERSIZE_OUTPUT
    } else {
        ERR_BACKEND
    }
}

fn ensure_output_bound(len: usize) -> Result<(), &'static str> {
    if len > raios_wasm_ir::MAX_WASM_OUTPUT_BYTES {
        Err(ERR_OVERSIZE_OUTPUT)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{format, string::String};
    use wasmi::{Engine, Linker, Module, Store};

    #[test]
    fn literal_precedence_golden_returns_42() {
        let expected = [
            // Magic/version, type () -> i32, and function type index 0.
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01,
            0x7f, 0x03, 0x02, 0x01, 0x00, // Export function 0 as "raios_service_main".
            0x07, 0x16, 0x01, 0x12, 0x72, 0x61, 0x69, 0x6f, 0x73, 0x5f, 0x73, 0x65, 0x72, 0x76,
            0x69, 0x63, 0x65, 0x5f, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            // No locals; 2 + (5 * 8); return; end.
            0x0a, 0x0d, 0x01, 0x0b, 0x00, 0x41, 0x02, 0x41, 0x05, 0x41, 0x08, 0x6c, 0x6a, 0x0f,
            0x0b,
        ];
        assert_literal_golden("2 + 5 * 8", &expected, 42);
    }

    #[test]
    fn literal_demo_golden_returns_31415() {
        let expected = [
            // Magic/version, type () -> i32, and function type index 0.
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01,
            0x7f, 0x03, 0x02, 0x01, 0x00, // Export function 0 as "raios_service_main".
            0x07, 0x16, 0x01, 0x12, 0x72, 0x61, 0x69, 0x6f, 0x73, 0x5f, 0x73, 0x65, 0x72, 0x76,
            0x69, 0x63, 0x65, 0x5f, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            // One local: answer=6*7; answer==42; if i32 {31415} else {0}.
            0x0a, 0x1d, 0x01, 0x1b, 0x01, 0x01, 0x7f, 0x41, 0x06, 0x41, 0x07, 0x6c, 0x21, 0x00,
            0x20, 0x00, 0x41, 0x2a, 0x46, 0x04, 0x7f, 0x41, 0xb7, 0xf5, 0x01, 0x05, 0x41, 0x00,
            0x0b, 0x0f, 0x0b,
        ];
        assert_literal_golden(
            "let answer = 6 * 7; if answer == 42 { 31415 } else { 0 }",
            &expected,
            31_415,
        );
    }

    #[test]
    fn literal_wrapping_division_golden_returns_1073741824() {
        let expected = [
            // Magic/version, type () -> i32, and function type index 0.
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01,
            0x7f, 0x03, 0x02, 0x01, 0x00, // Export function 0 as "raios_service_main".
            0x07, 0x16, 0x01, 0x12, 0x72, 0x61, 0x69, 0x6f, 0x73, 0x5f, 0x73, 0x65, 0x72, 0x76,
            0x69, 0x63, 0x65, 0x5f, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            // No locals; (i32::MAX + 1 wraps to MIN) / -2; return; end.
            0x0a, 0x11, 0x01, 0x0f, 0x00, 0x41, 0xff, 0xff, 0xff, 0xff, 0x07, 0x41, 0x01, 0x6a,
            0x41, 0x7e, 0x6d, 0x0f, 0x0b,
        ];
        assert_literal_golden("(2147483647 + 1) / -2", &expected, 1_073_741_824);
    }

    #[test]
    fn every_source_failure_reason_and_precedence_is_exercised() {
        let cases = [
            ("RAIOS_LANG_V0\n", ERR_WRONG_VERSION_LINE),
            (
                "RAIOS_LANG_V1\nfn raios_service_main() -> i32 {\t0 }\n",
                ERR_INVALID_CHARACTER,
            ),
            (
                "RAIOS_LANG_V1\nfn raios_service_main() -> i32 { +1 }\n",
                ERR_NON_CANONICAL_INTEGER_TEXT,
            ),
            (
                "RAIOS_LANG_V1\nfn raios_service_main() -> i32 { 01 }\n",
                ERR_NON_CANONICAL_INTEGER_TEXT,
            ),
            (
                "RAIOS_LANG_V1\nfn raios_service_main() -> i32 { -0 }\n",
                ERR_NON_CANONICAL_INTEGER_TEXT,
            ),
            (
                "RAIOS_LANG_V1\nfn raios_service_main() -> i32 { 2147483648 }\n",
                ERR_NON_CANONICAL_INTEGER_TEXT,
            ),
            (
                "RAIOS_LANG_V1\nfn raios_service_main() -> i32 { 1 < 2 < 3 }\n",
                ERR_SYNTAX,
            ),
            (
                "RAIOS_LANG_V1\nfn raios_service_main() -> i32 { let x = 1 0 }\n",
                ERR_SYNTAX,
            ),
            (
                "RAIOS_LANG_V1\nfn raios_service_main() -> i32 { let x = 0; let x = 1; x }\n",
                ERR_DUPLICATE_BINDING,
            ),
            (
                "RAIOS_LANG_V1\nfn raios_service_main() -> i32 { missing }\n",
                ERR_UNKNOWN_BINDING,
            ),
            (
                "RAIOS_LANG_V1\nfn raios_service_main() -> i32 { if 1 { 0 } else { 1 } }\n",
                ERR_TYPE_MISMATCH,
            ),
            (
                "RAIOS_LANG_V1\nfn raios_service_main() -> i32 { if 1 == 1 { 0 } else { 1 == 1 } }\n",
                ERR_TYPE_MISMATCH,
            ),
            (
                "RAIOS_LANG_V1\nfn raios_service_main() -> i32 { 1 == 1 }\n",
                ERR_TYPE_MISMATCH,
            ),
            (
                "RAIOS_LANG_V1\nfn raios_service_main() -> i32 { 1 / (2 - 2) }\n",
                ERR_DIVISION_BY_ZERO,
            ),
            (
                "RAIOS_LANG_V1\nfn raios_service_main() -> i32 { -2147483648 / -1 }\n",
                ERR_DIVISION_OVERFLOW,
            ),
        ];
        for (source, reason) in cases {
            assert_eq!(compile(source.as_bytes()), Err(reason), "{reason}");
        }

        let missing_terminator = b"RAIOS_LANG_V1\nfn raios_service_main() -> i32 { 0 }";
        assert_eq!(compile(missing_terminator), Err(ERR_MISSING_TERMINATOR));

        let mut non_ascii = source("0").into_bytes();
        non_ascii[20] = 0xff;
        assert_eq!(compile(&non_ascii), Err(ERR_NON_ASCII_SOURCE));

        let oversized_non_ascii = [0xff; MAX_SOURCE_BYTES + 1];
        assert_eq!(compile(&oversized_non_ascii), Err(ERR_OVERSIZE_SOURCE));

        assert_eq!(
            ensure_output_bound(raios_wasm_ir::MAX_WASM_OUTPUT_BYTES + 1),
            Err(ERR_OVERSIZE_OUTPUT)
        );
        assert_eq!(
            map_backend_error(raios_wasm_ir::ERR_TYPED_EMITTER_REJECTED),
            ERR_BACKEND
        );
    }

    #[test]
    fn binding_type_and_arithmetic_precedence_are_global() {
        assert_eq!(
            compile(source("let a = missing; let a = 0; a").as_bytes()),
            Err(ERR_DUPLICATE_BINDING)
        );
        assert_eq!(
            compile(source("let a = 1 + (1 == 1); missing").as_bytes()),
            Err(ERR_UNKNOWN_BINDING)
        );
        assert_eq!(
            compile(source("(1 == 1) + (1 / 0)").as_bytes()),
            Err(ERR_TYPE_MISMATCH)
        );
        assert_eq!(
            compile(source("if 1 == 1 { -2147483648 / -1 } else { 1 / 0 }").as_bytes()),
            Err(ERR_DIVISION_BY_ZERO)
        );
    }

    #[test]
    fn division_hazards_in_dead_branches_are_rejected() {
        assert_eq!(
            compile(source("if 1 == 1 { 7 } else { 1 / 0 }").as_bytes()),
            Err(ERR_DIVISION_BY_ZERO)
        );
        assert_eq!(
            compile(source("if 1 == 1 { 7 } else { -2147483648 / -1 }").as_bytes()),
            Err(ERR_DIVISION_OVERFLOW)
        );
    }

    #[test]
    fn lexical_boundaries_and_keyword_reservation_are_exact() {
        let name = "a".repeat(32);
        assert!(compile(source(&format!("let {name} = 1; {name}")).as_bytes()).is_ok());
        let name = "a".repeat(33);
        assert_eq!(
            compile(source(&format!("let {name} = 1; 0")).as_bytes()),
            Err(ERR_INVALID_CHARACTER)
        );
        assert_eq!(compile(source("let if = 1; 0").as_bytes()), Err(ERR_SYNTAX));
        assert_eq!(
            compile(source("1a").as_bytes()),
            Err(ERR_NON_CANONICAL_INTEGER_TEXT)
        );
        assert_runs(&source("2147483647"), i32::MAX);
        assert_runs(&source("-2147483648"), i32::MIN);
        assert_eq!(
            compile(source("-2147483649").as_bytes()),
            Err(ERR_NON_CANONICAL_INTEGER_TEXT)
        );
    }

    #[test]
    fn source_let_depth_token_and_node_caps_pin_exact_and_one_over() {
        let mut exact_source = source("0");
        let insert_at = exact_source.rfind('}').unwrap();
        let padding = " ".repeat(MAX_SOURCE_BYTES - exact_source.len());
        exact_source.insert_str(insert_at, &padding);
        assert_eq!(exact_source.len(), MAX_SOURCE_BYTES);
        assert!(compile(exact_source.as_bytes()).is_ok());
        exact_source.insert(insert_at, ' ');
        assert_eq!(compile(exact_source.as_bytes()), Err(ERR_OVERSIZE_SOURCE));

        let exact_lets = lets_source(MAX_LETS);
        assert!(compile(exact_lets.as_bytes()).is_ok());
        assert_eq!(
            compile(lets_source(MAX_LETS + 1).as_bytes()),
            Err(ERR_QUOTA_EXCEEDED)
        );

        let exact_depth = source(&format!(
            "{}0{}",
            "(".repeat(MAX_DEPTH as usize),
            ")".repeat(MAX_DEPTH as usize)
        ));
        assert!(compile(exact_depth.as_bytes()).is_ok());
        let over_depth = source(&format!(
            "{}0{}",
            "(".repeat(MAX_DEPTH as usize + 1),
            ")".repeat(MAX_DEPTH as usize + 1)
        ));
        assert_eq!(compile(over_depth.as_bytes()), Err(ERR_QUOTA_EXCEEDED));

        let exact_ast_depth = source(&format!("{}1", "1+".repeat(MAX_DEPTH as usize - 1)));
        assert!(compile(exact_ast_depth.as_bytes()).is_ok());
        let over_ast_depth = source(&format!("{}1", "1+".repeat(MAX_DEPTH as usize)));
        assert_eq!(compile(over_ast_depth.as_bytes()), Err(ERR_QUOTA_EXCEEDED));

        let mut wraps = 59;
        let expression = balanced_sum(128, "1", &mut wraps);
        assert_eq!(wraps, 0);
        let exact_tokens_nodes = source(&format!("let x = 0; {expression}"));
        let body_start = VERSION_LINE.len() + 1;
        let tokens = lex(exact_tokens_nodes.as_bytes(), body_start).unwrap();
        assert_eq!(tokens.len, MAX_TOKENS);
        let parsed = Parser::new(exact_tokens_nodes.as_bytes(), body_start, &tokens)
            .parse()
            .unwrap();
        assert_eq!(parsed.node_count, MAX_NODES);
        assert!(compile(exact_tokens_nodes.as_bytes()).is_ok());

        let token_over = source(&format!("let x = 0; {expression}+"));
        assert_eq!(
            compile(token_over.as_bytes()),
            Err(ERR_TOKEN_QUOTA_EXCEEDED)
        );

        let mut no_wraps = 0;
        let expression = balanced_sum(128, "1", &mut no_wraps);
        let node_over = source(&format!("let x = 0; let y = 0; {expression}"));
        assert_eq!(compile(node_over.as_bytes()), Err(ERR_QUOTA_EXCEEDED));
    }

    #[test]
    fn max_shape_output_is_pinned_below_4096_and_deterministic() {
        let mut body = String::new();
        for index in 0..MAX_LETS {
            body.push_str(&format!("let a{index} = -2147483648; "));
        }
        let then_expr = chunked_sum(55, "-2147483648");
        let else_expr = chunked_sum(56, "-2147483648");
        body.push_str(&format!(
            "if -2147483648 < 0 {{ {then_expr} }} else {{ {else_expr} }}"
        ));
        let source = source(&body);
        let body_start = VERSION_LINE.len() + 1;
        let tokens = lex(source.as_bytes(), body_start).unwrap();
        let parsed = Parser::new(source.as_bytes(), body_start, &tokens)
            .parse()
            .unwrap();
        assert_eq!(parsed.let_count, MAX_LETS);
        assert_eq!(parsed.node_count, MAX_NODES);
        let first = compile(source.as_bytes()).unwrap();
        let second = compile(source.as_bytes()).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.len(), 1098);
        assert!(first.len() < raios_wasm_ir::MAX_WASM_OUTPUT_BYTES);
        assert!(Module::new(&Engine::default(), first.as_slice()).is_ok());
    }

    #[test]
    fn signed_literals_and_all_operators_execute_with_wasm_semantics() {
        for (body, expected) in [
            ("1-1", 0),
            ("1--1", 2),
            ("-2147483648 - 1", 2_147_483_647),
            ("1073741824 * 4", 0),
            ("-7 / 3", -2),
            ("if -1 < 0 { 1 } else { 0 }", 1),
            ("if 1 > 0 { 1 } else { 0 }", 1),
            ("if 1 <= 1 { 1 } else { 0 }", 1),
            ("if 1 >= 1 { 1 } else { 0 }", 1),
            ("if 1 != 2 { 1 } else { 0 }", 1),
        ] {
            assert_runs(&source(body), expected);
        }
    }

    fn assert_literal_golden(body: &str, expected: &[u8], result: i32) {
        let source = source(body);
        let first = compile(source.as_bytes()).unwrap();
        let second = compile(source.as_bytes()).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.as_slice(), expected);
        assert_runs_bytes(expected, result);
    }

    fn assert_runs(source: &str, expected: i32) {
        let bytes = compile(source.as_bytes()).unwrap();
        assert_runs_bytes(bytes.as_slice(), expected);
    }

    fn assert_runs_bytes(bytes: &[u8], expected: i32) {
        let engine = Engine::default();
        let module = Module::new(&engine, bytes).unwrap();
        let mut store = Store::new(&engine, ());
        let linker = Linker::<()>::new(&engine);
        let instance = linker
            .instantiate(&mut store, &module)
            .unwrap()
            .start(&mut store)
            .unwrap();
        let function = instance
            .get_typed_func::<(), i32>(&store, "raios_service_main")
            .unwrap();
        assert_eq!(function.call(&mut store, ()).unwrap(), expected);
    }

    fn source(body: &str) -> String {
        format!("RAIOS_LANG_V1\nfn raios_service_main() -> i32 {{ {body} }}\n")
    }

    fn lets_source(count: usize) -> String {
        let mut body = String::new();
        for index in 0..count {
            body.push_str(&format!("let a{index} = {index}; "));
        }
        body.push_str(if count == 0 { "0" } else { "a0" });
        source(&body)
    }

    fn balanced_sum(leaves: usize, literal: &str, wraps: &mut usize) -> String {
        if leaves == 1 {
            if *wraps != 0 {
                *wraps -= 1;
                return format!("({literal})");
            }
            return String::from(literal);
        }
        let left_leaves = leaves / 2;
        let right_leaves = leaves - left_leaves;
        let left = balanced_sum(left_leaves, literal, wraps);
        let right = balanced_sum(right_leaves, literal, wraps);
        if right_leaves == 1 {
            format!("{left}+{right}")
        } else {
            format!("{left}+({right})")
        }
    }

    fn chunked_sum(leaves: usize, literal: &str) -> String {
        let mut result = String::new();
        let mut remaining = leaves;
        while remaining != 0 {
            let chunk = remaining.min(8);
            let mut expression = String::from(literal);
            for _ in 1..chunk {
                expression.push('+');
                expression.push_str(literal);
            }
            if result.is_empty() {
                result = expression;
            } else {
                result.push_str("+(");
                result.push_str(&expression);
                result.push(')');
            }
            remaining -= chunk;
        }
        result
    }
}
