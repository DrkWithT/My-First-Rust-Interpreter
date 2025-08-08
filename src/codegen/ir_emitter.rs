use std::collections::HashMap;

use crate::codegen::ir::*;
use crate::frontend::ast::*;
use crate::frontend::parser::ASTDecls;
use crate::frontend::token::*;
use crate::semantics::types::OperatorTag;
use crate::vm::value::Value;

type IRLinkPair = (i32, i32);
pub type IRResult = (CFGStorage, Vec<Vec<Value>>, i32);

/// NOTE: add logic to find main function ID during visitation!
pub struct IREmitter {
    fun_locals: HashMap<String, Locator>,
    fun_locations: HashMap<String, Locator>,
    result: CFGStorage,

    /// NOTE contains a Vec of corresponding constant Values per bytecode Chunk.
    proto_constants: Vec<Vec<Value>>,

    /// NOTE contains `(from: i32, to: i32)` tuples to process after CFG node generation... Each "proto" link is applied to a CFG node by `from` before exhaustion which clears this Vec- The function's CFG is done by then!
    proto_links: Vec<IRLinkPair>,

    source_copy: String,

    /// NOTE tracks how many Values remain around the top stack slots for the current call frame.
    relative_stack_offset: i32,

    main_id: i32,
    skip_emit: bool,
    has_error: bool,
}

impl IREmitter {
    pub fn new(old_src: &str) -> Self {
        Self {
            fun_locals: HashMap::new(),
            fun_locations: HashMap::new(),
            result: CFGStorage::new(),
            proto_constants: Vec::<Vec<Value>>::new(),
            proto_links: Vec::<IRLinkPair>::new(),
            source_copy: String::from(old_src),
            relative_stack_offset: -1,
            main_id: -1,
            skip_emit: false,
            has_error: false,
        }
    }

    fn record_fun_by_name(&mut self, name: String) -> bool {
        if self.fun_locations.contains_key(name.as_str()) {
            return false;
        }

        let next_fun_id = self.fun_locations.len();

        if self.main_id == -1 && name.as_str() == "main" {
            self.main_id = next_fun_id as i32;
        }

        self.fun_locations
            .insert(name, (Region::Functions, next_fun_id as i32));

        true
    }

    fn enter_fun_scope(&mut self) {
        self.result.push(CFG::new());
        self.proto_constants.push(Vec::new());
        self.reset_relative_offset();
    }

    fn leave_fun_scope(&mut self) {
        self.fun_locals.clear();
    }

    fn record_name_locator(&mut self, name: String, item: Locator) {
        self.fun_locals.insert(name, item);
    }

    fn record_proto_constant(&mut self, item: Value) -> Locator {
        let mut proto_id = 0;

        #[allow(clippy::explicit_counter_loop)]
        for existing_item in self.proto_constants.last().unwrap() {
            if existing_item.is_equal(&item) {
                return (Region::Immediate, proto_id);
            }

            proto_id += 1;
        }

        let next_constant_id = self.proto_constants.last().as_ref().unwrap().len();

        self.proto_constants.last_mut().as_mut().unwrap().push(item);

        let next_constant_id_i32 = next_constant_id as i32;

        (Region::Immediate, next_constant_id_i32)
    }

    fn lookup_locator_of(&self, name: &str) -> Option<Locator> {
        if self.fun_locals.contains_key(name) {
            return Some(self.fun_locals.get(name).unwrap().clone());
        } else if self.fun_locations.contains_key(name) {
            return Some(self.fun_locations.get(name).unwrap().clone());
        }

        None
    }

    fn record_proto_link(&mut self, from_id: i32, to_id: i32) {
        self.proto_links.push((from_id, to_id));
    }

    fn apply_proto_links(&mut self) {
        for (pair_from, pair_to) in &self.proto_links {
            self.result
                .last_mut()
                .unwrap()
                .connect_nodes_by_id(*pair_from, *pair_to);
        }

        self.proto_links.clear();
    }

    fn get_relative_offset(&self) -> i32 {
        self.relative_stack_offset
    }

    fn reset_relative_offset(&mut self) {
        self.relative_stack_offset = -1;
    }

    fn update_relative_offset(&mut self, count: i32) {
        self.relative_stack_offset += count;
    }

    fn emit_step(&mut self, step: Instruction) {
        self.result.last_mut().unwrap().add_instruction_recent(step);
    }

    pub fn emit_bytecode_from(&mut self, ast_tops: &ASTDecls) -> Option<IRResult> {
        for temp in ast_tops {
            if !temp.accept_visitor(self) {
                eprintln!("Oops: failed to generate function from declaration");
                return None;
            }
        }

        let saved_main_id = self.main_id;

        Some((
            std::mem::take(&mut self.result),
            std::mem::take(&mut self.proto_constants),
            saved_main_id,
        ))
    }
}

