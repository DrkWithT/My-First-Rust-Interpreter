use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

pub mod frontend;
pub mod semantics;
pub mod codegen;
pub mod compiler;
pub mod utils;
pub mod vm;

use crate::compiler::driver::CompilerMain;
use crate::frontend::token::*;
// use crate::codegen::bytecode_printer::disassemble_program;
// use crate::codegen::ir_printer::print_cfg;
use crate::utils::bundle::Bundle;
use crate::utils::{loxie_stdio, loxie_varchar};
use crate::vm::callable::ExecStatus;
use crate::vm::engine::Engine;
use crate::vm::heap::TOTAL_STRING_OVERHEAD;

const LOXIM_VERSION_MAJOR: i32 = 0;
const LOXIM_VERSION_MINOR: i32 = 3;
const LOXIM_VERSION_PATCH: i32 = 0;
const LOXIM_MAX_ARGC: usize = 2;

// The default limit of stack slots for values.
const LOXIM_STACK_LIMIT: i32 = 128;

// The default limit for the VM's heap memory size in estimated bytes.
const LOXIM_HEAP_OVERHEAD_DEFAULT: usize = TOTAL_STRING_OVERHEAD * 128;

fn main() -> ExitCode {
    let mut arg_list = env::args();
    let arg_count: usize = arg_list.len() - 1;

    if arg_count > LOXIM_MAX_ARGC {
        println!("usage: ./loxim [--help | --version | <file-name>]");
        return ExitCode::FAILURE;
    }

    let first_arg_str = arg_list.nth(1).unwrap_or(String::from(""));

    if first_arg_str == "--version" {
        println!(
            "loxim v{LOXIM_VERSION_MAJOR}.{LOXIM_VERSION_MINOR}.{LOXIM_VERSION_PATCH}\nBy: DrkWithT (GitHub)"
        );
        return ExitCode::SUCCESS;
    } else if first_arg_str == "--help" {
        println!("usage: ./loxim [--help | --version | <file-name>]");
        return ExitCode::SUCCESS;
    }

    // Setup 1: Bind native functions to the interpreter's global scope.
    let mut global_natives = Bundle::new();

    global_natives.register_native("intrin_varchar_len", Box::new(loxie_varchar::native_intrin_varchar_len), 1);
    global_natives.register_native("intrin_varchar_get", Box::new(loxie_varchar::native_intrin_varchar_get), 2);
    global_natives.register_native("intrin_varchar_set", Box::new(loxie_varchar::native_intrin_varchar_set), 3);
    global_natives.register_native("intrin_varchar_push", Box::new(loxie_varchar::native_intrin_varchar_push), 2);
    global_natives.register_native("intrin_varchar_pop", Box::new(loxie_varchar::native_intrin_varchar_pop), 1);
    global_natives.register_native("read_int", Box::new(loxie_stdio::native_read_int), 0);
    global_natives.register_native("print_val", Box::new(loxie_stdio::native_print_val), 1);

    let first_arg_copy_str = first_arg_str.clone();
    let first_arg_str_view = first_arg_copy_str.as_str();
    let source_path = Path::new(first_arg_str_view);

    if !source_path.exists() {
        println!("Path not found: '{}'", source_path.to_str().expect(""));
        return ExitCode::FAILURE;
    }

    let source_text_opt = fs::read_to_string(source_path);

    if source_text_opt.is_err() {
        println!("Failed to read file.");
        return ExitCode::FAILURE;
    }

    // Setup 2: Register important lexical items to the lexer for parsing later.
    let source_text = source_text_opt.expect("Failed to unbox source string?");

    let mut lexical_items = HashMap::<String, TokenType>::new();
    lexical_items.insert(String::from("foreign"), TokenType::Keyword);
    lexical_items.insert(String::from("fun"), TokenType::Keyword);
    lexical_items.insert(String::from("ctor"), TokenType::Keyword);
    lexical_items.insert(String::from("class"), TokenType::Keyword);
    lexical_items.insert(String::from("met"), TokenType::Keyword);
    lexical_items.insert(String::from("private"), TokenType::Keyword);
    lexical_items.insert(String::from("public"), TokenType::Keyword);
    lexical_items.insert(String::from("let"), TokenType::Keyword);
    lexical_items.insert(String::from("if"), TokenType::Keyword);
    lexical_items.insert(String::from("else"), TokenType::Keyword);
    lexical_items.insert(String::from("while"), TokenType::Keyword);
    lexical_items.insert(String::from("return"), TokenType::Keyword);
    lexical_items.insert(String::from("exit"), TokenType::Keyword);
    lexical_items.insert(String::from("bool"), TokenType::Typename);
    lexical_items.insert(String::from("char"), TokenType::Typename);
    lexical_items.insert(String::from("int"), TokenType::Typename);
    lexical_items.insert(String::from("float"), TokenType::Typename);
    lexical_items.insert(String::from("varchar"), TokenType::Typename);
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

    let mut loxie_compiler = CompilerMain::new(first_arg_str_view, source_text.as_str(), global_natives.peek_registry());

    let program_opt = loxie_compiler.compile_from_start(lexical_items);

    if program_opt.is_none() {
        eprintln!("Compilation failed, see errors above.");
        return ExitCode::FAILURE;
    }

    let program = program_opt.unwrap();

    let mut engine = Engine::new(program, LOXIM_HEAP_OVERHEAD_DEFAULT, LOXIM_STACK_LIMIT);

    let pre_run_time = Instant::now();
    let engine_status = engine.run(&global_natives);
    let running_time = Instant::now() - pre_run_time;

    println!(
        "\x1b[1;33mFinished in {} microseconds\x1b[0m",
        running_time.as_micros()
    );

    match engine_status {
        ExecStatus::Ok => {
            println!("\x1b[1;32mOK\x1b[0m");
            ExitCode::SUCCESS
        },
        ExecStatus::AccessError => {
            eprintln!("\x1b[1;31mRunError: AccessError of stack operation.\x1b[0m");
            ExitCode::FAILURE
        },
        ExecStatus::ValueError => {
            eprintln!("\x1b[1;31mRunError: Invalid Value materialized.\x1b[0m");
            ExitCode::FAILURE
        },
        ExecStatus::RefError => {
            eprintln!("\x1b[1;31mRefError: Invalid (empty) heap reference materialized.\x1b[0m");
            ExitCode::FAILURE
        },
        ExecStatus::BadMath => {
            eprintln!("\x1b[1;31mRunError: Division by zero.\x1b[0m");
            ExitCode::FAILURE
        },
        ExecStatus::IllegalInstruction => {
            eprintln!("\x1b[1;31mRunError: Illegal instruction fetched.\x1b[0m");
            ExitCode::FAILURE
        },
        ExecStatus::BadArgs => {
            eprintln!("\x1b[1;31mRunError: Invalid argument passed to opcode.\x1b[0m");
            ExitCode::FAILURE
        },
        ExecStatus::NotOk => {
            eprintln!("\x1b[1;31mRunError: Exited with non-zero status.\x1b[0m");
            ExitCode::FAILURE
        },
    }
}
