use crate::frontend::ast::*;
use crate::frontend::lexer::Lexer;
use crate::frontend::token::{Token, TokenType};
use crate::semantics::types::*;
use crate::token_from;

pub type ASTDecls = Vec<Box<dyn Stmt>>;
pub type ParseResult = Option<ASTDecls>;

pub struct Parser {
    tokenizer: Lexer,
    previous: Token,
    current: Token,
    error_count: i32,
    parse_error_max: i32,
}

impl Parser {
    pub fn new(tokenizer: Lexer) -> Self {
        Self {
            tokenizer,
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

    fn advance(&mut self) -> Token {
        loop {
            let temp = self.tokenizer.next();

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

    fn recover_and_report(&mut self, msg: &str) {
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
            "Syntax error no. {}:\nCulprit: '{}' at [{}:{}]\nReason: {}",
            self.error_count, culprit_lexeme, culprit_line, culprit_col, msg
        );

        self.error_count += 1;

        while !self.at_eof() {
            if self.match_here([TokenType::Keyword]) {
                break;
            }

            self.consume_any();
        }
    }

    fn consume_any(&mut self) {
        self.previous = self.current;
        self.current = self.advance();
    }

    fn consume_of<const N: usize>(&mut self, picks: [TokenType; N]) -> bool {
        if self.match_here(picks) {
            self.consume_any();
            return true;
        }

        self.recover_and_report("Unexpected token!");

        false
    }

    fn parse_type(&mut self) -> Box<dyn TypeKind> {
        self.consume_any();
        let typename_lexeme = self
            .previous()
            .to_lexeme_str(self.tokenizer.view_source())
            .unwrap_or("");

        match typename_lexeme {
            "bool" => Box::new(PrimitiveInfo::new(PrimitiveTag::Boolean)),
            "int" => Box::new(PrimitiveInfo::new(PrimitiveTag::Integer)),
            "float" => Box::new(PrimitiveInfo::new(PrimitiveTag::Floating)),
            _ => Box::new(PrimitiveInfo::new(PrimitiveTag::Unknown)),
        }
    }

    fn parse_primitive(&mut self) -> Option<Box<dyn Expr>> {
        if self.match_here([TokenType::ParenOpen]) {
            self.consume_any();
            let parenthesized_expr = self.parse_compare();
            self.consume_of([TokenType::ParenClose]);

            return parenthesized_expr;
        }

        if !self.consume_of([
            TokenType::LiteralBool,
            TokenType::LiteralInt,
            TokenType::LiteralFloat,
            TokenType::Identifier,
        ]) {
            return None;
        }

        let token = *self.current();

        Some(Box::new(Primitive::new(token)))
    }

    fn parse_access(&mut self) -> Option<Box<dyn Expr>> {
        let mut lhs = self.parse_primitive()?;

        while !self.at_eof() {
            if !self.match_here([TokenType::OpAccess]) {
                break;
            }

            self.consume_any();

            let rhs = self.parse_primitive()?;

            lhs = Box::new(Binary::new(lhs, rhs, OperatorTag::Access));
        }

        Some(lhs)
    }

    fn parse_call(&mut self) -> Option<Box<dyn Expr>> {
        let callee_expr = self.parse_access()?;

        if !self.match_here([TokenType::ParenOpen]) {
            return Some(callee_expr);
        }

        self.consume_any();

        let mut calling_args = Vec::<Box<dyn Expr>>::new();

        if self.match_here([TokenType::ParenClose]) {
            self.consume_any();
            return Some(Box::new(Call::new(callee_expr, calling_args)));
        }

        let first_arg = self.parse_compare()?;

        calling_args.push(first_arg);

        while !self.at_eof() {
            if self.match_here([TokenType::ParenClose]) {
                self.consume_any();
                break;
            }

            self.consume_of([TokenType::Comma]);

            let next_arg = self.parse_compare()?;

            calling_args.push(next_arg);
        }

        Some(Box::new(Call::new(callee_expr, calling_args)))
    }

    fn parse_unary(&mut self) -> Option<Box<dyn Expr>> {
        if !self.match_here([
            TokenType::OpMinus,
            TokenType::OpIncrement,
            TokenType::OpDecrement,
        ]) {
            return self.parse_call();
        }

        let current_tag = self.current().tag;
        let prefixed_op = match current_tag {
            TokenType::OpMinus => OperatorTag::Minus,
            TokenType::OpIncrement => OperatorTag::Increment,
            _ => OperatorTag::Decrement,
        };

        self.consume_any();

        let temp_inner = self.parse_call()?;

        Some(Box::new(Unary::new(temp_inner, prefixed_op)))
    }

    fn parse_factor(&mut self) -> Option<Box<dyn Expr>> {
        let mut lhs = self.parse_unary()?;

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

            self.consume_any();

            let rhs = self.parse_unary()?;

            lhs = Box::new(Binary::new(lhs, rhs, temp_op));
        }

        Some(lhs)
    }

    fn parse_term(&mut self) -> Option<Box<dyn Expr>> {
        let mut lhs = self.parse_factor()?;

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

            self.consume_any();

            let rhs = self.parse_factor()?;

            lhs = Box::new(Binary::new(lhs, rhs, temp_op));
        }

        Some(lhs)
    }

    fn parse_equality(&mut self) -> Option<Box<dyn Expr>> {
        let mut lhs = self.parse_term()?;

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

            self.consume_any();

            let rhs = self.parse_term()?;

            lhs = Box::new(Binary::new(lhs, rhs, temp_op));
        }

        Some(lhs)
    }

