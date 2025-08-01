#[derive(Clone, PartialEq)]
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
    pub fn eq(self: &Self, rhs: &Self) -> bool {
        self == rhs
    }

    pub fn name(self: &Self) -> String {
        match self {
            Self::Unknown => "Unknown".to_string(),
            Self::Spaces => "Spaces".to_string(),
            Self::Comment => "Comment".to_string(),
            Self::Keyword => "Keyword".to_string(),
            Self::Typename => "Typename".to_string(),
            Self::Identifier => "Identifier".to_string(),
            Self::LiteralBool => "LiteralBool".to_string(),
            Self::LiteralInt => "LiteralInt".to_string(),
            Self::LiteralFloat => "LiteralFloat".to_string(),
            // LiteralString,
            Self::OpAccess => "OpAccess".to_string(),
            Self::OpIncrement => "OpIncrement".to_string(),
            Self::OpDecrement => "OpDecrement".to_string(),
            Self::OpTimes => "OpTimes".to_string(),
            Self::OpSlash => "OpSlash".to_string(),
            Self::OpPlus => "OpPlus".to_string(),
            Self::OpMinus => "OpMinus".to_string(),
            Self::OpEquality => "OpEquality".to_string(),
            Self::OpInequality => "OpInequality".to_string(),
            Self::OpLessThan => "OpLessThan".to_string(),
            Self::OpGreaterThan => "OpGreaterThan".to_string(),
            Self::OpAssign => "OpAssign".to_string(),
            Self::Colon => "Colon".to_string(),
            Self::Comma => "Comma".to_string(),
            Self::ParenOpen => "ParenOpen".to_string(),
            Self::ParenClose => "ParenClose".to_string(),
            Self::BraceOpen => "BraceOpen".to_string(),
            Self::BraceClose => "BraceClose".to_string(),
            Self::BracketOpen => "BracketOpen".to_string(),
            Self::BracketClose => "BracketClose".to_string(),
            Self::Eof => "Eof".to_string()
        }
    }
}

pub struct Token {
    pub tag: TokenType,
    pub start: usize,
    pub length: usize,
    pub line_no: usize,
    pub col_no: usize
}

impl Token {
    pub fn to_lexeme_str(self: Self, source: &str) -> Option<&str> {
        let lexeme_start = self.start;
        let lexeme_len = self.length;

        source.get(lexeme_start..(lexeme_start + lexeme_len))
    }

    pub fn to_info_str(self: &Self) -> String {
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