impl ExprVisitor<Option<Locator>> for IREmitter {
    fn visit_primitive(&mut self, e: &Primitive) -> Option<Locator> {
        let literal_token_ref = e.get_token();
        let literal_lexeme = literal_token_ref.to_lexeme_str(&self.source_copy).unwrap();
        let literal_token_tag = literal_token_ref.tag;

        match literal_token_tag {
            TokenType::LiteralBool => {
                let temp_flag = literal_lexeme == "true";
                let temp_flag_locator = self.record_proto_constant(Value::Bool(temp_flag));

                if !self.skip_emit {
                    self.emit_step(Instruction::Unary(
                        Opcode::LoadConst,
                        temp_flag_locator.clone(),
                    ));
                    self.update_relative_offset(1);
                }

                Some(temp_flag_locator)
            }
            TokenType::LiteralInt => {
                let temp_int: i32 = literal_lexeme.parse().unwrap();
                let temp_int_locator = self.record_proto_constant(Value::Int(temp_int));

                if !self.skip_emit {
                    self.emit_step(Instruction::Unary(
                        Opcode::LoadConst,
                        temp_int_locator.clone(),
                    ));
                    self.update_relative_offset(1);
                }

                Some(temp_int_locator)
            }
            TokenType::LiteralFloat => {
                let temp_float: f32 = literal_lexeme.parse().unwrap();
                let temp_float_locator = self.record_proto_constant(Value::Float(temp_float));

                if !self.skip_emit {
                    self.emit_step(Instruction::Unary(
                        Opcode::LoadConst,
                        temp_float_locator.clone(),
                    ));
                    self.update_relative_offset(1);
                }

                Some(temp_float_locator)
            }
            TokenType::Identifier => {
                let named_locator_opt = self.lookup_locator_of(literal_lexeme);

                named_locator_opt.as_ref()?;

                let named_locator = named_locator_opt.unwrap().clone();

                // NOTE: avoid emitting function values for now, see `visit_call()`!
                if !self.skip_emit {
                    match named_locator.0 {
                        Region::Immediate | Region::TempStack | Region::ArgStore => {
                            self.emit_step(Instruction::Unary(Opcode::Push, named_locator.clone()));
                            self.update_relative_offset(1);
                        }
                        _ => {}
                    }
                }

                Some(named_locator)
            }
            _ => None,
        }
    }

    /// NOTE: Here, the visitation of the callee part of the call-expr results in an extra function PUSH- Removal of the PUSH is needed for correctness, as the engine doesn't support 1st-class functions yet... See `visit_primitive()` where the function name is checked!
    fn visit_call(&mut self, e: &Call) -> Option<Locator> {
        let callee_locator_opt = e.get_callee().accept_visitor(self);

        callee_locator_opt.as_ref()?;

        let callee_locator = callee_locator_opt.unwrap();
        let calling_args = e.get_args();
        let mut calling_arg_count = 0;

        // NOTE: all args are temporary values and the consuming function call will automatically pop them all...
        for arg_ref in calling_args {
            arg_ref.accept_visitor(self)?;

            self.update_relative_offset(1);
            calling_arg_count += 1;
        }

        self.emit_step(Instruction::Binary(
            Opcode::Call,
            callee_locator,
            (Region::Immediate, calling_arg_count),
        ));
        self.update_relative_offset(2 - calling_arg_count);

        Some((Region::TempStack, self.get_relative_offset() + 1))
    }

    // fn visit_array(&self) -> Locator {}
    // fn visit_lambda(&self) -> Locator {}

    fn visit_unary(&mut self, e: &Unary) -> Option<Locator> {
        let expr_op = e.get_operator();
        let result_locator_opt = e.accept_visitor(self);

        result_locator_opt.as_ref()?;

        let result_locator = result_locator_opt.unwrap();
        let expr_opcode = match expr_op {
            OperatorTag::Minus => Opcode::Neg,
            _ => Opcode::Nop,
        };

        if expr_opcode == Opcode::Nop {
            return None;
        }

        self.emit_step(Instruction::Unary(expr_opcode, result_locator.clone()));

        Some(result_locator)
    }

