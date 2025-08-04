# Runtime Notes

### Design
  - Pseudo-stack-based VM!
  - Bytecode is stored in chunks per function & constants are stored per chunk as a vector.
  - The call stack contains call frames which track:
    - `saved_fn_id`: i32
    - `saved_fn_ip`: i32
    - `base_pos`: i32
  - The value stack contains data.
  - Variables become stack values offset from a base position from a call frame.
  - Planned: GC or ref-counting for chunky objects

### Sample Diagram (stack values)
```
| Value(42) | (call_frame.base_pos + 1)
| Value(24) | (call_frame.base_pos)
| ..other.. |
```

### IR Opcodes
 - `load_const <constant-id>`
 - `push <src-slot>`
 - `pop`
 - `replace <dest-slot> <src-slot>`
 - `neg <dest-slot>`
 - `inc <dest-slot>`
 - `dec <dest-slot>`
 - `add`
 - `sub`
 - `mul`
 - `div`
 - `begin_block`: begins a branched-to IR block
 - `end_block`: ends a branched-to IR block
 - `compare_eq`
 - `compare_ne`
 - `compare_lt`
 - `compare_gt`
 - `jump_if <src-id> <new-ip>`
 - `jump_else <src-id> <new-ip>`
 - `jump <new-ip>`
 - `return <src-id>`
 - `call <function-id>`

### Opcodes
 - `load_const <constant-id>`
 - `pop`
 - `replace <dest-slot> <src-slot>`
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
 - `call <function-id>`
