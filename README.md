# README

### Brief
My second ever project in Rust. This will become a mini-language called Loxie as a very small derivative of the educational Lox language.

### A peek at Loxie
<img src="./docs/assets/Loxie_Lang_Highlight_1.png" width="50%" alt="fibonacci program">

### Design
 - Procedural & functional paradigm
 - Static & strong typing + null safety
 - Declared procedures & classes
 - Built-in functions galore!
 - NOTE: the compiler only reads sources relative to the `loxie_lib` directory or the invoked-from directory for now.

### Feature Roadmap
 - Support simple classes e.g `String`. (0.3.0)
   - Add support for semantic analysis of class.
      - Complete methods for class visitations.
      - Refactor name lookups to handle ctor calls, class fields, and method calls if needed.
   - Add support in codegen for class... use `MAKE_OBJECT` instruction for heap values in instances pushed to the stack... **(WIP)**
      - Support member and by-index access.
      - Classes resolve to a compile-time-known mapping of slots (indices) to corresponding fields for field member accesses. At runtime, all instance objects per class will become compact clusters of values on the stack... Constructors will always emit all member-`Value` pushes before their body generation.
      - Later add `RETURN_MULTI <instance-base-slot-id> <value-count>` to support returning classes.
 - Support flexible arrays. (0.4.0)
 - Improve error diagnostics (0.4.1)
 - Add optimization passes on IR (0.4.2)
    - Instruction substitutions
 - Add more standard I/O native functions! (0.4.3)
 - Improve syntax highlighting on Loxie's local VSCode extension:
   - Add these keywords: `ctor`, `class`, `private`, `public`, `met`
   - Add these typenames: `char`, `varchar`

### Other Docs
 - [Grammar Info](./docs/Grammar.md)
 - [Runtime Info](./docs/Runtime.md)
 - [Progress Images](./docs/Progress.md)
