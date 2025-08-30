use std::{collections::HashMap};

use crate::{frontend::token::*, token_from};

pub mod matchers {
    pub fn check_spaces(c: char) -> bool {
        c.is_ascii_whitespace()
    }

    pub fn check_alpha(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    pub fn check_digit(c: char) -> bool {
        c.is_ascii_digit()
    }

    pub fn check_numeric(c: char) -> bool {
        check_digit(c) || c == '.'
    }

    pub fn check_multi<const N: usize>(c: char, targets: [char; N]) -> bool {
        for temp in targets {
            if c == temp {
                return true;
            }
        }

        false
    }
}

pub struct Lexer<'ll_1> {
    source: &'ll_1 str,
    pos: usize,
    end: usize,
    line: usize,
    column: usize
}

impl<'ll_2> Lexer<'ll_2> {
    pub fn new(source_view: &'ll_2 str) -> Self {
        Self {
            source: source_view,
            pos: 0,
            end: source_view.len(),
            line: 1,
            column: 1,
        }
    }

    pub fn reset_with(&mut self, next_source: &'ll_2 str) {
        self.source = next_source;
        self.pos = 0;
        self.end = self.source.len();
        self.line = 1;
        self.column = 1;
    }

    pub fn view_source(&self) -> &str {
        self.source
    }
    
    fn at_end(&self) -> bool {
        self.pos >= self.end
    }

    fn peek_off(&self, offset: usize) -> char {
        let raw_src_pos = self.pos + offset;

        if raw_src_pos >= self.end {
            return '\0';
        }

        self.source.chars().nth(raw_src_pos).expect("Could not peek symbol at {raw_src_pos}")
    }

    fn update_source_location(&mut self, c: char) {
        match c {
            '\n' => {
                self.line += 1;
                self.column = 1;
            },
            _ => {
                self.column += 1;
            }
        }
    }

    fn lex_single(&mut self, tag: TokenType) -> Token {
        let temp_start = self.pos;
        let temp_line = self.line;
        let temp_column = self.column;

        self.update_source_location(self.peek_off(0));
        self.pos += 1;

        token_from!(tag, temp_start, 1, temp_line, temp_column)
    }

    fn lex_spaces(&mut self) -> Token {
        let temp_start = self.pos;
        let mut temp_len: usize = 0;
        let temp_line = self.line;
        let temp_column = self.column;

        while !self.at_end() {
            let s = self.peek_off(0);

            if !matchers::check_spaces(s) {
                break;
            }

            self.update_source_location(s);
            temp_len += 1;
            self.pos += 1;
        }

        token_from!(TokenType::Spaces, temp_start, temp_len, temp_line, temp_column)
    }

    fn lex_comment(&mut self) -> Token {
        self.update_source_location(self.peek_off(0));
        self.pos += 1;

        let temp_start = self.pos;
        let mut temp_len: usize = 0;
        let temp_line = self.line;
        let temp_column = self.column;

        while !self.at_end() {
            let s = self.peek_off(0);
            
            if s == '\n' {
                break;
            }

            self.update_source_location(s);
            temp_len += 1;
            self.pos += 1;
        }

        token_from!(TokenType::Comment, temp_start, temp_len, temp_line, temp_column)
    }

    fn lex_word(&mut self, items: &'ll_2 HashMap<String, TokenType>) -> Token {
        let temp_start = self.pos;
        let mut temp_len: usize = 0;
        let temp_line = self.line;
        let temp_column = self.column;

        while !self.at_end() {
            let s = self.peek_off(0);

            if !matchers::check_alpha(s) {
                break;
            }

            self.update_source_location(s);
            temp_len += 1;
            self.pos += 1;
        }

        let result: Token = token_from!(TokenType::Unknown, temp_start, temp_len, temp_line, temp_column);
        let source_view = self.source;

        let result_lexeme = result.to_lexeme_str(source_view).or(Some("")).expect("Lexer::lex_word panicked while getting lexeme");

        token_from!(
            *items.get(result_lexeme).or(
                Some(&TokenType::Identifier)
            ).expect("Lexer::lex_word panicked while deducing lexical tag"),
            temp_start,
            temp_len,
            temp_line,
            temp_column
        )
    }

