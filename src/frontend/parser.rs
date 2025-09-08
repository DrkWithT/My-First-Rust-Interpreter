use std::collections::{HashMap, VecDeque};

use crate::frontend::ast::*;
use crate::frontend::lexer::Lexer;
use crate::frontend::token::{Token, TokenType};
use crate::semantics::types::*;
use crate::token_from;

pub type ASTDecls = Vec<Box<dyn Stmt>>;
pub type ParseResult = (Option<ASTDecls>, VecDeque<String>);

pub struct Parser<'pl_1> {
    tokenizer: Lexer<'pl_1>,
    next_sources: VecDeque<String>,
    previous: Token,
    current: Token,
    error_count: i32,
    parse_error_max: i32,
}

impl<'pl_2> Parser<'pl_2> {
    pub fn new(tokenizer: Lexer<'pl_2>) -> Self {
        Self {
            tokenizer,
            next_sources: VecDeque::<String>::new(),
            previous: token_from!(TokenType::Unknown, 0, 1, 1, 1),
            current: token_from!(TokenType::Unknown, 0, 1, 1, 1),
            error_count: 0,
            parse_error_max: 5,
        }
    }

    fn previous(&self) -> &Token {
        &self.previous
    }

    fn current(&self) -> &Token {
        &self.current
    }

    fn at_eof(&self) -> bool {
        self.current().tag == TokenType::Eof
    }

