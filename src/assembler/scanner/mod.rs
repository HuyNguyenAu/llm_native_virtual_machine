use crate::assembler::scanner::token::{Token, TokenType};

pub mod token;

pub struct Scanner {
    source: String,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
    source_len: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        let source_len = source.len();

        Scanner {
            source,
            current: 0,
            start: 0,
            line: 1,
            column: 0,
            source_len,
        }
    }

    fn is_alpha(ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_' || ch == ':'
    }

    fn is_digit(ch: char) -> bool {
        ch.is_ascii_digit()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source_len
    }

    fn advance(&mut self) {
        if self.current >= self.source_len {
            panic!(
                "Tried to advance past end of source. Source length: {}, current: {}",
                self.source_len, self.current
            )
        }

        self.current += self.peek().len_utf8();
        self.column += 1;
    }

    fn peek(&self) -> char {
        self.source[self.current..]
            .chars()
            .next()
            .unwrap_or_else(|| {
                panic!(
                    "Tried to peek past end of source. Source length: {}, current: {}",
                    self.source_len, self.current
                )
            })
    }

    fn peek_next(&self) -> char {
        self.source[self.current..].chars().nth(1).unwrap_or('\0')
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        Token::new(
            token_type,
            self.start,
            self.current,
            self.line,
            self.column,
            None,
        )
    }

    fn make_error(&self, message: &str) -> Token {
        Token::new(
            TokenType::Error,
            self.start,
            self.current,
            self.line,
            self.column,
            Some(message.to_string()),
        )
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.column = 0;

                    self.advance();
                }
                ';' => {
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                }
                _ => return,
            }
        }
    }

    fn label(&mut self) -> Token {
        let token = self.make_token(TokenType::Label);

        // Consume the ':'.
        self.advance();

        token
    }

    fn identifier(&mut self) -> Token {
        while !self.is_at_end()
            && let char = self.peek()
            && (Self::is_alpha(char) || Self::is_digit(char))
        {
            self.advance();
        }

        let identifier = &self.source[self.start..self.current];

        if identifier.ends_with(':') {
            return self.label();
        }

        match TokenType::try_from(identifier.to_lowercase().as_str()) {
            Ok(token_type) => self.make_token(token_type),
            Err(_) => self.make_token(TokenType::Identifier),
        }
    }

    fn number(&mut self) -> Token {
        while !self.is_at_end()
            && let char = self.peek()
            && Self::is_digit(char)
        {
            self.advance();
        }

        // Look for a fractional part.
        if self.peek() == '.'
            && let next_char = self.peek_next()
            && Self::is_digit(next_char)
        {
            // Consume the decimal point.
            self.advance();

            while !self.is_at_end()
                && let char = self.peek()
                && Self::is_digit(char)
            {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn string(&mut self) -> Token {
        while !self.is_at_end() {
            // If we hit an unescaped quote, string ends.
            if self.peek() == '"' {
                break;
            }

            // Handle escaped double quote: skip the backslash and the quote.
            if self.peek() == '\\' && self.peek_next() == '"' {
                self.advance(); // Consumes the backslash.
                self.advance(); // Consumes the escaped quote.

                continue;
            }

            if self.peek() == '\n' {
                self.line += 1;
                self.column = 0;
            }

            self.advance();
        }

        if self.is_at_end() {
            self.make_error("Unterminated string.")
        } else {
            // Consume the closing quote.
            self.advance();
            self.make_token(TokenType::String)
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();

        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let ch = self.peek();
        self.advance();

        if Self::is_alpha(ch) {
            return self.identifier();
        }

        if Self::is_digit(ch) {
            return self.number();
        }

        match ch {
            // Single-character tokens.
            ',' => self.make_token(TokenType::Comma),
            '"' => self.string(),
            _ => self.make_error("Unexpected character"),
        }
    }
}
