use std::collections::HashMap;

use crate::frontend::token::*;
use crate::frontend::ast::*;
use crate::semantics::scope::*;
use crate::semantics::types::{AccessFlag, OperatorTag, ValueCategoryTag};
use crate::semantics::blueprint::*;

const BOOLEAN_TYPE_ID_N: i32 = 0;
const CHAR_TYPE_ID_N: i32 = 1;
const INTEGER_TYPE_ID_N: i32 = 2;
const FLOATING_TYPE_ID_N: i32 = 3;
const VARCHAR_TYPE_ID_N: i32 = 4;
const ANY_TYPE_ID_N: i32 = 5;

/**
 * ### ABOUT
 * Specifies how non-class-specific names are recorded.
 */
#[repr(i8)]
#[derive(Clone, Copy, PartialEq)]
enum RecordInfoMode {
    /// Denotes a local variable.
    Local,

    /// Denotes a function or class (and eventually lambdas).
    Global,

    /// Denotes a class-local member.
    Member,
}

/**
 # NOTE
 This is a helper function to check types for homogeneously typed expressions e.g arithmetics, assignments, etc. However, this check will fail on any unknown types for things such as undeclared names.
 */
fn check_binary_typing_homogeneously(lhs_info: &SemanticNote, rhs_info: &SemanticNote) -> bool {
    let lhs_type_idx = if let SemanticNote::DataValue(idx, _) = lhs_info {
        *idx
    } else { -1 };

    let rhs_type_idx = if let SemanticNote::DataValue(idx, _) = rhs_info {
        *idx
    } else { -1 };

    lhs_type_idx != -1 && rhs_type_idx != -1 && lhs_type_idx == rhs_type_idx
}

fn check_assignment_value_groups(lhs_info: &SemanticNote, rhs_info: &SemanticNote) -> bool {
    let lhs_value_group = if let SemanticNote::DataValue(_, value_group) = lhs_info {
        *value_group
    } else { ValueCategoryTag::Temporary };

    let rhs_value_group = if let SemanticNote::DataValue(_, rhs_value_group) = rhs_info {
        *rhs_value_group
    } else { ValueCategoryTag::Unknown };

    lhs_value_group == ValueCategoryTag::Identity && rhs_value_group != ValueCategoryTag::Unknown
}

/**
 ### ABOUT
 * Contains the implementation of the semantic checker. This logic enforces type-safety and value semantics.
 ### EXAMPLES
 Here are some illegal cases of code:
 * `1 = 42;` is an invalid assignment since the LHS has no identity.
 * `a = a + 1;` is an undeclared variable since a has no declaration.
 * `let a: int = 42;`, `a = 3.1415` is invalid because the types are mismatched.
 ### CAVEATS
 No support for arrays exists yet.
 ### TODO's
 * Fix name resolution to use and handle class-specific member lookups.
 * Add visitations for class constructs.
 */
pub struct Analyzer {
    class_blueprints: BlueprintTable,
    type_table: HashMap<i32, String>,
    temp_token: Token,
    scopes: ScopeStack,
    source_str: String,
    
    /// **NOTE:** Indicates the current class decl. being analyzed by its type ID.
    current_class_id: i32,

    /// **NOTE:** Indicates the current access modifier of a visiting member decl. in the currently visited class.
    current_class_mod: AccessFlag,

    /// **NOTE:** Indicates whether the currently referenced name of a member / variable / function is scope visible.
    current_name_accessible: AccessFlag,

    /// **NOTE:** Indicates that top-level decls. must be recorded before body processing. If `false`, body processing takes place instead.
    prepass_flag: bool,
}

impl Analyzer {
    pub fn new(source_view: String) -> Self {
        let mut temp_type_table = HashMap::<i32, String>::new();
        temp_type_table.insert(ANY_TYPE_ID_N, String::from("any"));
        temp_type_table.insert(BOOLEAN_TYPE_ID_N, String::from("bool"));
        temp_type_table.insert(CHAR_TYPE_ID_N, String::from("char"));
        temp_type_table.insert(INTEGER_TYPE_ID_N, String::from("int"));
        temp_type_table.insert(FLOATING_TYPE_ID_N, String::from("float"));
        temp_type_table.insert(VARCHAR_TYPE_ID_N, String::from("varchar"));

        Self {
            class_blueprints: BlueprintTable::default(),
            type_table: temp_type_table,
            temp_token: Token {
                tag: TokenType::Unknown,
                start: 0,
                length: 1,
                line_no: 0,
                col_no: 0
            },
            scopes: ScopeStack::default(),
            source_str: source_view,
            current_class_id: -1,
            current_class_mod: AccessFlag::Hidden,
            current_name_accessible: AccessFlag::Hidden,
            prepass_flag: true,
        }
    }

