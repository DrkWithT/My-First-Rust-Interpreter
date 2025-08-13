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
    semanator: Analyzer<'cml_1>,
    ir_emitter: IREmitter<'cml_1>,
    bc_emitter: BytecodeEmitter,
    first_source_name: &'cml_1 str,
}

impl<'cml_2> CompilerMain<'cml_2> {
    pub fn new(lexicals: HashMap<String, TokenType>, first_source_name_arg: &'cml_2 str, main_source: &'cml_2 str, native_catalog: &'cml_2 HashMap<&'static str, NativeBrief>) -> Self {
        let temp_lexer = Lexer::new(lexicals, main_source, 0, main_source.len(), 1, 1);
        let temp_parser = Parser::new(temp_lexer);

        Self {
           parser: temp_parser,
           semanator: Analyzer::<'cml_2>::new(main_source),
           ir_emitter: IREmitter::<'cml_2>::new(main_source, native_catalog),
           bc_emitter: BytecodeEmitter::default(),
           first_source_name: first_source_name_arg,
        }
    }

    fn step_parse(&mut self) -> Option<FullProgramAST> {
        let mut source_frontier = VecDeque::<QueuedSource>::new();
        source_frontier.push_back((String::from(self.first_source_name), 0));

        let mut temp_full_ast = FullProgramAST::new();
        let mut finished_srcs = HashSet::<String>::new();

        while !source_frontier.is_empty() {
            let (next_src_name, _) = source_frontier.pop_back().unwrap();

            let temp_tu_src = fs::read_to_string(format!("./loxie_lib/{next_src_name}.loxie"));

            if temp_tu_src.is_err() {
                eprintln!("CompileError: Failed to read file of import '{}'", next_src_name.as_str());
                return None;
            }

            self.parser.reset_with(temp_tu_src.unwrap());
            let (tu_ast_opt, tu_successors) = self.parser.parse_file();

            tu_ast_opt.as_deref()?;

            let tu_ast = tu_ast_opt.unwrap();
            for fun_ast in tu_ast {
                temp_full_ast.push_front(fun_ast);
            }

            finished_srcs.insert(next_src_name);

            for successor_src in tu_successors {
                if !finished_srcs.contains(&successor_src.0) {
                    source_frontier.push_back(successor_src);
                }
            }
        }

        Some(temp_full_ast)
    }

    fn step_sema(&mut self, full_ast: &FullProgramAST) -> bool {
        self.semanator.check_source_unit(full_ast)
    }

    fn step_ir_emit(&mut self, full_ast: &FullProgramAST) -> Option<IRResult> {
        self.ir_emitter.emit_all_ir(full_ast)
    }

    fn step_bc_emit(&mut self, full_ir: &mut IRResult) -> Option<bytecode::Program> {
        let (full_cfg_list, full_const_groups, main_id) = full_ir;

        self.bc_emitter.generate_bytecode(full_cfg_list, full_const_groups, *main_id)
    }

    pub fn compile_from_start(&mut self) -> Option<bytecode::Program> {
        let full_program_ast_opt = self.step_parse();

        full_program_ast_opt.as_ref()?;

        let full_program_ast = full_program_ast_opt.unwrap();

        if !self.step_sema(&full_program_ast) {
            return None;
        }

        let full_program_ir_opt = self.step_ir_emit(&full_program_ast);

        full_program_ir_opt.as_ref()?;

        let mut full_program_ir = full_program_ir_opt.unwrap();

        // disassemble_program(&temp_bc);

        self.step_bc_emit(&mut full_program_ir)
    }
}
