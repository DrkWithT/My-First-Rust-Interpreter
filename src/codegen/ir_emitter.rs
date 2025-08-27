use std::collections::HashMap;
use std::collections::VecDeque;

use crate::codegen::ir::*;
use crate::codegen::layouts::ClassLayout;
use crate::codegen::layouts::LayoutTable;
use crate::frontend::ast::*;
use crate::frontend::token::*;
use crate::semantics::types::OperatorTag;
use crate::compiler::driver::SourceIndexedAST;
use crate::token_from;
use crate::utils::bundle::NativeBrief;
use crate::vm::heap::HeapValue;
use crate::vm::value::Value;

type IRLinkPair = (i32, i32);
pub type IRResult = (CFGStorage, Vec<Vec<Value>>, i32, Vec<HeapValue>);
type FuncInfo = (Locator, i32);

pub struct IREmitter<'b> {
    class_layouts: LayoutTable,
    class_var_names: HashMap<String, (String, Locator)>,
    fun_locals: HashMap<String, Locator>,
    fun_locations: HashMap<String, FuncInfo>,
    result: CFGStorage,

    /// NOTE: contains a Vec of corresponding constant Values per bytecode Chunk.
    proto_constants: Vec<Vec<Value>>,

    proto_heap_vals: Vec<HeapValue>,

    /// NOTE: contains `(from: i32, to: i32)` tuples to process after CFG node generation... Each "proto" link is applied to a CFG node by `from` before exhaustion which clears this Vec- The function's CFG is done by then!
    proto_links: Vec<IRLinkPair>,

    source_copy: String,

    /// NOTE: Stores the class name (if applicable) of a visited declaration.
    ctx_class_name: String,

    native_registry: &'b HashMap<&'static str, NativeBrief>,

    /// NOTE: tracks how many Values remain around the top stack slots for the current call frame.
    ctx_instance_locator: Locator,
    relative_stack_offset: i32,
    next_heap_id: i32,
    main_id: i32,
    has_prepass: bool,
    skip_emit: bool,
    in_ctor: bool,
    has_error: bool,
}

impl<'b> IREmitter<'b> {
    pub fn new(old_src: &str, native_mapping: &'b HashMap<&'static str, NativeBrief>) -> Self {
        Self {
            class_layouts: LayoutTable::default(),
            class_var_names: HashMap::new(),
            fun_locals: HashMap::new(),
            fun_locations: HashMap::new(),
            result: CFGStorage::new(),
            proto_constants: Vec::<Vec<Value>>::new(),
            proto_heap_vals: Vec::<HeapValue>::new(),
            proto_links: Vec::<IRLinkPair>::new(),
            source_copy: String::from(old_src),
            ctx_class_name: String::default(),
            native_registry: native_mapping,
            ctx_instance_locator: (Region::TempStack, -1),
            relative_stack_offset: -1,
            next_heap_id: -1,
            main_id: -1,
            has_prepass: false,
            skip_emit: false,
            in_ctor: false,
            has_error: false,
        }
    }

    fn set_prepass_flag(&mut self, flag: bool) {
        self.has_prepass = flag;
    }

    fn record_class_field(&mut self, class_name: &str, field_name: &str, ) -> bool {
        if let Some(class_layout_ref) = self.class_layouts.get_mut(class_name) {
            return class_layout_ref.add_member(field_name.to_string());
        }

        false
    }

    fn lookup_method_as_fun(&self, class_name: &str, method_name: &str) -> Option<i32> {
        let class_name_s = class_name.to_string();

        if let Some(layout_ref) = self.class_layouts.get(&class_name_s) {
            if let Some(method_mapping) = layout_ref.get_real_method_id(method_name.to_string()) {
                return Some(method_mapping.1);
            }
        }

        None
    }

    fn record_fun_by_name(&mut self, name: String, arity: i32) -> Option<i32> {        
        if self.fun_locations.contains_key(name.as_str()) {
            return None;
        }
        
        let next_fun_id = self.fun_locations.len();

        if self.main_id == -1 && name.as_str() == "main" {
            self.main_id = next_fun_id as i32;
        }

        self.fun_locations
            .insert(name, ((Region::Functions, next_fun_id as i32), arity));

        Some(next_fun_id as i32)
    }

    fn enter_fun_scope(&mut self) {
        self.result.push(CFG::new());
        self.proto_constants.push(Vec::new());

        self.reset_relative_offset(-1);
    }

    fn leave_fun_scope(&mut self) {
        self.fun_locals.clear();
    }

