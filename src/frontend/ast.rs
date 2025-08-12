use crate::codegen::ir::Locator;
use crate::frontend::token::{Token, TokenType};
use crate::semantics::types;
use crate::semantics::scope::SemanticNote;

pub struct ParamDecl {
    name_token: Token,
    typing: Box<dyn types::TypeKind>,
}

impl ParamDecl {
    pub fn new(name_token: Token, typing: Box<dyn types::TypeKind>) -> Self {
        Self { name_token, typing }
    }

    pub fn get_name_token(&self) -> &Token {
        &self.name_token
    }

    pub fn get_typing(&self) -> &dyn types::TypeKind {
        &*self.typing
    }
}

pub trait ExprVisitor<'evl, Res> {
    fn visit_primitive(&mut self, e: &Primitive) -> Res;
    fn visit_call(&mut self, e: &Call) -> Res;
    // fn visit_array(&self) -> Res;
    // fn visit_lambda(&self) -> Res;
    fn visit_unary(&mut self, e: &Unary) -> Res;
    fn visit_binary(&mut self, e: &Binary) -> Res;
}

pub trait Expr {
    fn get_operator(&self) -> types::OperatorTag;
    fn get_token_opt(&self) -> Option<Token>;
    fn try_deduce_type(&self) -> Box<dyn types::TypeKind>;
    fn accept_visitor(&self, visitor: &mut dyn ExprVisitor<Option<Locator>>) -> Option<Locator>;
    fn accept_visitor_sema(&self, visitor: &mut dyn ExprVisitor<SemanticNote>) -> SemanticNote;
}

pub struct Primitive {
    token: Token,
}

impl Primitive {
    pub fn new(token: Token) -> Self {
        Self { token }
    }

    pub fn get_token(&self) -> &Token {
        &self.token
    }
}

impl Expr for Primitive {
    fn get_operator(&self) -> types::OperatorTag {
        types::OperatorTag::Noop.clone()
    }

    fn get_token_opt(&self) -> Option<Token> {
        Some(*self.get_token())
    }

    fn try_deduce_type(&self) -> Box<dyn types::TypeKind> {
        let temp_token_tag = self.token.tag;

        let deduced_type_tag = match temp_token_tag {
            TokenType::LiteralBool => types::PrimitiveTag::Boolean,
            TokenType::LiteralInt => types::PrimitiveTag::Integer,
            TokenType::LiteralFloat => types::PrimitiveTag::Floating,
            _ => types::PrimitiveTag::Unknown,
        };

        Box::new(types::PrimitiveInfo::new(deduced_type_tag))
    }

    fn accept_visitor(&self, visitor: &mut dyn ExprVisitor<Option<Locator>>) -> Option<Locator> {
        visitor.visit_primitive(self)
    }

    fn accept_visitor_sema(&self, visitor: &mut dyn ExprVisitor<SemanticNote>) -> SemanticNote {
        visitor.visit_primitive(self)
    }
}

pub struct Call {
    callee: Box<dyn Expr>,
    args: Vec<Box<dyn Expr>>,
}

impl Call {
    pub fn new(callee: Box<dyn Expr>, args: Vec<Box<dyn Expr>>) -> Self {
        Self { callee, args }
    }

    pub fn get_callee(&self) -> &dyn Expr {
        &*self.callee
    }

    pub fn get_args(&self) -> &Vec<Box<dyn Expr>> {
        &self.args
    }
}

impl Expr for Call {
    fn get_operator(&self) -> types::OperatorTag {
        types::OperatorTag::Call
    }

    fn get_token_opt(&self) -> Option<Token> {
        None
    }

    fn try_deduce_type(&self) -> Box<dyn types::TypeKind> {
        Box::new(types::PrimitiveInfo::new(types::PrimitiveTag::Unknown))
    }

    fn accept_visitor(&self, visitor: &mut dyn ExprVisitor<Option<Locator>>) -> Option<Locator> {
        visitor.visit_call(self)
    }

    fn accept_visitor_sema(&self, visitor: &mut dyn ExprVisitor<SemanticNote>) -> SemanticNote {
        visitor.visit_call(self)
    }
}

// pub struct Array {
//     // todo
// }

// pub struct Lambda {
//     // todo
// }

pub struct Unary {
    inner: Box<dyn Expr>,
    op_tag: types::OperatorTag,
}

impl Unary {
    pub fn new(inner: Box<dyn Expr>, op_tag: types::OperatorTag) -> Self {
        Self { inner, op_tag }
    }

    pub fn get_inner(&self) -> &dyn Expr {
        &*self.inner
    }

    pub fn get_op_tag(&self) -> &types::OperatorTag {
        &self.op_tag
    }
}

