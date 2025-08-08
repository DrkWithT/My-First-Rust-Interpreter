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
<typename> ::= "bool" | "int" | "float" | <fun-type> | <array-type>
<fun-type> ::= <typename> (, <typename>)* -> <typename>
<array-type> ::= "[" <int> : <typename> "]"
<comment> ::= "#" ...

; exprs (TODO: fuse atom to primitive)
<atom> ::= <primitive> | <lambda> | <array>
<primitive> ::= <boolean> | <int> | <float> | <identifier> | (<compare>)
<array> ::= "[" ( <compare> ( "," <compare> )* )? "]"
<lambda> ::= "fun" <params> ":" <typename> <block>
<access> ::= <atom> ("." <atom>)*
<call> ::= <access> ( ( <compare> (, <compare>)* )? )?
<unary> ::= <negate> | <increment> | <decrement>
<negate> ::= "-"? <call>
<increment> ::= "++"? <call>
<decrement> ::= "--"? <call>
<factor> ::= <unary> (("*" | "/") <unary>)*
<term> ::= <factor> (("+" | "-") <factor>)*
<equality> ::= <term> (("==" | "!=") <term>)*
<compare> ::= <equality> (("<" | ">") <equality>)*
<assign> ::= <access> ("=" <compare>)?

; statements
<variable-decl> ::= "let" <identifier> ":" <typename> "=" <compare> ";"
<if> ::= "if" <compare> <block> (<else>)?
<else> ::= "else" <block>
<return> ::= "return" <compare> ";"
<expr-stmt> ::= <assign> ";"
<nestable> ::= <variable-decl> | <if> | <return> | <expr-stmt>
<block> ::= { <nestable>* }
<function-decl> ::= "fun" <identifier> <params> : <typename> <block>
<params> ::= "(" (<param-decl> ("," <param-decl>)* )? ")"
<param-decl> ::= <identifier> ":" <typename>
<program> ::= <function-decl>*
```

### Roadmap
 - Support while loops (**PLANNED**)
 - Add native functions (**PLANNED**)
 - Add simple checks for types and declarations
 - Add optimization passes on IR
    - Instruction substitutions
    - Dead code elimination
 - Support arrays
 - Support strings
 - Support lambdas

### Other Docs
 - [Runtime Info](./docs/Runtime.md)
 - [Progress Images](./docs/Progress.md)