    fn record_varname_locator(&mut self, name: String, item: Locator) {
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

    fn lookup_locator_of(&self, opt_class_name: &str, name: &str) -> Option<Locator> {
        if !opt_class_name.is_empty() {
            if let Some(class_layout_ref) = self.class_layouts.get(opt_class_name) {
                if let Some(field_id) = class_layout_ref.get_member_id(name.to_string()) {
                    return Some((Region::Field, field_id));
                } else if let Some((_, met_external_id)) = class_layout_ref.get_real_method_id(name.to_string()) {
                    return Some((Region::Functions, met_external_id));
                }

                return None;
            }

            return None;
        }
 
        if self.fun_locals.contains_key(name) {
            return Some(self.fun_locals.get(name).unwrap().clone());
        } else if self.native_registry.contains_key(name) {
            return Some((Region::Natives, self.native_registry.get(name).unwrap().id));
        } else if self.fun_locations.contains_key(name) {
            return Some(self.fun_locations.get(name).unwrap().clone().0);
        }

        None
    }

    fn lookup_fun_arity(&self, opt_class_name: &str, fun_name: &str) -> Option<i32> {
        if !opt_class_name.is_empty() {
            if let Some(layout_ref) = self.class_layouts.get(opt_class_name) {
                if layout_ref.get_real_method_id(fun_name.to_string()).is_some() {
                    if let Some((_, fun_info)) = self.fun_locations.iter().find(|entry| {
                        entry.0 == fun_name
                    }) {
                        return Some(fun_info.1);
                    }

                    return None;
                }

                return None;
            }

            return None;
        }

        if self.native_registry.contains_key(fun_name) {
            return Some(self.native_registry.get(fun_name).unwrap().arity);
        } else if self.fun_locations.contains_key(fun_name) {
            return Some(self.fun_locations.get(fun_name).unwrap().1);
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

    fn reset_relative_offset(&mut self, arg: i32) {
        self.relative_stack_offset = arg;
    }

    fn update_relative_offset(&mut self, count: i32) {
        self.relative_stack_offset += count;
    }

    fn get_next_heap_id(&mut self) -> i32 {
        self.next_heap_id += 1;
        self.next_heap_id
    }

    fn emit_step(&mut self, step: Instruction) {
        self.result.last_mut().unwrap().add_instruction_recent(step);
    }

    fn help_emit_assign(&mut self, e: &Binary) -> Option<Locator> {
        let lhs_arity = ast_op_to_ir_op(e.get_lhs().get_operator()).arity();
        let rhs_arity = ast_op_to_ir_op(e.get_rhs().get_operator()).arity();

        #[allow(unused_assignments)]
        let mut lhs_locator = (Region::TempStack, -1);
        #[allow(unused_assignments)]
        let mut rhs_locator = (Region::TempStack, -1);

        self.skip_emit = rhs_arity < 2;
        let rhs_locator_opt = e.get_rhs().accept_visitor(self);
        rhs_locator_opt.as_ref()?;
        rhs_locator = rhs_locator_opt.unwrap();

        self.skip_emit = lhs_arity == 0;
        let lhs_locator_opt = e.get_lhs().accept_visitor(self);
        lhs_locator_opt.as_ref()?;
        lhs_locator = lhs_locator_opt.unwrap();
        self.skip_emit = false;

        if e.get_rhs().get_operator().arity() == 2 {
            self.update_relative_offset(-1);
            rhs_locator.1 -= 1;
        }

        self.emit_step(Instruction::Binary(
            Opcode::Replace,
            lhs_locator.clone(),
            rhs_locator.clone(),
        ));

        Some(lhs_locator)
    }

    fn help_emit_access(&mut self, e: &Binary) -> Option<Locator> {
        self.skip_emit = true;

        let instance_name_token = e.get_lhs().get_token_opt().unwrap_or(token_from!(TokenType::Unknown, 0, 0, 0, 0));
        let access_expr_line_no = instance_name_token.line_no;

        if instance_name_token.tag == TokenType::Unknown {
            eprintln!("Oops: Unsupported LHS token for codegen of class member access, expected a name.");
            self.has_error = true;
            return None;
        }

        let method_name_token = e.get_rhs().get_token_opt().unwrap_or(token_from!(TokenType::Unknown, 0, 0, 0, 0));

        if method_name_token.tag == TokenType::Unknown {
            eprintln!("Oops: Unsupported RHS token for codegen of class member access, expected a field / method name.");
            self.has_error = true;
            return None;
        }

        let instance_name_lexeme = instance_name_token.to_lexeme_str(&self.source_copy).unwrap_or("");
        let instance_class_info_opt = self.class_var_names.get(instance_name_lexeme);

        if instance_class_info_opt.is_none() {
            eprintln!("Oops: At line {access_expr_line_no}, no valid LHS class name exists- Cannot determine the object's layout information.");
            self.has_error = true;
            return None;
        }

        let instance_method_name_opt = method_name_token.to_lexeme_str(&self.source_copy);

        if instance_method_name_opt.is_none() {
            eprintln!("Oops: At line {access_expr_line_no}, no valid RHS method name exists- Cannot determine the procedure's location.");
            self.has_error = true;
            return None;
        }

        self.skip_emit = false;

        let class_info = instance_class_info_opt.unwrap().clone();
        let method_name = instance_method_name_opt.unwrap().to_string();

        if let Some(real_method_fun_id) = self.lookup_method_as_fun(&class_info.0, &method_name) {
            self.ctx_instance_locator = class_info.1;

            return Some((Region::Methods, real_method_fun_id));
        } else if let Some(read_field_id) = self.lookup_locator_of(&class_info.0, &method_name) {
            return Some(read_field_id);
        }

        None
    }

    fn help_emit_bin_normal(&mut self, e: &Binary) -> Option<Locator> {
        let expr_opcode = ast_op_to_ir_op(e.get_operator());

        let result_locator = (Region::TempStack, self.get_relative_offset() + 1);

        self.skip_emit = false;
        let lhs_locator_opt_2 = e.get_lhs().accept_visitor(self);
        lhs_locator_opt_2.as_ref()?;

        let rhs_locator_opt_2 = e.get_rhs().accept_visitor(self);
        rhs_locator_opt_2.as_ref()?;

        self.emit_step(Instruction::Nonary(expr_opcode));
        self.update_relative_offset(expr_opcode.get_stack_delta());

        Some(result_locator)
    }

    pub fn emit_all_ir(&mut self, ast_tops: &VecDeque<SourceIndexedAST>) -> Option<IRResult> {
        self.set_prepass_flag(true);

        for (_, temp) in ast_tops {
            if !temp.accept_visitor(self) {
                eprintln!("Oops: failed to track declaration");
                return None;
            }
        }

        self.set_prepass_flag(false);

        for (_, temp) in ast_tops {
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
            std::mem::take(&mut self.proto_heap_vals),
        ))
    }
}

impl<'evl3> ExprVisitor<'evl3, Option<Locator>> for IREmitter<'evl3> {
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
            TokenType::LiteralChar => {
                let temp_char = literal_lexeme.chars().nth(0).unwrap_or('?');
                let temp_char_locator = self.record_proto_constant(Value::Char(temp_char as u8));

                if !self.skip_emit {
                    self.emit_step(Instruction::Unary(
                        Opcode::LoadConst,
                        temp_char_locator.clone(),
                    ));
                    self.update_relative_offset(1);
                }

                Some(temp_char_locator)
            },
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
            },
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
            },
            TokenType::LiteralVarchar => {
                let temp_varchar = literal_lexeme.to_string();

                let temp_varchar_heap_id = self.get_next_heap_id() as i16;
                let temp_varchar_locator = self.record_proto_constant(Value::HeapRef(temp_varchar_heap_id));
                self.proto_heap_vals.push(HeapValue::Varchar(temp_varchar));
                self.emit_step(Instruction::Unary(Opcode::Push, (Region::ObjectHeap, temp_varchar_heap_id as i32)));

                Some(temp_varchar_locator)
            },
            TokenType::Identifier => {
                let named_locator_opt = self.lookup_locator_of(&self.ctx_class_name, literal_lexeme);

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
        let old_skip_emit = self.skip_emit;
        self.skip_emit = true;
        let callee_locator_opt = e.get_callee().accept_visitor(self);
        self.skip_emit = old_skip_emit;

        callee_locator_opt.as_ref()?;

        let callee_locator = callee_locator_opt.unwrap();

        let callee_name = e.get_callee().get_token_opt().unwrap().to_lexeme_str(&self.source_copy).unwrap_or("");

        let callee_arity = self.lookup_fun_arity( &self.ctx_class_name, callee_name).unwrap_or(0);

        let calling_args = e.get_args();
        let result_locator = (Region::TempStack, self.get_relative_offset() + 1);

        // NOTE: all args are temporary values and the consuming function call will automatically pop them all...
        for arg_ref in calling_args {
            arg_ref.accept_visitor(self)?;
        }

        match callee_locator.0 {
            Region::Natives => {
                self.emit_step(Instruction::Unary(
                    Opcode::NativeCall,
                    callee_locator,
                ));
            },
            Region::Functions => {
                self.emit_step(Instruction::Binary(
                    Opcode::Call,
                    callee_locator,
                    (Region::Immediate, callee_arity),
                ));
            },
            Region::Methods => {
                self.emit_step(Instruction::Ternary(
                    Opcode::InstanceCall,
                    self.ctx_instance_locator.clone(),
                    (Region::Functions, callee_locator.1),
                    (Region::Immediate, callee_arity),
                ));
            }
            _ => {
                return None;
            }
        }

        self.update_relative_offset(1 - callee_arity);

        Some(result_locator)
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

    fn visit_binary(&mut self, e: &Binary) -> Option<Locator> {
        match e.op_tag {
            OperatorTag::Assign => self.help_emit_assign(e),
            OperatorTag::Access => self.help_emit_access(e),
            _ => self.help_emit_bin_normal(e),
        }
    }
}