    pub fn reset_source(&mut self, source_view_next: String) {
        self.source_str = source_view_next;
    }

    pub fn reset_with(&mut self) {
        self.temp_token = Token {
            tag: TokenType::Unknown,
            start: 0,
            length: 1,
            line_no: 0,
            col_no: 0
        };
        // self.scopes.reset();
    }

    pub fn set_current_class_id(&mut self, cid: i32) {
        self.current_class_id = cid;
    }

    pub fn update_current_class_mod(&mut self, access_mod: AccessFlag) {
        self.current_class_mod = access_mod;
    }

    /**
     * ### ABOUT
     * Allows the semantic analyzer to visit declaration bodies.
     */
    pub fn set_preprocess_decls_flag(&mut self) {
        self.prepass_flag = true;
    }

    /**
     * ### ABOUT
     * Allows the semantic analyzer to visit declaration bodies.
     */
    pub fn clear_preprocess_decls_flag(&mut self) {
        self.prepass_flag = false;
    }

    fn record_type(&mut self, type_str: String) -> i32 {
        let next_type_id = self.type_table.len() as i32;

        let pre_result_opt = self.type_table.iter().find(
            |&item| item.1.as_str() == type_str.as_str()
        );

        if pre_result_opt.is_none() {
            self.type_table.insert(next_type_id, type_str);
            return next_type_id;
        }

        *pre_result_opt.unwrap().0
    }

    fn record_new_class_bp(&mut self, type_id: i32) -> bool {
        self.class_blueprints.try_set_entry(type_id, ClassBlueprint::new(type_id))
    }

    fn lookup_name_info(&mut self, name: &str) -> SemanticNote {
        if self.current_class_id != -1 {
            if let Some(bp_ref) = self.class_blueprints.try_get_entry_mut(self.current_class_id) {
                if let Some(bp_ref_member_ref) = bp_ref.try_get_entry_mut(name) {
                    self.current_name_accessible = bp_ref_member_ref.0;
                    return bp_ref_member_ref.1.note.clone();
                }
            }
        }

        let normal_info = self.scopes.lookup_name_info(name);
        self.current_name_accessible = if normal_info.is_dud() { AccessFlag::Hidden } else { AccessFlag::Exposed };

        normal_info
    }

    fn record_name_info(&mut self, name: &str, info: SemanticNote, mode: RecordInfoMode) -> bool {
        match mode {
            RecordInfoMode::Local => self.scopes.current_scope_mut().unwrap().try_set_entry(name, info),
            RecordInfoMode::Global => self.scopes.global_scope_mut().unwrap().try_set_entry(name, info),
            RecordInfoMode::Member => {
                self.record_class_member_info(self.current_class_id, name, self.current_class_mod, info)
            }
        }
    }

    fn record_class_member_info(&mut self, class_id: i32, name: &str, access_mod_arg: AccessFlag, note_arg: SemanticNote) -> bool {
        let class_bp_ref_opt = self.class_blueprints.try_get_entry_mut(class_id);

        if class_bp_ref_opt.is_none() {
            return false;
        }

        class_bp_ref_opt.unwrap().try_set_entry(name, ClassMember {
            note: note_arg,
            access_mod: access_mod_arg,
        })
    }

    fn report_plain_error(&self, msg: &str) {
        eprintln!("SemaError:\n{msg}");
    }

    fn report_culprit_error(&self, culprit: &Token, msg: &str) {
        eprintln!("SemaError at [Ln {}, Col {}]:\nCulprit token: '{}'\n{}", culprit.line_no, culprit.col_no, culprit.to_lexeme_str(self.source_str.as_str()).unwrap_or("..."), msg);
    }

    pub fn check_top_ast(&mut self, func_ast: &dyn Stmt) -> bool {
        func_ast.accept_visitor(self)
    }
}

