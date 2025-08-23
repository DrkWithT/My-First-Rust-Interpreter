#[repr(i8)]
#[derive(Clone, Copy, PartialEq)]
pub enum ExecStatus {
    Ok,
    AccessError,
    ValueError,
    RefError,
    BadMath,
    IllegalInstruction,
    BadArgs,
    NotOk,
}

/*
 * Declares a native function wrapper alias.
 * `E` is the generic parameter taking an interpreter type.
 * An `ExecStatus` value must be returned by all wrapped natives. Also, their registered arity (expected argument count) must be followed or else the VM will cause errorneous output.
 */
pub type Callable<E> = Box<dyn Fn(&mut E) -> ExecStatus>;
