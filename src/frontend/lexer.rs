use std::{collections::HashMap};

use crate::{frontend::token::*, token_from};

pub mod matchers {
    pub fn check_spaces(c: char) -> bool {
        return c == ' ' || c == '\t' || c == '\r' || c == '\n';
    }

    pub fn check_alpha(c: char) -> bool {
        return (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') || c == '_';
    }

    pub fn check_digit(c: char) -> bool {
        return c >= '0' && c <= '9';
    }

    pub fn check_numeric(c: char) -> bool {
        return check_digit(c) || c == '.';
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

pub struct Lexer<'a> {
    pub items: HashMap<&'a str, TokenType>,
    pub source_view: &'a str,
    pub pos: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize
}

impl Lexer<'_> {
    fn at_end(self: &Self) -> bool {
        self.pos >= self.end
    }

    fn peek_off(self: &Self, offset: usize) -> char {
        let raw_src_pos = self.pos + offset;

        if raw_src_pos >= self.end {
            return '\0';
        }

        return self.source_view
            .chars()
            .nth(raw_src_pos)
            .expect(format!("Could not peek symbol at {}", raw_src_pos)
            .as_str());
    }

    fn update_source_location(self: &mut Self, c: char) -> () {
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

    fn lex_single(self: &mut Self, tag: TokenType) -> Token {
        let temp_start = self.pos;
        let temp_line = self.line;
        let temp_column = self.column;

        self.update_source_location(self.peek_off(0));
        self.pos += 1;

        token_from!(tag, temp_start, 1, temp_line, temp_column)
    }

    fn lex_spaces(self: &mut Self) -> Token {
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

    fn lex_comment(self: &mut Self) -> Token {
        self.update_source_location(self.peek_off(0));
        self.pos += 1;

        let temp_start = self.pos;
        let mut temp_len: usize = 0;
        let temp_line = self.line;
        let temp_column = self.column;

        while !self.at_end() {
            let s = self.peek_off(0);

            self.update_source_location(s);

            if s == '\n' {
                break;
            }

            temp_len += 1;
            self.pos += 1;
        }

        token_from!(TokenType::Comment, temp_start, temp_len, temp_line, temp_column)
    }

    fn lex_word(self: &mut Self) -> Token {
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

        let result_lexeme = result.to_lexeme_str(self.source_view).or(Some("")).expect("Lexer::lex_word panicked while getting lexeme");

        token_from!(
            self.items.get(result_lexeme).or(
                Some(&TokenType::Identifier)
            ).expect("Lexer::lex_word panicked while deducing lexical tag").clone(),
            temp_start,
            temp_len,
            temp_line,
            temp_column
        )
    }

    fn lex_numbers(self: &mut Self) -> Token {
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

    fn lex_operator(self: &mut Self) -> Token {
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

        let result_lexeme = result.to_lexeme_str(self.source_view).or(Some("")).expect("Lexer::lex_operator panicked while getting lexeme");

        token_from!(
            self.items.get(result_lexeme).or(
                Some(&TokenType::Unknown)
            ).expect("Lexer::lex_operator panicked while deducing lexical tag").clone(),
            temp_start,
            temp_len,
            temp_line,
            temp_column
        )
    }

    fn lex_complex(self: &mut Self, c: char) -> Token {
        if matchers::check_spaces(c) {
            return self.lex_spaces();
        } else if matchers::check_alpha(c) {
            return self.lex_word();
        } else if matchers::check_numeric(c) {
            return self.lex_numbers();
        } else if matchers::check_multi(c, ['.', '+', '-', '*', '/', '!', '=', '<', '>']) {
            return self.lex_operator();
        } else {
            return self.lex_single(TokenType::Unknown);
        }
    }

    pub fn next(self: &mut Self) -> Token {
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
            ':' => self.lex_single(TokenType::Colon),
            ',' => self.lex_single(TokenType::Comma),
            '(' => self.lex_single(TokenType::ParenOpen),
            ')' => self.lex_single(TokenType::ParenClose),
            '{' => self.lex_single(TokenType::BraceOpen),
            '}' => self.lex_single(TokenType::BraceClose),
            '[' => self.lex_single(TokenType::BracketOpen),
            ']' => self.lex_single(TokenType::BracketClose),
            _ => self.lex_complex(next_symbol)
        }
    }
}