use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

pub mod codegen;
pub mod frontend;
pub mod semantics;
pub mod vm;

use crate::codegen::bytecode_emitter::BytecodeEmitter;
// use crate::codegen::bytecode_printer::disassemble_program;
use crate::codegen::ir_emitter::IREmitter;
// use crate::codegen::ir_printer::print_cfg;
use crate::frontend::parser::*;
use crate::frontend::token::*;
use crate::vm::callable::ExecStatus;
use crate::vm::engine::Engine;

const MAX_ARG_COUNT: usize = 2;
const CONCH_VERSION_MAJOR: i32 = 0;
const CONCH_VERSION_MINOR: i32 = 1;
const CONCH_VERSION_PATCH: i32 = 0;
const CONCH_VALUE_STACK_LIMIT: i32 = 32767;

fn main() -> ExitCode {
    let mut arg_list = env::args();
    let arg_count: usize = arg_list.len() - 1;

    if arg_count > MAX_ARG_COUNT {
        println!("usage: ./conchvm [--help | --version | <file-name>]");
        return ExitCode::FAILURE;
    }

    let first_arg_str = arg_list.nth(1).unwrap_or(String::from(""));

    if first_arg_str == "--version" {
        println!(
            "conchvm v{CONCH_VERSION_MAJOR}.{CONCH_VERSION_MINOR}.{CONCH_VERSION_PATCH}\nBy: DrkWithT (GitHub)"
        );
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
    lexical_items.insert(String::from("while"), TokenType::Keyword);
    lexical_items.insert(String::from("return"), TokenType::Keyword);
    lexical_items.insert(String::from("exit"), TokenType::Keyword);
    lexical_items.insert(String::from("bool"), TokenType::Typename);
    lexical_items.insert(String::from("int"), TokenType::Typename);
    lexical_items.insert(String::from("float"), TokenType::Typename);
    lexical_items.insert(String::from("true"), TokenType::LiteralBool);
    lexical_items.insert(String::from("false"), TokenType::LiteralBool);
    lexical_items.insert(String::from("."), TokenType::OpAccess);
    lexical_items.insert(String::from("*"), TokenType::OpTimes);
    lexical_items.insert(String::from("/"), TokenType::OpSlash);
    lexical_items.insert(String::from("+"), TokenType::OpPlus);
    lexical_items.insert(String::from("-"), TokenType::OpMinus);
    lexical_items.insert(String::from("=="), TokenType::OpEquality);
    lexical_items.insert(String::from("!="), TokenType::OpInequality);
    lexical_items.insert(String::from("<"), TokenType::OpLessThan);
    lexical_items.insert(String::from(">"), TokenType::OpGreaterThan);
    lexical_items.insert(String::from("="), TokenType::OpAssign);

    let tokenizer =
        frontend::lexer::Lexer::new(lexical_items, &source_text, 0, source_length, 1, 1);
    let mut parser = Parser::new(tokenizer);

    let ast_opt = parser.parse_file();

    if ast_opt.is_none() {
        println!("Parsing failed, please see all errors above.");
        return ExitCode::FAILURE;
    }

    let ast_decls = ast_opt.unwrap();

    println!("function declaration count: {}", ast_decls.len());

    let mut ir_emitter = IREmitter::new(source_text.as_str());
    let ir_opt = ir_emitter.emit_bytecode_from(&ast_decls);

    if ir_opt.is_none() {
        return ExitCode::FAILURE;
    }

    let (cfg_list, mut constant_groups_list, main_id) = ir_opt.unwrap();

    // TODO: add printing for Value constant region.
    // for graph in &cfg_list {
    //     print_cfg(graph);
    // }

    let mut bc_emitter = BytecodeEmitter::default();

    let program_opt = bc_emitter.generate_bytecode(&cfg_list, &mut constant_groups_list, main_id);

    if program_opt.is_none() {
        eprintln!("Error: Failed to compile program.");
        return ExitCode::FAILURE;
    }

    let program = program_opt.unwrap();

    // disassemble_program(&program);

    let mut engine = Engine::new(program, CONCH_VALUE_STACK_LIMIT);

    let pre_run_time = Instant::now();
    let engine_status = engine.run();
    let running_time = Instant::now() - pre_run_time;

    println!(
        "\x1b[1;33mFinished in {} microseconds\x1b[0m",
        running_time.as_micros()
    );

    match engine_status {
        ExecStatus::Ok => {
            println!("\x1b[1;32mOK\x1b[0m");
            ExitCode::SUCCESS
        }
        ExecStatus::AccessError => {
            eprintln!("\x1b[1;31mRunError: AccessError of stack operation.\x1b[0m");
            ExitCode::FAILURE
        }
        ExecStatus::ValueError => {
            eprintln!("\x1b[1;31mRunError: Invalid Value materialized.\x1b[0m");
            ExitCode::FAILURE
        }
        ExecStatus::BadMath => {
            eprintln!("\x1b[1;31mRunError: Division by zero.\x1b[0m");
            ExitCode::FAILURE
        }
        ExecStatus::IllegalInstruction => {
            eprintln!("\x1b[1;31mRunError: Illegal instruction fetched.\x1b[0m");
            ExitCode::FAILURE
        }
        ExecStatus::BadArgs => {
            eprintln!("\x1b[1;31mRunError: Invalid argument passed to opcode.\x1b[0m");
            ExitCode::FAILURE
        }
        ExecStatus::NotOk => {
            eprintln!("\x1b[1;31mRunError: Exited with non-zero status.\x1b[0m");
            ExitCode::FAILURE
        }
    }
    // ExitCode::SUCCESS
}
