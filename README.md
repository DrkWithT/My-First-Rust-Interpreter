# README

### Brief
My second ever project in Rust. This will become a mini-language called Loxie as a very small derivative of the educational Lox language.

### Design
 - Procedural & functional paradigm
 - Static & strong typing + null safety
 - First-class lambdas vs. declared procedures
 - Built-in functions galore!

### Grammar
```
; other
<typename> ::= "bool" | "int" | "float" | <fun-type>
<fun-type> ::= <typename> -> <typename>
<comment> ::= "#" ...

; exprs
<atom> ::= <boolean> | <int> | <float> | <identifier> | <lambda> | (<compare>)
<lambda> ::= "fun" <params> ":" <typename> <block>
<access> ::= <atom> ("." <atom>)* | <call>
<call> ::= <atom> ( ( <compare> (, <compare>)* )? )
<unary> ::= <negate> | <increment> | <decrement>
<negate> ::= "-" <access>
<increment> ::= "++" <access>
<decrement> ::= "--" <access>
<factor> ::= <unary> (("*" | "/") <unary>)*
<term> ::= <factor> (("+" | "-") <factor>)*
<equality> ::= <term> (("==" | "!=") <term>)*
<compare> ::= <equality> (("<" | ">") <equality>)*
<assign> ::= <access> "=" <compare>

; statements
<program> ::= <function-decl>*
<function-decl> ::= "fun" <identifier> <params> : <typename> <block>
<params> ::= "(" (<param-decl> ("," <param-decl>)* )? ")"
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