impl<'evl2> ExprVisitor<'evl2, SemanticNote> for Analyzer{
    fn visit_primitive(&mut self, e: &Primitive) -> SemanticNote {
        let source_copy = self.source_str.clone();
        let literal_lexeme = e.get_token().to_lexeme_str(source_copy.as_str()).unwrap_or("");
        let literal_tag = e.get_token().tag;
        self.temp_token = *e.get_token();

        match literal_tag {
            TokenType::LiteralBool => {
                SemanticNote::DataValue(BOOLEAN_TYPE_ID_N, ValueCategoryTag::Temporary)
            },
            TokenType::LiteralChar => {
                SemanticNote::DataValue(CHAR_TYPE_ID_N, ValueCategoryTag::Temporary)
            },
            TokenType::LiteralInt => {
                SemanticNote::DataValue(INTEGER_TYPE_ID_N, ValueCategoryTag::Temporary)
            },
            TokenType::LiteralFloat => {
                SemanticNote::DataValue(FLOATING_TYPE_ID_N, ValueCategoryTag::Temporary)
            },
            TokenType::LiteralVarchar => {
                SemanticNote::DataValue(VARCHAR_TYPE_ID_N, ValueCategoryTag::Temporary)
            },
            TokenType::Identifier => {
                self.lookup_name_info(literal_lexeme)
            },
            _ => {
                SemanticNote::Dud
            },
        }
    }

    fn visit_call(&mut self, e: &Call) -> SemanticNote {
        let source_copy = self.source_str.clone();
        let callee_info = e.get_callee().accept_visitor_sema(self);
        let callee_token = self.temp_token;

        if callee_info.is_dud() {
            self.report_culprit_error(&callee_token, "The callee name is likely undeclared, did you declare <name> before?");
            return SemanticNote::Dud;
        }

        let callable_info_opt_1 = callee_info.try_unbox_callable_info(); // NOTE: this is a procedure OR ctor!
        let callable_info_opt_2: Option<RawMethodCallable> = if callable_info_opt_1.is_none() { callee_info.try_unbox_method_info() } else { None };

        if callable_info_opt_1.is_none() && callable_info_opt_2.is_none() {
            self.report_culprit_error(&callee_token, "The callee is a non-callable entity OR an inaccessible member.");
            SemanticNote::Dud
        } else if callable_info_opt_1.is_some() {
            let proc_or_ctor_info = callable_info_opt_1.unwrap();
            
            let callee_lexeme_1 = callee_token.to_lexeme_str(source_copy.as_str()).unwrap_or("...");
            let callable_arity = proc_or_ctor_info.2;
            let passed_arity = e.get_args().len() as i32;
            
            if passed_arity != callable_arity {
                let msg_string = format!("For callee '{callee_lexeme_1}'- Expected {callable_arity} arguments instead of {passed_arity}.");
                self.report_culprit_error(&self.temp_token, msg_string.as_str());

                return SemanticNote::Dud;
            }
            
            for arg_it in 0..passed_arity {
                let temp_arg_type_id = e.get_args().get(arg_it as usize).unwrap().accept_visitor_sema(self);

                match temp_arg_type_id {
                    SemanticNote::DataValue(argv_type_id, _) => {
                        let expected_type_id = *proc_or_ctor_info.0.get(arg_it as usize).unwrap();
                        
                        if expected_type_id != ANY_TYPE_ID_N && expected_type_id != argv_type_id {
                            let mismatch_err_msg = format!("For argument #{arg_it}, a mismatched type was found. Please check the declaration of 'fun {callee_lexeme_1}'.");
                            
                            self.report_culprit_error(&callee_token, mismatch_err_msg.as_str());
                            return SemanticNote::Dud;
                        }
                    },
                    _ => {
                        let invalid_arg_msg = format!("For argument {arg_it}, an invalid type was found. Please check the declaration of '{callee_lexeme_1}'.");
                        self.report_culprit_error(&callee_token, invalid_arg_msg.as_str());
                        
                        return SemanticNote::Dud;
                    }
                }
            }

            let callable_result_type_id = proc_or_ctor_info.1;
            
            SemanticNote::DataValue(callable_result_type_id, ValueCategoryTag::Temporary)
        } else {
            let method_info = callable_info_opt_2.unwrap();

            let callee_lexeme_2 = callee_token.to_lexeme_str(source_copy.as_str()).unwrap_or("...");
            let callable_arity = method_info.2;
            let passed_arity = e.get_args().len() as i32;
            
            if passed_arity != callable_arity {
                let msg_string = format!("For callee '{callee_lexeme_2}'- Expected {callable_arity} arguments instead of {passed_arity}.");
                self.report_culprit_error(&self.temp_token, msg_string.as_str());

                return SemanticNote::Dud;
            }

            for arg_it in 0..passed_arity {
                let temp_arg_type_id = e.get_args().get(arg_it as usize).unwrap().accept_visitor_sema(self);

                match temp_arg_type_id {
                    SemanticNote::DataValue(argv_type_id, _) => {
                        let expected_type_id = *method_info.0.get(arg_it as usize).unwrap();
                        
                        if expected_type_id != ANY_TYPE_ID_N && expected_type_id != argv_type_id {
                            let mismatch_err_msg = format!("For argument #{arg_it}, a mismatched type was found. Please check the declaration of 'met {callee_lexeme_2}'.");
                            
                            self.report_culprit_error(&callee_token, mismatch_err_msg.as_str());
                            return SemanticNote::Dud;
                        }
                    },
                    _ => {
                        let invalid_arg_msg = format!("For argument {arg_it}, an invalid type was found. Please check the declaration of met '{callee_lexeme_2}'.");
                        self.report_culprit_error(&callee_token, invalid_arg_msg.as_str());
                        
                        return SemanticNote::Dud;
                    }
                }
            }

            let method_result_type_id = method_info.1;

            SemanticNote::DataValue(method_result_type_id, ValueCategoryTag::Temporary)
        }
    }

