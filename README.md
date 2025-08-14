# README

### Brief
My second ever project in Rust. This will become a mini-language called Loxie as a very small derivative of the educational Lox language.

### What Loxie Looks Like
<img src="./docs/assets/Loxie_Lang_Highlight_1.png" width="50%" alt="disassembled bytecode of ifs case">

### Design
 - Procedural & functional paradigm
 - Static & strong typing + null safety
 - First-class lambdas vs. declared procedures
 - Built-in functions galore!
 - NOTE: the compiler only reads sources relative to the `loxie_lib` directory or the invoked-from directory for now.

### Feature Roadmap
 - Improve error diagnostics
 - Add optimization passes on IR
    - Instruction substitutions
 - Support strings
 - Improve syntax highlighting on Loxie's local VSCode extension
 - Support arrays
 - Support lambdas
 - Add more standard I/O native functions!

### Other Docs
 - [Grammar Info](./docs/Grammar.md)
 - [Runtime Info](./docs/Runtime.md)
 - [Progress Images](./docs/Progress.md)