    /// FIXME: this function fails to consider cases of `a = a + 1`: either a wrong push occurs before the replace, or the LHS gets pushed at the wrong time. Fix when to emit operands of binary... make helpers??
    fn visit_binary(&mut self, e: &Binary) -> Option<Locator> {
        let expr_opcode = ast_op_to_ir_op(e.get_operator());
        let found_assign = expr_opcode == Opcode::Replace;
        let lhs_arity = ast_op_to_ir_op(e.get_lhs().get_operator()).arity();
        let rhs_arity = ast_op_to_ir_op(e.get_rhs().get_operator()).arity();

        #[allow(unused_assignments)]
        let mut lhs_locator = (Region::TempStack, -1);
        #[allow(unused_assignments)]
        let mut rhs_locator = (Region::TempStack, -1);
        let result_locator = (Region::TempStack, self.get_relative_offset() + 1);

        if found_assign {
            self.skip_emit = rhs_arity < 2;
            let rhs_locator_opt = e.get_rhs().accept_visitor(self);
            rhs_locator_opt.as_ref()?;
            rhs_locator = rhs_locator_opt.unwrap();
            
            self.skip_emit = lhs_arity == 0;
            let lhs_locator_opt = e.get_lhs().accept_visitor(self);
            lhs_locator_opt.as_ref()?;
            lhs_locator = lhs_locator_opt.unwrap();
            self.skip_emit = false;
        } else {
            self.skip_emit = false;
            let lhs_locator_opt_2 = e.get_lhs().accept_visitor(self);
            lhs_locator_opt_2.as_ref()?;
            lhs_locator = lhs_locator_opt_2.unwrap();

            let rhs_locator_opt_2 = e.get_rhs().accept_visitor(self);
            rhs_locator_opt_2.as_ref()?;
            rhs_locator = rhs_locator_opt_2.unwrap();
        }

        // NOTE: Temporary results of binary-arity AST operations will be popped by immediately following REPLACE instructions.
        if found_assign {
            if e.get_rhs().get_operator().arity() == 2 {
                self.update_relative_offset(-1);
                rhs_locator.1 -= 1;
            }

            self.emit_step(Instruction::Binary(
                Opcode::Replace,
                lhs_locator.clone(),
                rhs_locator.clone(),
            ));
        } else {
            self.emit_step(Instruction::Nonary(expr_opcode));

            self.update_relative_offset(expr_opcode.get_stack_delta());
        }

        Some(if found_assign {
            lhs_locator
        } else {
            result_locator
        })
    }
}

impl StmtVisitor<bool> for IREmitter {
    fn visit_function_decl(&mut self, s: &FunctionDecl) -> bool {
        self.enter_fun_scope();

        let function_name =
            String::from(s.get_name_token().to_lexeme_str(&self.source_copy).unwrap());
        let is_func_name_recorded = self.record_fun_by_name(function_name);
        let mut arg_id = 0;

        #[allow(clippy::explicit_counter_loop)]
        for param in s.get_params() {
            let param_name = param
                .get_name_token()
                .to_lexeme_str(&self.source_copy)
                .unwrap();

            self.record_name_locator(String::from(param_name), (Region::ArgStore, arg_id));

            arg_id += 1;
        }

        if !s.get_body().accept_visitor(self) {
            eprintln!("Oops: failed to generate function body from declaration");
            self.has_error = true;
            return false;
        }

        self.leave_fun_scope();
        is_func_name_recorded
    }

    fn visit_block(&mut self, s: &Block) -> bool {
        if s.get_items().is_empty() {
            return false;
        }

        self.result
            .last_mut()
            .unwrap()
            .add_node(Node::new(Vec::new(), -1, -1));

        for temp_stmt in s.get_items() {
            if !temp_stmt.accept_visitor(self) {
                eprintln!("Oops: failed to generate nested block");
                self.has_error = true;
                return false;
            }
        }

        true
    }

    fn visit_variable_decl(&mut self, s: &VariableDecl) -> bool {
        let var_object_locator_opt = s.get_init_expr().accept_visitor(self);
        let var_locator = (Region::TempStack, self.get_relative_offset());

        if var_object_locator_opt.is_none() {
            self.has_error = true;
            return false;
        }

        let var_name = String::from(s.get_name_token().to_lexeme_str(&self.source_copy).unwrap());
        self.record_name_locator(var_name, var_locator);

        true
    }

