use std::env;
use std::process::ExitCode;
use std::collections::HashMap;
use std::fs;

use crate::frontend::token::TokenType;

pub mod frontend;

const MIN_ARG_COUNT: usize = 1;
const MAX_ARG_COUNT: usize = 2;

const CONCH_VERSION_MAJOR: i32 = 0;
const CONCH_VERSION_MINOR: i32 = 1;
const CONCH_VERSION_PATCH: i32 = 0;

/// TODO: I need a fucking constructor or macro AHHHHH
fn main() -> ExitCode {
    let mut arg_list= env::args();
    let arg_count: usize = arg_list.len();

    if arg_count < MIN_ARG_COUNT || arg_count > MAX_ARG_COUNT {
        println!("usage: ./conchvm [--help | --version | <file-name>]");

        return ExitCode::FAILURE;
    }

    let first_arg = arg_list.nth(0);

    if first_arg == Some("--version".into()) {
        println!("conchvm v.{}.{}.{}\nBy: DrkWithT (GitHub)", CONCH_VERSION_MAJOR, CONCH_VERSION_MINOR, CONCH_VERSION_PATCH);

        return ExitCode::SUCCESS;
    } else if first_arg == Some("--help".into()) {
        println!("usage: ./conchvm [--help | --version | <file-name>]");

        return ExitCode::SUCCESS;
    }

    // todo: implement lexer and then use it here...
    let source_text_opt = fs::read_to_string(first_arg.expect("Possibly missing argument #1 - expected path relative to launch path."));

    if source_text_opt.is_err() {
        println!("Failed to read file.");

        return ExitCode::FAILURE;
    }

    let source_text = source_text_opt.expect("Failed to unbox source string?");
    let source_view = source_text.as_str();

    let mut lexical_items = HashMap::<&str, TokenType>::new();

    lexical_items.insert("fun", TokenType::Keyword);
    lexical_items.insert("let", TokenType::Keyword);
    lexical_items.insert("if", TokenType::Keyword);
    lexical_items.insert("else", TokenType::Keyword);
    lexical_items.insert("return", TokenType::Keyword);
    lexical_items.insert("exit", TokenType::Keyword);
    lexical_items.insert("bool", TokenType::Typename);
    lexical_items.insert("int", TokenType::Typename);
    lexical_items.insert("float", TokenType::Typename);
    lexical_items.insert(".", TokenType::OpAccess);
    lexical_items.insert("++", TokenType::OpIncrement);
    lexical_items.insert("--", TokenType::OpDecrement);
    lexical_items.insert("*", TokenType::OpTimes);
    lexical_items.insert("/", TokenType::OpSlash);
    lexical_items.insert("+", TokenType::OpPlus);
    lexical_items.insert("-", TokenType::OpMinus);
    lexical_items.insert("==", TokenType::OpEquality);
    lexical_items.insert("!=", TokenType::OpInequality);
    lexical_items.insert("<", TokenType::OpLessThan);
    lexical_items.insert(">", TokenType::OpGreaterThan);
    lexical_items.insert("=", TokenType::OpAssign);

    let mut tokenizer = frontend::lexer::Lexer {
        items: lexical_items,
        source_view: source_view,
        pos: 0,
        end: source_view.len(),
        line: 1,
        column: 1
    };

    loop {
        let temp_token = tokenizer.next();
        let token_info_msg = temp_token.to_info_str();

        let token_tag: TokenType = temp_token.tag.into();

        if token_tag == TokenType::Unknown.into() {
            println!("BAD TOKEN LUL:\n{}", token_info_msg);

            return ExitCode::FAILURE;
        }

        if token_tag == TokenType::Eof.into() {
            break;
        }
    }

    ExitCode::SUCCESS
}