    fn advance(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Token {        loop {
            let temp = self.tokenizer.lex_next(items);

            let temp_tag = temp.tag;

            match temp_tag {
                TokenType::Spaces | TokenType::Comment => {
                    continue;
                }
                _ => {
                    return temp;
                }
            }
        }
    }

    fn match_here<const N: usize>(&self, picks: [TokenType; N]) -> bool {
        picks.contains(&self.current().tag)
    }

    fn recover_and_report(&mut self, msg: &str, items: &'pl_2 HashMap<String, TokenType>) {
        if self.error_count > self.parse_error_max {
            return;
        }

        let culprit_line = self.current().line_no;
        let culprit_col = self.current().col_no;
        let culprit_lexeme_opt = self.current().to_lexeme_str(self.tokenizer.view_source());

        if culprit_lexeme_opt.is_none() {
            return;
        }

        let culprit_lexeme =
            culprit_lexeme_opt.expect("Unexpected invalid lexeme, out of source bound!");

        println!(
            "Syntax error #{}:\nCulprit: '{}' at [{}:{}]\nReason: {}",
            self.error_count, culprit_lexeme, culprit_line, culprit_col, msg
        );

        self.error_count += 1;

        while !self.at_eof() {
            if self.match_here([TokenType::Keyword]) {
                break;
            }

            self.consume_any(items);
        }
    }

    fn consume_any(&mut self, items: &'pl_2 HashMap<String, TokenType>) {
        self.previous = self.current;
        self.current = self.advance(items);
    }

    fn consume_of<const N: usize>(&mut self, picks: [TokenType; N], items: &'pl_2 HashMap<String, TokenType>) -> bool {
        if self.match_here(picks) {
            self.consume_any(items);
            return true;
        }

        self.recover_and_report("Unexpected token!", items);

        false
    }

    fn parse_type(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Box<dyn TypeKind> {
        self.consume_any(items);
        let typename_lexeme = self
            .previous()
            .to_lexeme_str(self.tokenizer.view_source())
            .unwrap_or("");

        match typename_lexeme {
            "bool" => Box::new(PrimitiveInfo::new(PrimitiveTag::Boolean)),
            "char" => Box::new(PrimitiveInfo::new(PrimitiveTag::Char)),
            "int" => Box::new(PrimitiveInfo::new(PrimitiveTag::Integer)),
            "float" => Box::new(PrimitiveInfo::new(PrimitiveTag::Floating)),
            "varchar" => Box::new(PrimitiveInfo::new(PrimitiveTag::Varchar)),
            "any" => Box::new(PrimitiveInfo::new(PrimitiveTag::Any)),
            _ => Box::new(ClassInfo::new(String::from(typename_lexeme))),
        }
    }

    fn parse_primitive(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Expr>> {
        if self.match_here([TokenType::ParenOpen]) {
            // println!("parse_primitive ((expr))...");
            self.consume_any(items);
            let parenthesized_expr = self.parse_compare(items);
            self.consume_of([TokenType::ParenClose], items);

            return parenthesized_expr;
        }

        let token_copy = *self.current();

        if !self.consume_of([
            TokenType::Identifier,
            TokenType::LiteralBool,
            TokenType::LiteralChar,
            TokenType::LiteralInt,
            TokenType::LiteralFloat,
            TokenType::LiteralVarchar,
        ], items) {
            // println!("parse_primitive ERROR :(");
            return None;
        }

        Some(Box::new(Primitive::new(token_copy)))
    }

    // fn parse_atom(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Expr>> {
        // TODO ...
    // }

    fn parse_access(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Expr>> {
        // println!("parse_access --> parse_primitive");
        let lhs_opt = self.parse_primitive(items);

        lhs_opt.as_ref()?;

        let mut lhs = lhs_opt.unwrap();

        while !self.at_eof() {
            if !self.match_here([TokenType::OpAccess]) {
                // println!("stopped parse_access at token of: {}", self.current().to_info_str());
                break;
            }

            self.consume_any(items);

            // println!("parse_access --> parse_primitive");
            let rhs_box = self.parse_primitive(items);

            rhs_box.as_ref()?;

            let rhs = rhs_box.unwrap();

            lhs = Box::new(Binary::new(lhs, rhs, OperatorTag::Access));
        }

        Some(lhs)
    }

    fn parse_call(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Expr>> {
        // println!("parse_call --> parse_access");
        let callee_opt = self.parse_access(items);

        callee_opt.as_ref()?;

        let callee_expr = callee_opt.unwrap();

        if !self.match_here([TokenType::ParenOpen]) {
            return Some(callee_expr);
        }

        self.consume_any(items);

        let mut calling_args = Vec::<Box<dyn Expr>>::new();

        if self.match_here([TokenType::ParenClose]) {
            self.consume_any(items);
            return Some(Box::new(Call::new(callee_expr, calling_args)));
        }

        let first_arg_opt = self.parse_compare(items);

        first_arg_opt.as_ref()?;

        calling_args.push(first_arg_opt.unwrap());

        while !self.at_eof() {
            if self.match_here([TokenType::ParenClose]) {
                self.consume_any(items);
                break;
            }

            self.consume_of([TokenType::Comma], items);

            let next_arg_opt = self.parse_compare(items);

            next_arg_opt.as_ref()?;

            calling_args.push(next_arg_opt.unwrap());
        }

        Some(Box::new(Call::new(callee_expr, calling_args)))
    }

    fn parse_unary(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Expr>> {
        if !self.match_here([TokenType::OpMinus]) {
            // println!("parse_unary (no negation)...");
            return self.parse_call(items);
        }

        let current_tag = self.current().tag;
        let prefixed_op = match current_tag {
            TokenType::OpMinus => OperatorTag::Negate,
            _ => OperatorTag::Noop,
        };

        self.consume_any(items);

        // println!("parse_unary (negation) --> parse_call");
        let temp_inner_opt = self.parse_call(items);

        temp_inner_opt.as_ref()?;

        Some(Box::new(Unary::new(temp_inner_opt.unwrap(), prefixed_op)))
    }

    fn parse_factor(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Expr>> {
        // println!("parse_factor...");
        let lhs_opt = self.parse_unary(items);

        lhs_opt.as_ref()?;

        let mut lhs = lhs_opt.unwrap();

        while !self.at_eof() {
            if !self.match_here([TokenType::OpTimes, TokenType::OpSlash]) {
                break;
            }

            let temp_tag = self.current().tag;

            let temp_op = if temp_tag == TokenType::OpTimes {
                OperatorTag::Times
            } else {
                OperatorTag::Slash
            };

            self.consume_any(items);

            let rhs_opt = self.parse_unary(items);

            rhs_opt.as_ref()?;

            lhs = Box::new(Binary::new(lhs, rhs_opt.unwrap(), temp_op));
        }

        Some(lhs)
    }

    fn parse_term(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Expr>> {
        // println!("parse_term...");
        let lhs_opt: Option<Box<dyn Expr>> = self.parse_factor(items);

        lhs_opt.as_ref()?;

        let mut lhs = lhs_opt.unwrap();

        while !self.at_eof() {
            if !self.match_here([TokenType::OpPlus, TokenType::OpMinus]) {
                break;
            }

            let temp_tag = self.current().tag;

            let temp_op = if temp_tag == TokenType::OpPlus {
                OperatorTag::Plus
            } else {
                OperatorTag::Minus
            };

            self.consume_any(items);

            let rhs_opt = self.parse_factor(items);

            rhs_opt.as_ref()?;

            lhs = Box::new(Binary::new(lhs, rhs_opt.unwrap(), temp_op));
        }

        Some(lhs)
    }

    fn parse_equality(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Expr>> {
        // println!("parse_equality...");
        let lhs_opt: Option<Box<dyn Expr>> = self.parse_term(items);

        lhs_opt.as_ref()?;

        let mut lhs = lhs_opt.unwrap();

        while !self.at_eof() {
            if !self.match_here([TokenType::OpEquality, TokenType::OpInequality]) {
                break;
            }

            let temp_tag = self.current().tag;

            let temp_op = if temp_tag == TokenType::OpEquality {
                OperatorTag::Equality
            } else {
                OperatorTag::Inequality
            };

            self.consume_any(items);

            let rhs_opt = self.parse_term(items);

            rhs_opt.as_ref()?;

            lhs = Box::new(Binary::new(lhs, rhs_opt.unwrap(), temp_op));
        }

        Some(lhs)
    }

    fn parse_compare(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Expr>> {
        // println!("parse_compare...");
        let lhs_opt: Option<Box<dyn Expr>> = self.parse_equality(items);

        lhs_opt.as_ref()?;

        let mut lhs = lhs_opt.unwrap();

        while !self.at_eof() {
            if !self.match_here([TokenType::OpLessThan, TokenType::OpGreaterThan]) {
                break;
            }

            let temp_tag = self.current().tag;

            let temp_op = if temp_tag == TokenType::OpLessThan {
                OperatorTag::LessThan
            } else {
                OperatorTag::GreaterThan
            };

            self.consume_any(items);

            let rhs_opt = self.parse_equality(items);

            rhs_opt.as_ref()?;

            lhs = Box::new(Binary::new(lhs, rhs_opt.unwrap(), temp_op));
        }

        Some(lhs)
    }

    fn parse_assign(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Expr>> {
        // println!("parse_assign -> parse_access");
        let lhs_opt = self.parse_access(items);

        lhs_opt.as_ref()?;

        if !self.match_here([TokenType::OpAssign]) {
            return Some(lhs_opt.unwrap());
        }

        self.consume_any(items);

        // println!("parse_assign -> parse_compare");
        let rhs_opt = self.parse_compare(items);

        rhs_opt.as_ref()?;

        Some(Box::new(Binary::new(
            lhs_opt.unwrap(),
            rhs_opt.unwrap(),
            OperatorTag::Assign,
        )))
    }

    fn parse_variable_decl(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Stmt>> {
        // println!("parse_variable_decl...");
        self.consume_of([TokenType::Keyword], items);

        let var_name = *self.current();

        self.consume_of([TokenType::Identifier], items);
        self.consume_of([TokenType::Colon], items);

        let var_type_box = self.parse_type(items);

        self.consume_of([TokenType::OpAssign], items);

        // println!("parse_variable_decl --> parse_compare");
        let var_init_expr_opt = self.parse_compare(items);

        var_init_expr_opt.as_ref()?;

        let var_init_expr = var_init_expr_opt.unwrap();

        if !self.consume_of([TokenType::Semicolon], items) {
            self.recover_and_report("Expected ';' .", items);
            return None;
        }

        Some(Box::new(VariableDecl::new(
            var_name,
            var_type_box,
            var_init_expr,
        )))
    }

    fn parse_if(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Stmt>> {
        self.consume_any(items);

        let conds_opt = self.parse_compare(items);

        conds_opt.as_ref()?;

        let conds_expr = conds_opt.unwrap();

        let truthy_body_opt = self.parse_block(items);

        truthy_body_opt.as_ref()?;

        let truthy_body = truthy_body_opt.unwrap();

        if self
            .current()
            .to_lexeme_str(self.tokenizer.view_source())
            .expect("")
            == "else"
        {
            self.consume_any(items);

            let falsy_body_opt = self.parse_block(items);

            falsy_body_opt.as_ref()?;

            return Some(Box::new(If::new(
                truthy_body,
                falsy_body_opt.unwrap(),
                conds_expr,
            )));
        }

        let dud_falsy_body = Box::new(Block::new(Vec::new()));

        Some(Box::new(If::new(truthy_body, dud_falsy_body, conds_expr)))
    }

    fn parse_while(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Stmt>> {
        self.consume_any(items);

        let check_expr = self.parse_compare(items);

        let body_stmt = self.parse_block(items);

        Some(Box::new(While::new(
            check_expr.unwrap(),
            body_stmt.unwrap(),
        )))
    }

    fn parse_return(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Stmt>> {
        self.consume_any(items);

        let result_expr_opt = self.parse_compare(items);

        result_expr_opt.as_ref()?;

        if !self.consume_of([TokenType::Semicolon], items) {
            self.recover_and_report("Expected ';' .", items);
            return None;
        }

        Some(Box::new(Return::new(result_expr_opt.unwrap())))
    }

    fn parse_expr_stmt(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Stmt>> {
        let inner_expr_opt = self.parse_assign(items);

        inner_expr_opt.as_ref()?;

        if !self.consume_of([TokenType::Semicolon], items) {
            self.recover_and_report("Expected ';' .", items);
            return None;
        }

        Some(Box::new(ExprStmt::new(inner_expr_opt.unwrap())))
    }

    fn parse_nestable(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Stmt>> {
        let keyword = self
            .current()
            .to_lexeme_str(self.tokenizer.view_source())
            .expect("");

        match keyword {
            "let" => self.parse_variable_decl(items),
            "if" => self.parse_if(items),
            "while" => self.parse_while(items),
            "return" => self.parse_return(items),
            _ => self.parse_expr_stmt(items),
        }
    }

    fn parse_block(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Stmt>> {
        self.consume_of([TokenType::BraceOpen], items);

        let mut stmts = Vec::<Box<dyn Stmt>>::new();

        while !self.at_eof() {
            if self.match_here([TokenType::BraceClose]) {
                self.consume_any(items);
                break;
            }

            let next_stmt_opt = self.parse_nestable(items);

            next_stmt_opt.as_ref()?;

            stmts.push(next_stmt_opt.unwrap());
        }

        Some(Box::new(Block::new(stmts)))
    }

    fn parse_import(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Stmt>> {
        self.consume_any(items);

        self.consume_of([TokenType::Identifier], items);
        let temp_target_token = *self.previous();

        if !self.consume_of([TokenType::Semicolon], items) {
            self.recover_and_report("Expected ';' .", items);
            return None;
        }

        let temp_target_name = temp_target_token.to_lexeme_str(self.tokenizer.view_source()).unwrap_or("");
        self.next_sources.push_front(String::from(temp_target_name));

        Some(Box::new(Import::new(temp_target_token)))
    }

    fn parse_foreign_stub(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Stmt>> {
        self.consume_any(items);

        let stub_name_token = *self.current();
        self.consume_of([TokenType::Identifier], items);

        let stub_params = self.parse_function_params(items);

        self.consume_of([TokenType::Colon], items);
        let stub_ret_type_box = self.parse_type(items);

        if !self.consume_of([TokenType::Semicolon], items) {
            self.recover_and_report("Expected ';' .", items);
            return None;
        }

        Some(Box::new(ForeignStub::new(
            stub_name_token,
            stub_params,
            stub_ret_type_box
        )))
    }

    fn parse_function_decl(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Stmt>> {
        self.consume_of([TokenType::Keyword], items);

        let func_name_token = *self.current();
        self.consume_of([TokenType::Identifier], items);

        let func_params = self.parse_function_params(items);

        self.consume_of([TokenType::Colon], items);
        let func_type_box = self.parse_type(items);

        let func_body_opt = self.parse_block(items);

        func_body_opt.as_ref()?;

        Some(Box::new(FunctionDecl::new(
            func_name_token,
            func_params,
            func_type_box,
            func_body_opt.unwrap(),
        )))
    }

    fn parse_field_decl(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Stmt>> {
        self.consume_of([TokenType::Keyword], items);

        let field_name_token = *self.current();

        self.consume_of([TokenType::Identifier], items);
        self.consume_of([TokenType::Colon], items);

        let field_typing = self.parse_type(items);

        self.consume_of([TokenType::Semicolon], items);

        Some(Box::new(FieldDecl::new(
            field_name_token, field_typing
        )))
    }

    fn parse_constructor_decl(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Stmt>> {
        self.consume_of([TokenType::Keyword], items);

        let ctor_params = self.parse_function_params(items);

        let ctor_body = self.parse_block(items);

        ctor_body.as_deref()?;

        Some(Box::new(ConstructorDecl::new(
            ctor_params,
            ctor_body.unwrap(),
        )))
    }

    fn parse_method_decl(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Stmt>> {
        self.consume_of([TokenType::Keyword], items);

        let method_name_token = *self.current();
        self.consume_of([TokenType::Identifier], items);

        let method_params = self.parse_function_params(items);

        self.consume_of([TokenType::Colon], items);
        let method_type_box = self.parse_type(items);

        let method_body_opt = self.parse_block(items);

        method_body_opt.as_ref()?;

        Some(Box::new(MethodDecl::new(
            method_name_token,
            method_params,
            method_type_box,
            method_body_opt.unwrap(),
        )))
    }

    fn parse_class_member(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<ClassMemberDecl> {
        self.consume_of([TokenType::Keyword], items);

        let access_modify_word = self.previous().to_lexeme_str(self.tokenizer.view_source()).unwrap_or("private");
        let access_modify_enum = if access_modify_word == "public" { ClassAccess::Public } else { ClassAccess::Private };

        let hint_keyword = self.current().to_lexeme_str(self.tokenizer.view_source()).unwrap_or("");

        let class_decl_opt = match hint_keyword {
            "let" => self.parse_field_decl(items),
            "ctor" => self.parse_constructor_decl(items),
            "met" => self.parse_method_decl(items),
            _ => {
                self.recover_and_report("Invalid class member declaration- Only let, ctor, and met are valid for fields, a constructor, and methods are valid.", items);
                None
            },
        };

        class_decl_opt.as_deref()?;

        Some((
            class_decl_opt.unwrap(),
            access_modify_enum
        ))
    }

    fn parse_class_decl(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Stmt>> {
        self.consume_of([TokenType::Keyword], items);

        let class_typename = self.parse_type(items);
        let mut class_members = Vec::<ClassMemberDecl>::new();

        self.consume_of([TokenType::BraceOpen], items);

        while !self.at_eof() {
            if self.match_here([TokenType::BraceClose]) {
                self.consume_any(items);
                break;
            }

            let temp_member_stmt = self.parse_class_member(items);
            temp_member_stmt.as_ref()?;

            class_members.push(temp_member_stmt.unwrap());
        }

        Some(Box::new(ClassDecl::new(class_members, class_typename)))
    }

    fn parse_param_decl(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> ParamDecl {
        let name_token = *self.current();
        self.consume_of([TokenType::Identifier], items);
        self.consume_of([TokenType::Colon], items);

        let typing_box = self.parse_type(items);

        ParamDecl::new(name_token, typing_box)
    }

    fn parse_function_params(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Vec<ParamDecl> {
        self.consume_of([TokenType::ParenOpen], items);

        let mut parameters = Vec::<ParamDecl>::new();

        if self.match_here([TokenType::ParenClose]) {
            self.consume_any(items);
            return parameters;
        }

        parameters.push(self.parse_param_decl(items));

        while !self.at_eof() {
            if self.match_here([TokenType::ParenClose]) {
                self.consume_any(items);
                break;
            }

            self.consume_of([TokenType::Comma], items);

            parameters.push(self.parse_param_decl(items));
        }

        parameters
    }

    fn parse_top_decl(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> Option<Box<dyn Stmt>> {
        let start_word = self.current().to_lexeme_str(self.tokenizer.view_source()).unwrap_or("");

        match start_word {
            "import" => self.parse_import(items),
            "foreign" => self.parse_foreign_stub(items),
            "fun" => self.parse_function_decl(items),
            "class" => self.parse_class_decl(items),
            _ => None
        }
    }

    pub fn reset_with(&mut self, next_source: &'pl_2 str) {
        self.tokenizer.reset_with(next_source);
        self.next_sources.clear();

        self.current = Token {
            tag: TokenType::Unknown,
            start: 0,
            length: 1,
            line_no: 0,
            col_no: 0
        };
        self.previous = *self.current();

        self.error_count = 0;
    }

    pub fn parse_file(&mut self, items: &'pl_2 HashMap<String, TokenType>) -> ParseResult {
        self.consume_any(items);

        let mut all_top_stmts = ASTDecls::new();

        while !self.at_eof() {
            let func_decl_opt = self.parse_top_decl(items);

            if func_decl_opt.is_none() {
                break;
            }

            all_top_stmts.push(func_decl_opt.unwrap());
        }

        let mut temp_src_targets = VecDeque::<String>::new();

        std::mem::swap(&mut temp_src_targets, &mut self.next_sources);

        if self.error_count == 0 {
            (Some(all_top_stmts), temp_src_targets)
        } else {
            (None, VecDeque::<String>::default())
        }
    }
}
