# Runtime Notes

### Design
  - Pseudo-stack-based VM!
  - Bytecode is stored in chunks per function & constants are stored per chunk as a vector.
  - The call stack contains call frames which track:
    - Caller's return address
    - Caller' old `RBP`
    - Optional heap ID for the callee if a method is calling
  - The value stack contains data.
  - Variables become stack values offset from a base position from a call frame.
  - WIP: GC or ref-counting for chunky objects
    - A fix for missing reference count updates is pending.

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
```

### Sample Object Method Table (class method maps are compile time!)
- NOTE: These tables map method IDs to offsets of top-level generated functions in the `Program`. Constructors should be first in the method table.
- NOTE: the `INSTANCE_CALL` instruction will dispatch the corresponding routine to the method while the instance reference is set to the call frame by the VM, allowing valid field accesses.
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
 - `make_heap_object <member-count>`: heap allocates a class instance of `member-count` members and places its reference on the stack.
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
 - `leave`: returns control from a constructor like a normal `return` yet pushes the instance reference to the stack.
 - `call <function-id> <argc>`: Saves the caller return address of the current call frame before setting `RBP = RSP - ARGC + 1` to treat arguments as in-place locals.
 - `instance_call <object-ref-slot> <actual-function-id> <argc>`: Similar to a normal `call` yet places the object's heap ID into the next call frame _rather_ than a stack slot!
 - `native_call <native-function-id>`

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
 - `instance_call <object-ref-slot> <actual-function-id> <argc>`
 - `native_call <native-function-id>`