    // fn visit_array(&self) -> Res;
    // fn visit_lambda(&self) -> Res;

    fn visit_unary(&mut self, e: &Unary) -> SemanticNote {
        let expr_op = e.get_operator();
        let expr_inner_type = e.get_inner().accept_visitor_sema(self);

        let inner_result_info_opt = match expr_op {
            OperatorTag::Negate => {
                expr_inner_type.try_unbox_data_value()
            },
            _ => {
                None
            },
        };

        if inner_result_info_opt.is_none() {
            self.report_plain_error("Invalid unary expression- Only arithmetic negations are allowed for now.");
            return SemanticNote::Dud;
        }

        if let SemanticNote::DataValue(inner_type_id, _) = expr_inner_type {
            match inner_type_id {
                1 | 2=> {
                    return SemanticNote::DataValue(inner_type_id, ValueCategoryTag::Temporary);
                },
                _ => {
                    self.report_plain_error("The negated value was not a numeric type (int or float).");
                },
            }
        } else {
            self.report_plain_error("The negated value was a non-data value. Non-data types include callable objects e.g lambdas.");
        }

        SemanticNote::Dud
    }

    /// # TODO
    /// Support member accesses.
    fn visit_binary(&mut self, e: &Binary) -> SemanticNote {
        let expr_op = e.get_operator();
        let lhs_info = e.get_lhs().accept_visitor_sema(self);
        let lhs_accessibility = self.current_name_accessible;
        let expr_line_no = self.temp_token.line_no;

        if expr_op == OperatorTag::Access && self.current_class_id == -1 {
            self.set_current_class_id(
                lhs_info.try_unbox_class_info_id().unwrap_or(-1)
            );
        }

        let rhs_info = e.get_rhs().accept_visitor_sema(self);
        let rhs_accessibility = self.current_name_accessible;

        if expr_op.is_homogeneously_typed() {
            if !check_binary_typing_homogeneously(&lhs_info, &rhs_info) {
                let mismatched_opers_msg = format!("Found mismatched types for {} expression around Ln. {}", expr_op.as_symbol(), expr_line_no);
                self.report_plain_error(mismatched_opers_msg.as_str());

                return SemanticNote::Dud;
            }
        } else if expr_op == OperatorTag::Access {
            return if rhs_info.is_dud() || lhs_accessibility == AccessFlag::Hidden || rhs_accessibility == AccessFlag::Hidden {
                let class_type_id = self.current_class_id;
                let class_name = if self.current_class_id != -1 {
                    self.type_table.get(&class_type_id).unwrap().as_str()
                } else { "(unknown-type)" };
                let bad_member_access_msg = format!("Cannot access member of {class_name} by name around Ln. {expr_line_no}");
                self.report_plain_error(bad_member_access_msg.as_str());

                SemanticNote::Dud
            } else { rhs_info };
        } else {
            let unsupported_operator_msg = format!("Unsupported operator found around Ln. {}: {}", expr_line_no, expr_op.as_symbol());
            self.report_plain_error(unsupported_operator_msg.as_str());

            return SemanticNote::Dud;
        }

        if expr_op.is_value_group_sensitive() {
            if let OperatorTag::Assign = expr_op {
                if !check_assignment_value_groups(&lhs_info, &rhs_info) {
                    let bad_lhs_msg = format!("Invalid assignment at Ln. {expr_line_no}- LHS is not assignable.");
                    self.report_plain_error(bad_lhs_msg.as_str());

                    return SemanticNote::Dud;
                }
            } else {
                let bad_binary_op_msg = format!("Unsupported operator {} for binary expr. at Ln. {}", expr_op.as_symbol(), expr_line_no);
                self.report_plain_error(bad_binary_op_msg.as_str());

                return SemanticNote::Dud;
            }
        }

        let (unboxed_type_id, unboxed_value_group) = lhs_info.try_unbox_data_value().unwrap_or((-1, ValueCategoryTag::Unknown));

        SemanticNote::DataValue(unboxed_type_id, unboxed_value_group)
    }
}

