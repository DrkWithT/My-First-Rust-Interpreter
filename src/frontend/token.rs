#[repr(i32)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum TokenType {
    Unknown,
    Spaces,
    Comment,
    Keyword,
    Typename,
    Identifier,
    LiteralBool,
    LiteralInt,
    LiteralFloat,
    // LiteralString,
    OpAccess,
    OpIncrement,
    OpDecrement,
    OpTimes,
    OpSlash,
    OpPlus,
    OpMinus,
    OpEquality,
    OpInequality,
    OpLessThan,
    OpGreaterThan,
    OpAssign,
    Colon,
    Comma,
    ParenOpen,
    ParenClose,
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    Eof
}

impl TokenType {
    pub fn name(&self) -> String {
        match self {
            Self::Unknown => String::from("Unknown"),
            Self::Spaces => String::from("Spaces"),
            Self::Comment => String::from("Comment"),
            Self::Keyword => String::from("Keyword"),
            Self::Typename => String::from("Typename"),
            Self::Identifier => String::from("Identifier"),
            Self::LiteralBool => String::from("LiteralBool"),
            Self::LiteralInt => String::from("LiteralInt"),
            Self::LiteralFloat => String::from("LiteralFloat"),
            // LiteralString,
            Self::OpAccess => String::from("OpAccess"),
            Self::OpIncrement => String::from("OpIncrement"),
            Self::OpDecrement => String::from("OpDecrement"),
            Self::OpTimes => String::from("OpTimes"),
            Self::OpSlash => String::from("OpSlash"),
            Self::OpPlus => String::from("OpPlus"),
            Self::OpMinus => String::from("OpMinus"),
            Self::OpEquality => String::from("OpEquality"),
            Self::OpInequality => String::from("OpInequality"),
            Self::OpLessThan => String::from("OpLessThan"),
            Self::OpGreaterThan => String::from("OpGreaterThan"),
            Self::OpAssign => String::from("OpAssign"),
            Self::Colon => String::from("Colon"),
            Self::Comma => String::from("Comma"),
            Self::ParenOpen => String::from("ParenOpen"),
            Self::ParenClose => String::from("ParenClose"),
            Self::BraceOpen => String::from("BraceOpen"),
            Self::BraceClose => String::from("BraceClose"),
            Self::BracketOpen => String::from("BracketOpen"),
            Self::BracketClose => String::from("BracketClose"),
            Self::Eof => String::from("Eof")
        }
    }
}

impl Copy for TokenType {
    // NOTE: this impl is a required dud to make the int32-based enum copyable.
}

#[derive(Clone)]
pub struct Token {
    pub tag: TokenType,
    pub start: usize,
    pub length: usize,
    pub line_no: usize,
    pub col_no: usize
}

impl Copy for Token {
    // NOTE: This is a dud for implementing copy operations for Token.
}

impl Token {
    pub fn to_lexeme_str(self, source: &str) -> Option<&str> {
        let lexeme_start = self.start;
        let lexeme_len = self.length;

        source.get(lexeme_start..(lexeme_start + lexeme_len))
    }

    pub fn to_info_str(&self) -> String {
        format!(
            "Token ({}, {}, {}, {}, {})",
            self.tag.name(),
            self.start,
            self.length,
            self.line_no,
            self.col_no
        )
    }
}

#[macro_export]
macro_rules! token_from {
    ($tag: expr, $begin: expr, $length: expr, $line: expr, $col: expr) => {
        Token {
            tag: $tag,
            start: $begin,
            length: $length,
            line_no: $line,
            col_no: $col
        }
    }
}