impl Expr for Unary {
    fn get_operator(&self) -> types::OperatorTag {
        self.op_tag.clone()
    }

    fn get_token_opt(&self) -> Option<Token> {
        None
    }

    fn try_deduce_type(&self) -> Box<dyn types::TypeKind> {
        let temp_op = self.get_operator();
        let inner_type_box = self.get_inner().try_deduce_type();
        let inner_is_primitive = inner_type_box.as_ref().is_primitive();

        if inner_is_primitive {
            return match temp_op {
                types::OperatorTag::Minus => inner_type_box,
                types::OperatorTag::Increment => inner_type_box,
                types::OperatorTag::Decrement => inner_type_box,
                _ => Box::new(types::PrimitiveInfo::new(types::PrimitiveTag::Unknown)),
            };
        }

        Box::new(types::PrimitiveInfo::new(types::PrimitiveTag::Unknown))
    }

    fn accept_visitor(&self, visitor: &mut dyn ExprVisitor<Option<Locator>>) -> Option<Locator> {
        visitor.visit_unary(self)
    }

    fn accept_visitor_sema(&self, visitor: &mut dyn ExprVisitor<SemanticNote>) -> SemanticNote {
        visitor.visit_unary(self)
    }
}

pub struct Binary {
    pub lhs: Box<dyn Expr>,
    pub rhs: Box<dyn Expr>,
    pub op_tag: types::OperatorTag,
}

impl Binary {
    pub fn new(lhs: Box<dyn Expr>, rhs: Box<dyn Expr>, op_tag: types::OperatorTag) -> Self {
        Self { lhs, rhs, op_tag }
    }

    pub fn get_lhs(&self) -> &dyn Expr {
        &*self.lhs
    }

    pub fn get_rhs(&self) -> &dyn Expr {
        &*self.rhs
    }
}

impl Expr for Binary {
    fn get_operator(&self) -> types::OperatorTag {
        self.op_tag.clone()
    }

    fn get_token_opt(&self) -> Option<Token> {
        None
    }

    /// NOTE: self-deduction of any binary expr here is too complex, so I leave that for the semantic checker...
    fn try_deduce_type(&self) -> Box<dyn types::TypeKind> {
        Box::new(types::PrimitiveInfo::new(types::PrimitiveTag::Unknown))
    }

    fn accept_visitor(&self, visitor: &mut dyn ExprVisitor<Option<Locator>>) -> Option<Locator> {
        visitor.visit_binary(self)
    }

    fn accept_visitor_sema(&self, visitor: &mut dyn ExprVisitor<SemanticNote>) -> SemanticNote {
        visitor.visit_binary(self)
    }
}

pub trait StmtVisitor<Res> {
    fn visit_function_decl(&mut self, s: &FunctionDecl) -> Res;
    fn visit_block(&mut self, s: &Block) -> Res;
    fn visit_variable_decl(&mut self, s: &VariableDecl) -> Res;
    fn visit_if(&mut self, s: &If) -> Res;
    fn visit_while(&mut self, s: &While) -> Res;
    fn visit_return(&mut self, s: &Return) -> Res;
    fn visit_expr_stmt(&mut self, s: &ExprStmt) -> Res;
}

pub trait Stmt {
    fn is_directive(&self) -> bool;
    fn is_declaration(&self) -> bool;
    fn is_expr_stmt(&self) -> bool;
    fn accept_visitor(&self, v: &mut dyn StmtVisitor<bool>) -> bool;
}

pub struct FunctionDecl {
    name_token: Token,
    params: Vec<ParamDecl>,
    result_typing: Box<dyn types::TypeKind>,
    body: Box<dyn Stmt>,
}

impl FunctionDecl {
    pub fn new(
        name_token: Token,
        params: Vec<ParamDecl>,
        result_typing: Box<dyn types::TypeKind>,
        body: Box<dyn Stmt>,
    ) -> Self {
        Self {
            name_token,
            params,
            result_typing,
            body,
        }
    }

    pub fn get_name_token(&self) -> &Token {
        &self.name_token
    }

    pub fn get_params(&self) -> &Vec<ParamDecl> {
        &self.params
    }

    pub fn get_result_type(&self) -> &dyn types::TypeKind {
        &*self.result_typing
    }

    pub fn get_body(&self) -> &dyn Stmt {
        &*self.body
    }
}

impl Stmt for FunctionDecl {
    fn is_directive(&self) -> bool {
        false
    }

    fn is_declaration(&self) -> bool {
        true
    }

    fn is_expr_stmt(&self) -> bool {
        false
    }

    fn accept_visitor(&self, v: &mut dyn StmtVisitor<bool>) -> bool {
        v.visit_function_decl(self)
    }
}

pub struct Block {
    items: Vec<Box<dyn Stmt>>,
}

