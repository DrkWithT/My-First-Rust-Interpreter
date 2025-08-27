# Runtime Notes

### Design
  - Pseudo-stack-based VM!
  - Bytecode is stored in chunks per function & constants are stored per chunk as a vector.
  - The call stack contains call frames which track:
    - Caller's return address
    - Callee arguments
    - Caller's old rbp
    - Optional "self" reference to the callee's object.
  - The value stack contains data.
  - Variables become stack values offset from a base position from a call frame.
  - Planned: GC or ref-counting for chunky objects

### Sample Diagram (stack values)
```
| Value(42) | (call_frame.base_pos + 1)
| Value(24) | (call_frame.base_pos)
| .(other). |
```

### Sample Object Layout (class instance on heap)
- NOTE: any referenced class member maps to some index into a class object's member table.
```
0: | Value(0)           | (self<Stack<100>>.sp --> table_0.members + 0)
1: | Value([int, 100]{})| (self<Stack<100>>.data --> table_0.members + 1)
n: | .................. |
   ----------------------
.. | (method table 0x0) | <-- (class method table's ID)
```

### Sample Object Method Table (class methods table)
- NOTE: These tables map method IDs to offsets of top-level generated functions in the `Program`. Constructors are always first in the method table.
- NOTE: the `CALL_METHOD <method-id> <arity-n>` instruction will dispatch through the table by the call frame reference's table ID.
```
.. | MetTable_x0000 |
0: | ClassProcedure | (Stack<int>(capacity: int) --> IDX A)
1: | ClassProcedure | (Stack<int>.top() --> IDX B)
.. | (more methods) |
```

### IR Opcodes
 - `load_const <constant-id>`
 - `push <arg>`
 - `pop`
 - `make_heap_value <kind-tag>`: heap allocates a heap typed value and pushes its reference onto the stack
 - `make_heap_object <member-count> <method-table-id>`: heap allocates a class instance and places its reference on the stack.
 - `replace <dest-slot> <src-slot>`: can also emplace a fresh heap value to its corresponding heap cell.
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
 - `leave`: returns control from a special function (such as constructors that push in member-wise order) without an extra result value push
 - `call <function-id> <argc>`
 - `native_call <native-function-id>`
 - `method_call <method-table-id> <>`

### Opcodes
 - `load_const <constant-id>`
 - `push <arg>`
 - `pop`
 - `make_heap_value <kind-tag>`
 - `make_heap_object <member-count> <method-table-id>`
 - `replace <dest-slot> <src-loc>`
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
 - `leave`
 - `call <function-id> <argc>`
 - `native_call <native-function-id>`
