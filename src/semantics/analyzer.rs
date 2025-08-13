use std::collections::HashMap;

use crate::frontend::token::*;
use crate::frontend::ast::*;
use crate::semantics::scope::*;
use crate::semantics::types::OperatorTag;
use crate::semantics::types::ValueCategoryTag;
use crate::compiler::driver::FullProgramAST;

const BOOLEAN_TYPE_ID_N: i32 = 0;
const INTEGER_TYPE_ID_N: i32 = 1;
const FLOATING_TYPE_ID_N: i32 = 2;
const ANY_TYPE_ID_N: i32 = 3;

#[repr(i8)]
#[derive(Clone, Copy, PartialEq)]
enum RecordInfoMode {
    AsVariable,
    AsCallable,
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
 # ABOUT
 * Contains the implementation of the semantic checker. This logic enforces type-safety and value semantics.
 # EXAMPLES
 Here are some illegal cases of code:
 * `1 = 42;` is an invalid assignment since the LHS has no identity.
 * `a = a + 1;` is an undeclared variable since a has no declaration.
 * `let a: int = 42;`, `a = 3.1415` is invalid because the types are mismatched.
 # CAVEATS
 No support for arrays or strings exists yet.
 */
pub struct Analyzer<'al1> {
    type_table: HashMap<i32, String>,
    temp_token: Token,
    scopes: ScopeStack,
    source_str: &'al1 str,
}

impl<'al2> Analyzer<'al2> {
    pub fn new(source_view: &'al2 str) -> Self {
        let mut temp_type_table = HashMap::<i32, String>::new();
        temp_type_table.insert(ANY_TYPE_ID_N, String::from("any"));
        temp_type_table.insert(BOOLEAN_TYPE_ID_N, String::from("bool"));
        temp_type_table.insert(INTEGER_TYPE_ID_N, String::from("int"));
        temp_type_table.insert(FLOATING_TYPE_ID_N, String::from("float"));

        Self {
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
        }
    }

    pub fn reset_with(&mut self, source_view_next: &'al2 str) {
        self.temp_token = Token {
            tag: TokenType::Unknown,
            start: 0,
            length: 1,
            line_no: 0,
            col_no: 0
        };
        self.scopes.reset();
        self.source_str = source_view_next;
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

    /// ### TODO
    /// Add support for function stub declarations which provide type info for native funcs.
    fn lookup_name_info(&self, name: &str) -> SemanticNote {
        self.scopes.lookup_name_info(name)
    }

    fn record_name_info(&mut self, name: &str, info: SemanticNote, mode: RecordInfoMode) -> bool {
        match mode {
            RecordInfoMode::AsVariable => self.scopes.current_scope_mut().unwrap().try_set_entry(name, info),
            RecordInfoMode::AsCallable => self.scopes.global_scope_mut().unwrap().try_set_entry(name, info),
        }
    }

    fn report_plain_error(&self, msg: &str) {
        eprintln!("SemaError:\n{msg}");
    }

    fn report_culprit_error(&self, culprit: &Token, msg: &str) {
        eprintln!("SemaError at [Ln {}, Col {}]:\nCulprit token: '{}'\n{}", culprit.line_no, culprit.col_no, culprit.to_lexeme_str(self.source_str).unwrap_or("..."), msg);
    }

    pub fn check_source_unit(&mut self, ast: &FullProgramAST) -> bool {
        for fun_declaration in ast {
            if !fun_declaration.accept_visitor(self) {
                return false;
            }
        }

        true
    }
}

impl<'evl2> ExprVisitor<'evl2, SemanticNote> for Analyzer<'_> {
    fn visit_primitive(&'_ mut self, e: &Primitive) -> SemanticNote {
        let literal_lexeme = e.get_token().to_lexeme_str(self.source_str).unwrap_or("");
        let literal_tag = e.get_token().tag;
        self.temp_token = *e.get_token();

        match literal_tag {
            TokenType::LiteralBool => {
                SemanticNote::DataValue(BOOLEAN_TYPE_ID_N, ValueCategoryTag::Temporary)
            },
            TokenType::LiteralInt => {
                SemanticNote::DataValue(INTEGER_TYPE_ID_N, ValueCategoryTag::Temporary)
            },
            TokenType::LiteralFloat => {
                SemanticNote::DataValue(FLOATING_TYPE_ID_N, ValueCategoryTag::Temporary)
            },
            TokenType::Identifier => {
                self.lookup_name_info(literal_lexeme)
            },
            _ => {
                SemanticNote::Dud
            },
        }
    }