impl StmtVisitor<bool> for Analyzer {
    #[allow(unused_variables)]
    fn visit_import(&mut self, s: &Import) -> bool {
        true
    }

    fn visit_foreign_stub(&mut self, s: &ForeignStub) -> bool {
        if !self.prepass_flag {
            return true;
        }

        let source_copy = self.source_str.clone();
        let stub_name = s.get_name_token().to_lexeme_str(source_copy.as_str()).unwrap_or("");

        let stub_ret_type = s.get_result_type().typename();
        let stub_ret_type_id = self.record_type(stub_ret_type);

        let mut stub_param_types = Vec::<i32>::new();
        let stub_arity = s.get_params().len() as i32;

        for param in s.get_params() {
            let param_type_name = param.get_typing().typename();
            let param_type_id = self.record_type(param_type_name.clone());

            self.record_name_info(
                param.get_name_token().to_lexeme_str(source_copy.as_str()).unwrap(),
                SemanticNote::DataValue(param_type_id, ValueCategoryTag::Identity),
                RecordInfoMode::Local
            );
            
            stub_param_types.push(param_type_id);
        }

        if !self.record_name_info(
            stub_name,
            SemanticNote::Callable(stub_param_types, stub_ret_type_id, stub_arity), RecordInfoMode::Global
        ) {
            let redef_stub_msg = format!("Invalid redeclaration of foreign stub '{stub_name}'");
            self.report_plain_error(redef_stub_msg.as_str());

            return false;
        }

        true
    }

    fn visit_function_decl(&mut self, s: &FunctionDecl) -> bool {
        let source_copy = self.source_str.clone();
        let fun_name = s.get_name_token().to_lexeme_str(source_copy.as_str()).unwrap_or("");
        
        let fun_ret_type = s.get_result_type().typename();
        let ret_type_id = self.record_type(fun_ret_type);
        let fun_arity = s.get_params().len() as i32;

        if !self.prepass_flag {
            self.scopes.enter_scope(fun_name);

            for param in s.get_params() {
                let param_type_name = param.get_typing().typename();
                let param_type_id = self.record_type(param_type_name.clone());

                self.record_name_info(
                    param.get_name_token().to_lexeme_str(source_copy.as_str()).unwrap(),
                    SemanticNote::DataValue(param_type_id, ValueCategoryTag::Identity),
                    RecordInfoMode::Local
                );
            }

            if !s.get_body().accept_visitor(self) {
                self.scopes.leave_scope();
                return false;
            }

            self.scopes.leave_scope();
        } else {
            let mut fun_param_types = Vec::<i32>::new();

            for param in s.get_params() {
                let param_type_name = param.get_typing().typename();
                let param_type_id = self.record_type(param_type_name.clone());

                fun_param_types.push(param_type_id);
            }

            if !self.record_name_info(fun_name, SemanticNote::Callable(fun_param_types, ret_type_id, fun_arity), RecordInfoMode::Global) {
                let redef_fun_msg = format!("Invalid redeclaration of procedure '{fun_name}'");
                self.report_plain_error(redef_fun_msg.as_str());

                return false;
            }
        }

        true
    }

    fn visit_field_decl(&mut self, s: &FieldDecl) -> bool {
        if self.prepass_flag {
            let src_copy = self.source_str.clone();
            let field_typename = s.get_type().typename();
            let field_type_id = self.record_type(field_typename.clone());
            let field_name_str = s.get_name_token().to_lexeme_str(&src_copy).unwrap_or("");

            println!("recording field '{field_name_str}'...");
            self.record_name_info(field_name_str, SemanticNote::DataValue(field_type_id, ValueCategoryTag::Identity), RecordInfoMode::Member);
        }

        true
    }

