# README

### Brief
My second project in Rust. This might become a mini-language.

### Grammar
```
; other
<typename> ::= "bool" | "int" | "float" | <fun-type>
<fun-type> ::= <typename> -> <typename>
<comment> ::= "#" ...

; exprs
<atom> ::= <boolean> | <int> | <float> | <identifier> | <lambda> | (<compare>)
<lambda> ::= "fun" <params> : <typename> <block>
<access> ::= <atom> ("." <atom>)*
<unary> ::= <negate> | <increment> | <decrement>
<factor> ::= <unary> (("*" | "/") <unary>)*
<term> ::= <factor> (("*" | "/") <factor>)*
<equality> ::= <term> (("==" | "!=") <term>)*
<compare> ::= <equality> (("<" | ">") <equality>)*
<assign> ::= <access> "=" <compare>

; statements
<program> ::= <function-decl>*
<function-decl> ::= "fun" <identifier> <params> : <typename> <block>
<params> ::= "(" <param-decl>* ")"
<param-decl> ::= <identifier> ":" <typename>
<block> ::= { <nestable>* }
<nestable> ::= <variable-decl> | <if> | <return> | <expr-stmt> | <exit>
<variable-decl> ::= "let" <identifier> ":" <typename> "=" <compare>
<if> ::= "if" <compare> <block> (<else>)?
<else> ::= "else" <block>
<return> ::= "return" <compare>
<expr-stmt> ::= <assign>
<exit> ::= "exit" <int>
```