    fn visit_if(&mut self, s: &If) -> bool {
        let condition_value_locator_opt = s.get_check().accept_visitor(self);

        if condition_value_locator_opt.is_none() {
            eprintln!("Oops: failed to generate if-check");
            self.has_error = true;
            return false;
        }

        let pre_if_block_id: i32 = self.result.last().unwrap().get_node_count() - 1;
        let block_1_id = pre_if_block_id + 1;

        self.emit_step(Instruction::Binary(
            Opcode::JumpElse,
            condition_value_locator_opt.unwrap(),
            (Region::BlockId, -1),
        ));

        if !s.get_truthy_body().accept_visitor(self) {
            eprintln!("Oops: failed to generate if-block");
            self.has_error = true;
            return false;
        }

        // NOTE: Here, I must patch the jump_else from before the if-block to go to a NOP before a possible JUMP skipping the else-block if available. This is done for correctness.
        self.emit_step(Instruction::Unary(Opcode::Jump, (Region::BlockId, -1)));
        self.emit_step(Instruction::Nonary(Opcode::Nop));
        self.emit_step(Instruction::Nonary(Opcode::GenPatch));

        self.record_proto_link(pre_if_block_id, block_1_id);

        let falsy_body_ok = s.get_falsy_body().accept_visitor(self);

        if !falsy_body_ok && !self.has_error {
            let if_fallthrough_id = block_1_id + 1;
            self.result
                .last_mut()
                .unwrap()
                .add_node(Node::new(Vec::new(), -1, -1));
            self.emit_step(Instruction::Nonary(Opcode::Nop));
            // self.emit_step(Instruction::Nonary(Opcode::GenPatch));

            self.record_proto_link(pre_if_block_id, if_fallthrough_id);
            self.record_proto_link(block_1_id, if_fallthrough_id);
            self.apply_proto_links();

            return true;
        } else if self.has_error {
            eprintln!("Oops: failed to generate true-block");
            self.has_error = true;
            return false;
        }

        self.emit_step(Instruction::Nonary(Opcode::Nop));
        self.emit_step(Instruction::Nonary(Opcode::GenPatch));

        let block_2_id = self.result.last().unwrap().get_node_count() - 1;
        self.record_proto_link(pre_if_block_id, block_2_id);

        let post_if_block_id = block_2_id + 1;
        self.result
            .last_mut()
            .unwrap()
            .add_node(Node::new(Vec::new(), -1, -1));
        self.emit_step(Instruction::Nonary(Opcode::Nop));
        self.emit_step(Instruction::Nonary(Opcode::GenPatch));

        self.record_proto_link(block_1_id, post_if_block_id);
        self.record_proto_link(block_2_id, post_if_block_id);
        self.apply_proto_links();

        true
    }

    fn visit_while(&mut self, s: &While) -> bool {
        self.emit_step(Instruction::Nonary(Opcode::Nop));
        self.emit_step(Instruction::Nonary(Opcode::GenBeginLoop));
        let condition_value_locator_opt = s.get_check().accept_visitor(self);

        if condition_value_locator_opt.is_none() {
            eprintln!("Oops: failed to generate while-check");
            self.has_error = true;
            return false;
        }

        let pre_while_block_id: i32 = self.result.last().unwrap().get_node_count() - 1;
        let while_block_id = pre_while_block_id + 1;

        self.emit_step(Instruction::Binary(
            Opcode::JumpElse,
            condition_value_locator_opt.unwrap(),
            (Region::BlockId, -1),
        ));
        self.record_proto_link(pre_while_block_id, while_block_id);

        if !s.get_body().accept_visitor(self) {
            eprintln!("Oops: failed to generate while-body");
            self.has_error = true;
            return false;
        }
        self.emit_step(Instruction::Unary(Opcode::Jump, (Region::BlockId, -1)));
        self.emit_step(Instruction::Nonary(Opcode::GenPatchBack));
        self.record_proto_link(while_block_id, while_block_id);

        self.result
            .last_mut()
            .unwrap()
            .add_node(Node::new(Vec::new(), -1, -1));
        let post_while_block_id = self.result.last().unwrap().get_node_count() - 1;
        self.record_proto_link(while_block_id, post_while_block_id);
        self.emit_step(Instruction::Nonary(Opcode::Nop));
        self.emit_step(Instruction::Nonary(Opcode::GenPatch));

        self.apply_proto_links();

        true
    }

    fn visit_return(&mut self, s: &Return) -> bool {
        let result_locator_opt = s.get_result().accept_visitor(self);

        if result_locator_opt.is_none() {
            eprintln!("Oops: failed to find locator for return result");
            return false;
        }

        self.emit_step(Instruction::Unary(
            Opcode::Return,
            result_locator_opt.unwrap(),
        ));

        true
    }

    fn visit_expr_stmt(&mut self, s: &ExprStmt) -> bool {
        let temp_result_locator_opt = s.get_inner().accept_visitor(self);

        if let Some(inner_result) = temp_result_locator_opt {
            // NOTE: If the inner-expr is an assignment, the result is always a reserved local variable slot. Therefore, I cannot POP since that would break stack correctness.
            if inner_result.0 != Region::TempStack {
                self.emit_step(Instruction::Nonary(Opcode::Pop));
                self.update_relative_offset(-1);
            }
        }

        true
    }
}