    fn lex_char(&mut self) -> Token {
        // Skip first '\'' symbol here
        self.update_source_location(self.peek_off(0));
        self.pos += 1;

        let temp_start = self.pos;
        let mut temp_len: usize = 0;
        let temp_line = self.line;
        let temp_column = self.column;
        let mut escapes = 0;
        let mut closed = false;

        while !self.at_end() {
            let s = self.peek_off(0);
            self.update_source_location(s);
            self.pos += 1;

            if s == '\'' {
                closed = true;
                break;
            }

            if s == '\\' {
                escapes += 1;
            }

            temp_len += 1;
        }

        let temp_tag = if escapes <= 1 && closed { TokenType::LiteralChar } else { TokenType::Unknown };

        token_from!(temp_tag, temp_start, temp_len, temp_line, temp_column)
    }

    fn lex_numbers(&mut self) -> Token {
        let temp_start = self.pos;
        let mut temp_len: usize = 0;
        let temp_line = self.line;
        let temp_column = self.column;
        let mut dots: i32 = 0;

        while !self.at_end() {
            let s = self.peek_off(0);

            if !matchers::check_numeric(s) {
                break;
            }

            if s == '.' {
                dots += 1;
            }

            self.update_source_location(s);
            temp_len += 1;
            self.pos += 1;
        }

        match dots {
            0 => token_from!(TokenType::LiteralInt, temp_start, temp_len, temp_line, temp_column),
            1 => token_from!(TokenType::LiteralFloat, temp_start, temp_len, temp_line, temp_column),
            _ => token_from!(TokenType::Unknown, temp_start, temp_len, temp_line, temp_column)
        }
    }

    fn lex_string(&mut self) -> Token {
        self.update_source_location(self.peek_off(0));
        self.pos += 1;

        let temp_start = self.pos;
        let mut temp_len: usize = 0;
        let temp_line = self.line;
        let temp_column = self.column;
        let mut closed = false;

        while !self.at_end() {
            let s = self.peek_off(0);

            self.update_source_location(s);
            self.pos += 1;

            if s == '\"' {
                closed = true;
                break;
            }

            temp_len += 1;
        }

        let temp_tag = if closed { TokenType::LiteralVarchar } else { TokenType::Unknown };

        token_from!(temp_tag, temp_start, temp_len, temp_line, temp_column)
    }

    fn lex_operator(&mut self, items: &'ll_2 HashMap<String, TokenType>) -> Token {
        let temp_start = self.pos;
        let mut temp_len: usize = 0;
        let temp_line = self.line;
        let temp_column = self.column;

        while !self.at_end() {
            let s = self.peek_off(0);

            if !matchers::check_multi(s, ['.', '+', '-', '*', '/', '!', '=', '<', '>']) {
                break;
            }

            self.update_source_location(s);
            temp_len += 1;
            self.pos += 1;
        }

        let result: Token = token_from!(TokenType::Unknown, temp_start, temp_len, temp_line, temp_column);
        let source_view = self.source;

        let result_lexeme = result.to_lexeme_str(source_view).or(Some("")).expect("Lexer::lex_operator panicked while getting lexeme");

        token_from!(
            *items.get(result_lexeme).or(
                Some(&TokenType::Unknown)
            ).expect("Lexer::lex_operator panicked while deducing lexical tag"),
            temp_start,
            temp_len,
            temp_line,
            temp_column
        )
    }

    fn lex_complex(&mut self, c: char, items: &'ll_2 HashMap<String, TokenType>) -> Token {
        if matchers::check_spaces(c) {
            self.lex_spaces()
        } else if matchers::check_alpha(c) {
            self.lex_word(items)
        } else if matchers::check_digit(c) {
            self.lex_numbers()
        } else if matchers::check_multi(c, ['.', '+', '-', '*', '/', '!', '=', '<', '>']) {
            self.lex_operator(items)
        } else {
            self.lex_single(TokenType::Unknown)
        }
    }

    pub fn lex_next(&mut self, items: &'ll_2 HashMap<String, TokenType>) -> Token {
        if self.at_end() {
            return token_from!(
                TokenType::Eof,
                self.end,
                1,
                self.line,
                self.pos
            );
        }

        let next_symbol = self.peek_off(0);

        match next_symbol {
            '#' => self.lex_comment(),
            '\'' => self.lex_char(),
            '\"' => self.lex_string(),
            ':' => self.lex_single(TokenType::Colon),
            ',' => self.lex_single(TokenType::Comma),
            ';' => self.lex_single(TokenType::Semicolon),
            '(' => self.lex_single(TokenType::ParenOpen),
            ')' => self.lex_single(TokenType::ParenClose),
            '{' => self.lex_single(TokenType::BraceOpen),
            '}' => self.lex_single(TokenType::BraceClose),
            '[' => self.lex_single(TokenType::BracketOpen),
            ']' => self.lex_single(TokenType::BracketClose),
            _ => self.lex_complex(next_symbol, items)
        }
    }
}