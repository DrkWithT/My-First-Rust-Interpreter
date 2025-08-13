# Grammar Rules

### All Rules (pseudo-ABNF)
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
<while> ::= "while" <compare> <block>
<return> ::= "return" <compare> ";"
<expr-stmt> ::= <assign> ";"
<nestable> ::= <variable-decl> | <if> | <return> | <expr-stmt> | <while>
<block> ::= { <nestable>* }
<native-decl> ::= "foreign" <identifier> <params> ":" <typename> ";"
<function-decl> ::= "fun" <identifier> <params> ":" <typename> <block>
<decl> ::= <native-decl> | <function-decl>
<params> ::= "(" (<param-decl> ("," <param-decl>)* )? ")"
<param-decl> ::= <identifier> ":" <typename>
<program> ::= <decl>*
```
