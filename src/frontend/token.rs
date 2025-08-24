#[repr(i32)]
#[derive(Clone, PartialEq)]
pub enum TokenType {
    Unknown,
    Spaces,
    Comment,
    Keyword,
    Typename,
    ClassSelf,
    Identifier,
    LiteralBool,
    LiteralChar,
    LiteralInt,
    LiteralFloat,
    LiteralVarchar,
    OpAccess,
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
    Semicolon,
    ParenOpen,
    ParenClose,
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    Eof,
}

impl TokenType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Spaces => "Spaces",
            Self::Comment => "Comment",
            Self::Keyword => "Keyword",
            Self::ClassSelf => "ClassSelf",
            Self::Typename => "Typename",
            Self::Identifier => "Identifier",
            Self::LiteralBool => "LiteralBool",
            Self::LiteralChar => "LiteralChar",
            Self::LiteralInt => "LiteralInt",
            Self::LiteralFloat => "LiteralFloat",
            Self::LiteralVarchar => "LiteralVarchar",
            Self::OpAccess => "OpAccess",
            Self::OpTimes => "OpTimes",
            Self::OpSlash => "OpSlash",
            Self::OpPlus => "OpPlus",
            Self::OpMinus => "OpMinus",
            Self::OpEquality => "OpEquality",
            Self::OpInequality => "OpInequality",
            Self::OpLessThan => "OpLessThan",
            Self::OpGreaterThan => "OpGreaterThan",
            Self::OpAssign => "OpAssign",
            Self::Colon => "Colon",
            Self::Comma => "Comma",
            Self::Semicolon => "Semicolon",
            Self::ParenOpen => "ParenOpen",
            Self::ParenClose => "ParenClose",
            Self::BraceOpen => "BraceOpen",
            Self::BraceClose => "BraceClose",
            Self::BracketOpen => "BracketOpen",
            Self::BracketClose => "BracketClose",
            Self::Eof => "Eof",
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
    pub col_no: usize,
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
            col_no: $col,
        }
    };
}
