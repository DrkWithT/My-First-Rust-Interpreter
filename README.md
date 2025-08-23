# README

### Brief
My second ever project in Rust. This will become a mini-language called Loxie as a very small derivative of the educational Lox language.

### What Loxie Looks Like
<img src="./docs/assets/Loxie_Lang_Highlight_1.png" width="50%" alt="fibonacci program">

### Design
 - Procedural & functional paradigm
 - Static & strong typing + null safety
 - Declared procedures & classes
 - Built-in functions galore!
 - NOTE: the compiler only reads sources relative to the `loxie_lib` directory or the invoked-from directory for now.

### Feature Roadmap
 - Support simple classes e.g `String`. (0.3.0)
   - Support intrinsic operations on `varchar` for now: `intrin_varchar_len` `intrin_varchar_get`, `intrin_varchar_set`, `intrin_varchar_push`, `intrin_varchar_pop`. **(WIP)**
   - Add parsing support for class syntax.
   - Add support for semantic analysis of class.
   - Add support in codegen for class... use `MAKE_OBJECT` instruction for heap values in instances pushed to the stack... **(WIP)**
      - Support member and by-index access.
      - Classes resolve to a compile-time-known mapping of slots (indices) to corresponding fields for field member accesses. At runtime, all instance objects per class will become compact clusters of values on the stack.
      - Later add `MAKE_REF <slot-id>` to push references to function-local class instances.
      - Later add `RETURN_MULTI <instance-base-slot-id> <const-n>` to support returning classes.
 - Support flexible arrays. (0.4.0)
 - Improve error diagnostics (0.4.1)
 - Add optimization passes on IR (0.4.2)
    - Instruction substitutions
 - Add more standard I/O native functions! (0.4.3)
 - Improve syntax highlighting on Loxie's local VSCode extension

### Other Docs
 - [Grammar Info](./docs/Grammar.md)
 - [Runtime Info](./docs/Runtime.md)
 - [Progress Images](./docs/Progress.md)