    fn visit_constructor_decl(&mut self, s: &ConstructorDecl) -> bool {
        let ctor_class_id = self.current_class_id;
        let ctor_class_name_opt = self.type_table.get_mut(&ctor_class_id);

        if ctor_class_name_opt.is_none() {
            self.report_plain_error("Unreachable case- No associated class was found for a constructor.");
            return false;
        }

        let ctor_access_mod = self.current_class_mod;
        let ctor_class_name = ctor_class_name_opt.unwrap().clone();

        if self.prepass_flag {
            let mut ctor_param_type_ids = Vec::<i32>::new();
            let ctor_arity = s.get_params().len() as i32;

            for param in s.get_params() {
                let param_type_name = param.get_typing().typename();
                let param_type_id = self.record_type(param_type_name.clone());

                ctor_param_type_ids.push(param_type_id);
            }

            if !self.record_class_member_info(ctor_class_id, ctor_class_name.as_str(), ctor_access_mod, SemanticNote::Constructor(ctor_param_type_ids.clone(), ctor_class_id, ctor_arity)) {
                let duped_ctor_msg = format!("Cannot redeclare the '{}' constructor", ctor_class_name.as_str());
                self.report_plain_error(&duped_ctor_msg);

                return false;
            }

            if ctor_access_mod == AccessFlag::Hidden || !self.record_name_info(ctor_class_name.as_str(), SemanticNote::Callable(ctor_param_type_ids, ctor_class_id, ctor_arity), RecordInfoMode::Global) {
                let top_ctor_decl_fail_msg = format!("Failed to record constructor at top-level for class '{}'\n\tNote: constructors must be public.", ctor_class_name.as_str());
                self.report_plain_error(&top_ctor_decl_fail_msg);

                return false;
            }
        } else {
            let source_copy = self.source_str.clone();
            self.scopes.enter_scope(ctor_class_name.as_str());

            // println!("for class of type ID {}... processing ctor", self.current_class_id);

            for param in s.get_params() {
                let param_type_name = param.get_typing().typename();
                let param_type_id = self.record_type(param_type_name.clone());

                self.record_name_info(
                    param.get_name_token().to_lexeme_str(source_copy.as_str()).unwrap(),
                    SemanticNote::DataValue(param_type_id, ValueCategoryTag::Identity),
                    RecordInfoMode::Local
                );
            }

            if !s.get_body().accept_visitor(self) {
                self.scopes.leave_scope();
                return false;
            }

            self.scopes.leave_scope();
        }

        true
    }

    fn visit_method_decl(&mut self, s: &MethodDecl) -> bool {
        let source_copy = self.source_str.clone();
        let met_name = s.get_name_token().to_lexeme_str(source_copy.as_str()).unwrap_or("");
        
        let met_ret_type = s.get_result_type().typename();
        let met_ret_type_id = self.record_type(met_ret_type);
        let met_arity = s.get_params().len() as i32;

        if !self.prepass_flag {
            self.scopes.enter_scope(met_name);
            // println!("for class of type ID {}... processing method", self.current_class_id);

            for param in s.get_params() {
                let param_type_name = param.get_typing().typename();
                let param_type_id = self.record_type(param_type_name.clone());

                self.record_name_info(
                    param.get_name_token().to_lexeme_str(source_copy.as_str()).unwrap(),
                    SemanticNote::DataValue(param_type_id, ValueCategoryTag::Identity),
                    RecordInfoMode::Local
                );
            }

            if !s.get_body().accept_visitor(self) {
                self.scopes.leave_scope();
                return false;
            }

            self.scopes.leave_scope();
        } else {
            let mut met_param_types = Vec::<i32>::new();

            for param in s.get_params() {
                let param_type_name = param.get_typing().typename();
                let param_type_id = self.record_type(param_type_name.clone());

                met_param_types.push(param_type_id);
            }

            if !self.record_name_info(met_name, SemanticNote::Method(met_param_types, met_ret_type_id, met_arity, self.current_class_id), RecordInfoMode::Member) {
                let redef_fun_msg = format!("Invalid redeclaration of method '{met_name}'");
                self.report_plain_error(redef_fun_msg.as_str());

                return false;
            }
        }

        true
    }

