//! High-speed lexical scanner for the Nova programming language.

use crate::token::{Span, Token, TokenKind};

pub struct Lexer<'a> {
    _source: &'a str,
    chars: Vec<(usize, char)>,
    cursor: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            _source: source,
            chars: source.char_indices().collect(),
            cursor: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.advance();
            } else if c == '/' && self.peek_next_char() == Some('/') {
                // Single-line comment
                self.skip_line_comment();
            } else if c == '/' && self.peek_next_char() == Some('*') {
                // Multi-line block comment
                self.skip_block_comment()?;
            } else if c.is_ascii_alphabetic() || c == '_' {
                tokens.push(self.lex_identifier_or_keyword());
            } else if c.is_ascii_digit() {
                tokens.push(self.lex_number()?);
            } else if c == '"' {
                tokens.push(self.lex_string()?);
            } else {
                tokens.push(self.lex_operator_or_delimiter()?);
            }
        }

        tokens.push(Token::new(TokenKind::Eof, Span::new(self.line, self.col, 0)));
        Ok(tokens)
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.cursor).map(|(_, c)| *c)
    }

    fn peek_next_char(&self) -> Option<char> {
        self.chars.get(self.cursor + 1).map(|(_, c)| *c)
    }

    fn advance(&mut self) -> Option<char> {
        if let Some((_, c)) = self.chars.get(self.cursor) {
            self.cursor += 1;
            if *c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(*c)
        } else {
            None
        }
    }

    fn skip_line_comment(&mut self) {
        // Skip '//'
        self.advance();
        self.advance();
        while let Some(c) = self.peek_char() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) -> Result<(), String> {
        let start_line = self.line;
        let start_col = self.col;
        self.advance(); // '/'
        self.advance(); // '*'

        while let Some(c) = self.peek_char() {
            if c == '*' && self.peek_next_char() == Some('/') {
                self.advance();
                self.advance();
                return Ok(());
            }
            self.advance();
        }

        Err(format!("Unterminated block comment starting at {}:{}", start_line, start_col))
    }

    fn lex_identifier_or_keyword(&mut self) -> Token {
        let start_col = self.col;
        let start_line = self.line;
        let mut ident = String::new();

        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let len = ident.len();
        let span = Span::new(start_line, start_col, len);

        let kind = match ident.as_str() {
            "fn" => TokenKind::Fn,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "return" => TokenKind::Return,
            "export" => TokenKind::Export,
            "import" => TokenKind::Import,
            "memory" => TokenKind::Memory,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "break" => TokenKind::Break,
            "i32" => TokenKind::TypeI32,
            "i64" => TokenKind::TypeI64,
            "f32" => TokenKind::TypeF32,
            "f64" => TokenKind::TypeF64,
            "bool" => TokenKind::TypeBool,
            "void" => TokenKind::TypeVoid,
            _ => TokenKind::Ident(ident),
        };

        Token::new(kind, span)
    }

    fn lex_number(&mut self) -> Result<Token, String> {
        let start_col = self.col;
        let start_line = self.line;
        let mut text = String::new();
        let mut is_float = false;

        // Check for hex (0x) or binary (0b)
        if self.peek_char() == Some('0') {
            match self.peek_next_char() {
                Some('x') | Some('X') => {
                    self.advance(); // '0'
                    self.advance(); // 'x'
                    let mut hex = String::new();
                    while let Some(c) = self.peek_char() {
                        if c.is_ascii_hexdigit() || c == '_' {
                            if c != '_' {
                                hex.push(c);
                            }
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    let val = i64::from_str_radix(&hex, 16)
                        .map_err(|e| format!("Invalid hex literal at {}:{}: {}", start_line, start_col, e))?;
                    return Ok(Token::new(TokenKind::Int(val), Span::new(start_line, start_col, hex.len() + 2)));
                }
                Some('b') | Some('B') => {
                    self.advance(); // '0'
                    self.advance(); // 'b'
                    let mut bin = String::new();
                    while let Some(c) = self.peek_char() {
                        if c == '0' || c == '1' || c == '_' {
                            if c != '_' {
                                bin.push(c);
                            }
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    let val = i64::from_str_radix(&bin, 2)
                        .map_err(|e| format!("Invalid binary literal at {}:{}: {}", start_line, start_col, e))?;
                    return Ok(Token::new(TokenKind::Int(val), Span::new(start_line, start_col, bin.len() + 2)));
                }
                _ => {}
            }
        }

        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                text.push(c);
                self.advance();
            } else if c == '.' && !is_float && self.peek_next_char().map_or(false, |nc| nc.is_ascii_digit()) {
                is_float = true;
                text.push(c);
                self.advance();
            } else if c == '_' {
                // allow numeric separators
                self.advance();
            } else {
                break;
            }
        }

        let len = text.len();
        let span = Span::new(start_line, start_col, len);

        if is_float {
            let val = text.parse::<f64>()
                .map_err(|e| format!("Invalid float at {}:{}: {}", start_line, start_col, e))?;
            Ok(Token::new(TokenKind::Float(val), span))
        } else {
            let val = text.parse::<i64>()
                .map_err(|e| format!("Invalid integer at {}:{}: {}", start_line, start_col, e))?;
            Ok(Token::new(TokenKind::Int(val), span))
        }
    }

    fn lex_string(&mut self) -> Result<Token, String> {
        let start_col = self.col;
        let start_line = self.line;
        self.advance(); // opening '"'

        let mut str_val = String::new();
        while let Some(c) = self.peek_char() {
            if c == '"' {
                self.advance(); // closing '"'
                let len = str_val.len() + 2;
                return Ok(Token::new(TokenKind::Str(str_val), Span::new(start_line, start_col, len)));
            } else if c == '\\' {
                self.advance();
                match self.advance() {
                    Some('n') => str_val.push('\n'),
                    Some('t') => str_val.push('\t'),
                    Some('r') => str_val.push('\r'),
                    Some('0') => str_val.push('\0'),
                    Some('\\') => str_val.push('\\'),
                    Some('"') => str_val.push('"'),
                    Some(other) => str_val.push(other),
                    None => return Err(format!("Unterminated escape at {}:{}", self.line, self.col)),
                }
            } else {
                str_val.push(c);
                self.advance();
            }
        }

        Err(format!("Unterminated string literal starting at {}:{}", start_line, start_col))
    }

    fn lex_operator_or_delimiter(&mut self) -> Result<Token, String> {
        let start_col = self.col;
        let start_line = self.line;
        let c = self.advance().unwrap();

        let (kind, len) = match c {
            '+' => {
                if self.peek_char() == Some('=') {
                    self.advance();
                    (TokenKind::PlusEq, 2)
                } else {
                    (TokenKind::Plus, 1)
                }
            }
            '-' => {
                if self.peek_char() == Some('>') {
                    self.advance();
                    (TokenKind::Arrow, 2)
                } else if self.peek_char() == Some('=') {
                    self.advance();
                    (TokenKind::MinusEq, 2)
                } else {
                    (TokenKind::Minus, 1)
                }
            }
            '*' => (TokenKind::Star, 1),
            '/' => (TokenKind::Slash, 1),
            '%' => (TokenKind::Percent, 1),
            '=' => {
                if self.peek_char() == Some('=') {
                    self.advance();
                    (TokenKind::EqEq, 2)
                } else {
                    (TokenKind::Eq, 1)
                }
            }
            '!' => {
                if self.peek_char() == Some('=') {
                    self.advance();
                    (TokenKind::BangEq, 2)
                } else {
                    (TokenKind::Bang, 1)
                }
            }
            '<' => {
                if self.peek_char() == Some('=') {
                    self.advance();
                    (TokenKind::LtEq, 2)
                } else if self.peek_char() == Some('<') {
                    self.advance();
                    (TokenKind::Shl, 2)
                } else {
                    (TokenKind::Lt, 1)
                }
            }
            '>' => {
                if self.peek_char() == Some('=') {
                    self.advance();
                    (TokenKind::GtEq, 2)
                } else if self.peek_char() == Some('>') {
                    self.advance();
                    (TokenKind::Shr, 2)
                } else {
                    (TokenKind::Gt, 1)
                }
            }
            '&' => (TokenKind::Amp, 1),
            '|' => (TokenKind::Pipe, 1),
            '^' => (TokenKind::Caret, 1),
            '(' => (TokenKind::LParen, 1),
            ')' => (TokenKind::RParen, 1),
            '{' => (TokenKind::LBrace, 1),
            '}' => (TokenKind::RBrace, 1),
            '[' => (TokenKind::LBracket, 1),
            ']' => (TokenKind::RBracket, 1),
            ';' => (TokenKind::Semicolon, 1),
            ':' => (TokenKind::Colon, 1),
            ',' => (TokenKind::Comma, 1),
            '@' => (TokenKind::At, 1),
            unknown => return Err(format!("Unexpected character '{}' at {}:{}", unknown, start_line, start_col)),
        };

        Ok(Token::new(kind, Span::new(start_line, start_col, len)))
    }
}