impl StmtVisitor<bool> for IREmitter<'_> {
    #[allow(unused_variables)]
    fn visit_import(&mut self, s: &Import) -> bool {
        true
    }

    #[allow(unused_variables)]
    fn visit_foreign_stub(&mut self, s: &ForeignStub) -> bool {
        true
    }

    fn visit_function_decl(&mut self, s: &FunctionDecl) -> bool {
        let function_name = String::from(s.get_name_token().to_lexeme_str(&self.source_copy).unwrap());

        if self.has_prepass {
            let arity_i32 = s.get_params().len() as i32;
            let func_name_record_id = self.record_fun_by_name(function_name.clone(), arity_i32).unwrap_or(-1);

            func_name_record_id != -1
        } else {
            self.enter_fun_scope();

            if let Some(maybe_main_loc) = self.lookup_locator_of("", &function_name) {
                // Since the initial main call started upon an empty stack (rsp = -1), adjust the offset accordingly.
                if maybe_main_loc.1 == self.main_id {
                    self.reset_relative_offset(-1);
                }
            }

            #[allow(clippy::explicit_counter_loop)]
            for (param_it, param) in s.get_params().iter().enumerate() {
                let param_name = param
                    .get_name_token()
                    .to_lexeme_str(&self.source_copy)
                    .unwrap();

                self.record_varname_locator(String::from(param_name), (Region::ArgStore, param_it as i32));
            }

            if !s.get_body().accept_visitor(self) {
                eprintln!("Oops: failed to generate function body from declaration");
                self.has_error = true;
            }

            self.leave_fun_scope();
            !self.has_error
        }
    }

    fn visit_field_decl(&mut self, s: &FieldDecl) -> bool {
        let source_copy_view = self.source_copy.clone();
        let field_name = s.get_name_token().to_lexeme_str(&source_copy_view).unwrap_or("");

        if field_name.is_empty() {
            return false;
        }

        let current_class_name_copy = self.ctx_class_name.clone();
        self.record_class_field(&current_class_name_copy, field_name)
    }

    fn visit_constructor_decl(&mut self, s: &ConstructorDecl) -> bool {
        let ctor_class_name = self.ctx_class_name.clone();

        if self.has_prepass {
            let ctor_arity = s.get_params().len() as i32;
            let ctor_top_id = self.record_fun_by_name(ctor_class_name.clone(), ctor_arity).unwrap_or(-1);

            if ctor_top_id == -1 {
                eprintln!("Found duplicate symbol of {} constructor- Check the class declaration.", ctor_class_name.as_str());
                self.has_error = true;
                return false;
            }

            if let Some(class_layout_ref) = self.class_layouts.get_mut(ctor_class_name.as_str()) {
                return class_layout_ref.add_method_id(ctor_class_name, ctor_top_id);
            }

            false
        } else {
            self.enter_fun_scope();
            self.in_ctor = true;

            let layout_field_count_opt = self.class_layouts.get(&ctor_class_name);

            if layout_field_count_opt.is_none() {
                eprintln!("Oops: failed to find member layout count for class '{}'", ctor_class_name.as_str());
                self.has_error = true;
                return false;
            }

            let layout_field_count = layout_field_count_opt.unwrap().get_field_count();

            for (param_it, param) in s.get_params().iter().enumerate() {
                let param_name = param
                    .get_name_token()
                    .to_lexeme_str(&self.source_copy)
                    .unwrap();

                self.record_varname_locator(String::from(param_name), (Region::ArgStore, param_it as i32));
            }

            // NOTE: add MAKE_HEAP_OBJECT instruction to guarantee the object exists by the time the ctor body runs.
            let pre_ctor_body_block_id = self.result.last().unwrap().get_node_count() - 1;
            let ctor_start_block_id = pre_ctor_body_block_id + 1;

            self.result
                .last_mut()
                .unwrap()
                .add_node(Node::new(Vec::new(), -1, -1));
            
            self.emit_step(Instruction::Unary(Opcode::MakeHeapObject, (Region::Immediate, layout_field_count)));

            self.record_proto_link(pre_ctor_body_block_id, ctor_start_block_id);

            if !s.get_body().accept_visitor(self) {
                eprintln!("Oops: failed to generate constructor body for class '{}'", ctor_class_name.as_str());
                self.has_error = true;
            }

            self.in_ctor = false;
            self.leave_fun_scope();

            !self.has_error
        }
    }

    fn visit_method_decl(&mut self, s: &MethodDecl) -> bool {
        let class_name = self.ctx_class_name.clone();

        if self.has_prepass {
            let source_copy = self.source_copy.clone();
            let met_arity = s.get_params().len() as i32;
            let met_top_id = self.record_fun_by_name(class_name.clone(), met_arity).unwrap_or(-1);
            let met_name = s.get_name_token().to_lexeme_str(&source_copy).unwrap_or("");

            if met_top_id == -1 {
                eprintln!("Oops: Found duplicate symbol of {} method of class '{}'.", met_name, class_name.as_str());
                self.has_error = true;
                return false;
            }

            if let Some(class_layout_ref) = self.class_layouts.get_mut(class_name.as_str()) {
                return class_layout_ref.add_method_id(class_name, met_top_id);
            }

            false
        } else {
            self.enter_fun_scope();

            for (param_it, param) in s.get_params().iter().enumerate() {
                let param_name = param
                    .get_name_token()
                    .to_lexeme_str(&self.source_copy)
                    .unwrap();

                self.record_varname_locator(String::from(param_name), (Region::ArgStore, param_it as i32));
            }

            if !s.get_body().accept_visitor(self) {
                eprintln!("Oops: failed to generate constructor body for class '{}'", class_name.as_str());
                self.has_error = true;
            }

            self.leave_fun_scope();

            !self.has_error
        }
    }

    fn visit_class_decl(&mut self, s: &ClassDecl) -> bool {
        let temp_class_name = s.get_class_type().typename();

        if self.has_prepass {
            self.class_layouts.insert(temp_class_name, ClassLayout::default()).is_none()
        } else {
            self.ctx_class_name = temp_class_name;

            for (member_stmt, _) in s.get_members() {
                if !member_stmt.accept_visitor(self) {
                    break;
                }
            }

            self.ctx_class_name.clear();

            !self.has_error
        }
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

        let opt_class_name = s.get_typing().typename();
        let is_of_class_type = self.class_layouts.contains_key(&opt_class_name);

        let var_name = String::from(s.get_name_token().to_lexeme_str(&self.source_copy).unwrap());

        if is_of_class_type {
            self.class_var_names.insert(var_name, (opt_class_name, var_locator)).is_none()
        } else {
            self.record_varname_locator(var_name, var_locator);
            true
        }
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
        if self.in_ctor {
            self.emit_step(Instruction::Nonary(Opcode::Leave));
            return true;
        }

        let result_locator_opt = s.get_result().accept_visitor(self);
        let result_delta = ast_op_to_ir_op(s.get_result().get_operator()).get_stack_delta();

        if result_locator_opt.is_none() {
            eprintln!("Oops: failed to find locator for return result");
            return false;
        }

        let mut checked_locator = if let Some(result_locator) = result_locator_opt {
            let (result_region, result_n) = result_locator.clone();

            match result_region {
                Region::Immediate | Region::ArgStore => result_locator,
                Region::TempStack => (result_region, result_n + result_delta),
                _ => (Region::Immediate, -1),
            }
        } else {
            (Region::Immediate, -1)
        };

        if checked_locator.1 == -1 {
            checked_locator.1 += 1;
        }

        self.emit_step(Instruction::Unary(
            Opcode::Return,
            checked_locator,
        ));

        self.reset_relative_offset(-1);

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