    /// # TODO
    /// Refactor this call-checking logic to avoid extra String allocations... This is probably very slow.
    fn visit_call(&mut self, e: &Call) -> SemanticNote {
        let callee_info = e.get_callee().accept_visitor_sema(self);
        let callee_token = self.temp_token;

        if callee_info.is_dud() {
            self.report_culprit_error(&callee_token, "The callee name is likely undeclared, did you declare fun <name> before?");
            return SemanticNote::Dud;
        }

        let callable_info_opt = callee_info.try_unbox_callable_info();

        if callable_info_opt.is_none() {
            self.report_culprit_error(&callee_token, "The callee name is a non-callable type- Not a 'fun' procedure or lambda.");
            return SemanticNote::Dud;
        }

        let callable_info = callable_info_opt.unwrap();
        
        let callee_lexeme = callee_token.to_lexeme_str(self.source_str).unwrap_or("...");
        let callable_arity = callable_info.2;
        let passed_arity = e.get_args().len() as i32;
        
        if passed_arity != callable_arity {
            let msg_string = format!("For procedure '{callee_lexeme}'- Expected {callable_arity} arguments instead of {passed_arity}.");
            self.report_culprit_error(&self.temp_token, msg_string.as_str());
            
            return SemanticNote::Dud;
        }

        for arg_it in 0..passed_arity {
            let temp_arg_type_id = e.get_args().get(arg_it as usize).unwrap().accept_visitor_sema(self);

            match temp_arg_type_id {
                SemanticNote::DataValue(argv_type_id, _) => {
                    let expected_type_id = *callable_info.0.get(arg_it as usize).unwrap();

                    if expected_type_id != ANY_TYPE_ID_N && expected_type_id != argv_type_id {
                        let mismatch_err_msg = format!("For argument #{arg_it}, a mismatched type was found. Please check the declaration of 'fun {callee_lexeme}'.");

                        self.report_culprit_error(&callee_token, mismatch_err_msg.as_str());
                        return SemanticNote::Dud;
                    }
                },
                _ => {
                    let invalid_arg_msg = format!("For argument {arg_it}, an invalid type was found. Please check the declaration of 'fun {callee_lexeme}'.");
                    self.report_culprit_error(&callee_token, invalid_arg_msg.as_str());

                    return SemanticNote::Dud;
                }
            }
        }
        
        let callable_result_type_id = callable_info.1;

        SemanticNote::DataValue(callable_result_type_id, ValueCategoryTag::Temporary)
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

    fn visit_binary(&mut self, e: &Binary) -> SemanticNote {
        let expr_op = e.get_operator();
        let lhs_info = e.get_lhs().accept_visitor_sema(self);
        let expr_line_no = self.temp_token.line_no;
        let rhs_info = e.get_rhs().accept_visitor_sema(self);

        if expr_op.is_homogeneously_typed() {
            if !check_binary_typing_homogeneously(&lhs_info, &rhs_info) {
                let mismatched_opers_msg = format!("Found mismatched types for {} expression around Ln. {}", expr_op.as_symbol(), expr_line_no);
                self.report_plain_error(mismatched_opers_msg.as_str());

                return SemanticNote::Dud;
            }
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

impl StmtVisitor<bool> for Analyzer<'_> {
    #[allow(unused_variables)]
    fn visit_import(&mut self, s: &Import) -> bool {
        true
    }

    fn visit_foreign_stub(&mut self, s: &ForeignStub) -> bool {
        let stub_name = s.get_name_token().to_lexeme_str(self.source_str).unwrap_or("");

        let stub_ret_type = s.get_result_type().typename();
        let stub_ret_type_id = self.record_type(stub_ret_type);

        let mut stub_param_types = Vec::<i32>::new();
        let stub_arity = s.get_params().len() as i32;

        for param in s.get_params() {
            let param_type_name = param.get_typing().typename();
            let param_type_id = self.record_type(param_type_name.clone());

            self.record_name_info(
                param.get_name_token().to_lexeme_str(self.source_str).unwrap(),
                SemanticNote::DataValue(param_type_id, ValueCategoryTag::Identity),
                RecordInfoMode::AsVariable
            );
            
            stub_param_types.push(param_type_id);
        }

        if !self.record_name_info(
            stub_name,
            SemanticNote::Callable(stub_param_types, stub_ret_type_id, stub_arity), RecordInfoMode::AsCallable
        ) {
            let redef_stub_msg = format!("Invalid redeclaration of procedure '{stub_name}'");
            self.report_plain_error(redef_stub_msg.as_str());

            return false;
        }

        true
    }

    fn visit_function_decl(&mut self, s: &FunctionDecl) -> bool {
        let fun_name = s.get_name_token().to_lexeme_str(self.source_str).unwrap_or("");
        
        let fun_ret_type = s.get_result_type().typename();
        let ret_type_id = self.record_type(fun_ret_type);

        let mut fun_param_types = Vec::<i32>::new();
        let fun_arity = s.get_params().len() as i32;

        self.scopes.enter_scope(fun_name);

        for param in s.get_params() {
            let param_type_name = param.get_typing().typename();
            let param_type_id = self.record_type(param_type_name.clone());

            self.record_name_info(
                param.get_name_token().to_lexeme_str(self.source_str).unwrap(),
                SemanticNote::DataValue(param_type_id, ValueCategoryTag::Identity),
                RecordInfoMode::AsVariable
            );

            fun_param_types.push(param_type_id);
        }

        if !self.record_name_info(fun_name, SemanticNote::Callable(fun_param_types, ret_type_id, fun_arity), RecordInfoMode::AsCallable) {
            let redef_fun_msg = format!("Invalid redeclaration of procedure '{fun_name}'");
            self.report_plain_error(redef_fun_msg.as_str());

            self.scopes.leave_scope();
            return false;
        }

        if !s.get_body().accept_visitor(self) {
            self.scopes.leave_scope();
            return false;
        }

        self.scopes.leave_scope();
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
        let var_name_token_ref = s.get_name_token();
        let var_name_lexeme = var_name_token_ref.to_lexeme_str(self.source_str).unwrap_or("");
        let var_name_line_no = var_name_token_ref.line_no;

        let var_type_name = s.get_typing().typename();
        let var_type_id = self.record_type(var_type_name);

        if !self.record_name_info(
            var_name_lexeme,
            SemanticNote::DataValue(var_type_id, ValueCategoryTag::Identity),
            RecordInfoMode::AsVariable
        ) {
            let redef_var_msg = format!("Invalid redeclaration of variable '{var_name_lexeme}'");
            self.report_plain_error(redef_var_msg.as_str());

            return false;
        }

        let init_info = s.get_init_expr().accept_visitor_sema(self);

        let init_type_id = if let SemanticNote::DataValue(type_id, _) = init_info {
            type_id
        } else { -1 };

        if var_type_id != init_type_id {
            let bad_rhs_msg = format!("Cannot set variable '{var_name_lexeme}' at Ln. {var_name_line_no} to the RHS expression- Its type was mismatched (type-id {init_type_id}).");
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
