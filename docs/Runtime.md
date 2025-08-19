# Runtime Notes

### Design
  - Pseudo-stack-based VM!
  - Bytecode is stored in chunks per function & constants are stored per chunk as a vector.
  - The call stack contains call frames which track:
    - Caller's return address
    - Callee arguments
    - Caller's old rbp
  - The value stack contains data.
  - Variables become stack values offset from a base position from a call frame.
  - Planned: GC or ref-counting for chunky objects

### Sample Diagram (stack values)
```
| Value(42) | (call_frame.base_pos + 1)
| Value(24) | (call_frame.base_pos)
| ..other.. |
```

### Sample Object Layout (class instance on stack)
```
| Value(varchar{""}) | (self<String>.data = "" --> self_stack_pos + 0)
| Value(0)           | (self<String>.length = 0 --> self_stack_pos + 1)
| .................. |
```

### Sample Object Method Table (class methods table)
```
| ClassProcedure | (String::new(s: varchar) --> IDX 0)
| ClassProcedure | (String::len() --> IDX 1)
| ....others.... |
```
  - NOTE: Any reference to `self` of the current instance is just a calculated offset into the stack.

### IR Opcodes
 - `load_const <constant-id>`
 - `push <src-slot>`
 - `pop`
 - `replace <dest-slot> <src-slot>`
 - `append_item <dest-slot-object-ref>`
 - `neg <dest-slot>`
 - `inc <dest-slot>`
 - `dec <dest-slot>`
 - `add`
 - `sub`
 - `mul`
 - `div`
 - `gen_begin_loop`: marks the start of a while loop's steps (adds a NOP)
 - `gen_patch`: marks the previous IR step as the "patch" of a forward jump
 - `gen_patch_back`: marks the previous while-loop starting step as the "patch" of a recent jump- results in a "backwards patch"
 - `compare_eq`
 - `compare_ne`
 - `compare_lt`
 - `compare_gt`
 - `jump_if <src-id> <dest?>`
 - `jump_else <src-id> <dest?>`
 - `jump <dest?>`
 - `return <src-id>`
 - `call <function-id> <argc>`
 - `native_call <native-function-id>`

### Opcodes
 - `load_const <constant-id>`
 - `pop`
 - `replace <dest-slot> <src-loc>`
 - `append_item <dest-slot-object-ref>`
 - `neg <dest-slot>`
 - `inc <dest-slot>`
 - `dec <dest-slot>`
 - `add`
 - `sub`
 - `mul`
 - `div`
 - `compare_eq`
 - `compare_ne`
 - `compare_lt`
 - `compare_gt`
 - `jump_if <src-slot> <new-ip>`
 - `jump_else <src-slot> <new-ip>`
 - `jump <new-ip>`
 - `return <src-slot>`
 - `call <function-id> <argc>`
 - `native_call <native-function-id>`
