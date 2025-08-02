use std::env;
use std::path::Path;
use std::process::ExitCode;
use std::collections::HashMap;
use std::fs;

pub mod frontend;
pub mod semantics;

use crate::frontend::token::*;
use crate::frontend::parser::*;

const MIN_ARG_COUNT: usize = 1;
const MAX_ARG_COUNT: usize = 2;
const CONCH_VERSION_MAJOR: i32 = 0;
const CONCH_VERSION_MINOR: i32 = 1;
const CONCH_VERSION_PATCH: i32 = 0;

fn main() -> ExitCode {
    let mut arg_list= env::args();
    let arg_count: usize = arg_list.len() - 1;

    if arg_count < MIN_ARG_COUNT || arg_count > MAX_ARG_COUNT {
        println!("usage: ./conchvm [--help | --version | <file-name>]");
        return ExitCode::FAILURE;
    }

    let first_arg_str = arg_list.nth(1).unwrap_or(String::from(""));

    if first_arg_str == "--version" {
        println!("conchvm v.{}.{}.{}\nBy: DrkWithT (GitHub)", CONCH_VERSION_MAJOR, CONCH_VERSION_MINOR, CONCH_VERSION_PATCH);
        return ExitCode::SUCCESS;
    } else if first_arg_str == "--help" {
        println!("usage: ./conchvm [--help | --version | <file-name>]");
        return ExitCode::SUCCESS;
    }

    let first_arg_copy_str = first_arg_str.clone();
    let source_path = Path::new(first_arg_copy_str.as_str());

    if !source_path.exists() {
        println!("Path not found: '{}'", source_path.to_str().expect(""));
        return ExitCode::FAILURE;
    }

    // todo: implement lexer and then use it here...
    let source_text_opt = fs::read_to_string(source_path);

    if source_text_opt.is_err() {
        println!("Failed to read file.");
        return ExitCode::FAILURE;
    }

    let source_text = source_text_opt.expect("Failed to unbox source string?");
    let source_length = source_text.len();

    let mut lexical_items = HashMap::<String, TokenType>::new();

    lexical_items.insert(String::from("fun"), TokenType::Keyword);
    lexical_items.insert(String::from("let"), TokenType::Keyword);
    lexical_items.insert(String::from("if"), TokenType::Keyword);
    lexical_items.insert(String::from("else"), TokenType::Keyword);
    lexical_items.insert(String::from("return"), TokenType::Keyword);
    lexical_items.insert(String::from("exit"), TokenType::Keyword);
    lexical_items.insert(String::from("bool"),TokenType::Typename);
    lexical_items.insert(String::from("int"), TokenType::Typename);
    lexical_items.insert(String::from("float"), TokenType::Typename);
    lexical_items.insert(String::from("true"), TokenType::LiteralBool);
    lexical_items.insert(String::from("false"), TokenType::LiteralBool);
    lexical_items.insert(String::from("."), TokenType::OpAccess);
    lexical_items.insert(String::from("++"), TokenType::OpIncrement);
    lexical_items.insert(String::from("--"), TokenType::OpDecrement);
    lexical_items.insert(String::from("*"), TokenType::OpTimes);
    lexical_items.insert(String::from("/"), TokenType::OpSlash);
    lexical_items.insert(String::from("+"), TokenType::OpPlus);
    lexical_items.insert(String::from("-"), TokenType::OpMinus);
    lexical_items.insert(String::from("=="), TokenType::OpEquality);
    lexical_items.insert(String::from("!="), TokenType::OpInequality);
    lexical_items.insert(String::from("<"), TokenType::OpLessThan);
    lexical_items.insert(String::from(">"), TokenType::OpGreaterThan);
    lexical_items.insert(String::from("="), TokenType::OpAssign);

    let tokenizer = frontend::lexer::Lexer::new(lexical_items, &source_text, 0, source_length, 1, 1);
    let mut parser = Parser::new(tokenizer);

    let ast_opt = parser.parse_file();

    if ast_opt.is_none() {
        println!("Parsing failed, please see all errors above.");
        return ExitCode::FAILURE;
    }

    let ast_decls = ast_opt.unwrap();

    println!("function declaration count: {}", ast_decls.len());

    ExitCode::SUCCESS
}
