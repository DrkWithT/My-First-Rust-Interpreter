# Grammar Rules

### All Rules (pseudo-ABNF)
```
; other
<typename> ::= "bool" | "int" | "float" | "char" | "varchar" | <fun-type> | <array-type>
<fun-type> ::= <typename> (, <typename>)* -> <typename>
<array-type> ::= "[" <int> : <typename> "]"
<comment> ::= "#" ...

; exprs
<primitive> ::= <boolean> | <int> | <float> | <char> | <identifier> | (<compare>)
<char> ::= "\'" <NON-SINGLE-QUOTE> "\'"
<atom> ::= <primitive> | <varchar> | <array> | <lambda>
<varchar> ::= "\"" <NON-QUOTE>* "\""
<array> ::= "[" ( <compare> ( "," <compare> )* )? "]"
<lambda> ::= "fun" <params> ":" <typename> <block>
<access> ::= <atom> ("." <atom>)*
<call> ::= <access> ( ( <compare> (, <compare>)* )? )?
<unary> ::= <negate>
<negate> ::= "-"? <call>
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
<import> ::= "import" <identifier> ";"
<native-decl> ::= <native-stub> | <native-class>
<native-stub> ::= "foreign" <identifier> <params> ":" <typename> ";"
<function-decl> ::= "fun" <identifier> <params> ":" <typename> <block>
<constructor-decl> ::= "ctor" <params> <block>
<class-decl> ::= "class" <identifier> <class-body>
<class-body> ::= "{" <member-decl>+ "}"
<member-decl> ::= ( "private" | "public" ) (<variable-decl> | <function-decl> | <constructor-decl>)
<top-decl> ::= <import> | <native-stub> | <function-decl> | <class-decl>
<params> ::= "(" (<param-decl> ("," <param-decl>)* )? ")"
<param-decl> ::= <identifier> ":" <typename>
<program> ::= <top-decl>*
```