    fn visit_class_decl(&mut self, s: &ClassDecl) -> bool {
        let class_name = s.get_class_type().typename();
        let class_type_id = self.record_type(class_name.clone());

        if self.prepass_flag {
            if !self.record_new_class_bp(class_type_id) {
                let temp_line_no = self.temp_token.line_no;
                let redecl_class_msg = format!("Cannot redeclare structure of class '{}' at source [ln. {}]", class_name.as_str(), temp_line_no);
                self.report_plain_error(&redecl_class_msg);

                return false;
            }
        } else {
            self.set_preprocess_decls_flag();
            self.set_current_class_id(class_type_id);

            for (member_stmt, member_mod) in s.get_members() {
                self.update_current_class_mod(*member_mod);

                if !member_stmt.as_ref().accept_visitor(self) {
                    return false;
                }
            }

            self.clear_preprocess_decls_flag();
            // println!("processing class of type ID {class_type_id}...");

            for (member_stmt, member_mod) in s.get_members() {
                self.update_current_class_mod(*member_mod);

                if !member_stmt.as_ref().accept_visitor(self) {
                    return false;
                }
            }

            self.set_current_class_id(-1);
        }

        true
    }

    fn visit_block(&mut self, s: &Block) -> bool {
        for stmt in s.get_items() {
            if !stmt.accept_visitor(self) {
                return false;
            }
        }

        true
    }

    fn visit_variable_decl(&mut self, s: &VariableDecl) -> bool {
        let source_copy_fml = self.source_str.clone();
        let var_name_token_ref = s.get_name_token();
        let var_name_lexeme = var_name_token_ref.to_lexeme_str(source_copy_fml.as_str()).unwrap_or("");
        let var_name_line_no = var_name_token_ref.line_no;

        let var_type_name = s.get_typing().typename();
        let var_type_id = self.record_type(var_type_name.clone());
        
        if self.class_blueprints.try_get_entry_mut(var_type_id).is_none() {
            if !self.record_name_info(
                var_name_lexeme,
                SemanticNote::DataValue(var_type_id, ValueCategoryTag::Identity),
                RecordInfoMode::Local
            ) {
                let redef_var_msg = format!("Invalid redeclaration of non-instance variable '{var_name_lexeme}'");
                self.report_plain_error(redef_var_msg.as_str());
                
                return false;
            }
        } else if !self.record_name_info(
            var_name_lexeme,
            SemanticNote::ClassEntity(var_type_id, ValueCategoryTag::Identity),
            RecordInfoMode::Local
        ) {
            let redef_var_msg = format!("Invalid redeclaration of '{}' instance variable '{var_name_lexeme}'", var_type_name.as_str());
            self.report_plain_error(redef_var_msg.as_str());

            return false;
        }

        let init_info = s.get_init_expr().accept_visitor_sema(self);

        let init_type_id = if let SemanticNote::DataValue(type_id, _) = init_info {
            type_id
        } else { -1 };

        if var_type_id != init_type_id {
            let bad_rhs_msg = format!("Cannot set variable '{var_name_lexeme}' at Ln. {var_name_line_no} to the RHS expression- The RHS value type was mismatched (type-id {init_type_id}).");
            self.report_culprit_error(var_name_token_ref, bad_rhs_msg.as_str());

            return false;
        }

        true
    }

    /// # TODO
    /// Add a check for requiring bool condition types.
    fn visit_if(&mut self, s: &If) -> bool {
        if s.get_check().accept_visitor_sema(self).is_dud() {
            return false;
        }

        if !s.get_truthy_body().accept_visitor(self) {
            return false;
        }

        if !s.get_falsy_body().accept_visitor(self) {
            return false;
        }

        true
    }

    fn visit_while(&mut self, s: &While) -> bool {
        if s.get_check().accept_visitor_sema(self).is_dud() {
            return false;
        }

        if !s.get_body().accept_visitor(self) {
            return false;
        }

        true
    }

    /// # TODO
    /// Add checks for return types against their parent function return type.
    fn visit_return(&mut self, s: &Return) -> bool {
        if s.get_result().accept_visitor_sema(self).is_dud() {
            return false;
        }

        true
    }

    fn visit_expr_stmt(&mut self, s: &ExprStmt) -> bool {
        if s.get_inner().accept_visitor_sema(self).is_dud() {
            return false;
        }

        true
    }
}