    fn parse_compare(&mut self) -> Option<Box<dyn Expr>> {
        let mut lhs = self.parse_equality()?;

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

            self.consume_any();

            let rhs = self.parse_equality()?;

            lhs = Box::new(Binary::new(lhs, rhs, temp_op));
        }

        Some(lhs)
    }

    fn parse_assign(&mut self) -> Option<Box<dyn Expr>> {
        let lhs = self.parse_access()?;

        if !self.match_here([TokenType::OpAssign]) {
            return Some(lhs);
        }

        self.consume_any();

        let rhs = self.parse_compare()?;

        Some(Box::new(Binary::new(lhs, rhs, OperatorTag::Assign)))
    }

    fn parse_variable_decl(&mut self) -> Option<Box<dyn Stmt>> {
        self.consume_of([TokenType::Keyword]);

        let var_name = *self.current();

        self.consume_of([TokenType::Identifier]);
        self.consume_of([TokenType::Colon]);

        let var_type_box = self.parse_type();

        self.consume_of([TokenType::OpAssign]);

        let var_init_expr = self.parse_compare()?;

        self.consume_of([TokenType::Semicolon]);

        Some(Box::new(VariableDecl::new(
            var_name,
            var_type_box,
            var_init_expr,
        )))
    }

    fn parse_if(&mut self) -> Option<Box<dyn Stmt>> {
        self.consume_any();

        let conds_expr = self.parse_compare()?;

        let truthy_body = self.parse_block()?;

        if self
            .current()
            .to_lexeme_str(self.tokenizer.view_source())
            .expect("")
            == "else"
        {
            self.consume_any();

            let falsy_body = self.parse_block()?;

            return Some(Box::new(If::new(truthy_body, falsy_body, conds_expr)));
        }

        let dud_falsy_body = Box::new(Block::new(Vec::new()));

        Some(Box::new(If::new(truthy_body, dud_falsy_body, conds_expr)))
    }

    fn parse_return(&mut self) -> Option<Box<dyn Stmt>> {
        self.consume_any();

        let result_expr = self.parse_compare()?;

        self.consume_of([TokenType::Semicolon]);

        Some(Box::new(Return::new(result_expr)))
    }

    fn parse_expr_stmt(&mut self) -> Option<Box<dyn Stmt>> {
        let inner_expr = self.parse_assign()?;

        self.consume_of([TokenType::Semicolon]);

        Some(Box::new(ExprStmt::new(inner_expr)))
    }

    fn parse_nestable(&mut self) -> Option<Box<dyn Stmt>> {
        let keyword = self
            .current()
            .to_lexeme_str(self.tokenizer.view_source())
            .expect("");

        match keyword {
            "let" => self.parse_variable_decl(),
            "if" => self.parse_if(),
            "return" => self.parse_return(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_block(&mut self) -> Option<Box<dyn Stmt>> {
        self.consume_of([TokenType::BraceOpen]);

        let mut items = Vec::<Box<dyn Stmt>>::new();

        while !self.at_eof() {
            if self.match_here([TokenType::BraceClose]) {
                self.consume_any();
                break;
            }

            let next_stmt = self.parse_nestable()?;

            items.push(next_stmt);
        }

        Some(Box::new(Block::new(items)))
    }

    fn parse_function_decl(&mut self) -> Option<Box<dyn Stmt>> {
        self.consume_of([TokenType::Keyword]);

        let func_name_token = *self.current();
        self.consume_of([TokenType::Identifier]);

        let func_params = self.parse_function_params();

        self.consume_of([TokenType::Colon]);
        let func_type_box = self.parse_type();

        let func_body = self.parse_block()?;

        Some(Box::new(FunctionDecl::new(
            func_name_token,
            func_params,
            func_type_box,
            func_body,
        )))
    }

    fn parse_param_decl(&mut self) -> ParamDecl {
        let name_token = *self.current();
        self.consume_of([TokenType::Identifier]);
        self.consume_of([TokenType::Colon]);

        let typing_box = self.parse_type();

        ParamDecl::new(name_token, typing_box)
    }

    fn parse_function_params(&mut self) -> Vec<ParamDecl> {
        self.consume_of([TokenType::ParenOpen]);

        let mut parameters = Vec::<ParamDecl>::new();

        if self.match_here([TokenType::ParenClose]) {
            self.consume_any();
            return parameters;
        }

        parameters.push(self.parse_param_decl());

        while !self.at_eof() {
            if self.match_here([TokenType::ParenClose]) {
                self.consume_any();
                break;
            }

            self.consume_of([TokenType::Comma]);

            parameters.push(self.parse_param_decl());
        }

        parameters
    }

    // TODO: implement this method.
    pub fn parse_file(&mut self) -> ParseResult {
        self.consume_any();

        let mut all_top_stmts = ASTDecls::new();

        while !self.at_eof() {
            let func_decl = self.parse_function_decl()?;

            all_top_stmts.push(func_decl);
        }

        if self.error_count != 0 {
            return None;
        }
        Some(all_top_stmts)
    }
}
