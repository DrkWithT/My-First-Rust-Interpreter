use std::{
    collections::{
        HashMap, HashSet, VecDeque
    },
    fs
};

use crate::{
    codegen::{
        bytecode_emitter::BytecodeEmitter,
        ir_emitter::{IREmitter, IRResult},
    },
    frontend::{
        ast::Stmt, lexer::Lexer, parser::Parser, token::TokenType
    },
    semantics::analyzer::Analyzer, utils::bundle::NativeBrief, vm::bytecode,
};

/// ### NOTE
/// Stores relatively-pathed source names and their TU (translation unit) ID for a soon-to-compile Loxie file.
pub type QueuedSource = (String, i32);

/// ### NOTE
/// Stores the combined declaration ASTs in DFS-like order for all sources reached in compilation.
pub type FullProgramAST = VecDeque<Box<dyn Stmt>>;

pub type SourceIndexedAST = (i32, Box<dyn Stmt>);

pub type FullProgramASTSourced = (VecDeque<SourceIndexedAST>, HashMap<i32, String>);

/**
 ### BRIEF
 This logical entity contains all major stages of the bytecode compiler:
 * Tokenizer & Parser (frontend)
 * Analyzer (semantics)
 * IR, IR passes, and code emitters (codegen)
 * Import logic
 ### TODO's
 * Add compiler debug flags which use the below...
    * Do `print_cfg` invocations if debugging needs it.
    * Do `disassemble_prgm` invocation.
 */
pub struct CompilerMain<'cml_1> {
    parser: Parser,
    semanator: Analyzer,
    ir_emitter: IREmitter<'cml_1>,
    bc_emitter: BytecodeEmitter,
    temp_tu_src: String,
    first_source_name: &'cml_1 str,
}

impl<'cml_2> CompilerMain<'cml_2> {
    pub fn new(lexicals: HashMap<String, TokenType>, first_source_name_arg: &'cml_2 str, main_source: &'cml_2 str, native_catalog: &'cml_2 HashMap<&'static str, NativeBrief>) -> Self {
        let temp_lexer = Lexer::new(lexicals, main_source, 0, main_source.len(), 1, 1);
        let temp_parser = Parser::new(temp_lexer);

        Self {
           parser: temp_parser,
           semanator: Analyzer::new(String::from(main_source)),
           ir_emitter: IREmitter::<'cml_2>::new(main_source, native_catalog),
           bc_emitter: BytecodeEmitter::default(),
           temp_tu_src: String::new(),
           first_source_name: first_source_name_arg,
        }
    }

    fn step_parse(&mut self) -> Option<FullProgramASTSourced> {
        let mut source_frontier = VecDeque::<QueuedSource>::new();
        source_frontier.push_back((String::from(self.first_source_name), 0));

        let mut full_sourced_ast_seq = VecDeque::<SourceIndexedAST>::new();
        let mut recorded_srcs = HashMap::<i32, String>::new();
        let mut finished_srcs = HashSet::<String>::new();

        while !source_frontier.is_empty() {
            let (next_src_name, src_tu_id) = source_frontier.pop_back().unwrap();

            let temp_tu_src_opt = if next_src_name.starts_with("./") {
                fs::read_to_string(next_src_name.clone())
            } else {
                fs::read_to_string(format!("./loxie_lib/{next_src_name}.loxie"))
            };

            if temp_tu_src_opt.is_err() {
                eprintln!("CompileError: Failed to read file of import '{}'", next_src_name.as_str());
                return None;
            }

            self.temp_tu_src = temp_tu_src_opt.unwrap();

            self.parser.reset_with(self.temp_tu_src.clone());
            let (tu_ast_opt, tu_successors) = self.parser.parse_file();

            tu_ast_opt.as_deref()?;

            // NOTE: Why do I reverse each TU's decls? For each TU, push top declarations in a certain order to ensure proper semantic scan ordering:
            // TU Main: | D Main | -- (imports) --> TU 1: | A B C | 
            // --> A, B, C, D, Main
            let mut temp_tu_ast = tu_ast_opt.unwrap();
            temp_tu_ast.reverse();

            for fun_ast in temp_tu_ast {
                full_sourced_ast_seq.push_front(
                    (src_tu_id, fun_ast)
                );
            }

            recorded_srcs.insert(src_tu_id, self.temp_tu_src.clone());
            finished_srcs.insert(next_src_name);

            for successor_src in tu_successors {
                if !finished_srcs.contains(&successor_src.0) {
                    source_frontier.push_back(successor_src);
                }
            }
        }

        Some((full_sourced_ast_seq, recorded_srcs))
    }

    fn step_sema(&mut self, full_ast: &VecDeque<SourceIndexedAST>, srcs_table: &HashMap<i32, String>) -> bool {
        for (temp_ast_src_idx, temp_ast) in full_ast {
            self.semanator.reset_source(srcs_table.get(temp_ast_src_idx).unwrap().clone());
            if !self.semanator.check_fun_ast(temp_ast.as_ref()) {
                return false;
            }
        }

        true
    }

    fn step_ir_emit(&mut self, full_ast: &VecDeque<SourceIndexedAST>) -> Option<IRResult> {
        self.ir_emitter.emit_all_ir(full_ast)
    }

    fn step_bc_emit(&mut self, full_ir: &mut IRResult) -> Option<bytecode::Program> {
        let (full_cfg_list, full_const_groups, main_id) = full_ir;

        self.bc_emitter.generate_bytecode(full_cfg_list, full_const_groups, *main_id)
    }

    pub fn compile_from_start(&mut self) -> Option<bytecode::Program> {
        let full_program_ast_opt = self.step_parse();

        full_program_ast_opt.as_ref()?;

        let (full_asts, full_src_table) = full_program_ast_opt.unwrap();

        if !self.step_sema(&full_asts, &full_src_table) {
            return None;
        }

        let full_program_ir_opt = self.step_ir_emit(&full_asts);

        full_program_ir_opt.as_ref()?;

        let mut full_program_ir = full_program_ir_opt.unwrap();

        // disassemble_program(&temp_bc);

        self.step_bc_emit(&mut full_program_ir)
    }
}