impl Block {
    pub fn new(items: Vec<Box<dyn Stmt>>) -> Self {
        Self { items }
    }

    pub fn get_items(&self) -> &Vec<Box<dyn Stmt>> {
        &self.items
    }
}

impl Stmt for Block {
    fn is_directive(&self) -> bool {
        false
    }

    fn is_declaration(&self) -> bool {
        false
    }

    fn is_expr_stmt(&self) -> bool {
        false
    }

    fn accept_visitor(&self, v: &mut dyn StmtVisitor<bool>) -> bool {
        v.visit_block(self)
    }
}

pub struct VariableDecl {
    name_token: Token,
    typing: Box<dyn types::TypeKind>,
    init_expr: Box<dyn Expr>,
}

impl VariableDecl {
    pub fn new(
        name_token: Token,
        typing: Box<dyn types::TypeKind>,
        init_expr: Box<dyn Expr>,
    ) -> Self {
        Self {
            name_token,
            typing,
            init_expr,
        }
    }

    pub fn get_name_token(&self) -> &Token {
        &self.name_token
    }

    pub fn get_typing(&self) -> &dyn types::TypeKind {
        &*self.typing
    }

    pub fn get_init_expr(&self) -> &dyn Expr {
        &*self.init_expr
    }
}

impl Stmt for VariableDecl {
    fn is_directive(&self) -> bool {
        false
    }

    fn is_declaration(&self) -> bool {
        true
    }

    fn is_expr_stmt(&self) -> bool {
        false
    }

    fn accept_visitor(&self, v: &mut dyn StmtVisitor<bool>) -> bool {
        v.visit_variable_decl(self)
    }
}

pub struct If {
    truthy: Box<dyn Stmt>,
    falsy: Box<dyn Stmt>,
    check: Box<dyn Expr>,
}

impl If {
    pub fn new(truthy: Box<dyn Stmt>, falsy: Box<dyn Stmt>, check: Box<dyn Expr>) -> Self {
        Self {
            truthy,
            falsy,
            check,
        }
    }

    pub fn get_truthy_body(&self) -> &dyn Stmt {
        &*self.truthy
    }

    pub fn get_falsy_body(&self) -> &dyn Stmt {
        &*self.falsy
    }

    pub fn get_check(&self) -> &dyn Expr {
        &*self.check
    }
}

impl Stmt for If {
    fn is_directive(&self) -> bool {
        false
    }

    fn is_declaration(&self) -> bool {
        true
    }

    fn is_expr_stmt(&self) -> bool {
        false
    }

    fn accept_visitor(&self, v: &mut dyn StmtVisitor<bool>) -> bool {
        v.visit_if(self)
    }
}

pub struct While {
    check: Box<dyn Expr>,
    body: Box<dyn Stmt>,
}

impl While {
    pub fn new(check_arg: Box<dyn Expr>, body_arg: Box<dyn Stmt>) -> Self {
        Self {
            check: check_arg,
            body: body_arg,
        }
    }

    pub fn get_check(&self) -> &dyn Expr {
        &*self.check
    }

    pub fn get_body(&self) -> &dyn Stmt {
        &*self.body
    }
}

impl Stmt for While {
    fn is_directive(&self) -> bool {
        false
    }

    fn is_declaration(&self) -> bool {
        false
    }

    fn is_expr_stmt(&self) -> bool {
        false
    }

    fn accept_visitor(&self, v: &mut dyn StmtVisitor<bool>) -> bool {
        v.visit_while(self)
    }
}

pub struct Return {
    result: Box<dyn Expr>,
}

impl Return {
    pub fn new(result: Box<dyn Expr>) -> Self {
        Self { result }
    }

    pub fn get_result(&self) -> &dyn Expr {
        &*self.result
    }
}

impl Stmt for Return {
    fn is_directive(&self) -> bool {
        false
    }

    fn is_declaration(&self) -> bool {
        false
    }

    fn is_expr_stmt(&self) -> bool {
        false
    }

    fn accept_visitor(&self, v: &mut dyn StmtVisitor<bool>) -> bool {
        v.visit_return(self)
    }
}

pub struct ExprStmt {
    inner: Box<dyn Expr>,
}

impl ExprStmt {
    pub fn new(inner: Box<dyn Expr>) -> Self {
        Self { inner }
    }

    pub fn get_inner(&self) -> &dyn Expr {
        &*self.inner
    }
}

impl Stmt for ExprStmt {
    fn is_directive(&self) -> bool {
        false
    }

    fn is_declaration(&self) -> bool {
        false
    }

    fn is_expr_stmt(&self) -> bool {
        true
    }

    fn accept_visitor(&self, v: &mut dyn StmtVisitor<bool>) -> bool {
        v.visit_expr_stmt(self)
    }
}
